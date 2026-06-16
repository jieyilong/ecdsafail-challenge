# PZ1050 Exact Toffoli-Reduction Handoff

## Summary

Workspace:

`/private/tmp/ecdsafail-pz1050-clean-codex-fresh`

Base branch:

`shrunken-pz-1050q`

Base commit:

`f4a8a18bab94e929f4ca3e57d3ea1f2136fb9336`

The clean 1050q shrunken-PZ design was improved using only exact op-stream postprocessing. No approximation, truncation, island hunting, or nonce dependence was used.

Final verified metrics:

```text
qubits                : 1048
avg executed Toffoli  : 31,836,407.000
avg executed Clifford : 41,551,540.778
emitted ops           : 98,860,578
correctness           : 9024/9024 shots OK, cls/pha/anc = 0/0/0
```

Baseline metrics from the clean branch before these passes:

```text
qubits                : 1050
avg executed Toffoli  : 32,252,071.000
emitted ops           : 99,669,267
correctness           : 9024/9024 shots OK, cls/pha/anc = 0/0/0
```

Net improvement:

```text
qubits                : -2
avg executed Toffoli  : -415,664
emitted ops           : -808,689
```

## Main Source Change

The embedded PZ1050 op stream is now decoded and then passed through exact postprocessing:

```rust
pub(crate) fn pz1050_embedded_ops() -> Vec<Op> {
    static BLOB: &[u8] = include_bytes!("pz1050/ec_shrunken_pz.kmx.lz");
    let text = pz1050_lz_decode(BLOB);
    let text = std::str::from_utf8(&text).expect("pz1050: utf8");
    let ops: Vec<Op> = text.lines().filter_map(Op::from_text).collect();
    pz1050_postprocess_ops(ops)
}
```

The relevant implementation lives in:

`src/point_add/mod.rs`

Key functions:

```text
pz1050_postprocess_ops
pz1050_exact_reduce_ops
pz1050_exact_hmr_uncompute_ops
pz1050_exact_zero_control_noops
pz1050_exact_compact_ops
```

## Pass Stack

The default pass order is:

```text
1. pz1050_exact_reduce_ops
2. pz1050_exact_hmr_uncompute_ops
3. pz1050_exact_zero_control_noops
4. pz1050_exact_reduce_ops
5. pz1050_exact_zero_control_noops
6. pz1050_exact_reduce_ops
7. pz1050_exact_hmr_uncompute_ops
8. pz1050_exact_compact_ops
```

Each pass is exact and has a disable knob for ablation:

```text
PZ1050_DISABLE_EXACT_REDUCE=1
PZ1050_DISABLE_EXACT_HMR_UNCOMPUTE=1
PZ1050_DISABLE_ZERO_CONTROL_NOOPS=1
PZ1050_DISABLE_FINAL_EXACT_REDUCE=1
PZ1050_DISABLE_SECOND_ZERO_CONTROL_NOOPS=1
PZ1050_DISABLE_SECOND_FINAL_EXACT_REDUCE=1
PZ1050_DISABLE_SECOND_HMR_UNCOMPUTE=1
PZ1050_DISABLE_EXACT_COMPACT=1
```

## How The Toffoli Cut Works

### 1. Resource-Aware Self-Inverse Cancellation

`pz1050_exact_reduce_ops` removes identical self-inverse op pairs when no touched resource is modified between them.

It tracks stack tops per qubit, bit, and register. A cancellation is allowed only when the same op is the top operation for every resource it touches. The pass flushes across `PushCondition` and `PopCondition`, so it does not commute gates through conditional regions.

Measured counters:

```text
first reducer:
  canceled 374,891 inverse pairs
  removed 749,782 ops
  removed 30,776 Toffoli ops

middle reducer:
  canceled 44,754 inverse pairs
  removed 89,508 ops
  removed 4,446 Toffoli ops

final reducer:
  canceled 0 inverse pairs
```

### 2. Strict Measured AND-Uncompute

`pz1050_exact_hmr_uncompute_ops` recognizes the pattern:

```text
CCX(a, b, t)
...
CCX(a, b, t)
```

where:

```text
t is known zero before the first CCX
t is not otherwise value-touched
a and b are not value-touched while t is live
the region is unconditional
```

The second `CCX` is replaced with:

```text
HMR(t -> fresh classical bit)
CZ(a, b | measured bit)
```

This is Gidney-style measured uncompute. It is exact because the measurement clears the AND target and the classically conditioned `CZ` repairs the phase.

Measured counters:

```text
first HMR pass:
  replaced 79,281 CCX clears with HMR+CZ

second HMR pass:
  replaced 52 CCX clears with HMR+CZ
```

### 3. Constant-Control Toffoli Demotion

`pz1050_exact_zero_control_noops` was upgraded from a zero-control remover into a small exact constant-propagation pass over qubits.

It tracks each qubit as:

```text
Zero
One
Unknown
```

Register qubits 0 and 1 are treated as unknown inputs. Other qubits start at zero. The pass is conservative: it only reasons through unconditional value operations and marks affected targets unknown under classical conditions.

Exact rewrites include:

```text
CCX(0, q, t) -> removed
CCX(1, q, t) -> CX(q, t)
CCX(1, 1, t) -> X(t)

CCZ(0, q, r) -> removed
CCZ(1, q, r) -> CZ(q, r)
CCZ(1, 1, q) -> Z(q)
CCZ(1, 1, 1) -> global phase NEG
```

Measured counters:

```text
first constant-control pass:
  removed 48,714 CCX
  removed 0 CCZ
  demoted 252,373 CCX to Clifford gates
  demoted 0 CCZ

second constant-control pass:
  removed 18 CCX
  removed 0 CCZ
  demoted 4 CCX
  demoted 0 CCZ
```

This was the largest new Toffoli cut.

### 4. Reset-Bounded Qubit Compaction

`pz1050_exact_compact_ops` relabels non-register qubit live segments after unconditional `R`/`HMR` clears.

It pins the two quantum input/output registers, but allows temporary reset-bounded segments to share physical qubit ids when their live ranges do not overlap.

Measured counter:

```text
max q 1050 -> 1048
moved 3,432,983 reset-bounded segments
```

This pass does not change Toffoli count directly, but it lowers peak qubits while preserving the exact op semantics.

## Verification

Rebuild and verify from the workspace:

```bash
cd /private/tmp/ecdsafail-pz1050-clean-codex-fresh
cargo run --release --bin build_circuit
cargo run --release --bin eval_circuit
```

Expected final build counters:

```text
pz1050 exact reducer: canceled 374891 inverse pairs (749782 ops, 30776 Toffoli ops)
pz1050 exact HMR uncompute: replaced 79281 CCX clears with HMR+CZ
pz1050 constant-control reductions: removed 48714 CCX and 0 CCZ; demoted 252373 CCX and 0 CCZ
pz1050 exact reducer: canceled 44754 inverse pairs (89508 ops, 4446 Toffoli ops)
pz1050 constant-control reductions: removed 18 CCX and 0 CCZ; demoted 4 CCX and 0 CCZ
pz1050 exact reducer: canceled 0 inverse pairs (0 ops, 0 Toffoli ops)
pz1050 exact HMR uncompute: replaced 52 CCX clears with HMR+CZ
pz1050 exact compactor: max q 1050 -> 1048
emitted ops : 98860578
```

Expected trusted eval:

```text
tested shots            : 9024
classical mismatches    : 0
phase-garbage batches   : 0
ancilla-garbage batches : 0
all 9024 shots OK

avg executed Toffoli  : 31836407.000
avg executed Clifford : 41551540.778
emitted ops           : 98860578
qubits                : 1048
```

## Failed Or Non-Promoted Probes

These are useful context for the next person:

### Broad HMR Was Not Exact

An experimental broad mode allowed AND controls to be touched while the AND target was live:

```text
PZ1050_HMR_ALLOW_CONTROL_TOUCH=1
```

It found many more candidates but failed full validation hard:

```text
9024 classical mismatches
141 phase-garbage batches
```

Leave this disabled unless a stricter proof of control restoration is added.

### Classical Condition Constants Were A No-Op

A temporary pass attempted to simplify classically conditioned operations by proving condition bits zero or one. It found:

```text
removed 0 false-conditioned ops
cleared 0 true conditions
```

It was removed rather than kept in the default stack.

### Phase Pair Probe Found No Extra Pairs

The `pz_phase_probe.rs` helper found no extra diagonal phase cancellation pairs after the default passes:

```text
phase_cancel_pairs 0
phase_cancel_ccz_pairs 0
```

## Suggested Next Exact Work

The remaining promising exact direction is to recover more measured-uncompute cases without enabling the unsafe broad HMR mode.

Concrete next step:

1. Classify the broad HMR candidates by how their controls are touched.
2. Allow only categories where the controls are provably restored before the clear.
3. Validate each category with full `eval_circuit`.

Potential safe categories to investigate:

```text
controls touched only by self-inverse pairs that cancel locally
controls swapped out and swapped back with no observation
controls toggled by known-constant CX/X patterns that restore exactly
controls touched only by phase gates, which do not change value
```

Do not promote any broader HMR category without full 9024-shot validation, including phase and ancilla checks.

## Cleanup Notes

Analysis helpers currently present in the workspace are not part of the final route:

```text
src/bin/pz_hmr_probe.rs
src/bin/pz_phase_probe.rs
```

Do not commit generated artifacts:

```text
ops.bin
target/
results.tsv
```

