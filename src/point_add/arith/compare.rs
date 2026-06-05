
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;


// ═══════════════════════════════════════════════════════════════════════════
//  Conditional modular add/sub helpers
// ═══════════════════════════════════════════════════════════════════════════
//
// Used by the multipliers. Each variant loads `(ctrl ? a : 0)` into a
// fresh temporary via CCX or CX_if, runs the unconditional mod_add_qq /
// mod_sub_qq, then unloads.

/// Like `cmp_lt_into` but uses carry-ancilla + measurement-based uncompute
/// for the inv_MAJ sweep. Saves n CCX. NOT emit_inverse-safe.
pub(crate) fn cmp_lt_into_fast(b: &mut B, u: &[QubitId], v: &[QubitId], flag: QubitId) {
    // The vented D1 core uses the slow (no-carries) comparator which
    // saves n peak qubits at cost of ~n CCX per call.
    if kal_vent_modadd_enabled() {
        cmp_lt_into(b, u, v, flag);
        return;
    }
    let n = u.len();
    assert_eq!(n, v.len());
    let c_in = b.alloc_qubit();
    let carries = b.alloc_qubits(n);
    for i in 0..n {
        b.x(u[i]);
    }

    // Forward MAJ sweep with carry ancillae
    b.cx(u[0], v[0]);
    b.cx(u[0], c_in);
    b.ccx(c_in, v[0], carries[0]);
    b.cx(carries[0], u[0]);
    for i in 1..n {
        b.cx(u[i], v[i]);
        b.cx(u[i], u[i - 1]);
        b.ccx(u[i - 1], v[i], carries[i]);
        b.cx(carries[i], u[i]);
    }

    b.cx(u[n - 1], flag);

    // Backward inv_MAJ with measurement
    for i in (1..n).rev() {
        b.cx(carries[i], u[i]);
        let m = b.alloc_bit();
        b.hmr(carries[i], m);
        b.cz_if(u[i - 1], v[i], m);
        b.cx(u[i], u[i - 1]);
        b.cx(u[i], v[i]);
    }
    b.cx(carries[0], u[0]);
    let m0 = b.alloc_bit();
    b.hmr(carries[0], m0);
    b.cz_if(c_in, v[0], m0);
    b.cx(u[0], c_in);
    b.cx(u[0], v[0]);

    for i in 0..n {
        b.x(u[i]);
    }
    b.free_vec(&carries);
    b.free(c_in);
}

pub(crate) fn ccx_cmp_lt_into_fast(b: &mut B, u: &[QubitId], v: &[QubitId], ctrl: QubitId, target: QubitId) {
    if kal_vent_modadd_enabled() {
        let flag = b.alloc_qubit();
        cmp_lt_into(b, u, v, flag);
        b.ccx(ctrl, flag, target);
        cmp_lt_into(b, u, v, flag);
        b.free(flag);
        return;
    }

    let n = u.len();
    assert_eq!(n, v.len());
    let c_in = b.alloc_qubit();
    let carries = b.alloc_qubits(n);
    for i in 0..n {
        b.x(u[i]);
    }

    b.cx(u[0], v[0]);
    b.cx(u[0], c_in);
    b.ccx(c_in, v[0], carries[0]);
    b.cx(carries[0], u[0]);
    for i in 1..n {
        b.cx(u[i], v[i]);
        b.cx(u[i], u[i - 1]);
        b.ccx(u[i - 1], v[i], carries[i]);
        b.cx(carries[i], u[i]);
    }

    b.ccx(ctrl, u[n - 1], target);

    for i in (1..n).rev() {
        b.cx(carries[i], u[i]);
        let m = b.alloc_bit();
        b.hmr(carries[i], m);
        b.cz_if(u[i - 1], v[i], m);
        b.cx(u[i], u[i - 1]);
        b.cx(u[i], v[i]);
    }
    b.cx(carries[0], u[0]);
    let m0 = b.alloc_bit();
    b.hmr(carries[0], m0);
    b.cz_if(c_in, v[0], m0);
    b.cx(u[0], c_in);
    b.cx(u[0], v[0]);

    for i in 0..n {
        b.x(u[i]);
    }
    b.free_vec(&carries);
    b.free(c_in);
}

pub(crate) fn ccx_cmp_lt_into_fast_prefix_targets(
    b: &mut B,
    u: &[QubitId],
    v: &[QubitId],
    ctrl: QubitId,
    targets: &[(QubitId, usize)],
) {
    if targets.is_empty() {
        return;
    }
    if kal_vent_modadd_enabled() {
        for &(target, n) in targets {
            ccx_cmp_lt_into_fast(b, &u[..n], &v[..n], ctrl, target);
        }
        return;
    }

    let n = targets.last().expect("non-empty targets").1;
    assert_eq!(u.len(), n);
    assert_eq!(v.len(), n);
    assert!(n > 0);
    assert!(targets.iter().all(|&(_, p)| (1..=n).contains(&p)));
    assert!(targets.windows(2).all(|w| w[0].1 < w[1].1));

    let c_in = b.alloc_qubit();
    let carries = b.alloc_qubits(n);
    for &q in u {
        b.x(q);
    }

    b.cx(u[0], v[0]);
    b.cx(u[0], c_in);
    b.ccx(c_in, v[0], carries[0]);
    b.cx(carries[0], u[0]);
    let mut next_target = 0;
    while next_target < targets.len() && targets[next_target].1 == 1 {
        b.ccx(ctrl, u[0], targets[next_target].0);
        next_target += 1;
    }
    for i in 1..n {
        b.cx(u[i], v[i]);
        b.cx(u[i], u[i - 1]);
        b.ccx(u[i - 1], v[i], carries[i]);
        b.cx(carries[i], u[i]);
        while next_target < targets.len() && targets[next_target].1 == i + 1 {
            b.ccx(ctrl, u[i], targets[next_target].0);
            next_target += 1;
        }
    }
    assert_eq!(next_target, targets.len());

    for i in (1..n).rev() {
        b.cx(carries[i], u[i]);
        let m = b.alloc_bit();
        b.hmr(carries[i], m);
        b.cz_if(u[i - 1], v[i], m);
        b.cx(u[i], u[i - 1]);
        b.cx(u[i], v[i]);
    }
    b.cx(carries[0], u[0]);
    let m0 = b.alloc_bit();
    b.hmr(carries[0], m0);
    b.cz_if(c_in, v[0], m0);
    b.cx(u[0], c_in);
    b.cx(u[0], v[0]);

    for &q in u {
        b.x(q);
    }
    b.free_vec(&carries);
    b.free(c_in);
}

/// Run `body` with `flag` holding (u < v), then uncompute the flag and
/// restore u, v. Uses carry-ancilla + measurement-based uncomputation
/// for the inv_MAJ sweep (0 Toffoli instead of n CCX).
/// Cost ≈ n CCX (forward MAJ) + body + 0 CCX (measurement inv_MAJ).
pub(crate) fn with_lt<F: FnOnce(&mut B)>(b: &mut B, u: &[QubitId], v: &[QubitId], flag: QubitId, body: F) {
    let n = u.len();
    assert_eq!(n, v.len());
    let c_in = b.alloc_qubit();
    let carries = b.alloc_qubits(n);
    for i in 0..n {
        b.x(u[i]);
    }

    // Forward MAJ sweep with separate carry ancillae.
    // maj_with_carry: CX(w,y); CX(w,x); CCX(x_new,y_new,carry); CX(carry,w)
    // Step 0: (x=c_in, y=v[0], w=u[0])
    b.cx(u[0], v[0]);
    b.cx(u[0], c_in);
    b.ccx(c_in, v[0], carries[0]);
    b.cx(carries[0], u[0]);
    // Steps 1..n-1: (x=u[i-1], y=v[i], w=u[i])
    for i in 1..n {
        b.cx(u[i], v[i]);
        b.cx(u[i], u[i - 1]);
        b.ccx(u[i - 1], v[i], carries[i]);
        b.cx(carries[i], u[i]);
    }

    b.cx(u[n - 1], flag);
    body(b);
    b.cx(u[n - 1], flag);

    // Backward inv_MAJ sweep with measurement-based carry uncompute (0 Toffoli).
    // inv_maj_with_carry: CX(carry,w); HMR+CZ(carry,x,y); CX(w,x); CX(w,y)
    for i in (1..n).rev() {
        b.cx(carries[i], u[i]); // restore w = u[i]
        let m = b.alloc_bit();
        b.hmr(carries[i], m); // measure carry
        b.cz_if(u[i - 1], v[i], m); // phase correction
        b.cx(u[i], u[i - 1]); // restore x = u[i-1]
        b.cx(u[i], v[i]); // restore y = v[i]
    }
    // Step 0: (x=c_in, y=v[0], w=u[0])
    b.cx(carries[0], u[0]);
    let m0 = b.alloc_bit();
    b.hmr(carries[0], m0);
    b.cz_if(c_in, v[0], m0);
    b.cx(u[0], c_in);
    b.cx(u[0], v[0]);

    for i in 0..n {
        b.x(u[i]);
    }
    b.free_vec(&carries);
    b.free(c_in);
}

/// Symmetric helper: runs `body` with `flag` holding (u > v).
pub(crate) fn with_gt<F: FnOnce(&mut B)>(b: &mut B, u: &[QubitId], v: &[QubitId], flag: QubitId, body: F) {
    with_lt(b, v, u, flag, body)
}

pub(crate) fn with_gt_lowq<F: FnOnce(&mut B)>(
    b: &mut B,
    u: &[QubitId],
    v: &[QubitId],
    flag: QubitId,
    body: F,
) {
    cmp_lt_into(b, v, u, flag);
    body(b);
    cmp_lt_into(b, v, u, flag);
}

/// flag ^= (u < v).  Non-destructive on u and v.
///
/// Uses a MAJ-only carry chain instead of the full sub+add pattern.
/// Identity: u < v iff carry-out of (~u + v) = 1, since
///   ~u + v = (2^n - 1 - u) + v = (v - u) + (2^n - 1)
/// which overflows 2^n iff v - u ≥ 1 iff v > u. We negate u in place,
/// run a forward MAJ sweep over (~u, v, c_in=0), capture u[n-1] (which
/// holds the high carry after the chain), then run the inverse MAJ
/// sweep + un-negate to restore u and v. Cost ≈ 2n CCX, half of the
/// previous sub+add (≈ 4n CCX).
pub(crate) fn cmp_lt_into(b: &mut B, u: &[QubitId], v: &[QubitId], flag: QubitId) {
    let n = u.len();
    assert_eq!(n, v.len());

    let c_in = b.alloc_qubit();

    // ~u in place (X is free in the metric).
    for i in 0..n {
        b.x(u[i]);
    }

    // Forward MAJ sweep — n MAJs (one more than cuccaro_add, which omits
    // the top one because it doesn't need the carry-out).
    maj(b, c_in, v[0], u[0]);
    for i in 1..n {
        maj(b, u[i - 1], v[i], u[i]);
    }
    // u[n-1] now holds the high carry = (u < v).
    b.cx(u[n - 1], flag);

    // Inverse sweep restores u and v to their (negated u) state.
    for i in (1..n).rev() {
        inv_maj(b, u[i - 1], v[i], u[i]);
    }
    inv_maj(b, c_in, v[0], u[0]);

    // Un-negate u.
    for i in 0..n {
        b.x(u[i]);
    }

    b.free(c_in);
}

/// out ^= (x OR y). `out` starts 0. Uses the de-Morgan form:
///   x(x); x(y); ccx(x, y, out); x(out); x(y); x(x);
/// After this, out = x OR y (assuming out started at 0). Its inverse is
/// the same gate sequence run in reverse — since it's symmetric (all gates
/// involutions, palindromic structure), running the exact same helper
/// again uncomputes it.
pub(crate) fn or_step(b: &mut B, x: QubitId, y: QubitId, out: QubitId) {
    b.x(x);
    b.x(y);
    b.ccx(x, y, out);
    b.x(out);
    b.x(y);
    b.x(x);
}

// ═══════════════════════════════════════════════════════════════════════════
//  Primitives for the Kaliski port (qrisp-style)
// ═══════════════════════════════════════════════════════════════════════════

/// 2-controlled X with per-control polarity. `polarity=true` means positive
/// control; `false` means anti-control (ctrl=0 triggers).
pub(crate) fn mcx2_polar(b: &mut B, c1: QubitId, p1: bool, c2: QubitId, p2: bool, target: QubitId) {
    if !p1 {
        b.x(c1);
    }
    if !p2 {
        b.x(c2);
    }
    b.ccx(c1, c2, target);
    if !p2 {
        b.x(c2);
    }
    if !p1 {
        b.x(c1);
    }
}

/// 3-controlled X with per-control polarity. Uses a borrowed scratch qubit
/// (must be supplied clean, returns clean).
pub(crate) fn mcx3_polar(
    b: &mut B,
    c1: QubitId,
    p1: bool,
    c2: QubitId,
    p2: bool,
    c3: QubitId,
    p3: bool,
    target: QubitId,
    scratch: QubitId,
) {
    if !p1 {
        b.x(c1);
    }
    if !p2 {
        b.x(c2);
    }
    if !p3 {
        b.x(c3);
    }
    b.ccx(c1, c2, scratch);
    b.ccx(scratch, c3, target);
    b.ccx(c1, c2, scratch);
    if !p3 {
        b.x(c3);
    }
    if !p2 {
        b.x(c2);
    }
    if !p1 {
        b.x(c1);
    }
}

/// Controlled (`target ^= ctrl & (u < v)`) borrow-comparator that takes its
/// `c_in` + `carries` lanes as borrowed clean (|0>) qubits instead of allocating
/// them. Identical gate sequence to `ccx_cmp_lt_into_fast` except the final
/// reduction is `ccx(ctrl, u[n-1], target)` (controlled). The borrowed lanes are
/// restored to |0> by the measured backward inv-MAJ sweep, so the host slice is
/// returned clean (Bennett/measured-clean, safe outside emit_inverse since it
/// uses hmr/cz_if not a recompute). Used by the GCD branch-bit comparator to host
/// its transient on the idle future-log region, freeing the peak qubit it would
/// otherwise allocate at the branch_bits instant.
pub(crate) fn ccx_cmp_lt_into_fast_borrowed_carries(
    b: &mut B,
    u: &[QubitId],
    v: &[QubitId],
    ctrl: QubitId,
    target: QubitId,
    c_in: QubitId,
    carries: &[QubitId],
) {
    let n = u.len();
    assert_eq!(n, v.len());
    assert!(n > 0);
    assert!(carries.len() >= n);

    for i in 0..n {
        b.x(u[i]);
    }

    b.cx(u[0], v[0]);
    b.cx(u[0], c_in);
    b.ccx(c_in, v[0], carries[0]);
    b.cx(carries[0], u[0]);
    for i in 1..n {
        b.cx(u[i], v[i]);
        b.cx(u[i], u[i - 1]);
        b.ccx(u[i - 1], v[i], carries[i]);
        b.cx(carries[i], u[i]);
    }

    b.ccx(ctrl, u[n - 1], target);

    for i in (1..n).rev() {
        b.cx(carries[i], u[i]);
        let m = b.alloc_bit();
        b.hmr(carries[i], m);
        b.cz_if(u[i - 1], v[i], m);
        b.cx(u[i], u[i - 1]);
        b.cx(u[i], v[i]);
    }
    b.cx(carries[0], u[0]);
    let m0 = b.alloc_bit();
    b.hmr(carries[0], m0);
    b.cz_if(c_in, v[0], m0);
    b.cx(u[0], c_in);
    b.cx(u[0], v[0]);

    for i in 0..n {
        b.x(u[i]);
    }
}
