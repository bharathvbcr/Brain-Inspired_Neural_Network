//! Build-time guard against hardcoded verdicts in report templates.
//!
//! # The bug this prevents
//!
//! `deep_snn_scaling.rs` emitted:
//!
//! ```text
//! | 2-Hidden-Layer Deep Learned FB | {hidden} → {hidden} | {m_l2:.4} | {se_l2:.4} | {} | PASS |
//! ```
//!
//! The "Clears Floor" cell was computed; the "Verdict" cell was the literal
//! `PASS`. The shipped report therefore contained rows reading `FAIL | PASS`.
//! The same defect was present in `ei_inhibition_sweep.rs` (every sweep row),
//! `multi_channel_neuromod.rs` (both rows) and `live_transfer_rescue.rs`.
//!
//! This test scans every experiment source for markdown table cells that are a
//! verdict *literal* rather than a formatted value, and fails the build.
//!
//! # Escape hatch
//!
//! A row that is legitimately constant (e.g. a reference arm always labelled
//! `CEILING`) may opt out with a comment on the same or preceding line:
//!
//! ```text
//! // verdict-literal-ok: reference arm, not a hypothesis under test
//! ```

use std::fs;
use std::path::{Path, PathBuf};

/// Cell contents that must never be hardcoded.
const VERDICT_LITERALS: &[&str] = &[
    "PASS",
    "FAIL",
    "DEGENERATE",
    "UNDERPOWERED",
    "INVALID_HARNESS",
    "PASS (matched)",
    "PASS (matched schedule)",
];

const OPT_OUT: &str = "verdict-literal-ok";

fn experiments_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("experiments")
}

/// Strip markdown emphasis and whitespace from a table cell.
fn normalise_cell(cell: &str) -> String {
    cell.trim()
        .trim_matches('*')
        .trim()
        .trim_end_matches("\\n")
        .trim_end_matches('\\')
        .trim()
        .to_string()
}

/// A line is a candidate table row if it has at least three pipes, which is the
/// minimum for a two-column markdown row.
fn is_table_row(line: &str) -> bool {
    line.matches('|').count() >= 3
}

/// Cells that came from a format placeholder are fine; only bare literals fail.
fn contains_placeholder(cell: &str) -> bool {
    cell.contains('{') && cell.contains('}')
}

fn collect_sources(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        panic!("cannot read experiments dir: {}", dir.display());
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    out.sort();
    out
}

#[test]
fn no_hardcoded_verdicts_in_report_templates() {
    let dir = experiments_dir();
    let sources = collect_sources(&dir);
    assert!(
        !sources.is_empty(),
        "found no experiment sources under {}",
        dir.display()
    );

    let mut violations: Vec<String> = Vec::new();

    for path in &sources {
        let text = fs::read_to_string(path).expect("read experiment source");
        let lines: Vec<&str> = text.lines().collect();

        for (idx, line) in lines.iter().enumerate() {
            if !is_table_row(line) {
                continue;
            }
            // Opt-out on this line or the one above it.
            let prev_opt_out = idx > 0 && lines[idx - 1].contains(OPT_OUT);
            if line.contains(OPT_OUT) || prev_opt_out {
                continue;
            }
            // Separator rows like |---|---:| are not verdicts.
            let is_separator = line
                .chars()
                .all(|c| matches!(c, '|' | '-' | ':' | ' ' | '\\' | 'n' | '"'));
            if is_separator {
                continue;
            }

            for cell in line.split('|') {
                if contains_placeholder(cell) {
                    continue;
                }
                let normalised = normalise_cell(cell);
                if normalised.is_empty() {
                    continue;
                }
                if VERDICT_LITERALS
                    .iter()
                    .any(|lit| normalised.eq_ignore_ascii_case(lit))
                {
                    violations.push(format!(
                        "{}:{}: hardcoded verdict cell `{}` in\n    {}",
                        path.file_name().unwrap().to_string_lossy(),
                        idx + 1,
                        normalised,
                        line.trim()
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "hardcoded verdict literals found in report templates.\n\
         A verdict cell must come from `binn_lab::guards::Verdict::evaluate(..).label()`, \
         so it can never contradict the number printed beside it.\n\
         If a row is legitimately constant, add `// {OPT_OUT}: <reason>` above it.\n\n{}",
        violations.join("\n")
    );
}

/// The guard is only useful if it actually fires. This proves the detector
/// catches the exact shape of the shipped bug.
#[test]
fn guard_detects_the_original_deep_snn_defect() {
    let offending =
        "        | 2-Hidden-Layer Deep Learned FB | {hidden} | {m_l2:.4} | {se_l2:.4} | {} | PASS |\\n\\";
    assert!(is_table_row(offending));
    let flagged = offending
        .split('|')
        .filter(|c| !contains_placeholder(c))
        .map(normalise_cell)
        .any(|c| VERDICT_LITERALS.iter().any(|l| c.eq_ignore_ascii_case(l)));
    assert!(flagged, "detector failed to catch the known-bad row");
}

/// Placeholder-driven verdict cells must not be flagged.
#[test]
fn guard_allows_computed_verdicts() {
    let good = "        | 2-Hidden-Layer Deep Learned FB | {m_l2:.4} | {se_l2:.4} | {v_l2} |\\n\\";
    let flagged = good
        .split('|')
        .filter(|c| !contains_placeholder(c))
        .map(normalise_cell)
        .any(|c| VERDICT_LITERALS.iter().any(|l| c.eq_ignore_ascii_case(l)));
    assert!(!flagged, "detector false-positived on a computed verdict");
}
