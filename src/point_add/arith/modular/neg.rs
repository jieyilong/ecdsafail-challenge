
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

/// Fast mod_neg using measurement-based Cuccaro for the addition.
pub(crate) fn mod_neg_inplace_fast(b: &mut B, v: &[QubitId], p: U256) {
    for &q in v {
        b.x(q);
    }
    let n = v.len();
    let ca = load_const(b, n, p.wrapping_add(U256::from(1)));
    add_nbit_qq_fast(b, &ca, v);
    unload_const(b, &ca, p.wrapping_add(U256::from(1)));
}
