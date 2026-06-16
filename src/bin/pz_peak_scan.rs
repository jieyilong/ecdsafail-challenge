use quantum_ecc::circuit::{
    BitId, Op, OperationType, QubitId, QubitOrBit, RegisterId, NO_BIT, NO_QUBIT,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;

const MAGIC: &[u8; 8] = b"QECCOPS1";
const OP_BYTES: usize = 56;

#[derive(Clone, Copy, Debug)]
struct Segment {
    q: u64,
    start: u64,
    end: u64,
    register: bool,
}

#[derive(Default)]
struct State {
    start: Option<u64>,
    register: bool,
}

fn read_u64(bytes: &[u8], off: usize) -> u64 {
    u64::from_le_bytes(bytes[off..off + 8].try_into().unwrap())
}

fn kind(v: u32) -> OperationType {
    match v {
        0 => OperationType::Neg,
        1 => OperationType::Register,
        2 => OperationType::AppendToRegister,
        3 => OperationType::BitInvert,
        4 => OperationType::BitStore0,
        5 => OperationType::BitStore1,
        6 => OperationType::X,
        7 => OperationType::Z,
        8 => OperationType::CX,
        9 => OperationType::CZ,
        10 => OperationType::Swap,
        11 => OperationType::R,
        12 => OperationType::Hmr,
        13 => OperationType::CCX,
        14 => OperationType::CCZ,
        15 => OperationType::PushCondition,
        16 => OperationType::PopCondition,
        17 => OperationType::DebugPrint,
        _ => panic!("bad op kind {v}"),
    }
}

fn parse_op(bytes: &[u8], index: usize) -> Op {
    let off = MAGIC.len() + 8 + index * OP_BYTES;
    Op {
        kind: kind(u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap())),
        q_control2: QubitId(read_u64(bytes, off + 8)),
        q_control1: QubitId(read_u64(bytes, off + 16)),
        q_target: QubitId(read_u64(bytes, off + 24)),
        c_target: BitId(read_u64(bytes, off + 32)),
        c_condition: BitId(read_u64(bytes, off + 40)),
        r_target: RegisterId(read_u64(bytes, off + 48)),
    }
}

fn touch(states: &mut [State], q: QubitId, t: u64) {
    if q == NO_QUBIT {
        return;
    }
    let state = &mut states[q.0 as usize];
    if state.start.is_none() {
        state.start = Some(t);
    }
}

fn close(states: &mut [State], segs: &mut Vec<Segment>, q: QubitId, t: u64) {
    if q == NO_QUBIT {
        return;
    }
    let state = &mut states[q.0 as usize];
    if let Some(start) = state.start.take() {
        segs.push(Segment {
            q: q.0,
            start,
            end: t,
            register: state.register,
        });
    }
}

fn q(q: QubitId) -> String {
    if q == NO_QUBIT {
        "-".to_string()
    } else {
        format!("q{}", q.0)
    }
}

fn b(b: BitId) -> String {
    if b == NO_BIT {
        "-".to_string()
    } else {
        format!("b{}", b.0)
    }
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| "ops.bin".into());
    let bytes = fs::read(path).expect("read ops");
    assert_eq!(&bytes[..MAGIC.len()], MAGIC);
    let n = u64::from_le_bytes(bytes[MAGIC.len()..MAGIC.len() + 8].try_into().unwrap());

    let mut max_q = 0u64;
    let mut registers: BTreeMap<u64, Vec<QubitOrBit>> = BTreeMap::new();
    let mut ops = Vec::with_capacity(n as usize);
    for i in 0..n as usize {
        let op = parse_op(&bytes, i);
        for q in [op.q_control2, op.q_control1, op.q_target] {
            if q != NO_QUBIT {
                max_q = max_q.max(q.0);
            }
        }
        if op.kind == OperationType::AppendToRegister {
            let entry = registers.entry(op.r_target.0).or_default();
            if op.q_target != NO_QUBIT {
                entry.push(QubitOrBit::Qubit(op.q_target));
            } else {
                entry.push(QubitOrBit::Bit(op.c_target));
            }
        }
        ops.push(op);
    }

    let mut states = (0..=max_q).map(|_| State::default()).collect::<Vec<_>>();
    let mut register_qs = BTreeSet::new();
    for reg in [0u64, 1u64] {
        if let Some(items) = registers.get(&reg) {
            for item in items {
                if let QubitOrBit::Qubit(q) = *item {
                    register_qs.insert(q.0);
                    states[q.0 as usize].register = true;
                    states[q.0 as usize].start = Some(0);
                }
            }
        }
    }

    let mut segs = Vec::new();
    for (i, op) in ops.iter().enumerate() {
        let t = i as u64;
        match op.kind {
            OperationType::Register
            | OperationType::AppendToRegister
            | OperationType::BitInvert
            | OperationType::BitStore0
            | OperationType::BitStore1
            | OperationType::PushCondition
            | OperationType::PopCondition
            | OperationType::DebugPrint
            | OperationType::Neg => {}
            _ => {
                touch(&mut states, op.q_control2, t);
                touch(&mut states, op.q_control1, t);
                touch(&mut states, op.q_target, t);
            }
        }
        if matches!(op.kind, OperationType::R | OperationType::Hmr)
            && op.q_target != NO_QUBIT
            && op.c_condition == NO_BIT
            && !register_qs.contains(&op.q_target.0)
        {
            close(&mut states, &mut segs, op.q_target, t);
        }
    }
    for (q, state) in states.iter_mut().enumerate() {
        if let Some(start) = state.start.take() {
            segs.push(Segment {
                q: q as u64,
                start,
                end: n,
                register: state.register,
            });
        }
    }

    let mut events = BTreeMap::<u64, i64>::new();
    for s in &segs {
        *events.entry(s.start).or_default() += 1;
        *events.entry(s.end.saturating_add(1)).or_default() -= 1;
    }
    let mut live = 0i64;
    let mut peak = 0i64;
    let mut plateaus = Vec::<(u64, u64)>::new();
    let mut prev_t = None::<u64>;
    for (&t, &delta) in &events {
        if let Some(prev) = prev_t {
            if prev < t && live == peak && peak > 0 {
                plateaus.push((prev, t - 1));
            }
        }
        live += delta;
        if live > peak {
            peak = live;
            plateaus.clear();
        }
        prev_t = Some(t);
    }

    let total_plateau_ops: u64 = plateaus.iter().map(|(a, b)| b - a + 1).sum();
    println!("ops={n} named_qubits={} peak={peak}", max_q + 1);
    println!(
        "plateau_intervals={} plateau_ops={total_plateau_ops}",
        plateaus.len()
    );
    let mut family_counts = BTreeMap::<String, (u64, u64)>::new();
    let mut live_high_counts = BTreeMap::<u64, u64>::new();
    let mut op_target_counts = BTreeMap::<String, u64>::new();
    for &(start, end) in &plateaus {
        let op = &ops[start as usize];
        let family = format!(
            "{:?} {} {} {}",
            op.kind,
            q(op.q_control2),
            q(op.q_control1),
            q(op.q_target)
        );
        let entry = family_counts.entry(family).or_default();
        entry.0 += 1;
        entry.1 += end - start + 1;
        *op_target_counts.entry(q(op.q_target)).or_default() += 1;

        for s in segs.iter().filter(|s| s.start <= start && start <= s.end) {
            if s.q >= 1024 {
                *live_high_counts.entry(s.q).or_default() += 1;
            }
        }
    }

    println!("top plateau op families:");
    let mut families = family_counts.into_iter().collect::<Vec<_>>();
    families.sort_by_key(|(_, (intervals, ops))| (std::cmp::Reverse(*ops), std::cmp::Reverse(*intervals)));
    for (family, (intervals, ops)) in families.into_iter().take(16) {
        println!("  {ops:6} ops {intervals:4} intervals  {family}");
    }

    println!("top high-qubit live-at-plateau counts:");
    let mut live_counts = live_high_counts.into_iter().collect::<Vec<_>>();
    live_counts.sort_by_key(|(qubit, count)| (std::cmp::Reverse(*count), std::cmp::Reverse(*qubit)));
    for (qubit, count) in live_counts.into_iter().take(24) {
        println!("  q{qubit}: {count} intervals");
    }

    println!("top plateau op targets:");
    let mut targets = op_target_counts.into_iter().collect::<Vec<_>>();
    targets.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    for (target, count) in targets.into_iter().take(12) {
        println!("  {target}: {count} intervals");
    }

    for (rank, &(start, end)) in plateaus.iter().take(24).enumerate() {
        let op = &ops[start as usize];
        println!(
            "plateau[{rank}] {start}..{end} len={} op={:?} {} {} {} c={} cond={}",
            end - start + 1,
            op.kind,
            q(op.q_control2),
            q(op.q_control1),
            q(op.q_target),
            b(op.c_target),
            b(op.c_condition)
        );
        let mut live_segs = segs
            .iter()
            .copied()
            .filter(|s| s.start <= start && start <= s.end)
            .collect::<Vec<_>>();
        live_segs.sort_by_key(|s| {
            let left = start.saturating_sub(s.start);
            let right = s.end.saturating_sub(start);
            (right.min(left), right, left, s.q)
        });
        for s in live_segs.iter().take(12) {
            println!(
                "  live q{} [{}..{}] left={} right={} len={} reg={}",
                s.q,
                s.start,
                s.end,
                start.saturating_sub(s.start),
                s.end.saturating_sub(start),
                s.end.saturating_sub(s.start),
                s.register
            );
        }
    }
}
