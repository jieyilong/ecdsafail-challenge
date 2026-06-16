//! Offline LZSS encoder for the shrunken-PZ kmx (dev tool, NOT submitted).
//! Produces a blob decodable by the pure-std decoder in src/point_add/pz1050.
//! Token stream: repeat { varint lit_len; lit_len raw bytes; varint match_len;
//! if match_len>0 { varint dist } }. match_len is stored as (len - MIN_MATCH).
//! usage: pz_lz_encode <in> <out>

use std::fs;
use std::io::Write;

const MIN_MATCH: usize = 4;
const WINDOW: usize = 1 << 24; // 16 MiB back-reference window
const MAX_CHAIN: usize = 128; // match-search effort
const HASH_BITS: u32 = 21;

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

#[inline]
fn hash4(d: &[u8], i: usize) -> usize {
    let x = u32::from_le_bytes([d[i], d[i + 1], d[i + 2], d[i + 3]]);
    (x.wrapping_mul(2654435761) >> (32 - HASH_BITS)) as usize
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let inp = fs::read(&args[1]).expect("read in");
    let n = inp.len();
    eprintln!("input {} bytes", n);

    let mut head = vec![-1i32; 1 << HASH_BITS];
    let mut prev = vec![-1i32; n.max(1)];
    let mut out: Vec<u8> = Vec::with_capacity(n / 8);

    let mut pos = 0usize;
    let mut lit_start = 0usize;
    let last = if n >= MIN_MATCH { n - MIN_MATCH } else { 0 };

    while pos < n {
        let mut best_len = 0usize;
        let mut best_dist = 0usize;
        if pos <= last {
            let h = hash4(&inp, pos);
            let mut cand = head[h];
            let mut chain = 0;
            let limit = pos.saturating_sub(WINDOW) as i64;
            while cand >= 0 && (cand as i64) >= limit && chain < MAX_CHAIN {
                let c = cand as usize;
                // extend match
                let max_len = n - pos;
                let mut l = 0usize;
                while l < max_len && inp[c + l] == inp[pos + l] {
                    l += 1;
                }
                if l > best_len {
                    best_len = l;
                    best_dist = pos - c;
                    if l >= max_len {
                        break;
                    }
                }
                cand = prev[c];
                chain += 1;
            }
        }

        if best_len >= MIN_MATCH {
            // flush pending literals + this match
            let lit_len = pos - lit_start;
            put_varint(&mut out, lit_len as u64);
            out.extend_from_slice(&inp[lit_start..pos]);
            put_varint(&mut out, (best_len - MIN_MATCH) as u64 + 1); // +1 so 0 reserved for "no match"
            put_varint(&mut out, best_dist as u64);
            // insert hashes for the matched span
            let end = (pos + best_len).min(last + 1);
            let mut q = pos;
            while q < end {
                let h = hash4(&inp, q);
                prev[q] = head[h];
                head[h] = q as i32;
                q += 1;
            }
            pos += best_len;
            lit_start = pos;
        } else {
            if pos <= last {
                let h = hash4(&inp, pos);
                prev[pos] = head[h];
                head[h] = pos as i32;
            }
            pos += 1;
        }
    }
    // trailing literals
    let lit_len = n - lit_start;
    put_varint(&mut out, lit_len as u64);
    out.extend_from_slice(&inp[lit_start..n]);
    put_varint(&mut out, 0); // no match -> terminates

    let mut f = fs::File::create(&args[2]).expect("create out");
    f.write_all(&out).unwrap();
    eprintln!(
        "wrote {} bytes -> {} ({:.1}x)",
        out.len(),
        args[2],
        n as f64 / out.len() as f64
    );
}
