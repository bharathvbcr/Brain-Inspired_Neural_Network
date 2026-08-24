//! An experiment binary may not re-declare a helper `binn_lab` already exports.
//!
//! # Why
//!
//! Sixteen copies of `mean` once existed across the experiment binaries. All
//! sixteen agreed on every non-empty input and disagreed on the empty one —
//! nine returned `0.0`, five NaN, one panicked — and `--seeds 0` reaches that
//! case, so which copy a binary happened to carry decided whether a report said
//! `0.0000` or `NaN` for an accuracy computed from nothing.
//!
//! The consolidation that fixed it does not stay fixed on its own. A day after
//! the copies were removed, `credit_depth_scaling.rs` had two of them back —
//! `mean` and `std_error`, byte-identical in semantics to the canonical pair,
//! in a file that already imported from `binn_lab`. Nothing objected, because
//! nothing was looking. Each new experiment binary is another chance to paste
//! them in.
//!
//! # What this checks, and what it deliberately does not
//!
//! It fails on a **name collision** with a `binn_lab` export — not on textual
//! similarity. Comparing bodies would miss a copy whose arithmetic was spelled
//! differently while computing the same thing, and `credit_depth_scaling.rs`
//! was exactly that: its `std_error` divided by `(len() - 1) as f32` where the
//! canonical one divides by `len() as f32 - 1.0`, identical for every seed
//! count this repository uses and textually distinct.
//!
//! Colliding is therefore not automatically wrong — it means *state why*. Two
//! local wrappers are allowed below, each because it deliberately differs from
//! the canonical function on the empty input, which is the only place these
//! functions have ever disagreed.
//!
//! It cannot see a duplicate under a **different name**. `mean_or_nan` is a
//! separate function from `mean` on purpose, and the three `sigmoid` variants
//! are three distinct numerics that must not be merged; `binn_lab` exports
//! neither `sigmoid` nor a second `mean`, so neither is reachable by this check
//! — and that is the correct outcome, not an oversight.
//!
//! Be precise about which of those two is out of scope, because it is easy to
//! over-claim here. **`sigmoid` is structurally out of scope**: `binn_lab` does
//! not export it, so no local `sigmoid` can ever collide. **`mean_or_nan` is
//! not** — it is exported from `lib.rs` and imported by six experiments, so a
//! local re-declaration of that name would be caught, which is right. And the
//! two deliberate local `mean` wrappers are likewise **in scope and
//! allow-listed**, not exempt by pattern. That is the stronger arrangement: a
//! future edit to either has to walk past a written justification rather than
//! silently fail to match.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Local declarations that collide with a `binn_lab` export **on purpose**.
///
/// Each entry is `(file, function, why)`. The reason is the point of the entry:
/// an allow-list without one is just a way to stop a check from talking.
const ALLOWED: &[(&str, &str, &str)] = &[
    (
        "shortcut_accessibility_contrast.rs",
        "mean",
        "asserts non-empty and takes an iterator; binn_lab::mean returns 0.0 for \
         an empty slice. The wrapper keeps the assert and delegates the arithmetic.",
    ),
    (
        "temporal_eligibility_diagnostic.rs",
        "mean",
        "NaN for an empty iterator, delegating to binn_lab::mean_or_nan, which is \
         what its unguarded division already did. Deliberately not binn_lab::mean, \
         which would report 0.0 for a mean of nothing.",
    ),
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// `pub fn NAME` at the top level of a source file.
fn declared(text: &str, prefix: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix(prefix) {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            // A generic or argument list must follow the name, or this is a
            // prefix of some longer item rather than a declaration.
            if !name.is_empty() && rest[name.len()..].starts_with(['(', '<']) {
                names.insert(name);
            }
        }
    }
    names
}

#[test]
fn no_experiment_redeclares_a_binn_lab_export() {
    let root = repo_root();
    let mut exports = BTreeSet::new();
    for source in ["src/runner.rs", "src/lib.rs"] {
        let text = std::fs::read_to_string(root.join(source)).expect("read binn-lab source");
        exports.extend(declared(&text, "pub fn "));
    }
    assert!(
        exports.contains("mean") && exports.contains("std_error"),
        "the export scan found neither `mean` nor `std_error`; the pattern has \
         stopped matching and this test is not checking anything"
    );

    let mut offences = Vec::new();
    let mut seen_allowed = BTreeSet::new();
    let experiments = root.join("experiments");
    let mut files: Vec<_> = std::fs::read_dir(&experiments)
        .expect("read experiments directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "rs"))
        .collect();
    files.sort();
    assert!(
        files.len() > 10,
        "only {} experiment files found; the directory scan is wrong",
        files.len()
    );

    for path in &files {
        let file = path.file_name().unwrap().to_string_lossy().to_string();
        let text = std::fs::read_to_string(path).expect("read experiment");
        for name in declared(&text, "fn ") {
            if !exports.contains(&name) {
                continue;
            }
            match ALLOWED.iter().find(|(f, n, _)| *f == file && *n == name) {
                Some(entry) => {
                    seen_allowed.insert((entry.0, entry.1));
                }
                None => offences.push(format!(
                    "{file}::{name} re-declares `binn_lab::{name}`. Import it, or \
                     add an entry to ALLOWED saying how this one differs."
                )),
            }
        }
    }

    // A stale allow-list entry is worse than none: it reads as a live exception
    // and silently permits a future re-declaration under the same name.
    let stale: Vec<_> = ALLOWED
        .iter()
        .filter(|(f, n, _)| !seen_allowed.contains(&(*f, *n)))
        .map(|(f, n, _)| format!("{f}::{n}"))
        .collect();

    assert!(
        offences.is_empty() && stale.is_empty(),
        "duplicated helpers: {offences:#?}\nstale ALLOWED entries (no such \
         declaration any more, delete them): {stale:#?}"
    );
}
