# Exact QROM cleanup for window-selected ping-pong

This branch starts from `7664bc2`, the validated 1,338-qubit replay-based window adapter. The arithmetic backend, 704-round GCD schedule, all arithmetic widths, coordinate routines, 16-bit window, and runtime two-column table ABI are unchanged. The optimization is enabled explicitly by `WINDOWED_QROM_UNLOAD=split`. The default remains the frozen replay construction.

## Exact transformation

Immediately before unloading, the payload must again hold its table value c[j]. Measuring its b qubits in the X basis with outcome m multiplies the address component by

    2^(-b/2) * (-1)^(m dot c[j]).

The magnitude is independent of j. The known phase is canceled without recomputing c[j] into a quantum payload. This is measurement-based uncomputation, following the phase-correction approach of Gidney, *Windowed quantum arithmetic*, arXiv:1905.07682, Section 2: https://arxiv.org/html/1905.07682.

Split the w-bit address into floor(w/2) low bits and ceil(w/2) high bits. Decode each half into a one-hot register. For each measured payload bit, condition on its measurement outcome and apply a CZ between the two selected one-hot flags, conditioned additionally on the corresponding runtime table bit. The only active flag pair is the one belonging to j, so these commuting phases multiply to (-1)^(m dot c[j]). Uncompute both decoders unitarily.

The decoder splits a parent flag S into S(1-a) and Sa with one CCX and one CX. Its inverse reverses these two gates. No arithmetic estimate or probability cutoff is used. For two table columns, both phase contributions share the decoders. For the K=2 forward lookup, first undo the low-address controlled swaps. Measure both restored banks and correct their combined phase as a function of the high address.

All phase corrections commute. Keeping both decoders resident permits the measurement condition to be hoisted outside the row loop. Each row then uses one CZ instruction with the table bit as its classical condition. This avoids expanding each payload-bit correction into nested condition-stack instructions.

## Cost and width

For an unblocked w-bit lookup, this implementation's decoder computation and inverse cost

    C(w) = 2 * (2^floor(w/2) + 2^ceil(w/2)) - 4

Toffolis. The two decoders occupy the sum of the two powers of two above. We use unitary decoder inverses for a simple exact implementation, rather than claiming the smallest known asymptotic constant for lookup uncomputation.

At w=16, each paired-column unload costs C(16)=1,020 Toffolis instead of 65,534. The single-column K=2 unload costs 256+C(15)=1,020 instead of 33,022. Across the three unloads this saves exactly 161,030 static Toffolis. Forward lookup costs do not change.

The complete emitted circuit has peak Q=1,338, unchanged from replay. Decoder workspace is allocated only after the payload is measured and released. With X, Y, J, and the identity flag live, the largest decoder phase has 512+16+1+512=1,041 live qubits. The existing arithmetic phase remains width-determining.

The completed shared 100,000-input evaluation measures mean T=1,127,527.25496 and Q*T=1,508,631,467.13648, a 12.4971% reduction from replay at unchanged Q. It records 35 any-channel failures versus 36 for replay, with identical classical-output failure sets and no ancilla failures. The pass fraction is 0.99965 and the per-call retry-sensitivity proxy is 1,509,159,673.02204. This one-failure difference is not evidence of better arithmetic accuracy. Full records and qualifications are in [the experiment report](experiments/exact_qrom/EXPERIMENTS.md#exact-qrom-cleanup-follow-up).

## Scope

The cleanup transformation is exact under the restored-table-payload invariant. It adds no arithmetic approximation. This does not make the inherited point-addition circuit exact on all inputs. Its finite-round and truncated-arithmetic failures remain possible. On inputs where arithmetic already violates the payload-restoration invariant, the two implementations need not have identical erroneous outputs or measurement histories. Different random phase outcomes alone are not evidence of a new approximation.

## Validation and reproduction

```sh
cargo build --release --locked --offline --bin build_circuit --bin eval_circuit

python3 experiments/exact_qrom/run_experiment.py selfcheck --output /path/to/results
python3 experiments/exact_qrom/run_experiment.py pilot --output /path/to/results --mode split
python3 experiments/exact_qrom/run_experiment.py full --output /path/to/results --mode split --threads 8
```

The self-check covers all 144 measurement branches in three reduced circuits, all address components at w=0..6 under four prescribed measurement records for each cleanup mode, and all 65,536 addresses for each of three w=16 lookup layouts. The full-address test uses injective 16-bit payload labels. Arbitrary 256-bit payloads are checked separately at small widths. These are branchwise basis-image checks with a normalization argument, not a dense simulation of the complete elliptic-curve state vector.

All complete-circuit outcomes are recorded without selecting a nonce or discarding failures. The primary comparison uses the existing 100,000-input corpus for direct pairing. Results and their statistical qualifications are recorded in `experiments/exact_qrom/` and the paper workspace's `ECDSAFAIL_PINGPONG_VALIDATION_EXPERIMENTS.md`.
