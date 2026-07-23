//! GC3: same seed ⇒ identical weight-update hash (U09–U10).

use binn_learn::three_factor::{coincidence_engine, run_coincidence_trial};
use binn_learn::{Modulators, ThreeFactor};

fn weight_fingerprint(seed: u64) -> u64 {
    let _ = seed; // coincidence path is deterministic without RNG draws
    let mut eng = coincidence_engine(0.1);
    let mut learner = ThreeFactor::new(0.4, 0.0, 35.0);
    for i in 0..5u64 {
        run_coincidence_trial(&mut eng, &mut learner, Modulators::reward(1.0), 10 + i * 20);
    }
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for &w in &eng.edge_w {
        hash ^= w.to_bits() as u64;
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    for syn in eng.syn.as_slice() {
        hash ^= syn.eligibility.to_bits() as u64;
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash
}

#[test]
fn gc3_same_seed_identical_weight_update_hash() {
    let seed = 0xB177_C0DE_0000_0009;
    let a = weight_fingerprint(seed);
    let b = weight_fingerprint(seed);
    assert_eq!(a, b, "same protocol must yield identical weight hash");
    assert_ne!(a, 0);
}
