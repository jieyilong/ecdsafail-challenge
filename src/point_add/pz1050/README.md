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

`point_add::build()` (`src/point_add/mod.rs`) has a small env gate: if
`POINT_ADD_FROM_KMX=<path>` is set it returns the op stream parsed from that
`.kmx` via the repo's own `Circuit::from_kmx`, instead of building the dialog
circuit. The two stacks share the same op IR and register layout
(`reg0=tx` P.x, `reg1=ty` P.y quantum; `reg2=ox` Q.x, `reg3=oy` Q.y classical),
so the translation is a 1:1 op-name remap.

The 1.4 GB kmx is bundled here **compressed** (`ec_shrunken_pz.kmx.zst`, ~8 MB,
~159x). Decompress + run:

```bash
bash src/point_add/pz1050/run_pz_1050.sh
# or manually:
zstd -d --long=27 src/point_add/pz1050/ec_shrunken_pz.kmx.zst -o /tmp/ec_shrunken_pz.kmx
POINT_ADD_FROM_KMX=/tmp/ec_shrunken_pz.kmx ecdsafail run
# => qubits 1050, all 9024 shots OK
```

`src/bin/kmx_to_ops.rs` is a standalone translator (`.kmx` -> `ops.bin`) if you
prefer to bypass `build()`.

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
