# Handover: Exact 1050q -> 1045q Shrunken-PZ Reduction

## Goal

Starting from the TrailMix / shrunken-PZ 1050q design, apply only exact reversible
op-stream transformations to reduce peak qubits.

This branch packages two replayable milestones:

```text
1050q -> 1046q: baseline exact reduction package
1046q -> 1045q: additional high-temp cleanup package
```

No approximation, nonce dependence, carry truncation, or island-specific behavior is used.
Every accepted pass must preserve classical output, phase cleanup, and ancilla cleanup.

Verified q1045 result on this branch:

```text
qubits                : 1045
classical mismatches  : 0
phase-garbage batches : 0
ancilla-garbage       : 0
avg executed Toffoli  : 32245679.000
emitted ops           : 98806081
```

## Replay Commands

Use the normal challenge binaries:

```text
cargo build --release --bin build_circuit --bin eval_circuit --bin pz_peak_scan
./target/release/build_circuit
./target/release/pz_peak_scan ops.bin
./target/release/eval_circuit
```

The full evaluator, not just the peak scan, is the acceptance gate.

## Pass Stack

The route is implemented inside `pz1050_embedded_ops()` in `src/point_add/mod.rs`.
The order matters because every pass exposes the next peak owner.

```text
1. exact inverse-pair cancellation
2. reset-bounded exact compaction
3. q1047 dirty-borrow C3X
4. measured-AND fanout removal for q1046/q1045/q1044
5. q1045 dirty-borrow C3X
6. q1046 dirty-borrow C3X
7. q1046 inline single-temp AND
8. q1045 saved-copy dirty borrow
9. q1045 single-use copy removal
10. q1044 single-use copy removal
11. q1044 dirty-borrow C3X
12. targeted zero-reset retargeting
13. high-temp first-touch reset drop
14. final reset-bounded compaction
15. final high-temp reset drop
```

To reproduce only the q1046 milestone, stop after the older package described in
`HANDOVER_1050_TO_1046.md` or disable the q1045/q1044 add-on passes with their env flags.

## Exact Transformations

### Reset-Bounded Compaction

If two temporary live ranges are separated by an unconditional reset/HMR to a clean state,
they can share the same logical wire:

```text
value A live -> reset to |0>
later value B allocated
=> assign B to A's wire
```

This is register allocation over proven reset-bounded segments. It does not change the
unitary/measurement semantics.

Expected first milestone:

```text
pz1050 exact compactor: max q 1050 -> 1048
```

### Dirty-Borrow C3X

Clean temp C3X packets:

```text
temp ^= a & b
target ^= temp & c
temp ^= a & b
reset temp
```

are replaced by a dirty-borrow identity:

```text
target ^= d & c
d      ^= a & b
target ^= d & c
d      ^= a & b
```

For arbitrary dirty value `d`, the two target toggles differ by exactly `a & b & c`,
and `d` is restored. This saves the clean temp qubit when the borrowed wire is already live
and is not observed between borrow and restore.

### Measured-AND Fanout Removal

Measured AND packets:

```text
temp = a & b
CX temp -> target_i
HMR temp
CZ(a,b) under the measurement outcome
```

are replaced by direct fanout:

```text
CCX a b target_i
```

This removes the measured signpost qubit. It is exact because the HMR phase correction is the
standard quadratic `CZ(a,b)` for the measured AND; once all fanouts are direct, the measured temp
and its phase receipt disappear together.

### q1045 Saved-Copy Borrow

The extra q1045 layer removes an eight-op saved-copy packet:

```text
temp = x
u ^= x
x ^= u & temp
target ^= c & x
x ^= u & temp
u ^= x
u ^= c & temp
temp ^= x
```

Its net effect is:

```text
target ^= c & x & u_original
u      ^= c & x
```

The branch implements this with q1044 as a restored dirty borrow:

```text
target ^= q1044 & c
q1044  ^= x & u
target ^= q1044 & c
q1044  ^= x & u
u      ^= c & x
```

The borrowed q1044 value is restored exactly before subsequent observation.

### Single-Use Copy Removal

Copy-use-uncompute windows:

```text
temp ^= src
target ^= temp & other
temp ^= src
```

become:

```text
target ^= src & other
```

This is exact when the temp has no other intervening use and the target/control aliases are
excluded.

### q1044 Dirty-Borrow C3X

The q1045 branch also removes a small set of q1044 temp windows by borrowing q1043. This is
conservative: it only accepts windows where q1043 is not touched inside the compute/use/uncompute
region and where q1044 is reset immediately after the uncompute.

This is not enough to lower the global peak below 1045, but it is exact and removes local q1044
pressure for the next reduction attempt.

### High-Temp Reset Handling

The route does not broadly delete resets. It uses two safe reset operations:

```text
targeted retarget:
  move an inactive high-temp reset to a lower inactive sink

first-touch high reset drop:
  remove an unconditional reset on q1045/q1044 only when that wire is inactive,
  non-register, and the reset is the first touch of the next segment
```

Do not generalize this to arbitrary wires without re-proving phase and reset semantics.

## Expected Build Signature

A successful q1045 build should include counters like:

```text
pz1050 exact reducer: canceled 374891 inverse pairs
pz1050 exact compactor: max q 1050 -> 1048
pz1050 dirty-borrow C3X: removed 88 clean q1047 temp windows
pz1050 MBU AND fanout:
  replaced q1046/q1045/q1044 measured temp windows
pz1050 q1045 dirty-borrow C3X: removed 260 q1045 temp windows
pz1050 q1046 dirty-borrow C3X: removed 4 q1046 temp windows
pz1050 inline single-temp AND: replaced 88 q1046 windows
pz1050 q1045 saved-copy borrow: removed 88 clean q1045 packets
pz1050 q1045 single-use copy: replaced 4 windows
pz1050 q1044 single-use copy: replaced 68 windows
pz1050 q1044 dirty-borrow C3X: removed 56 windows
pz1050 zero-reset retarget: moved 92 resets
pz1050 high zero-reset drop: removed 320 then 192 resets
pz1050 exact compactor: max q 1047 -> 1046
```

Final metric line:

```text
qubits: 1045
all 9024 shots OK
```

## Remaining Peak Owner

After q1045, the measured peak is still a q1044-heavy plateau:

```text
named_qubits=1045
peak=1045
plateau_intervals=477
dominant targets: q1044, q1043, q1042
```

The next exact reduction should not keep turning approximate knobs. It should attack the
remaining plateau with one of:

```text
- broader exact measured-AND fanout removal for lower temps after proving phase cleanup
- a dynamic dirty-borrow pass that picks a safe live passenger per q1044 window
- a local live-range hole around long q1044 AND windows
- a reset-bounded compaction improvement that frees one lower live passenger across all q1044 plateaus
```

The long q1044 windows cannot be naively replaced by storing q1044 into an unknown dirty wire:
the unknown dirty value would contaminate later controls. Any q1044 removal must either use a
per-use dirty-borrow identity while the original controls are still valid, or prove a clean
reconstruction/hole for the whole window.

## Validation Rules

- Peak scan alone is not enough.
- Classical output alone is not enough.
- Every exact-width pass must pass:

```text
classical mismatches    : 0
phase-garbage batches   : 0
ancilla-garbage batches : 0
```

If any phase or ancilla dirt appears, treat the pass as structurally invalid until repaired.
