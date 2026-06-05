# Kaliski ~1200q line — revisit verdict (2026-06-04)

Revisited the `kaliski_1200q_feasibility.md` line at the user's request. The
1200q projection hinges entirely on **m_hist elimination via recomputation**
(saves 407 persistent qubits; without it the decision-tree floor is ~1400q at
+28% Toffoli — already worse than today's 1350q dialog on both axes). That
unlock is **not viable**, for the same structural reason that killed divstep.

## The flaw in the feasibility doc's "verified" claim (§7, §13)

The doc verifies `m_i = F_min(f, u[0], v_w[0], u>v_w)` with **0 conflicts on
256k samples** — but `F_min` is evaluated on the iteration-**START** state
`S_i`. That is useless for the backward pass: when backward is about to undo
iteration `i`, the live state is `S_{i+1}` (iteration END). `U_i†` needs
`m_i` to pick the inverse branch, and `m_i = F_min(S_i)` requires `S_i`, which
needs `m_i` to recover — circular (identical to the divstep `g0` circularity).
The doc hand-waves this at lines 806–813 ("registers hold forward-iteration-
start values at backward-iter start") — that's wrong; they hold the END values.

## The decisive question: is `m_i` recoverable from the POST state?

Two independent lines of evidence say **no**:

1. **Repo's own exact enumeration** (`kaliski_linear_transform.rs`,
   `exact_local_predecessor_branch_count` + test
   `secp_local_poststate_predecessor_branch_is_ambiguous`): for each reached
   secp post-state, enumerate all locally-consistent inverse branches and
   re-run the step to verify. **>60% of post-states have ≥2 valid predecessor
   branches** (test asserts `frac > 0.60`). So no local post-state predicate
   recovers the branch; history is required.

2. **Fresh fingerprint sweep this session** (`kaliski_poststate_recoverability`
   in `kaliski_classical_replay.rs`, 255,500 samples, predicting `m_hist[i]`
   from `snaps[i+1]`):
   - low-k bits of (u',v',r',s') + f': low4=51429, low6=28784, low8=2609,
     low10=1061, low12=1064, low16=423 conflicts — **plateaus, never reaches 0**.
   - adding the post-comparator `gt'=(u'>v')`: low12+gt' = 1064 (IDENTICAL to
     low12 alone) — the comparator adds nothing.
   - FULL post-state: 0 conflicts, but this is **VACUOUS** — 255k samples in a
     2^1024 space never collide, so 0 is guaranteed regardless of recoverability.

   The persistent low-bit plateau is the shadow of the >60% true ambiguity.

## Verdict

The Kaliski 1200q line's m_hist-elimination is **dead via local post-state
recomputation** — same wall as divstep, already disproven by the repo's own
`kaliski_linear_transform.rs` (the feasibility doc predates/ignores it). The
remaining m_hist options (Kim unconditional +28% Toffoli → ~1280q; Bennett
pebbling → multiplicative Toffoli) do not yield a clean win, and even the best
non-elimination Kaliski (~1400q, +28% Toffoli) is worse than the live 1350q
dialog on BOTH qubits and Toffoli. **Not pursued.**

Net: the qubit-floor reduction below ~1350 requires breaking the per-step
history-storage barrier, which neither divstep nor Kaliski local-recompute can
do. The 1350 dialog floor (410 un-hostable transcript bits) stands.

## Artifacts (analysis-only, not wired into route)
- `kaliski_classical_replay.rs::kaliski_poststate_recoverability` + the
  `KalPostStateSummary` sweep.
- `bin/divstep_feasibility` prints the Kaliski post-state conflict sweep.
- Pre-existing: `kaliski_linear_transform.rs` exact predecessor enumeration.
