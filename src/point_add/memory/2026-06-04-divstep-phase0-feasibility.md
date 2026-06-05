# Divstep dialog GCD — Phase 0 classical feasibility (GO)

Date: 2026-06-04
Plan: `divstep_dialog_gcd` (break the 1350-qubit floor)
Harness: `src/point_add/kaliski_classical_replay.rs` (divstep section) +
`src/bin/divstep_feasibility.rs` (LOCAL-ONLY, not in submission).

Run: `cargo run --release --bin divstep_feasibility -- 300000`

## TL;DR — GO

Bernstein-Yang divstep modular inversion of `value mod p` (secp256k1 p):
- **Correct**: 0 inverse mismatches vs Fermat (`value^(p-2)`) over 300k random
  inputs; `|f|` reaches 1 in every case within the 768-step budget.
- **1 bit/step transcript is sufficient**: the parity stream `g&1` ALONE,
  with `delta` recomputed from a small signed counter, reproduces the exact
  per-step control sequence (swap / add / halve) — **0 control mismatches**
  over 300k inputs. This is the structural win: transcript drops from the
  current ~2 bits/step to 1 bit/step.
- **N_div is tightly clustered and low enough**: convergence (g→0) over 300k
  inputs is min=496, mean=531, max=565, p99.99=559. The 9024-worst-case
  quantile is ≈ **559–565 steps**.
- **Projected floor = 1254–1257** (< 1320 gate) → **PROCEED**.

## Algorithm (what was validated)

State `(delta, f, g, d, e)`, init `delta=1, f=p, g=value, d=0, e=1`.
`f,g` are signed (g±f can be negative) — modeled in 512-bit two's complement.
`d,e` tracked in `[0,p)` via mod-p halving. Invariant: `d·value ≡ f (mod p)`,
`e·value ≡ g (mod p)`.

Per step (records parity bit `g0 = g&1`):
- if `delta>0 && g0`:  `(f,g)=(g,(g−f)/2)`, `(d,e)=(e,(e−d)/2 mod p)`, `delta=1−delta`   [SwapSub]
- elif `g0`:          `g=(g+f)/2`, `e=(e+d)/2 mod p`, `delta=1+delta`                    [Add]
- else:               `g=g/2`,     `e=e/2 mod p`,     `delta=1+delta`                    [Halve]

At convergence `f=±1`; inverse = `d` if `f=1` else `p−d`.

The only data-dependent control is `g0`. `delta` is a deterministic function
of the `g0` history, so it is recomputed in lockstep during apply/reverse and
need NOT be stored. swap = `(delta>0) && g0`; the add-vs-halve choice = `g0`.

## Why this breaks the floor

Floor decomposition (current binary-GCD dialog):
`tx(256)+ty(256)+f=u(256)+g=factor(256)+allocated_transcript(410)+raw_block = 1350`.
The transcript is the only reducible term: 655 total branch-bits, 245 hosted
on idle high lanes, **410 un-hostable** (early blocks, written while full-width
f,g are live and surviving across apply).

Divstep transcript = `N_div` bits at 1 bit/step (≈ 560), vs 655 today. Reusing
the same ~245 hosting capacity gives un-hostable ≈ `560−245 = 315`, saving
≈ 95 qubits → **floor ≈ 1255**.

This is conservative — see width envelope below.

## Width envelope (max |g| bitlen per step, 20k inputs)

```
step:   0    56   112  168  225  281  337  394  450  506
|g|:   256  236  210  185  157  131  108   79   53   26
```

`|g|` (the active arithmetic width) shrinks ≈ **0.45 bits/step**, ≈ linearly.
By step ~506, |g| ≤ 26 bits, so the top ~230 lanes are idle. 533 of ~560 steps
have `|g| ≤ 248`. This means the hosting capacity for late-block transcript
cells is plausibly **much larger than the 245** used in the estimate above —
i.e. the true un-hostable count (and floor) could be lower than 1255. To be
re-derived precisely in Phase 4 with the real runway-layout machinery.

## Cost caveat (CRITICAL — likely score regression)

Divstep needs ≈ **560 steps vs 393** for the current binary GCD (**+43%**).
This is the plan's "catch", but the plan framed it only as transcript size
(N_div vs 655 bits). The bigger problem is **Toffoli**, via the apply phase.

Current Toffoli breakdown (`2026-06-02-session-handoff.md`, 1.73M total):
- apply add/sub (materialized_special_chunked_raw): ~470K (27%)
- apply cswap (fwd+rev):                            ~204K (12%)
- apply double/halve:                                ~52K  (3%)
- => **apply phase ≈ 726K = ~42% of Toffoli**, run once per step at FULL
  256-bit width (the (x,y) accumulators don't shrink like the GCD width does).
- tobitvector cswap (forward+reverse GCD pass):     ~225K (13%, scales with
  the shrinking width envelope, i.e. with step-area).

Projection for per-step divstep at 560 steps:
- apply phase: 726K × 560/393 ≈ **1034K** (+308K).
- forward/reverse GCD passes (~225K + comparator) scale with step-area
  (+~42% area), but divstep DELETES the per-step ~56-bit `u>v` comparator
  (~4.4%, ~76K) and the 2nd transcript stream.
- Net Toffoli ≈ 1.73M + ~308K (apply) + ~80K (gcd passes) − ~76K (comparator)
  ≈ **~2.05M (+18%)**.
- Score = Toffoli × peak ≈ 2.05M × 1255 ≈ **2.57e9**, vs current promoted
  **2.381e9** → **~8% WORSE**.

**Conclusion**: the per-step divstep (the approved plan) BREAKS the qubit floor
(−95q, good) but LOSES on score, because +43% steps hits the full-width apply
phase (42% of cost) far harder than the −7% qubit win and the −4.4% comparator
deletion can offset.

The only divstep variant that could win on score is the **jump / batched-matrix
(safegcd) form**: accumulate a k-step 2×2 transition matrix from small (~k-bit)
arithmetic, then apply it to (x,y) with O(1) full-width multiplies per block of
k steps — amortizing the full-width apply cost over k steps. (Note: the
2026-06-02 handoff already found step-batching gives only "5-11% Toffoli
reduction for high complexity" for the binary GCD, because the apply phase
"doesn't benefit from step-batching" in that formulation. Jump-divstep would
need its apply to be genuinely matrix-based to avoid that trap.)

### Recommendation
The Phase-0 qubit-floor gate passes, but the score gate (Phase 5: beat
2.381e9) is projected to FAIL for the per-step divstep. Decision required
before the multi-session circuit build (Phases 1–5): proceed as planned and
confirm by measurement, pivot to jump-divstep, or stop here.

## Phase 1–2 circuit results (data-path validated) + Phase 3 BLOCKER

Implemented in `src/point_add/divstep_dialog.rs` (flag-gated, not wired into the
default route), validated with the in-process `Simulator` via
`src/bin/divstep_circuit_check.rs`:
- **Forward divstep circuit**: over 4096 random inputs, g→0, |f|=1, and the
  recorded 1-bit transcript matches the classical parity stream exactly.
- **Apply circuit** (forward+apply composed): over 2048 random (value,num)
  pairs, recovers `num·value^{-1} mod p` exactly (sign-fixed by f's sign bit).

So the divstep DATA transformation is correct in-circuit. BUT these validations
deliberately leave the per-step `swap` ancilla DIRTY. Making the circuit
ancilla-clean (a hard requirement: the benchmark rejects any ancilla/phase
garbage) runs into a **fundamental obstacle**:

**The delta counter advance is not a reversible function of `(delta, g0)`.**
The map `delta -> (g0 ? (delta>0 ? 1-delta : 1+delta) : 1+delta)` is NOT
injective for fixed `g0` (e.g. with the biased counter `c=delta-1`, both
`c_old=0` (swap) and `c_old=-2` (no-swap, g0=1) map to `c_new=-1`). The swap/sign
information is irreducibly lost. Consequences:
- In the **forward/reverse** GCD passes the lost info can in principle be
  recovered from the live `(f,g)` (the full divstep IS a bijection on
  `(delta,f,g)`), at the cost of inverse-divstep recomputation.
- In the **decoupled apply** phase there is NO `(f,g)` — only `(d,e)` and the
  transcript. So the apply counter's per-step `swap` ancilla CANNOT be cleaned
  without either (a) destroying the result `d` (reversing the apply), or
  (b) storing the swap decision.

Storing the swap decision means the apply needs BOTH `g0` (for the conditional
mod-add) AND `swap` (for the conditional mod-swap) — i.e. **2 bits/step**, which
is exactly the current binary-GCD transcript. The add-control `g0` is NOT
recoverable from `swap` alone (`swap=0` admits `g0∈{0,1}`), and `swap` is NOT
recoverable from `g0` without the (uncleanable) counter. So:

**A clean reversible divstep apply requires 2 bits/step ⇒ NO qubit-floor win.**
The plan's central premise ("store only g0, recompute delta") holds for the
forward DATA path and for classical replay, but it does NOT survive the
ancilla-cleanliness requirement in the decoupled apply phase. This was not
visible in the Phase-0 classical model (which never has to uncompute).

## Phase 3 re-analysis — INTERLEAVED variant DISPROVEN (hard numeric evidence)

The user chose to evaluate the interleaved (Kaliski-style: carry Bezout coeffs
`d,e` alongside `f,g`, no decoupled apply) variant, gated on "does the peak
actually beat 1350?". Re-analysis says **no**, for two independent, now-proven
reasons:

**(1) The parity bit `g0` is irreversibly consumed and unrecoverable from the
post-step state.** divstep is NOT injective on `(δ,f,g)`: the non-swap branches
`g_old=2g'` (g0=0) and `g_old=2g'−f` (g0=1) both map to the same `(δ',f',g')`.
So uncomputing a step (in ANY arrangement) needs `g0` stored. Verified
numerically (`divstep_g0_recoverability`, ~11M post-step samples,
`bin/divstep_feasibility`): every candidate post-step fingerprint conflicts
hugely on `g0` —
  - `(δ'sign, g0', f0')`: 9.37M conflicts
  - `(full δ', g0')`: 3.87M conflicts
  - `(δ'sign, g_lo4, f_lo4)`: 7.23M conflicts
  - `(full δ', g_lo8, f_lo8)`: 3.22M conflicts
i.e. the plan's "reverse recomputes g0 from the live g" premise is FALSE. The
g0 transcript (1 bit/step) is mandatory in every arrangement, including
interleaved (so interleaving cannot eliminate the transcript; it only adds
`d,e` to the live set ⇒ strictly worse peak than the decoupled design).

**(2) The counter advance is non-injective even with `g0` known**, so the swap
decision must ALSO be stored to clean the counter in the apply (no f,g there;
and the counter never reads f,g, so reversing the pass doesn't help). Both
`c=0` (swap) and `c=−2` (no-swap, g0=1) advance to `c=−1`.

⇒ A clean reversible per-step divstep needs **2 bits/step (g0 + swap)** —
identical transcript size to today's binary-GCD dialog — while taking ~560
steps vs ~393. **Strictly worse: no qubit win AND more Toffolis.**

### VERDICT
Per-step divstep (decoupled OR interleaved) cannot beat the 1350 floor for a
clean reversible circuit. Build abandoned. Validated artifacts retained as a
correctness reference: the forward + apply divstep circuits are data-correct
(`divstep_dialog.rs`, `bin/divstep_circuit_check`), and the classical model +
recoverability disproof live in `kaliski_classical_replay.rs` +
`bin/divstep_feasibility`. The only theoretical path left for a real win is a
**jump/batched-matrix safegcd** (k-step transition matrix with block-local
counter scope) — a much larger research bet, NOT pursued without a go-ahead.

### Paths that could still win the floor (each a different design)
1. **Interleaved forward+apply** (Kaliski-style: carry the Bezout coefficients
   `d,e` alongside `f,g` and never store a transcript). Cleans the counter via
   the live `f,g`. Cost: `f,g,d,e` all live simultaneously (~1032q for the four
   256-bit registers) — competes with, doesn't obviously beat, today's 1350; and
   it abandons the transcript-hosting machinery entirely. Needs its own analysis.
2. **Jump/batched-matrix safegcd**: accumulate a k-step transition matrix from
   small (~k-bit) arithmetic where the counter lives in a tiny block-local scope
   that can be uncomputed within the block (the block boundary re-derives state
   from the matrix), then apply the matrix to `(x,y)` per block. This is also the
   only variant with a shot at the Toffoli/score win.
3. **Store 2 bits/step** (g0+swap) — clean and simple, but identical transcript
   size to today ⇒ no win. (Confirms the negative result.)

## Artifacts

- `kaliski_classical_replay.rs`: `divstep_inverse`, `divstep_controls_from_parity`,
  `divstep_width_envelope`, `ref_inverse` (Fermat), plus `#[cfg(test)] mod divstep_tests`.
  (Tests can't run via `cargo test --lib` due to PRE-EXISTING compile errors in
  sibling test modules — `num_bigint` missing, stale `sim::Simulator` API — so the
  feasibility is driven by the binary below.)
- `src/bin/divstep_feasibility.rs`: standalone driver, prints all metrics.
