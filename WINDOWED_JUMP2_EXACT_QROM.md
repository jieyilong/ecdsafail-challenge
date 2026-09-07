# Exact QROM cleanup for Windowed 8e9c9a2

This branch applies the exact split-address cleanup to the two-column Windowed 8e9c9a2 circuit at `15a29ac9b1e97fc21eb81b9a6f0f6d9c4dc7a92e`. It does not use the later three-column circuit. The QROM implementation and dedicated checks are copied unchanged from the validated ping-pong cleanup source at `2e0187c99fef038f079aaf76b6d36f64cce7481e`, released with data at `c6ad760`.

Select the new path with `WINDOWED_QROM_UNLOAD=split`. The default remains replay. Rebuilding replay from this branch reproduces the original compressed operation stream byte for byte, SHA-256 `cb820d2829750cc73814ae1ea2ceb908696462270b3f5a60ebbc7128f4c0e274`.

## Exactness and scope

After use, the selected payload must again equal its runtime table entry c[j]. Measuring the payload in the X basis produces an address-dependent sign (-1)^(m dot c[j]) with input-independent branch magnitude. Two one-hot address decoders and classically conditioned CZ gates cancel this sign exactly. The decoders are uncomputed unitarily. For K=2 lookup, reverse the low-address swaps first and correct the joint phase of both restored banks.

The phase-correction approach follows Gidney, *Windowed quantum arithmetic*, Section 2: https://arxiv.org/html/1905.07682. Our implementation computes and reverses both decoders, rather than claiming the smallest known constant for lookup uncomputation. The measurement condition is hoisted outside the table-row loop because the diagonal corrections commute.

The arithmetic, GCD schedule, widths, modular comparisons, square, and two-column runtime ABI are unchanged. No new arithmetic approximation is introduced. The transformation is exact under the restored-payload invariant, not a proof of all-input correctness of the inherited approximate arithmetic. It remains a single-call window-selected addition, not a complete Shor implementation.

## Static resource change

Each paired-column unload falls from 65,534 to 1,020 raw Toffolis. The K=2 single-column unload falls from 33,022 to 1,020. The direct reduction is 161,030 Toffolis. The unchanged constant-propagation pass additionally folds six decoder gates with known-one controls into CNOTs. Thus the final static count falls from 1,754,382 to 1,593,346, a reduction of 161,036.

The full 16-bit circuit has Q=1,162, unchanged. The decoder phase uses about 1,041 live qubits after the selected payload is released, below the original arithmetic peak. The full operation stream has 212,090,239 records, down from 213,387,745. These records include metadata and classical-control instructions, not just quantum gates.

## Reproduction

```sh
cargo build --release --locked --offline --bin build_circuit --bin eval_circuit

python3 experiments/jump2_exact_qrom/run_experiment.py selfcheck --output /path/to/results
python3 experiments/jump2_exact_qrom/run_experiment.py pilot --mode split --output /path/to/results
python3 experiments/jump2_exact_qrom/run_experiment.py full --mode split --output /path/to/results --threads 8
```

The runner uses the shared seed `ecdsafail-windowed-independent-random-100k-v1`, 100,000 cases, a 16-bit quantum address, `SINGLE_CCX_FANOUT_DISABLE=1`, and the unchanged evaluator. Its allowed-error setting collects all failures. It is not an acceptance threshold. The arithmetic development pilots use a distinct seed.

The tests cover all 144 HMR branches in three reduced layouts, arbitrary 256-bit payloads at small widths, and all 65,536 addresses in the three full-width routing layouts. The port adds only the QROM files and test access. The test hook is kept on the existing build-function opening line to preserve inherited call-site line numbers. The evaluator, simulator, and dependency files remain unchanged.

The experiment will compare replay Jump-2, exact-cleanup Jump-2, and exact-cleanup ping-pong on identical input records. This closes the previous cleanup mismatch between the two backends. Results and data are recorded under `experiments/jump2_exact_qrom/` and in the paper workspace's shared experiment report.
