//! Translate a TrailMix `.kmx` text op-stream into the challenge `ops.bin`
//! binary format. Streams line-by-line so a ~100M-op circuit doesn't need to
//! be held in RAM. Reuses the trusted `Op::from_text` parser + validate().
//!
//! usage: kmx_to_ops <in.kmx> [out.bin=ops.bin]

use quantum_ecc::circuit::Op;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};

const MAGIC: &[u8; 8] = b"QECCOPS1";

fn main() {
    let kmx = std::env::args().nth(1).expect("usage: kmx_to_ops <in.kmx> [out.bin]");
    let outp = std::env::args().nth(2).unwrap_or_else(|| "ops.bin".to_string());

    let inf = BufReader::new(File::open(&kmx).expect("open kmx"));
    let mut out = BufWriter::new(File::create(&outp).expect("create out"));

    out.write_all(MAGIC).unwrap();
    out.write_all(&0u64.to_le_bytes()).unwrap(); // placeholder op count

    let mut count: u64 = 0;
    let mut max_q: u64 = 0;
    let mut max_b: u64 = 0;
    for line in inf.lines() {
        let line = line.unwrap();
        if let Some(op) = Op::from_text(&line) {
            for id in [op.q_control2.0, op.q_control1.0, op.q_target.0] {
                if id != u64::MAX && id > max_q {
                    max_q = id;
                }
            }
            for id in [op.c_target.0, op.c_condition.0] {
                if id != u64::MAX && id > max_b {
                    max_b = id;
                }
            }
            out.write_all(&(op.kind as u32).to_le_bytes()).unwrap();
            out.write_all(&[0u8; 4]).unwrap();
            out.write_all(&op.q_control2.0.to_le_bytes()).unwrap();
            out.write_all(&op.q_control1.0.to_le_bytes()).unwrap();
            out.write_all(&op.q_target.0.to_le_bytes()).unwrap();
            out.write_all(&op.c_target.0.to_le_bytes()).unwrap();
            out.write_all(&op.c_condition.0.to_le_bytes()).unwrap();
            out.write_all(&op.r_target.0.to_le_bytes()).unwrap();
            count += 1;
        }
    }
    out.flush().unwrap();
    let mut f = out.into_inner().unwrap();
    f.seek(SeekFrom::Start(MAGIC.len() as u64)).unwrap();
    f.write_all(&count.to_le_bytes()).unwrap();
    f.flush().unwrap();

    eprintln!(
        "wrote {count} ops -> {outp}  (max_qubit_id={}, => num_qubits={}; max_bit_id={})",
        max_q,
        max_q + 1,
        max_b
    );
}
