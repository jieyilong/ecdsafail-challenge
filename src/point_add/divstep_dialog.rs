//! Bernstein-Yang "divstep dialog" modular inverter (circuit core).
//!
//! Phase 1: the FORWARD divstep GCD pass. Replaces the binary-GCD dialog's
//! 2-bits/step transcript (parity + data-dependent `u>v` comparison) with a
//! 1-bit/step transcript (parity `g0` only); the swap decision is recomputed
//! from a small signed counter `delta` (tracked here, biased, as `c = delta-1`
//! so that `delta>0` is just `c >= 0`, i.e. the sign bit of `c` is 0).
//!
//! Classical feasibility (correctness, 1-bit sufficiency, N_div) was settled in
//! `kaliski_classical_replay.rs` + `bin/divstep_feasibility.rs`. This module is
//! the reversible-circuit realization, validated against that classical model
//! via the in-process `Simulator`.
//!
//! Status: forward pass + simulator validation. The per-step `swap` ancilla is
//! left dirty in the forward-only path (cleaned in the reverse/uncompute phase);
//! the forward DATA registers (f, g, c, transcript) are exact.

#![allow(dead_code)]

use super::*;
use crate::circuit::{analyze_ops, QubitOrBit};
use crate::point_add::kaliski_classical_replay as dsr;
use crate::sim::Simulator;
use alloy_primitives::U256;
use sha3::{
    digest::{ExtendableOutput, Update},
    Shake256,
};

/// Signed register width for f, g (two's complement). Magnitudes stay < p, with
/// the transient `g±f` < 2p < 2^257, so 258 bits suffice; 260 leaves margin.
pub(crate) const DIVSTEP_W: usize = N + 4;

/// Width of the biased delta counter `c = delta - 1` (two's complement). |delta|
/// is bounded by the step count (< ~600), so 12 bits (±2048) is ample.
pub(crate) const DIVSTEP_DELTA_W: usize = 12;

// ─────────────────────────────────────────────────────────────────────────
// Generic-width reversible primitives (built on the monolith's cuccaro_add).
// ─────────────────────────────────────────────────────────────────────────

/// `acc += a mod 2^w`, controlled on `ctrl`. `a` is unchanged; `ctrl` clean.
fn cadd_ctrl(b: &mut B, a: &[QubitId], acc: &[QubitId], ctrl: QubitId) {
    let w = a.len();
    debug_assert_eq!(w, acc.len());
    let ff = b.alloc_qubits(w);
    for i in 0..w {
        b.ccx(ctrl, a[i], ff[i]);
    }
    let c_in = b.alloc_qubit();
    cuccaro_add(b, &ff, acc, c_in);
    b.free(c_in);
    // cuccaro_add leaves the addend `ff` unchanged, so the same CCX load zeroes it.
    for i in 0..w {
        b.ccx(ctrl, a[i], ff[i]);
    }
    b.free_vec(&ff);
}

/// `acc += 1 mod 2^w`, controlled on `ctrl`.
fn cinc(b: &mut B, acc: &[QubitId], ctrl: QubitId) {
    let w = acc.len();
    let t = b.alloc_qubits(w);
    b.cx(ctrl, t[0]);
    let c_in = b.alloc_qubit();
    cuccaro_add(b, &t, acc, c_in);
    b.free(c_in);
    b.cx(ctrl, t[0]);
    b.free_vec(&t);
}

/// `acc := -acc` (two's complement), controlled on `ctrl`.
fn cneg(b: &mut B, acc: &[QubitId], ctrl: QubitId) {
    for &q in acc {
        b.cx(ctrl, q);
    }
    cinc(b, acc, ctrl);
}

/// In-place arithmetic shift right by 1 (sign-preserving) of a two's-complement
/// register whose low bit is known to be 0 (exact halving). 0 Toffoli: a SWAP
/// bubble plus a single sign-extension CX.
fn ashr1(b: &mut B, g: &[QubitId]) {
    let w = g.len();
    if w < 2 {
        return;
    }
    for i in 0..w - 1 {
        b.swap(g[i], g[i + 1]);
    }
    // After the bubble: g[i]=old g[i+1], g[w-1]=old g[0]=0; g[w-2]=old sign.
    // Sign-extend the top: g[w-1] := old sign.
    b.cx(g[w - 2], g[w - 1]);
}

// ─────────────────────────────────────────────────────────────────────────
// Forward divstep step.
// ─────────────────────────────────────────────────────────────────────────
//
// State: f (DIVSTEP_W, odd), g (DIVSTEP_W, signed), c = delta-1 (DIVSTEP_DELTA_W,
// signed), and the per-step transcript bit. Per step:
//   g0    := g & 1                          (recorded into transcript)
//   pos   := (delta > 0) == (c >= 0) == NOT c[sign]
//   swap  := pos AND g0
//   if swap: swap(f,g)                       -> f=old g, g=old f
//   if swap: g := -g
//   if g0:   g := g + f                      -> swap: old g - old f ; else: old g + old f
//   g := g >> 1                              (arithmetic; g is even here)
//   delta update:  swap ? 1-delta : 1+delta  <=>  c := swap ? ~c : c+1
//
// The `swap` ancilla is allocated fresh and left dirty (forward-only milestone).
fn divstep_forward_step(
    b: &mut B,
    f: &[QubitId],
    g: &[QubitId],
    c: &[QubitId],
    t_bit: QubitId,
) {
    let w = f.len();
    let dsign = c.len() - 1;

    // 1. record g0 into the transcript bit.
    b.cx(g[0], t_bit);

    // 2. swap = (c[sign]==0) AND (t_bit==1).
    let sw = b.alloc_qubit();
    mcx2_polar(b, c[dsign], false, t_bit, true, sw);

    // 3. conditional full-width swap (f,g).
    for i in 0..w {
        cswap(b, sw, f[i], g[i]);
    }

    // 4. conditional negate g.
    cneg(b, g, sw);

    // 5. conditional add f into g (control g0).
    cadd_ctrl(b, f, g, t_bit);

    // 6. halve g (arithmetic).
    ashr1(b, g);

    // 7. delta/counter update: c := swap ? ~c : c+1.
    for i in 0..c.len() {
        b.cx(sw, c[i]);
    }
    b.x(sw);
    cinc(b, c, sw);
    b.x(sw);

    // sw left dirty (reverse pass will clean per-step ancillas).
}

/// `v := (-v) mod p`, controlled on `ctrl`. Mirrors `mod_neg_inplace_fast`
/// (bitflip + add p+1) but gated. For `v == 0` this yields the register value
/// `p` (≡ 0 mod p); callers always follow with a mod-reducing op so the quirk
/// is washed out.
fn cmod_neg(b: &mut B, v: &[QubitId], p: U256, ctrl: QubitId) {
    for &q in v {
        b.cx(ctrl, q);
    }
    cadd_nbit_const(b, v, p.wrapping_add(U256::from(1u64)), ctrl);
}

// ─────────────────────────────────────────────────────────────────────────
// Apply (inverse-multiply / Bezout reconstruction) step.
// ─────────────────────────────────────────────────────────────────────────
//
// Replays the (d,e) coefficient recurrence mod p, seeded d=0, e=num, using the
// stored transcript (g0) and a counter c=delta-1 recomputed in lockstep. After
// `n` steps, d = ± num·value^{-1} mod p (sign = final f sign from the forward
// pass). Per step:
//   g0   := transcript[i]
//   swap := (c[sign]==0) AND g0
//   if swap: swap(d,e); e := -e mod p
//   if g0:   e := e + d mod p
//   e := e/2 mod p
//   c := swap ? ~c : c+1
fn divstep_apply_step(
    b: &mut B,
    d: &[QubitId],
    e: &[QubitId],
    c: &[QubitId],
    t_bit: QubitId,
    p: U256,
) {
    let n = d.len();
    let dsign = c.len() - 1;

    let sw = b.alloc_qubit();
    mcx2_polar(b, c[dsign], false, t_bit, true, sw);

    for i in 0..n {
        cswap(b, sw, d[i], e[i]);
    }
    cmod_neg(b, e, p, sw);
    cmod_add_qq(b, e, d, t_bit, p);
    mod_halve_inplace_fast(b, e, p);

    for i in 0..c.len() {
        b.cx(sw, c[i]);
    }
    b.x(sw);
    cinc(b, c, sw);
    b.x(sw);
}

/// Emit `transcript.len()` apply divsteps over (d, e, c).
fn emit_divstep_apply(
    b: &mut B,
    d: &[QubitId],
    e: &[QubitId],
    c: &[QubitId],
    transcript: &[QubitId],
    p: U256,
) {
    for &t in transcript {
        divstep_apply_step(b, d, e, c, t, p);
    }
}

/// Emit `transcript.len()` forward divsteps over (f, g, c).
fn emit_divstep_forward(
    b: &mut B,
    f: &[QubitId],
    g: &[QubitId],
    c: &[QubitId],
    transcript: &[QubitId],
) {
    for &t in transcript {
        divstep_forward_step(b, f, g, c, t);
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Simulator validation (Phase 1).
// ─────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Default)]
pub struct ForwardStats {
    pub inputs: usize,
    pub n_steps: usize,
    /// Inputs where g did NOT reach 0.
    pub g_nonzero: usize,
    /// Inputs where |f| != 1 at the end.
    pub f_not_unit: usize,
    /// Inputs where the circuit transcript disagreed with the classical g0 stream.
    pub transcript_mismatch: usize,
}

impl ForwardStats {
    pub fn ok(&self) -> bool {
        self.g_nonzero == 0 && self.f_not_unit == 0 && self.transcript_mismatch == 0
    }
}

fn qb(reg: &[QubitId]) -> Vec<QubitOrBit> {
    reg.iter().map(|&q| QubitOrBit::Qubit(q)).collect()
}

/// Build the forward divstep circuit once, then simulate `inputs` random
/// secp256k1 values (64 per shot batch) and check that f→±1, g→0, and the
/// recorded transcript matches the classical divstep parity stream.
pub fn validate_forward(inputs: usize, n_steps: usize) -> ForwardStats {
    let p = dsr::SECP256K1_P;

    // Build the circuit. f is initialized to p inside the circuit; g is an input
    // register (set per shot); c and transcript start at 0.
    let mut b = B::new();
    let f = b.alloc_qubits(DIVSTEP_W);
    let g = b.alloc_qubits(DIVSTEP_W);
    let c = b.alloc_qubits(DIVSTEP_DELTA_W);
    let transcript = b.alloc_qubits(n_steps);
    for i in 0..N {
        if p.bit(i) {
            b.x(f[i]);
        }
    }
    emit_divstep_forward(&mut b, &f, &g, &c, &transcript);

    let g_reg = qb(&g[0..N]);
    let f_reg = qb(&f[0..N]);
    let g_set = qb(&g);

    let (total_qubits, num_bits, _nr, _regs) = analyze_ops(b.ops.iter());

    let mut stats = ForwardStats {
        inputs,
        n_steps,
        ..Default::default()
    };

    let mut seed: u64 = 1;
    let mut done = 0usize;
    while done < inputs {
        let batch = (inputs - done).min(64);
        let mut xof = Shake256::default().chain(b"divstep-fwd").finalize_xof();
        let mut sim = Simulator::new(total_qubits as usize, num_bits as usize, &mut xof);

        let mut vals = Vec::with_capacity(batch);
        for shot in 0..batch {
            let v = dsr::random_element(seed);
            seed += 1;
            sim.set_register(&g_set, v, shot);
            vals.push(v);
        }
        sim.apply_iter(b.ops.iter());

        for shot in 0..batch {
            let v = vals[shot];
            let run = dsr::divstep_inverse(v, p, n_steps);

            let g_out = sim.get_register(&g_reg, shot);
            if g_out != U256::ZERO {
                stats.g_nonzero += 1;
            }
            let f_out = sim.get_register(&f_reg, shot);
            if f_out != U256::from(1u64) && f_out != U256::MAX {
                stats.f_not_unit += 1;
            }
            // Transcript check.
            let mut mismatch = false;
            for i in 0..n_steps {
                let bit = ((sim.qubit(transcript[i]) >> shot) & 1) as u8;
                if bit != run.parity[i] {
                    mismatch = true;
                    break;
                }
            }
            if mismatch {
                stats.transcript_mismatch += 1;
            }
        }
        done += batch;
    }

    stats
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ApplyStats {
    pub inputs: usize,
    pub n_steps: usize,
    /// Inputs where the recovered ratio d != num·value^{-1} mod p.
    pub ratio_mismatch: usize,
}

impl ApplyStats {
    pub fn ok(&self) -> bool {
        self.ratio_mismatch == 0
    }
}

/// Build a combined forward+apply divstep circuit (forward produces the
/// transcript + final f sign; apply replays it on (d=0, e=num) and a sign-fix
/// negates d when f=-1), then simulate random (value, num) and check that the
/// recovered ratio equals num·value^{-1} mod p.
pub fn validate_apply(inputs: usize, n_steps: usize) -> ApplyStats {
    let p = dsr::SECP256K1_P;

    let mut b = B::new();
    // Forward registers.
    let f = b.alloc_qubits(DIVSTEP_W);
    let g = b.alloc_qubits(DIVSTEP_W);
    let c_fwd = b.alloc_qubits(DIVSTEP_DELTA_W);
    let transcript = b.alloc_qubits(n_steps);
    // Apply registers.
    let d = b.alloc_qubits(N);
    let e = b.alloc_qubits(N);
    let c_app = b.alloc_qubits(DIVSTEP_DELTA_W);

    for i in 0..N {
        if p.bit(i) {
            b.x(f[i]);
        }
    }
    emit_divstep_forward(&mut b, &f, &g, &c_fwd, &transcript);
    emit_divstep_apply(&mut b, &d, &e, &c_app, &transcript, p);
    // Sign-fix: if final f is -1 (sign bit set), negate d mod p so d = num/value.
    cmod_neg(&mut b, &d, p, f[DIVSTEP_W - 1]);

    let g_set = qb(&g);
    let e_set = qb(&e);
    let d_reg = qb(&d);

    let (total_qubits, num_bits, _nr, _regs) = analyze_ops(b.ops.iter());

    let mut stats = ApplyStats {
        inputs,
        n_steps,
        ..Default::default()
    };

    let mut seed: u64 = 1;
    let mut done = 0usize;
    while done < inputs {
        let batch = (inputs - done).min(64);
        let mut xof = Shake256::default().chain(b"divstep-apply").finalize_xof();
        let mut sim = Simulator::new(total_qubits as usize, num_bits as usize, &mut xof);

        let mut pairs = Vec::with_capacity(batch);
        for shot in 0..batch {
            let value = dsr::random_element(seed);
            let num = dsr::random_element(seed + 0x9000_0000);
            seed += 1;
            sim.set_register(&g_set, value, shot);
            sim.set_register(&e_set, num, shot);
            pairs.push((value, num));
        }
        sim.apply_iter(b.ops.iter());

        for shot in 0..batch {
            let (value, num) = pairs[shot];
            let inv = dsr::ref_inverse(value, p);
            let expected = dsr::ref_mulmod_pub(num, inv, p);
            let got = sim.get_register(&d_reg, shot);
            if got != expected {
                stats.ratio_mismatch += 1;
            }
        }
        done += batch;
    }

    stats
}
