#!/usr/bin/env python3
"""Evaluate the two-column windowed Jump-2 circuit with exact QROM cleanup."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time

REPO = Path(__file__).resolve().parents[2]
SEED = "ecdsafail-windowed-independent-random-100k-v1"
PILOT = "ecdsafail-jump2-exact-qrom-development-20260907-v1"


def clean_environment():
    env = dict(os.environ)
    prefixes = ("WINDOW", "SUB4_", "EVAL_", "MIXED_WINDOW_", "TLM_", "DIALOG_", "POINT_ADD_", "SINGLE_CCX_", "PROFILE_", "RUSTFLAGS")
    for key in list(env):
        if key.startswith(prefixes):
            del env[key]
    return env


def run_logged(command, cwd, env, path):
    start = time.monotonic()
    with path.open("w") as log:
        process = subprocess.Popen(command, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        for line in process.stdout:
            log.write(line)
            log.flush()
            print(line, end="", flush=True)
        code = process.wait()
    if code:
        raise RuntimeError(f"Command exited {code}; see {path}")
    return time.monotonic() - start


def execute(out, mode, width, tests, calls, seed, threads, tag, reuse=False, build_only=False):
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
    if build_only:
        return
    elapsed = run_logged([str(REPO / "target/release/eval_circuit"), "--note", tag], case, env, case / "eval.log")
    (case / "timing.json").write_text(json.dumps({"evaluation_wall_seconds": elapsed}, indent=2) + "\n")


if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("task", choices=["selfcheck", "pilot", "build", "full"])
    p.add_argument("--mode", choices=["replay", "split"], default="split")
    p.add_argument("--output", type=Path, required=True)
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
        for w in [*range(7), 10, 12]:
            execute(out, args.mode, w, 256, 4, PILOT, 1, f"pilot-{args.mode}-w{w}")
    else:
        execute(out, args.mode, 16, 100000, 1, SEED, args.threads, args.mode,
                reuse=args.reuse, build_only=args.task == "build")
