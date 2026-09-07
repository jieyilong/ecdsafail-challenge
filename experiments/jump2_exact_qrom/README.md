# Windowed Jump-2 exact-cleanup experiment

The circuit is frozen at `080452804328698214f36655d95c9883be6a3080`. See [WINDOWED_JUMP2_EXACT_QROM.md](../../WINDOWED_JUMP2_EXACT_QROM.md) for the port, exactness conditions, gate-count decomposition, and reproduction commands. The parent is the two-column Windowed 8e9c9a2 source `15a29ac`, not the later three-column variant.

## Data and analysis

`references/corpus.tsv.gz` stores all 100,000 input and expected-output records. The replay Jump-2 and exact-cleanup ping-pong outcome files are copied from the previously published complete ledgers. They are historical references, not newly sampled runs. The new outcome file is `results/split/outcomes.tsv.gz`. An observed-value field equal to `=` means exact equality with the corresponding expected coordinate or address in the shared corpus. This removes repeated data without dropping successful cases or failed outputs.

`analyze.py` reconstructs the observations, verifies every input and expected output, checks batch totals and complete index coverage, and computes aggregate and paired statistics. Its generic parsing/statistics helper, static counter, fingerprint checker, and basic tests are reused unchanged from the earlier validated release. The new analysis wrapper adds a matched-cleanup backend comparison and conservative paired-difference bounds.

```sh
python3 experiments/jump2_exact_qrom/analyze.py --verify-release
python3 -m unittest discover -s experiments/jump2_exact_qrom -p 'test_*.py'
```

For each paired difference, an additional conservative 95% upper bound subtracts a one-sided 97.5% Clopper-Pearson lower bound on baseline-only failures from the corresponding upper bound on candidate-only failures. Bonferroni coverage does not require those two counts to be independent. Approximate paired normal intervals and exact paired sign tests are also reported, with their methods labeled.

The source and protocol were frozen before the full optimized run. No arithmetic setting or nonce was adjusted after its outcomes were viewed. This is an exploratory comparison on a previously studied corpus, not a new blinded confirmation.

## Limits

Exact cleanup introduces no additional arithmetic approximation under the restored-payload invariant. The inherited arithmetic can still fail. The circuit implements one coherent window-selected point-addition call, not full Shor. Q*T/pass fraction is only a per-call retry-sensitivity proxy.

Mean Toffoli costs include failures and are calculated from exact per-batch totals. Individual input costs, executed measurement counts, and Toffoli/feed-forward depth are not available from this evaluator. Its counted Clifford metric excludes tracked X/Z operations and includes CX, CZ, SWAP, HMR, and reset. Total serialized operations also include metadata and classical-control instructions.

The local raw experiment is stored under `outputs/ecdsafail-windowed-jump2-exact-qrom-20260907` in the paper workspace. The code, compressed ledgers, logs, manifests, and shared report are released in this branch.
