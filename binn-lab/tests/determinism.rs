//! GC3: same seed / config ⇒ identical C1 fingerprint.
//! C2 hash is a separate protocol (must not alias the C1 kill-gate).

use binn_lab::{C2Config, Config, Runner};

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

#[test]
fn c2_hash_does_not_alias_c1_kill_gate() {
    let c1 = Config::c1_default();
    let c2 = C2Config::c2_default();
    let c2q = C2Config::c2_quick();
    assert_eq!(c1.hash_string(), "c1-118207fbc3eaba53");
    assert!(c2.hash_string().starts_with("c2-"));
    assert!(c2q.hash_string().starts_with("c2-"));
    assert_ne!(c2.hash_string(), c1.hash_string());
    assert_ne!(c2q.hash_string(), c1.hash_string());
    assert_ne!(c2.hash(), c2q.hash());
}
