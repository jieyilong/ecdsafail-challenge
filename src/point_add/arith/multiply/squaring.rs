
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn squaring_sub_from_acc_walk_controls_lowq(b: &mut B, acc: &[QubitId], x: &[QubitId], p: U256) {
    let n = acc.len();
    debug_assert_eq!(n, 256);
    debug_assert_eq!(x.len(), n);

    let ctrl_copy = b.alloc_qubits(n);
    for i in 0..n {
        b.cx(x[i], ctrl_copy[i]);
    }

    mod_neg_inplace_fast(b, x, p);
    for i in 0..n {
        cmod_add_qq(b, acc, x, ctrl_copy[i], p);
        if i < n - 1 {
            mod_double_inplace_fast(b, x, p);
        }
    }
    for _ in 0..(n - 1) {
        mod_halve_inplace_fast(b, x, p);
    }
    mod_neg_inplace_fast(b, x, p);

    for i in 0..n {
        b.cx(x[i], ctrl_copy[i]);
    }
    b.free_vec(&ctrl_copy);
}
