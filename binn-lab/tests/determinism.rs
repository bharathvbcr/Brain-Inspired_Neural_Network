//! GC3: same seed / config ⇒ identical C1 fingerprint.

use binn_lab::{Config, Runner};

#[test]
fn c1_same_config_same_hash() {
    let a = Config::c1_quick();
    let b = Config::c1_quick();
    assert_eq!(a.hash(), b.hash());
    assert_eq!(a.hash_string(), b.hash_string());
}

#[test]
fn c1_same_seed_identical_seed_accuracies() {
    let cfg = Config::c1_quick();
    let mut r1 = Runner::new();
    let mut r2 = Runner::new();
    let a = r1.run_c1(&cfg);
    let b = r2.run_c1(&cfg);
    assert_eq!(a.config_hash, b.config_hash);
    assert_eq!(a.seeds.len(), b.seeds.len());
    for (x, y) in a.seeds.iter().zip(b.seeds.iter()) {
        assert_eq!(x.seed, y.seed);
        assert_eq!(x.local_assembly, y.local_assembly);
        assert_eq!(x.dense_local, y.dense_local);
        assert_eq!(x.gradient_reference, y.gradient_reference);
    }
    assert_eq!(a.verdict, b.verdict);
}
