//! The cell's timestamp must never enter a bit-identity comparison.
//!
//! `emitted_unix_s` and `emitted_utc` differ on every run by construction. Gate
//! F re-runs a recorded cell and demands every compared field match **exactly**,
//! so adding either to its list would turn every regression red for a reason
//! that has nothing to do with the kernel — and, worse, would train whoever saw
//! it to widen the comparison.
//!
//! Gate F compares an explicit list, so the fields are excluded by construction
//! rather than by remembering. This pins that, because "excluded by
//! construction" is exactly the kind of property that survives until somebody
//! adds a field to a list.

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

/// The two fields the instrument now writes and no comparison may read.
const TIMESTAMP_FIELDS: [&str; 2] = ["emitted_unix_s", "emitted_utc"];

#[test]
fn gate_f_does_not_compare_the_timestamp() {
    let gate = std::fs::read_to_string(repo_root().join("scripts/gate_f_rust.py"))
        .expect("read gate_f_rust.py");

    // The lists are what the comparison actually walks. Read them rather than
    // the whole file, or a mention in a comment would satisfy this test.
    let lists: String = ["COMPARED_FIELDS", "COMPARED_TRACES"]
        .iter()
        .map(|name| {
            let start = gate.find(name).unwrap_or_else(|| {
                panic!("{name} is gone from gate_f_rust.py; this test is not checking anything")
            });
            let end = gate[start..]
                .find(')')
                .map(|i| start + i)
                .unwrap_or(gate.len());
            gate[start..end].to_string()
        })
        .collect();

    for field in TIMESTAMP_FIELDS {
        assert!(
            !lists.contains(field),
            "gate_f_rust.py compares `{field}`, which changes on every run. Gate F \
             would fail for a reason that is not the kernel, and the fix would look \
             like widening the comparison."
        );
    }
}

#[test]
fn the_instrument_writes_both_fields() {
    let source =
        std::fs::read_to_string(repo_root().join("binn-lab/experiments/shd_instrument.rs"))
            .expect("read shd_instrument.rs");
    for field in TIMESTAMP_FIELDS {
        assert!(
            source.contains(field),
            "the instrument no longer emits `{field}`; a cell would go back to \
             carrying only a duration and no instant"
        );
    }
    // Both must describe the SAME instant. The instrument binds `unix_seconds()`
    // once and renders that binding, rather than calling the clock twice, which
    // could straddle a second boundary and put the two fields a second apart.
    assert!(
        source.contains("let emitted = unix_seconds();") && source.contains("iso8601_utc(emitted)"),
        "the two timestamp fields must be rendered from one reading of the clock"
    );
}

#[test]
fn the_validity_gate_ignores_them() {
    // A cell's validity is about what it measured, not when. If the gate ever
    // required or rejected on these, every cell recorded before today would
    // become invalid — a re-scoring of the whole record by accident.
    let gate = std::fs::read_to_string(repo_root().join("scripts/cell_validity.py"))
        .expect("read cell_validity.py");
    for field in TIMESTAMP_FIELDS {
        assert!(
            !gate.contains(field),
            "cell_validity.py reads `{field}`; every cell recorded before it \
             existed would be judged on a field it cannot have"
        );
    }
}
