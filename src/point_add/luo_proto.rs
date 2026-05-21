//! Classical prototype / qubit-budget scratchpad for Luo-style register sharing.
//!
//! Goal of this file: validate *cheaply* whether a Luo/PZ-style inversion track
//! is even compatible with the user's budget:
//!   - only ~600 qubits over the 512 input-point-coordinate qubits,
//!   - i.e. total target around 1100–1200 for the full point-add, or at least
//!     meaningfully below our current 2716q.
//!
//! We do NOT attempt a reversible circuit here. We only:
//!   1. model the qubit budget implied by Luo's register sharing,
//!   2. compare it to our current Kaliski scaffold,
//!   3. record the minimum structural consequences.
//!
//! Literature anchor: `/tmp/luo_ec_clean.txt`, especially Table 1 and Algorithm 3.

#![cfg(test)]

use alloy_primitives::U256;

use super::SECP256K1_P;

/// Very coarse qubit budget for the current live affine scaffold.
#[derive(Debug, Clone, Copy)]
struct Budget {
    tx_ty: usize,
    inversion_state: usize,
    lambda_and_mul_state: usize,
    classical_bits: usize,
    total: usize,
}

/// Current live build (best stable before the 511 detour):
/// - tx,ty = 2n = 512
/// - Kaliski persistent state = u,v_w,r,s,m_hist,f ≈ 4n + iters + 1
///   with iters ≈ 407/404 → ~1432
/// - live body state + mul transients explain the observed 2716 peak.
fn current_budget_estimate(n: usize, iters: usize) -> Budget {
    let tx_ty = 2 * n;
    let inversion_state = 4 * n + iters + 1;
    // Remaining gap to the observed peak (2716) is dominated by lam,
    // tmp_ext, carries, and a few flags.
    let total = 2716;
    let lambda_and_mul_state = total - tx_ty - inversion_state;
    Budget {
        tx_ty,
        inversion_state,
        lambda_and_mul_state,
        classical_bits: 2 * n, // ox, oy
        total,
    }
}

/// Luo-style inversion state from `/tmp/luo_ec_clean.txt`:
/// Table 1 says inversion can be done in roughly `3n + 4 log2 n + O(1)`
/// qubits *total* for the inversion component.
///
/// For n=256 this is about 3*256 + 4*8 = 800 qubits total, INCLUDING the
/// input/output pair of the inversion itself.
///
/// In our point-add context the inversion input is one n-bit value (dx or
/// similar), and we still need the 2n point coordinates live. So the key
/// number is the non-tx/ty overhead: about n + O(log n), not 4n+iters.
fn luo_inversion_total_qubits(n: usize) -> usize {
    3 * n + 4 * (n.ilog2() as usize)
}

/// Conservative point-add budget if we swapped our Kaliski block for a Luo/PZ
/// block *without* changing anything else in the affine scaffold.
fn naive_luo_point_add_budget_conservative(n: usize, current_other_peak: usize) -> usize {
    // Keep tx,ty live. Add the full Luo inversion block. Keep the rest of the
    // current non-inversion transients as-is.
    let tx_ty = 2 * n;
    let inversion_total = luo_inversion_total_qubits(n);
    tx_ty + inversion_total + current_other_peak
}

/// Optimistic overlap model for the same swap.
///
/// Luo's `3n + 4 log n` inversion count already includes the n-bit inversion
/// input register. In our point-add scaffold that register is already part of
/// tx/ty, so only `luo_total - n` is really "extra".
fn naive_luo_point_add_budget_optimistic(n: usize, current_other_peak: usize) -> usize {
    let tx_ty = 2 * n;
    let inversion_extra = luo_inversion_total_qubits(n) - n;
    tx_ty + inversion_extra + current_other_peak
}

/// Clean arithmetic helper for a tiny classical sanity check.
fn sub_mod(a: U256, b: U256, p: U256) -> U256 {
    if a >= b {
        (a - b) % p
    } else {
        p - ((b - a) % p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn luo_budget_is_qubit_relevant() {
        let n = 256usize;
        let cur = current_budget_estimate(n, 407);
        eprintln!("current budget estimate: {:?}", cur);

        let current_other_peak = cur.lambda_and_mul_state;
        let luo_cons = naive_luo_point_add_budget_conservative(n, current_other_peak);
        let luo_opt = naive_luo_point_add_budget_optimistic(n, current_other_peak);
        eprintln!("naive Luo swap-in peak estimate: conservative={luo_cons}, optimistic={luo_opt}");

        assert!(
            luo_cons < cur.total,
            "Luo-style inversion must reduce peak (conservative)"
        );
        assert!(
            luo_opt < cur.total,
            "Luo-style inversion must reduce peak (optimistic)"
        );

        // User budget is ~600 qubits over the 512 input coords.
        assert!(
            luo_opt > 1112,
            "If this flips, Luo-alone got us into the user budget"
        );
    }

    #[test]
    fn luo_alone_is_not_sota_but_is_structural() {
        let n = 256usize;
        let cur = current_budget_estimate(n, 407);
        let current_other_peak = cur.lambda_and_mul_state;
        let luo_cons = naive_luo_point_add_budget_conservative(n, current_other_peak);
        let luo_opt = naive_luo_point_add_budget_optimistic(n, current_other_peak);

        eprintln!(
            "luo_cons={}, luo_opt={}, current_total={}, saved_cons={}, saved_opt={}",
            luo_cons,
            luo_opt,
            cur.total,
            cur.total - luo_cons,
            cur.total - luo_opt
        );
        assert!(
            cur.total - luo_cons >= 500,
            "Luo should save ~500+ qubits conservatively"
        );
        assert!(
            cur.total - luo_opt >= 800,
            "Luo should save ~800+ qubits optimistically"
        );
    }

    #[test]
    fn even_free_inversion_needs_scaffold_collapse_to_hit_user_budget() {
        let n = 256usize;
        let cur = current_budget_estimate(n, 407);
        let target_total = 512 + 600;
        let inversion_free_total = cur.tx_ty + cur.lambda_and_mul_state;
        eprintln!(
            "inversion-free affine scaffold total={}, target_total={}, excess={}",
            inversion_free_total,
            target_total,
            inversion_free_total - target_total
        );
        assert_eq!(inversion_free_total, 1284);
        assert_eq!(inversion_free_total - target_total, 172);
        assert!(inversion_free_total > target_total);
    }

    #[test]
    fn luo_pz_gate_slope_is_not_point_add_sota_shaped() {
        // Luo/PZ register sharing is a real qubit lever, but the published
        // long-division EEA gate slope is not a hidden Google-style low-gate
        // point-add primitive.  The paper-level whole-ECDLP estimate is about
        // 976 n^3 Toffoli.  Dividing by the 2n point-add invocations gives a
        // per point-add inversion-scale cost of 488 n^2, before our affine
        // multiplications/cleanup.  At n=256 this is ~32M Toffoli, over an
        // order of magnitude above the 2.1M--2.7M Google point-add targets.
        let n = 256usize;
        let luo_whole_ecdlp_toffoli = 976usize * n * n * n;
        let per_point_add_proxy = luo_whole_ecdlp_toffoli / (2 * n);
        let google_low_qubit = 2_700_000usize;
        let google_low_gate = 2_100_000usize;
        eprintln!(
            "Luo/PZ gate-slope proxy: whole_ecdlp={luo_whole_ecdlp_toffoli}, per_point_add={per_point_add_proxy}, ratios_vs_google=({:.2}, {:.2})",
            per_point_add_proxy as f64 / google_low_qubit as f64,
            per_point_add_proxy as f64 / google_low_gate as f64
        );
        assert_eq!(per_point_add_proxy, 31_981_568);
        assert!(per_point_add_proxy > 10 * google_low_qubit);
        assert!(per_point_add_proxy > 15 * google_low_gate / 1); // loose integer guard
    }

    #[test]
    fn optimistic_luo_still_needs_hundreds_more_qubits_cut() {
        let n = 256usize;
        let cur = current_budget_estimate(n, 407);
        let target_total = 512 + 600;
        let luo_opt = naive_luo_point_add_budget_optimistic(n, cur.lambda_and_mul_state);
        eprintln!(
            "optimistic Luo total={}, target_total={}, remaining_gap={}",
            luo_opt,
            target_total,
            luo_opt - target_total
        );
        assert_eq!(luo_opt, 1828);
        assert_eq!(luo_opt - target_total, 716);
    }

    #[test]
    fn dy_py_relation_sanity() {
        // Tiny guard against the kind of algebra drift we had earlier.
        let p = SECP256K1_P;
        let px = U256::from(123u64);
        let py = U256::from(456u64);
        let qx = U256::from(17u64);
        let qy = U256::from(31u64);
        let dx = sub_mod(px, qx, p);
        let dy = sub_mod(py, qy, p);
        assert_eq!(dx, U256::from(106u64));
        assert_eq!(dy, U256::from(425u64));
    }

    /// H212-LUO-CLASSICAL-REPLAY-LAND — single-iteration falsification probe
    /// for the Luo length-register-coupling reversibility claim, evaluated
    /// at real secp256k1 inputs by replaying our concrete Kaliski iteration.
    ///
    /// Question (per research-205-213): does the candidate Luo "key"
    ///     (Λ_uv_pre, Λ_rs_pre, ΔΛ_uv, ΔΛ_rs, W1_lsb_pre, W2_lsb_pre, f_pre)
    /// where Λ_uv = bit_len(u) + bit_len(v_w), Λ_rs = bit_len(r) + bit_len(s),
    /// uniquely determine the per-iter branch tuple (a_f, m_i, add_f_step4)?
    ///
    /// Verdict semantics:
    ///   - collisions = 0 ⇒ NOT_FALSIFIED (Luo reversibility door OPEN; A203
    ///     stays demoted/medium with this measurement codified).
    ///   - collisions ≥ 1 ⇒ FALSIFIED (Luo register-sharing reversibility
    ///     does not hold at our primitive level; A203 closes).
    ///
    /// This is intentionally measurement-only — no `assert!` on collisions.
    #[test]
    fn luo_length_register_branch_determinism_at_real_secp256k1_inputs() {
        use sha3::{
            digest::{ExtendableOutput, Update, XofReader},
            Shake128,
        };
        use std::collections::HashMap;

        // Bit-length semantics sanity (the very first risk flagged by the
        // hypothesis): U256::bit_len() returns position-of-highest-set-bit+1,
        // so 0 → 0 and SECP256K1_P → 256.
        assert_eq!(U256::ZERO.bit_len(), 0, "U256::ZERO.bit_len() must be 0");
        assert_eq!(
            SECP256K1_P.bit_len(),
            256,
            "SECP256K1_P.bit_len() must be 256 for n=256 secp256k1"
        );

        // Local Shake128-deterministic element generator (mirrors
        // kaliski_classical_replay::random_element so this test is self-
        // contained and does not depend on a private item).
        fn random_element(seed: u64) -> U256 {
            let mut h = Shake128::default();
            h.update(&seed.to_le_bytes());
            let mut reader = h.finalize_xof();
            loop {
                let mut buf = [0u8; 32];
                reader.read(&mut buf);
                let v = U256::from_be_bytes(buf);
                if v != U256::ZERO && v < SECP256K1_P {
                    return v;
                }
            }
        }

        // Inline Kaliski-iter branch-tuple extractor — mirrors
        // `kaliski_classical_replay::tests::kaliski_iter_with_af` but also
        // returns `add_f_step4`. This is the Kaliski-circuit-equivalent of
        // Luo's (a_k, b_k, q_k) at our primitive level. See risks #2 of the
        // hypothesis for the structural mapping argument.
        fn kaliski_iter_branch_tuple(
            u: &mut U256,
            v_w: &mut U256,
            r: &mut U256,
            s: &mut U256,
            f: &mut u8,
            p: U256,
        ) -> (u8, u8, u8) {
            let is_zero = if *v_w == U256::ZERO { 1u8 } else { 0 };
            let mut m_i: u8 = 0;
            if *f == 1 && is_zero == 1 {
                m_i ^= 1;
            }
            *f ^= m_i;
            let u0 = (u.as_limbs()[0] & 1) as u8;
            let v0 = (v_w.as_limbs()[0] & 1) as u8;
            let mut a_f: u8 = 0;
            if *f == 1 && u0 == 0 {
                a_f ^= 1;
            }
            if *f == 1 && u0 == 1 && v0 == 0 {
                m_i ^= 1;
            }
            let b_f = a_f ^ m_i;
            let l_gt = if *u > *v_w { 1u8 } else { 0 };
            let add_f_step2 = (*f & l_gt) as u8;
            let delta = add_f_step2 & (1 ^ b_f);
            a_f ^= delta;
            m_i ^= delta;
            if a_f == 1 {
                std::mem::swap(u, v_w);
                std::mem::swap(r, s);
            }
            let add_f_step4 = *f & (1 ^ b_f);
            if add_f_step4 == 1 {
                *v_w = v_w.wrapping_sub(*u);
                *s = s.wrapping_add(*r);
            }
            *v_w = *v_w >> 1;
            let r2 = r.wrapping_add(*r);
            *r = if r2 >= p || r2 < *r {
                r2.wrapping_sub(p)
            } else {
                r2
            };
            if a_f == 1 {
                std::mem::swap(u, v_w);
                std::mem::swap(r, s);
            }
            (a_f, m_i, add_f_step4)
        }

        const N_INPUTS: usize = 200;
        const ITERS: usize = 407;
        let expected_samples = N_INPUTS * ITERS;

        // Primary key includes f_pre. Secondary key omits f_pre to answer
        // the variant-probe question (suggestedVerification #3).
        type Key = (u16, u16, i16, i16, u8, u8, u8);
        type KeyNoF = (u16, u16, i16, i16, u8, u8);
        type Val = (u8, u8, u8);

        let mut tbl: HashMap<Key, Val> = HashMap::new();
        let mut tbl_no_f: HashMap<KeyNoF, Val> = HashMap::new();
        let mut collisions: usize = 0;
        let mut collisions_no_f: usize = 0;
        let mut samples: usize = 0;
        let mut first_few: Vec<(Key, Val, Val)> = Vec::new();

        for seed in 0..N_INPUTS as u64 {
            let v_in = random_element(seed + 1);
            let mut u = SECP256K1_P;
            let mut v_w = v_in;
            let mut r = U256::ZERO;
            let mut s = U256::from(1u64);
            let mut f: u8 = 1;
            for _ in 0..ITERS {
                // Pre-iter Luo length registers.
                let luv_pre_u = u.bit_len() as u16;
                let luv_pre_v = v_w.bit_len() as u16;
                let lrs_pre_r = r.bit_len() as u16;
                let lrs_pre_s = s.bit_len() as u16;
                let w1_lsb = (u.as_limbs()[0] & 1) as u8;
                let w2_lsb = (v_w.as_limbs()[0] & 1) as u8;
                let f_pre = f;

                let (af, mi, add4) =
                    kaliski_iter_branch_tuple(&mut u, &mut v_w, &mut r, &mut s, &mut f, SECP256K1_P);

                let luv_post_u = u.bit_len() as u16;
                let luv_post_v = v_w.bit_len() as u16;
                let lrs_post_r = r.bit_len() as u16;
                let lrs_post_s = s.bit_len() as u16;

                let luv_pre = luv_pre_u + luv_pre_v;
                let lrs_pre = lrs_pre_r + lrs_pre_s;
                let dluv = (luv_post_u as i16 + luv_post_v as i16)
                    - (luv_pre_u as i16 + luv_pre_v as i16);
                let dlrs = (lrs_post_r as i16 + lrs_post_s as i16)
                    - (lrs_pre_r as i16 + lrs_pre_s as i16);

                let key: Key = (luv_pre, lrs_pre, dluv, dlrs, w1_lsb, w2_lsb, f_pre);
                let key_no_f: KeyNoF = (luv_pre, lrs_pre, dluv, dlrs, w1_lsb, w2_lsb);
                let val: Val = (af, mi, add4);

                match tbl.get(&key) {
                    None => {
                        tbl.insert(key, val);
                    }
                    Some(&prev) if prev != val => {
                        collisions += 1;
                        if first_few.len() < 4 {
                            first_few.push((key, prev, val));
                        }
                    }
                    _ => {}
                }
                match tbl_no_f.get(&key_no_f) {
                    None => {
                        tbl_no_f.insert(key_no_f, val);
                    }
                    Some(&prev) if prev != val => {
                        collisions_no_f += 1;
                    }
                    _ => {}
                }
                samples += 1;
            }
        }

        // Sanity #1 from hypothesis: kaliski_run on seed=1 should produce the
        // modular inverse v_in^{-1} mod p in the final s value (post 407
        // iters; Kaliski terminates within ≤2n iters). We do not have
        // direct access to s here in the loop above (it was consumed during
        // the per-iter replay), so re-run via the public helper for the
        // first seed only and confirm correctness.
        {
            use crate::point_add::kaliski_classical_replay::kaliski_run;
            let v_in1 = random_element(1);
            let (_m_hist, snaps) = kaliski_run(v_in1, SECP256K1_P, ITERS);
            // After Kaliski terminates, s holds v_in^{-1} mod p (possibly up
            // to a sign/2^k factor depending on the variant). We don't
            // verify the exact algebra here — kaliski_classical_replay's own
            // tests cover that — we just check the snapshot scaffolding
            // produced ITERS entries and the f flag did terminate.
            assert_eq!(snaps.len(), ITERS, "kaliski_run snapshot scaffold broken");
            let terminated = snaps.iter().any(|sn| sn.4 == 0);
            assert!(
                terminated,
                "kaliski_run never terminated in {} iters — replay scaffold broken",
                ITERS
            );
        }

        // Greppable, deterministic output for coordinator/researcher.
        println!("LUO_TOTAL_SAMPLES={}", samples);
        assert_eq!(samples, expected_samples, "sample count drift");
        println!("LUO_DISTINCT_KEYS={}", tbl.len());
        println!("LUO_DISTINCT_KEYS_NO_F={}", tbl_no_f.len());
        println!("LUO_COLLISIONS={}", collisions);
        println!("LUO_COLLISIONS_NO_F={}", collisions_no_f);
        if collisions > 0 {
            println!("LUO_REVERSIBILITY_VERDICT=FALSIFIED");
            for (k, prev, cur) in &first_few {
                println!(
                    "  collision key={:?} prev_branch={:?} new_branch={:?}",
                    k, prev, cur
                );
            }
        } else {
            println!("LUO_REVERSIBILITY_VERDICT=NOT_FALSIFIED");
        }
        // NOTE: intentionally NO `assert!(collisions == 0)` — this is a
        // measurement, not a correctness gate. The coordinator reads the
        // printed verdict and updates the approach ledger A203.
    }
}
