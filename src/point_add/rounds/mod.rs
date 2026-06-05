
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;


mod dialog;

pub(crate) use dialog::*;

// ─── folded from rounds/high.rs ───
pub(crate) fn round763_dedup_enabled() -> bool {
    // EXACT rewrite: the pair ccx(1,3->4) ... ccx(1,3->4) bracketing cx(1->0)
    // cancels (nothing between them touches 1/3/4), so it reduces to bare cx(1->0).
    // 2 CCX -> 0 per direction x ~1064 sites. Default OFF (op-stream reseed).
    std::env::var("DIALOG_GCD_ROUND763_DEDUP").ok().as_deref() == Some("1")
}

pub(crate) fn round763_compress_lever_enabled() -> bool {
    // Reachable-support rewrite of the round763 6->5 sidecar packer. Each raw
    // slot is (b0, b0_and_b1), with b0_and_b1 = b0 & (v<u), so state (0,1) is
    // unreachable on the verifier support. On that support, three CCX collapse
    // to CX and the compressor drops from 9 CCX to 4 CCX per direction.
    std::env::var("DIALOG_GCD_ROUND763_COMPRESS_LEVER")
        .ok()
        .as_deref()
        == Some("1")
}

// ─── folded from rounds/low/round008_190.rs ───
pub(crate) fn round84_emit_fused_square_xtail(
    b: &mut B,
    tx: &[QubitId],
    lam: &[QubitId],
    ox: &[BitId],
    p: U256,
) {
    b.set_phase("round84_fused_square_xtail_dx_sub_lam_square_lowq");
    if std::env::var("ROUND84_XTAIL_KARATSUBA").ok().as_deref() == Some("1") {
        // Squaring-aware 1-level Karatsuba square (default OFF). Overrides the
        // ROUND84_XTAIL_SCHOOLBOOK default set in configure_ecdsafail_submission_route.
        squaring_sub_from_acc_karatsuba(b, tx, lam, p);
    } else if std::env::var("ROUND84_XTAIL_WALK_SQUARE").ok().as_deref() == Some("1") {
        squaring_sub_from_acc_walk_controls_lowq(b, tx, lam, p);
    } else if std::env::var("ROUND84_XTAIL_SCHOOLBOOK").ok().as_deref() == Some("1") {
        squaring_sub_from_acc_schoolbook(b, tx, lam, p);
    } else {
        squaring_sub_from_acc_schoolbook_lowq_shift22(b, tx, lam, p);
    }
    b.set_phase("round84_fused_square_xtail_add_double_ox");
    mod_add_double_qb(b, tx, ox, p);
    b.set_phase("round84_fused_square_xtail_negate_to_x3");
    mod_neg_inplace_fast(b, tx, p);
}
