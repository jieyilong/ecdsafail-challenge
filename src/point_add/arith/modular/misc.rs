
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

// ═══════════════════════════════════════════════════════════════════════════
//  Kaliski almost-inverse
// ═══════════════════════════════════════════════════════════════════════════

/// Fredkin (controlled swap): swap (a, t) if ctrl. Decomposed as CX/CCX/CX.
pub(crate) fn cswap(b: &mut B, ctrl: QubitId, a: QubitId, t: QubitId) {
    if a == t {
        return;
    }
    assert!(
        ctrl != a && ctrl != t,
        "invalid CSWAP with control aliased to swapped wire"
    );
    b.cx(t, a);
    b.ccx(ctrl, a, t);
    b.cx(t, a);
}
