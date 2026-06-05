
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;


// ─── helpers: bit access on U256 ────────────────────────────────────────────

pub(crate) fn bit(c: U256, i: usize) -> bool {
    // alloy's U256::bit returns bool for index < 256.
    c.bit(i)
}

// ═══════════════════════════════════════════════════════════════════════════
//  Loading classical operands into a fresh qubit register
// ═══════════════════════════════════════════════════════════════════════════
//
// Cuccaro needs two qubit registers. To add a classical constant or a
// classical bit register to a quantum register, we allocate a fresh
// qubit register, load the classical value into it, run Cuccaro, then
// unload. The load/unload is not counted against Toffolis.

pub(crate) fn load_const(b: &mut B, n: usize, c: U256) -> Vec<QubitId> {
    let qs = b.alloc_qubits(n);
    for i in 0..n {
        if bit(c, i) {
            b.x(qs[i]);
        }
    }
    qs
}

pub(crate) fn unload_const(b: &mut B, qs: &[QubitId], c: U256) {
    for i in 0..qs.len() {
        if bit(c, i) {
            b.x(qs[i]);
        }
    }
    b.free_vec(qs);
}

pub(crate) fn load_bits(b: &mut B, bits: &[BitId]) -> Vec<QubitId> {
    let n = bits.len();
    let qs = b.alloc_qubits(n);
    for i in 0..n {
        // qs[i] ← bits[i] via conditional X
        b.x_if(qs[i], bits[i]);
    }
    qs
}

pub(crate) fn unload_bits(b: &mut B, qs: &[QubitId], bits: &[BitId]) {
    for i in 0..qs.len() {
        b.x_if(qs[i], bits[i]);
    }
    b.free_vec(qs);
}

// ═══════════════════════════════════════════════════════════════════════════
//  Extended registers and modular reduction
// ═══════════════════════════════════════════════════════════════════════════
//
// All modular arithmetic operates on "extended" registers of width n+1
// where bit n is an overflow/sign ancilla. The primitive quantum
// registers handed to us (Px, Py) are exactly n=256 wide; the extension
// bit is a transient ancilla allocated for the duration of a mod-op.

/// Build an (n+1)-bit view by attaching a freshly-allocated 0 ancilla.
pub(crate) fn ext_reg(b: &mut B, reg: &[QubitId]) -> (Vec<QubitId>, QubitId) {
    let ovf = b.alloc_qubit();
    let mut r = reg.to_vec();
    r.push(ovf);
    (r, ovf)
}

/// Release the overflow ancilla (which must be 0 on exit).
pub(crate) fn unext_reg(b: &mut B, ovf: QubitId) {
    b.free(ovf);
}

/// Fast `v := 2*v mod p` using measurement-based Cuccaro.
pub(crate) fn highest_set_bit(c: U256) -> usize {
    let mut hi = 0usize;
    for i in 0..256 {
        if bit(c, i) {
            hi = i;
        }
    }
    hi
}
