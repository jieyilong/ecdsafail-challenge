# 2026-06-05 Direct-cross Karatsuba square: correct, -0.32%, but island-starved

Base = the promoted **1350 q × 1,763,987 T = 2,381,382,450** lowq route
(`configure_ecdsafail_submission_route`). Submission UNCHANGED this session.

## Goal & result

Chase a Toffoli win from the round84 x-tail square while staying <= 1350 q.
The `(lo+hi)^2` Karatsuba (`ROUND84_XTAIL_KARATSUBA`, ~33k fewer emitted T) is
structurally pinned at **1411 q** (the 258-bit `z1_reg` must coexist with the
512-bit `tmp_ext`); at fixed truncations it SCORES WORSE (1411 × ~1.73M = 2.44B).
So it cannot be used under the hard 1350 cap.

**Direct-cross variant** (`ROUND84_XTAIL_KARATSUBA_DC=1`,
`squaring_sub_from_acc_karatsuba_directcross` in mod.rs ~L7166): computes the
cross term `2*lo*hi` straight into `tmp_ext[h+1 .. h+1+2h]` via
`schoolbook_mul_into` (NO z1_reg), z0=lo^2 / z2=hi^2 via `schoolbook_square_
symmetric`, then the SAME Solinas fold as lowq. Measured **floor 1282 q**
(probe), live peak **1285 q** -> global peak stays **1350** (GCD binder).
- Cross is a FULL 128x128 multiply (16,384 ANDs), so it loses the symmetric-
  square edge: only **-5,618 executed Toffoli** total (1,763,987 -> 1,758,369).
- Score 1350 × 1,758,369 = **2,373,798,150 (-0.32%)**. This is the CEILING for
  a square optimization under the 1350 cap with fixed truncations.

## The square is PROVEN CORRECT (not the problem)

Isolated self-check (built only `acc -= x^2 mod p`, simulated classically): 8
cases incl. lo/hi boundaries (2^128, 2^128±1), p-1, random 256-bit -> all
value-exact, x restored, every ancilla |0>. So eval failures are NOT a bug.

## Why it doesn't submit: island density (the real blocker)

The round84 op stream is hashed into the Fiat-Shamir seed, so the DC stream
reseeds the 9024 test inputs onto cases that trip the EXISTING GCD truncations
(the 17 classical mismatches + 8 phase batches at the default island 385307).
- The GCD + truncations are IDENTICAL between lowq and DC; the hash just permutes
  which inputs get sampled (SHAKE256 ~ uniform). So per-nonce clean probability
  is the SAME for both streams. lowq HAS an island (385307) -> DC islands exist
  at the same density, just sparse.
- **Swept ~302,000 DIALOG_TAIL_NONCE values (1..301000) for the DC stream: 0
  clean islands.** Consistent with density ~1-in-(few-hundred-k) (lowq's island
  sits at 385,307). Finding one would likely need ~1M+ nonces (~tens of hours).

Verdict: -0.32% is not worth the multi-hour island hunt now. DC code is kept
GATED (off by default; route never sets the flag) as a correct, reusable asset.

## Reusable tool built this session

`src/bin/island_search_fast.rs` — ~70x faster than `island_search` and
**bit-exact** (validated: finds lowq island 385307 -> avg_toffoli=1763987.000).
Three optimizations:
1. Build the 12.7M-op circuit ONCE; swap only the 96-op nonce tail (last ops).
2. Pre-absorb the prefix into a SHAKE256 sponge, clone + absorb tail per nonce.
3. **Fixed-base comb table for G** (libsecp256k1 idea): k*G in ~32 point-adds,
   not ~256 double-and-add. This was the 87% bottleneck (9024x2 scalar mults =
   7.3s of the old 8.75s/nonce). Now ~8 nonce/s on 11 threads.
Usage: `./target/release/island_search_fast <start> <count> [step]`
(set `ISLAND_LOWQ=1` to sweep the lowq stream for validation).

## What to try next (if revisiting the square)

1. Run the fast searcher over a much wider range (e.g. 301k..1.5M) — islands
   should exist; this is pure compute for the -0.32%.
2. A cross-term scheme that keeps the symmetric-square advantage without z1_reg
   coexisting with tmp_ext would beat -0.32%, but no such layout found (the
   (lo+hi)^2 combine fundamentally needs z1 live alongside tmp_ext -> 1411).
3. Bigger score levers remain the GCD-truncation frontier, not the square
   (see 2026-06-04-qubit-floor-and-island-density.md: 1-D tightenings exhausted).
