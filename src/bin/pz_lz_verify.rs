//! Verify the pz1050 LZSS blob round-trips bit-exactly (dev tool, not submitted).
//! usage: pz_lz_verify <blob.lz> <original.kmx>
use std::fs;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let blob = fs::read(&a[1]).expect("read blob");
    let orig = fs::read(&a[2]).expect("read orig");
    let dec = quantum_ecc::point_add::pz1050_lz_decode(&blob);
    println!("decoded {} bytes, original {} bytes", dec.len(), orig.len());
    if dec == orig {
        println!("ROUND-TRIP OK (bit-exact)");
    } else {
        let mismatch = dec
            .iter()
            .zip(orig.iter())
            .position(|(x, y)| x != y)
            .unwrap_or(dec.len().min(orig.len()));
        println!("ROUND-TRIP MISMATCH at byte {mismatch}");
        std::process::exit(1);
    }
}
