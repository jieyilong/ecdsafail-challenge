
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;
use crate::point_add::venting;

pub(crate) fn mod_sub_qq(b: &mut B, acc: &[QubitId], a: &[QubitId], p: U256) {
    // mod_add_qq is a bijection on (acc, a): (acc, a) ↦ (acc + a mod p, a).
    // Its gate-level inverse therefore acts as (acc, a) ↦ (acc - a mod p, a),
    // which is exactly what we want. emit_inverse replays the forward's gates
    // reversed, skipping R markers — valid because mod_add_qq is clean
    // (every ancilla is driven to |0⟩ before its R).
    let a_copy: Vec<QubitId> = a.to_vec();
    emit_inverse(b, move |b| mod_add_qq(b, acc, &a_copy, p));
}

/// Fast `acc := (acc - a) mod p`. Direct sub + conditional add-p + flag
/// uncompute via neg+cmp_lt+neg. All ops use measurement-based Cuccaro.
pub(crate) fn mod_sub_qq_fast(b: &mut B, acc: &[QubitId], a: &[QubitId], p: U256) {
    let n = acc.len();
    assert_eq!(n, a.len());
    debug_assert_eq!(n, 256);

    let (acc_ext, acc_ovf) = ext_reg(b, acc);
    let (a_ext, a_ovf) = ext_reg(b, a);

    // Step 1: (n+1)-bit sub.
    sub_nbit_qq_fast(b, &a_ext, &acc_ext);

    // Step 2: flag = acc_ovf (=1 iff underflow, i.e. acc < a).
    let flag = b.alloc_qubit();
    b.cx(acc_ovf, flag);
    // We only need the borrow as a separate flag; the low register is
    // corrected modulo 2^n, so clear the extension bit immediately.
    b.cx(flag, acc_ovf);

    // Step 3: underflow correction. With p = 2^n - c, the wrapped 256-bit
    // subtraction needs only a conditional subtract of c on the low register.
    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1));
    if kal_vent_modadd_enabled() {
        // Use venting cisub with a_ext as dirty qubits.
        let c_low = c.as_limbs()[0];
        let q_clean2: [QubitId; 2] = [b.alloc_qubit(), b.alloc_qubit()];
        venting::cisub_dirty_2clean_classical(
            b,
            &acc_ext[..n],
            &a_ext[..n - 2],
            &q_clean2,
            c_low,
            flag,
        );
        b.free(q_clean2[0]);
        b.free(q_clean2[1]);
    } else if secp_direct_const_arith_enabled() {
        csub_nbit_const_direct_fast(b, &acc_ext[..n], c, flag);
    } else {
        csub_nbit_const_fast(b, &acc_ext[..n], c, flag);
    }

    // Step 4: uncompute flag. Identity: flag = NOT(acc_final < (p - a)).
    // Negate a in place, compare, un-negate.
    b.x(flag);
    mod_neg_inplace_fast(b, &a_ext[..n], p);
    cmp_lt_into_fast(b, &acc_ext[..n], &a_ext[..n], flag);
    mod_neg_inplace_fast(b, &a_ext[..n], p);
    b.free(flag);

    unext_reg(b, a_ovf);
    unext_reg(b, acc_ovf);
    let _ = (acc_ext, a_ext);
}

/// `acc := (acc - a) mod p`, low-peak. Explicit gate-reverse of
/// `mod_add_qq_vent` (the venting protocols use measurement, so `emit_inverse`
/// cannot reverse them; each venting step is undone by its matched dual:
/// iadd↔isub, cisub↔ciadd). The flag-uncompute is `cmp_lt_into` (self-inverse,
/// no materialized neg), so no n-wide const register is ever live.
pub(crate) fn mod_sub_qq_vent(b: &mut B, acc: &[QubitId], a: &[QubitId], p: U256) {
    let n = acc.len();
    assert_eq!(n, a.len());
    debug_assert_eq!(n, 256);

    let (acc_ext, acc_ovf) = ext_reg(b, acc);
    let (a_ext, a_ovf) = ext_reg(b, a);

    let c = U256::MAX.wrapping_sub(p).wrapping_add(U256::from(1));
    let c_low = c.as_limbs()[0];
    let n1 = acc_ext.len();

    // Reverse of forward step 6: cmp_lt_into is its own inverse (XOR into flag).
    let flag = b.alloc_qubit();
    cmp_lt_into(b, &acc_ext[..n], &a_ext[..n], flag);

    // Reverse of step 5.
    b.cx(flag, acc_ovf);

    // Reverse of step 4: forward applied (cisub c) under !flag; undo with ciadd.
    b.x(flag);
    {
        let q_clean2: [QubitId; 2] = [b.alloc_qubit(), b.alloc_qubit()];
        venting::ciadd_dirty_2clean_classical(
            b,
            &acc_ext,
            &a_ext[..n1 - 2],
            &q_clean2,
            c_low,
            flag,
            false,
        );
        b.free(q_clean2[0]);
        b.free(q_clean2[1]);
    }
    b.x(flag);

    // Reverse of step 3.
    b.cx(acc_ovf, flag);
    b.free(flag);

    // Reverse of step 2: undo the unconditional (iadd c) with a cisub under an
    // always-on control.
    {
        let one = b.alloc_qubit();
        b.x(one);
        let q_clean2: [QubitId; 2] = [b.alloc_qubit(), b.alloc_qubit()];
        venting::cisub_dirty_2clean_classical(
            b,
            &acc_ext,
            &a_ext[..n1 - 2],
            &q_clean2,
            c_low,
            one,
        );
        b.free(q_clean2[0]);
        b.free(q_clean2[1]);
        b.x(one);
        b.free(one);
    }

    // Reverse of step 1.
    sub_nbit_qq(b, &a_ext, &acc_ext);

    unext_reg(b, a_ovf);
    unext_reg(b, acc_ovf);
    let _ = (acc_ext, a_ext);
}


pub(crate) fn mod_sub_qb(b: &mut B, acc: &[QubitId], bits: &[BitId], p: U256) {
    // acc -= bits mod p. Uses fast mod_sub_qq via neg+add+neg.
    let a = load_bits(b, bits);
    mod_sub_qq_fast(b, acc, &a, p);
    unload_bits(b, &a, bits);
}
