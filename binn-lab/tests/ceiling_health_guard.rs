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

/// Every Gate G2 verdict compared against a reference must go through the owner.
///
/// # Why the guard above could not have caught this
///
/// That guard looks for *vocabulary*: a file that says "ceiling health" or
/// "INVERTED" must reference `CeilingHealth`. Six sources decided a G2 verdict
/// against a gradient reference and used none of those words, so the guard was
/// silent on all six — including `runner_dfa_match.rs`, whose published output
/// `results/c1_dfa.md` reports a treatment at 0.9387 against its own ceiling at
/// 0.8963 and calls it `PASS`. `CeilingHealth::evaluate(0.8963, 0.9387, 0.5)`
/// returns `Inverted`, and this crate has asserted exactly that since
/// 2026-08-21 in `the_dfa_arm_exceeds_its_own_ceiling`.
///
/// A guard keyed on what a file *says* cannot see a file that says nothing.
/// This one is keyed on what a file *does*: constructing `GateG2Verdict::Pass`
/// is the act, and any source that performs it must obtain the verdict from
/// the canonical owner.
#[test]
fn every_matched_g2_verdict_goes_through_the_canonical_owner() {
    const OWNER: &str = "decide_matched_verdict";
    // The act that requires the owner: returning a G2 verdict. Keyed on the
    // signature rather than on `GateG2Verdict::Pass`, because after a source
    // delegates correctly it no longer constructs the variant — so a marker on
    // the construction would have gone quiet exactly when the fix landed, and
    // the guard would have reported "nothing to check" as though it were
    // "nothing wrong".
    const ACT: &str = "-> GateG2Verdict";
    const EXEMPT: &[(&str, &str)] = &[("guards.rs", "is the canonical owner")];

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut sources = collect_sources(&root.join("src"));
    sources.extend(collect_sources(&root.join("experiments")));
    assert!(
        sources.len() > 20,
        "found only {} sources under src/ and experiments/ - the guard would \
         pass vacuously",
        sources.len()
    );

    let mut violations = Vec::new();
    let mut checked = 0usize;
    for path in &sources {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let text = fs::read_to_string(path).expect("read source");
        if !text.contains(ACT) || EXEMPT.iter().any(|(f, _)| *f == name) {
            continue;
        }
        checked += 1;
        if !text.contains(OWNER) {
            violations.push(format!(
                "{name}: decides a Gate G2 verdict without `guards::{OWNER}`.\n    \
                 A local rule that only checks `reference < floor` is blind to the \
                 treatment EXCEEDING its reference, and `gap_closed` is clamped to \
                 [0,1] downstream so the inversion arrives as a clean PASS."
            ));
        }
    }

    // Refuse a vacuous pass: if the enum is renamed or the deciders move,
    // finding nothing must not read as finding nothing wrong.
    assert!(
        checked >= 6,
        "expected at least 6 sources to decide a G2 verdict, found {checked} - \
         `{ACT}` has drifted and this guard covers nothing"
    );
    assert!(
        violations.is_empty(),
        "G2 verdicts bypassing the canonical owner:\n\n{}",
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
