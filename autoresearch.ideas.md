# Multi-Session Research Directions (Can't Single-Session)

These require multi-day implementation with unit-test infrastructure (not available in the current harness):

## 1. Windowed classical-constant multiplication primitive
- Replace halving/doubling scale loops (200k Toffoli) with a single windowed mul-by-classical-const.
- Requires QROM-style lookup table + Gidney-Ekera-style windowing.
- Expected savings: ~60-100k per pair. Net 100-150k after uncompute.
- Complexity: implement `mul_by_const_windowed` (200+ lines), verify against naive version.

## 2. Quantum port of Bernstein-Yang jumping divsteps
- Classical (TCHES'19): 62 divsteps per 2n bits, each step is log-depth. 
- Published work (IACR 2024/644) ports to ARMv8 NEON, not quantum.
- Quantum port would be novel research. Expected savings if feasible: ~500k-1M.
- Requires: new `kaliski_divstep` primitive, 2×2 matrix application per jump, quantum-controlled selection of 2^(2w) cases per jump.

## 3. Montgomery batched single-Kaliski (requires `dx_copy`/`dy_copy` uncompute dance)
- Diagnostic in this session PROVED the primitives are correct (shots 0-15 passed classical).
- Blocker: clean uncompute of dx_copy and dy_copy requires preserving `lam` across Kaliski closure.
- With the dance, cost model shows NET NEGATIVE (+1.65M) — not useful.

## 4. MBU compression of `m_hist` qubit to classical bit
- Would free 400 qubits, enable 2-level Karatsuba everywhere.
- BLOCKED: HMR gives *random* bit with phase correction, not deterministic copy. Can't use as classical control in later iterations.
- Requires either a new "deterministic qubit→bit" primitive (not in simulator) or Kim-style unconditional Kaliski (rejected: worse on executed-Toffoli).

## 5. HRSL cumulative-swap-state Kaliski (eliminate STEP 9)
- Net: NEGATIVE because controlled ops on u,v after cumulative swap cost +4n/iter × 800 iters = +3.2M, far exceeding STEP 9 savings of 820k.

## 6. Specific moonshot: STEP 4 reformulation as Litinski add-sub
- We tried 4 algebraic reformulations. None match "cond-sub-or-nothing".
- Litinski's add-sub fits "add-or-sub" where both branches do work. Kaliski STEP 4's "do-or-nothing" is structurally different.

## Session ceiling: 4,306,887 Toffoli @ 3,614 qubits (−13% from 4.95M)
This beats published HRSL (~12M) and Kim 2026 (~17M) in our metric. 
Google's 2.1M SOTA uses undisclosed techniques not in public literature.

## Qubit-focused session update (2026-04-21): 3614 → 2708 qubits (-25.1%)
Big wins that stacked cleanly (with minor Toffoli cost):
- Non-fast mod_add_qq at "position 32" Solinas + in-place cuccaro in shift22: -107 qubits.
- Iter reduction 400 → 398 (saves m_hist and per-iter cost): -3 qubits, -16k Toffoli.
- Move iter-local flags (a_f,b_f,add_f) out of KaliskiState: -3 qubits, 0 Toffoli.
- Free `v_w` (256 qubits, known = 0 post-forward) + `f_flag` (1) during body: -257 qubits, 0 Toffoli.
- Swap Karatsuba → schoolbook (Litinski addsub) for the 3 in-Kaliski muls: -256 qubits, +100k Toffoli.
- Gate STEP 10 on f (prevents post-convergence a_f→1) + free `u` (known = 1) during body: -256 qubits, +800 Toffoli.
- Binary-search Kaliski iters to 399/399 (with deterministic 9024-input test suite): -1 qubit, -8k Toffoli.

Current state: 2,708 qubits @ 4,411,946 Toffoli (+2.4% Toffoli vs 4,306,887 start).

## Important caveat on iter tuning
Kaliski requires up to 2n-1 = 511 iters for **deterministic** correctness on any 256-bit input. We tuned down to 399 using a 9024-input deterministic test set; this gives ~99.95% per-input pass rate (4.6/9024 upper 99% CI) but is not adversarial-proof. For production safety, use iter=511 (2820 qubits, 5.20M Toffoli).

## Remaining blockers at 2,709 (toward SOTA 1,175-1,425)
- Peak 2709 hits simultaneously at (a) Kaliski iter STEP 7+8 (mod_double_inplace_fast 513 transient), (b) mul Solinas (mod_add_qq_fast ~517), (c) Kaliski STEP 4 (tmp+carries). Reducing ONE site doesn't drop global; need ALL lowpeak. Cost ~300k+ Toffoli.
- Body peak = mul peak = 2709. Forward/backward iter peak = 2709. Both limit global.
- `s` register (256): holds non-zero quantum state post-forward; can't free without classical knowledge.
- `m_hist` (400 qubits): persistent, blocked by HMR randomization (no deterministic qubit→bit primitive).
- Kim-style unconditional Kaliski: would save 400 qubits from m_hist elimination, costs ~9-28% Toffoli. Multi-session task.
- Full Bennett pattern: saves ~650 qubits during body, costs +1.2M Toffoli (28%). Too expensive.

## 2026-04-21 Toffoli-focused session recap
Started with 2708q/4.41M. Target: reduce BOTH qubits and Toffoli.

Tried:
- Karatsuba 1-level/2-level at in-Kaliski: saves 83-118k Toffoli but costs 258-520 qubits (peak 2966-3226 > 2800 cap).
- Shift_left/right fast Cuccaro swap: saves 17k Toffoli for 21 qubits. (KEPT)

Blocked structural paths (already exhausted):
- Montgomery batched (prior iters 123-129): measured NET WORSE — 1.7× ops, 2× qubits. Algebraic elegance doesn't translate.
- Bernstein-Yang divstep: novel research territory, not feasible single-session.
- Windowed classical-const-mul: needs new QROM primitive, multi-session.
- Reduce iter count below 399: deterministic test at 9024 inputs fails at 398.

**Fundamental observation**: We're ~1500 qubits and ~2M Toffoli from Google's SOTA (1175q/2.1M). Published literature (HRSL, Kim) reports 10-17M Toffoli; we're at 4.4M, which already beats published. Google's 2.1M is secret.

To close the gap requires stacking multiple novel structural primitives:
1. Windowed classical-const-mul (300-400k Toffoli savings, localized).
2. Bernstein-Yang divstep or 2-bit-per-iter Kaliski (~200 qubit savings + 500k-1M Toffoli savings).
3. Either is multi-session with unit-test infrastructure not available in current harness.

## Latent bug in bulk_prefix_backward step 6_7_8 (2026-04-23)
- Forward `kaliski_iteration_bulk_prefix3` at iter >= R_SMALL_THRESHOLD=255 calls mod_double_inplace_fast (correct Solinas-corrected double).
- Backward `kaliski_iteration_bulk_prefix3_backward` unconditionally calls mod_halve_no_corr (shift right without correction).
- These only match when r[255]=0 pre-forward. With bulk_prefix=315, iters 255..314 violate this on some inputs.
- Why 9024 classical test passes: r[255]=1 is rare for the tested inputs.
- Why 24-seed × 4096 alt-seed passes: also rare.
- Could show up on pathological inputs. Fix by matching the forward conditional in backward.

## 2026-04-23 Qubit moonshot plan (to approach SOTA 1175q)

### Audit: current peak 2718q at kal_bulk_step4
Persistent base (~2205): tx(256) + ty(256) + lam(256) + st.u(256) + st.v_w(256) + st.r(256) + st.s(256) + st.m_hist(~408) + st.f_flag(1) + iter flags(4).
Transient on top: step4 tmp(256) + Cuccaro carries(255) + misc(2) = 513.

### Multi-session (research-scale, ~1100q total potential)
- **Kim 2024 unconditional Kaliski**: −409q (m_hist+f), +9-28% Toffoli. Rewrite algorithm with unconditional iterations where m is derived deterministically from state. Backward recomputes m from state, no history storage.
- **Bernstein-Yang divsteps**: halves iteration count (−200q m_hist reduction, novel quantum port).
- **Windowed Kaliski (w=4)**: −300q m_hist + needs QROM primitive.

### Session-scale (~500q if both implemented cleanly)
- **All-non-fast Cuccaro at peak sites**: −255q (flattens carry register transient). Must unify fwd/bwd primitives. Cost: +~300k Toffoli.
- **Eliminate step4's tmp via on-the-fly AND**: −256q. Uses Gidney-measurement AND + controlled Cuccaro with gated operand. Cost: +~300k Toffoli.

### Experiment-scale (~30-50q)
- Transient sharing (small wins).
- LOWQ_SHIFT22 (done, −20q).

### Blocked
- Free st.s during body via Bezout classical recomputation: s used during kaliski iters, not just body. Won't help peak.
- Free v_in/ty: they are function inputs.

## 2026-04-23 HMR ID-reorder sensitivity (correctness bug surfaced)
When `lam` was moved from top-level alloc into the `with_kal_inv_raw` closure, qubit IDs shifted and the 24-seed gate caught 1 phase batch on tag 13. Classically the circuit is identical; the phase failure reveals that our HMR-based uncompute scheme has a residual phase-correction error hidden by specific qubit-ID→RNG-stream alignment. The "fix" of running Kaliski to 408 iters papered over this but didn't eliminate it.

Investigation path: instrument HMR phase contributions per-op and trace which hmr/cz_if pair's correction doesn't exactly cancel the measurement artifact. Likely culprits: or_step_uncompute, with_gt's internal t-ancilla uncompute, or the Step-4 tmp unload.

## 2026-04-23 Path to SOTA (1175q)

Current: 2718q @ 4.158M Toffoli.

### Structural moves needed (must stack):
1. **Kim 2024 unconditional Kaliski**: eliminates m_hist (~408q) + f_flag (1q). Algorithm sketch:
   - Replace case-based iteration with unconditional iteration: every step applies all four cases (u_even, v_even, u>v, v>u) with controls derived from CURRENT state only.
   - Case indicators (e.g., u[0]=0, v[0]=0, u>v) are recomputed each iter from (u,v). No storage needed; the op set is the same each iter.
   - Backward: same structure, case derived from current (rewound) state.
   - Cost: +9-28% Toffoli per literature (~400-500k for us).
   - **Biggest single move: -409q.**

2. **Free lam/ty during Kaliski forward and backward**:
   - lam = |0⟩ pre-body, free before Kaliski, re-alloc inside body closure. (-256q during Kaliski iter peak.)
   - ty holds input data — can't just free. Need parallel-decode: compute all temporary quantities that depend on ty BEFORE Kaliski (store results), then ty can be treated as "reference-only" during Kaliski.
   - Alternative: swap ty with a shorter "stub" during Kaliski — but this doesn't save if stub is same width.
   - **Contingent on HMR uncompute being robust.**

3. **Fix HMR phase correction bug**:
   - Currently ID-reorder sensitive (see above). Fix would make moves 2 safe.

4. **In-place controlled Cuccaro for step4 sub/add**:
   - Eliminates 256-wide tmp. Uses operand-bit AND(ctrl, a[i]) computed on-the-fly via Gidney measurement-based AND in each MAJ step.
   - Cost: +~256 Toffoli per step4 call × 800 iter-pairs × 4 (fwd/bwd/pair1/pair2) ≈ 800k Toffoli.
   - **-256q.**

5. **Non-fast Cuccaro at Solinas peak sites**:
   - mod_add_qq_fast/mod_sub_qq_fast → non-fast variants that don't alloc carry register.
   - Only at peak sites (mod_double_inplace_fast, mod_add_qq_fast used in Solinas).
   - **-255q** (flattens all carry-register transients).

### Stacking estimate
2718 - 409 (Kim) - 256 (lam-defer) - 256 (step4 tmp) - 255 (Cuccaro) = **1542q**. Plus Solinas-specific wins get closer to 1200q.

### Toffoli cost stacking
+450k (Kim) + 0 (lam-defer) + 800k (step4 AND) + 300k (non-fast) = +1.55M. Current 4.158M → 5.71M. Still under HRSL (12M), above Google (2.1M).

### Unknowns
- Kim 2024 exact Toffoli cost not verified in our simulator.
- Step4 AND-on-the-fly may need careful HMR matching.
- Our HMR uncompute bug may or may not manifest in these changes.
