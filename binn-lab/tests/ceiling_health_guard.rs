//! One canonical owner for "can this reference bound anything?".
//!
//! # What this guard exists to prevent
//!
//! Five experiments independently implemented a ceiling-health check, and all
//! five implemented the *same* check — some form of
//! `ceiling_mean < treatment_mean` — which has a hole: it is silent when the
//! reference never learned **and the treatment is below it**.
//!
//! `deep-snn-scaling` v134 walked straight into it. Its depth-4 gradient ceiling
//! scored `0.5000 ± 0.0000` on a two-class task — a constant predictor — and the
//! report printed **`ok`**, because 0.5000 is not below the treatment's 0.4435.
//! The 1-hidden-layer arm printed **PASS** against a ceiling of 0.4880 on the
//! same task.
//!
//! The defect was not that any one site was written carelessly. It was that the
//! behaviour had five owners, so a hole in the shared idea had to be found five
//! times to be fixed once. `guards::CeilingHealth` is now the single owner, and
//! this test keeps it that way.
//!
//! See `RESULT_2026-08-20_DEEP_SNN_V134_CEILING_IS_AT_CHANCE.md`.

use std::fs;
use std::path::{Path, PathBuf};

/// Phrases that mean a source is making a ceiling-health claim in a report.
const CEILING_CLAIM_MARKERS: &[&str] = &[
    "Ceiling health",
    "ceiling health",
    "INVERTED",
    "ceiling inversion",
    "ceiling below treatment",
];

/// The canonical owner. A source making a ceiling claim must go through it.
const CANONICAL: &str = "CeilingHealth";

/// Per-file opt-out, with a reason, for a source that legitimately mentions a
/// marker without making the claim itself.
const OPT_OUT: &str = "ceiling-health-ok";

fn experiments_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("experiments")
}

fn collect_sources(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(dir)
        .expect("read experiments dir")
        .filter_map(|entry| {
            let path = entry.expect("dir entry").path();
            (path.extension().and_then(|e| e.to_str()) == Some("rs")).then_some(path)
        })
        .collect();
    out.sort();
    out
}

#[test]
fn every_ceiling_claim_goes_through_the_canonical_owner() {
    let dir = experiments_dir();
    let sources = collect_sources(&dir);
    assert!(
        !sources.is_empty(),
        "found no experiment sources under {} - the guard would pass vacuously",
        dir.display()
    );

    let mut violations = Vec::new();
    let mut checked = 0usize;

    for path in &sources {
        let text = fs::read_to_string(path).expect("read experiment source");
        if text.contains(OPT_OUT) {
            continue;
        }
        let markers: Vec<&str> = CEILING_CLAIM_MARKERS
            .iter()
            .copied()
            .filter(|marker| text.contains(marker))
            .collect();
        if markers.is_empty() {
            continue;
        }
        checked += 1;
        if !text.contains(CANONICAL) {
            violations.push(format!(
                "{}: reports ceiling health ({}) without `guards::{CANONICAL}`.\n    \
                 A local `ceiling < treatment` test cannot see a reference that never \
                 learned. Use `CeilingHealth::evaluate(reference, treatment, chance)`, \
                 or add `// {OPT_OUT}: <reason>` if this file does not make the claim.",
                path.file_name().unwrap().to_string_lossy(),
                markers.join(", "),
            ));
        }
    }

    assert!(
        checked >= 4,
        "expected at least 4 experiments to make ceiling claims, found {checked} - \
         the markers have drifted and this guard is no longer covering anything"
    );
    assert!(
        violations.is_empty(),
        "ceiling-health claims bypassing the canonical owner:\n\n{}",
        violations.join("\n\n")
    );
}

/// The guard must be able to fail. A marker with no canonical reference is a
/// violation; the same text with the reference is not.
#[test]
fn the_guard_detects_the_original_deep_snn_defect() {
    let defective = r#"
        let ceiling_inverted = cm + 1e-6 < m;
        push(if ceiling_inverted { "INVERTED - ceiling below treatment" } else { "ok" });
    "#;
    let repaired = r#"
        let health = CeilingHealth::evaluate(cm, m, CHANCE);
        push(health.label());
    "#;
    let has_marker = |t: &str| {
        CEILING_CLAIM_MARKERS
            .iter()
            .any(|marker| t.contains(marker))
    };

    assert!(has_marker(defective));
    assert!(
        !defective.contains(CANONICAL),
        "defective sample must violate"
    );
    assert!(repaired.contains(CANONICAL), "repaired sample must pass");
}
