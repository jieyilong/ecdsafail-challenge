#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;


// ─── merged from misc1.rs ───

pub(crate) fn dialog_gcd_apply_chunked_f_blocks() -> Option<usize> {
    std::env::var("DIALOG_GCD_APPLY_CHUNKED_F_BLOCKS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&blocks| blocks >= 2)
}

pub(crate) fn dialog_gcd_apply_chunked_f_cut() -> Option<usize> {
    std::env::var("DIALOG_GCD_APPLY_CHUNKED_F_CUT")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&cut| (1..N).contains(&cut))
}

pub(crate) fn dialog_gcd_apply_chunked_f_cut2() -> Option<usize> {
    std::env::var("DIALOG_GCD_APPLY_CHUNKED_F_CUT2")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&cut| (1..N).contains(&cut))
}

pub(crate) fn dialog_gcd_apply_chunked_f_cut3() -> Option<usize> {
    std::env::var("DIALOG_GCD_APPLY_CHUNKED_F_CUT3")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&cut| (1..N).contains(&cut))
}

pub(crate) fn dialog_gcd_apply_chunked_f_custom4_enabled() -> bool {
    std::env::var("DIALOG_GCD_APPLY_CHUNKED_F_CUSTOM4")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_apply_chunked_f_reuse_cin_zero_enabled() -> bool {
    std::env::var("DIALOG_GCD_APPLY_CHUNKED_F_REUSE_CIN_ZERO")
        .ok()
        .as_deref()
        != Some("0")
}

pub(crate) fn dialog_gcd_apply_chunked_f_fuse_boundary_clears_enabled() -> bool {
    std::env::var("DIALOG_GCD_APPLY_CHUNKED_F_FUSE_BOUNDARY_CLEARS")
        .ok()
        .as_deref()
        != Some("0")
}

pub(crate) fn dialog_gcd_active_iterations() -> usize {
    std::env::var(DIALOG_GCD_ACTIVE_ITERATIONS_ENV)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&iters| (1..=DIALOG_GCD_MAX_ITERATIONS).contains(&iters))
        .unwrap_or(DIALOG_GCD_MAX_ITERATIONS)
}

pub(crate) fn dialog_gcd_compare_bits() -> usize {
    std::env::var(DIALOG_GCD_COMPARE_BITS_ENV)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&bits| (1..=N).contains(&bits))
        .unwrap_or(DIALOG_GCD_DEFAULT_COMPARE_BITS)
}

pub(crate) fn dialog_gcd_apply_clean_compare_bits() -> usize {
    std::env::var(DIALOG_GCD_APPLY_CLEAN_COMPARE_BITS_ENV)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&bits| (1..=N).contains(&bits))
        .unwrap_or_else(dialog_gcd_compare_bits)
}

pub(crate) fn dialog_gcd_pa9024_compare_schedule_enabled() -> bool {
    std::env::var(DIALOG_GCD_PA9024_COMPARE_SCHEDULE_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_pa9024_compare_schedule_floor() -> usize {
    std::env::var(DIALOG_GCD_PA9024_COMPARE_SCHEDULE_FLOOR_ENV)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&bits| bits <= N)
        .unwrap_or(1)
        .max(1)
}

pub(crate) fn dialog_gcd_pa9024_compare_schedule_margin() -> usize {
    std::env::var("DIALOG_GCD_PA9024_COMPARE_SCHEDULE_MARGIN")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0)
}

pub(crate) fn dialog_gcd_compare_bits_for_step(step: usize, active_width: usize) -> usize {
    let global = dialog_gcd_compare_bits().min(active_width);
    if dialog_gcd_pa9024_compare_schedule_enabled() {
        let scheduled = (DIALOG_GCD_PA9024_COMPARE_SCHEDULE
            .get(step)
            .copied()
            .unwrap_or(global)
            + dialog_gcd_pa9024_compare_schedule_margin())
        .max(dialog_gcd_pa9024_compare_schedule_floor())
        .min(active_width);
        return scheduled.min(global).max(1);
    }
    global.max(1)
}

pub(crate) fn dialog_gcd_fused_branch_bits_enabled() -> bool {
    std::env::var("DIALOG_GCD_FUSED_BRANCH_BITS")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_odd_u_lowbit_fastpath_enabled() -> bool {
    std::env::var("DIALOG_GCD_ODD_U_LOWBIT_FASTPATH")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_cmp_gt_truncated_into_width(
    b: &mut B,
    u: &[QubitId],
    v: &[QubitId],
    flag: QubitId,
    compare_bits: usize,
) {
    assert_eq!(u.len(), v.len());
    assert!(!u.is_empty());
    let compare_bits = compare_bits.min(u.len()).max(1);
    let start = u.len() - compare_bits;
    cmp_lt_into_fast(b, &v[start..], &u[start..], flag);
}

pub(crate) fn dialog_gcd_ccx_cmp_gt_truncated_into_width(
    b: &mut B,
    u: &[QubitId],
    v: &[QubitId],
    ctrl: QubitId,
    target: QubitId,
    compare_bits: usize,
) {
    assert_eq!(u.len(), v.len());
    assert!(!u.is_empty());
    let compare_bits = compare_bits.min(u.len()).max(1);
    let start = u.len() - compare_bits;
    ccx_cmp_lt_into_fast(b, &v[start..], &u[start..], ctrl, target);
}

pub(crate) fn dialog_gcd_branch_bits_host_comparator_enabled() -> bool {
    std::env::var("DIALOG_GCD_BRANCH_BITS_HOST_COMPARATOR")
        .ok()
        .as_deref()
        == Some("1")
}

/// Truncated controlled branch-bit comparator that hosts its borrow `c_in` +
/// `carries` transient on a borrowed clean slice (the idle future-log region)
/// when one of sufficient length is supplied, freeing the peak qubit the fresh
/// allocation would otherwise consume at the branch_bits instant. Falls back to
/// the self-allocating comparator when no slice (or a too-short one) is given, so
/// behaviour is identical to `dialog_gcd_ccx_cmp_gt_truncated_into_width` in that
/// case. Value-exact either way.
pub(crate) fn dialog_gcd_ccx_cmp_gt_truncated_into_width_hosted(
    b: &mut B,
    u: &[QubitId],
    v: &[QubitId],
    ctrl: QubitId,
    target: QubitId,
    compare_bits: usize,
    borrowed: Option<&[QubitId]>,
) {
    assert_eq!(u.len(), v.len());
    assert!(!u.is_empty());
    let compare_bits = compare_bits.min(u.len()).max(1);
    let start = u.len() - compare_bits;
    let cmp_u = &v[start..];
    let cmp_v = &u[start..];
    let n = cmp_u.len();
    // Need c_in (1) + carries (n) = n+1 clean lanes. PARTIAL hosting: borrow the
    // future-log prefix that fits and allocate only the deficit, instead of
    // all-or-nothing (which fully self-allocs n+1 at the late GCD steps where the
    // slice runs short, pinning the branch_bits peak at 1446). The borrowed-carries
    // comparator indexes c_in and each carries[i] independently, so a gathered
    // [borrowed_prefix ++ owned] vec is value-identical; borrowed lanes are restored
    // to |0> by the measured backward inv-MAJ sweep, owned lanes are freed.
    let need = n + 1;
    let avail = borrowed.map(|s| s.len()).unwrap_or(0);
    if dialog_gcd_partial_host_comparator_enabled() && avail > 0 && avail < need {
        let slice = borrowed.expect("avail>0");
        let owned = b.alloc_qubits(need - avail);
        let mut clean: Vec<QubitId> = Vec::with_capacity(need);
        clean.extend_from_slice(slice);
        clean.extend_from_slice(&owned);
        let (c_in, carries) = clean.split_first().expect("need >= 1");
        ccx_cmp_lt_into_fast_borrowed_carries(b, cmp_u, cmp_v, ctrl, target, *c_in, &carries[..n]);
        b.free_vec(&owned);
    } else if let Some(slice) = borrowed.filter(|s| s.len() >= need) {
        let (c_in, carries) = slice.split_first().expect("slice len >= n+1 > 0");
        ccx_cmp_lt_into_fast_borrowed_carries(b, cmp_u, cmp_v, ctrl, target, *c_in, &carries[..n]);
    } else {
        ccx_cmp_lt_into_fast(b, cmp_u, cmp_v, ctrl, target);
    }
}

pub(crate) fn dialog_gcd_partial_host_comparator_enabled() -> bool {
    std::env::var("DIALOG_GCD_PARTIAL_HOST_COMPARATOR")
        .ok()
        .as_deref()
        != Some("0")
}

pub(crate) fn dialog_gcd_shift_right_assuming_even(b: &mut B, v: &[QubitId]) {
    assert!(!v.is_empty());
    for i in 0..v.len() - 1 {
        b.swap(v[i], v[i + 1]);
    }
}

pub(crate) fn dialog_gcd_unshift_right_assuming_even(b: &mut B, v: &[QubitId]) {
    assert!(!v.is_empty());
    for i in (0..v.len() - 1).rev() {
        b.swap(v[i], v[i + 1]);
    }
}

pub(crate) fn dialog_gcd_width_margin() -> f64 {
    // W-TRUNC safety margin added to the empirical bit-length envelope.
    // Default 37.0 reproduces pldallairedemers' baseline byte-for-byte.
    // Lowering it tightens every GCD-body width (cswap/sub/add) -> fewer
    // Toffoli, peak-neutral (early steps clamp at N). Co-tune with reroll.
    std::env::var("DIALOG_GCD_WIDTH_MARGIN")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|m| m.is_finite() && *m >= 0.0 && *m <= N as f64)
        .unwrap_or(37.0)
}

pub(crate) fn dialog_gcd_width_slope() -> f64 {
    // Per-step shrink rate of the realizable max(bitlen(u),bitlen(v)).
    // Default 0.5*1.415 = 0.7075 reproduces the baseline.
    std::env::var("DIALOG_GCD_WIDTH_SLOPE_X1000")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|s| s.is_finite() && *s > 0.0 && *s <= 4000.0)
        .map(|s| s / 1000.0)
        .unwrap_or(0.5 * 1.415)
}

pub(crate) fn dialog_gcd_tobitvector_active_width(step: usize) -> usize {
    if !dialog_gcd_raw_tobitvector_variable_width_enabled() {
        return N;
    }
    let ideal = N as f64 - (step as f64) * dialog_gcd_width_slope() + dialog_gcd_width_margin();
    let rounded = ((ideal.max(1.0) / 2.0).ceil() as usize) * 2;
    rounded.clamp(1, N)
}

/// Carry-tail truncation window for the materialized controlled sub/add BODY
/// (and its gated LOAD). Default 0 (OFF). When `w > 0`, the controlled
/// `acc -= ctrl·subtrahend` / `acc += ctrl·addend` only loads + ripples the
/// low `active_width - w` bits. The GCD work registers u/v are bounded by the
/// realizable bitlen, which sits `WIDTH_MARGIN` (=28) bits below `active_width`,
/// so the top `w <= margin` bits of both operands are 0 in the no-truncation
/// regime: the gated LOAD there is `ctrl & 0 = 0` and the body's top carries
/// are 0, so neither the load nor the carry ripple above `active_width - w`
/// affects the result. Failure mode (a step whose realizable bitlen actually
/// reaches into the truncated window) is selected away by the co-tuned reroll,
/// exactly like the global WIDTH_MARGIN — but applied to the sub/add ONLY,
/// leaving the cswap and comparator at full active_width. Returns the truncated
/// body width, clamped to >= 2.
pub(crate) fn dialog_gcd_body_carry_trunc_width(active_width: usize) -> usize {
    let w = std::env::var("DIALOG_GCD_BODY_CARRY_TRUNC_W")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    active_width.saturating_sub(w).max(2)
}

pub(crate) fn dialog_gcd_host_gated_enabled() -> bool {
    // Port of our KAL_GZ_EARLY_RECOVER carry-pool relocation: host the
    // materialized `gated` register (width = active_width, up to 256 at peak)
    // on the provably-|0> future-log slots that already host the ripple carry,
    // instead of allocating fresh ancilla. The borrowed slice (when long enough
    // for carry + gated = 2n-1) is split: [..n-1] = carry, [n-1..2n-1] = gated.
    // Both are restored to |0> (carry by the adder, gated by measurement-clear),
    // so the future-log slots are clean for the future blocks that own them.
    // Peak-neutral->down: removes the +256 fresh ancilla at the GCD-body peak.
    // Default off = byte-identical baseline.
    std::env::var("DIALOG_GCD_HOST_GATED").ok().as_deref() == Some("1")
}

pub(crate) fn dialog_gcd_body_host_cin_enabled() -> bool {
    // When the odd-u low-bit fastpath is active (body_start>=1), the low gated
    // slot gated[0] is never loaded or cleared, so it stays |0> across the body
    // and is distinct from the operands and the borrowed carry lane. Hosting the
    // Cuccaro carry-in there instead of a fresh alloc removes the single qubit
    // that pinned the materialized add/sub BODY one slot above the marker tier.
    // Value-exact (c_in=0 is the carry-in either way; returned to |0>).
    std::env::var("DIALOG_GCD_BODY_HOST_CIN").ok().as_deref() == Some("1")
}

pub(crate) fn dialog_gcd_late_borrow_uv_high_enabled() -> bool {
    std::env::var("DIALOG_GCD_LATE_BORROW_UV_HIGH").ok().as_deref() == Some("1")
}

/// Pick the carry/gated borrow slice for a GCD step. Prefer the compressed
/// future-log; when it is too short to host the full gated(n)+carry(n-1) lane
/// (late steps, where the compressed future region has shrunk), fall back to the
/// high zero bits of `u`. By the same premise the width truncation relies on,
/// `u < 2^active_width` here so `u[active_width..]` is |0>; it is already
/// allocated, so borrowing it as scratch is peak-neutral and adds no failure
/// modes (any input with nonzero u-high already fails the truncation). The
/// returned slice is disjoint from `u[..active_width]` and the `v` accumulator.
pub(crate) fn dialog_gcd_pick_borrow_slice<'a>(
    future: Option<&'a [QubitId]>,
    u: &'a [QubitId],
    active_width: usize,
) -> Option<&'a [QubitId]> {
    if dialog_gcd_late_borrow_uv_high_enabled() && active_width >= 1 {
        let want = 2 * active_width - 1;
        let short = future.map_or(true, |s| s.len() < want);
        if short && u.len() >= active_width + want {
            return Some(&u[active_width..active_width + want]);
        }
    }
    future
}

pub(crate) fn dialog_gcd_controlled_sub_selected(
    b: &mut B,
    subtrahend: &[QubitId],
    acc: &[QubitId],
    ctrl: QubitId,
    borrowed_carries: Option<&[QubitId]>,
) {
    assert_eq!(subtrahend.len(), acc.len());
    assert!(!subtrahend.is_empty());
    if dialog_gcd_raw_tobitvector_materialized_sub_enabled() {
        let n = subtrahend.len();
        // Host the gated register on the tail of the borrowed clean slice when
        // it is long enough for both carry (n-1) and gated (n).
        let gated_host: Option<&[QubitId]> = if dialog_gcd_host_gated_enabled() {
            borrowed_carries.and_then(|c| {
                if c.len() >= 2 * n - 1 {
                    Some(&c[n - 1..2 * n - 1])
                } else {
                    None
                }
            })
        } else {
            None
        };
        let mut gated_owned: Vec<QubitId> = Vec::new();
        let gated: &[QubitId] = match gated_host {
            Some(h) => h,
            None => {
                gated_owned = b.alloc_qubits(n);
                gated_owned.as_slice()
            }
        };
        let body_w = dialog_gcd_body_carry_trunc_width(n);
        let odd_lowbit_fast = dialog_gcd_odd_u_lowbit_fastpath_enabled();
        let body_start = if odd_lowbit_fast { 1 } else { 0 };
        b.set_phase("dialog_gcd_raw_tobitvector_materialized_sub_load");
        for i in body_start..body_w {
            b.ccx(ctrl, subtrahend[i], gated[i]);
        }
        if odd_lowbit_fast {
            // Reachable GCD states have subtrahend[0]=1 and acc[0]=ctrl here:
            // ctrl - ctrl has result bit 0 and no borrow into bit 1.
            b.cx(ctrl, acc[0]);
        }
        b.set_phase("dialog_gcd_raw_tobitvector_materialized_sub_body");
        if body_start < body_w {
            let body_len = body_w - body_start;
            if let Some(carries) =
                borrowed_carries.filter(|carries| carries.len() >= body_len.saturating_sub(1))
            {
                if dialog_gcd_body_host_cin_enabled() && body_start >= 1 {
                    // gated[0] is unused (load/clear start at body_start) and |0>:
                    // use it as the Cuccaro carry-in, dropping the fresh c_in alloc.
                    cuccaro_sub_fast_borrowed_carries(
                        b,
                        &gated[body_start..body_w],
                        &acc[body_start..body_w],
                        gated[0],
                        &carries[..body_len.saturating_sub(1)],
                    );
                } else {
                    sub_nbit_qq_fast_borrowed_carries(
                        b,
                        &gated[body_start..body_w],
                        &acc[body_start..body_w],
                        &carries[..body_len.saturating_sub(1)],
                    );
                }
            } else {
                sub_nbit_qq_fast(b, &gated[body_start..body_w], &acc[body_start..body_w]);
            }
        }
        b.set_phase("dialog_gcd_raw_tobitvector_materialized_sub_clear");
        for i in body_start..body_w {
            let m = b.alloc_bit();
            b.hmr(gated[i], m);
            b.cz_if(ctrl, subtrahend[i], m);
        }
        if gated_host.is_none() {
            b.free_vec(&gated_owned);
        }
    } else {
        cucc_sub_ctrl_lowq(b, subtrahend, acc, ctrl);
    }
}

pub(crate) fn dialog_gcd_controlled_add_selected(
    b: &mut B,
    addend: &[QubitId],
    acc: &[QubitId],
    ctrl: QubitId,
    borrowed_carries: Option<&[QubitId]>,
) {
    assert_eq!(addend.len(), acc.len());
    assert!(!addend.is_empty());
    if dialog_gcd_raw_tobitvector_materialized_sub_enabled() {
        let n = addend.len();
        let gated_host: Option<&[QubitId]> = if dialog_gcd_host_gated_enabled() {
            borrowed_carries.and_then(|c| {
                if c.len() >= 2 * n - 1 {
                    Some(&c[n - 1..2 * n - 1])
                } else {
                    None
                }
            })
        } else {
            None
        };
        let mut gated_owned: Vec<QubitId> = Vec::new();
        let gated: &[QubitId] = match gated_host {
            Some(h) => h,
            None => {
                gated_owned = b.alloc_qubits(n);
                gated_owned.as_slice()
            }
        };
        let body_w = dialog_gcd_body_carry_trunc_width(n);
        let odd_lowbit_fast = dialog_gcd_odd_u_lowbit_fastpath_enabled();
        let body_start = if odd_lowbit_fast { 1 } else { 0 };
        b.set_phase("dialog_gcd_raw_tobitvector_materialized_add_load");
        for i in body_start..body_w {
            b.ccx(ctrl, addend[i], gated[i]);
        }
        if odd_lowbit_fast {
            // In reverse, acc[0] is zero after unshift and addend[0]=1:
            // adding ctrl sets the low result bit with no carry into bit 1.
            b.cx(ctrl, acc[0]);
        }
        b.set_phase("dialog_gcd_raw_tobitvector_materialized_add_body");
        if body_start < body_w {
            let body_len = body_w - body_start;
            if let Some(carries) =
                borrowed_carries.filter(|carries| carries.len() >= body_len.saturating_sub(1))
            {
                if dialog_gcd_body_host_cin_enabled() && body_start >= 1 {
                    // gated[0] is unused (load/clear start at body_start) and |0>:
                    // use it as the Cuccaro carry-in, dropping the fresh c_in alloc.
                    cuccaro_add_fast_borrowed_carries(
                        b,
                        &gated[body_start..body_w],
                        &acc[body_start..body_w],
                        gated[0],
                        &carries[..body_len.saturating_sub(1)],
                    );
                } else {
                    add_nbit_qq_fast_borrowed_carries(
                        b,
                        &gated[body_start..body_w],
                        &acc[body_start..body_w],
                        &carries[..body_len.saturating_sub(1)],
                    );
                }
            } else {
                add_nbit_qq_fast(b, &gated[body_start..body_w], &acc[body_start..body_w]);
            }
        }
        b.set_phase("dialog_gcd_raw_tobitvector_materialized_add_clear");
        for i in body_start..body_w {
            let m = b.alloc_bit();
            b.hmr(gated[i], m);
            b.cz_if(ctrl, addend[i], m);
        }
        if gated_host.is_none() {
            b.free_vec(&gated_owned);
        }
    } else {
        cucc_add_ctrl_lowq(b, addend, acc, ctrl);
    }
}

pub(crate) fn dialog_gcd_future_log_carry_slice(
    dialog_log: &[QubitId],
    step: usize,
    active_width: usize,
) -> Option<&[QubitId]> {
    if !dialog_gcd_raw_tobitvector_borrow_future_log_carries_enabled() {
        return None;
    }
    let carry_need = active_width.saturating_sub(1);
    let want = if dialog_gcd_host_gated_enabled() {
        2 * active_width - 1
    } else {
        carry_need
    };
    let start = 2 * (step + 1);
    dialog_log
        .get(start..)
        .filter(|future| future.len() >= carry_need)
        .map(|future| &future[..future.len().min(want)])
}

pub(crate) fn dialog_gcd_cmod_add_pseudomersenne_lowq(
    b: &mut B,
    acc: &[QubitId],
    a: &[QubitId],
    ctrl: QubitId,
    p: U256,
) {
    assert_eq!(acc.len(), N);
    assert_eq!(a.len(), N);
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1u64));

    let (acc_ext, acc_ovf) = ext_reg(b, acc);
    let a_ovf = b.alloc_qubit();
    let mut a_ext = a.to_vec();
    a_ext.push(a_ovf);
    let c_in = b.alloc_qubit();
    let scratch = b.alloc_qubit();

    b.set_phase("dialog_gcd_direct_special_cadd_raw_sum");
    cuccaro_add_ctrl_lowq(b, &a_ext, &acc_ext, ctrl, c_in, scratch);
    b.free(scratch);
    b.free(c_in);
    b.free(a_ovf);

    // If the controlled 256-bit add overflowed, subtract p by adding
    // c = 2^256 - p to the low word.  The low slice is the explicit
    // approximation knob: carry beyond this window is treated as a rare
    // arithmetic failure branch, not as phase dirt.
    b.set_phase("dialog_gcd_direct_special_overflow_fold");
    cadd_nbit_const_fast(b, &acc[..DIALOG_GCD_SPECIAL_ADD_LSBS], c, acc_ovf);

    // For successful branches this is the exact overflow cleanup identity:
    // after subtracting p, the final low word is smaller than the addend iff
    // the overflow branch happened.  The omitted no-overflow sum>=p case is
    // the approximation budgeted by the caller.
    b.set_phase("dialog_gcd_direct_special_overflow_clean");
    cmp_lt_into(b, acc, a, acc_ovf);
    unext_reg(b, acc_ovf);
}

pub(crate) fn dialog_gcd_cmod_add_materialized_pseudomersenne(
    b: &mut B,
    acc: &[QubitId],
    a: &[QubitId],
    ctrl: QubitId,
    p: U256,
) {
    dialog_gcd_cmod_add_materialized_pseudomersenne_with_clean_scratch(b, acc, a, ctrl, p, &[]);
}
pub(crate) fn dialog_gcd_measured_apply_sub_enabled() -> bool {
    std::env::var("DIALOG_GCD_MEASURED_APPLY_SUB")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_apply_window_blocks() -> Option<usize> {
    std::env::var("DIALOG_GCD_APPLY_WINDOW_BLOCKS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&w| w >= 2)
}

pub(crate) fn dialog_gcd_clean_truncated_underflow(
    b: &mut B,
    acc: &[QubitId],
    a: &[QubitId],
    ctrl: QubitId,
    acc_ovf: QubitId,
) {
    let compare_start = N - dialog_gcd_apply_clean_compare_bits();
    for &q in &a[compare_start..] {
        b.x(q);
    }
    b.cx(ctrl, acc_ovf);
    ccx_cmp_lt_into_fast(b, &acc[compare_start..], &a[compare_start..], ctrl, acc_ovf);
    for &q in &a[compare_start..] {
        b.x(q);
    }
}

pub(crate) fn dialog_gcd_load_controlled_slice(
    b: &mut B,
    ctrl: QubitId,
    source: &[QubitId],
    lo: usize,
    hi: usize,
) -> Vec<QubitId> {
    assert!(lo <= hi);
    assert!(hi <= source.len());
    let out = b.alloc_qubits(hi - lo);
    for (i, &q) in source[lo..hi].iter().enumerate() {
        b.ccx(ctrl, q, out[i]);
    }
    out
}

pub(crate) fn dialog_gcd_clear_controlled_slice_hmr(
    b: &mut B,
    ctrl: QubitId,
    source: &[QubitId],
    lo: usize,
    loaded: &[QubitId],
) {
    assert!(lo + loaded.len() <= source.len());
    for (i, &q) in loaded.iter().enumerate() {
        let m = b.alloc_bit();
        b.hmr(q, m);
        b.cz_if(ctrl, source[lo + i], m);
    }
}

pub(crate) fn dialog_gcd_chunk_hi(blocks: usize, block: usize, ext_n: usize) -> usize {
    if blocks == 4 && dialog_gcd_apply_chunked_f_custom4_enabled() {
        let cuts = [
            dialog_gcd_apply_chunked_f_cut().unwrap_or(ext_n / 4),
            dialog_gcd_apply_chunked_f_cut2().unwrap_or(ext_n / 2),
            dialog_gcd_apply_chunked_f_cut3().unwrap_or(3 * ext_n / 4),
        ];
        assert!(
            cuts[0] < cuts[1] && cuts[1] < cuts[2] && cuts[2] < ext_n,
            "custom four-chunk apply boundaries must be strictly increasing and below {ext_n}: {cuts:?}"
        );
        if block < cuts.len() {
            return cuts[block];
        }
    }
    if block == 0 && blocks <= 3 {
        return dialog_gcd_apply_chunked_f_cut().unwrap_or(ext_n / 2).min(ext_n - 1);
    }
    if blocks == 3 && block == 1 {
        return dialog_gcd_apply_chunked_f_cut2()
            .unwrap_or(2 * ext_n / 3)
            .min(ext_n - 1);
    }
    ((block + 1) * ext_n) / blocks
}

// ─── merged from misc2.rs ───

pub(crate) fn dialog_gcd_add_ctrl_chunked_low_to_ext(
    b: &mut B,
    source: &[QubitId],
    acc_ext: &[QubitId],
    ctrl: QubitId,
    c_in: QubitId,
    blocks: usize,
    clean_scratch: &[QubitId],
) {
    let n = source.len();
    assert_eq!(acc_ext.len(), n + 1);
    for (i, &q) in clean_scratch.iter().enumerate() {
        assert!(!clean_scratch[..i].contains(&q));
        assert!(!source.contains(&q));
        assert!(!acc_ext.contains(&q));
        assert_ne!(q, ctrl);
        assert_ne!(q, c_in);
    }
    let ext_n = acc_ext.len();
    let blocks = blocks.max(2).min(ext_n);
    let mut carry = c_in;
    let mut lo = 0usize;
    // Reserve the first borrowed cell as the transient high-zero lane.  It is
    // restored after every chunk and may be reused if REUSE_CIN_ZERO=0.  The
    // remaining cells can hold dirty boundary carries until the exact
    // cumulative comparator sweep clears them.
    let zero_host = clean_scratch.first().copied();
    let boundary_hosts = &clean_scratch[usize::from(zero_host.is_some())..];
    let mut couts: Vec<(QubitId, usize, bool)> = Vec::new();

    for blk in 0..blocks {
        let hi = dialog_gcd_chunk_hi(blocks, blk, ext_n);
        if hi <= lo {
            continue;
        }
        if blk == blocks - 1 || hi == ext_n {
            let f = dialog_gcd_load_controlled_slice(b, ctrl, source, lo.min(n), n);
            cuccaro_add_fast_low_to_ext(b, &f, &acc_ext[lo..hi], carry);
            dialog_gcd_clear_controlled_slice_hmr(b, ctrl, source, lo.min(n), &f);
            b.free_vec(&f);
            break;
        }

        assert!(hi <= n);
        let f = dialog_gcd_load_controlled_slice(b, ctrl, source, lo, hi);
        let needs_distinct_zero =
            carry == c_in || !dialog_gcd_apply_chunked_f_reuse_cin_zero_enabled();
        let (zero, owned_zero) = if needs_distinct_zero {
            zero_host.map_or_else(|| (b.alloc_qubit(), true), |q| (q, false))
        } else {
            (c_in, false)
        };
        let (cout, owned_cout) = boundary_hosts
            .get(couts.len())
            .copied()
            .map_or_else(|| (b.alloc_qubit(), true), |q| (q, false));
        let mut a_block = f.clone();
        a_block.push(zero);
        let mut acc_block = acc_ext[lo..hi].to_vec();
        acc_block.push(cout);
        cuccaro_add_fast(b, &a_block, &acc_block, carry);
        if owned_zero {
            b.free(zero);
        }
        dialog_gcd_clear_controlled_slice_hmr(b, ctrl, source, lo, &f);
        b.free_vec(&f);
        couts.push((cout, hi, owned_cout));
        carry = cout;
        lo = hi;
    }

    if dialog_gcd_apply_chunked_f_fuse_boundary_clears_enabled() {
        if let Some(&(_, p, _)) = couts.last() {
            let targets = couts
                .iter()
                .map(|&(cout, p, _)| (cout, p))
                .collect::<Vec<_>>();
            ccx_cmp_lt_into_fast_prefix_targets(
                b,
                &acc_ext[..p],
                &source[..p],
                ctrl,
                &targets,
            );
        }
    } else {
        for &(cout, p, _) in couts.iter().rev() {
            ccx_cmp_lt_into_fast(b, &acc_ext[..p], &source[..p], ctrl, cout);
        }
    }
    for &(cout, _, owned_cout) in couts.iter().rev() {
        if owned_cout {
            b.free(cout);
        }
    }
}
pub(crate) fn dialog_gcd_sub_ctrl_chunked_low_to_ext(
    b: &mut B,
    source: &[QubitId],
    acc_ext: &[QubitId],
    ctrl: QubitId,
    c_in: QubitId,
    blocks: usize,
    clean_scratch: &[QubitId],
) {
    let n = source.len();
    assert_eq!(acc_ext.len(), n + 1);
    for (i, &q) in clean_scratch.iter().enumerate() {
        assert!(!clean_scratch[..i].contains(&q));
        assert!(!source.contains(&q));
        assert!(!acc_ext.contains(&q));
        assert_ne!(q, ctrl);
        assert_ne!(q, c_in);
    }
    let ext_n = acc_ext.len();
    let blocks = blocks.max(2).min(ext_n);
    let mut borrow = c_in;
    let mut lo = 0usize;
    // Symmetric to the add path: reserve one clean transient high-zero host and
    // retain borrowed boundary-borrow cells until their comparator clear.
    let zero_host = clean_scratch.first().copied();
    let boundary_hosts = &clean_scratch[usize::from(zero_host.is_some())..];
    let mut bouts: Vec<(QubitId, usize, bool)> = Vec::new();

    for blk in 0..blocks {
        let hi = dialog_gcd_chunk_hi(blocks, blk, ext_n);
        if hi <= lo {
            continue;
        }
        if blk == blocks - 1 || hi == ext_n {
            let f = dialog_gcd_load_controlled_slice(b, ctrl, source, lo.min(n), n);
            cuccaro_sub_fast_low_to_ext(b, &f, &acc_ext[lo..hi], borrow);
            dialog_gcd_clear_controlled_slice_hmr(b, ctrl, source, lo.min(n), &f);
            b.free_vec(&f);
            break;
        }

        assert!(hi <= n);
        let f = dialog_gcd_load_controlled_slice(b, ctrl, source, lo, hi);
        let needs_distinct_zero =
            borrow == c_in || !dialog_gcd_apply_chunked_f_reuse_cin_zero_enabled();
        let (zero, owned_zero) = if needs_distinct_zero {
            zero_host.map_or_else(|| (b.alloc_qubit(), true), |q| (q, false))
        } else {
            (c_in, false)
        };
        let (bout, owned_bout) = boundary_hosts
            .get(bouts.len())
            .copied()
            .map_or_else(|| (b.alloc_qubit(), true), |q| (q, false));
        let mut a_block = f.clone();
        a_block.push(zero);
        let mut acc_block = acc_ext[lo..hi].to_vec();
        acc_block.push(bout);
        cuccaro_sub_fast(b, &a_block, &acc_block, borrow);
        if owned_zero {
            b.free(zero);
        }
        dialog_gcd_clear_controlled_slice_hmr(b, ctrl, source, lo, &f);
        b.free_vec(&f);
        bouts.push((bout, hi, owned_bout));
        borrow = bout;
        lo = hi;
    }

    if dialog_gcd_apply_chunked_f_fuse_boundary_clears_enabled() {
        if let Some(&(_, p, _)) = bouts.last() {
            for i in 0..p {
                b.x(source[i]);
            }
            let targets = bouts
                .iter()
                .map(|&(bout, p, _)| (bout, p))
                .collect::<Vec<_>>();
            ccx_cmp_lt_into_fast_prefix_targets(
                b,
                &source[..p],
                &acc_ext[..p],
                ctrl,
                &targets,
            );
            for i in 0..p {
                b.x(source[i]);
            }
        }
    } else {
        for &(bout, p, _) in bouts.iter().rev() {
            for i in 0..p {
                b.x(source[i]);
            }
            ccx_cmp_lt_into_fast(b, &source[..p], &acc_ext[..p], ctrl, bout);
            for i in 0..p {
                b.x(source[i]);
            }
        }
    }
    for &(bout, _, owned_bout) in bouts.iter().rev() {
        if owned_bout {
            b.free(bout);
        }
    }
}
pub(crate) fn dialog_gcd_cmod_add_materialized_pseudomersenne_chunked(
    b: &mut B,
    acc: &[QubitId],
    a: &[QubitId],
    ctrl: QubitId,
    p: U256,
    blocks: usize,
    clean_scratch: &[QubitId],
) {
    assert_eq!(acc.len(), N);
    assert_eq!(a.len(), N);
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1u64));

    let (acc_ext, acc_ovf) = ext_reg(b, acc);
    for (i, &q) in clean_scratch.iter().enumerate() {
        assert!(!clean_scratch[..i].contains(&q));
        assert!(!acc_ext.contains(&q));
        assert!(!a.contains(&q));
        assert_ne!(q, ctrl);
    }
    let (c_in, owned_c_in, inner_scratch) = clean_scratch.split_first().map_or_else(
        || (b.alloc_qubit(), true, &[][..]),
        |(&q, rest)| (q, false, rest),
    );

    b.set_phase("dialog_gcd_materialized_special_chunked_raw_sum");
    dialog_gcd_add_ctrl_chunked_low_to_ext(b, a, &acc_ext, ctrl, c_in, blocks, inner_scratch);
    if owned_c_in {
        b.free(c_in);
    }

    b.set_phase("dialog_gcd_materialized_special_overflow_fold");
    if let Some(w) = fold_carry_trunc_window() {
        cadd_nbit_const_direct_trunc_fast(b, &acc[..DIALOG_GCD_SPECIAL_ADD_LSBS], c, acc_ovf, w);
    } else {
        cadd_nbit_const_fast(b, &acc[..DIALOG_GCD_SPECIAL_ADD_LSBS], c, acc_ovf);
    }

    b.set_phase("dialog_gcd_materialized_special_overflow_clean");
    let compare_start = N - dialog_gcd_apply_clean_compare_bits();
    ccx_cmp_lt_into_fast(b, &acc[compare_start..], &a[compare_start..], ctrl, acc_ovf);
    unext_reg(b, acc_ovf);
}
pub(crate) fn dialog_gcd_cmod_sub_materialized_pseudomersenne_chunked(
    b: &mut B,
    acc: &[QubitId],
    a: &[QubitId],
    ctrl: QubitId,
    p: U256,
    blocks: usize,
    clean_scratch: &[QubitId],
) {
    assert_eq!(acc.len(), N);
    assert_eq!(a.len(), N);
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1u64));

    let (acc_ext, acc_ovf) = ext_reg(b, acc);
    for (i, &q) in clean_scratch.iter().enumerate() {
        assert!(!clean_scratch[..i].contains(&q));
        assert!(!acc_ext.contains(&q));
        assert!(!a.contains(&q));
        assert_ne!(q, ctrl);
    }
    let (c_in, owned_c_in, inner_scratch) = clean_scratch.split_first().map_or_else(
        || (b.alloc_qubit(), true, &[][..]),
        |(&q, rest)| (q, false, rest),
    );

    b.set_phase("dialog_gcd_materialized_special_chunked_raw_difference");
    dialog_gcd_sub_ctrl_chunked_low_to_ext(b, a, &acc_ext, ctrl, c_in, blocks, inner_scratch);
    if owned_c_in {
        b.free(c_in);
    }

    b.set_phase("dialog_gcd_materialized_special_underflow_fold");
    if let Some(w) = fold_carry_trunc_window() {
        csub_nbit_const_direct_trunc_fast(b, &acc[..DIALOG_GCD_SPECIAL_ADD_LSBS], c, acc_ovf, w);
    } else {
        csub_nbit_const_fast(b, &acc[..DIALOG_GCD_SPECIAL_ADD_LSBS], c, acc_ovf);
    }

    b.set_phase("dialog_gcd_materialized_special_underflow_clean");
    dialog_gcd_clean_truncated_underflow(b, acc, a, ctrl, acc_ovf);
    unext_reg(b, acc_ovf);
}
pub(crate) fn dialog_gcd_cmod_sub_materialized_pseudomersenne(
    b: &mut B,
    acc: &[QubitId],
    a: &[QubitId],
    ctrl: QubitId,
    p: U256,
) {
    dialog_gcd_cmod_sub_materialized_pseudomersenne_with_clean_scratch(b, acc, a, ctrl, p, &[]);
}
pub(crate) fn dialog_gcd_cmod_sub_materialized_pseudomersenne_borrowed_subtrahend(
    b: &mut B,
    acc: &[QubitId],
    a: &[QubitId],
    ctrl: QubitId,
    p: U256,
    f: &[QubitId],
) {
    assert_eq!(acc.len(), N);
    assert_eq!(a.len(), N);
    assert_eq!(f.len(), N);
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1u64));

    b.set_phase("dialog_gcd_materialized_special_borrowed_load_subtrahend");
    for i in 0..N {
        b.ccx(ctrl, a[i], f[i]);
    }

    let (acc_ext, acc_ovf) = ext_reg(b, acc);
    let f_ovf = b.alloc_qubit();
    let mut f_ext = f.to_vec();
    f_ext.push(f_ovf);

    b.set_phase("dialog_gcd_materialized_special_borrowed_raw_difference");
    sub_nbit_qq(b, &f_ext, &acc_ext);
    b.free(f_ovf);

    b.set_phase("dialog_gcd_materialized_special_borrowed_underflow_fold");
    if let Some(w) = fold_carry_trunc_window() {
        csub_nbit_const_direct_trunc_fast(b, &acc[..DIALOG_GCD_SPECIAL_ADD_LSBS], c, acc_ovf, w);
    } else {
        csub_nbit_const_fast(b, &acc[..DIALOG_GCD_SPECIAL_ADD_LSBS], c, acc_ovf);
    }

    b.set_phase("dialog_gcd_materialized_special_borrowed_underflow_clean");
    if dialog_gcd_raw_apply_truncated_clean_enabled() {
        dialog_gcd_clean_truncated_underflow(b, acc, a, ctrl, acc_ovf);
    } else {
        b.x(acc_ovf);
        mod_neg_inplace_fast(b, f, p);
        cmp_lt_into_fast(b, acc, f, acc_ovf);
        mod_neg_inplace_fast(b, f, p);
    }
    unext_reg(b, acc_ovf);

    b.set_phase("dialog_gcd_materialized_special_borrowed_clear_subtrahend");
    for i in (0..N).rev() {
        b.ccx(ctrl, a[i], f[i]);
    }
}


pub(crate) fn dialog_gcd_cmod_add_materialized_pseudomersenne_with_clean_scratch(
    b: &mut B,
    acc: &[QubitId],
    a: &[QubitId],
    ctrl: QubitId,
    p: U256,
    clean_scratch: &[QubitId],
) {
    assert_eq!(acc.len(), N);
    assert_eq!(a.len(), N);
    if let Some(blocks) = dialog_gcd_apply_chunked_f_blocks()
        .filter(|_| dialog_gcd_raw_apply_truncated_clean_enabled())
    {
        dialog_gcd_cmod_add_materialized_pseudomersenne_chunked(
            b,
            acc,
            a,
            ctrl,
            p,
            blocks,
            clean_scratch,
        );
        return;
    }
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1u64));

    let f = b.alloc_qubits(N);
    b.set_phase("dialog_gcd_materialized_special_load_addend");
    for i in 0..N {
        b.ccx(ctrl, a[i], f[i]);
    }

    let (acc_ext, acc_ovf) = ext_reg(b, acc);
    let c_in = b.alloc_qubit();

    b.set_phase("dialog_gcd_materialized_special_raw_sum");
    if let Some(w) = dialog_gcd_apply_window_blocks() {
        cuccaro_add_fast_windowed_low_to_ext(b, &f, &acc_ext, c_in, w);
    } else {
        let f_ovf = b.alloc_qubit();
        let mut f_ext = f.clone();
        f_ext.push(f_ovf);
        cuccaro_add_fast(b, &f_ext, &acc_ext, c_in);
        b.free(f_ovf);
    }
    b.free(c_in);

    b.set_phase("dialog_gcd_materialized_special_overflow_fold");
    if let Some(w) = fold_carry_trunc_window() {
        cadd_nbit_const_direct_trunc_fast(b, &acc[..DIALOG_GCD_SPECIAL_ADD_LSBS], c, acc_ovf, w);
    } else {
        cadd_nbit_const_fast(b, &acc[..DIALOG_GCD_SPECIAL_ADD_LSBS], c, acc_ovf);
    }

    b.set_phase("dialog_gcd_materialized_special_overflow_clean");
    if dialog_gcd_raw_apply_truncated_clean_enabled() {
        let compare_start = N - dialog_gcd_apply_clean_compare_bits();
        cmp_lt_into_fast(b, &acc[compare_start..], &f[compare_start..], acc_ovf);
    } else {
        cmp_lt_into(b, acc, &f, acc_ovf);
    }
    unext_reg(b, acc_ovf);

    b.set_phase("dialog_gcd_materialized_special_clear_addend");
    for i in 0..N {
        let m = b.alloc_bit();
        b.hmr(f[i], m);
        b.cz_if(ctrl, a[i], m);
    }
    b.free_vec(&f);
}

pub(crate) fn dialog_gcd_cmod_sub_materialized_pseudomersenne_with_clean_scratch(
    b: &mut B,
    acc: &[QubitId],
    a: &[QubitId],
    ctrl: QubitId,
    p: U256,
    clean_scratch: &[QubitId],
) {
    assert_eq!(acc.len(), N);
    assert_eq!(a.len(), N);
    if let Some(blocks) = dialog_gcd_apply_chunked_f_blocks()
        .filter(|_| dialog_gcd_raw_apply_truncated_clean_enabled())
        .filter(|_| dialog_gcd_measured_apply_sub_enabled())
    {
        dialog_gcd_cmod_sub_materialized_pseudomersenne_chunked(
            b,
            acc,
            a,
            ctrl,
            p,
            blocks,
            clean_scratch,
        );
        return;
    }
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1u64));

    let f = b.alloc_qubits(N);
    b.set_phase("dialog_gcd_materialized_special_load_subtrahend");
    for i in 0..N {
        b.ccx(ctrl, a[i], f[i]);
    }

    let (acc_ext, acc_ovf) = ext_reg(b, acc);

    b.set_phase("dialog_gcd_materialized_special_raw_difference");
    if dialog_gcd_measured_apply_sub_enabled() {
        // Measured (Gidney) difference: ~n Toffoli instead of the ~2n of the
        // non-fast cuccaro_sub uncompute. Peak-safe: the symmetric apply ADD
        // already runs cuccaro_add_fast with its carry lane in this same phase.
        let c_in = b.alloc_qubit();
        if let Some(w) = dialog_gcd_apply_window_blocks() {
            cuccaro_sub_fast_windowed_low_to_ext(b, &f, &acc_ext, c_in, w);
        } else {
            let f_ovf = b.alloc_qubit();
            let mut f_ext = f.clone();
            f_ext.push(f_ovf);
            cuccaro_sub_fast(b, &f_ext, &acc_ext, c_in);
            b.free(f_ovf);
        }
        b.free(c_in);
    } else {
        let f_ovf = b.alloc_qubit();
        let mut f_ext = f.clone();
        f_ext.push(f_ovf);
        sub_nbit_qq(b, &f_ext, &acc_ext);
        b.free(f_ovf);
    }

    b.set_phase("dialog_gcd_materialized_special_underflow_fold");
    if let Some(w) = fold_carry_trunc_window() {
        csub_nbit_const_direct_trunc_fast(b, &acc[..DIALOG_GCD_SPECIAL_ADD_LSBS], c, acc_ovf, w);
    } else {
        csub_nbit_const_fast(b, &acc[..DIALOG_GCD_SPECIAL_ADD_LSBS], c, acc_ovf);
    }

    b.set_phase("dialog_gcd_materialized_special_underflow_clean");
    if dialog_gcd_raw_apply_truncated_clean_enabled() {
        dialog_gcd_clean_truncated_underflow(b, acc, a, ctrl, acc_ovf);
    } else {
        b.x(acc_ovf);
        mod_neg_inplace_fast(b, &f, p);
        cmp_lt_into_fast(b, acc, &f, acc_ovf);
        mod_neg_inplace_fast(b, &f, p);
    }
    unext_reg(b, acc_ovf);

    b.set_phase("dialog_gcd_materialized_special_clear_subtrahend");
    for i in 0..N {
        let m = b.alloc_bit();
        b.hmr(f[i], m);
        b.cz_if(ctrl, a[i], m);
    }
    b.free_vec(&f);
}