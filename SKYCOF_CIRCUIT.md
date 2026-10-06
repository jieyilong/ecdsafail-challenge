# SKY-COF 1,065-qubit clean circuit

This branch pins a clean affine secp256k1 point-addition circuit derived from
gnuchev's `57aa8ae` SKY-COF submission.

Configuration:

- 404-clock original SKY-COF walk and reverse
- parking odometer embedded in seven terminal cofactor owners
- field multiplication guard 25 and comparison window 40
- exact coordinate carry bits
- Fiat-Shamir identity-tail nonce 5

Trusted local evaluation on the pinned operation stream:

```text
tested shots            : 9024
classical mismatches    : 0
phase-garbage batches   : 0
ancilla-garbage batches : 0
avg executed Toffoli    : 2733904.170
qubits                  : 1065
score                   : 2911607760
```

The seven-qubit reduction is a storage adaptation: after park, positions
64..70 of the constant `R=p` cofactor bank hold the odometer. The circuit
restores those prime bits coherently before reversing the walk. The GCD
algorithm and overall point-addition architecture remain SKY-COF.
