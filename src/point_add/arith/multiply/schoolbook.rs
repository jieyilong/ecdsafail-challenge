#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;


/// Symmetric schoolbook for squaring: x² = sum_i x[i]·2^(2i) + sum_{i<j} 2·x[i]·x[j]·2^(i+j).
/// Each cross-product is computed ONCE (instead of twice in full schoolbook),
/// halving the AND count + Cuccaro_add length. Saves ~130k CCX per squaring.
///
/// Row i layout (width n-i): bit 0 = diagonal x[i] at position 2i, bit 1 = 0
/// (gap), bit k+2 = cross-product (x[i] AND x[i+1+k]) at position i+(i+1+k)+1.
pub(crate) fn schoolbook_square_symmetric(b: &mut B, x: &[QubitId], tmp_ext: &[QubitId]) {
    let n = x.len();
    debug_assert_eq!(tmp_ext.len(), 2 * n);
    for i in 0..n {
        // Width: bit 0 = diag at pos 2i, bit 1 = gap, bits 2..(n-i) = cross-
        // products at positions 2i+2..i+n. Last bit index = n-i, so width = n-i+1.
        // Edge case: i = n-1 has only the diagonal, width = 1.
        let width = if i == n - 1 { 1 } else { n - i + 1 };
        let num_cross = if i + 1 < n { n - i - 1 } else { 0 };
        // num_cross = number of cross-products in this row = width - 2 when width >= 2.
        let row = b.alloc_qubits(width);
        b.cx(x[i], row[0]);
        for k in 0..num_cross {
            b.ccx(x[i], x[i + 1 + k], row[k + 2]);
        }
        let pad = b.alloc_qubit();
        let mut row_padded = row.clone();
        row_padded.push(pad);
        let slice: Vec<QubitId> = tmp_ext[2 * i..2 * i + width + 1].to_vec();
        let c_in = b.alloc_qubit();
        cuccaro_add_fast(b, &row_padded, &slice, c_in);
        b.free(c_in);
        b.free(pad);
        b.cx(x[i], row[0]);
        for k in 0..num_cross {
            let m = b.alloc_bit();
            b.hmr(row[k + 2], m);
            b.cz_if(x[i], x[i + 1 + k], m);
        }
        b.free_vec(&row);
    }
}

// ─── merged from schoolbook2.rs ───

pub(crate) fn schoolbook_square_symmetric_inverse(b: &mut B, x: &[QubitId], tmp_ext: &[QubitId]) {
    let n = x.len();
    for i in (0..n).rev() {
        let width = if i == n - 1 { 1 } else { n - i + 1 };
        let num_cross = if i + 1 < n { n - i - 1 } else { 0 };
        let row = b.alloc_qubits(width);
        b.cx(x[i], row[0]);
        for k in 0..num_cross {
            b.ccx(x[i], x[i + 1 + k], row[k + 2]);
        }
        let pad = b.alloc_qubit();
        let mut row_padded = row.clone();
        row_padded.push(pad);
        let slice: Vec<QubitId> = tmp_ext[2 * i..2 * i + width + 1].to_vec();
        let c_in = b.alloc_qubit();
        cuccaro_sub_fast(b, &row_padded, &slice, c_in);
        b.free(c_in);
        b.free(pad);
        b.cx(x[i], row[0]);
        for k in 0..num_cross {
            let m = b.alloc_bit();
            b.hmr(row[k + 2], m);
            b.cz_if(x[i], x[i + 1 + k], m);
        }
        b.free_vec(&row);
    }
}

pub(crate) fn schoolbook_square_symmetric_lowq(b: &mut B, x: &[QubitId], tmp_ext: &[QubitId]) {
    let n = x.len();
    debug_assert_eq!(tmp_ext.len(), 2 * n);
    for i in 0..n {
        let width = if i == n - 1 { 1 } else { n - i + 1 };
        let num_cross = if i + 1 < n { n - i - 1 } else { 0 };
        let row = b.alloc_qubits(width);
        b.cx(x[i], row[0]);
        for k in 0..num_cross {
            b.ccx(x[i], x[i + 1 + k], row[k + 2]);
        }
        let pad = b.alloc_qubit();
        let mut row_padded = row.clone();
        row_padded.push(pad);
        let slice: Vec<QubitId> = tmp_ext[2 * i..2 * i + width + 1].to_vec();
        let c_in = b.alloc_qubit();
        cuccaro_add(b, &row_padded, &slice, c_in);
        b.free(c_in);
        b.free(pad);
        b.cx(x[i], row[0]);
        for k in 0..num_cross {
            let m = b.alloc_bit();
            b.hmr(row[k + 2], m);
            b.cz_if(x[i], x[i + 1 + k], m);
        }
        b.free_vec(&row);
    }
}

pub(crate) fn schoolbook_square_symmetric_lowq_inverse(b: &mut B, x: &[QubitId], tmp_ext: &[QubitId]) {
    let n = x.len();
    for i in (0..n).rev() {
        let width = if i == n - 1 { 1 } else { n - i + 1 };
        let num_cross = if i + 1 < n { n - i - 1 } else { 0 };
        let row = b.alloc_qubits(width);
        b.cx(x[i], row[0]);
        for k in 0..num_cross {
            b.ccx(x[i], x[i + 1 + k], row[k + 2]);
        }
        let pad = b.alloc_qubit();
        let mut row_padded = row.clone();
        row_padded.push(pad);
        let slice: Vec<QubitId> = tmp_ext[2 * i..2 * i + width + 1].to_vec();
        let c_in = b.alloc_qubit();
        cuccaro_sub(b, &row_padded, &slice, c_in);
        b.free(c_in);
        b.free(pad);
        b.cx(x[i], row[0]);
        for k in 0..num_cross {
            let m = b.alloc_bit();
            b.hmr(row[k + 2], m);
            b.cz_if(x[i], x[i + 1 + k], m);
        }
        b.free_vec(&row);
    }
}

/// Like `schoolbook_square_symmetric` (fast, measurement UMA) but the per-row
/// Cuccaro carry lane is hosted on a caller-supplied clean register `host`
/// (returned clean) instead of a fresh allocation. Toffoli-identical to the
/// fast square, peak-identical to the lowq square — used for the z0 lobe of the
/// round84 Karatsuba square, where the not-yet-written z2 slice is clean scratch.
pub(crate) fn schoolbook_square_symmetric_hosted(b: &mut B, x: &[QubitId], tmp_ext: &[QubitId], host: &[QubitId]) {
    let n = x.len();
    debug_assert_eq!(tmp_ext.len(), 2 * n);
    if square_selfhost_safe_lane_reuse_enabled() {
        assert_qubit_slices_disjoint(&[x, tmp_ext, host]);
    }
    for i in 0..n {
        let width = if i == n - 1 { 1 } else { n - i + 1 };
        let num_cross = if i + 1 < n { n - i - 1 } else { 0 };
        let row = b.alloc_qubits(width);
        b.cx(x[i], row[0]);
        for k in 0..num_cross {
            b.ccx(x[i], x[i + 1 + k], row[k + 2]);
        }
        let slice: Vec<QubitId> = tmp_ext[2 * i..2 * i + width + 1].to_vec();
        if square_selfhost_safe_lane_reuse_enabled() {
            // The z2 sibling host is clean and disjoint from x and z0.  It has
            // ample room for both the width carry lanes and one clean c_in.
            assert!(host.len() > width);
            cuccaro_add_fast_low_to_ext_borrowed_carries(
                b,
                &row,
                &slice,
                host[width],
                &host[..width],
            );
        } else {
            let pad = b.alloc_qubit();
            let mut row_padded = row.clone();
            row_padded.push(pad);
            let c_in = b.alloc_qubit();
            cuccaro_add_fast_borrowed_carries(
                b,
                &row_padded,
                &slice,
                c_in,
                &host[..row_padded.len() - 1],
            );
            b.free(c_in);
            b.free(pad);
        }
        b.cx(x[i], row[0]);
        for k in 0..num_cross {
            let m = b.alloc_bit();
            b.hmr(row[k + 2], m);
            b.cz_if(x[i], x[i + 1 + k], m);
        }
        b.free_vec(&row);
    }
}
pub(crate) fn schoolbook_square_symmetric_hosted_inverse(
    b: &mut B,
    x: &[QubitId],
    tmp_ext: &[QubitId],
    host: &[QubitId],
) {
    let n = x.len();
    if square_selfhost_safe_lane_reuse_enabled() {
        assert_qubit_slices_disjoint(&[x, tmp_ext, host]);
    }
    for i in (0..n).rev() {
        let width = if i == n - 1 { 1 } else { n - i + 1 };
        let num_cross = if i + 1 < n { n - i - 1 } else { 0 };
        let row = b.alloc_qubits(width);
        b.cx(x[i], row[0]);
        for k in 0..num_cross {
            b.ccx(x[i], x[i + 1 + k], row[k + 2]);
        }
        let slice: Vec<QubitId> = tmp_ext[2 * i..2 * i + width + 1].to_vec();
        if square_selfhost_safe_lane_reuse_enabled() {
            assert!(host.len() > width);
            cuccaro_sub_fast_low_to_ext_borrowed_carries(
                b,
                &row,
                &slice,
                host[width],
                &host[..width],
            );
        } else {
            let pad = b.alloc_qubit();
            let mut row_padded = row.clone();
            row_padded.push(pad);
            let c_in = b.alloc_qubit();
            cuccaro_sub_fast_borrowed_carries(
                b,
                &row_padded,
                &slice,
                c_in,
                &host[..row_padded.len() - 1],
            );
            b.free(c_in);
            b.free(pad);
        }
        b.cx(x[i], row[0]);
        for k in 0..num_cross {
            let m = b.alloc_bit();
            b.hmr(row[k + 2], m);
            b.cz_if(x[i], x[i + 1 + k], m);
        }
        b.free_vec(&row);
    }
}
pub(crate) fn schoolbook_square_symmetric_lowq_selfhosted(b: &mut B, x: &[QubitId], tmp_ext: &[QubitId]) {
    schoolbook_square_symmetric_lowq_selfhosted_with_clean_supplement(b, x, tmp_ext, &[]);
}
pub(crate) fn schoolbook_square_symmetric_lowq_selfhosted_inverse(b: &mut B, x: &[QubitId], tmp_ext: &[QubitId]) {
    schoolbook_square_symmetric_lowq_selfhosted_inverse_with_clean_supplement(b, x, tmp_ext, &[]);
}
pub(crate) fn kara_z2_selfhost_enabled() -> bool {
    std::env::var("KARA_Z2_SELFHOST").ok().as_deref() != Some("0")
}

pub(crate) fn xtail_sq_selfhost_enabled() -> bool {
    std::env::var("XTAIL_SQ_SELFHOST").ok().as_deref() != Some("0")
}

/// Schoolbook squarer with Bennett uncompute. For squaring `tmp_ext = x*x`
/// (2n bits, no mod reduction), then sub from acc with on-the-fly Solinas
/// reduction, then uncompute tmp_ext via gate-level inverse. Saves ~170k
/// CCX vs walk-x squaring (459k → 289k) by avoiding 256 expensive
/// cmod_add_qq calls (each 5n) in favor of 2n²=131k of cheap AND+Cuccaro.
pub(crate) fn squaring_sub_from_acc_schoolbook(b: &mut B, acc: &[QubitId], x: &[QubitId], p: U256) {
    let n = acc.len();
    debug_assert_eq!(n, 256);
    debug_assert_eq!(x.len(), n);
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1));

    // Wide accumulator (2n bits) starts at 0.
    let tmp_ext = b.alloc_qubits(2 * n);

    // Phase 1: symmetric schoolbook tmp_ext = x*x (~half the CCX of full).
    schoolbook_square_symmetric(b, x, &tmp_ext);

    // Phase 2: subtract (lo + hi*c mod p) from acc.
    // For each set bit k of c, sub (hi shifted by k mod p) from acc, by
    // walking hi via mod_double in place. Sub lo first.
    let lo: Vec<QubitId> = tmp_ext[0..n].to_vec();
    let hi: Vec<QubitId> = tmp_ext[n..2 * n].to_vec();
    mod_sub_qq_fast(b, acc, &lo, p);
    let _ = c;
    // 977 consolidation: c = {+2^0, +2^4, -2^6, +2^10, +2^32}. For acc-=hi·c, signs flip:
    // acc -= hi·2^0, acc -= hi·2^4, acc += hi·2^6, acc -= hi·2^10, acc -= hi·2^32.
    mod_sub_qq_fast(b, acc, &hi, p);
    for _ in 0..4 {
        mod_double_inplace_fast(b, &hi, p);
    }
    mod_sub_qq_fast(b, acc, &hi, p);
    for _ in 0..2 {
        mod_double_inplace_fast(b, &hi, p);
    }
    mod_add_qq_fast(b, acc, &hi, p); // sign flipped
    for _ in 0..4 {
        mod_double_inplace_fast(b, &hi, p);
    }
    mod_sub_qq_fast(b, acc, &hi, p);
    let (spill, flag_inv, ovf) = mod_shift_left_by_k(b, &hi, p, 22);
    mod_sub_qq(b, acc, &hi, p);
    mod_shift_right_by_k(b, &hi, p, 22, spill, flag_inv, ovf);
    for _ in 0..10 {
        mod_halve_inplace_fast(b, &hi, p);
    }

    // Phase 3: uncompute tmp_ext via symmetric schoolbook inverse.
    schoolbook_square_symmetric_inverse(b, x, &tmp_ext);

    b.free_vec(&tmp_ext);
}

pub(crate) fn squaring_sub_from_acc_schoolbook_lowq_shift22(
    b: &mut B,
    acc: &[QubitId],
    x: &[QubitId],
    p: U256,
) {
    let n = acc.len();
    debug_assert_eq!(n, 256);
    debug_assert_eq!(x.len(), n);
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1));

    let tmp_ext = b.alloc_qubits(2 * n);
    if xtail_sq_selfhost_enabled() {
        schoolbook_square_symmetric_lowq_selfhosted(b, x, &tmp_ext);
    } else {
        schoolbook_square_symmetric_lowq(b, x, &tmp_ext);
    }

    let lo: Vec<QubitId> = tmp_ext[0..n].to_vec();
    let hi: Vec<QubitId> = tmp_ext[n..2 * n].to_vec();
    mod_sub_qq(b, acc, &lo, p);
    let _ = c;
    mod_sub_qq(b, acc, &hi, p);
    for _ in 0..4 {
        mod_double_inplace_direct_const_fast(b, &hi, p);
    }
    mod_sub_qq(b, acc, &hi, p);
    for _ in 0..2 {
        mod_double_inplace_direct_const_fast(b, &hi, p);
    }
    mod_add_qq(b, acc, &hi, p);
    for _ in 0..4 {
        mod_double_inplace_direct_const_fast(b, &hi, p);
    }
    mod_sub_qq(b, acc, &hi, p);
    let (spill, flag_inv, ovf) = mod_shift_left_by_k_lowq(b, &hi, p, 22);
    mod_sub_qq(b, acc, &hi, p);
    mod_shift_right_by_k_lowq(b, &hi, p, 22, spill, flag_inv, ovf);
    for _ in 0..10 {
        mod_halve_inplace_direct_const_fast(b, &hi, p);
    }

    if xtail_sq_selfhost_enabled() {
        schoolbook_square_symmetric_lowq_selfhosted_inverse(b, x, &tmp_ext);
    } else {
        schoolbook_square_symmetric_lowq_inverse(b, x, &tmp_ext);
    }
    b.free_vec(&tmp_ext);
}


pub(crate) fn schoolbook_square_symmetric_lowq_selfhosted_with_clean_supplement(
    b: &mut B,
    x: &[QubitId],
    tmp_ext: &[QubitId],
    clean_supplement: &[QubitId],
) {
    let n = x.len();
    debug_assert_eq!(tmp_ext.len(), 2 * n);
    let safe_reuse = square_selfhost_safe_lane_reuse_enabled();
    if safe_reuse {
        assert_qubit_slices_disjoint(&[x, tmp_ext, clean_supplement]);
    }
    let gate_prefix_rows = std::env::var("SQUARE_SELFHOST_GATE_PREFIX_ROWS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    for i in 0..n {
        let width = if i == n - 1 { 1 } else { n - i + 1 };
        let num_cross = if i + 1 < n { n - i - 1 } else { 0 };
        let row = b.alloc_qubits(width);
        b.cx(x[i], row[0]);
        for k in 0..num_cross {
            b.ccx(x[i], x[i + 1 + k], row[k + 2]);
        }
        let hi = 2 * i + width + 1;
        let slice: Vec<QubitId> = tmp_ext[2 * i..hi].to_vec();
        if i < gate_prefix_rows {
            let pad = b.alloc_qubit();
            let mut row_padded = row.clone();
            row_padded.push(pad);
            let c_in = b.alloc_qubit();
            cuccaro_add(b, &row_padded, &slice, c_in);
            b.free(c_in);
            b.free(pad);
        } else if safe_reuse {
            let need = row.len() - square_selfhost_gate_suffix_carries(row.len());
            let avail = tmp_ext.len() - hi;
            let from_tmp = need.min(avail);
            let from_supplement = (need - from_tmp).min(clean_supplement.len());
            let from_global = need - from_tmp - from_supplement;
            let gpool = b.alloc_qubits(from_global);
            let mut carries: Vec<QubitId> = tmp_ext[hi..hi + from_tmp].to_vec();
            carries.extend_from_slice(&clean_supplement[..from_supplement]);
            carries.extend_from_slice(&gpool);
            cuccaro_add_fast_low_to_ext_borrowed_carries_no_cin(b, &row, &slice, &carries);
            b.free_vec(&gpool);
        } else {
            let pad = b.alloc_qubit();
            let mut row_padded = row.clone();
            row_padded.push(pad);
            let c_in = b.alloc_qubit();
            let need = row_padded.len() - 1;
            let avail = tmp_ext.len() - hi;
            let from_tmp = need.min(avail);
            let from_global = need - from_tmp;
            let gpool = b.alloc_qubits(from_global);
            let mut carries: Vec<QubitId> = tmp_ext[hi..hi + from_tmp].to_vec();
            carries.extend_from_slice(&gpool);
            cuccaro_add_fast_borrowed_carries(b, &row_padded, &slice, c_in, &carries);
            b.free(c_in);
            b.free_vec(&gpool);
            b.free(pad);
        }
        b.cx(x[i], row[0]);
        for k in 0..num_cross {
            let m = b.alloc_bit();
            b.hmr(row[k + 2], m);
            b.cz_if(x[i], x[i + 1 + k], m);
        }
        b.free_vec(&row);
    }
}

pub(crate) fn schoolbook_square_symmetric_lowq_selfhosted_inverse_with_clean_supplement(
    b: &mut B,
    x: &[QubitId],
    tmp_ext: &[QubitId],
    clean_supplement: &[QubitId],
) {
    let n = x.len();
    debug_assert_eq!(tmp_ext.len(), 2 * n);
    let safe_reuse = square_selfhost_safe_lane_reuse_enabled();
    if safe_reuse {
        assert_qubit_slices_disjoint(&[x, tmp_ext, clean_supplement]);
    }
    let gate_prefix_rows = std::env::var("SQUARE_SELFHOST_GATE_PREFIX_ROWS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    for i in (0..n).rev() {
        let width = if i == n - 1 { 1 } else { n - i + 1 };
        let num_cross = if i + 1 < n { n - i - 1 } else { 0 };
        let row = b.alloc_qubits(width);
        b.cx(x[i], row[0]);
        for k in 0..num_cross {
            b.ccx(x[i], x[i + 1 + k], row[k + 2]);
        }
        let hi = 2 * i + width + 1;
        let slice: Vec<QubitId> = tmp_ext[2 * i..hi].to_vec();
        if i < gate_prefix_rows {
            let pad = b.alloc_qubit();
            let mut row_padded = row.clone();
            row_padded.push(pad);
            let c_in = b.alloc_qubit();
            cuccaro_sub(b, &row_padded, &slice, c_in);
            b.free(c_in);
            b.free(pad);
        } else if safe_reuse {
            let need = row.len() - square_selfhost_gate_suffix_carries(row.len());
            let avail = tmp_ext.len() - hi;
            let from_tmp = need.min(avail);
            let from_supplement = (need - from_tmp).min(clean_supplement.len());
            let from_global = need - from_tmp - from_supplement;
            let gpool = b.alloc_qubits(from_global);
            let mut carries: Vec<QubitId> = tmp_ext[hi..hi + from_tmp].to_vec();
            carries.extend_from_slice(&clean_supplement[..from_supplement]);
            carries.extend_from_slice(&gpool);
            cuccaro_sub_fast_low_to_ext_borrowed_carries_no_cin(b, &row, &slice, &carries);
            b.free_vec(&gpool);
        } else {
            let pad = b.alloc_qubit();
            let mut row_padded = row.clone();
            row_padded.push(pad);
            let c_in = b.alloc_qubit();
            let need = row_padded.len() - 1;
            let avail = tmp_ext.len() - hi;
            let from_tmp = need.min(avail);
            let from_global = need - from_tmp;
            let gpool = b.alloc_qubits(from_global);
            let mut carries: Vec<QubitId> = tmp_ext[hi..hi + from_tmp].to_vec();
            carries.extend_from_slice(&gpool);
            cuccaro_sub_fast_borrowed_carries(b, &row_padded, &slice, c_in, &carries);
            b.free(c_in);
            b.free_vec(&gpool);
            b.free(pad);
        }
        b.cx(x[i], row[0]);
        for k in 0..num_cross {
            let m = b.alloc_bit();
            b.hmr(row[k + 2], m);
            b.cz_if(x[i], x[i + 1 + k], m);
        }
        b.free_vec(&row);
    }
}

pub(crate) fn square_selfhost_gate_suffix_carries(n: usize) -> usize {
    std::env::var("SQUARE_SELFHOST_GATE_SUFFIX_CARRIES")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0)
        .min(n.saturating_sub(1))
}

/// Experimental square-only reclaim.  This is deliberately opt-in: every lane
/// borrowed by the prototype is either an untouched high tail of the square
/// accumulator, a caller-proved square bit that is exactly zero, or a clean
/// sibling square destination.  Dirty-but-idle data and operand aliases are not
/// eligible.
pub(crate) fn square_selfhost_safe_lane_reuse_enabled() -> bool {
    std::env::var("SQUARE_SELFHOST_SAFE_LANE_REUSE")
        .ok()
        .as_deref()
        == Some("1")
}