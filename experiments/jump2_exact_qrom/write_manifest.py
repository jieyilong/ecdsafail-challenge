#!/usr/bin/env python3
import argparse
from pathlib import Path

from analyze_results import sha256

HERE=Path(__file__).resolve().parent
REPO=HERE.parents[1]


def write(base, files, destination):
    destination.write_text("".join(f"{sha256(p)}  {p.relative_to(base).as_posix()}\n" for p in sorted(files)))
    print(f"Wrote {len(files)} checksums to {destination}")


if __name__=="__main__":
    p=argparse.ArgumentParser()
    p.add_argument("--workspace", type=Path)
    args=p.parse_args()
    files=[p for p in HERE.rglob("*") if p.is_file() and "__pycache__" not in p.parts and p.name!="ARTIFACT_MANIFEST.sha256"]
    files.append(REPO/"WINDOWED_JUMP2_EXACT_QROM.md")
    write(REPO,files,HERE/"ARTIFACT_MANIFEST.sha256")
    if args.workspace:
        root=args.workspace.resolve()
        local=root/"outputs/ecdsafail-windowed-jump2-exact-qrom-20260907"
        files=[p for p in local.rglob("*") if p.is_file() and p.name!="ARTIFACT_MANIFEST.sha256"]
        files.append(root/"ECDSAFAIL_PINGPONG_VALIDATION_EXPERIMENTS.md")
        write(root,files,local/"ARTIFACT_MANIFEST.sha256")
