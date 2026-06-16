# Handover: Exact 1050q -> 1046q Shrunken-PZ Reduction

## Goal

Starting from the TrailMix / shrunken-PZ 1050q design, apply only exact reversible and quantum-safe transformations to reduce peak qubits from 1050 to 1046.

No approximation, no nonce dependence, no heuristic truncation. The resulting circuit should pass full validation cleanly:

```text
classical mismatches: 0
phase-garbage batches: 0
ancilla-garbage batches: 0
peak qubits: 1046
avg Toffoli: ~32,245,199
```

## Core Idea

The reduction is not from changing the high-level algorithm. It is peak live-range surgery:

1. Reuse reset-bounded scratch wires.
2. Replace clean temporary AND/C3X qubits with dirty-borrow identities.
3. Inline measurement-uncomputed AND fanout.
4. Re-run peak-owner analysis after every drop.
5. Preserve reset/HMR semantics exactly.

The guiding invariant:

> A qubit may be removed or reused only if the replacement is value-exact, phase-exact, and restores every borrowed dirty qubit before it is observed again.

## Starting Point

Use the embedded shrunken-PZ 1050q op stream. The circuit initially emits around 100M ops and peaks at 1050 qubits before the exact reductions.

Expected first-stage compaction:

```text
pz1050 exact compactor: max q 1050 -> 1048
```

This comes from reset-bounded lifetime packing.

## Pass Order

Apply the transformations in this order:

```text
1. exact inverse-pair cancellation
2. reset-bounded exact compaction
3. dirty-borrow C3X for q1047 peak temps
4. MBU AND fanout for q1046/q1045/q1044 temp windows
5. dirty-borrow C3X for q1045 windows
6. dirty-borrow C3X for q1046 windows
7. inline single-temp AND cleanup for q1046
8. targeted zero-reset retargeting
9. final reset-bounded exact compaction
```

The order matters. Earlier passes expose later peak binders.

## Transformation 1: Reset-Bounded Compaction

### Purpose

If two temporary values are never live at the same time, assign them to the same qubit.

Quantum interpretation:

```text
value A lives, is uncomputed/reset to |0>
later value B is allocated
=> A and B can share one wire
```

This is reversible register allocation, not approximation.

### Expected Effect

```text
max q 1050 -> 1048
moved ~3.39M reset-bounded segments
```

## Transformation 2: Dirty-Borrow C3X

### Pattern

Some clean temp qubits are used to implement a three-control update:

```text
temp ^= a & b
target ^= temp & c
temp ^= a & b
reset temp
```

Instead, borrow an already-live dirty qubit `d`:

```text
target ^= d & c
d      ^= a & b
target ^= d & c
d      ^= a & b
```

### Why It Works

Let the borrowed qubit initially hold arbitrary value `d`.

The two target toggles differ by:

```text
c & d  XOR  c & (d XOR a&b)
= c & a & b
```

So the target receives exactly the intended three-control toggle, and `d` is restored exactly.

### Quantum Meaning

We use an already-live messy qubit as scratch, but return it with the same computational value and phase obligations. No information is erased.

### Expected Counters

```text
pz1050 dirty-borrow C3X:
  removed 88 clean q1047 temp windows
  rewrote 352 uses through dirty q1044

pz1050 q1045 dirty-borrow C3X:
  removed 260 q1045 temp windows
  rewrote 656 uses through dirty q1043

pz1050 q1046 dirty-borrow C3X:
  removed 4 q1046 temp windows
  rewrote 16 uses through dirty q1043
```

## Transformation 3: MBU AND Fanout

### Pattern

Original circuit often does:

```text
temp = a & b
CX temp -> target_1
CX temp -> target_2
...
HMR temp
conditional CZ phase repair
```

If `temp` only exists to fan out `a & b`, replace with direct Toffolis:

```text
CCX a b target_1
CCX a b target_2
...
```

### Why It Is Exact

The measured temp is only a carrier for the Boolean `a & b`. If all fanouts are direct and the phase repair is exactly the standard quadratic CZ correction, the temp can be removed.

### Quantum Meaning

Do not create a measured signpost qubit if the original controls can directly drive the targets.

### Expected Counters

```text
pz1050 MBU AND fanout:
  replaced 5664 measured q1046 temp windows with 10620 direct CCX fanouts
  replaced 11808 measured q1045 temp windows with 23516 direct CCX fanouts
  replaced 5460 measured q1044 temp windows with 10332 direct CCX fanouts
```

## Transformation 4: Inline Single-Temp AND

### Pattern

Some q1046 windows are just:

```text
q1046 ^= src
target ^= q1046 & other
q1046 ^= src
```

Replace with:

```text
target ^= src & other
```

Then retarget the now-useless late reset to a lower already-clean sink when safe.

### Expected Counter

```text
pz1050 inline single-temp AND:
  replaced 88 q1046 windows
  retargeted their late resets to q1043
```

## Reset Handling Rules

Do not blindly delete resets.

Resets/HMR are not ordinary no-ops in this simulator model. They can carry measurement randomness and phase obligations. A reset may be retargeted or removed only when the pass proves the qubit is inactive or already clean at that exact point.

Safe actions used here:

```text
- retarget a reset from a removed high temp to a lower inactive scratch wire
- preserve HMR + phase correction unless the entire measured-AND packet is exactly rewritten
```

Unsafe action:

```text
- broad zero-reset deletion across arbitrary wires
```

That can introduce classical or phase errors.

## Expected Final Build Signature

A successful 1046q build should look roughly like:

```text
pz1050 exact reducer:
  canceled 374891 inverse pairs
  removed 749782 ops
  removed 30776 Toffoli ops

pz1050 exact compactor:
  max q 1050 -> 1048

pz1050 dirty-borrow C3X:
  removed 88 clean q1047 temp windows

pz1050 MBU AND fanout:
  q1046/q1045/q1044 fanout windows replaced

pz1050 q1045 dirty-borrow C3X:
  removed 260 q1045 temp windows

pz1050 q1046 dirty-borrow C3X:
  removed 4 q1046 temp windows

pz1050 inline single-temp AND:
  replaced 88 q1046 windows

pz1050 zero-reset retarget:
  moved 192 q1046 resets

pz1050 exact compactor:
  max q 1046 -> 1046

emitted ops: ~98,806,665
```

## Validation Checklist

After applying the passes:

1. Build the circuit.
2. Run peak scan.
3. Run full evaluator.

Required final validation:

```text
qubits: 1046
classical mismatches: 0
phase-garbage batches: 0
ancilla-garbage batches: 0
avg executed Toffoli: ~32,245,199
experiment OK
```

Do not trust the route if only classical output passes. Phase and ancilla checks are mandatory.

## Mental Model

The four-qubit drop is a sequence of exposed peak owners:

```text
1050 -> 1048:
  reset-bounded wire reuse

1048 -> 1047:
  remove q1047 clean temp C3X windows

1047 -> 1046:
  remove q1046/q1045/q1044 measured AND fanout and small clean-temp packets

final 1046:
  q1046 no longer needed as a clean live temporary
```

Each step exposes the next binder. Do not skip the peak scan between steps.

## Pitfalls

- Do not apply approximate carry truncation.
- Do not use nonce/island search to justify this route.
- Do not drop HMR/reset packets unless their phase obligation is discharged.
- Do not borrow a dirty qubit if it is read before being restored.
- Do not assume a local reduction lowers global peak; always rescan the plateau.

## Summary

The 1050q -> 1046q reduction is an exact width trade:

```text
fewer clean ancillas
+ more direct Toffoli work
+ restored dirty borrows
+ strict reset/HMR preservation
= same computation, lower peak qubits
```

The important reusable technique is not a single peephole. It is the loop:

```text
measure peak
identify live temp owner
replace clean temp with exact dirty/direct form
preserve phase/reset semantics
rescan
repeat
```
