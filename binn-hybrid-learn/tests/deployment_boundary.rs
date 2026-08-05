#[test]
fn runtime_manifest_cannot_depend_on_training_lab() {
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("runtime manifest");
    assert!(
        !manifest.contains("binn-hybrid-lab"),
        "deployment runtime must not depend on the teacher-bearing lab crate"
    );
}
