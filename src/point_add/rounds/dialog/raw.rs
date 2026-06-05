
#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;

pub(crate) fn dialog_gcd_raw_apply_direct_special_add_enabled() -> bool {
    std::env::var(DIALOG_GCD_RAW_APPLY_DIRECT_SPECIAL_ADD_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_raw_apply_materialized_special_add_enabled() -> bool {
    std::env::var(DIALOG_GCD_RAW_APPLY_MATERIALIZED_SPECIAL_ADD_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_raw_apply_reverse_fast_sub_enabled() -> bool {
    std::env::var(DIALOG_GCD_RAW_APPLY_REVERSE_FAST_SUB_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_raw_apply_reverse_materialized_special_sub_enabled() -> bool {
    std::env::var(DIALOG_GCD_RAW_APPLY_REVERSE_MATERIALIZED_SPECIAL_SUB_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_raw_tobitvector_materialized_sub_enabled() -> bool {
    std::env::var(DIALOG_GCD_RAW_TOBITVECTOR_MATERIALIZED_SUB_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_raw_tobitvector_variable_width_enabled() -> bool {
    std::env::var(DIALOG_GCD_RAW_TOBITVECTOR_VARIABLE_WIDTH_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_raw_tobitvector_borrow_future_log_carries_enabled() -> bool {
    std::env::var(DIALOG_GCD_RAW_TOBITVECTOR_BORROW_FUTURE_LOG_CARRIES_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_raw_ipmul_terminal_reuse_enabled() -> bool {
    std::env::var(DIALOG_GCD_RAW_IPMUL_TERMINAL_REUSE_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_raw_ipmul_clear_p_residual_enabled() -> bool {
    std::env::var(DIALOG_GCD_RAW_IPMUL_CLEAR_P_RESIDUAL_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_raw_quotient_terminal_reuse_enabled() -> bool {
    if let Ok(value) = std::env::var(DIALOG_GCD_RAW_QUOTIENT_TERMINAL_REUSE_ENV) {
        return value == "1";
    }
    dialog_gcd_raw_ipmul_terminal_reuse_enabled()
}

pub(crate) fn dialog_gcd_raw_quotient_keep_terminal_u_enabled() -> bool {
    std::env::var(DIALOG_GCD_RAW_QUOTIENT_KEEP_TERMINAL_U_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_raw_apply_truncated_clean_enabled() -> bool {
    std::env::var(DIALOG_GCD_RAW_APPLY_TRUNCATED_CLEAN_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_raw_pa_stop_after_quotient_enabled() -> bool {
    std::env::var(DIALOG_GCD_RAW_PA_STOP_AFTER_QUOTIENT_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_raw_pa_stop_after_xtail_enabled() -> bool {
    std::env::var(DIALOG_GCD_RAW_PA_STOP_AFTER_XTAIL_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_raw_pa_stop_after_c_enabled() -> bool {
    std::env::var(DIALOG_GCD_RAW_PA_STOP_AFTER_C_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_raw_pa_stop_after_pair2_enabled() -> bool {
    std::env::var(DIALOG_GCD_RAW_PA_STOP_AFTER_PAIR2_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn emit_dialog_gcd_raw_tobitvector_steps(
    b: &mut B,
    u: &[QubitId],
    v: &[QubitId],
    dialog_log: &[QubitId],
) {
    assert_eq!(u.len(), N);
    assert_eq!(v.len(), N);
    assert!(dialog_log.len() >= 2 * dialog_gcd_active_iterations());

    for step in 0..dialog_gcd_active_iterations() {
        let b0 = dialog_log[2 * step];
        let b0_and_b1 = dialog_log[2 * step + 1];
        let cmp = b.alloc_qubit();
        let active_width = dialog_gcd_tobitvector_active_width(step);
        let u_active = &u[..active_width];
        let v_active = &v[..active_width];
        let compare_bits = dialog_gcd_compare_bits_for_step(step, active_width);

        b.set_phase("dialog_gcd_raw_tobitvector_branch_bits");
        b.cx(v[0], b0);
        if dialog_gcd_fused_branch_bits_enabled() {
            dialog_gcd_ccx_cmp_gt_truncated_into_width(
                b,
                u_active,
                v_active,
                b0,
                b0_and_b1,
                compare_bits,
            );
        } else {
            dialog_gcd_cmp_gt_truncated_into_width(b, u_active, v_active, cmp, compare_bits);
            b.ccx(b0, cmp, b0_and_b1);
            dialog_gcd_cmp_gt_truncated_into_width(b, u_active, v_active, cmp, compare_bits);
        }
        b.free(cmp);

        b.set_phase("dialog_gcd_raw_tobitvector_cswap");
        for (i, (&ui, &vi)) in u_active.iter().zip(v_active.iter()).enumerate() {
            if i == 0 && dialog_gcd_odd_u_lowbit_fastpath_enabled() {
                continue;
            }
            cswap(b, b0_and_b1, ui, vi);
        }

        b.set_phase("dialog_gcd_raw_tobitvector_subtract");
        let borrowed_carries = dialog_gcd_future_log_carry_slice(dialog_log, step, active_width);
        dialog_gcd_controlled_sub_selected(b, u_active, v_active, b0, borrowed_carries);

        b.set_phase("dialog_gcd_raw_tobitvector_shift");
        dialog_gcd_shift_right_assuming_even(b, v_active);
    }
}

pub(crate) fn emit_dialog_gcd_raw_tobitvector_steps_reverse(
    b: &mut B,
    u: &[QubitId],
    v: &[QubitId],
    dialog_log: &[QubitId],
) {
    assert_eq!(u.len(), N);
    assert_eq!(v.len(), N);
    assert!(dialog_log.len() >= 2 * dialog_gcd_active_iterations());

    for step in (0..dialog_gcd_active_iterations()).rev() {
        let b0 = dialog_log[2 * step];
        let b0_and_b1 = dialog_log[2 * step + 1];
        let cmp = b.alloc_qubit();
        let active_width = dialog_gcd_tobitvector_active_width(step);
        let u_active = &u[..active_width];
        let v_active = &v[..active_width];
        let compare_bits = dialog_gcd_compare_bits_for_step(step, active_width);

        b.set_phase("dialog_gcd_raw_tobitvector_reverse_unshift");
        dialog_gcd_unshift_right_assuming_even(b, v_active);

        b.set_phase("dialog_gcd_raw_tobitvector_reverse_add");
        let borrowed_carries = dialog_gcd_future_log_carry_slice(dialog_log, step, active_width);
        dialog_gcd_controlled_add_selected(b, u_active, v_active, b0, borrowed_carries);

        b.set_phase("dialog_gcd_raw_tobitvector_reverse_cswap");
        for (i, (&ui, &vi)) in u_active.iter().zip(v_active.iter()).enumerate() {
            if i == 0 && dialog_gcd_odd_u_lowbit_fastpath_enabled() {
                continue;
            }
            cswap(b, b0_and_b1, ui, vi);
        }

        b.set_phase("dialog_gcd_raw_tobitvector_reverse_branch_bits");
        if dialog_gcd_fused_branch_bits_enabled() {
            dialog_gcd_ccx_cmp_gt_truncated_into_width(
                b,
                u_active,
                v_active,
                b0,
                b0_and_b1,
                compare_bits,
            );
        } else {
            dialog_gcd_cmp_gt_truncated_into_width(b, u_active, v_active, cmp, compare_bits);
            b.ccx(b0, cmp, b0_and_b1);
            dialog_gcd_cmp_gt_truncated_into_width(b, u_active, v_active, cmp, compare_bits);
        }
        b.free(cmp);
        b.cx(v[0], b0);
    }
}

pub(crate) fn emit_dialog_gcd_raw_apply_bitvector(
    b: &mut B,
    dialog_log: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
) {
    assert!(dialog_log.len() >= 2 * dialog_gcd_active_iterations());
    assert_eq!(x.len(), N);
    assert_eq!(y.len(), N);

    for step in (0..dialog_gcd_active_iterations()).rev() {
        let b0 = dialog_log[2 * step];
        let b0_and_b1 = dialog_log[2 * step + 1];

        b.set_phase("dialog_gcd_raw_apply_double_y");
        mod_double_inplace_fast(b, y, p);

        b.set_phase("dialog_gcd_raw_apply_cadd");
        if dialog_gcd_raw_apply_materialized_special_add_enabled() {
            dialog_gcd_cmod_add_materialized_pseudomersenne(b, y, x, b0, p);
        } else if dialog_gcd_raw_apply_direct_special_add_enabled() {
            dialog_gcd_cmod_add_pseudomersenne_lowq(b, y, x, b0, p);
        } else {
            cmod_add_qq_lowq(b, y, x, b0, p);
        }

        b.set_phase("dialog_gcd_raw_apply_cswap");
        for (&xi, &yi) in x.iter().zip(y.iter()) {
            cswap(b, b0_and_b1, xi, yi);
        }
    }
}

pub(crate) fn emit_dialog_gcd_raw_apply_bitvector_reverse_exact(
    b: &mut B,
    dialog_log: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
) {
    assert!(dialog_log.len() >= 2 * dialog_gcd_active_iterations());
    assert_eq!(x.len(), N);
    assert_eq!(y.len(), N);

    for step in 0..dialog_gcd_active_iterations() {
        let b0 = dialog_log[2 * step];
        let b0_and_b1 = dialog_log[2 * step + 1];

        b.set_phase("dialog_gcd_raw_apply_reverse_cswap");
        for (&xi, &yi) in x.iter().zip(y.iter()) {
            cswap(b, b0_and_b1, xi, yi);
        }

        b.set_phase("dialog_gcd_raw_apply_reverse_csub");
        if dialog_gcd_raw_apply_reverse_materialized_special_sub_enabled() {
            dialog_gcd_cmod_sub_materialized_pseudomersenne(b, y, x, b0, p);
        } else if dialog_gcd_raw_apply_reverse_fast_sub_enabled() {
            cmod_sub_qq(b, y, x, b0, p);
        } else {
            cmod_sub_qq_lowq(b, y, x, b0, p);
        }

        b.set_phase("dialog_gcd_raw_apply_reverse_halve_y");
        mod_halve_inplace_fast(b, y, p);
    }
}

pub(crate) fn emit_dialog_gcd_raw_apply_bitvector_reverse_borrowed_subtrahend(
    b: &mut B,
    dialog_log: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
    f: &[QubitId],
) {
    assert!(dialog_log.len() >= 2 * dialog_gcd_active_iterations());
    assert_eq!(x.len(), N);
    assert_eq!(y.len(), N);
    assert_eq!(f.len(), N);

    for step in 0..dialog_gcd_active_iterations() {
        let b0 = dialog_log[2 * step];
        let b0_and_b1 = dialog_log[2 * step + 1];

        b.set_phase("dialog_gcd_raw_apply_reverse_borrowed_cswap");
        for (&xi, &yi) in x.iter().zip(y.iter()) {
            cswap(b, b0_and_b1, xi, yi);
        }

        b.set_phase("dialog_gcd_raw_apply_reverse_borrowed_csub");
        if dialog_gcd_raw_apply_reverse_materialized_special_sub_enabled() {
            dialog_gcd_cmod_sub_materialized_pseudomersenne_borrowed_subtrahend(b, y, x, b0, p, f);
        } else {
            cmod_sub_qq_lowq_borrowed_subtrahend(b, y, x, b0, p, f);
        }

        b.set_phase("dialog_gcd_raw_apply_reverse_borrowed_halve_y");
        mod_halve_inplace_fast(b, y, p);
    }
}

pub(crate) fn emit_dialog_gcd_raw_ipmul(b: &mut B, factor: &[QubitId], target: &[QubitId], p: U256) {
    assert_eq!(factor.len(), N);
    assert_eq!(target.len(), N);

    if dialog_gcd_compressed_sidecar_log_enabled() {
        emit_dialog_gcd_compressed_sidecar_ipmul(b, factor, target, p);
        return;
    }

    let dialog_log = b.alloc_qubits(DIALOG_GCD_RAW_LOG_BITS);
    let u = b.alloc_qubits(N);
    b.set_phase("dialog_gcd_raw_ipmul_load_p");
    for i in 0..N {
        if bit(p, i) {
            b.x(u[i]);
        }
    }

    b.set_phase("dialog_gcd_raw_ipmul_tobitvector");
    emit_dialog_gcd_raw_tobitvector_steps(b, &u, factor, &dialog_log);

    if dialog_gcd_raw_ipmul_terminal_reuse_enabled() {
        b.set_phase("dialog_gcd_raw_ipmul_release_terminal_u");
        b.x(u[0]);
        b.free_vec(&u);

        b.set_phase("dialog_gcd_raw_ipmul_apply_bitvector_reuse_factor_zero");
        emit_dialog_gcd_raw_apply_bitvector(b, &dialog_log, target, factor, p);

        if dialog_gcd_raw_ipmul_clear_p_residual_enabled() {
            b.set_phase("dialog_gcd_raw_ipmul_clear_p_residual_source_lane");
            for i in 0..N {
                if bit(p, i) {
                    b.x(target[i]);
                }
            }
        }

        b.set_phase("dialog_gcd_raw_ipmul_swap_product_into_target");
        for i in 0..N {
            b.swap(target[i], factor[i]);
        }

        b.set_phase("dialog_gcd_raw_ipmul_reacquire_terminal_u");
        b.reacquire_vec(&u);
        b.set_phase("dialog_gcd_raw_ipmul_seed_terminal_u");
        b.x(u[0]);

        b.set_phase("dialog_gcd_raw_ipmul_uncompute_tobitvector");
        emit_dialog_gcd_raw_tobitvector_steps_reverse(b, &u, factor, &dialog_log);

        b.set_phase("dialog_gcd_raw_ipmul_unload_p");
        for i in 0..N {
            if bit(p, i) {
                b.x(u[i]);
            }
        }
        b.free_vec(&u);
        b.free_vec(&dialog_log);
        return;
    }

    let tmp = b.alloc_qubits(N);
    b.set_phase("dialog_gcd_raw_ipmul_apply_bitvector");
    emit_dialog_gcd_raw_apply_bitvector(b, &dialog_log, target, &tmp, p);

    b.set_phase("dialog_gcd_raw_ipmul_swap_product_into_target");
    for i in 0..N {
        b.swap(target[i], tmp[i]);
    }

    b.set_phase("dialog_gcd_raw_ipmul_free_zero_tmp");
    b.free_vec(&tmp);

    b.set_phase("dialog_gcd_raw_ipmul_uncompute_tobitvector");
    emit_dialog_gcd_raw_tobitvector_steps_reverse(b, &u, factor, &dialog_log);

    b.set_phase("dialog_gcd_raw_ipmul_unload_p");
    for i in 0..N {
        if bit(p, i) {
            b.x(u[i]);
        }
    }
    b.free_vec(&u);
    b.free_vec(&dialog_log);
}

pub(crate) fn dialog_gcd_clear_raw_block_copy(b: &mut B, compressed_block: &[QubitId], raw_block: &[QubitId]) {
    assert_eq!(
        compressed_block.len(),
        DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS
    );
    assert_eq!(raw_block.len(), 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE);
    emit_dialog_gcd_round763_compressor(b, raw_block);
    for i in 0..DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS {
        if dialog_gcd_apply_replay_swap_host_enabled() {
            b.swap(compressed_block[i], raw_block[i]);
        } else {
            b.cx(compressed_block[i], raw_block[i]);
        }
    }
}
pub(crate) fn emit_dialog_gcd_raw_quotient(b: &mut B, factor: &[QubitId], target: &[QubitId], p: U256) {
    assert_eq!(factor.len(), N);
    assert_eq!(target.len(), N);

    if dialog_gcd_compressed_sidecar_log_enabled() {
        emit_dialog_gcd_compressed_sidecar_quotient(b, factor, target, p);
        return;
    }

    let dialog_log = b.alloc_qubits(DIALOG_GCD_RAW_LOG_BITS);
    let u = b.alloc_qubits(N);
    b.set_phase("dialog_gcd_raw_quotient_load_p");
    for i in 0..N {
        if bit(p, i) {
            b.x(u[i]);
        }
    }

    b.set_phase("dialog_gcd_raw_quotient_tobitvector");
    emit_dialog_gcd_raw_tobitvector_steps(b, &u, factor, &dialog_log);

    if dialog_gcd_raw_quotient_keep_terminal_u_enabled() {
        b.set_phase("dialog_gcd_raw_quotient_zero_terminal_u_for_borrow");
        b.x(u[0]);

        b.set_phase("dialog_gcd_raw_quotient_apply_reverse_reuse_factor_zero_keep_u");
        emit_dialog_gcd_raw_apply_bitvector_reverse_borrowed_subtrahend(
            b,
            &dialog_log,
            factor,
            target,
            p,
            &u,
        );

        b.set_phase("dialog_gcd_raw_quotient_swap_quotient_into_target_keep_u");
        for i in 0..N {
            b.swap(target[i], factor[i]);
        }

        b.set_phase("dialog_gcd_raw_quotient_restore_terminal_u_after_borrow");
        b.x(u[0]);

        b.set_phase("dialog_gcd_raw_quotient_uncompute_tobitvector_keep_u");
        emit_dialog_gcd_raw_tobitvector_steps_reverse(b, &u, factor, &dialog_log);

        b.set_phase("dialog_gcd_raw_quotient_unload_p_keep_u");
        for i in 0..N {
            if bit(p, i) {
                b.x(u[i]);
            }
        }
        b.free_vec(&u);
        b.free_vec(&dialog_log);
        return;
    }

    if dialog_gcd_raw_quotient_terminal_reuse_enabled() {
        b.set_phase("dialog_gcd_raw_quotient_release_terminal_u");
        b.x(u[0]);
        b.free_vec(&u);

        b.set_phase("dialog_gcd_raw_quotient_apply_reverse_reuse_factor_zero");
        emit_dialog_gcd_raw_apply_bitvector_reverse_exact(b, &dialog_log, factor, target, p);

        b.set_phase("dialog_gcd_raw_quotient_swap_quotient_into_target");
        for i in 0..N {
            b.swap(target[i], factor[i]);
        }

        b.set_phase("dialog_gcd_raw_quotient_reacquire_terminal_u");
        b.reacquire_vec(&u);
        b.set_phase("dialog_gcd_raw_quotient_seed_terminal_u");
        b.x(u[0]);

        b.set_phase("dialog_gcd_raw_quotient_uncompute_tobitvector");
        emit_dialog_gcd_raw_tobitvector_steps_reverse(b, &u, factor, &dialog_log);

        b.set_phase("dialog_gcd_raw_quotient_unload_p");
        for i in 0..N {
            if bit(p, i) {
                b.x(u[i]);
            }
        }
        b.free_vec(&u);
        b.free_vec(&dialog_log);
        return;
    }

    let tmp = b.alloc_qubits(N);
    b.set_phase("dialog_gcd_raw_quotient_apply_reverse");
    emit_dialog_gcd_raw_apply_bitvector_reverse_exact(b, &dialog_log, &tmp, target, p);

    b.set_phase("dialog_gcd_raw_quotient_swap_quotient_into_target");
    for i in 0..N {
        b.swap(target[i], tmp[i]);
    }

    b.set_phase("dialog_gcd_raw_quotient_free_zero_tmp");
    b.free_vec(&tmp);

    b.set_phase("dialog_gcd_raw_quotient_uncompute_tobitvector");
    emit_dialog_gcd_raw_tobitvector_steps_reverse(b, &u, factor, &dialog_log);

    b.set_phase("dialog_gcd_raw_quotient_unload_p");
    for i in 0..N {
        if bit(p, i) {
            b.x(u[i]);
        }
    }
    b.free_vec(&u);
    b.free_vec(&dialog_log);
}

pub(crate) fn emit_dialog_gcd_raw_pa(
    b: &mut B,
    tx: &[QubitId],
    ty: &[QubitId],
    ox: &[BitId],
    oy: &[BitId],
    p: U256,
) {
    assert_eq!(tx.len(), N);
    assert_eq!(ty.len(), N);
    assert_eq!(ox.len(), N);
    assert_eq!(oy.len(), N);

    b.set_phase("dialog_gcd_raw_pa_pair1_quotient");
    emit_dialog_gcd_raw_quotient(b, tx, ty, p);
    if dialog_gcd_raw_pa_stop_after_quotient_enabled() {
        return;
    }

    round84_emit_fused_square_xtail(b, tx, ty, ox, p);
    if dialog_gcd_raw_pa_stop_after_xtail_enabled() {
        return;
    }

    b.set_phase("dialog_gcd_raw_pa_c_ox_minus_rx");
    mod_sub_qb(b, tx, ox, p);
    mod_neg_inplace_fast(b, tx, p);
    if dialog_gcd_raw_pa_stop_after_c_enabled() {
        return;
    }

    b.set_phase("dialog_gcd_raw_pa_pair2_product");
    emit_dialog_gcd_raw_ipmul(b, tx, ty, p);
    if dialog_gcd_raw_pa_stop_after_pair2_enabled() {
        return;
    }

    b.set_phase("dialog_gcd_raw_pa_y_output");
    mod_sub_qb(b, ty, oy, p);

    b.set_phase("dialog_gcd_raw_pa_x_restore");
    mod_neg_inplace_fast(b, tx, p);
    mod_add_qb(b, tx, ox, p);
}
