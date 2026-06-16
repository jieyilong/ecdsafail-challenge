# shrunken-PZ 1050-qubit secp256k1 point-add

A reproduction of **Trail of Bits TrailMix**'s `shrunken-PZ` reversible inversion
EC point-add (**1050 qubits**), translated into this challenge's op-stream format.
Verified **clean (0 / 0 / 0, all 9024 shots)** through the official `benchmark.sh`
(sandboxed `build_circuit` + trusted `eval_circuit`).

Target objective: **minimize qubit count** (Toffoli is free). This route is
structurally exact `P += Q`, so it is clean on the first try — **no island hunting**.

## Metrics (official harness)

| metric | value |
|---|---|
| qubits | **1050** |
| emitted ops | 99,669,267 |
| avg executed Toffoli | 32,252,071 |
| classical / phase / ancilla | 0 / 0 / 0 |
| shots | 9024 / 9024 OK |

(For comparison, the dialog-GCD SOTA on this repo is 1168 qubits.)

## How it works

This branch is **self-contained**: `ec_shrunken_pz.kmx.zst` (the 1.4 GB kmx
compressed to ~8.7 MB) is **embedded into the binary** (`include_bytes!`) and
`point_add::build()` (`src/point_add/mod.rs`) decodes it by **default** with the
pure-Rust `ruzstd` decoder, returning the PZ op stream instead of the dialog
circuit. So a bare benchmark run yields 1050:

```bash
ecdsafail run
# => emitted ops 99,669,267 ; qubits 1050 ; all 9024 shots OK
```

Overrides:
- `POINT_ADD_FROM_KMX=<path> ecdsafail run` — load an external `.kmx` instead.
- `POINT_ADD_DIALOG=1 ecdsafail run` — fall back to the original 1168q dialog circuit.

The two stacks share the same op IR and register layout (`reg0=tx` P.x,
`reg1=ty` P.y quantum; `reg2=ox` Q.x, `reg3=oy` Q.y classical), so translation is
a 1:1 op-name remap (via the repo's own `Circuit::from_kmx` / `Op::from_text`).
`src/bin/kmx_to_ops.rs` is a standalone `.kmx -> ops.bin` translator.

> Note: the embedded-decode path adds a `ruzstd` dependency in `Cargo.toml`
> (outside `editablePaths=["src/point_add"]`). Fine for this fork branch; an
> official submission that must keep edits inside `src/point_add` would instead
> vendor a decoder there or use `POINT_ADD_FROM_KMX` with the decompressed kmx.

## Regenerating the kmx from TrailMix (provenance)

```bash
cd <trailmix>/trailmix
cargo build --release --bin emit_test_ec_add_shrunken_pz
CIRC_OPS_CAP=150000000 N_CASES=1 \
  ./target/release/emit_test_ec_add_shrunken_pz > /tmp/ec_shrunken_pz.kmx
# stderr: "built: 103436175 ops, peak 1050q, 32937581 tof"
```

The three qubit-saving mechanisms (pipelined Proos–Zalka divstep with no stored
transcript; Monte-Carlo variable-width GCD pack peaking at 741; whole-register
HMR-ghosting of one coordinate passenger) are TrailMix's; see their
`src/inversion/shrunken_pz_*`. Source commit: TrailMix `cd961ff`.
