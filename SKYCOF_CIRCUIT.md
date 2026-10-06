# SKY-COF 1,065-qubit targeted-window clean circuit

This branch pins a clean affine secp256k1 point-addition circuit derived from
gnuchev's `57aa8ae` SKY-COF submission.

Configuration:

- 404-clock original SKY-COF walk and reverse
- parking odometer embedded in seven terminal cofactor owners
- semantic field-product windows:
  - slope value: guard 21, comparison 24
  - numerator debt: guard 25, comparison 20
  - value product: guard 21, comparison 27
  - slope debt: guard 21, comparison 20
- exact coordinate carry bits
- Fiat-Shamir identity-tail nonce 1,001,055

Trusted local evaluation on the pinned operation stream:

```text
tested shots            : 9024
classical mismatches    : 0
phase-garbage batches   : 0
ancilla-garbage batches : 0
avg executed Toffoli    : 2709078.722
qubits                  : 1065
score                   : 2885169135
```

The seven-qubit reduction is a storage adaptation of SKY-COF. The field
configuration uses explicit semantic call roles so a value-producing product
and a later HMR-debt product can use different guard/comparison windows even
when their scale and sign are identical.
