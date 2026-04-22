# Pair1 forward-end findings

On the actual first strict phase-failing batch for `k = 4` (batch 10), the
internal Kaliski forward-end terminal registers for pair1 still match the known
post-forward invariant in both the generic and experimental circuits:

- `u = 1`
- `v_w = 0`
- `f_flag = 0` on all shots except the same single-shot mask seen in the global
  phase (`0x0000040000000000`)

Measured masks:
- generic: `u_bad=0`, `vw_any=0x0000000000000000`, `f_mask=0x0000040000000000`
- special: `u_bad=0`, `vw_any=0x0000000000000000`, `f_mask=0x0000040000000000`

## Interpretation
This rules out a simple dirty-free explanation for the pair1 terminal frees of:
- `u`,
- `v_w`,
- or `f_flag`.

So the remaining phase bug is not caused by those registers failing to reach
their known terminal classical values on the failing batch.

Combined with the earlier pair1 body finding, the strongest concrete suspect
remains the phase interaction around:
- `pair1_halve`,
- `pair1_mul2`,
- and the way the experimental prefix changes the phase context rather than the
  classical terminal state itself.
