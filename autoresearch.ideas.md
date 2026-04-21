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
