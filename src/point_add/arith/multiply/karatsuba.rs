#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;


// ─── merged from karatsuba1.rs ───

// ═══════════════════════════════════════════════════════════════════════════
//  1-level Karatsuba multiplication
// ═══════════════════════════════════════════════════════════════════════════

pub(crate) fn karatsuba_half_sum_compute(b: &mut B, lo: &[QubitId], hi: &[QubitId], acc: &[QubitId]) {
    let h = lo.len();
    debug_assert_eq!(h, hi.len());
    debug_assert_eq!(acc.len(), h + 1);
    for i in 0..h {
        b.cx(lo[i], acc[i]);
    }
    let hi_pad = b.alloc_qubit();
    let mut hi_ext = hi.to_vec();
    hi_ext.push(hi_pad);
    add_nbit_qq_fast(b, &hi_ext, acc);
    b.free(hi_pad);
}

pub(crate) fn karatsuba_half_sum_uncompute(b: &mut B, lo: &[QubitId], hi: &[QubitId], acc: &[QubitId]) {
    let h = lo.len();
    let hi_pad = b.alloc_qubit();
    let mut hi_ext = hi.to_vec();
    hi_ext.push(hi_pad);
    sub_nbit_qq_fast(b, &hi_ext, acc);
    b.free(hi_pad);
    for i in 0..h {
        b.cx(lo[i], acc[i]);
    }
}

// ─── merged from karatsuba2.rs ───

/// Squaring-aware 1-level Karatsuba variant of [`squaring_sub_from_acc_schoolbook`].
///
/// Computes `acc -= x^2 mod p` (Solinas-reduced) via a 1-level Karatsuba
/// SQUARE. Split `x = hi‖lo` (`h = n/2` bits each) and form the three
/// SYMMETRIC sub-squares
///   z0 = lo^2,  z2 = hi^2,  z1 = (lo+hi)^2,
/// then combine `z1 -= z0 + z2` (= 2·lo·hi) and add the middle term:
///   x^2 = z0 + (z1 - z0 - z2)·2^h + z2·2^{2h}.
/// Each sub-square is the existing symmetric square (`schoolbook_square_symmetric`,
/// cross-products counted once via Gidney-uncomputed AND lanes), so the dominant
/// cross-product AND budget drops ~25 % vs the symmetric 256-bit schoolbook
/// square: 3·(n/2)(n/2-1)/2 cross ANDs instead of n(n-1)/2. Using a plain
/// Karatsuba MUL with x=y would re-introduce the cross terms and be strictly
/// worse — the symmetry of the SQUARE is what buys the win.
///
/// Peak control: the (lo+hi)^2 square is emitted FIRST, before the 2n-bit
/// `tmp_ext` result register is allocated, and its `x_sum` operand is freed
/// before `tmp_ext` is taken — so the z1 step (z1_reg + x_sum + row) and the
/// z0/z2 step (tmp_ext + z1_reg + row) never coexist. The combine carries use
/// the non-fast (ancilla-free) Cuccaro, and the Solinas lanes default to the
/// low-peak set (non-fast add/sub, direct-const double/halve, lowq shift) so the
/// extra z1_reg register (2(h+1) q) is absorbed without pushing the affine
/// square phase over the global GCD-body peak binder (~1567 < 1698).
pub(crate) fn squaring_sub_from_acc_karatsuba(b: &mut B, acc: &[QubitId], x: &[QubitId], p: U256) {
    let n = acc.len();
    debug_assert_eq!(n, 256);
    debug_assert_eq!(x.len(), n);
    let h = n / 2;
    let x_lo: Vec<QubitId> = x[0..h].to_vec();
    let x_hi: Vec<QubitId> = x[h..n].to_vec();

    // z1_reg holds z1 = (lo+hi)^2, width 2*(h+1).
    let mut z1_reg = b.alloc_qubits(2 * (h + 1));
    // KARA_FREE_Z1_TOPBIT: after z1 -= z0; z1 -= z2, z1_reg holds 2*lo*hi < 2^257,
    // so its top bit (index 2(h+1)-1 = 257) is provably 0 throughout the Solinas
    // peak. Free it for that window; re-grab a fresh zero before z1 += z2 restores
    // (lo+hi)^2 for the inverse uncompute. Bennett-clean (free zero, alloc zero).
    let free_z1_top = std::env::var("KARA_FREE_Z1_TOPBIT").ok().as_deref() == Some("1");
    // The z0=lo^2 / z2=hi^2 squares coexist with tmp_ext(2n)+z1_reg, and the
    // _fast symmetric square allocates a ~(h)-wide cuccaro carry lane on top of
    // its ~(h)-wide row — that lane is the round84 peak binder. The ancilla-free
    // _lowq square drops the carry lane (peak −~h) at a higher Toffoli cost.
    // z1=(lo+hi)^2 is computed before tmp_ext (low peak), so it stays _fast.
    let z02_lowq = std::env::var("KARA_Z02_LOWQ").ok().as_deref() == Some("1");

    // ── Forward z1 = (lo+hi)^2 FIRST (tmp_ext not yet allocated → low peak). ──
    {
        let x_sum = b.alloc_qubits(h + 1);
        karatsuba_half_sum_compute(b, &x_lo, &x_hi, &x_sum);
        schoolbook_square_symmetric(b, &x_sum, &z1_reg);
        karatsuba_half_sum_uncompute(b, &x_lo, &x_hi, &x_sum);
        b.free_vec(&x_sum);
    }

    // 2n-bit result accumulator for x^2 (allocated after the z1 square so its
    // 2n qubits never coexist with the z1 operand/row registers).
    let tmp_ext = b.alloc_qubits(2 * n);

    // z0 = lo^2 → tmp_ext[0..2h], z2 = hi^2 → tmp_ext[2h..4h].
    {
        let slice: Vec<QubitId> = tmp_ext[0..2 * h].to_vec();
        if z02_lowq {
            // z2 slice (tmp_ext[2h..4h]) is still clean here → host z0's fast
            // carry there (Toffoli-free peak drop) instead of paying lowq.
            let host: Vec<QubitId> = tmp_ext[2 * h..4 * h].to_vec();
            schoolbook_square_symmetric_hosted(b, &x_lo, &slice, &host);
        } else {
            schoolbook_square_symmetric(b, &x_lo, &slice);
        }
    }
    {
        let slice: Vec<QubitId> = tmp_ext[2 * h..4 * h].to_vec();
        if z02_lowq {
            if kara_z2_selfhost_enabled() {
                if square_selfhost_safe_lane_reuse_enabled() {
                    // z1=(lo+hi)^2 and z0=lo^2 are exact integer squares here.
                    // Every square is 0 or 1 mod 4, so bit 1 of each register is
                    // provably |0>.  Both lanes are disjoint from x_hi, z2, and
                    // z2's own untouched-tail carry lanes.
                    let clean_square_bits = [z1_reg[1], tmp_ext[1]];
                    schoolbook_square_symmetric_lowq_selfhosted_with_clean_supplement(
                        b,
                        &x_hi,
                        &slice,
                        &clean_square_bits,
                    );
                } else {
                    schoolbook_square_symmetric_lowq_selfhosted(b, &x_hi, &slice);
                }
            } else {
                schoolbook_square_symmetric_lowq(b, &x_hi, &slice);
            }
        } else {
            schoolbook_square_symmetric(b, &x_hi, &slice);
        }
    }

    // Combine: z1 -= z0; z1 -= z2; mid (tmp_ext[h..4h]) += z1. Non-fast Cuccaro
    // (no carry ancilla) keeps the peak flat while tmp_ext + z1_reg are live.
    {
        let pad = b.alloc_qubits(2);
        let mut z0_ext: Vec<QubitId> = tmp_ext[0..2 * h].to_vec();
        z0_ext.extend_from_slice(&pad);
        sub_nbit_qq(b, &z0_ext, &z1_reg);
        b.free_vec(&pad);
    }
    {
        let pad = b.alloc_qubits(2);
        let mut z2_ext: Vec<QubitId> = tmp_ext[2 * h..4 * h].to_vec();
        z2_ext.extend_from_slice(&pad);
        sub_nbit_qq(b, &z2_ext, &z1_reg);
        b.free_vec(&pad);
    }
    // z1_reg == 2*lo*hi < 2^257 here ⇒ bit 257 is 0. Release it for the peak window.
    if free_z1_top {
        let top = z1_reg.pop().expect("z1_reg width 2*(h+1) >= 2");
        b.free(top);
    }
    {
        let pad = b.alloc_qubits(3 * h - z1_reg.len());
        let mut z1_ext: Vec<QubitId> = z1_reg.to_vec();
        z1_ext.extend_from_slice(&pad);
        let acc_slice: Vec<QubitId> = tmp_ext[h..4 * h].to_vec();
        add_nbit_qq(b, &z1_ext, &acc_slice);
        b.free_vec(&pad);
    }

    // ── Solinas reduction: acc -= (lo + hi·c) mod p. ──
    // z1_reg (2(h+1) q) is still live through this whole block, so the lanes
    // that allocate a full-width carry ancilla (fast Cuccaro add/sub, fast
    // shift) bind the affine-square phase peak. Each lane defaults to its
    // low-peak (ancilla-free) variant so the phase peak stays below the global
    // GCD-body binder; per-lane env knobs select the higher-peak fast variants
    // for measurement (each computes the SAME value on `acc`, so any mix is
    // value-correct):
    //   KARA_SOL_MOD_FAST=1   → fast mod add/sub          (else non-fast)
    //   KARA_SOL_DBL_FAST=1   → fast in-place double/halve (else direct-const)
    //   KARA_SOL_SHIFT_FAST=1 → fast shift-by-22          (else lowq shift)
    let mod_fast = std::env::var("KARA_SOL_MOD_FAST").ok().as_deref() == Some("1");
    let dbl_fast = std::env::var("KARA_SOL_DBL_FAST").ok().as_deref() == Some("1");
    let shift_fast = std::env::var("KARA_SOL_SHIFT_FAST").ok().as_deref() == Some("1");
    let lo: Vec<QubitId> = tmp_ext[0..n].to_vec();
    let hi: Vec<QubitId> = tmp_ext[n..2 * n].to_vec();
    // The non-fast mod_add/sub materialize a 256-q load_const for the Solinas
    // `c` correction, which coexists with tmp_ext + z1_reg and binds the phase
    // peak. The vent form hosts that correction on the operand `a_ext` (dirty,
    // value-preserved) for 2 clean qubits, dropping the transient ~n.
    let mod_vent = std::env::var("KARA_SOL_MOD_VENT").ok().as_deref() == Some("1");
    let mod_sub = |b: &mut B, acc: &[QubitId], a: &[QubitId]| {
        if mod_vent {
            mod_sub_qq_vent(b, acc, a, p);
        } else if mod_fast {
            mod_sub_qq_fast(b, acc, a, p);
        } else {
            mod_sub_qq(b, acc, a, p);
        }
    };
    let mod_add = |b: &mut B, acc: &[QubitId], a: &[QubitId]| {
        if mod_vent {
            mod_add_qq_vent(b, acc, a, p);
        } else if mod_fast {
            mod_add_qq_fast(b, acc, a, p);
        } else {
            mod_add_qq(b, acc, a, p);
        }
    };
    let mod_dbl = |b: &mut B, v: &[QubitId]| {
        if dbl_fast {
            mod_double_inplace_fast(b, v, p);
        } else {
            mod_double_inplace_direct_const_fast(b, v, p);
        }
    };
    let mod_hlv = |b: &mut B, v: &[QubitId]| {
        if dbl_fast {
            mod_halve_inplace_fast(b, v, p);
        } else {
            mod_halve_inplace_direct_const_fast(b, v, p);
        }
    };
    b.set_phase("r84k_sol_subadd");
    mod_sub(b, acc, &lo);
    mod_sub(b, acc, &hi);
    for _ in 0..4 {
        mod_dbl(b, &hi);
    }
    mod_sub(b, acc, &hi);
    for _ in 0..2 {
        mod_dbl(b, &hi);
    }
    mod_add(b, acc, &hi); // sign flipped
    for _ in 0..4 {
        mod_dbl(b, &hi);
    }
    mod_sub(b, acc, &hi);
    b.set_phase("r84k_sol_shift");
    // The shift-by-22 lane binds the affine-square phase peak: its lowq form
    // allocates a ~(n+1)-wide `padded` scratch on top of the live z1_reg+tmp_ext,
    // overflowing the free pool. `acc` (tx) is idle and value-preserved during the
    // shift itself, so the dirty-borrow form hosts that scratch on `acc` (venting
    // 2-clean), dropping the phase peak well under the GCD-apply binder. Same value
    // on `acc`; gated so it can be A/B compared.
    let shift_dirty = std::env::var("ROUND84_XTAIL_BORROW_CARRIES").ok().as_deref() == Some("1");
    if shift_dirty {
        // Dirty-doubles form of `acc -= hi * 2^22 mod p`: 22 in-place doubles
        // (each borrows `acc` via Gidney venting) avoid the shift's persistent
        // k-wide `spill` lane that — stacked on the live z1_reg+tmp_ext base —
        // pushed the shift/mid-sub over the GCD-apply binder. `acc` is idle and
        // value-preserved during each double/halve, so the phase peak drops well
        // under 1558. Mirrors the schoolbook_peak_lowq D1 reduction lane.
        b.set_phase("r84k_sol_dbl22");
        for _ in 0..22 {
            mod_dbl(b, &hi);
        }
        b.set_phase("r84k_sol_midsub");
        mod_sub(b, acc, &hi);
        b.set_phase("r84k_sol_hlv22");
        for _ in 0..22 {
            mod_hlv(b, &hi);
        }
    } else {
        b.set_phase("r84k_sol_shiftL");
        let (spill, flag_inv, ovf) = if shift_fast {
            mod_shift_left_by_k(b, &hi, p, 22)
        } else {
            mod_shift_left_by_k_lowq(b, &hi, p, 22)
        };
        b.set_phase("r84k_sol_midsub");
        mod_sub(b, acc, &hi);
        b.set_phase("r84k_sol_shiftR");
        if shift_fast {
            mod_shift_right_by_k(b, &hi, p, 22, spill, flag_inv, ovf);
        } else {
            mod_shift_right_by_k_lowq(b, &hi, p, 22, spill, flag_inv, ovf);
        }
    }
    b.set_phase("r84k_sol_halve");
    for _ in 0..10 {
        mod_hlv(b, &hi);
    }

    // ── Inverse combine: mid -= z1; z1 += z2; z1 += z0. ──
    b.set_phase("r84k_inv_combine");
    {
        let pad = b.alloc_qubits(3 * h - z1_reg.len());
        let mut z1_ext: Vec<QubitId> = z1_reg.to_vec();
        z1_ext.extend_from_slice(&pad);
        let acc_slice: Vec<QubitId> = tmp_ext[h..4 * h].to_vec();
        sub_nbit_qq(b, &z1_ext, &acc_slice);
        b.free_vec(&pad);
    }
    // Restore z1_reg top bit (fresh zero) before z1 += z2 can re-set it.
    if free_z1_top {
        let top = b.alloc_qubit();
        z1_reg.push(top);
    }
    {
        let pad = b.alloc_qubits(2);
        let mut z2_ext: Vec<QubitId> = tmp_ext[2 * h..4 * h].to_vec();
        z2_ext.extend_from_slice(&pad);
        add_nbit_qq(b, &z2_ext, &z1_reg);
        b.free_vec(&pad);
    }
    {
        let pad = b.alloc_qubits(2);
        let mut z0_ext: Vec<QubitId> = tmp_ext[0..2 * h].to_vec();
        z0_ext.extend_from_slice(&pad);
        add_nbit_qq(b, &z0_ext, &z1_reg);
        b.free_vec(&pad);
    }

    // Uncompute z2, z0 (reverse of forward compute order), then free tmp_ext.
    b.set_phase("r84k_z_inv_squares");
    {
        let slice: Vec<QubitId> = tmp_ext[2 * h..4 * h].to_vec();
        if z02_lowq {
            if kara_z2_selfhost_enabled() {
                if square_selfhost_safe_lane_reuse_enabled() {
                    // Inverse-combine restored the exact z1 and z0 squares
                    // before this block, so their square-bit-1 lanes are clean
                    // scratch again (the mirror of the forward z2 proof).
                    let clean_square_bits = [z1_reg[1], tmp_ext[1]];
                    schoolbook_square_symmetric_lowq_selfhosted_inverse_with_clean_supplement(
                        b,
                        &x_hi,
                        &slice,
                        &clean_square_bits,
                    );
                } else {
                    schoolbook_square_symmetric_lowq_selfhosted_inverse(b, &x_hi, &slice);
                }
            } else {
                schoolbook_square_symmetric_lowq_inverse(b, &x_hi, &slice);
            }
        } else {
            schoolbook_square_symmetric_inverse(b, &x_hi, &slice);
        }
    }
    {
        let slice: Vec<QubitId> = tmp_ext[0..2 * h].to_vec();
        if z02_lowq {
            // z2 slice was just uncomputed above → clean again, host inv-z0's
            // borrow there (mirror of the forward z0 hosting).
            let host: Vec<QubitId> = tmp_ext[2 * h..4 * h].to_vec();
            schoolbook_square_symmetric_hosted_inverse(b, &x_lo, &slice, &host);
        } else {
            schoolbook_square_symmetric_inverse(b, &x_lo, &slice);
        }
    }
    b.free_vec(&tmp_ext);

    // Uncompute z1 last (mirrors the forward z1-first ordering, tmp_ext freed).
    {
        let x_sum = b.alloc_qubits(h + 1);
        karatsuba_half_sum_compute(b, &x_lo, &x_hi, &x_sum);
        schoolbook_square_symmetric_inverse(b, &x_sum, &z1_reg);
        karatsuba_half_sum_uncompute(b, &x_lo, &x_hi, &x_sum);
        b.free_vec(&x_sum);
    }

    b.free_vec(&z1_reg);
}