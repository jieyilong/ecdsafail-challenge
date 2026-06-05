# 2026-06-04 Qubit peak 1350 is a hard floor; single-knob island density ~0

Base verified at HEAD `8ef64e9` (= promoted best `f664d6f`, stevenhao):
**1350 qubits × 1,763,987 Toffoli = 2,381,382,450**. Reproduced 0/0/0 over all
9024 shots via the real `eval_circuit` and an in-process clone.

## Peak 1350 is a 5-way co-bind (next tier 1309, gap = 41q)

`TRACE_PHASE_ACTIVE=1` shows five phases all pinned at 1350:
- `dialog_gcd_compressed_block_tobitvector_{compress_block,shift,reverse_add}` (GCD body)
- `dialog_gcd_materialized_special_chunked_raw_{sum,difference}` (apply add/sub)

The apply pair is already tuned (via `APPLY_CHUNKED_F_CUT*`) to sit exactly on
the GCD-body floor, so the body is the true binder. Next tier is 1309
(`round84_fused_square_xtail_dx_sub_lam_square_lowq`).

## Why the peak will not move with the cheap knobs (verified empirically)

`active_iterations` (393), `width_margin` (25), `width_slope` (711) are all
**peak-neutral**: peak stayed 1350 at active=393/390/387 and margin=25/23/20/15.
They only trade Toffoli, because the early GCD steps clamp `active_width` at
N=256, and the peak occurs there.

## Why the base floor is structural (the real blocker)

Forward GCD runs on `u` (seeded = p) and `factor` (= v, the input). Two passes
(quotient + ipmul). The branch transcript is 655 compressed bits (3 steps -> 5
bits, round763). Measured via instrumentation:
- 245 transcript bits hosted on idle high-`u` lanes (the "runway", cap=999=maxed)
- **410 transcript bits allocated as real qubits** -> these are the 82 EARLY
  blocks. 1024 (tx+ty+u+factor=4×256) + 410 + raw_block ≈ 1350.

The 410 early-block cells are un-hostable because:
- They must stay live across the apply phase (written forward, read in reverse).
- Only `u` is dormant across apply (`release_terminal_u` frees it); `factor`(=v)
  IS used by apply, so high-`v` hosting is ruled out for cross-apply cells
  (checked: extending the runway to v-high cannot host these).
- Their host lanes would need to be idle for the whole [forward-write..reverse-
  read] window, but early steps run at full width-256 (no idle u/v lanes).

=> Reducing the peak needs a fundamentally different inversion (safegcd /
jump-GCD to cut transcript length) or a better-than-3->5 transcript compression.
Not a knob.

## Toffoli: single-knob tightenings have ~0 reachable island

Built `src/bin/island_search.rs` (in-process clone of `eval_circuit::run_tests`
with early-exit; reproduces nonce 385307 -> 1,763,987 exactly). Swept
DIALOG_TAIL_NONCE (pure test-input reseed, circuit-cost-neutral) for:
- `ACTIVE_ITERATIONS=392`, `WIDTH_MARGIN=24`, `COMPARE_BITS=55`, `WIDTH_SLOPE=713`

**~2,220 nonces total, 0 clean islands.** ~8.5–12s/candidate. The 393/25/56/711
settings sit at a tight convergence/width bound for the (≈uniform) test
distribution; nonce==reroll for reseeding purposes, so the reroll space is no
denser. Single-knob tightening on this base appears exhausted.

## What to try next (all require real work, not knobs)

1. safegcd / jump-GCD inversion to shrink the 655-bit transcript (kills the
   early-block floor; this is the only credible path below 1350q).
2. A transcript compressor better than round763's 3->5 (would shrink the 410
   un-hostable cells directly: e.g. 4->6 over 4 steps = 1.5 b/step vs 1.667).
3. A genuinely value-exact Toffoli rewrite in the apply add/sub hot path
   (27% of Toffoli) that needs no island.
4. A 2-D (knob, knob) stack that opens a clean island where 1-D does not —
   low odds given the 1-D density measured here.
