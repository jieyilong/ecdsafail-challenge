
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn cmod_add_qq(b: &mut B, acc: &[QubitId], a: &[QubitId], ctrl: QubitId, p: U256) {
    let n = acc.len();
    let f = b.alloc_qubits(n);
    for i in 0..n {
        b.ccx(ctrl, a[i], f[i]);
    }
    mod_add_qq_fast(b, acc, &f, p);
    // Gidney measurement-based AND uncomputation: f[i] = ctrl AND a[i],
    // which is unchanged by mod_add_qq (Cuccaro restores the addend).
    // HMR + classically-conditioned CZ costs 0 Toffoli vs 256 CCX.
    for i in 0..n {
        let m = b.alloc_bit();
        b.hmr(f[i], m);
        b.cz_if(ctrl, a[i], m);
    }
    b.free_vec(&f);
}

pub(crate) fn cmod_sub_qq(b: &mut B, acc: &[QubitId], a: &[QubitId], ctrl: QubitId, p: U256) {
    let n = acc.len();
    let f = b.alloc_qubits(n);
    for i in 0..n {
        b.ccx(ctrl, a[i], f[i]);
    }
    mod_sub_qq_fast(b, acc, &f, p);
    for i in 0..n {
        let m = b.alloc_bit();
        b.hmr(f[i], m);
        b.cz_if(ctrl, a[i], m);
    }
    b.free_vec(&f);
}

pub(crate) fn cmod_add_qq_lowq(b: &mut B, acc: &[QubitId], a: &[QubitId], ctrl: QubitId, p: U256) {
    let n = acc.len();
    let f = b.alloc_qubits(n);
    for i in 0..n {
        b.ccx(ctrl, a[i], f[i]);
    }
    mod_add_qq(b, acc, &f, p);
    for i in 0..n {
        let m = b.alloc_bit();
        b.hmr(f[i], m);
        b.cz_if(ctrl, a[i], m);
    }
    b.free_vec(&f);
}

pub(crate) fn cmod_sub_qq_lowq(b: &mut B, acc: &[QubitId], a: &[QubitId], ctrl: QubitId, p: U256) {
    let n = acc.len();
    let f = b.alloc_qubits(n);
    for i in 0..n {
        b.ccx(ctrl, a[i], f[i]);
    }
    mod_sub_qq(b, acc, &f, p);
    for i in 0..n {
        let m = b.alloc_bit();
        b.hmr(f[i], m);
        b.cz_if(ctrl, a[i], m);
    }
    b.free_vec(&f);
}

pub(crate) fn cmod_sub_qq_lowq_borrowed_subtrahend(
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

    for i in 0..N {
        b.ccx(ctrl, a[i], f[i]);
    }
    mod_sub_qq(b, acc, f, p);
    for i in (0..N).rev() {
        b.ccx(ctrl, a[i], f[i]);
    }
}
