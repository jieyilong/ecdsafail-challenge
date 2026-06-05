
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;


pub(crate) fn ctrl_maj(b: &mut B, ctrl: QubitId, x: QubitId, y: QubitId, w: QubitId, scratch: QubitId) {
    b.ccx(ctrl, w, y);
    b.ccx(ctrl, w, x);
    mcx3_polar(b, ctrl, true, x, true, y, true, w, scratch);
}

pub(crate) fn ctrl_uma(b: &mut B, ctrl: QubitId, x: QubitId, y: QubitId, w: QubitId, scratch: QubitId) {
    mcx3_polar(b, ctrl, true, x, true, y, true, w, scratch);
    b.ccx(ctrl, w, x);
    b.ccx(ctrl, x, y);
}

pub(crate) fn ctrl_inv_maj(b: &mut B, ctrl: QubitId, x: QubitId, y: QubitId, w: QubitId, scratch: QubitId) {
    mcx3_polar(b, ctrl, true, x, true, y, true, w, scratch);
    b.ccx(ctrl, w, x);
    b.ccx(ctrl, w, y);
}

pub(crate) fn ctrl_inv_uma(b: &mut B, ctrl: QubitId, x: QubitId, y: QubitId, w: QubitId, scratch: QubitId) {
    b.ccx(ctrl, x, y);
    b.ccx(ctrl, w, x);
    mcx3_polar(b, ctrl, true, x, true, y, true, w, scratch);
}

pub(crate) fn cucc_add_ctrl_lowq(b: &mut B, a: &[QubitId], acc: &[QubitId], ctrl: QubitId) {
    let c_in = b.alloc_qubit();
    let scratch = b.alloc_qubit();
    cuccaro_add_ctrl_lowq(b, a, acc, ctrl, c_in, scratch);
    b.free(scratch);
    b.free(c_in);
}

pub(crate) fn cucc_sub_ctrl_lowq(b: &mut B, a: &[QubitId], acc: &[QubitId], ctrl: QubitId) {
    let c_in = b.alloc_qubit();
    let scratch = b.alloc_qubit();
    cuccaro_sub_ctrl_lowq(b, a, acc, ctrl, c_in, scratch);
    b.free(scratch);
    b.free(c_in);
}
