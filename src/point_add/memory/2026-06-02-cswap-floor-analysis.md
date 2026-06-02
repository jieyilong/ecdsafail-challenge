# 2026-06-02 cswap reduction — FLOOR analysis (NEGATIVE)

Base during this study moved (concurrent commit 48d2cdf = 1,697,569 / 1698,
score 2,882,472,162: fold-W22 + round763-dedup + measured-underflow + reroll22).
cswap phases are IDENTICAL on both bases (tobit 113,860 / apply 102,144), so the
analysis below holds regardless.

Target phases:
- `dialog_gcd_compressed_block_tobitvector_cswap` fwd+rev = 227,720 (over u/v, active_width-truncated, 2 pairs)
- `dialog_gcd_compressed_block_apply_cswap` fwd + `..._reverse_cswap` = 204,288 (full-width N=256 over x/y; 399*256 each)

Each cswap = cx;ccx;cx = 1 CCX/bit. eval counts a CCX as `c_condition` popcount,
NOT quantum-control popcount -> every cswap CCX counts fully every shot (no
firing-rate discount).

## All three suggested approaches are BLOCKED

### (1) Merge adjacent apply cswaps via cswap(c1)·cswap(c2)=cswap(c1⊕c2): NO
- apply step = `double_y; cadd(b0); cswap(c)`, c=b0∧b1. Consecutive cswaps are
  separated by `double_y; cadd` — BOTH asymmetric (single out the y register).
- The XOR-merge identity needs the two swaps ADJACENT. Commuting a cswap through
  double_y/cadd conjugates them into c-CONTROLLED ops (controlled double of the
  OTHER register; controlled add direction). A controlled mod-double ≈ 2× the
  uncontrolled 69 CCX and you need it on both registers; routing the cadd doubles
  the 408k materialized_special phase. Net LOSS, large.
- Tobitvector has the same blocker (`cswap(c); sub(b0)` separated by asymmetric sub).
- NOTE: dialog-GCD already uses only 1 cswap/step — the old-Kaliski 2-cswap/iter
  merge was already captured by this algorithm choice. Nothing left to merge.

### (2) Tighten / fuse the tobitvector cswap: TAPPED / NO
- cswap width is LOCKED to active_width: the immediately-following controlled-sub
  reads the SAME bits, so narrowing the cswap below the sub corrupts any shot
  whose value reaches the margin zone. No cswap-specific width exists.
- active_width = N - 0.7075·step + margin is calibrated to the TRUE GCD envelope:
  * slope floor: 0.74 -> 303 mism, 0.78 -> 6208, 0.82 -> 9005. Real shrink ≈ 0.7075.
  * margin floor: 27 works (reroll 5, 0/0/0); 26 -> 1+ mism; 24 -> 9-18 mism.
- Compare-exchange-subtract fuse: removing the physical swap forces either a
  c-controlled subtract direction or a sign-magnitude subtractor that must
  preserve the discarded operand — both cost ≥ the swap. No CCX sweep saved.

### (3) Relabel / measure the cswap away: NO
- Swaps act on LIVE data (u/v, x/y), not ancilla -> measured-uncompute (free
  ancilla clear) does not apply.
- Only measurement primitive is Hmr = X-basis measure-and-reset with RANDOM
  outcome (faithful). Can't read a control into a classical c_condition. So the
  ~3/4 of apply swaps that don't fire (c=0) still cost a full 256-CCX sweep, and
  that waste is information-theoretically locked.
- Static relabel impossible: swap decisions depend on the superposed input.

## EMPIRICAL PROOF apply cswap is full-width (the big 204k target)
Flag-gated probe `DIALOG_GCD_APPLY_CSWAP_TRUNC_W=W` (truncate apply cswap to W
bits). W=255 (drop only the top bit) -> **9024/9024 classical mismatches + 141
phase-garbage**. x=target/y=accumulator are uniform 256-bit field elements; the
swap needs all 256 bits. No envelope. (Probe reverted; baseline byte-identical.)

## Confirmed floors
- ACTIVE_ITERATIONS=399 = convergence floor (397 -> 1 input fails to converge).
- slope 0.7075 = true GCD shrink rate (0.74 -> 303 mism).
- margin: on the OLD base (1,704,086) margin 27 reroll 5 was a clean -4,636 win;
  on the NEW base (1,697,569, reroll22 + fold/dedup/underflow levers baked) that
  slack is GONE — margin 27 fails over 24 rerolls tried (1-6 mism). So margin is
  now floored at 28. No clean width win remains for cswap on the current base.

## Net: cswap (432k, 25% of circuit) is at its structural floor. No local lever.

## Where the cswaps actually live (measured via STOP flags)
- pair1 quotient (GCD division dy/dx):           782,978 CCX
- pair2 ipmul   (GCD multiply  λ·(x1-x3)):       764,464 CCX  <-- 45% of circuit!
- round84 square + c-step:                       ~147,571 CCX
pair2's cswaps = tobit 56,930 + apply-fwd 102,144 (+ reverse). pair1's cswaps =
tobit 56,930 + apply-reverse 102,144. (apply_cswap phase = pair2 fwd; apply_
reverse_cswap = pair1.) pair1 and pair2 use DIFFERENT factors (dx vs x1-x3), so
their transcripts can't be shared.

## Two viable paths to a BIG cswap cut — both ARCHITECTURAL, not cswap-local
1. **Replace pair2's GCD-replay multiply with a direct Karatsuba/schoolbook
   modular multiply.** pair2 is a PURE multiply (λ·(x1-x3)) currently paying 764k
   CCX of GCD just to multiply (a deliberate qubit-for-Toffoli trade to hold peak
   1698). A direct mul is ~135-150k CCX (cf round84 square = 135k) -> would erase
   pair2's 159k cswaps AND ~450k of its add/sub. RISK: peak. Peak 1698 sits in
   the apply materialized add/sub; a direct mul needs a ~2N product register, so
   peak-neutrality is NOT guaranteed and must be engineered (reuse freed GCD
   scratch: compressed_log + raw_block + u). High value, high risk, big change.
2. Batched / jump GCD (safegcd-style) for pair1: fold T per-step swaps into a 2×2
   transition matrix on the LOW ~2T bits, apply once to full width (no per-step
   swaps). Major dialog-GCD core rewrite (see kaliski_jump.rs/round185_halfgcd_*).

Recommend: the swarm's cswap/Toffoli budget is best spent on (1). Pure cswap
local restructuring is exhausted.
