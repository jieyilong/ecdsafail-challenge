//! Compact binary tokenizer for the shrunken-PZ kmx (dev tool, not submitted).
//! Reuses Op::from_text, emits a tight per-kind binary so LZSS compresses far
//! better than on the ASCII text (and build() can decode straight to Vec<Op>,
//! skipping the 1.4 GB text intermediate). usage: pz_bin_encode <kmx> <out.bin>
use quantum_ecc::circuit::{Op, OperationType, NO_BIT, NO_QUBIT, NO_REG};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

fn put_varint(out: &mut Vec<u8>, mut v: u64) {
    loop {
        let b = (v & 0x7f) as u8;
        v >>= 7;
        if v != 0 {
            out.push(b | 0x80);
        } else {
            out.push(b);
            break;
        }
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let inf = BufReader::new(File::open(&a[1]).expect("open kmx"));
    let mut out = BufWriter::new(File::create(&a[2]).expect("create out"));
    let mut buf: Vec<u8> = Vec::with_capacity(1 << 20);
    let mut n: u64 = 0;
    for line in inf.lines() {
        let line = line.unwrap();
        let Some(op) = Op::from_text(&line) else { continue };
        let kind = op.kind as u8; // 0..=17
        let is_push = matches!(op.kind, OperationType::PushCondition);
        let has_cond = !is_push && op.c_condition != NO_BIT;
        let append_is_bit =
            op.kind == OperationType::AppendToRegister && op.c_target != NO_BIT;
        let flags = kind | ((has_cond as u8) << 5) | ((append_is_bit as u8) << 6);
        buf.push(flags);
        match op.kind {
            OperationType::Neg | OperationType::PopCondition | OperationType::DebugPrint => {}
            OperationType::Register => put_varint(&mut buf, op.r_target.0),
            OperationType::AppendToRegister => {
                if append_is_bit {
                    put_varint(&mut buf, op.c_target.0);
                } else {
                    put_varint(&mut buf, op.q_target.0);
                }
                put_varint(&mut buf, op.r_target.0);
            }
            OperationType::BitInvert | OperationType::BitStore0 | OperationType::BitStore1 => {
                put_varint(&mut buf, op.c_target.0)
            }
            OperationType::X | OperationType::Z | OperationType::R => {
                put_varint(&mut buf, op.q_target.0)
            }
            OperationType::CX | OperationType::CZ | OperationType::Swap => {
                put_varint(&mut buf, op.q_control1.0);
                put_varint(&mut buf, op.q_target.0);
            }
            OperationType::Hmr => {
                put_varint(&mut buf, op.q_target.0);
                put_varint(&mut buf, op.c_target.0);
            }
            OperationType::CCX | OperationType::CCZ => {
                put_varint(&mut buf, op.q_control2.0);
                put_varint(&mut buf, op.q_control1.0);
                put_varint(&mut buf, op.q_target.0);
            }
            OperationType::PushCondition => put_varint(&mut buf, op.c_condition.0),
        }
        if has_cond {
            put_varint(&mut buf, op.c_condition.0);
        }
        n += 1;
        if buf.len() > (1 << 20) {
            out.write_all(&buf).unwrap();
            buf.clear();
        }
    }
    out.write_all(&buf).unwrap();
    out.flush().unwrap();
    let _ = (NO_QUBIT, NO_REG); // silence unused import if schema changes
    eprintln!("encoded {n} ops -> {}", a[2]);
}
