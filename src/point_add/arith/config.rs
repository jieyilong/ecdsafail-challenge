
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;


pub(crate) fn d1_phase_corrected_product_core_active() -> bool {
    D1_PHASE_CORRECTED_PRODUCT_CORE_SCOPE.with(|scope| scope.get())
}

pub(crate) fn direct_const_walks_enabled() -> bool {
    std::env::var("KAL_DIRECT_CONST_WALKS").ok().as_deref() == Some("1")
}

pub(crate) fn secp_direct_const_arith_enabled() -> bool {
    std::env::var("SECP_DIRECT_CONST_ARITH").ok().as_deref() == Some("1")
}

pub(crate) fn double_carry_trunc_window() -> Option<usize> {
    std::env::var("KAL_DOUBLE_CARRY_TRUNC_W")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&w| w > 0)
}

/// Carry/borrow-tail truncation window for the pseudomersenne overflow/underflow
/// FOLD adders (the controlled `acc[..LSBS] += c` / `-= c` correction after a
/// raw 256-bit add/sub in the materialized-special apply path). Default OFF.
/// Same idea as `double_carry_trunc_window`: the secp256k1 constant
/// c = 2^32+977 is 7-bit-sparse, so the fold's carry ripple can stop a small
/// window above bit 32. Forward (cadd) and inverse (csub) read the same window,
/// so the reverse apply exactly inverts the forward when no truncation triggers
/// (the regime selected by the co-tuned reroll).
pub(crate) fn fold_carry_trunc_window() -> Option<usize> {
    std::env::var("KAL_FOLD_CARRY_TRUNC_W")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&w| w > 0)
}

/// Shift v left by k bits mod p. Returns (spill, flag_inv, ovf) which MUST
/// be passed to mod_shift_right_by_k for cleanup. Bennett-pattern: flags
/// stay alive across the body so the inverse can cleanly cancel them.
///
/// k must be small enough that spill·c < p. For k≤22 with secp256k1 this holds.
pub(crate) fn lowq_shift22() -> bool {
    if d1_phase_corrected_product_core_active() {
        return true;
    }
    // Default OFF: on the current scaffold it no longer reduces every global
    // peak, but it is the measured phase-corrected low-Q shift core for D1.
    // Keep the historical standalone knob for qubit-first experiments.
    match std::env::var("LOWQ_SHIFT22") {
        Ok(v) => v != "0",
        Err(_) => false,
    }
}
