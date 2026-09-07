# Exact cleanup experiment

See [EXACT_QROM.md](../../EXACT_QROM.md) for the construction, its exactness argument, gate formulas, and measured peak width. `PROTOCOL_FROZEN.md` fixes the evaluation policy. The implementation is frozen at `2e0187c99fef038f079aaf76b6d36f64cce7481e`.

The comparison uses the identical 100,000-input corpus already released in `../windowed_pingpong/results/corpus.tsv.gz`. The baseline ledgers are also reused from that release. No baseline has been regenerated with a different seed or treated as newly sampled data.

## Reproduction

```sh
cargo build --release --locked --offline --bin build_circuit --bin eval_circuit
python3 experiments/exact_qrom/run_experiment.py selfcheck --output /path/to/results
python3 experiments/exact_qrom/run_experiment.py pilot --output /path/to/results --mode split
python3 experiments/exact_qrom/run_experiment.py full --output /path/to/results --mode split --threads 8
python3 experiments/exact_qrom/analyze.py --verify-release
```

`full --reuse` resumes against an existing operation file and matching checkpoint. The evaluator rejects a mismatched checkpoint fingerprint. To collect replay pilots, select `--mode replay`. The intermediate `measurement` cleanup is tested by the reduced self-check but was not evaluated on a full new 100,000-input corpus. Its previously stated 1.207M estimate must not be reported as a measured result.

## Released data

`results/split/outcomes.tsv.gz` retains all optimized-circuit outputs and failure flags, with equality to an expected coordinate or address encoded as `=`. Input coordinates and expected outputs are read from the unchanged shared corpus. Every observed output can therefore be reconstructed exactly. `analyze.py` verifies reconstruction, every batch aggregate, complete index coverage, and all paired statistics. The scripts for the prior release remain unchanged.

`results/analysis.json` contains the optimized and reference operating points, paired outcomes, error intervals, static counts, source revision, and original ledger hashes. `results/pilots` and the self-check logs retain the development results, including failures. `provenance.json` pins the operation stream and trusted sources. Raw local artifacts are retained under `outputs/ecdsafail-pingpong-exact-qrom-20260907` in the paper workspace.

## Interpretation

Only QROM cleanup changes. Non-QROM arithmetic and evaluator files are byte-identical to the parent. The generic cleanup technique is not specific to ping-pong and could also be applied to other backends. Comparisons against the old windowed Jump-2 circuit therefore compare that published replay-based artifact, not a Jump-2 circuit given the same cleanup improvement.

Mean executed costs include failed inputs. Failure categories overlap. Primary paired accuracy comparisons exclude the two identity rows that mixed addition does not support. The corpus was previously studied, so this is exploratory evidence. The per-call Q*T/pass-fraction proxy is not a full-Shor success or cost model. Executed measurement counts and feed-forward/Toffoli depth remain unmeasured, and inherited arithmetic approximations remain in place.
