
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn emit_dialog_gcd_round763_compressor(b: &mut B, block: &[QubitId]) {
    assert_eq!(block.len(), 6);
    if round763_compress_lever_enabled() {
        b.cx(block[5], block[3]);
        b.ccx(block[3], block[4], block[5]);
        b.cx(block[1], block[4]);
        b.cx(block[1], block[0]);
        b.ccx(block[4], block[5], block[1]);
        b.cx(block[0], block[2]);
        b.ccx(block[2], block[5], block[0]);
        b.ccx(block[0], block[1], block[5]);
        return;
    }
    b.ccx(block[4], block[5], block[3]);
    b.ccx(block[3], block[4], block[5]);
    b.ccx(block[1], block[2], block[4]);
    if round763_dedup_enabled() {
        b.cx(block[1], block[0]);
    } else {
        b.ccx(block[1], block[3], block[4]);
        b.cx(block[1], block[0]);
        b.ccx(block[1], block[3], block[4]);
    }
    b.ccx(block[4], block[5], block[1]);
    b.ccx(block[0], block[5], block[2]);
    b.ccx(block[2], block[5], block[0]);
    b.ccx(block[0], block[1], block[5]);
}

pub(crate) fn emit_dialog_gcd_round763_compressor_inverse(b: &mut B, block: &[QubitId]) {
    assert_eq!(block.len(), 6);
    if round763_compress_lever_enabled() {
        b.ccx(block[0], block[1], block[5]);
        b.ccx(block[2], block[5], block[0]);
        b.cx(block[0], block[2]);
        b.ccx(block[4], block[5], block[1]);
        b.cx(block[1], block[0]);
        b.cx(block[1], block[4]);
        b.ccx(block[3], block[4], block[5]);
        b.cx(block[5], block[3]);
        return;
    }
    b.ccx(block[0], block[1], block[5]);
    b.ccx(block[2], block[5], block[0]);
    b.ccx(block[0], block[5], block[2]);
    b.ccx(block[4], block[5], block[1]);
    if round763_dedup_enabled() {
        b.cx(block[1], block[0]);
    } else {
        b.ccx(block[1], block[3], block[4]);
        b.cx(block[1], block[0]);
        b.ccx(block[1], block[3], block[4]);
    }
    b.ccx(block[1], block[2], block[4]);
    b.ccx(block[3], block[4], block[5]);
    b.ccx(block[4], block[5], block[3]);
}

pub(crate) fn emit_dialog_gcd_round763_compressed_block_swapper(
    b: &mut B,
    pair: &[QubitId],
    compressed_block: &[QubitId],
    scratch: QubitId,
    slot: usize,
) {
    assert_eq!(pair.len(), 2);
    assert_eq!(compressed_block.len(), 5);
    assert!(slot < 3);
    let mut block = compressed_block.to_vec();
    block.push(scratch);
    emit_dialog_gcd_round763_compressor_inverse(b, &block);
    b.swap(pair[0], block[2 * slot]);
    b.swap(pair[1], block[2 * slot + 1]);
    emit_dialog_gcd_round763_compressor(b, &block);
}
