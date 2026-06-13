# q1194 K5 Step-Frame Island

Base: promoted submission `833642f` K5 lineage.

Route:

- `DIALOG_GCD_K5_APPLY_STEP_FRAME=1`
- `SQUARE_ROW_MAX_SEG=167`
- `DIALOG_TAIL_NONCE=150427664`

Validation:

- Distributed q1194 validators were rebuilt from this exact source on Linux and
  cross-checked against local anchors before use.
- The stale remote validators previously reported q=1216 and mismatched
  `cls / pha / anc`; those results were discarded.
- Nonce `150427664` validated locally and remotely with `0 / 0 / 0` over all
  9024 trusted eval shots.

Measured local trusted eval:

- qubits: 1194
- average executed Toffoli: 1,446,487.888
- total Toffoli: 13,053,106,700 over 9024 shots
- emitted ops: 10,308,258

This route is a low-qubit artifact rather than a score-SOTA candidate. Do not
submit it to the ecdsafail site unless the measured score beats current SOTA,
but preserve and push the validated challenge branch for future work.
