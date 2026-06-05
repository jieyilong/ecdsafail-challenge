
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

// ═══════════════════════════════════════════════════════════════════════════
//  Cuccaro ripple-carry adder
// ═══════════════════════════════════════════════════════════════════════════
//
// Operates on two n-wide qubit registers `a` (addend, unchanged) and
// `acc` (accumulator, becomes a + acc mod 2^n). Also takes:
//   * c_in: one ancilla qubit, = 0 on entry, = 0 on exit (unchanged)
//   * z   : one ancilla qubit, = 0 on entry, = carry_out ⊕ z_in on exit
//           (i.e., the output carry is XORed into z; pass a fresh 0 bit
//           to receive the high bit)
//
// Based on Cuccaro et al. 2004 (arXiv:quant-ph/0410184), Figure 3.
//
// `MAJ(x, y, w)` triple:
//     CX(w, y)        # y ← y ⊕ w
//     CX(w, x)        # x ← x ⊕ w
//     CCX(x, y, w)    # w ← w ⊕ (x·y)        w becomes MAJ(w_old, y_old, x_old)
//
// `UMA(x, y, w)` triple (undoes MAJ, leaves sum bit in y):
//     CCX(x, y, w)
//     CX(w, x)
//     CX(x, y)

pub(crate) fn maj(b: &mut B, x: QubitId, y: QubitId, w: QubitId) {
    b.cx(w, y);
    b.cx(w, x);
    b.ccx(x, y, w);
}

pub(crate) fn uma(b: &mut B, x: QubitId, y: QubitId, w: QubitId) {
    b.ccx(x, y, w);
    b.cx(w, x);
    b.cx(x, y);
}

pub(crate) fn inv_maj(b: &mut B, x: QubitId, y: QubitId, w: QubitId) {
    // maj = CX(w,y); CX(w,x); CCX(x,y,w)
    // inv = CCX(x,y,w); CX(w,x); CX(w,y)
    b.ccx(x, y, w);
    b.cx(w, x);
    b.cx(w, y);
}

pub(crate) fn inv_uma(b: &mut B, x: QubitId, y: QubitId, w: QubitId) {
    // uma = CCX(x,y,w); CX(w,x); CX(x,y)
    // inv = CX(x,y); CX(w,x); CCX(x,y,w)
    b.cx(x, y);
    b.cx(w, x);
    b.ccx(x, y, w);
}
