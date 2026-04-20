# Autoresearch: quantum_ecc secp256k1 point-add Toffoli reduction

## Objective
Reduce the **average executed Toffoli count** of the reversible secp256k1 affine point-add circuit built in `src/point_add.rs`, while preserving harness correctness and keeping qubits within the current regime.

Current working baseline on `autoresearch/2026-04-20`:
- avg_toffoli: ~4,948,607
- avg_clifford: ~25,610,164
- qubits: 3249
- emitted_ops: 36,734,835

The workload is `cargo run --release`, which builds the circuit, runs 4096 randomized correctness shots, checks phase cleanliness and ancilla cleanup, and prints circuit metrics.

## Metrics
- **Primary**: `avg_toffoli` (lower is better)
- **Secondary**:
  - `avg_clifford`
  - `qubits`
  - `emitted_ops`
  - `correctness_ok`

## How to Run
`AUTORESEARCH_NOTE="baseline" ./autoresearch.sh`

The script emits structured `METRIC ...=...` lines for autoresearch.

## Files in Scope
- `src/point_add.rs` — the only project source file allowed to change; contains all arithmetic / Kaliski / point-add circuit construction.
- `autoresearch.md` — session context and experiment notes.
- `autoresearch.sh` — benchmark wrapper for autoresearch.
- `autoresearch.checks.sh` — secondary correctness rerun for passing candidates.
- `autoresearch.ideas.md` — backlog for larger deferred ideas.

## Off Limits
- Everything except `src/point_add.rs` and autoresearch session files.
- In particular: `src/main.rs`, `src/circuit.rs`, `src/sim.rs`, `src/weierstrass_elliptic_curve.rs`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain`, and direct manual edits to `results.tsv`.
- No new dependencies.

## Constraints
- `cargo run --release` must finish with `=== experiment OK ===`.
- All harness correctness conditions must hold: classical shots, phase cleanliness, ancilla cleanup.
- `qubits <= 3700`, and preferably do not exceed the current best qubit count by >5% unless Toffoli improves by >10%.
- `cargo build --release` must succeed; baseline already emits many warnings, so the practical gate here is build success plus benchmark correctness.
- Do not modify the harness or the benchmark workload.

## Workload Anatomy
High-level algorithm in `build()`:
1. `tx -= Qx`, `ty -= Qy`
2. Kaliski almost-inverse on `tx` (kept as raw scaled inverse)
3. Build `lam` from `ty * inv_raw`, then scale `lam` down by repeated modular halving
4. Use schoolbook multiply to zero `ty`
5. Compute `tx = Rx - Qx` using `lam^2`, `2*Qx`, `Qx`, and `mod_neg`
6. Compute `ty = lam * (Rx - Qx)`
7. Second raw-inverse pass on `tx = Rx - Qx`, then use it to uncompute `lam` and finish `ty = Ry`
8. Restore `tx = Rx`

Dominant structures:
- Kaliski forward/backward loop (`kaliski_iteration`, `kaliski_iteration_backward`)
- repeated `mod_double_inplace_fast` / `mod_halve_inplace_fast`
- schoolbook multiply / square reductions
- conditional modular add/sub helpers

## What's Been Tried
- Current branch already contains large wins from:
  - schoolbook multiply/square paths replacing more expensive Horner-style accumulation
  - symmetric schoolbook squaring
  - measurement-based backward Kaliski uncomputation
  - late-iteration truncation of `(u, v_w)` compares / swaps / OR chains
  - early-iteration no-correction `r` doubling in Kaliski
  - Solinas reduction consolidation around the sparse secp256k1 constant `2^32 + 977`
  - reducing Kaliski iterations from 512 to `2n-112 = 400`
- Known edge of correctness:
  - `2n-120` Kaliski iterations fails classical correctness
  - `2n-115` fails phase-garbage checks
  - `shift10 + shift22` Solinas variant failed with phase garbage
- Kaliski-floor probing outcome:
  - 399 iterations is safe
  - 398 iterations is safe
  - 397/397 fails badly
  - asymmetric split works: first pass must stay at 398, second pass can drop to 397
  - current best after that split: ~4,927,684 Toffoli
- Important conclusion:
  - single-iteration trims only buy ~4.2k Toffoli per affected Kaliski pass; this is nowhere near the ~2M improvement target.
  - The first Kaliski invocation is the fragile one; the second has a little extra slack.
- Google paper / Appendix A takeaways:
  - Their proof target uses the same kickmix gate set and the same average-executed-Toffoli style metric over Fiat-Shamir-derived tests.
  - Appendix A explicitly calls out measurement-based uncomputation (MBUC), but we already use MBUC inside the current design for Cuccaro sweeps, OR-chain cleanup, and many temporary AND unloads.
  - Therefore the remaining gap is unlikely to be a single missed MBUC trick on our current affine+two-Kaliski architecture.
  - More plausible hidden savings sources are: a cheaper inversion architecture, a materially different point-add formula / coordinate choice, or more aggressive multiplication architecture.
- Immediate promising directions:
  - eliminate the second inverse entirely by restructuring lambda uncomputation around preserved first-pass information
  - replace the 1-bit Kaliski loop with multi-bit divsteps / safegcd-style grouped iterations
  - try larger multiplier-architecture swaps (Karatsuba / beyond) as intermediate structural steps while investigating the inversion/formula gap
