#!/usr/bin/env python3
"""Analyze replay/split Jump-2 and split ping-pong on the same complete corpus."""
import argparse
import json
import math
from pathlib import Path
import shutil
import time

from analyze_results import META, OUT, FLAGS, N, read_case, paired, wilson, tsv, sha256, write_gzip_tsv

HERE = Path(__file__).resolve().parent


def binomial_cdf(k, n, p):
    term = n * math.log1p(-p)
    total = term
    for j in range(1, k + 1):
        term += math.log(n - j + 1) - math.log(j) + math.log(p) - math.log1p(-p)
        high, low = max(total, term), min(total, term)
        total = high + math.log1p(math.exp(low - high))
    return min(1.0, math.exp(total))


def invert_cdf(k, n, target):
    lo, hi = 0.0, 1.0
    for _ in range(70):
        p = (lo + hi) / 2
        if binomial_cdf(k, n, p) > target:
            lo = p
        else:
            hi = p
    return hi


def conservative_difference_upper(candidate_only, baseline_only, n):
    # Bonferroni: simultaneous one-sided 97.5% Clopper-Pearson bounds.
    upper = 1.0 if candidate_only == n else invert_cdf(candidate_only, n, 0.025)
    lower = 0.0 if baseline_only == 0 else invert_cdf(baseline_only - 1, n, 0.975)
    return upper - lower


def summarize(split, export=None):
    refs = HERE / "references"
    corpus = {int(r["index"]): r for r in tsv(refs / "corpus.tsv.gz")}
    cases = {"jump2_replay": refs / "jump2_replay", "jump2_split": split, "pingpong_split": refs / "pingpong_split"}
    ledgers, metrics = {}, {}
    for name, path in cases.items():
        rows, manifest, batches = read_case(path, corpus)
        ledgers[name] = rows
        for i in range(N):
            assert tuple(rows[i][f] for f in META) == tuple(corpus[i][f] for f in META), (name, i)
        counts = {f: sum(int(r[f]) for r in rows.values()) for f in FLAGS}
        q = int(manifest["qubits"])
        mean_t = sum(int(b["toffoli"]) for b in batches) / N
        mean_c = sum(int(b["clifford"]) for b in batches) / N
        identity = [i for i, r in rows.items() if int(r["address"]) == 0]
        identity_fail = sum(int(rows[i]["any_failure"]) for i in identity)
        f = counts["any_failure"]
        metrics[name] = dict(qubits=q, mean_toffoli=mean_t, mean_clifford=mean_c,
                             static_operations=int(manifest["ops"]), ops_fingerprint=manifest["ops_fingerprint"],
                             **counts, pass_rate=1-f/N, failure_rate_wilson_95=wilson(f, N),
                             raw_qxt=q*mean_t, retry_adjusted_qxt=q*mean_t/(1-f/N),
                             identity_indices=sorted(identity), identity_failures=identity_fail,
                             nonidentity_failures=f-identity_fail, nonidentity_n=N-len(identity),
                             address_failures=sum(int(r["address"]) != int(r["got_address"],16) for r in rows.values()))
    common = [i for i, r in corpus.items() if int(r["address"]) != 0 and r["target_x"] != r["addend_x"] and r["expected_x"] != r["addend_x"]]
    result = dict(tests=N, common_domain_size=len(common), corpus_mismatches=0,
                  corpus_file_sha256=sha256(refs/"corpus.tsv.gz"), circuits=metrics, paired={})
    for a, b in (("jump2_replay", "jump2_split"), ("jump2_split", "pingpong_split")):
        result["paired"][a+"_vs_"+b] = {}
        for scope, indices in (("all_inputs", list(range(N))), ("common_domain", common)):
            stats = paired(ledgers[a], ledgers[b], indices)
            bound = conservative_difference_upper(stats["candidate_only_fail"], stats["baseline_only_fail"], len(indices))
            stats["conservative_exact_95_upper_on_failure_rate_difference"] = bound
            stats["exact_upper_below_exploratory_margin"] = bound <= 0.0005
            result["paired"][a+"_vs_"+b][scope] = stats
    old, new = ledgers["jump2_replay"], ledgers["jump2_split"]
    result["classical_discordance"] = {
        "candidate_only_indices": [i for i in common if int(new[i]["classical_failure"]) and not int(old[i]["classical_failure"])],
        "baseline_only_indices": [i for i in common if int(old[i]["classical_failure"]) and not int(new[i]["classical_failure"])],
    }
    keys = ("qubits", "mean_toffoli", "mean_clifford", "raw_qxt", "retry_adjusted_qxt")
    result["jump2_change_percent"] = {k:100*(metrics["jump2_split"][k]/metrics["jump2_replay"][k]-1) for k in keys}
    result["matched_pingpong_change_percent"] = {k:100*(metrics["pingpong_split"][k]/metrics["jump2_split"][k]-1) for k in keys}
    if export:
        export.mkdir(parents=True, exist_ok=True)
        packed=[]
        for i in range(N):
            row=dict(new[i])
            for got, expected in (("got_x", "expected_x"), ("got_y", "expected_y"), ("got_address", "address")):
                equal = int(row[got],16)==int(row[expected]) if expected=="address" else row[got]==row[expected]
                if equal: row[got]="="
            packed.append(row)
        write_gzip_tsv(export/"outcomes.tsv.gz", OUT, packed)
        for f in ("manifest.tsv", "batches.tsv"):
            shutil.copy2(split/f, export/f)
        rebuilt, _, _ = read_case(export, corpus)
        assert rebuilt == new
        assert summarize(export) == result
    return result


def package(workspace, wait):
    local=workspace/"outputs/ecdsafail-windowed-jump2-exact-qrom-20260907"
    deadline=time.monotonic()+7200
    while not (local/"split/timing.json").exists():
        if not wait or time.monotonic()>deadline: raise RuntimeError("Full evaluation is not complete")
        time.sleep(30)
    release=HERE/"results"
    report=summarize(local/"split/checkpoint", release/"split")
    config=json.loads((local/"split/configuration.json").read_text())
    static=json.loads((local/"split/static_resources.json").read_text())
    assert static["static_toffoli"]==1593346 and report["circuits"]["jump2_split"]["qubits"]==1162
    report.update(source_commit=config["source_commit"], parent_commit="15a29ac9b1e97fc21eb81b9a6f0f6d9c4dc7a92e",
                  cleanup_donor="2e0187c99fef038f079aaf76b6d36f64cce7481e", static_resources=static,
                  raw_ledger_sha256={f:sha256(local/"split/checkpoint"/f) for f in ("inputs.tsv", "batches.tsv", "manifest.tsv")})
    for f in (local/"analysis.json", release/"analysis.json"):
        f.write_text(json.dumps(report, indent=2, sort_keys=True)+"\n")
    for f in ("build.log", "eval.log", "configuration.json", "timing.json", "static_resources.json"):
        (release/"split"/f).write_text((local/"split"/f).read_text().replace(str(workspace), "<paper-workspace>"))
    shutil.copy2(local/"selfcheck.log", release/"selfcheck.log")
    for case in sorted(local.glob("pilot-*")):
        dest=release/"pilots"/case.name
        dest.mkdir(parents=True, exist_ok=True)
        for f in ("build.log", "eval.log"):
            if (case/f).exists(): shutil.copy2(case/f, dest/f)
    print(json.dumps({k:report[k] for k in ("circuits", "jump2_change_percent", "matched_pingpong_change_percent", "classical_discordance")}, indent=2))
    print("All raw and released ledger statistics agree.")


if __name__=="__main__":
    p=argparse.ArgumentParser()
    p.add_argument("--workspace", type=Path)
    p.add_argument("--wait", action="store_true")
    p.add_argument("--verify-release", action="store_true")
    args=p.parse_args()
    if args.verify_release:
        rebuilt=summarize(HERE/"results/split")
        saved=json.loads((HERE/"results/analysis.json").read_text())
        assert all(saved[k]==v for k,v in rebuilt.items())
        print("All released statistics verified.")
    else:
        if args.workspace is None: p.error("--workspace is required")
        package(args.workspace.resolve(), args.wait)
