#!/usr/bin/env python3
"""Isolated runs of exact QROM cleanup, keeping the arithmetic settings fixed."""
import argparse
import json
from pathlib import Path
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "windowed_pingpong"))
from run_cases import REPO, SEED, clean_environment, run_logged

PILOT_SEED = "ecdsafail-exact-qrom-development-20260907-v1"


def execute(out, mode, width, tests, calls, seed, threads, tag, reuse=False):
    case = out / tag
    case.mkdir(parents=True, exist_ok=True)
    env = clean_environment()
    env.update(WINDOWED_MODE="1", WINDOW_BITS=str(width), WINDOWED_QROM_UNLOAD=mode,
               SINGLE_CCX_FANOUT_DISABLE="1", WINDOWED_TESTS=str(tests),
               WINDOWED_SEQUENCE_CALLS=str(calls), WINDOWED_MAX_ERROR_RATE="1",
               EVAL_SHARED_SEED=seed, EVAL_THREADS=str(threads))
    if calls == 1:
        env["EVAL_CHECKPOINT_DIR"] = str(case / "checkpoint")
    config = {k: v for k, v in env.items() if k not in clean_environment()}
    config["source_commit"] = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip()
    (case / "configuration.json").write_text(json.dumps(config, indent=2) + "\n")
    if not reuse:
        run_logged([str(REPO / "target/release/build_circuit")], case, env, case / "build.log")
    elapsed = run_logged([str(REPO / "target/release/eval_circuit"), "--note", tag], case, env, case / "eval.log")
    (case / "timing.json").write_text(json.dumps({"evaluation_wall_seconds": elapsed}, indent=2) + "\n")


if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("task", choices=["selfcheck", "pilot", "full", "build"])
    p.add_argument("--output", type=Path, required=True)
    p.add_argument("--mode", choices=["replay", "measurement", "split"], default="split")
    p.add_argument("--threads", type=int, default=8)
    p.add_argument("--reuse", action="store_true")
    args = p.parse_args()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    if args.task == "selfcheck":
        env = clean_environment()
        env["WINDOWED_QROM_BRANCH_SELFTEST"] = "1"
        run_logged([str(REPO / "target/release/build_circuit")], out, env, out / "selfcheck.log")
    elif args.task == "pilot":
        for width in [*range(7), 10, 12]:
            execute(out, args.mode, width, 256, 4, PILOT_SEED, 1, f"pilot-{args.mode}-w{width}")
    elif args.task == "build":
        case = out / args.mode
        case.mkdir(exist_ok=True)
        env = clean_environment()
        env.update(WINDOWED_MODE="1", WINDOW_BITS="16", WINDOWED_QROM_UNLOAD=args.mode, SINGLE_CCX_FANOUT_DISABLE="1")
        run_logged([str(REPO / "target/release/build_circuit")], case, env, case / "build.log")
    else:
        execute(out, args.mode, 16, 100000, 1, SEED, args.threads, args.mode, reuse=args.reuse)
