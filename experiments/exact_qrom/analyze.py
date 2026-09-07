#!/usr/bin/env python3
"""Reconstruct the exact-cleanup comparison from complete input/outcome ledgers."""
import argparse
import json
import math
from pathlib import Path
import shutil
import sys
import time

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
sys.path.insert(0, str(HERE.parent / "windowed_pingpong"))
from analyze_results import analyze, paired, read_case, sha256, tsv, write_gzip_tsv, OUT


def exact_upper(failures, shots, alpha=0.05):
    """One-sided Clopper-Pearson bound, used here for rare discordant events."""
    lo, hi = 0.0, 1.0
    for _ in range(80):
        rate = (lo + hi) / 2
        term = math.exp(shots * math.log1p(-rate))
        cdf = term
        for k in range(1, failures + 1):
            term *= (shots - k + 1) / k * rate / (1 - rate)
            cdf += term
        if cdf > alpha:
            lo = rate
        else:
            hi = rate
    return hi


def summarize(split, export=None):
    old = HERE.parent / "windowed_pingpong/results"
    corpus_path = old / "corpus.tsv.gz"
    corpus = {int(row["index"]): row for row in tsv(corpus_path)}
    cases = {
        "mixed_pingpong": old / "mixed_pingpong",
        "windowed_replay": old / "windowed_pingpong",
        "windowed_jump2": old / "windowed_jump2",
        "windowed_pingpong": split,
    }
    result = analyze(cases, corpus_path)
    before, _, _ = read_case(cases["windowed_replay"], corpus)
    after, _, _ = read_case(split, corpus)
    common = [i for i, r in corpus.items() if int(r["address"]) != 0 and r["target_x"] != r["addend_x"] and r["expected_x"] != r["addend_x"]]
    result["paired"]["windowed_replay_vs_windowed_pingpong"] = {
        "all_inputs": paired(before, after, list(range(100000))),
        "common_domain": paired(before, after, common),
    }
    for comparison in result["paired"]["windowed_replay_vs_windowed_pingpong"].values():
        # Failure-rate increase is at most the candidate-only failure probability.
        comparison["conservative_exact_one_sided_95_upper_on_regression"] = exact_upper(comparison["candidate_only_fail"], comparison["n"])
    new_classical = [i for i in common if int(after[i]["classical_failure"]) and not int(before[i]["classical_failure"])]
    removed_classical = [i for i in common if int(before[i]["classical_failure"]) and not int(after[i]["classical_failure"])]
    result["classical_discordance"] = dict(candidate_only_indices=new_classical, baseline_only_indices=removed_classical)
    result["source_commit"] = "2e0187c99fef038f079aaf76b6d36f64cce7481e"
    result["parent_commit"] = "7664bc2"
    result["cleanup_mode"] = "split"
    a, b = result["circuits"]["windowed_pingpong"], result["circuits"]["windowed_replay"]
    result["relative_change_percent"] = {k: 100 * (a[k] / b[k] - 1) for k in ("qubits", "mean_toffoli", "mean_clifford", "raw_qxt", "retry_adjusted_qxt", "static_operations")}
    if export:
        export.mkdir(parents=True, exist_ok=True)
        compact = []
        for i in range(100000):
            row = dict(after[i])
            for got, expected in (("got_x", "expected_x"), ("got_y", "expected_y"), ("got_address", "address")):
                same = int(row[got], 16) == int(row[expected]) if expected == "address" else row[got] == row[expected]
                if same:
                    row[got] = "="
            compact.append(row)
        write_gzip_tsv(export / "outcomes.tsv.gz", OUT, compact)
        for f in ("manifest.tsv", "batches.tsv"):
            shutil.copy2(split / f, export / f)
        reconstructed, _, _ = read_case(export, corpus)
        assert reconstructed == after, "Released outcome reconstruction differs from raw inputs"
        assert summarize(export) == result
    return result


def package(workspace, wait):
    local = workspace / "outputs/ecdsafail-pingpong-exact-qrom-20260907"
    stop = time.monotonic() + 7200
    while not (local / "split/timing.json").exists():
        if not wait or time.monotonic() > stop:
            raise RuntimeError("Full evaluation has not completed successfully")
        time.sleep(30)
    release = HERE / "results"
    report = summarize(local / "split/checkpoint", release / "split")
    static = json.loads((local / "split/static_resources.json").read_text())
    old_static = json.loads((HERE.parent / "windowed_pingpong/results/analysis.json").read_text())["static_resources"]["windowed_pingpong"]
    assert old_static["static_toffoli"] - static["static_toffoli"] == 161030
    assert report["circuits"]["windowed_pingpong"]["qubits"] == 1338
    report["static_resources"] = static
    report["static_toffoli_saving"] = 161030
    report["raw_ledger_sha256"] = {f: sha256(local / "split/checkpoint" / f) for f in ("inputs.tsv", "batches.tsv", "manifest.tsv")}
    content = json.dumps(report, indent=2, sort_keys=True) + "\n"
    (local / "analysis.json").write_text(content)
    (release / "analysis.json").write_text(content)
    for f in ("build.log", "eval.log", "configuration.json", "timing.json", "static_resources.json"):
        source = local / "split" / f
        (release / "split" / f).write_text(source.read_text().replace(str(workspace), "<paper-workspace>"))
    for f in ("selfcheck.log", "exhaustive-branches.log"):
        shutil.copy2(local / f, release / f)
    for mode in ("replay", "split"):
        for w in [*range(7), 10, 12]:
            tag = f"pilot-{mode}-w{w}"
            dest = release / "pilots" / tag
            dest.mkdir(parents=True, exist_ok=True)
            for f in ("build.log", "eval.log"):
                shutil.copy2(local / tag / f, dest / f)
    print(json.dumps({k: report[k] for k in ("circuits", "relative_change_percent", "classical_discordance")}, indent=2))
    print("Exact-cleanup release reconstructed and verified.")


if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("--workspace", type=Path)
    p.add_argument("--wait", action="store_true")
    p.add_argument("--verify-release", action="store_true")
    args = p.parse_args()
    if args.verify_release:
        rebuilt = summarize(HERE / "results/split")
        saved = json.loads((HERE / "results/analysis.json").read_text())
        assert all(saved[key] == value for key, value in rebuilt.items())
        print("All released aggregate and paired statistics verified.")
    else:
        if args.workspace is None:
            p.error("--workspace is required for packaging")
        package(args.workspace.resolve(), args.wait)
