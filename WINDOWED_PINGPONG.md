# Window-selected ping-pong point addition

This branch adapts submission 3616dbf2-f32d-41c8-9ea2-7a5bbe372fc0, source commit 897dda2b0cf267151ecd973252d2a5078cbf1b63. The lookup interface and evaluator are ported from the two-column Jump-2 adapter at 15a29ac9b1e97fc21eb81b9a6f0f6d9c4dc7a92e.

## Implemented map

For a classical runtime table of affine point coordinates and a quantum address J, the circuit targets

    |j>|R>|0> -> |j>|R + T[j]>|0>.

Its register ABI is X, Y, J, Tx[0], Ty[0], ..., Tx[2^w-1], Ty[2^w-1]. X and Y each contain 256 qubits. J contains w qubits. Each table coordinate is a 256-bit classical input register. Row zero must contain (0,0), encoding the identity point. Other rows must contain valid affine points.

## Circuit changes

1. Stages 1 and 6 load the selected coordinates together through the donor's unary traversal and clear them by exact payload replay.
2. Stage 3 uses the donor's K=2 QROAM at w=16. It loads x_A, applies x_A and 2*x_A, reverses the temporary doubling, and unloads x_A. No third table column is introduced.
3. Stages 2 and 5 call the original ping-pong in-place division/multiplication routines. Their 704-round schedule, sign decisions, transcript, and fused replay remain unchanged.
4. A coherent flag b=[J!=0] controls the square and final modular negation. Stage 4 computes a temporary masked slope M=b*lambda with 256 CCX gates, applies the original product-register square to M, and reverses the mask with 256 CCX gates. For b=0 the square contribution is zero.
5. Carry venting in the product-register square is capped by the remaining width budget, 1321+w+1 minus the current live allocation. The remaining carry bits are uncomputed with reverse Toffolis. This changes the exact adder schedule, with no additional arithmetic truncation.
6. The donor's low-peak coordinate routines are retained, including its 64-bit upper comparison and inherited short modular corrections.

With J=0, the arithmetic division and multiplication act on the same denominator and cancel on paths satisfying the arithmetic assumptions. All loaded coordinate words are zero, and the square and final negation are disabled. The accumulator is therefore restored without measuring J. A=O is supported on this basis. R=O and R in {A,-A,-2A} for nonidentity A remain outside the generic affine domain.

The default mixed mode emits the original operation stream byte for byte. The SHA-256 of its compressed ops.bin is 8d135db0aa0123fd143fc78ee1dbbe5fe6b851af9003352fde0f2fcc90458b34.

## Coherence argument and checks

For one table column c, the payload lookup performs |j>|z> -> |j>|z XOR c[j]>. Reapplying it after the consumer restores c[j] clears the payload without measuring it. In the K=2 case, the low-address controlled swaps are undone first.

Each temporary routing AND is measured in the X basis. Its measurement branch contributes (-1)^(mab)/sqrt(2), which is canceled by a classically conditioned CZ on the unchanged controls. Removing routing nodes in reverse order leaves a scalar independent of the address. The masked slope is computed and uncomputed with CCX gates and is preserved as the square's source.

This argument is conditional on the arithmetic backend implementing its intended coherent map. It does not make the backend's fixed iteration budget or truncated corrections exact on all inputs.

The branch self-check uses the actual unary, paired-column, and K=2 emitters:

- At w=0..5, all address components are evaluated under the same measurement record. Tests include all-zero, all-one, and two pseudorandom records and arbitrary runtime 256-bit payloads. Correct basis images with identical phase establish preservation of arbitrary complex coefficients on those tested components, conditional on the routing branch normalization identity.
- At w=16, injective 16-bit row labels exercise all 65,536 addresses for each routing variant and verify the address, output, phase, and cleared routing/payload state. The full payload width is separately exercised above.
- Complete point additions are tested in 256 four-call sequences at each w=0..5 on a development seed.

These are routing branch checks and finite complete-arithmetic tests. They are not a dense state-vector simulation of a 256-bit elliptic-curve computation.

## Reproduction

Build:

    cargo build --release --locked --offline --bin build_circuit --bin eval_circuit

Run routing checks:

    WINDOWED_QROM_BRANCH_SELFTEST=1 ./target/release/build_circuit

Run small-window development sequences:

    python3 experiments/windowed_pingpong/run_cases.py pilot --output /path/to/results

Run the same 100,000-input corpus used for Experiment 1:

    python3 experiments/windowed_pingpong/run_cases.py full --output /path/to/results --threads 8

Resume an interrupted full evaluation:

    python3 experiments/windowed_pingpong/run_cases.py full --output /path/to/results --threads 8 --resume

The runner enables WINDOWED_MODE=1, WINDOW_BITS=16, and SINGLE_CCX_FANOUT_DISABLE=1. It uses seed ecdsafail-windowed-independent-random-100k-v1 and durable per-input checkpoints. An allowed-error setting of 1 collects all failures and is not an acceptance threshold.

The source repository contains unrelated stale test modules that do not compile under cargo test. The self-check above avoids enabling those modules while exercising the new production routines.
