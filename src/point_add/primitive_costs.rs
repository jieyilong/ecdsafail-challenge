//! Measure the exact Toffoli cost of each modular arithmetic primitive in
//! isolation. Test-only; emits numbers via eprintln for the planner.
//!
//! We don't need these for live correctness — just for honest cost accounting
//! so we can sanity-check SOTA reachability.

#![cfg(test)]

use super::{
    mod_add_qb, mod_add_qc, mod_add_qq, mod_double_inplace_fast, mod_halve_inplace_fast,
    mod_mul_add_into_acc_schoolbook,
    mod_mul_sub_qq, mod_mul_write_into_zero_acc_schoolbook, mod_neg_inplace_fast, mod_sub_qb, N,
    SECP256K1_P,
};
use super::{B, QubitId};
use crate::circuit::OperationType;

fn count_ccx(ops: &[crate::circuit::Op]) -> usize {
    ops.iter()
        .filter(|o| matches!(o.kind, OperationType::CCX | OperationType::CCZ))
        .count()
}

fn new_builder_with_reg(n: usize) -> (B, Vec<QubitId>) {
    let mut b = B::new();
    let r = b.alloc_qubits(n);
    (b, r)
}

#[test]
fn cost_mul_write_schoolbook_n256() {
    let mut b = B::new();
    let p = SECP256K1_P;
    let acc = b.alloc_qubits(N);
    let x = b.alloc_qubits(N);
    let y = b.alloc_qubits(N);
    let start = b.ops.len();
    mod_mul_write_into_zero_acc_schoolbook(&mut b, &acc, &x, &y, p);
    let end = b.ops.len();
    let ccx = count_ccx(&b.ops[start..end]);
    eprintln!("mod_mul_write_into_zero_acc_schoolbook(n=256): {} CCX", ccx);
}

#[test]
fn cost_mul_add_schoolbook_n256() {
    let mut b = B::new();
    let p = SECP256K1_P;
    let acc = b.alloc_qubits(N);
    let x = b.alloc_qubits(N);
    let y = b.alloc_qubits(N);
    let start = b.ops.len();
    mod_mul_add_into_acc_schoolbook(&mut b, &acc, &x, &y, p);
    let end = b.ops.len();
    let ccx = count_ccx(&b.ops[start..end]);
    eprintln!("mod_mul_add_into_acc_schoolbook(n=256): {} CCX", ccx);
}

#[test]
fn cost_mul_sub_qq_n256() {
    let mut b = B::new();
    let p = SECP256K1_P;
    let acc = b.alloc_qubits(N);
    let x = b.alloc_qubits(N);
    let y = b.alloc_qubits(N);
    let start = b.ops.len();
    mod_mul_sub_qq(&mut b, &acc, &x, &y, p);
    let end = b.ops.len();
    let ccx = count_ccx(&b.ops[start..end]);
    eprintln!("mod_mul_sub_qq(n=256): {} CCX", ccx);
}

#[test]
fn cost_sub_qb_n256() {
    let mut b = B::new();
    let p = SECP256K1_P;
    let acc = b.alloc_qubits(N);
    let bits = b.alloc_bits(N);
    let start = b.ops.len();
    mod_sub_qb(&mut b, &acc, &bits, p);
    let end = b.ops.len();
    let ccx = count_ccx(&b.ops[start..end]);
    eprintln!("mod_sub_qb(n=256): {} CCX", ccx);
}

#[test]
fn cost_neg_inplace_fast_n256() {
    let (mut b, r) = new_builder_with_reg(N);
    let p = SECP256K1_P;
    let start = b.ops.len();
    mod_neg_inplace_fast(&mut b, &r, p);
    let end = b.ops.len();
    let ccx = count_ccx(&b.ops[start..end]);
    eprintln!("mod_neg_inplace_fast(n=256): {} CCX", ccx);
}
#[test]
fn cost_squaring_sub_n256() {
    use super::*;
    use crate::circuit::OperationType;
    fn count_ccx(ops: &[crate::circuit::Op]) -> usize {
        ops.iter().filter(|o| matches!(o.kind, OperationType::CCX | OperationType::CCZ)).count()
    }
    let mut b = B::new();
    let p = SECP256K1_P;
    let acc = b.alloc_qubits(N);
    let x = b.alloc_qubits(N);
    let start = b.ops.len();
    // mod_mul_sub_qq with same register is a squaring
    mod_mul_sub_qq(&mut b, &acc, &x, &x, p);
    let end = b.ops.len();
    let ccx = count_ccx(&b.ops[start..end]);
    eprintln!("squaring via mod_mul_sub_qq: {} CCX", ccx);
}

#[test]
fn fermat_fixed_chain_inversion_floor_misses_sota_by_order() {
    // Branchless inversion by Fermat/exponentiation is the obvious way to avoid
    // Euclidean branch histories.  But even an unrealistically optimal addition
    // chain for an exponent near 2^256 needs at least 255 modular
    // square/multiply layers (each layer can at most double the exponent).  With
    // the measured current n=256 modular-square floor, this is already tens of
    // millions of CCX per inverse before any Bennett cleanup, scratch pressure,
    // or the second point-add denominator.  So fixed-sequence exponentiation is
    // not the missing SOTA-shaped DIV/IMUL primitive.
    let mut b = B::new();
    let p = SECP256K1_P;
    let acc = b.alloc_qubits(N);
    let x = b.alloc_qubits(N);
    let start = b.ops.len();
    mod_mul_sub_qq(&mut b, &acc, &x, &x, p);
    let square_ccx = count_ccx(&b.ops[start..]);
    let chain_layer_lower_bound = 255usize;
    let inv_floor = square_ccx * chain_layer_lower_bound;
    println!("METRIC fermat_inv_square_floor_ccx={square_ccx}");
    println!("METRIC fermat_inv_chain_floor_ccx={inv_floor}");
    eprintln!(
        "Fermat inversion floor: square_ccx={square_ccx}, layers>={chain_layer_lower_bound}, inv_floor={inv_floor}"
    );
    assert!(inv_floor > 30_000_000);
}

#[test]
fn cost_halve_double_n256() {
    let mut b = B::new();
    let p = SECP256K1_P;
    let v = b.alloc_qubits(N);
    let start = b.ops.len();
    mod_halve_inplace_fast(&mut b, &v, p);
    let mid = b.ops.len();
    mod_double_inplace_fast(&mut b, &v, p);
    let end = b.ops.len();
    let halve_ccx = count_ccx(&b.ops[start..mid]);
    let double_ccx = count_ccx(&b.ops[mid..end]);
    eprintln!("mod_halve_inplace_fast(n=256): {} CCX", halve_ccx);
    eprintln!("mod_double_inplace_fast(n=256): {} CCX", double_ccx);
}

#[test]
fn profile_point_add_by_phase() {
    use std::collections::HashMap;
    use crate::circuit::OperationType;
    let mut b = B::new();
    let p = SECP256K1_P;
    let n = 256;
    let tx = b.alloc_qubits(n);
    let ty = b.alloc_qubits(n);
    let ox = b.alloc_bits(n);
    let oy = b.alloc_bits(n);
    super::build_standard_point_add(&mut b, &tx, &ty, &ox, &oy, p);

    let mut phase_ccx: HashMap<&str, usize> = HashMap::new();
    let mut current_phase: &str = "(none)";
    let trans = &b.phase_transitions;
    let mut ti = 0;
    for (idx, op) in b.ops.iter().enumerate() {
        while ti < trans.len() && trans[ti].0 <= idx {
            current_phase = trans[ti].1;
            ti += 1;
        }
        if matches!(op.kind, OperationType::CCX | OperationType::CCZ) {
            *phase_ccx.entry(current_phase).or_insert(0) += 1;
        }
    }

    let mut entries: Vec<_> = phase_ccx.into_iter().collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1));
    let mut total = 0usize;
    eprintln!("\n=== Point Add Toffoli Profile by Phase ===");
    for (phase, ccx) in &entries {
        total += ccx;
        eprintln!("{:>10} {}", ccx, phase);
    }
    eprintln!("{:>10} TOTAL", total);
}
