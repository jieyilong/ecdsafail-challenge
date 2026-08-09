# Coherent Window Integration for `8e9c9a2`

This branch adapts submission `8e9c9a2` from a classically specified mixed
addend to the coherent interface

```text
|J>|R>|0>_W -> |J>|R + T[J]>|0>_W.
```

The emitted ABI contains the quantum registers `X`, `Y`, and `J`, followed by
`2 * 2^k` classical 256-bit registers containing the `x` and `y` columns of an
arbitrary point table. The circuit no longer embeds a fixed `[j]G` table in its
operations: every table-payload operation is conditioned on the corresponding
runtime classical table bit.

## Circuit construction

The selected point is materialized only near each consumer:

1. A shared QROM traversal loads `x_J` and `y_J`, forms the coordinate
   differences, and is replayed to clear both payloads.
2. A QROM lookup loads `x_J`; the circuit applies `x_J + 2*x_J` to the recycled
   coordinate register, reverses the doubling, and replays the lookup. Thus
   `3*x_J` is derived at runtime rather than supplied as another table.
3. A shared traversal loads `y_J` and `x_J` for output recovery and is replayed
   exactly after use.

The default 16-bit single-column lookup uses `K=2` QROAM. A 15-bit high address
loads adjacent rows into two 256-qubit banks, and the low address bit swaps the
selected row into the arithmetic operand. These payload banks are real quantum
storage; the construction saves width by releasing them before the dialog-GCD
peak, not by pretending that the 16 address qubits hold the selected point.

The default payload unload exactly replays the XOR lookup. It does not measure
the payload banks; the unary-routing AND ancillas are removed with the circuit's
standard measurement-based uncomputation and conditional phase correction. This
leaves the QROM address, payload, routing workspace, and phase clean. The earlier
measurement-based *payload* unload remains available for experiments through
`WINDOWED_QROM_UNLOAD=measurement`, but it is not used for the reported circuit.

Address zero is handled coherently. A predicate `nz = [J != 0]` disables the
square-subtract and final modular negation that would otherwise make the affine
pipeline incorrect for the identity row. For `J=0`, the remaining reversible
division/multiplication pair cancels and the circuit returns `R` unchanged.
As in Schrottenloher's generic-affine construction and the original challenge
kernel, the exceptional cases `R = +/-T[J]` remain outside the interface domain.

## Commands

```sh
cargo build --release --locked --offline --bin build_circuit --bin eval_circuit

WINDOWED_MODE=1 WINDOW_BITS=16 SINGLE_CCX_FANOUT_DISABLE=1 \
  ./target/release/build_circuit

WINDOWED_MODE=1 WINDOW_BITS=16 WINDOWED_TESTS=9024 \
  WINDOWED_MAX_ERROR_RATE=1 EVAL_THREADS=8 \
  EVAL_CHECKPOINT_DIR=/tmp/windowed-runtime-k16 \
  ./target/release/eval_circuit --note runtime-table-k16-replay-unload
```

`WINDOWED_QROM_COUNT_ONLY=1` profiles lookup resources without emitting payload
operations and is not a correctness test. `WINDOWED_SEQUENCE_CALLS=N` repeatedly
applies one emitted kernel to the same accumulator with correlated random
addresses; the evaluator checks output, preserved address, incremental phase,
and all non-interface ancillas after every call.

## Measured 16-bit result

The fully emitted runtime-table circuit has:

- peak logical width `Q = 1,162`;
- average executed Toffoli count `T = 1,684,156.770` over 9,024 cases;
- `213,387,745` emitted operations;
- `328,180` QROM Toffolis and `167,772,160` table-payload CNOTs;
- 14 classical mismatches, 10 phase-failure shots, no ancilla failures, and
  20 shots failing any channel (`0.2216%`).

The one identity selection in the 9,024-case corpus returned the input point,
preserved `J=0`, and had no classical, phase, or ancilla failure. The sparse
remaining errors arise from finite-support arithmetic approximations, including
the inherited fixed-length dialog-GCD schedule and the window adapter's low-peak
64-bit upper comparison, not from QROM cleanup. Relative to the reported
`8e9c9a2` operating point
(`Q = 1,151`, `T = 1,299,453`), coherent runtime-table selection adds 11 logical
qubits and approximately 29.61% average Toffoli work. The supports are generated
from their respective emitted streams, so this is a resource comparison rather
than a paired statistical comparison.

## Correlated small-window tests

For each `k = 0,...,5`, the evaluator ran 256 correlated four-call sequences,
or 1,024 calls per width. All 6,144 calls returned their non-interface ancillas
to zero. The `k=0` test consists entirely of identity selections and had no
failure, and the `k=1` emitter/unloader base case completed normally.

| `k` | Identity selections | Shots failing any channel | Per-call average Toffoli |
|---:|---:|---:|---:|
| 0 | 1,024 / 1,024 | 0 / 256 | 1,312,343.476 |
| 1 | 473 / 1,024 | 2 / 256 | 1,321,065.429 |
| 2 | 253 / 1,024 | 0 / 256 | 1,322,636.318 |
| 3 | 125 / 1,024 | 4 / 256 | 1,324,426.755 |
| 4 | 63 / 1,024 | 0 / 256 | 1,326,497.064 |
| 5 | 30 / 1,024 | 1 / 256 | 1,328,558.680 |

These tests establish composability of the lookup/use/replay boundary on the
stated generic-affine domain. They do not constitute an all-input correctness
proof or a complete implementation of the full windowed Shor circuit.
