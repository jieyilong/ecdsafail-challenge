
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;
use crate::point_add::venting;

pub(crate) fn mod_add_double_qb(b: &mut B, acc: &[QubitId], bits: &[BitId], p: U256) {
    // acc := acc + 2*bits mod p. Reuse a single loaded copy of the classical
    // point and walk it through the cheap secp256k1 double/halve pair.
    let a = load_bits(b, bits);
    mod_double_inplace_fast(b, &a, p);
    mod_add_qq_fast(b, acc, &a, p);
    mod_halve_inplace_fast(b, &a, p);
    unload_bits(b, &a, bits);
}

pub(crate) fn mod_double_inplace_fast(b: &mut B, v: &[QubitId], p: U256) {
    mod_double_inplace_fast_with_dirty(b, v, p, None)
}

pub(crate) fn mod_double_inplace_fast_with_dirty(
    b: &mut B,
    v: &[QubitId],
    p: U256,
    dirty_src: Option<&[QubitId]>,
) {
    let n = v.len();
    let ovf = b.alloc_qubit();
    b.swap(v[n - 1], ovf);
    for i in (0..n - 1).rev() {
        b.swap(v[i], v[i + 1]);
    }
    debug_assert_eq!(n, 256);
    // For secp256k1, p = 2^n - c. After the shift, the old top bit is in
    // `ovf` and the low register holds T mod 2^n for T = 2*v. If ovf=1 then
    // T = 2^n + low and T mod p = low + c; otherwise T mod p = low.
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1));
    let use_venting = std::env::var("KAL_VENT_DOUBLE").ok().as_deref() == Some("1")
        && dirty_src.map_or(false, |d| d.len() >= n - 2);
    if let Some(w) = double_carry_trunc_window() {
        // Carry-tail-truncated sparse-constant add (default OFF).
        cadd_nbit_const_direct_trunc_fast(b, v, c, ovf, w);
    } else if use_venting {
        let dirty = dirty_src.unwrap();
        let q_clean2: [QubitId; 2] = [b.alloc_qubit(), b.alloc_qubit()];
        venting::ciadd_dirty_2clean_classical(
            b,
            v,
            &dirty[..n - 2],
            &q_clean2,
            c.as_limbs()[0],
            ovf,
            false,
        );
        b.free(q_clean2[0]);
        b.free(q_clean2[1]);
    } else if direct_const_walks_enabled()
        || std::env::var("KAL_DIRECT_CONST_DOUBLE").ok().as_deref() == Some("1")
    {
        cadd_nbit_const_direct_fast(b, v, c, ovf);
    } else {
        cadd_nbit_const_fast(b, v, c, ovf);
    }
    // Result parity equals the old top bit: even if ovf=0, odd if ovf=1.
    b.cx(v[0], ovf);
    b.free(ovf);
}

pub(crate) fn mod_double_inplace_direct_const_fast(b: &mut B, v: &[QubitId], p: U256) {
    let n = v.len();
    let ovf = b.alloc_qubit();
    b.swap(v[n - 1], ovf);
    for i in (0..n - 1).rev() {
        b.swap(v[i], v[i + 1]);
    }
    debug_assert_eq!(n, 256);
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1));
    cadd_nbit_const_direct_fast(b, v, c, ovf);
    b.cx(v[0], ovf);
    b.free(ovf);
}

/// `v := 2*v` assuming v[n-1] = 0 (no wrap). Just a shift-left cascade.
/// 0 Toffoli. Used in Kaliski STEP 7+8 for small iters where r[255]=0 guaranteed.
pub(crate) fn mod_double_no_corr(b: &mut B, v: &[QubitId]) {
    let n = v.len();
    for i in (0..n - 1).rev() {
        b.swap(v[i], v[i + 1]);
    }
}

pub(crate) fn mod_shift_left_by_k(
    b: &mut B,
    v: &[QubitId],
    p: U256,
    k: usize,
) -> (Vec<QubitId>, QubitId, QubitId) {
    let n = v.len();
    debug_assert_eq!(n, 256);
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1));

    let spill = b.alloc_qubits(k);
    let ovf = b.alloc_qubit();
    let flag_inv = b.alloc_qubit();

    // Step 1: k rounds of shift-by-1, capturing top bits into spill.
    for shift_i in 0..k {
        b.swap(v[n - 1], spill[k - 1 - shift_i]);
        for i in (0..n - 1).rev() {
            b.swap(v[i], v[i + 1]);
        }
    }

    // Step 2: add spill · c to v_ext (using ovf as bit n).
    // c = 2^32 + 977 = 2^32 + 2^10 - 2^6 + 2^4 + 2^0.
    // Consolidate 4 bits (6,7,8,9) of 977 into 2^10 - 2^6: saves 2 Cuccaros per shift.
    // Op list: ADD at 0, 4, 10, 32; SUB at 6. Total 5 ops instead of 7.
    let mut v_ext = v.to_vec();
    v_ext.push(ovf);
    let cuccaro_op = |b: &mut B, pos: usize, is_sub: bool| {
        let pad_width = n + 1 - pos;
        let padded = b.alloc_qubits(pad_width);
        for i in 0..k.min(pad_width) {
            b.cx(spill[i], padded[i]);
        }
        let v_slice: Vec<QubitId> = v_ext[pos..n + 1].to_vec();
        let c_in = b.alloc_qubit();
        if lowq_shift22() {
            if is_sub {
                cuccaro_sub(b, &padded, &v_slice, c_in);
            } else {
                cuccaro_add(b, &padded, &v_slice, c_in);
            }
        } else if is_sub {
            // Fast cuccaro: saves ~n CCX per op. Peak during this op (~514
            // transient) is still below the mod_add_qq_fast peak (517) inside
            // the enclosing Solinas, so no global peak increase.
            cuccaro_sub_fast(b, &padded, &v_slice, c_in);
        } else {
            cuccaro_add_fast(b, &padded, &v_slice, c_in);
        }
        b.free(c_in);
        for i in 0..k.min(pad_width) {
            b.cx(spill[i], padded[i]);
        }
        b.free_vec(&padded);
    };
    b.set_phase("shift22_cuccaro_op_0");
    cuccaro_op(b, 0, false);
    b.set_phase("shift22_cuccaro_op_4");
    cuccaro_op(b, 4, false);
    b.set_phase("shift22_cuccaro_op_6");
    cuccaro_op(b, 6, true);
    b.set_phase("shift22_cuccaro_op_10");
    cuccaro_op(b, 10, false);
    b.set_phase("shift22_cuccaro_op_32");
    cuccaro_op(b, 32, false);

    // Step 3: const add.
    b.set_phase("shift22_step3");
    if lowq_shift22() {
        add_nbit_const(b, &v_ext, c);
    } else {
        add_nbit_const_fast(b, &v_ext, c);
    }
    b.x(ovf);
    b.cx(ovf, flag_inv); // flag_inv = NOT(top_bit_after_add) = (value < p)
    b.x(ovf);

    // Step 4: conditional const sub.
    b.set_phase("shift22_step4");
    if lowq_shift22() {
        csub_nbit_const(b, &v_ext, c, flag_inv);
    } else {
        csub_nbit_const_fast(b, &v_ext, c, flag_inv);
    }
    b.x(flag_inv);
    b.cx(flag_inv, ovf);
    b.x(flag_inv);

    (spill, flag_inv, ovf)
}

/// Gate-level inverse of mod_shift_left_by_k.
pub(crate) fn mod_shift_right_by_k(
    b: &mut B,
    v: &[QubitId],
    p: U256,
    k: usize,
    spill: Vec<QubitId>,
    flag_inv: QubitId,
    ovf: QubitId,
) {
    let n = v.len();
    debug_assert_eq!(n, 256);
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1));

    let mut v_ext = v.to_vec();
    v_ext.push(ovf);

    // Reverse step 4.
    b.x(flag_inv);
    b.cx(flag_inv, ovf);
    b.x(flag_inv);
    b.set_phase("rshift22_rev_step4");
    if lowq_shift22() {
        cadd_nbit_const(b, &v_ext, c, flag_inv);
    } else {
        cadd_nbit_const_fast(b, &v_ext, c, flag_inv);
    }

    // Reverse step 3.
    b.x(ovf);
    b.cx(ovf, flag_inv);
    b.x(ovf);
    b.set_phase("rshift22_rev_step3");
    if lowq_shift22() {
        sub_nbit_const(b, &v_ext, c);
    } else {
        sub_nbit_const_fast(b, &v_ext, c);
    }
    b.free(flag_inv);
    b.set_phase("rshift22_rev_step2");

    // Reverse step 2: inverse of the consolidated op list (5 ops, in reverse order, flipped signs).
    let cuccaro_op = |b: &mut B, pos: usize, is_sub: bool| {
        let pad_width = n + 1 - pos;
        let padded = b.alloc_qubits(pad_width);
        for i in 0..k.min(pad_width) {
            b.cx(spill[i], padded[i]);
        }
        let v_slice: Vec<QubitId> = v_ext[pos..n + 1].to_vec();
        let c_in = b.alloc_qubit();
        if lowq_shift22() {
            if is_sub {
                cuccaro_sub(b, &padded, &v_slice, c_in);
            } else {
                cuccaro_add(b, &padded, &v_slice, c_in);
            }
        } else if is_sub {
            cuccaro_sub_fast(b, &padded, &v_slice, c_in);
        } else {
            cuccaro_add_fast(b, &padded, &v_slice, c_in);
        }
        b.free(c_in);
        for i in 0..k.min(pad_width) {
            b.cx(spill[i], padded[i]);
        }
        b.free_vec(&padded);
    };
    // Reverse: undo ADD at 32, 10; undo SUB at 6; undo ADD at 4, 0.
    cuccaro_op(b, 32, true); // undo +spill·2^32
    cuccaro_op(b, 10, true); // undo +spill·2^10
    cuccaro_op(b, 6, false); // undo -spill·2^6
    cuccaro_op(b, 4, true); // undo +spill·2^4
    cuccaro_op(b, 0, true); // undo +spill·2^0

    // Reverse step 1: reverse swap cascades.
    for shift_i in (0..k).rev() {
        for i in 0..n - 1 {
            b.swap(v[i], v[i + 1]);
        }
        b.swap(v[n - 1], spill[k - 1 - shift_i]);
    }

    b.free(ovf);
    b.free_vec(&spill);
}

pub(crate) fn mod_shift_left_by_k_lowq(
    b: &mut B,
    v: &[QubitId],
    p: U256,
    k: usize,
) -> (Vec<QubitId>, QubitId, QubitId) {
    let n = v.len();
    debug_assert_eq!(n, 256);
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1));

    let spill = b.alloc_qubits(k);
    let ovf = b.alloc_qubit();
    let flag_inv = b.alloc_qubit();

    for shift_i in 0..k {
        b.swap(v[n - 1], spill[k - 1 - shift_i]);
        for i in (0..n - 1).rev() {
            b.swap(v[i], v[i + 1]);
        }
    }

    let mut v_ext = v.to_vec();
    v_ext.push(ovf);
    let cuccaro_op = |b: &mut B, pos: usize, is_sub: bool| {
        let pad_width = n + 1 - pos;
        let padded = b.alloc_qubits(pad_width);
        for i in 0..k.min(pad_width) {
            b.cx(spill[i], padded[i]);
        }
        let v_slice: Vec<QubitId> = v_ext[pos..n + 1].to_vec();
        let c_in = b.alloc_qubit();
        if is_sub {
            cuccaro_sub(b, &padded, &v_slice, c_in);
        } else {
            cuccaro_add(b, &padded, &v_slice, c_in);
        }
        b.free(c_in);
        for i in 0..k.min(pad_width) {
            b.cx(spill[i], padded[i]);
        }
        b.free_vec(&padded);
    };
    cuccaro_op(b, 0, false);
    cuccaro_op(b, 4, false);
    cuccaro_op(b, 6, true);
    cuccaro_op(b, 10, false);
    cuccaro_op(b, 32, false);

    add_nbit_const(b, &v_ext, c);
    b.x(ovf);
    b.cx(ovf, flag_inv);
    b.x(ovf);
    csub_nbit_const(b, &v_ext, c, flag_inv);
    b.x(flag_inv);
    b.cx(flag_inv, ovf);
    b.x(flag_inv);

    (spill, flag_inv, ovf)
}

pub(crate) fn mod_shift_right_by_k_lowq(
    b: &mut B,
    v: &[QubitId],
    p: U256,
    k: usize,
    spill: Vec<QubitId>,
    flag_inv: QubitId,
    ovf: QubitId,
) {
    let n = v.len();
    debug_assert_eq!(n, 256);
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1));

    let mut v_ext = v.to_vec();
    v_ext.push(ovf);

    b.x(flag_inv);
    b.cx(flag_inv, ovf);
    b.x(flag_inv);
    cadd_nbit_const(b, &v_ext, c, flag_inv);

    b.x(ovf);
    b.cx(ovf, flag_inv);
    b.x(ovf);
    sub_nbit_const(b, &v_ext, c);
    b.free(flag_inv);

    let cuccaro_op = |b: &mut B, pos: usize, is_sub: bool| {
        let pad_width = n + 1 - pos;
        let padded = b.alloc_qubits(pad_width);
        for i in 0..k.min(pad_width) {
            b.cx(spill[i], padded[i]);
        }
        let v_slice: Vec<QubitId> = v_ext[pos..n + 1].to_vec();
        let c_in = b.alloc_qubit();
        if is_sub {
            cuccaro_sub(b, &padded, &v_slice, c_in);
        } else {
            cuccaro_add(b, &padded, &v_slice, c_in);
        }
        b.free(c_in);
        for i in 0..k.min(pad_width) {
            b.cx(spill[i], padded[i]);
        }
        b.free_vec(&padded);
    };
    cuccaro_op(b, 32, true);
    cuccaro_op(b, 10, true);
    cuccaro_op(b, 6, false);
    cuccaro_op(b, 4, true);
    cuccaro_op(b, 0, true);

    for shift_i in (0..k).rev() {
        for i in 0..n - 1 {
            b.swap(v[i], v[i + 1]);
        }
        b.swap(v[n - 1], spill[k - 1 - shift_i]);
    }

    b.free(ovf);
    b.free_vec(&spill);
}

/// Fast `v := v/2 mod p`. Explicit reverse of `mod_double_inplace` with
/// measurement-based Cuccaro (not emit_inverse).
pub(crate) fn mod_halve_inplace_fast(b: &mut B, v: &[QubitId], p: U256) {
    mod_halve_inplace_fast_with_dirty(b, v, p, None)
}

pub(crate) fn mod_halve_inplace_direct_const_fast(b: &mut B, v: &[QubitId], p: U256) {
    let n = v.len();
    let ovf = b.alloc_qubit();
    debug_assert_eq!(n, 256);
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1));
    b.cx(v[0], ovf);
    csub_nbit_const_direct_fast(b, v, c, ovf);
    for i in 0..n - 1 {
        b.swap(v[i], v[i + 1]);
    }
    b.swap(v[n - 1], ovf);
    b.free(ovf);
}

/// Variant of `mod_halve_inplace_fast` that optionally borrows `dirty_src`
/// qubits for the controlled-sub step, using Gidney's venting
/// `cisub_dirty_2clean_classical`. Saves n transient qubits at the peak
/// when dirty qubits are available from the caller.
pub(crate) fn mod_halve_inplace_fast_with_dirty(
    b: &mut B,
    v: &[QubitId],
    p: U256,
    dirty_src: Option<&[QubitId]>,
) {
    let n = v.len();
    let ovf = b.alloc_qubit();
    debug_assert_eq!(n, 256);
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1));
    b.cx(v[0], ovf);
    // If caller provided enough dirty qubits AND c fits in u64 (it does
    // for secp256k1: c = 2^32 + 977), use the venting variant.
    let use_venting = kal_vent_halve_enabled() && dirty_src.map_or(false, |d| d.len() >= n - 2);
    if let Some(w) = double_carry_trunc_window() {
        // Carry-tail-truncated sparse-constant sub (inverse of the truncated
        // double; default OFF; same window so double/halve stay exact inverses).
        csub_nbit_const_direct_trunc_fast(b, v, c, ovf, w);
    } else if use_venting {
        // c as u64 (it fits: c = 0x1000003D1).
        // For n=256, we still need to pass the full 256-bit constant via u64.
        // Since c only has 33 bits, u64 is fine.
        let c_u64: u64 = c.as_limbs()[0] | (c.as_limbs()[1] << 32); // hack for U256
                                                                    // Actually, U256 limbs are u64[4]. Bit 32 of U256 is limbs[0] bit 32.
                                                                    // limbs[0] holds bits 0..64. So just take limbs[0] for bits < 64.
        let c_low = c.as_limbs()[0];
        let dirty = dirty_src.unwrap();
        let dirty_slice = &dirty[..n - 2];
        // We need 2 clean ancilla. Alloc them fresh.
        let q_clean2: [QubitId; 2] = [b.alloc_qubit(), b.alloc_qubit()];
        venting::cisub_dirty_2clean_classical(b, v, dirty_slice, &q_clean2, c_low, ovf);
        b.free(q_clean2[0]);
        b.free(q_clean2[1]);
        let _ = c_u64; // unused, c_low is the right value
    } else if direct_const_walks_enabled()
        || std::env::var("KAL_DIRECT_CONST_HALVE").ok().as_deref() == Some("1")
    {
        csub_nbit_const_direct_fast(b, v, c, ovf);
    } else {
        csub_nbit_const_fast(b, v, c, ovf);
    }
    for i in 0..n - 1 {
        b.swap(v[i], v[i + 1]);
    }
    b.swap(v[n - 1], ovf);
    b.free(ovf);
}
