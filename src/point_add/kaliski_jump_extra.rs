//! Extra classical experiments for the hybrid Kaliski-jump moonshot.
//!
//! Goal: shrink the per-class ambiguity of local transition classes by adding
//! a little more side information than just `(u mod 2^w, v mod 2^w)`.
//!
//! The most promising extra information found so far is:
//! - `cmp0 = (u > v)` at the start of the window,
//! - `cmp1 = (u1 > v1)` after the first micro-step,
//! - and optionally the low bits of `(u1, v1)`.
//!
//! This file quantifies how much that extra info collapses the transition
//! ambiguity.

use std::collections::{BTreeMap, BTreeSet};

use alloy_primitives::U256;
use sha3::digest::{ExtendableOutput, Update, XofReader};

use super::SECP256K1_P;
use super::kaliski_jump::{observe_window, kaliski_step_uv};

pub struct Sampler {
    reader: Box<dyn XofReader>,
    p: U256,
}

impl Sampler {
    pub fn new(seed: &[u8], p: U256) -> Self {
        let mut hasher = sha3::Shake128::default();
        hasher.update(seed);
        Self { reader: Box::new(hasher.finalize_xof()), p }
    }

    pub fn next(&mut self) -> U256 {
        loop {
            let mut buf = [0u8; 32];
            self.reader.read(&mut buf);
            let x = U256::from_le_slice(&buf);
            if x < self.p && !x.is_zero() {
                return x;
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct AmbiguityStats {
    pub windows: usize,
    pub classes: usize,
    pub mean_mats_per_class: f64,
    pub max_mats_per_class: usize,
    pub singleton_classes: usize,
}

fn summarize<K: Ord>(map: &BTreeMap<K, BTreeSet<Vec<u8>>>) -> AmbiguityStats {
    let classes = map.len();
    let mut total = 0usize;
    let mut maxc = 0usize;
    let mut singletons = 0usize;
    for mats in map.values() {
        let c = mats.len();
        total += c;
        if c > maxc { maxc = c; }
        if c == 1 { singletons += 1; }
    }
    AmbiguityStats {
        windows: 0,
        classes,
        mean_mats_per_class: if classes == 0 { 0.0 } else { total as f64 / classes as f64 },
        max_mats_per_class: maxc,
        singleton_classes: singletons,
    }
}

/// Encode the branch sequence compactly (UE, VE, UG, VG -> 0..3) so we can
/// use it as the class label when counting ambiguity.
fn encode_cases(cases: &[super::kaliski_jump::KCase]) -> Vec<u8> {
    cases.iter().map(|c| match c {
        super::kaliski_jump::KCase::UEven => 0,
        super::kaliski_jump::KCase::VEven => 1,
        super::kaliski_jump::KCase::UGtV  => 2,
        super::kaliski_jump::KCase::VGtU  => 3,
    }).collect()
}

/// Survey ambiguity of t-step windows under several candidate lookup keys.
///
/// Keys examined:
/// - `low`: low w bits of (u, v)
/// - `low_cmp0`: low bits + initial compare bit
/// - `low_cmp0_cmp1`: low bits + initial compare bit + compare bit after first step
/// - `low_cmp0_cmp1_low1`: additionally low w bits of (u1, v1)
pub fn ambiguity_survey(
    seed: &[u8],
    n_inputs: usize,
    w: usize,
    t: usize,
) -> (AmbiguityStats, AmbiguityStats, AmbiguityStats, AmbiguityStats) {
    let mut sampler = Sampler::new(seed, SECP256K1_P);
    let mut low: BTreeMap<(u16,u16), BTreeSet<Vec<u8>>> = BTreeMap::new();
    let mut low_cmp0: BTreeMap<(u16,u16,bool), BTreeSet<Vec<u8>>> = BTreeMap::new();
    let mut low_cmp0_cmp1: BTreeMap<(u16,u16,bool,bool), BTreeSet<Vec<u8>>> = BTreeMap::new();
    let mut low_cmp0_cmp1_low1: BTreeMap<(u16,u16,bool,bool,u16,u16), BTreeSet<Vec<u8>>> = BTreeMap::new();
    let mask = if w == 16 { U256::from(0xFFFFu64) } else { (U256::from(1u64) << w).wrapping_sub(U256::from(1u64)) };

    let mut total_windows = 0usize;
    for _ in 0..n_inputs {
        let mut u = SECP256K1_P;
        let mut v = sampler.next();
        for _ in 0..742 {
            if v.is_zero() { break; }
            let (_nu, _nv, obs) = observe_window(u, v, w, t);
            let seq = encode_cases(&obs.cases);
            let cmp0 = u > v;
            let (u1, v1, _kc) = kaliski_step_uv(u, v);
            let cmp1 = u1 > v1;
            let low_u1 = u16::try_from((u1 & mask).to::<u64>()).unwrap();
            let low_v1 = u16::try_from((v1 & mask).to::<u64>()).unwrap();

            low.entry((obs.low_u, obs.low_v)).or_default().insert(seq.clone());
            low_cmp0.entry((obs.low_u, obs.low_v, cmp0)).or_default().insert(seq.clone());
            low_cmp0_cmp1.entry((obs.low_u, obs.low_v, cmp0, cmp1)).or_default().insert(seq.clone());
            low_cmp0_cmp1_low1.entry((obs.low_u, obs.low_v, cmp0, cmp1, low_u1, low_v1)).or_default().insert(seq.clone());

            total_windows += 1;
            u = u1;
            v = v1;
        }
    }

    let mut a = summarize(&low);
    let mut b = summarize(&low_cmp0);
    let mut c = summarize(&low_cmp0_cmp1);
    let mut d = summarize(&low_cmp0_cmp1_low1);
    a.windows = total_windows;
    b.windows = total_windows;
    c.windows = total_windows;
    d.windows = total_windows;
    (a, b, c, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ambiguity_survey_test() {
        let (a, b, c, d) = ambiguity_survey(b"kaliski-ambiguity-seed-v1", 10_000, 8, 4);
        eprintln!("=== Kaliski ambiguity survey (w=8, t=4) ===");
        eprintln!("windows               : {}", a.windows);
        eprintln!("low                   : mean {:.3}, max {}, singletons {}", a.mean_mats_per_class, a.max_mats_per_class, a.singleton_classes);
        eprintln!("low+cmp0              : mean {:.3}, max {}, singletons {}", b.mean_mats_per_class, b.max_mats_per_class, b.singleton_classes);
        eprintln!("low+cmp0+cmp1         : mean {:.3}, max {}, singletons {}", c.mean_mats_per_class, c.max_mats_per_class, c.singleton_classes);
        eprintln!("low+cmp0+cmp1+low1    : mean {:.3}, max {}, singletons {}", d.mean_mats_per_class, d.max_mats_per_class, d.singleton_classes);
        eprintln!("===========================================");
        // Each refinement should never make ambiguity worse.
        assert!(b.mean_mats_per_class <= a.mean_mats_per_class + 1e-9);
        assert!(c.mean_mats_per_class <= b.mean_mats_per_class + 1e-9);
        assert!(d.mean_mats_per_class <= c.mean_mats_per_class + 1e-9);
    }
}
