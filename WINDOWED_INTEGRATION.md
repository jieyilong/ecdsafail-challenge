# Coherent Window Integration Experiment

This branch adapts submission `8e9c9a2` from a classically specified mixed
addend to a coherently selected window addend. Set `WINDOWED_MODE=1` to expose a
quantum address register and select `A_j=[j]G` through a reversible QROM. The
selected coordinates and derived constants are loaded, used, and uncomputed
serially so that a complete 512-qubit copy of the addend is never resident at an
arithmetic peak.

The default 16-bit implementation uses clean QROAM with block size `K=2`. A
15-bit high address loads two adjacent 256-bit table words into a payload bank
and a temporary bank; the remaining address bit selects the required word with
bitwise controlled swaps. The arithmetic preserves the selected payload. The
swaps are then reversed, both banks are measured in the X basis, and a
phase-only traversal erases the lookup. This spends the available local width
on one additional payload bank to approximately halve the nonlinear lookup
cost. Set `WINDOWED_QROM_MODE=unary` to select the lower-workspace unary
implementation for comparison.

## Commands

```sh
cargo build --release --locked --offline --bins
WINDOWED_MODE=1 WINDOW_BITS=16 TRACE_TLM_PROFILE=1 \
  SINGLE_CCX_FANOUT_DISABLE=1 \
  ./target/release/build_circuit
WINDOWED_MODE=1 WINDOW_BITS=16 WINDOWED_TESTS=9024 \
  WINDOWED_MAX_ERROR_RATE=0.003 \
  ./target/release/eval_circuit --note qroam2-w16-full
```

`WINDOWED_QROM_COUNT_ONLY=1` models the lookup resources without emitting the
large QROM payload. It is intended for width profiling, not correctness tests.
`SINGLE_CCX_FANOUT_DISABLE=1` skips a legacy global rewrite that is prohibitively
slow on the CNOT-heavy windowed stream and changed the trial circuit by only a
few dozen operations.

## Results

The fully emitted 16-bit circuit contains 179,176,904 operations and uses 1,161
logical qubits. Its three loads and three measurement-based unloads account for
246,260 Toffolis and 41,952,586 payload CNOTs. A 9,024-input development run
measured 1,595,255.204 average executed Toffolis, down 39.94% from
2,656,280.233 for the initial serial binary-tree adapter at the same width. The
optimized development run observed 17 classical failures, 14 phase-failure
shots, no ancilla failures, and 21 shots failing at least one channel, for a
union error of 21/9,024 = 0.2327%.

The final comparison uses a deterministic common corpus of 100,000 inputs. The
window-compatible circuit measured 1,593,217.836 average executed Toffolis and
235 any-channel failures (0.235%). The original mixed-addition circuit measured
1,299,442.073 average executed Toffolis and 219 any-channel failures (0.219%).
Both had 184 classical mismatches and no ancilla failures; phase-failure shots
were 144 and 127, respectively. Coherent window compatibility therefore adds
10 logical qubits and 22.61% average executed Toffoli work on this corpus.

The evaluation writes one durable row per input and can resume from the last
committed 64-shot batch. The row `j=0` represents the elliptic-curve identity,
which the generic affine point-addition pipeline does not special-case. The
100,000-input corpus deliberately retained one such row so that this limitation
remains visible.
