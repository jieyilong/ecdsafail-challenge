//! `B` reversible-circuit builder: the gate-emission API + phase/peak
//! accounting that every emitter calls. Holds the op stream, qubit/bit
//! allocator and peak tracker; `impl B` is cx/ccx/alloc/free/set_phase/
//! hmr/cz_if/…; `PhaseResource`/`phase_resources` summarize per-phase cost.

#![allow(unused_imports, dead_code, clippy::all)]
use alloy_primitives::U256;
use crate::circuit::{analyze_ops, BitId, Op, OperationType, QubitId, QubitOrBit, RegisterId};
use crate::sim::Simulator;
use super::*;

pub(crate) struct B {
    pub ops: Vec<Op>,
    pub count_only: bool,
    pub counted_ops: usize,
    pub counted_kind_ops: [usize; 18],
    pub counted_phase_kind_ops: [usize; 18],
    pub counted_phase_start_ops: usize,
    pub counted_phase_rows: Vec<PhaseResource>,
    pub counted_registers: Vec<Vec<QubitOrBit>>,
    pub next_qubit: u32,
    pub next_bit: u32,
    pub next_register: u32,
    pub free_qubits: Vec<u32>,
    pub active_qubits: u32,
    pub peak_qubits: u32,
    pub peak_ops_idx: usize,
    pub peak_phase: &'static str,
    pub phase: &'static str,
    pub peak_log: Vec<(u32, &'static str, usize)>,
    pub phase_active_max: std::collections::BTreeMap<&'static str, u32>,
    pub phase_active_regions: Vec<(usize, &'static str, u32)>,
    pub current_phase_active_max: u32,
    // (ops_len_at_transition, new_phase)
    pub phase_transitions: Vec<(usize, &'static str)>,
}

#[derive(Clone, Copy)]
pub(crate) struct CountSnapshot {
    ops: usize,
    kind_ops: [usize; 18],
    phase_kind_ops: [usize; 18],
    phase_start_ops: usize,
    phase_rows_len: usize,
    phase: &'static str,
}

#[derive(Clone, Debug)]
pub struct PhaseResource {
    pub phase: &'static str,
    pub start: usize,
    pub end: usize,
    pub ops: usize,
    pub toffoli_ops: usize,
    pub ccx_ops: usize,
    pub ccz_ops: usize,
    pub hmr_ops: usize,
    pub r_ops: usize,
}

pub(crate) fn phase_resources(ops: &[Op], transitions: &[(usize, &'static str)]) -> Vec<PhaseResource> {
    let mut bounds: Vec<(usize, &'static str)> = Vec::new();
    bounds.push((0, "init"));
    for &(idx, phase) in transitions {
        if idx <= ops.len() {
            bounds.push((idx, phase));
        }
    }
    bounds.sort_by_key(|&(idx, _)| idx);
    bounds.dedup_by(|a, b| {
        if a.0 == b.0 {
            b.1 = a.1;
            true
        } else {
            false
        }
    });

    let mut rows = Vec::new();
    for i in 0..bounds.len() {
        let start = bounds[i].0;
        let end = bounds.get(i + 1).map(|(idx, _)| *idx).unwrap_or(ops.len());
        if start >= end {
            continue;
        }
        let slice = &ops[start..end];
        let ccx_ops = slice
            .iter()
            .filter(|op| matches!(op.kind, OperationType::CCX))
            .count();
        let ccz_ops = slice
            .iter()
            .filter(|op| matches!(op.kind, OperationType::CCZ))
            .count();
        let hmr_ops = slice
            .iter()
            .filter(|op| matches!(op.kind, OperationType::Hmr))
            .count();
        let r_ops = slice
            .iter()
            .filter(|op| matches!(op.kind, OperationType::R))
            .count();
        rows.push(PhaseResource {
            phase: bounds[i].1,
            start,
            end,
            ops: end - start,
            toffoli_ops: ccx_ops + ccz_ops,
            ccx_ops,
            ccz_ops,
            hmr_ops,
            r_ops,
        });
    }
    rows
}

impl B {
    pub(crate) fn new() -> Self {
        Self {
            ops: Vec::new(),
            count_only: false,
            counted_ops: 0,
            counted_kind_ops: [0; 18],
            counted_phase_kind_ops: [0; 18],
            counted_phase_start_ops: 0,
            counted_phase_rows: Vec::new(),
            counted_registers: Vec::new(),
            next_qubit: 0,
            next_bit: 0,
            next_register: 0,
            free_qubits: Vec::new(),
            active_qubits: 0,
            peak_qubits: 0,
            peak_ops_idx: 0,
            peak_phase: "",
            phase: "init",
            peak_log: Vec::new(),
            phase_active_max: std::collections::BTreeMap::new(),
            phase_active_regions: Vec::new(),
            current_phase_active_max: 0,
            phase_transitions: Vec::new(),
        }
    }
    pub(crate) fn new_count_only() -> Self {
        let mut b = Self::new();
        b.count_only = true;
        b
    }
    pub(crate) fn push_op(&mut self, op: Op) {
        self.counted_ops += 1;
        self.counted_kind_ops[op.kind as usize] += 1;
        self.counted_phase_kind_ops[op.kind as usize] += 1;
        if !self.count_only {
            self.ops.push(op);
        }
    }
    pub(crate) fn count_snapshot(&self) -> CountSnapshot {
        CountSnapshot {
            ops: self.counted_ops,
            kind_ops: self.counted_kind_ops,
            phase_kind_ops: self.counted_phase_kind_ops,
            phase_start_ops: self.counted_phase_start_ops,
            phase_rows_len: self.counted_phase_rows.len(),
            phase: self.phase,
        }
    }
    pub(crate) fn count_delta_since(&self, snap: CountSnapshot) -> [usize; 18] {
        let mut out = [0usize; 18];
        for (idx, slot) in out.iter_mut().enumerate() {
            *slot = self.counted_kind_ops[idx] - snap.kind_ops[idx];
        }
        out
    }
    pub(crate) fn restore_count_snapshot(&mut self, snap: CountSnapshot) {
        self.counted_ops = snap.ops;
        self.counted_kind_ops = snap.kind_ops;
        self.counted_phase_kind_ops = snap.phase_kind_ops;
        self.counted_phase_start_ops = snap.phase_start_ops;
        self.counted_phase_rows.truncate(snap.phase_rows_len);
        self.phase = snap.phase;
    }
    pub(crate) fn add_counted_kind(&mut self, kind: OperationType, count: usize) {
        self.counted_ops += count;
        self.counted_kind_ops[kind as usize] += count;
        self.counted_phase_kind_ops[kind as usize] += count;
    }
    pub(crate) fn current_ops_len(&self) -> usize {
        if self.count_only {
            self.counted_ops
        } else {
            self.ops.len()
        }
    }
    pub(crate) fn close_counted_phase(&mut self) {
        if !self.count_only {
            return;
        }
        let start = self.counted_phase_start_ops;
        let end = self.counted_ops;
        if start < end {
            let ccx_ops = self.counted_phase_kind_ops[OperationType::CCX as usize];
            let ccz_ops = self.counted_phase_kind_ops[OperationType::CCZ as usize];
            let hmr_ops = self.counted_phase_kind_ops[OperationType::Hmr as usize];
            let r_ops = self.counted_phase_kind_ops[OperationType::R as usize];
            self.counted_phase_rows.push(PhaseResource {
                phase: self.phase,
                start,
                end,
                ops: end - start,
                toffoli_ops: ccx_ops + ccz_ops,
                ccx_ops,
                ccz_ops,
                hmr_ops,
                r_ops,
            });
        }
        self.counted_phase_start_ops = self.counted_ops;
        self.counted_phase_kind_ops = [0; 18];
    }
    pub(crate) fn set_phase(&mut self, p: &'static str) {
        self.close_phase_active_region();
        self.close_counted_phase();
        self.phase = p;
        if std::env::var("TRACE_PHASE_ACTIVE").is_ok() {
            self.current_phase_active_max = self.active_qubits;
        }
        self.phase_transitions.push((self.current_ops_len(), p));
    }
    pub(crate) fn record_phase_active(&mut self) {
        if std::env::var("TRACE_PHASE_ACTIVE").is_ok() {
            let entry = self.phase_active_max.entry(self.phase).or_insert(0);
            if self.active_qubits > *entry {
                *entry = self.active_qubits;
            }
            if self.active_qubits > self.current_phase_active_max {
                self.current_phase_active_max = self.active_qubits;
            }
        }
    }
    pub(crate) fn close_phase_active_region(&mut self) {
        if std::env::var("TRACE_PHASE_ACTIVE").is_ok() && self.current_phase_active_max > 0 {
            self.phase_active_regions.push((
                self.current_ops_len(),
                self.phase,
                self.current_phase_active_max,
            ));
            self.current_phase_active_max = 0;
        }
    }
    pub(crate) fn alloc_qubit(&mut self) -> QubitId {
        self.active_qubits += 1;
        self.record_phase_active();
        if self.active_qubits > self.peak_qubits {
            self.peak_qubits = self.active_qubits;
            self.peak_ops_idx = self.current_ops_len();
            self.peak_phase = self.phase;
            if std::env::var("TRACE_EACH_PEAK").is_ok() {
                eprintln!(
                    "PEAK active={} next_idx={} phase='{}' ops_idx={}",
                    self.active_qubits,
                    self.next_qubit,
                    self.phase,
                    self.current_ops_len()
                );
            }
        }
        if std::env::var("TRACE_PEAK").is_ok() && self.active_qubits + 10 >= self.peak_qubits {
            self.peak_log
                .push((self.active_qubits, self.phase, self.current_ops_len()));
        }
        if let Some(q) = self.free_qubits.pop() {
            QubitId(q.into())
        } else {
            let q = self.next_qubit;
            self.next_qubit += 1;
            QubitId(q.into())
        }
    }
    pub(crate) fn alloc_qubits(&mut self, n: usize) -> Vec<QubitId> {
        (0..n).map(|_| self.alloc_qubit()).collect()
    }
    pub(crate) fn alloc_bit(&mut self) -> BitId {
        let b = self.next_bit;
        self.next_bit += 1;
        BitId(b.into())
    }
    pub(crate) fn alloc_bits(&mut self, n: usize) -> Vec<BitId> {
        (0..n).map(|_| self.alloc_bit()).collect()
    }
    pub(crate) fn free(&mut self, q: QubitId) {
        self.r(q);
        self.free_qubits
            .push(q.0.try_into().expect("qubit id fits in u32"));
        if self.active_qubits > 0 {
            self.active_qubits -= 1;
        }
    }
    pub(crate) fn free_vec(&mut self, qs: &[QubitId]) {
        for &q in qs {
            self.free(q);
        }
    }
    pub(crate) fn reacquire(&mut self, q: QubitId) {
        let pos = self
            .free_qubits
            .iter()
            .position(|&free_q| u64::from(free_q) == q.0)
            .expect("reacquire qubit that is not currently free");
        self.free_qubits.swap_remove(pos);
        self.active_qubits += 1;
        self.record_phase_active();
        if self.active_qubits > self.peak_qubits {
            self.peak_qubits = self.active_qubits;
            self.peak_ops_idx = self.current_ops_len();
            self.peak_phase = self.phase;
            if std::env::var("TRACE_EACH_PEAK").is_ok() {
                eprintln!(
                    "PEAK active={} next_idx={} phase='{}' ops_idx={}",
                    self.active_qubits,
                    self.next_qubit,
                    self.phase,
                    self.current_ops_len()
                );
            }
        }
        if std::env::var("TRACE_PEAK").is_ok() && self.active_qubits + 10 >= self.peak_qubits {
            self.peak_log
                .push((self.active_qubits, self.phase, self.current_ops_len()));
        }
    }
    pub(crate) fn reacquire_vec(&mut self, qs: &[QubitId]) {
        for &q in qs {
            self.reacquire(q);
        }
    }
    pub(crate) fn declare_qubit_register(&mut self, qs: &[QubitId]) {
        let r = RegisterId(self.next_register.into());
        self.next_register += 1;
        for &q in qs {
            while self.counted_registers.len() <= r.0 as usize {
                self.counted_registers.push(Vec::new());
            }
            self.counted_registers[r.0 as usize].push(QubitOrBit::Qubit(q));
            let mut op = Op::empty();
            op.kind = OperationType::AppendToRegister;
            op.q_target = q;
            op.r_target = r;
            self.push_op(op);
        }
        let mut op = Op::empty();
        op.kind = OperationType::Register;
        op.r_target = r;
        self.push_op(op);
    }
    pub(crate) fn declare_bit_register(&mut self, bs: &[BitId]) {
        let r = RegisterId(self.next_register.into());
        self.next_register += 1;
        for &b in bs {
            while self.counted_registers.len() <= r.0 as usize {
                self.counted_registers.push(Vec::new());
            }
            self.counted_registers[r.0 as usize].push(QubitOrBit::Bit(b));
            let mut op = Op::empty();
            op.kind = OperationType::AppendToRegister;
            op.c_target = b;
            op.r_target = r;
            self.push_op(op);
        }
        let mut op = Op::empty();
        op.kind = OperationType::Register;
        op.r_target = r;
        self.push_op(op);
    }
    pub(crate) fn x(&mut self, q: QubitId) {
        let mut op = Op::empty();
        op.kind = OperationType::X;
        op.q_target = q;
        self.push_op(op);
    }
    pub(crate) fn z(&mut self, q: QubitId) {
        let mut op = Op::empty();
        op.kind = OperationType::Z;
        op.q_target = q;
        self.push_op(op);
    }
    pub(crate) fn cx(&mut self, ctrl: QubitId, tgt: QubitId) {
        if ctrl == tgt {
            panic!("invalid CX with aliased control/target {:?}", ctrl);
        }
        let mut op = Op::empty();
        op.kind = OperationType::CX;
        op.q_control1 = ctrl;
        op.q_target = tgt;
        self.push_op(op);
    }
    pub(crate) fn cz(&mut self, a: QubitId, b: QubitId) {
        if a == b {
            self.z(a);
            return;
        }
        let mut op = Op::empty();
        op.kind = OperationType::CZ;
        op.q_control1 = a;
        op.q_target = b;
        self.push_op(op);
    }
    pub(crate) fn ccx(&mut self, c1: QubitId, c2: QubitId, tgt: QubitId) {
        if c1 == c2 {
            if c1 != tgt {
                self.cx(c1, tgt);
            }
            return;
        }
        if c1 == tgt || c2 == tgt {
            panic!(
                "invalid CCX with target aliased to a control: {:?}, {:?}, {:?}",
                c1, c2, tgt
            );
        }
        let mut op = Op::empty();
        op.kind = OperationType::CCX;
        op.q_control2 = c1;
        op.q_control1 = c2;
        op.q_target = tgt;
        self.push_op(op);
    }
    pub(crate) fn ccz(&mut self, c1: QubitId, c2: QubitId, tgt: QubitId) {
        if c1 == c2 {
            if c1 != tgt {
                self.cz(c1, tgt);
            } else {
                self.z(c1);
            }
            return;
        }
        if c1 == tgt {
            self.cz(c1, c2);
            return;
        }
        if c2 == tgt {
            self.cz(c1, c2);
            return;
        }
        let mut op = Op::empty();
        op.kind = OperationType::CCZ;
        op.q_control2 = c1;
        op.q_control1 = c2;
        op.q_target = tgt;
        self.push_op(op);
    }
    pub(crate) fn swap(&mut self, a: QubitId, b: QubitId) {
        if a == b {
            return;
        }
        let mut op = Op::empty();
        op.kind = OperationType::Swap;
        op.q_control1 = a;
        op.q_target = b;
        self.push_op(op);
    }
    pub(crate) fn r(&mut self, q: QubitId) {
        let mut op = Op::empty();
        op.kind = OperationType::R;
        op.q_target = q;
        self.push_op(op);
    }
    pub(crate) fn x_if(&mut self, q: QubitId, cond: BitId) {
        let mut op = Op::empty();
        op.kind = OperationType::X;
        op.q_target = q;
        op.c_condition = cond;
        self.push_op(op);
    }
    // ── Measurement / phase / classical bit ops ──
    pub(crate) fn hmr(&mut self, q: QubitId, c: BitId) {
        let mut op = Op::empty();
        op.kind = OperationType::Hmr;
        op.q_target = q;
        op.c_target = c;
        self.push_op(op);
    }
    pub(crate) fn neg(&mut self) {
        let mut op = Op::empty();
        op.kind = OperationType::Neg;
        self.push_op(op);
    }
    // ── Classically-conditioned variants for all remaining gates ──
    pub(crate) fn z_if(&mut self, q: QubitId, cond: BitId) {
        let mut op = Op::empty();
        op.kind = OperationType::Z;
        op.q_target = q;
        op.c_condition = cond;
        self.push_op(op);
    }
    pub(crate) fn cz_if(&mut self, a: QubitId, b: QubitId, cond: BitId) {
        if a == b {
            self.z_if(a, cond);
            return;
        }
        let mut op = Op::empty();
        op.kind = OperationType::CZ;
        op.q_control1 = a;
        op.q_target = b;
        op.c_condition = cond;
        self.push_op(op);
    }
    pub(crate) fn ccz_if(&mut self, c1: QubitId, c2: QubitId, tgt: QubitId, cond: BitId) {
        if c1 == c2 {
            if c1 != tgt {
                self.cz_if(c1, tgt, cond);
            } else {
                self.z_if(c1, cond);
            }
            return;
        }
        if c1 == tgt {
            self.cz_if(c1, c2, cond);
            return;
        }
        if c2 == tgt {
            self.cz_if(c1, c2, cond);
            return;
        }
        let mut op = Op::empty();
        op.kind = OperationType::CCZ;
        op.q_control2 = c1;
        op.q_control1 = c2;
        op.q_target = tgt;
        op.c_condition = cond;
        self.push_op(op);
    }
}
