
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

/// Fast Cuccaro add using carry ancillae + measurement-based UMA.
/// Same interface as `cuccaro_add` but uses n-1 carry ancillae so the
/// UMA sweep costs 0 Toffoli (measurement only). NOT emit_inverse-safe.
pub(crate) fn cuccaro_add_fast(b: &mut B, a: &[QubitId], acc: &[QubitId], c_in: QubitId) {
    let n = a.len();
    assert_eq!(n, acc.len());
    if n == 0 {
        return;
    }
    if n == 1 {
        b.cx(c_in, acc[0]);
        b.cx(a[0], acc[0]);
        return;
    }

    let carries = b.alloc_qubits(n - 1);

    // Forward MAJ sweep with carry ancillae.
    // Step 0: MAJ(c_in, acc[0], a[0]) → carry into carries[0]
    b.cx(a[0], acc[0]);
    b.cx(a[0], c_in);
    b.ccx(c_in, acc[0], carries[0]);
    b.cx(carries[0], a[0]);
    // Steps 1..n-2: MAJ(a[i-1], acc[i], a[i]) → carry into carries[i]
    for i in 1..n - 1 {
        b.cx(a[i], acc[i]);
        b.cx(a[i], a[i - 1]);
        b.ccx(a[i - 1], acc[i], carries[i]);
        b.cx(carries[i], a[i]);
    }

    // Final sum bit (same as original cuccaro_add)
    b.cx(a[n - 2], acc[n - 1]);
    b.cx(a[n - 1], acc[n - 1]);

    // Backward UMA sweep with measurement-based carry uncompute (0 Toffoli).
    for i in (1..n - 1).rev() {
        b.cx(carries[i], a[i]);
        let m = b.alloc_bit();
        b.hmr(carries[i], m);
        b.cz_if(a[i - 1], acc[i], m);
        b.cx(a[i], a[i - 1]);
        b.cx(a[i - 1], acc[i]);
    }
    // Step 0 UMA:
    b.cx(carries[0], a[0]);
    let m0 = b.alloc_bit();
    b.hmr(carries[0], m0);
    b.cz_if(c_in, acc[0], m0);
    b.cx(a[0], c_in);
    b.cx(c_in, acc[0]);

    b.free_vec(&carries);
}

/// Same arithmetic as `cuccaro_add_fast`, but the carry lane is supplied by the
/// caller and must be clean on entry.  The HMR uncompute returns it to zero, so
/// Kaliski step4 can reuse clean high `tmp` lanes without increasing peak Q.
pub(crate) fn cuccaro_add_fast_borrowed_carries(
    b: &mut B,
    a: &[QubitId],
    acc: &[QubitId],
    c_in: QubitId,
    carries: &[QubitId],
) {
    let n = a.len();
    assert_eq!(n, acc.len());
    if n == 0 {
        return;
    }
    if n == 1 {
        b.cx(c_in, acc[0]);
        b.cx(a[0], acc[0]);
        return;
    }
    assert!(carries.len() >= n - 1);

    b.cx(a[0], acc[0]);
    b.cx(a[0], c_in);
    b.ccx(c_in, acc[0], carries[0]);
    b.cx(carries[0], a[0]);
    for i in 1..n - 1 {
        b.cx(a[i], acc[i]);
        b.cx(a[i], a[i - 1]);
        b.ccx(a[i - 1], acc[i], carries[i]);
        b.cx(carries[i], a[i]);
    }

    b.cx(a[n - 2], acc[n - 1]);
    b.cx(a[n - 1], acc[n - 1]);

    for i in (1..n - 1).rev() {
        b.cx(carries[i], a[i]);
        let m = b.alloc_bit();
        b.hmr(carries[i], m);
        b.cz_if(a[i - 1], acc[i], m);
        b.cx(a[i], a[i - 1]);
        b.cx(a[i - 1], acc[i]);
    }
    b.cx(carries[0], a[0]);
    let m0 = b.alloc_bit();
    b.hmr(carries[0], m0);
    b.cz_if(c_in, acc[0], m0);
    b.cx(a[0], c_in);
    b.cx(c_in, acc[0]);
}

/// In-place addition `acc += a mod 2^n` on quantum n-bit registers.
/// * `c_in` is a fresh ancilla qubit at 0 on entry and returns to 0.
/// * `a` unchanged; `acc` becomes (a + acc) mod 2^n.
/// Pure mod-2^n: the high carry is discarded (no `z` ancilla). This is
/// honestly reversible because the last MAJ/UMA pair cancel out the
/// carry information on `a[n-1]`.
pub(crate) fn cuccaro_add(b: &mut B, a: &[QubitId], acc: &[QubitId], c_in: QubitId) {
    let n = a.len();
    assert_eq!(n, acc.len());
    if n == 0 {
        return;
    }
    if n == 1 {
        // acc[0] += a[0] + c_in  mod 2 ; c_in → 0
        b.cx(c_in, acc[0]);
        b.cx(a[0], acc[0]);
        return;
    }

    // Forward MAJ sweep.
    maj(b, c_in, acc[0], a[0]);
    for i in 1..n - 1 {
        maj(b, a[i - 1], acc[i], a[i]);
    }

    // Final sum bit: sum[n-1] = acc[n-1] XOR a[n-1] XOR carry_in_to_n-1,
    // where carry_in_to_n-1 is in a[n-2] after the MAJ sweep.
    b.cx(a[n - 2], acc[n - 1]);
    b.cx(a[n - 1], acc[n - 1]);

    // Reverse UMA sweep (skips the final MAJ since we didn't do it).
    for i in (1..n - 1).rev() {
        uma(b, a[i - 1], acc[i], a[i]);
    }
    uma(b, c_in, acc[0], a[0]);
}

/// Reverse of `cuccaro_add`: performs `acc -= a mod 2^n`.
/// Implemented as the exact inverse gate sequence of `cuccaro_add`.
pub(crate) fn cuccaro_sub(b: &mut B, a: &[QubitId], acc: &[QubitId], c_in: QubitId) {
    let n = a.len();
    assert_eq!(n, acc.len());
    if n == 0 {
        return;
    }
    if n == 1 {
        // Inverse of (cx c_in acc; cx a acc) is the same two gates in reverse.
        b.cx(a[0], acc[0]);
        b.cx(c_in, acc[0]);
        return;
    }

    // Inverse of `uma(c_in, acc[0], a[0])`, then the rest of UMA sweep
    // in reverse order.
    inv_uma(b, c_in, acc[0], a[0]);
    for i in 1..n - 1 {
        inv_uma(b, a[i - 1], acc[i], a[i]);
    }

    // Inverse of the final sum writes (both CX self-inverse; reverse order).
    b.cx(a[n - 1], acc[n - 1]);
    b.cx(a[n - 2], acc[n - 1]);

    // Inverse of the forward MAJ sweep.
    for i in (1..n - 1).rev() {
        inv_maj(b, a[i - 1], acc[i], a[i]);
    }
    inv_maj(b, c_in, acc[0], a[0]);
}

// ═══════════════════════════════════════════════════════════════════════════
//  Non-modular n-bit primitives
// ═══════════════════════════════════════════════════════════════════════════

/// Fast Cuccaro sub: `acc -= a mod 2^n` with measurement UMA (0 Toffoli
/// for UMA sweep). Exact gate-level inverse of `cuccaro_add_fast`.
pub(crate) fn cuccaro_sub_fast(b: &mut B, a: &[QubitId], acc: &[QubitId], c_in: QubitId) {
    let n = a.len();
    assert_eq!(n, acc.len());
    if n == 0 {
        return;
    }
    if n == 1 {
        b.cx(a[0], acc[0]);
        b.cx(c_in, acc[0]);
        return;
    }

    let carries = b.alloc_qubits(n - 1);

    // Forward inv_UMA sweep with carry ancillae (reversed UMA from cuccaro_sub).
    // Step 0:
    b.cx(c_in, acc[0]);
    b.cx(a[0], c_in);
    b.ccx(c_in, acc[0], carries[0]);
    b.cx(carries[0], a[0]);
    // Steps 1..n-2:
    for i in 1..n - 1 {
        b.cx(a[i - 1], acc[i]);
        b.cx(a[i], a[i - 1]);
        b.ccx(a[i - 1], acc[i], carries[i]);
        b.cx(carries[i], a[i]);
    }

    // Final sum bit (reversed from cuccaro_add)
    b.cx(a[n - 1], acc[n - 1]);
    b.cx(a[n - 2], acc[n - 1]);

    // Backward inv_MAJ sweep with measurement.
    for i in (1..n - 1).rev() {
        b.cx(carries[i], a[i]);
        let m = b.alloc_bit();
        b.hmr(carries[i], m);
        b.cz_if(a[i - 1], acc[i], m);
        b.cx(a[i], a[i - 1]);
        b.cx(a[i], acc[i]);
    }
    b.cx(carries[0], a[0]);
    let m0 = b.alloc_bit();
    b.hmr(carries[0], m0);
    b.cz_if(c_in, acc[0], m0);
    b.cx(a[0], c_in);
    b.cx(a[0], acc[0]);

    b.free_vec(&carries);
}

/// Fast Cuccaro add into an extended accumulator where the source high bit is
/// known zero: `acc_ext += a + c_in (mod 2^(n+1))`.
pub(crate) fn cuccaro_add_fast_low_to_ext(b: &mut B, a: &[QubitId], acc_ext: &[QubitId], c_in: QubitId) {
    let n = a.len();
    assert_eq!(acc_ext.len(), n + 1);
    if n == 0 {
        b.cx(c_in, acc_ext[0]);
        return;
    }

    let carries = b.alloc_qubits(n);

    b.cx(a[0], acc_ext[0]);
    b.cx(a[0], c_in);
    b.ccx(c_in, acc_ext[0], carries[0]);
    b.cx(carries[0], a[0]);
    for i in 1..n {
        b.cx(a[i], acc_ext[i]);
        b.cx(a[i], a[i - 1]);
        b.ccx(a[i - 1], acc_ext[i], carries[i]);
        b.cx(carries[i], a[i]);
    }

    b.cx(a[n - 1], acc_ext[n]);

    for i in (1..n).rev() {
        b.cx(carries[i], a[i]);
        let m = b.alloc_bit();
        b.hmr(carries[i], m);
        b.cz_if(a[i - 1], acc_ext[i], m);
        b.cx(a[i], a[i - 1]);
        b.cx(a[i - 1], acc_ext[i]);
    }
    b.cx(carries[0], a[0]);
    let m0 = b.alloc_bit();
    b.hmr(carries[0], m0);
    b.cz_if(c_in, acc_ext[0], m0);
    b.cx(a[0], c_in);
    b.cx(c_in, acc_ext[0]);

    b.free_vec(&carries);
}

/// Fast Cuccaro subtract from an extended accumulator where the source high bit
/// is known zero: `acc_ext -= a + c_in (mod 2^(n+1))`.
pub(crate) fn cuccaro_sub_fast_low_to_ext(b: &mut B, a: &[QubitId], acc_ext: &[QubitId], c_in: QubitId) {
    let n = a.len();
    assert_eq!(acc_ext.len(), n + 1);
    if n == 0 {
        b.cx(c_in, acc_ext[0]);
        return;
    }

    let carries = b.alloc_qubits(n);

    b.cx(c_in, acc_ext[0]);
    b.cx(a[0], c_in);
    b.ccx(c_in, acc_ext[0], carries[0]);
    b.cx(carries[0], a[0]);
    for i in 1..n {
        b.cx(a[i - 1], acc_ext[i]);
        b.cx(a[i], a[i - 1]);
        b.ccx(a[i - 1], acc_ext[i], carries[i]);
        b.cx(carries[i], a[i]);
    }

    b.cx(a[n - 1], acc_ext[n]);

    for i in (1..n).rev() {
        b.cx(carries[i], a[i]);
        let m = b.alloc_bit();
        b.hmr(carries[i], m);
        b.cz_if(a[i - 1], acc_ext[i], m);
        b.cx(a[i], a[i - 1]);
        b.cx(a[i], acc_ext[i]);
    }
    b.cx(carries[0], a[0]);
    let m0 = b.alloc_bit();
    b.hmr(carries[0], m0);
    b.cz_if(c_in, acc_ext[0], m0);
    b.cx(a[0], c_in);
    b.cx(a[0], acc_ext[0]);

    b.free_vec(&carries);
}
pub(crate) fn cuccaro_add_fast_windowed_low_to_ext(
    b: &mut B,
    a: &[QubitId],
    acc_ext: &[QubitId],
    c_in: QubitId,
    blocks: usize,
) {
    let n = a.len();
    assert_eq!(acc_ext.len(), n + 1);
    let ext_n = acc_ext.len();
    if ext_n == 0 {
        return;
    }
    let blocks = blocks.max(1).min(ext_n);
    if blocks == 1 {
        cuccaro_add_fast_low_to_ext(b, a, acc_ext, c_in);
        return;
    }

    let mut carry = c_in;
    let mut lo = 0usize;
    let mut couts: Vec<(QubitId, usize)> = Vec::new();
    for blk in 0..blocks {
        let hi = ((blk + 1) * ext_n) / blocks;
        if hi <= lo {
            continue;
        }
        if blk == blocks - 1 || hi == ext_n {
            cuccaro_add_fast_low_to_ext(b, &a[lo..n], &acc_ext[lo..hi], carry);
            break;
        }
        let cout = b.alloc_qubit();
        let zero = b.alloc_qubit();
        let mut a_block: Vec<QubitId> = a[lo..hi].to_vec();
        a_block.push(zero);
        let mut acc_block: Vec<QubitId> = acc_ext[lo..hi].to_vec();
        acc_block.push(cout);
        cuccaro_add_fast(b, &a_block, &acc_block, carry);
        b.free(zero);
        couts.push((cout, hi));
        carry = cout;
        lo = hi;
    }

    for &(cout, p) in couts.iter().rev() {
        cmp_lt_into_fast(b, &acc_ext[..p], &a[..p], cout);
        b.free(cout);
    }
}

pub(crate) fn cuccaro_sub_fast_windowed_low_to_ext(
    b: &mut B,
    a: &[QubitId],
    acc_ext: &[QubitId],
    c_in: QubitId,
    blocks: usize,
) {
    let n = a.len();
    assert_eq!(acc_ext.len(), n + 1);
    let ext_n = acc_ext.len();
    if ext_n == 0 {
        return;
    }
    let blocks = blocks.max(1).min(ext_n);
    if blocks == 1 {
        cuccaro_sub_fast_low_to_ext(b, a, acc_ext, c_in);
        return;
    }

    let mut borrow = c_in;
    let mut lo = 0usize;
    let mut bouts: Vec<(QubitId, usize)> = Vec::new();
    for blk in 0..blocks {
        let hi = ((blk + 1) * ext_n) / blocks;
        if hi <= lo {
            continue;
        }
        if blk == blocks - 1 || hi == ext_n {
            cuccaro_sub_fast_low_to_ext(b, &a[lo..n], &acc_ext[lo..hi], borrow);
            break;
        }
        let bout = b.alloc_qubit();
        let zero = b.alloc_qubit();
        let mut a_block: Vec<QubitId> = a[lo..hi].to_vec();
        a_block.push(zero);
        let mut acc_block: Vec<QubitId> = acc_ext[lo..hi].to_vec();
        acc_block.push(bout);
        cuccaro_sub_fast(b, &a_block, &acc_block, borrow);
        b.free(zero);
        bouts.push((bout, hi));
        borrow = bout;
        lo = hi;
    }

    for &(bout, p) in bouts.iter().rev() {
        for i in 0..p {
            b.x(a[i]);
        }
        cmp_lt_into_fast(b, &a[..p], &acc_ext[..p], bout);
        for i in 0..p {
            b.x(a[i]);
        }
        b.free(bout);
    }
}

pub(crate) fn cuccaro_sub_fast_borrowed_carries(
    b: &mut B,
    a: &[QubitId],
    acc: &[QubitId],
    c_in: QubitId,
    carries: &[QubitId],
) {
    let n = a.len();
    assert_eq!(n, acc.len());
    if n == 0 {
        return;
    }
    if n == 1 {
        b.cx(a[0], acc[0]);
        b.cx(c_in, acc[0]);
        return;
    }
    assert!(carries.len() >= n - 1);

    b.cx(c_in, acc[0]);
    b.cx(a[0], c_in);
    b.ccx(c_in, acc[0], carries[0]);
    b.cx(carries[0], a[0]);
    for i in 1..n - 1 {
        b.cx(a[i - 1], acc[i]);
        b.cx(a[i], a[i - 1]);
        b.ccx(a[i - 1], acc[i], carries[i]);
        b.cx(carries[i], a[i]);
    }

    b.cx(a[n - 1], acc[n - 1]);
    b.cx(a[n - 2], acc[n - 1]);

    for i in (1..n - 1).rev() {
        b.cx(carries[i], a[i]);
        let m = b.alloc_bit();
        b.hmr(carries[i], m);
        b.cz_if(a[i - 1], acc[i], m);
        b.cx(a[i], a[i - 1]);
        b.cx(a[i], acc[i]);
    }
    b.cx(carries[0], a[0]);
    let m0 = b.alloc_bit();
    b.hmr(carries[0], m0);
    b.cz_if(c_in, acc[0], m0);
    b.cx(a[0], c_in);
    b.cx(a[0], acc[0]);
}

pub(crate) fn cuccaro_add_ctrl_lowq(
    b: &mut B,
    a: &[QubitId],
    acc: &[QubitId],
    ctrl: QubitId,
    c_in: QubitId,
    scratch: QubitId,
) {
    let n = a.len();
    assert_eq!(n, acc.len());
    if n == 0 {
        return;
    }
    if n == 1 {
        b.ccx(ctrl, c_in, acc[0]);
        b.ccx(ctrl, a[0], acc[0]);
        return;
    }

    ctrl_maj(b, ctrl, c_in, acc[0], a[0], scratch);
    for i in 1..n - 1 {
        ctrl_maj(b, ctrl, a[i - 1], acc[i], a[i], scratch);
    }

    b.ccx(ctrl, a[n - 2], acc[n - 1]);
    b.ccx(ctrl, a[n - 1], acc[n - 1]);

    for i in (1..n - 1).rev() {
        ctrl_uma(b, ctrl, a[i - 1], acc[i], a[i], scratch);
    }
    ctrl_uma(b, ctrl, c_in, acc[0], a[0], scratch);
}

pub(crate) fn cuccaro_sub_ctrl_lowq(
    b: &mut B,
    a: &[QubitId],
    acc: &[QubitId],
    ctrl: QubitId,
    c_in: QubitId,
    scratch: QubitId,
) {
    let n = a.len();
    assert_eq!(n, acc.len());
    if n == 0 {
        return;
    }
    if n == 1 {
        b.ccx(ctrl, a[0], acc[0]);
        b.ccx(ctrl, c_in, acc[0]);
        return;
    }

    ctrl_inv_uma(b, ctrl, c_in, acc[0], a[0], scratch);
    for i in 1..n - 1 {
        ctrl_inv_uma(b, ctrl, a[i - 1], acc[i], a[i], scratch);
    }

    b.ccx(ctrl, a[n - 1], acc[n - 1]);
    b.ccx(ctrl, a[n - 2], acc[n - 1]);

    for i in (1..n - 1).rev() {
        ctrl_inv_maj(b, ctrl, a[i - 1], acc[i], a[i], scratch);
    }
    ctrl_inv_maj(b, ctrl, c_in, acc[0], a[0], scratch);
}


/// Borrowed-carry form of [`cuccaro_add_fast_low_to_ext`].  The source has no
/// materialized high-zero pad lane: `acc_ext` is one bit wider than `a`, and
/// the caller supplies `a.len()` clean, pairwise-disjoint carry lanes.
pub(crate) fn cuccaro_add_fast_low_to_ext_borrowed_carries(
    b: &mut B,
    a: &[QubitId],
    acc_ext: &[QubitId],
    c_in: QubitId,
    carries: &[QubitId],
) {
    let n = a.len();
    assert_eq!(acc_ext.len(), n + 1);
    if n == 0 {
        b.cx(c_in, acc_ext[0]);
        return;
    }
    assert!(carries.len() >= n);

    b.cx(a[0], acc_ext[0]);
    b.cx(a[0], c_in);
    b.ccx(c_in, acc_ext[0], carries[0]);
    b.cx(carries[0], a[0]);
    for i in 1..n {
        b.cx(a[i], acc_ext[i]);
        b.cx(a[i], a[i - 1]);
        b.ccx(a[i - 1], acc_ext[i], carries[i]);
        b.cx(carries[i], a[i]);
    }

    b.cx(a[n - 1], acc_ext[n]);

    for i in (1..n).rev() {
        b.cx(carries[i], a[i]);
        let m = b.alloc_bit();
        b.hmr(carries[i], m);
        b.cz_if(a[i - 1], acc_ext[i], m);
        b.cx(a[i], a[i - 1]);
        b.cx(a[i - 1], acc_ext[i]);
    }
    b.cx(carries[0], a[0]);
    let m0 = b.alloc_bit();
    b.hmr(carries[0], m0);
    b.cz_if(c_in, acc_ext[0], m0);
    b.cx(a[0], c_in);
    b.cx(c_in, acc_ext[0]);
}

/// Zero-carry-in specialization of
/// [`cuccaro_add_fast_low_to_ext_borrowed_carries`].  The omitted `c_in`
/// register is known zero: its only forward role is to preserve the original
/// low source bit until the measured carry clear.  After that clear `a[0]`
/// holds the same value, so it can control the phase correction directly.
pub(crate) fn cuccaro_add_fast_low_to_ext_borrowed_carries_no_cin(
    b: &mut B,
    a: &[QubitId],
    acc_ext: &[QubitId],
    carries: &[QubitId],
) {
    let n = a.len();
    assert_eq!(acc_ext.len(), n + 1);
    if n == 0 {
        return;
    }
    let gate_suffix = square_selfhost_gate_suffix_carries(n);
    let borrowed = n - gate_suffix;
    assert!(carries.len() >= borrowed);

    b.cx(a[0], acc_ext[0]);
    b.ccx(a[0], acc_ext[0], carries[0]);
    b.cx(carries[0], a[0]);
    for i in 1..borrowed {
        b.cx(a[i], acc_ext[i]);
        b.cx(a[i], a[i - 1]);
        b.ccx(a[i - 1], acc_ext[i], carries[i]);
        b.cx(carries[i], a[i]);
    }
    for i in borrowed..n {
        maj(b, a[i - 1], acc_ext[i], a[i]);
    }

    b.cx(a[n - 1], acc_ext[n]);

    for i in (borrowed..n).rev() {
        uma(b, a[i - 1], acc_ext[i], a[i]);
    }
    for i in (1..borrowed).rev() {
        b.cx(carries[i], a[i]);
        let m = b.alloc_bit();
        b.hmr(carries[i], m);
        b.cz_if(a[i - 1], acc_ext[i], m);
        b.cx(a[i], a[i - 1]);
        b.cx(a[i - 1], acc_ext[i]);
    }
    b.cx(carries[0], a[0]);
    let m0 = b.alloc_bit();
    b.hmr(carries[0], m0);
    b.cz_if(a[0], acc_ext[0], m0);
}

/// Borrowed-carry inverse of
/// [`cuccaro_add_fast_low_to_ext_borrowed_carries`].
pub(crate) fn cuccaro_sub_fast_low_to_ext_borrowed_carries(
    b: &mut B,
    a: &[QubitId],
    acc_ext: &[QubitId],
    c_in: QubitId,
    carries: &[QubitId],
) {
    let n = a.len();
    assert_eq!(acc_ext.len(), n + 1);
    if n == 0 {
        b.cx(c_in, acc_ext[0]);
        return;
    }
    assert!(carries.len() >= n);

    b.cx(c_in, acc_ext[0]);
    b.cx(a[0], c_in);
    b.ccx(c_in, acc_ext[0], carries[0]);
    b.cx(carries[0], a[0]);
    for i in 1..n {
        b.cx(a[i - 1], acc_ext[i]);
        b.cx(a[i], a[i - 1]);
        b.ccx(a[i - 1], acc_ext[i], carries[i]);
        b.cx(carries[i], a[i]);
    }

    b.cx(a[n - 1], acc_ext[n]);

    for i in (1..n).rev() {
        b.cx(carries[i], a[i]);
        let m = b.alloc_bit();
        b.hmr(carries[i], m);
        b.cz_if(a[i - 1], acc_ext[i], m);
        b.cx(a[i], a[i - 1]);
        b.cx(a[i], acc_ext[i]);
    }
    b.cx(carries[0], a[0]);
    let m0 = b.alloc_bit();
    b.hmr(carries[0], m0);
    b.cz_if(c_in, acc_ext[0], m0);
    b.cx(a[0], c_in);
    b.cx(a[0], acc_ext[0]);
}

/// Zero-carry-in inverse of
/// [`cuccaro_add_fast_low_to_ext_borrowed_carries_no_cin`].
pub(crate) fn cuccaro_sub_fast_low_to_ext_borrowed_carries_no_cin(
    b: &mut B,
    a: &[QubitId],
    acc_ext: &[QubitId],
    carries: &[QubitId],
) {
    let n = a.len();
    assert_eq!(acc_ext.len(), n + 1);
    if n == 0 {
        return;
    }
    let gate_suffix = square_selfhost_gate_suffix_carries(n);
    let borrowed = n - gate_suffix;
    assert!(carries.len() >= borrowed);

    b.ccx(a[0], acc_ext[0], carries[0]);
    b.cx(carries[0], a[0]);
    for i in 1..borrowed {
        b.cx(a[i - 1], acc_ext[i]);
        b.cx(a[i], a[i - 1]);
        b.ccx(a[i - 1], acc_ext[i], carries[i]);
        b.cx(carries[i], a[i]);
    }
    for i in borrowed..n {
        inv_uma(b, a[i - 1], acc_ext[i], a[i]);
    }

    b.cx(a[n - 1], acc_ext[n]);

    for i in (borrowed..n).rev() {
        inv_maj(b, a[i - 1], acc_ext[i], a[i]);
    }
    for i in (1..borrowed).rev() {
        b.cx(carries[i], a[i]);
        let m = b.alloc_bit();
        b.hmr(carries[i], m);
        b.cz_if(a[i - 1], acc_ext[i], m);
        b.cx(a[i], a[i - 1]);
        b.cx(a[i], acc_ext[i]);
    }
    b.cx(carries[0], a[0]);
    let m0 = b.alloc_bit();
    b.hmr(carries[0], m0);
    b.cz_if(a[0], acc_ext[0], m0);
    b.cx(a[0], acc_ext[0]);
}