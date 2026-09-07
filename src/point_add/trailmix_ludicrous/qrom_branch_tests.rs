use super::*;
use crate::circuit::QubitOrBit;
use crate::sim::Simulator;
use alloy_primitives::U256;
use sha3::digest::{ExtendableOutput, Update, XofReader};
use sha3::Shake256;

/// All columns share one measurement record, representing a single coherent branch.
struct BranchRandom {
    source: sha3::Shake256Reader,
    fixed: Option<u8>,
}
impl XofReader for BranchRandom {
    fn read(&mut self, bytes: &mut [u8]) {
        let mut bit = [0u8; 1];
        self.source.read(&mut bit);
        bytes.fill(if self.fixed.unwrap_or(bit[0]) & 1 != 0 { 255 } else { 0 });
    }
}

fn qr(qs: &[QubitId]) -> Vec<QubitOrBit> {
    qs.iter().copied().map(QubitOrBit::Qubit).collect()
}

pub(super) fn windowed_qrom_coherent_branches() {
    // Exercise the production unary and two-row-blocked lookup/replay paths.
    for width in 0..=5 {
        for (two_columns, blocked) in [(false, false), (true, false), (false, true)] {
            if blocked && width < 3 { continue; }
            std::env::set_var("WINDOWED_QROM_MODE", if blocked { "qroam2" } else { "unary" });
            std::env::remove_var("WINDOWED_QROM_UNLOAD");
            let rows = 1usize << width;
            let mut b = B::new();
            let address = b.alloc_qubits(width);
            let output = b.alloc_qubits(256);
            let output2 = b.alloc_qubits(256);
            let payload = b.alloc_qubits(256);
            let table: Vec<_> = (0..rows).map(|_| b.alloc_bits(256)).collect();
            let table2: Vec<_> = (0..rows).map(|_| b.alloc_bits(256)).collect();
            if two_columns {
                let payload2 = b.alloc_qubits(256);
                let ws = load_two_words(&mut b, &address, &payload, &table, &payload2, &table2, "load", "clear");
                for i in 0..256 {
                    b.cx(payload[i], output[i]);
                    b.cx(payload2[i], output2[i]);
                }
                unload_two_words(&mut b, &address, &payload, &table, &payload2, &table2, ws, "replay", "clear");
            } else {
                let ws = load_word(&mut b, &address, &payload, &table, "load", "clear");
                for i in 0..256 { b.cx(payload[i], output[i]); }
                unload_word(&mut b, &address, &payload, &table, ws, "replay", "clear");
            }
            let (nq, nb) = (b.next_qubit, b.next_bit);
            let ops = b.take_ops();
            let mut table_rng = Shake256::default();
            table_rng.update(b"windowed-qrom-branch-table-v1");
            let mut table_rng = table_rng.finalize_xof();
            let mut words = Vec::new();
            let mut words2 = Vec::new();
            for _ in 0..rows {
                let mut bytes = [0; 32];
                table_rng.read(&mut bytes);
                words.push(U256::from_le_bytes(bytes));
                table_rng.read(&mut bytes);
                words2.push(U256::from_le_bytes(bytes));
            }
            for mode in 0..4 {
                let mut rng = Shake256::default();
                rng.update(b"windowed-qrom-measurement-branch-v1");
                rng.update(&[mode]);
                let mut rng = BranchRandom {
                    source: rng.finalize_xof(),
                    fixed: if mode < 2 { Some(mode) } else { None },
                };
                let mut sim = Simulator::new(nq as usize, nb as usize, &mut rng);
                for j in 0..rows {
                    for i in 0..256 {
                        *sim.bit_mut(table[j][i]) = if words[j].bit(i) { u64::MAX } else { 0 };
                        *sim.bit_mut(table2[j][i]) = if words2[j].bit(i) { u64::MAX } else { 0 };
                    }
                }
                for column in 0..rows {
                    sim.set_register(&qr(&address), U256::from(column), column);
                    sim.set_register(&qr(&output), U256::from(column * 37), column);
                    sim.set_register(&qr(&output2), U256::from(column * 13), column);
                }
                let mask = (1u64 << rows) - 1;
                sim.apply_iter_masked(ops.iter(), mask);
                for column in 0..rows {
                    assert_eq!(sim.get_register(&qr(&address), column), U256::from(column));
                    assert_eq!(sim.get_register(&qr(&output), column), U256::from(column * 37) ^ words[column]);
                    let expected2 = U256::from(column * 13) ^ if two_columns { words2[column] } else { U256::ZERO };
                    assert_eq!(sim.get_register(&qr(&output2), column), expected2);
                }
                assert_eq!(sim.phase & mask, 0, "address-dependent phase, w={width}");
                for q in 0..nq {
                    let id = QubitId(q as u64);
                    if !address.contains(&id) && !output.contains(&id) && !output2.contains(&id) {
                        assert_eq!(sim.qubit(id) & mask, 0, "dirty workspace, w={width}");
                    }
                }
                // Each basis component has the correct image and identical phase.
                // Thus arbitrary complex amplitudes over these components are preserved.
                // Hmr branches have amplitude 2^(-m/2); the routing identity proves
                // this factor is independent of the component.
            }
        }
    }
    std::env::remove_var("WINDOWED_QROM_MODE");
    eprintln!("QROM coherent-branch checks passed: unary and K=2, w=0..5, four measurement records per layout");
}

pub(super) fn windowed_qrom_all_addresses() {
    const W: usize = 16;
    const WORD: usize = 16;
    let rows = 1usize << W;
    for variant in 0..3 {
        // A 16-bit row label is injective over all addresses. Routing and replay
        // use the production emitters; the 256-bit payload loop is tested above.
        let mut b = B::new();
        let address = b.alloc_qubits(W);
        let output = b.alloc_qubits(WORD);
        let output2 = b.alloc_qubits(WORD);
        let payload = b.alloc_qubits(WORD);
        let payload2 = b.alloc_qubits(WORD);
        let table: Vec<_> = (0..rows).map(|_| b.alloc_bits(WORD)).collect();
        let table2: Vec<_> = (0..rows).map(|_| b.alloc_bits(WORD)).collect();
        match variant {
            0 => emit_load(&mut b, &address, &payload, &table),
            1 => emit_two_load(&mut b, &address, &payload, &table, &payload2, &table2),
            _ => emit_qroam2_load(&mut b, &address, &payload, &payload2, &table),
        }
        for i in 0..WORD {
            b.cx(payload[i], output[i]);
            if variant == 1 { b.cx(payload2[i], output2[i]); }
        }
        match variant {
            0 => emit_load(&mut b, &address, &payload, &table),
            1 => emit_two_load(&mut b, &address, &payload, &table, &payload2, &table2),
            _ => emit_qroam2_replay_unload(&mut b, &address, &payload, &payload2, &table),
        }
        let (nq, nb) = (b.next_qubit, b.next_bit);
        let ops = b.take_ops();
        let mut rng = Shake256::default();
        rng.update(b"windowed-pingpong-routing-coverage-v1");
        let mut rng = rng.finalize_xof();
        let mut sim = Simulator::new(nq as usize, nb as usize, &mut rng);
        for j in 0..rows {
            for i in 0..WORD {
                *sim.bit_mut(table[j][i]) = if j >> i & 1 != 0 { u64::MAX } else { 0 };
                *sim.bit_mut(table2[j][i]) = if j >> i & 1 == 0 { u64::MAX } else { 0 };
            }
        }
        for batch in 0..rows / 64 {
            sim.qubits.fill(0);
            sim.phase = 0;
            for lane in 0..64 {
                sim.set_register(&qr(&address), U256::from(batch * 64 + lane), lane);
            }
            sim.apply_iter(ops.iter());
            for lane in 0..64 {
                let j = batch * 64 + lane;
                assert_eq!(sim.get_register(&qr(&address), lane), U256::from(j));
                assert_eq!(sim.get_register(&qr(&output), lane), U256::from(j));
                assert_eq!(sim.get_register(&qr(&output2), lane), U256::from(if variant == 1 { j ^ 65535 } else { 0 }));
            }
            assert_eq!(sim.phase, 0);
            for q in 0..nq {
                let id = QubitId(q as u64);
                if !address.contains(&id) && !output.contains(&id) && !output2.contains(&id) {
                    assert_eq!(sim.qubit(id), 0);
                }
            }
        }
        eprintln!("QROM address coverage passed: variant={variant}, 65,536 addresses, no address/phase/workspace failures");
    }
}
