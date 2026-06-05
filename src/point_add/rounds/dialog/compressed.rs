#![allow(unused_imports, dead_code, clippy::all)]
#[allow(unused_imports)]
use super::*;


// ─── merged from compressed1.rs ───

pub(crate) fn dialog_gcd_compressed_sidecar_log_enabled() -> bool {
    std::env::var(DIALOG_GCD_COMPRESSED_SIDECAR_LOG_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_compressed_block_lifecycle_enabled() -> bool {
    std::env::var(DIALOG_GCD_COMPRESSED_BLOCK_LIFECYCLE_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_compressed_sidecar_blocks() -> usize {
    (dialog_gcd_active_iterations() + DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE - 1)
        / DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE
}

pub(crate) fn dialog_gcd_compressed_sidecar_bits() -> usize {
    dialog_gcd_compressed_sidecar_blocks() * DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS
}

pub(crate) fn dialog_gcd_compressed_sidecar_block(compressed_log: &[QubitId], step: usize) -> &[QubitId] {
    let block = step / DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE;
    let start = block * DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS;
    &compressed_log[start..start + DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS]
}

pub(crate) fn dialog_gcd_host_reverse_raw_block_enabled() -> bool {
    std::env::var("DIALOG_GCD_HOST_REVERSE_RAW_BLOCK")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_reverse_raw_block_host<'a>(
    u: &'a [QubitId],
    compressed_log: &'a [QubitId],
    block: usize,
) -> Option<&'a [QubitId]> {
    if !dialog_gcd_host_reverse_raw_block_enabled() {
        return None;
    }
    let (start, _) = dialog_gcd_compressed_sidecar_block_step_range(block);
    let active_width = dialog_gcd_tobitvector_active_width(start);
    let want = 2 * active_width - 1;
    if u.len().saturating_sub(active_width) >= want + 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE {
        let candidate = &u[u.len() - 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE..];
        if !dialog_gcd_compressed_log_u_high_runway_enabled()
            || !dialog_gcd_slice_intersects(candidate, compressed_log)
        {
            return Some(candidate);
        }
    }
    let future_start = (block + 1) * DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS;
    let future = compressed_log.get(future_start..)?;
    let raw_bits = 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE;
    if future.len() < want + raw_bits {
        return None;
    }
    if !dialog_gcd_compressed_log_u_high_runway_enabled() {
        return Some(&future[future.len() - raw_bits..]);
    }
    // Keep the raw host after the largest possible carry+gated prefix and away
    // from active u.  With remapped runway cells the old final-six shortcut can
    // alias the growing reverse u prefix.
    future[want..]
        .windows(raw_bits)
        .rev()
        .find(|candidate| !dialog_gcd_slice_intersects(candidate, &u[..active_width]))
}
pub(crate) fn dialog_gcd_forward_raw_block_host<'a>(
    u: &'a [QubitId],
    compressed_log: &'a [QubitId],
    block: usize,
) -> Option<&'a [QubitId]> {
    if !dialog_gcd_host_reverse_raw_block_enabled() {
        return None;
    }
    let (start, _) = dialog_gcd_compressed_sidecar_block_step_range(block);
    let active_width = dialog_gcd_tobitvector_active_width(start);
    let want = 2 * active_width - 1;
    let future_start = (block + 1) * DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS;
    if let Some(future) = compressed_log.get(future_start..) {
        if future.len() >= want + 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE {
            let raw_bits = 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE;
            if !dialog_gcd_compressed_log_u_high_runway_enabled() {
                return Some(&future[future.len() - raw_bits..]);
            }
            if let Some(candidate) = future[want..]
                .windows(raw_bits)
                .rev()
                .find(|candidate| !dialog_gcd_slice_intersects(candidate, &u[..active_width]))
            {
                return Some(candidate);
            }
        }
    }
    if u.len().saturating_sub(active_width) >= want + 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE {
        let candidate = &u[u.len() - 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE..];
        if !dialog_gcd_compressed_log_u_high_runway_enabled()
            || !dialog_gcd_slice_intersects(candidate, compressed_log)
        {
            Some(candidate)
        } else {
            None
        }
    } else {
        None
    }
}
pub(crate) fn dialog_gcd_compressed_sidecar_future_carry_slice(
    compressed_log: &[QubitId],
    step: usize,
    active_width: usize,
) -> Option<&[QubitId]> {
    if !dialog_gcd_raw_tobitvector_borrow_future_log_carries_enabled() {
        return None;
    }
    let carry_need = active_width.saturating_sub(1);
    // When hosting the gated register too, request up to carry(n-1)+gated(n)=2n-1
    // clean slots; the consumer splits the returned slice. Graceful: never return
    // fewer than carry_need (so carry borrowing is preserved), never more than
    // what the future region holds.
    let want = if dialog_gcd_host_gated_enabled() {
        2 * active_width - 1
    } else {
        carry_need
    };
    let next_block = step / DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE + 1;
    let start = next_block * DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS;
    compressed_log
        .get(start..)
        .filter(|future| future.len() >= carry_need)
        .map(|future| &future[..future.len().min(want)])
}

pub(crate) fn dialog_gcd_compressed_sidecar_block_step_range(block: usize) -> (usize, usize) {
    let start = block * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE;
    let end = (start + DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE).min(dialog_gcd_active_iterations());
    (start, end)
}

pub(crate) fn dialog_gcd_copy_compressed_block_to_raw(
    b: &mut B,
    compressed_block: &[QubitId],
    raw_block: &[QubitId],
) {
    assert_eq!(
        compressed_block.len(),
        DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS
    );
    assert_eq!(raw_block.len(), 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE);
    for i in 0..DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS {
        if dialog_gcd_apply_replay_swap_host_enabled() {
            b.swap(compressed_block[i], raw_block[i]);
        } else {
            b.cx(compressed_block[i], raw_block[i]);
        }
    }
    emit_dialog_gcd_round763_compressor_inverse(b, raw_block);
}
pub(crate) fn emit_dialog_gcd_compressed_sidecar_tobitvector_steps_block_lifecycle(
    b: &mut B,
    u: &[QubitId],
    v: &[QubitId],
    compressed_log: &[QubitId],
    raw_block: &[QubitId],
) {
    assert_eq!(u.len(), N);
    assert_eq!(v.len(), N);
    assert!(
        raw_block.is_empty() || raw_block.len() == 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE
    );
    assert!(compressed_log.len() >= dialog_gcd_compressed_sidecar_bits());

    for block in 0..dialog_gcd_compressed_sidecar_blocks() {
        let (start, end) = dialog_gcd_compressed_sidecar_block_step_range(block);
        let hosted_raw_block = dialog_gcd_forward_raw_block_host(u, compressed_log, block);
        let owned_raw_block = if dialog_gcd_host_reverse_raw_block_enabled() && hosted_raw_block.is_none() {
            b.alloc_qubits(2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE)
        } else {
            Vec::new()
        };
        let raw_block = hosted_raw_block.unwrap_or_else(|| {
            if owned_raw_block.is_empty() {
                raw_block
            } else {
                &owned_raw_block
            }
        });
        for step in start..end {
            let slot = step - start;
            let b0 = raw_block[2 * slot];
            let b0_and_b1 = raw_block[2 * slot + 1];
            let active_width = dialog_gcd_tobitvector_active_width(step);
            let u_active = &u[..active_width];
            let v_active = &v[..active_width];
            let compare_bits = dialog_gcd_compare_bits_for_step(step, active_width);

            let future = dialog_gcd_compressed_sidecar_future_carry_slice(
                compressed_log,
                step,
                active_width,
            );
            let composite_scratch = dialog_gcd_composite_scratch_enabled().then(|| {
                dialog_gcd_build_composite_scratch(
                    b,
                    future,
                    u,
                    v,
                    compressed_log,
                    raw_block,
                    active_width,
                )
            });
            let borrowed_carries = composite_scratch.as_ref().map_or_else(
                || dialog_gcd_pick_runway_safe_borrow_slice(future, u, compressed_log, active_width),
                |scratch| Some(scratch.lanes.as_slice()),
            );

            b.set_phase("dialog_gcd_compressed_block_tobitvector_branch_bits");
            b.cx(v[0], b0);
            if dialog_gcd_fused_branch_bits_enabled() {
                // Fused path derives b0_and_b1 from the in-flight comparator carry
                // and never materializes a separate `cmp` ancilla. Allocating it
                // here would add a dead live-qubit at the branch_bits peak instant
                // (peak is measured by simultaneously-live count, not qubit-id reuse),
                // so it is allocated only on the non-fused branch below.
                if dialog_gcd_branch_bits_host_comparator_enabled() {
                    // Host the comparator's c_in+carries transient on the idle
                    // future-log slice (the same slice the subtract borrows below;
                    // it is unwritten at the comparator instant) so branch_bits no
                    // longer allocates its own peak qubit. Value-exact; the slice is
                    // returned clean by the measured uncompute sweep.
                    dialog_gcd_ccx_cmp_gt_truncated_into_width_hosted(
                        b,
                        u_active,
                        v_active,
                        b0,
                        b0_and_b1,
                        compare_bits,
                        borrowed_carries,
                    );
                } else {
                    dialog_gcd_ccx_cmp_gt_truncated_into_width(
                        b,
                        u_active,
                        v_active,
                        b0,
                        b0_and_b1,
                        compare_bits,
                    );
                }
            } else {
                let cmp = b.alloc_qubit();
                dialog_gcd_cmp_gt_truncated_into_width(b, u_active, v_active, cmp, compare_bits);
                b.ccx(b0, cmp, b0_and_b1);
                dialog_gcd_cmp_gt_truncated_into_width(b, u_active, v_active, cmp, compare_bits);
                b.free(cmp);
            }

            b.set_phase("dialog_gcd_compressed_block_tobitvector_cswap");
            for (i, (&ui, &vi)) in u_active.iter().zip(v_active.iter()).enumerate() {
                if i == 0 && dialog_gcd_odd_u_lowbit_fastpath_enabled() {
                    continue;
                }
                cswap(b, b0_and_b1, ui, vi);
            }

            b.set_phase("dialog_gcd_compressed_block_tobitvector_subtract");
            dialog_gcd_controlled_sub_selected(b, u_active, v_active, b0, borrowed_carries);

            b.set_phase("dialog_gcd_compressed_block_tobitvector_shift");
            dialog_gcd_shift_right_assuming_even(b, v_active);
            if let Some(scratch) = composite_scratch {
                b.free_vec(&scratch.owned);
            }
        }

        b.set_phase("dialog_gcd_compressed_block_tobitvector_compress_block");
        emit_dialog_gcd_round763_compressor(b, raw_block);
        let compressed_block = dialog_gcd_compressed_sidecar_block(compressed_log, start);
        if dialog_gcd_compressed_log_u_high_runway_enabled() {
            // A parked forward block is first written only after its high-u
            // hosts have left the active prefix.
            assert!(
                !dialog_gcd_slice_intersects(
                    compressed_block,
                    &u[..dialog_gcd_tobitvector_active_width(start)]
                ),
                "compressed-log runway overlaps active forward u prefix at block {block}"
            );
        }
        for i in 0..DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS {
            b.swap(raw_block[i], compressed_block[i]);
        }
        if !owned_raw_block.is_empty() {
            b.free_vec(&owned_raw_block);
        }
    }
}
pub(crate) fn emit_dialog_gcd_compressed_sidecar_tobitvector_steps_reverse_block_lifecycle(
    b: &mut B,
    u: &[QubitId],
    v: &[QubitId],
    compressed_log: &[QubitId],
    raw_block: &[QubitId],
) {
    assert_eq!(u.len(), N);
    assert_eq!(v.len(), N);
    assert!(
        raw_block.is_empty() || raw_block.len() == 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE
    );
    assert!(compressed_log.len() >= dialog_gcd_compressed_sidecar_bits());

    for block in (0..dialog_gcd_compressed_sidecar_blocks()).rev() {
        let (start, end) = dialog_gcd_compressed_sidecar_block_step_range(block);
        let compressed_block = dialog_gcd_compressed_sidecar_block(compressed_log, start);
        let hosted_raw_block = dialog_gcd_reverse_raw_block_host(u, compressed_log, block);
        let owned_raw_block = if dialog_gcd_host_reverse_raw_block_enabled() && hosted_raw_block.is_none() {
            b.alloc_qubits(2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE)
        } else {
            Vec::new()
        };
        let raw_block = hosted_raw_block.unwrap_or_else(|| {
            if owned_raw_block.is_empty() {
                raw_block
            } else {
                &owned_raw_block
            }
        });

        b.set_phase("dialog_gcd_compressed_block_tobitvector_reverse_decompress_block");
        if dialog_gcd_compressed_log_u_high_runway_enabled() {
            // A parked block must be consumed while all of its high-u hosts are
            // outside this block's active prefix.
            assert!(
                !dialog_gcd_slice_intersects(
                    compressed_block,
                    &u[..dialog_gcd_tobitvector_active_width(start)]
                ),
                "compressed-log runway overlaps active reverse u prefix at block {block}"
            );
        }
        for i in 0..DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS {
            b.swap(compressed_block[i], raw_block[i]);
        }
        emit_dialog_gcd_round763_compressor_inverse(b, raw_block);

        for step in (start..end).rev() {
            let slot = step - start;
            let b0 = raw_block[2 * slot];
            let b0_and_b1 = raw_block[2 * slot + 1];
            let active_width = dialog_gcd_tobitvector_active_width(step);
            let u_active = &u[..active_width];
            let v_active = &v[..active_width];
            let compare_bits = dialog_gcd_compare_bits_for_step(step, active_width);

            b.set_phase("dialog_gcd_compressed_block_tobitvector_reverse_unshift");
            dialog_gcd_unshift_right_assuming_even(b, v_active);

            b.set_phase("dialog_gcd_compressed_block_tobitvector_reverse_add");
            let future = dialog_gcd_compressed_sidecar_future_carry_slice(
                compressed_log,
                step,
                active_width,
            );
            let composite_scratch = dialog_gcd_composite_scratch_enabled().then(|| {
                dialog_gcd_build_composite_scratch(
                    b,
                    future,
                    u,
                    v,
                    compressed_log,
                    raw_block,
                    active_width,
                )
            });
            let borrowed_carries = composite_scratch.as_ref().map_or_else(
                || dialog_gcd_pick_runway_safe_borrow_slice(future, u, compressed_log, active_width),
                |scratch| Some(scratch.lanes.as_slice()),
            );
            dialog_gcd_controlled_add_selected(b, u_active, v_active, b0, borrowed_carries);

            b.set_phase("dialog_gcd_compressed_block_tobitvector_reverse_cswap");
            for (i, (&ui, &vi)) in u_active.iter().zip(v_active.iter()).enumerate() {
                if i == 0 && dialog_gcd_odd_u_lowbit_fastpath_enabled() {
                    continue;
                }
                cswap(b, b0_and_b1, ui, vi);
            }

            b.set_phase("dialog_gcd_compressed_block_tobitvector_reverse_branch_bits");
            if dialog_gcd_fused_branch_bits_enabled() {
                // Fused path: no separate `cmp` ancilla (derives b0_and_b1 from the
                // comparator carry). Allocating it would add a dead live-qubit at the
                // reverse_branch_bits peak instant, so allocate only on the non-fused
                // branch below. See forward lifecycle for the rationale.
                if dialog_gcd_branch_bits_host_comparator_enabled() {
                    // Mirror of the forward path: host the comparator transient on
                    // the idle future-log slice (same slice the add borrowed above).
                    dialog_gcd_ccx_cmp_gt_truncated_into_width_hosted(
                        b,
                        u_active,
                        v_active,
                        b0,
                        b0_and_b1,
                        compare_bits,
                        borrowed_carries,
                    );
                } else {
                    dialog_gcd_ccx_cmp_gt_truncated_into_width(
                        b,
                        u_active,
                        v_active,
                        b0,
                        b0_and_b1,
                        compare_bits,
                    );
                }
            } else {
                let cmp = b.alloc_qubit();
                dialog_gcd_cmp_gt_truncated_into_width(b, u_active, v_active, cmp, compare_bits);
                b.ccx(b0, cmp, b0_and_b1);
                dialog_gcd_cmp_gt_truncated_into_width(b, u_active, v_active, cmp, compare_bits);
                b.free(cmp);
            }
            b.cx(v[0], b0);
            if let Some(scratch) = composite_scratch {
                b.free_vec(&scratch.owned);
            }
        }
        if !owned_raw_block.is_empty() {
            b.free_vec(&owned_raw_block);
        }
    }
}
pub(crate) fn emit_dialog_gcd_compressed_sidecar_apply_bitvector_block_lifecycle(
    b: &mut B,
    compressed_log: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
    raw_block: &[QubitId],
) {
    assert_eq!(x.len(), N);
    assert_eq!(y.len(), N);
    assert_eq!(raw_block.len(), 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE);

    for block in (0..dialog_gcd_compressed_sidecar_blocks()).rev() {
        let (start, end) = dialog_gcd_compressed_sidecar_block_step_range(block);
        let compressed_block = dialog_gcd_compressed_sidecar_block(compressed_log, start);

        b.set_phase("dialog_gcd_compressed_block_apply_decompress_block");
        dialog_gcd_copy_compressed_block_to_raw(b, compressed_block, raw_block);
        let clean_scratch = if dialog_gcd_apply_replay_swap_host_enabled() {
            compressed_block
        } else {
            &[]
        };

        for step in (start..end).rev() {
            let slot = step - start;
            let b0 = raw_block[2 * slot];
            let b0_and_b1 = raw_block[2 * slot + 1];

            b.set_phase("dialog_gcd_compressed_block_apply_double_y");
            mod_double_inplace_fast(b, y, p);

            b.set_phase("dialog_gcd_compressed_block_apply_cadd");
            if dialog_gcd_raw_apply_materialized_special_add_enabled() {
                dialog_gcd_cmod_add_materialized_pseudomersenne_with_clean_scratch(
                    b,
                    y,
                    x,
                    b0,
                    p,
                    clean_scratch,
                );
            } else if dialog_gcd_raw_apply_direct_special_add_enabled() {
                dialog_gcd_cmod_add_pseudomersenne_lowq(b, y, x, b0, p);
            } else {
                cmod_add_qq_lowq(b, y, x, b0, p);
            }

            b.set_phase("dialog_gcd_compressed_block_apply_cswap");
            for (&xi, &yi) in x.iter().zip(y.iter()) {
                cswap(b, b0_and_b1, xi, yi);
            }
        }

        b.set_phase("dialog_gcd_compressed_block_apply_clear_block_copy");
        dialog_gcd_clear_raw_block_copy(b, compressed_block, raw_block);
    }
}
pub(crate) fn emit_dialog_gcd_compressed_sidecar_apply_bitvector_reverse_exact_block_lifecycle(
    b: &mut B,
    compressed_log: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
    raw_block: &[QubitId],
) {
    assert_eq!(x.len(), N);
    assert_eq!(y.len(), N);
    assert_eq!(raw_block.len(), 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE);

    for block in 0..dialog_gcd_compressed_sidecar_blocks() {
        let (start, end) = dialog_gcd_compressed_sidecar_block_step_range(block);
        let compressed_block = dialog_gcd_compressed_sidecar_block(compressed_log, start);

        b.set_phase("dialog_gcd_compressed_block_apply_reverse_decompress_block");
        dialog_gcd_copy_compressed_block_to_raw(b, compressed_block, raw_block);
        let clean_scratch = if dialog_gcd_apply_replay_swap_host_enabled() {
            compressed_block
        } else {
            &[]
        };

        for step in start..end {
            let slot = step - start;
            let b0 = raw_block[2 * slot];
            let b0_and_b1 = raw_block[2 * slot + 1];

            b.set_phase("dialog_gcd_compressed_block_apply_reverse_cswap");
            for (&xi, &yi) in x.iter().zip(y.iter()) {
                cswap(b, b0_and_b1, xi, yi);
            }

            b.set_phase("dialog_gcd_compressed_block_apply_reverse_csub");
            if dialog_gcd_raw_apply_reverse_materialized_special_sub_enabled() {
                dialog_gcd_cmod_sub_materialized_pseudomersenne_with_clean_scratch(
                    b,
                    y,
                    x,
                    b0,
                    p,
                    clean_scratch,
                );
            } else if dialog_gcd_raw_apply_reverse_fast_sub_enabled() {
                cmod_sub_qq(b, y, x, b0, p);
            } else {
                cmod_sub_qq_lowq(b, y, x, b0, p);
            }

            b.set_phase("dialog_gcd_compressed_block_apply_reverse_halve_y");
            mod_halve_inplace_fast(b, y, p);
        }

        b.set_phase("dialog_gcd_compressed_block_apply_reverse_clear_block_copy");
        dialog_gcd_clear_raw_block_copy(b, compressed_block, raw_block);
    }
}
pub(crate) fn emit_dialog_gcd_compressed_sidecar_tobitvector_steps(
    b: &mut B,
    u: &[QubitId],
    v: &[QubitId],
    compressed_log: &[QubitId],
    pair: &[QubitId],
    scratch: QubitId,
) {
    assert_eq!(u.len(), N);
    assert_eq!(v.len(), N);
    assert_eq!(pair.len(), 2);
    assert!(compressed_log.len() >= dialog_gcd_compressed_sidecar_bits());

    for step in 0..dialog_gcd_active_iterations() {
        let b0 = pair[0];
        let b0_and_b1 = pair[1];
        let cmp = b.alloc_qubit();
        let active_width = dialog_gcd_tobitvector_active_width(step);
        let u_active = &u[..active_width];
        let v_active = &v[..active_width];
        let compare_bits = dialog_gcd_compare_bits_for_step(step, active_width);

        b.set_phase("dialog_gcd_compressed_sidecar_tobitvector_branch_bits");
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

        b.set_phase("dialog_gcd_compressed_sidecar_tobitvector_cswap");
        for (i, (&ui, &vi)) in u_active.iter().zip(v_active.iter()).enumerate() {
            if i == 0 && dialog_gcd_odd_u_lowbit_fastpath_enabled() {
                continue;
            }
            cswap(b, b0_and_b1, ui, vi);
        }

        b.set_phase("dialog_gcd_compressed_sidecar_tobitvector_subtract");
        let borrowed_carries =
            dialog_gcd_compressed_sidecar_future_carry_slice(compressed_log, step, active_width);
        dialog_gcd_controlled_sub_selected(b, u_active, v_active, b0, borrowed_carries);

        b.set_phase("dialog_gcd_compressed_sidecar_tobitvector_shift");
        dialog_gcd_shift_right_assuming_even(b, v_active);

        b.set_phase("dialog_gcd_compressed_sidecar_tobitvector_absorb_pair");
        let block = dialog_gcd_compressed_sidecar_block(compressed_log, step);
        emit_dialog_gcd_round763_compressed_block_swapper(
            b,
            pair,
            block,
            scratch,
            step % DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE,
        );
    }
}

pub(crate) fn emit_dialog_gcd_compressed_sidecar_tobitvector_steps_reverse(
    b: &mut B,
    u: &[QubitId],
    v: &[QubitId],
    compressed_log: &[QubitId],
    pair: &[QubitId],
    scratch: QubitId,
) {
    assert_eq!(u.len(), N);
    assert_eq!(v.len(), N);
    assert_eq!(pair.len(), 2);
    assert!(compressed_log.len() >= dialog_gcd_compressed_sidecar_bits());

    for step in (0..dialog_gcd_active_iterations()).rev() {
        b.set_phase("dialog_gcd_compressed_sidecar_tobitvector_reverse_load_pair");
        let block = dialog_gcd_compressed_sidecar_block(compressed_log, step);
        emit_dialog_gcd_round763_compressed_block_swapper(
            b,
            pair,
            block,
            scratch,
            step % DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE,
        );

        let b0 = pair[0];
        let b0_and_b1 = pair[1];
        let cmp = b.alloc_qubit();
        let active_width = dialog_gcd_tobitvector_active_width(step);
        let u_active = &u[..active_width];
        let v_active = &v[..active_width];
        let compare_bits = dialog_gcd_compare_bits_for_step(step, active_width);

        b.set_phase("dialog_gcd_compressed_sidecar_tobitvector_reverse_unshift");
        dialog_gcd_unshift_right_assuming_even(b, v_active);

        b.set_phase("dialog_gcd_compressed_sidecar_tobitvector_reverse_add");
        let borrowed_carries =
            dialog_gcd_compressed_sidecar_future_carry_slice(compressed_log, step, active_width);
        dialog_gcd_controlled_add_selected(b, u_active, v_active, b0, borrowed_carries);

        b.set_phase("dialog_gcd_compressed_sidecar_tobitvector_reverse_cswap");
        for (i, (&ui, &vi)) in u_active.iter().zip(v_active.iter()).enumerate() {
            if i == 0 && dialog_gcd_odd_u_lowbit_fastpath_enabled() {
                continue;
            }
            cswap(b, b0_and_b1, ui, vi);
        }

        b.set_phase("dialog_gcd_compressed_sidecar_tobitvector_reverse_branch_bits");
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

pub(crate) fn emit_dialog_gcd_compressed_sidecar_apply_bitvector(
    b: &mut B,
    compressed_log: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
    pair: &[QubitId],
    scratch: QubitId,
) {
    assert_eq!(x.len(), N);
    assert_eq!(y.len(), N);
    assert_eq!(pair.len(), 2);

    for step in (0..dialog_gcd_active_iterations()).rev() {
        b.set_phase("dialog_gcd_compressed_sidecar_apply_load_pair");
        let block = dialog_gcd_compressed_sidecar_block(compressed_log, step);
        emit_dialog_gcd_round763_compressed_block_swapper(
            b,
            pair,
            block,
            scratch,
            step % DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE,
        );

        let b0 = pair[0];
        let b0_and_b1 = pair[1];

        b.set_phase("dialog_gcd_compressed_sidecar_apply_double_y");
        mod_double_inplace_fast(b, y, p);

        b.set_phase("dialog_gcd_compressed_sidecar_apply_cadd");
        if dialog_gcd_raw_apply_materialized_special_add_enabled() {
            dialog_gcd_cmod_add_materialized_pseudomersenne(b, y, x, b0, p);
        } else if dialog_gcd_raw_apply_direct_special_add_enabled() {
            dialog_gcd_cmod_add_pseudomersenne_lowq(b, y, x, b0, p);
        } else {
            cmod_add_qq_lowq(b, y, x, b0, p);
        }

        b.set_phase("dialog_gcd_compressed_sidecar_apply_cswap");
        for (&xi, &yi) in x.iter().zip(y.iter()) {
            cswap(b, b0_and_b1, xi, yi);
        }

        b.set_phase("dialog_gcd_compressed_sidecar_apply_unload_pair");
        let block = dialog_gcd_compressed_sidecar_block(compressed_log, step);
        emit_dialog_gcd_round763_compressed_block_swapper(
            b,
            pair,
            block,
            scratch,
            step % DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE,
        );
    }
}

pub(crate) fn emit_dialog_gcd_compressed_sidecar_apply_bitvector_reverse_exact(
    b: &mut B,
    compressed_log: &[QubitId],
    x: &[QubitId],
    y: &[QubitId],
    p: U256,
    pair: &[QubitId],
    scratch: QubitId,
) {
    assert_eq!(x.len(), N);
    assert_eq!(y.len(), N);
    assert_eq!(pair.len(), 2);

    for step in 0..dialog_gcd_active_iterations() {
        b.set_phase("dialog_gcd_compressed_sidecar_apply_reverse_load_pair");
        let block = dialog_gcd_compressed_sidecar_block(compressed_log, step);
        emit_dialog_gcd_round763_compressed_block_swapper(
            b,
            pair,
            block,
            scratch,
            step % DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE,
        );

        let b0 = pair[0];
        let b0_and_b1 = pair[1];

        b.set_phase("dialog_gcd_compressed_sidecar_apply_reverse_cswap");
        for (&xi, &yi) in x.iter().zip(y.iter()) {
            cswap(b, b0_and_b1, xi, yi);
        }

        b.set_phase("dialog_gcd_compressed_sidecar_apply_reverse_csub");
        if dialog_gcd_raw_apply_reverse_materialized_special_sub_enabled() {
            dialog_gcd_cmod_sub_materialized_pseudomersenne(b, y, x, b0, p);
        } else if dialog_gcd_raw_apply_reverse_fast_sub_enabled() {
            cmod_sub_qq(b, y, x, b0, p);
        } else {
            cmod_sub_qq_lowq(b, y, x, b0, p);
        }

        b.set_phase("dialog_gcd_compressed_sidecar_apply_reverse_halve_y");
        mod_halve_inplace_fast(b, y, p);

        b.set_phase("dialog_gcd_compressed_sidecar_apply_reverse_unload_pair");
        let block = dialog_gcd_compressed_sidecar_block(compressed_log, step);
        emit_dialog_gcd_round763_compressed_block_swapper(
            b,
            pair,
            block,
            scratch,
            step % DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE,
        );
    }
}

pub(crate) fn emit_dialog_gcd_compressed_sidecar_ipmul_block_lifecycle(
    b: &mut B,
    factor: &[QubitId],
    target: &[QubitId],
    p: U256,
) {
    assert_eq!(factor.len(), N);
    assert_eq!(target.len(), N);

    let compressed_log = b.alloc_qubits(dialog_gcd_allocated_compressed_sidecar_bits());
    let raw_block = if dialog_gcd_host_reverse_raw_block_enabled() {
        Vec::new()
    } else {
        b.alloc_qubits(2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE)
    };
    let u = b.alloc_qubits(N);
    let runway = dialog_gcd_build_compressed_log_u_high_runway(&u, &compressed_log);
    let replay_log = runway
        .as_ref()
        .map_or(compressed_log.as_slice(), |r| r.remapped_log.as_slice());
    b.set_phase("dialog_gcd_compressed_block_ipmul_load_p");
    for i in 0..N {
        if bit(p, i) {
            b.x(u[i]);
        }
    }

    b.set_phase("dialog_gcd_compressed_block_ipmul_tobitvector");
    emit_dialog_gcd_compressed_sidecar_tobitvector_steps_block_lifecycle(
        b,
        &u,
        factor,
        replay_log,
        &raw_block,
    );

    if dialog_gcd_raw_ipmul_terminal_reuse_enabled() {
        b.set_phase("dialog_gcd_compressed_block_ipmul_release_terminal_u");
        b.x(u[0]);
        dialog_gcd_release_terminal_u(b, &u, runway.as_ref());

        b.set_phase("dialog_gcd_compressed_block_ipmul_apply_bitvector_reuse_factor_zero");
        let apply_raw_block = if dialog_gcd_host_reverse_raw_block_enabled() {
            b.alloc_qubits(2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE)
        } else {
            Vec::new()
        };
        emit_dialog_gcd_compressed_sidecar_apply_bitvector_block_lifecycle(
            b,
            replay_log,
            target,
            factor,
            p,
            if apply_raw_block.is_empty() { &raw_block } else { &apply_raw_block },
        );
        if !apply_raw_block.is_empty() {
            b.free_vec(&apply_raw_block);
        }

        if dialog_gcd_raw_ipmul_clear_p_residual_enabled() {
            b.set_phase("dialog_gcd_compressed_block_ipmul_clear_p_residual_source_lane");
            for i in 0..N {
                if bit(p, i) {
                    b.x(target[i]);
                }
            }
        }

        b.set_phase("dialog_gcd_compressed_block_ipmul_swap_product_into_target");
        for i in 0..N {
            b.swap(target[i], factor[i]);
        }

        b.set_phase("dialog_gcd_compressed_block_ipmul_reacquire_terminal_u");
        dialog_gcd_reacquire_terminal_u(b, &u, runway.as_ref());
        b.set_phase("dialog_gcd_compressed_block_ipmul_seed_terminal_u");
        b.x(u[0]);

        b.set_phase("dialog_gcd_compressed_block_ipmul_uncompute_tobitvector");
        emit_dialog_gcd_compressed_sidecar_tobitvector_steps_reverse_block_lifecycle(
            b,
            &u,
            factor,
            replay_log,
            &raw_block,
        );

        b.set_phase("dialog_gcd_compressed_block_ipmul_unload_p");
        for i in 0..N {
            if bit(p, i) {
                b.x(u[i]);
            }
        }
        b.free_vec(&u);
        if !raw_block.is_empty() {
            b.free_vec(&raw_block);
        }
        b.free_vec(&compressed_log);
        return;
    }

    let tmp = b.alloc_qubits(N);
    b.set_phase("dialog_gcd_compressed_block_ipmul_apply_bitvector");
    emit_dialog_gcd_compressed_sidecar_apply_bitvector_block_lifecycle(
        b,
        replay_log,
        target,
        &tmp,
        p,
        &raw_block,
    );

    b.set_phase("dialog_gcd_compressed_block_ipmul_swap_product_into_target");
    for i in 0..N {
        b.swap(target[i], tmp[i]);
    }

    b.set_phase("dialog_gcd_compressed_block_ipmul_free_zero_tmp");
    b.free_vec(&tmp);

    b.set_phase("dialog_gcd_compressed_block_ipmul_uncompute_tobitvector");
    emit_dialog_gcd_compressed_sidecar_tobitvector_steps_reverse_block_lifecycle(
        b,
        &u,
        factor,
        replay_log,
        &raw_block,
    );

    b.set_phase("dialog_gcd_compressed_block_ipmul_unload_p");
    for i in 0..N {
        if bit(p, i) {
            b.x(u[i]);
        }
    }
    b.free_vec(&u);
    b.free_vec(&raw_block);
    b.free_vec(&compressed_log);
}
pub(crate) fn emit_dialog_gcd_compressed_sidecar_ipmul(
    b: &mut B,
    factor: &[QubitId],
    target: &[QubitId],
    p: U256,
) {
    assert_eq!(factor.len(), N);
    assert_eq!(target.len(), N);

    if dialog_gcd_compressed_block_lifecycle_enabled() {
        emit_dialog_gcd_compressed_sidecar_ipmul_block_lifecycle(b, factor, target, p);
        return;
    }

    let compressed_log = b.alloc_qubits(dialog_gcd_compressed_sidecar_bits());
    let pair = b.alloc_qubits(2);
    let compressor_scratch = b.alloc_qubit();
    let u = b.alloc_qubits(N);
    b.set_phase("dialog_gcd_compressed_sidecar_ipmul_load_p");
    for i in 0..N {
        if bit(p, i) {
            b.x(u[i]);
        }
    }

    b.set_phase("dialog_gcd_compressed_sidecar_ipmul_tobitvector");
    emit_dialog_gcd_compressed_sidecar_tobitvector_steps(
        b,
        &u,
        factor,
        &compressed_log,
        &pair,
        compressor_scratch,
    );

    if dialog_gcd_raw_ipmul_terminal_reuse_enabled() {
        b.set_phase("dialog_gcd_compressed_sidecar_ipmul_release_terminal_u");
        b.x(u[0]);
        b.free_vec(&u);

        b.set_phase("dialog_gcd_compressed_sidecar_ipmul_apply_bitvector_reuse_factor_zero");
        emit_dialog_gcd_compressed_sidecar_apply_bitvector(
            b,
            &compressed_log,
            target,
            factor,
            p,
            &pair,
            compressor_scratch,
        );

        if dialog_gcd_raw_ipmul_clear_p_residual_enabled() {
            b.set_phase("dialog_gcd_compressed_sidecar_ipmul_clear_p_residual_source_lane");
            for i in 0..N {
                if bit(p, i) {
                    b.x(target[i]);
                }
            }
        }

        b.set_phase("dialog_gcd_compressed_sidecar_ipmul_swap_product_into_target");
        for i in 0..N {
            b.swap(target[i], factor[i]);
        }

        b.set_phase("dialog_gcd_compressed_sidecar_ipmul_reacquire_terminal_u");
        b.reacquire_vec(&u);
        b.set_phase("dialog_gcd_compressed_sidecar_ipmul_seed_terminal_u");
        b.x(u[0]);

        b.set_phase("dialog_gcd_compressed_sidecar_ipmul_uncompute_tobitvector");
        emit_dialog_gcd_compressed_sidecar_tobitvector_steps_reverse(
            b,
            &u,
            factor,
            &compressed_log,
            &pair,
            compressor_scratch,
        );

        b.set_phase("dialog_gcd_compressed_sidecar_ipmul_unload_p");
        for i in 0..N {
            if bit(p, i) {
                b.x(u[i]);
            }
        }
        b.free_vec(&u);
        b.free(compressor_scratch);
        b.free_vec(&pair);
        b.free_vec(&compressed_log);
        return;
    }

    let tmp = b.alloc_qubits(N);
    b.set_phase("dialog_gcd_compressed_sidecar_ipmul_apply_bitvector");
    emit_dialog_gcd_compressed_sidecar_apply_bitvector(
        b,
        &compressed_log,
        target,
        &tmp,
        p,
        &pair,
        compressor_scratch,
    );

    b.set_phase("dialog_gcd_compressed_sidecar_ipmul_swap_product_into_target");
    for i in 0..N {
        b.swap(target[i], tmp[i]);
    }

    b.set_phase("dialog_gcd_compressed_sidecar_ipmul_free_zero_tmp");
    b.free_vec(&tmp);

    b.set_phase("dialog_gcd_compressed_sidecar_ipmul_uncompute_tobitvector");
    emit_dialog_gcd_compressed_sidecar_tobitvector_steps_reverse(
        b,
        &u,
        factor,
        &compressed_log,
        &pair,
        compressor_scratch,
    );

    b.set_phase("dialog_gcd_compressed_sidecar_ipmul_unload_p");
    for i in 0..N {
        if bit(p, i) {
            b.x(u[i]);
        }
    }
    b.free_vec(&u);
    b.free(compressor_scratch);
    b.free_vec(&pair);
    b.free_vec(&compressed_log);
}

pub(crate) fn emit_dialog_gcd_compressed_sidecar_quotient_block_lifecycle(
    b: &mut B,
    factor: &[QubitId],
    target: &[QubitId],
    p: U256,
) {
    assert_eq!(factor.len(), N);
    assert_eq!(target.len(), N);

    let compressed_log = b.alloc_qubits(dialog_gcd_allocated_compressed_sidecar_bits());
    let raw_block = if dialog_gcd_host_reverse_raw_block_enabled() {
        Vec::new()
    } else {
        b.alloc_qubits(2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE)
    };
    let u = b.alloc_qubits(N);
    let runway = dialog_gcd_build_compressed_log_u_high_runway(&u, &compressed_log);
    let replay_log = runway
        .as_ref()
        .map_or(compressed_log.as_slice(), |r| r.remapped_log.as_slice());
    b.set_phase("dialog_gcd_compressed_block_quotient_load_p");
    for i in 0..N {
        if bit(p, i) {
            b.x(u[i]);
        }
    }

    b.set_phase("dialog_gcd_compressed_block_quotient_tobitvector");
    emit_dialog_gcd_compressed_sidecar_tobitvector_steps_block_lifecycle(
        b,
        &u,
        factor,
        replay_log,
        &raw_block,
    );

    if dialog_gcd_raw_quotient_terminal_reuse_enabled() {
        b.set_phase("dialog_gcd_compressed_block_quotient_release_terminal_u");
        b.x(u[0]);
        dialog_gcd_release_terminal_u(b, &u, runway.as_ref());

        b.set_phase("dialog_gcd_compressed_block_quotient_apply_reverse_reuse_factor_zero");
        let apply_raw_block = if dialog_gcd_host_reverse_raw_block_enabled() {
            b.alloc_qubits(2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE)
        } else {
            Vec::new()
        };
        emit_dialog_gcd_compressed_sidecar_apply_bitvector_reverse_exact_block_lifecycle(
            b,
            replay_log,
            factor,
            target,
            p,
            if apply_raw_block.is_empty() { &raw_block } else { &apply_raw_block },
        );
        if !apply_raw_block.is_empty() {
            b.free_vec(&apply_raw_block);
        }

        b.set_phase("dialog_gcd_compressed_block_quotient_swap_quotient_into_target");
        for i in 0..N {
            b.swap(target[i], factor[i]);
        }

        b.set_phase("dialog_gcd_compressed_block_quotient_reacquire_terminal_u");
        dialog_gcd_reacquire_terminal_u(b, &u, runway.as_ref());
        b.set_phase("dialog_gcd_compressed_block_quotient_seed_terminal_u");
        b.x(u[0]);

        b.set_phase("dialog_gcd_compressed_block_quotient_uncompute_tobitvector");
        emit_dialog_gcd_compressed_sidecar_tobitvector_steps_reverse_block_lifecycle(
            b,
            &u,
            factor,
            replay_log,
            &raw_block,
        );

        b.set_phase("dialog_gcd_compressed_block_quotient_unload_p");
        for i in 0..N {
            if bit(p, i) {
                b.x(u[i]);
            }
        }
        b.free_vec(&u);
        if !raw_block.is_empty() {
            b.free_vec(&raw_block);
        }
        b.free_vec(&compressed_log);
        return;
    }

    b.set_phase("dialog_gcd_compressed_block_quotient_apply_reverse");
    emit_dialog_gcd_compressed_sidecar_apply_bitvector_reverse_exact_block_lifecycle(
        b,
        replay_log,
        factor,
        target,
        p,
        &raw_block,
    );

    b.set_phase("dialog_gcd_compressed_block_quotient_uncompute_tobitvector");
    emit_dialog_gcd_compressed_sidecar_tobitvector_steps_reverse_block_lifecycle(
        b,
        &u,
        factor,
        replay_log,
        &raw_block,
    );

    b.set_phase("dialog_gcd_compressed_block_quotient_unload_p");
    for i in 0..N {
        if bit(p, i) {
            b.x(u[i]);
        }
    }
    b.free_vec(&u);
    b.free_vec(&raw_block);
    b.free_vec(&compressed_log);
}
pub(crate) fn emit_dialog_gcd_compressed_sidecar_quotient(
    b: &mut B,
    factor: &[QubitId],
    target: &[QubitId],
    p: U256,
) {
    assert_eq!(factor.len(), N);
    assert_eq!(target.len(), N);

    if dialog_gcd_compressed_block_lifecycle_enabled() {
        emit_dialog_gcd_compressed_sidecar_quotient_block_lifecycle(b, factor, target, p);
        return;
    }

    let compressed_log = b.alloc_qubits(dialog_gcd_compressed_sidecar_bits());
    let pair = b.alloc_qubits(2);
    let compressor_scratch = b.alloc_qubit();
    let u = b.alloc_qubits(N);
    b.set_phase("dialog_gcd_compressed_sidecar_quotient_load_p");
    for i in 0..N {
        if bit(p, i) {
            b.x(u[i]);
        }
    }

    b.set_phase("dialog_gcd_compressed_sidecar_quotient_tobitvector");
    emit_dialog_gcd_compressed_sidecar_tobitvector_steps(
        b,
        &u,
        factor,
        &compressed_log,
        &pair,
        compressor_scratch,
    );

    if dialog_gcd_raw_quotient_terminal_reuse_enabled() {
        b.set_phase("dialog_gcd_compressed_sidecar_quotient_release_terminal_u");
        b.x(u[0]);
        b.free_vec(&u);

        b.set_phase("dialog_gcd_compressed_sidecar_quotient_apply_reverse_reuse_factor_zero");
        emit_dialog_gcd_compressed_sidecar_apply_bitvector_reverse_exact(
            b,
            &compressed_log,
            factor,
            target,
            p,
            &pair,
            compressor_scratch,
        );

        b.set_phase("dialog_gcd_compressed_sidecar_quotient_swap_quotient_into_target");
        for i in 0..N {
            b.swap(target[i], factor[i]);
        }

        b.set_phase("dialog_gcd_compressed_sidecar_quotient_reacquire_terminal_u");
        b.reacquire_vec(&u);
        b.set_phase("dialog_gcd_compressed_sidecar_quotient_seed_terminal_u");
        b.x(u[0]);

        b.set_phase("dialog_gcd_compressed_sidecar_quotient_uncompute_tobitvector");
        emit_dialog_gcd_compressed_sidecar_tobitvector_steps_reverse(
            b,
            &u,
            factor,
            &compressed_log,
            &pair,
            compressor_scratch,
        );

        b.set_phase("dialog_gcd_compressed_sidecar_quotient_unload_p");
        for i in 0..N {
            if bit(p, i) {
                b.x(u[i]);
            }
        }
        b.free_vec(&u);
        b.free(compressor_scratch);
        b.free_vec(&pair);
        b.free_vec(&compressed_log);
        return;
    }

    b.set_phase("dialog_gcd_compressed_sidecar_quotient_apply_reverse");
    emit_dialog_gcd_compressed_sidecar_apply_bitvector_reverse_exact(
        b,
        &compressed_log,
        factor,
        target,
        p,
        &pair,
        compressor_scratch,
    );

    b.set_phase("dialog_gcd_compressed_sidecar_quotient_uncompute_tobitvector");
    emit_dialog_gcd_compressed_sidecar_tobitvector_steps_reverse(
        b,
        &u,
        factor,
        &compressed_log,
        &pair,
        compressor_scratch,
    );

    b.set_phase("dialog_gcd_compressed_sidecar_quotient_unload_p");
    for i in 0..N {
        if bit(p, i) {
            b.x(u[i]);
        }
    }
    b.free_vec(&u);
    b.free(compressor_scratch);
    b.free_vec(&pair);
    b.free_vec(&compressed_log);
}


pub(crate) fn assert_qubit_slices_disjoint(slices: &[&[QubitId]]) {
    let mut seen = std::collections::BTreeSet::new();
    for slice in slices {
        for &q in *slice {
            assert!(seen.insert(q), "scratch lane q{} aliases an operand", q.0);
        }
    }
}

pub(crate) fn dialog_gcd_slice_intersects(a: &[QubitId], b: &[QubitId]) -> bool {
    a.iter().any(|q| b.contains(q))
}

#[derive(Clone, Debug)]
pub(crate) struct DialogGcdCompressedLogUHighRunway {
    remapped_log: Vec<QubitId>,
    parked_u_indices: Vec<usize>,
}

pub(crate) fn dialog_gcd_allocated_compressed_sidecar_bits() -> usize {
    if dialog_gcd_compressed_log_u_high_runway_enabled() {
        dialog_gcd_compressed_sidecar_bits() - dialog_gcd_runway_layout().len()
    } else {
        dialog_gcd_compressed_sidecar_bits()
    }
}

pub(crate) fn dialog_gcd_apply_replay_swap_host_enabled() -> bool {
    // Prototype, deliberately NOT enabled by configure_ecdsafail_submission_route.
    //
    // Block-lifecycle apply normally CNOT-copies the current compressed
    // transcript block into raw_block before decompressing it.  Swapping the
    // five compressed cells into raw_block instead leaves five allocated,
    // clean cells available throughout the three replay steps.  The matching
    // swap after recompression restores the transcript block.
    std::env::var("DIALOG_GCD_APPLY_REPLAY_SWAP_HOST")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_build_compressed_log_u_high_runway(
    u: &[QubitId],
    allocated_log: &[QubitId],
) -> Option<DialogGcdCompressedLogUHighRunway> {
    if !dialog_gcd_compressed_log_u_high_runway_enabled() {
        return None;
    }
    assert_eq!(u.len(), N);
    let layout = dialog_gcd_runway_layout();
    if layout.is_empty() {
        return None;
    }

    let expected_allocated = dialog_gcd_compressed_sidecar_bits() - layout.len();
    assert_eq!(allocated_log.len(), expected_allocated);
    let first_relocated = layout[0].0;
    assert_eq!(first_relocated, allocated_log.len());
    let mut remapped_log = allocated_log.to_vec();
    let mut parked_u_indices = Vec::with_capacity(layout.len());
    for (log_index, u_index) in layout {
        // These logical transcript cells are not needed until their late
        // forward blocks, when the width envelope guarantees that u[u_index] is
        // inactive and |0>.  Reverse consumes them before u grows back into the
        // same hosts.
        assert_eq!(log_index, remapped_log.len());
        remapped_log.push(u[u_index]);
        parked_u_indices.push(u_index);
    }
    assert_eq!(remapped_log.len(), dialog_gcd_compressed_sidecar_bits());
    Some(DialogGcdCompressedLogUHighRunway {
        remapped_log,
        parked_u_indices,
    })
}

pub(crate) fn dialog_gcd_compressed_log_u_high_runway_blocks() -> usize {
    // Optional tuning cap for the prototype.  The uncapped layout parks the
    // longest suffix; lowering the cap is useful when balancing wrapper savings
    // against reverse-replay scratch pressure.  On the accepted a8d8d5a route,
    // 16 whole blocks is the largest prefix-independent tail runway before the
    // reverse add loses its cheap scratch host.  Keep larger schedules available
    // as an explicit experiment, but default the opt-in prototype to that safe
    // subset.
    std::env::var("DIALOG_GCD_COMPRESSED_LOG_U_HIGH_RUNWAY_BLOCKS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(16)
}

pub(crate) fn dialog_gcd_compressed_log_u_high_runway_enabled() -> bool {
    // Prototype, deliberately NOT enabled by configure_ecdsafail_submission_route.
    //
    // The wrapper used to allocate all of u and the complete compressed
    // transcript at once.  Instead, a late transcript suffix can use high u
    // lanes: those cells are not touched until forward replay has shrunk u below
    // their hosts, stay live across terminal-reuse apply, and are consumed by
    // reverse replay before u grows back into them.
    //
    // This is an experimental support-envelope optimization: it relies on the
    // same terminal convergence and width envelope as terminal reuse and
    // variable-width tobitvector.  Default OFF keeps the accepted route
    // byte-identical.
    std::env::var("DIALOG_GCD_COMPRESSED_LOG_U_HIGH_RUNWAY")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) fn dialog_gcd_pick_runway_safe_borrow_slice<'a>(
    future: Option<&'a [QubitId]>,
    u: &'a [QubitId],
    compressed_log: &[QubitId],
    active_width: usize,
) -> Option<&'a [QubitId]> {
    if !dialog_gcd_compressed_log_u_high_runway_enabled() {
        return dialog_gcd_pick_borrow_slice(future, u, active_width);
    }

    let safe_future = dialog_gcd_runway_safe_future_prefix(future, u, active_width);
    if dialog_gcd_late_borrow_uv_high_enabled() && active_width >= 1 {
        let want = 2 * active_width - 1;
        let short = safe_future.map_or(true, |slice| slice.len() < want);
        if short && u.len() >= active_width + want {
            let candidate = &u[active_width..active_width + want];
            // Parked cells can still carry unread transcript data.  Be
            // conservative: only use an in-place high-u fallback when it is
            // disjoint from every logical transcript cell, including clean
            // parked cells already consumed by reverse replay.
            if !dialog_gcd_slice_intersects(candidate, compressed_log) {
                return Some(candidate);
            }
        }
    }
    safe_future
}

pub(crate) fn dialog_gcd_reacquire_terminal_u(
    b: &mut B,
    u: &[QubitId],
    runway: Option<&DialogGcdCompressedLogUHighRunway>,
) {
    for (index, &q) in u.iter().enumerate() {
        if runway.is_none_or(|r| !r.parked_u_indices.contains(&index)) {
            b.reacquire(q);
        }
    }
}

pub(crate) fn dialog_gcd_release_terminal_u(
    b: &mut B,
    u: &[QubitId],
    runway: Option<&DialogGcdCompressedLogUHighRunway>,
) {
    for (index, &q) in u.iter().enumerate() {
        if runway.is_none_or(|r| !r.parked_u_indices.contains(&index)) {
            b.free(q);
        }
    }
}

pub(crate) fn dialog_gcd_runway_layout() -> Vec<(usize, usize)> {
    // Leave the top six u lanes unparked.  The accepted a8d8d5a route hosts a
    // raw 3-step block there whenever the tail is wide enough; reserving those
    // lanes keeps that scratch host disjoint from parked transcript cells.
    let raw_block_bits = 2 * DIALOG_GCD_HIGH_TAIL_ALIAS_GROUP_SIZE;
    let Some(highest_host) = N.checked_sub(raw_block_bits + 1) else {
        return Vec::new();
    };
    let blocks = dialog_gcd_compressed_sidecar_blocks();

    // Find the longest whole-block suffix that fits.  Blocks are assigned in
    // forward order to descending u positions: the earliest parked block gets
    // the highest hosts because it is replayed last and therefore needs the
    // widest inactive-u threshold.
    let first_allowed = blocks.saturating_sub(dialog_gcd_compressed_log_u_high_runway_blocks());
    for first_block in first_allowed..blocks {
        let mut next_host = highest_host;
        let mut layout = Vec::with_capacity(
            (blocks - first_block) * DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS,
        );
        let mut fits = true;
        for block in first_block..blocks {
            let (start, end) = dialog_gcd_compressed_sidecar_block_step_range(block);
            let active_threshold = (start..end)
                .map(dialog_gcd_tobitvector_active_width)
                .max()
                .unwrap_or(1);
            for slot in 0..DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS {
                if next_host < active_threshold {
                    fits = false;
                    break;
                }
                layout.push((
                    block * DIALOG_GCD_HIGH_TAIL_ALIAS_BLOCK_BITS + slot,
                    next_host,
                ));
                let Some(next) = next_host.checked_sub(1) else {
                    fits = false;
                    break;
                };
                next_host = next;
            }
            if !fits {
                break;
            }
        }
        if fits {
            return layout;
        }
    }
    Vec::new()
}

pub(crate) fn dialog_gcd_runway_safe_future_prefix<'a>(
    future: Option<&'a [QubitId]>,
    u: &[QubitId],
    active_width: usize,
) -> Option<&'a [QubitId]> {
    let active_u = &u[..active_width];
    future
        .map(|slice| {
            let safe = slice
                .iter()
                .position(|q| active_u.contains(q))
                .unwrap_or(slice.len());
            &slice[..safe]
        })
        .filter(|slice| !slice.is_empty())
}

pub(crate) fn dialog_gcd_composite_scratch_enabled() -> bool {
    std::env::var("DIALOG_GCD_COMPOSITE_SCRATCH")
        .ok()
        .as_deref()
        == Some("1")
}

pub(crate) struct DialogGcdCompositeScratch {
    lanes: Vec<QubitId>,
    owned: Vec<QubitId>,
}

pub(crate) fn dialog_gcd_build_composite_scratch(
    b: &mut B,
    future: Option<&[QubitId]>,
    u: &[QubitId],
    v: &[QubitId],
    compressed_log: &[QubitId],
    raw_block: &[QubitId],
    active_width: usize,
) -> DialogGcdCompositeScratch {
    let want = 2 * active_width - 1;
    let mut lanes = Vec::with_capacity(want);
    let mut push = |q: QubitId| {
        if lanes.len() < want
            && !lanes.contains(&q)
            && !raw_block.contains(&q)
            && !u[..active_width].contains(&q)
            && !v[..active_width].contains(&q)
        {
            lanes.push(q);
        }
    };
    if let Some(future) = dialog_gcd_runway_safe_future_prefix(future, u, active_width) {
        for &q in future {
            push(q);
        }
    }
    for &q in &v[active_width..] {
        push(q);
    }
    for &q in &u[active_width..] {
        if !compressed_log.contains(&q) {
            push(q);
        }
    }
    let owned = b.alloc_qubits(want - lanes.len());
    lanes.extend_from_slice(&owned);
    DialogGcdCompositeScratch { lanes, owned }
}