//! Polars helpers for harvesting result tables from markdown / CSV notes.
//!
//! Optional `tables` feature only — never on the scientific hot path or config hashes.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use polars::prelude::*;

/// One harvested numeric cell from a results markdown table row.
#[derive(Clone, Debug, PartialEq)]
pub struct HarvestedRow {
    pub arm: String,
    pub hash: String,
    pub verdict: String,
    pub primary_mean: Option<f64>,
    pub gap_lcb: Option<f64>,
    pub source: String,
}

/// Parse pipe-table rows from a markdown sheet into a Polars [`DataFrame`].
///
/// Expects header columns that include at least `Arm` / `Hash` / `Verdict`
/// (case-insensitive). Extra numeric columns (`Primary mean`, `Gap LCB`,
/// `Local`, …) are pulled when present.
pub fn harvest_markdown_table(md: &str) -> PolarsResult<DataFrame> {
    let rows = parse_markdown_rows(md);
    let arms: Vec<String> = rows.iter().map(|r| r.arm.clone()).collect();
    let hashes: Vec<String> = rows.iter().map(|r| r.hash.clone()).collect();
    let verdicts: Vec<String> = rows.iter().map(|r| r.verdict.clone()).collect();
    let means: Vec<Option<f64>> = rows.iter().map(|r| r.primary_mean).collect();
    let lcbs: Vec<Option<f64>> = rows.iter().map(|r| r.gap_lcb).collect();
    let sources: Vec<String> = rows.iter().map(|r| r.source.clone()).collect();
    DataFrame::new(vec![
        Series::new("arm".into(), arms).into(),
        Series::new("hash".into(), hashes).into(),
        Series::new("verdict".into(), verdicts).into(),
        Series::new("primary_mean".into(), means).into(),
        Series::new("gap_lcb".into(), lcbs).into(),
        Series::new("source".into(), sources).into(),
    ])
}

/// Read a markdown file and harvest its first results-like pipe table.
pub fn harvest_markdown_file(path: &Path) -> PolarsResult<DataFrame> {
    let text = fs::read_to_string(path)
        .map_err(|e| PolarsError::ComputeError(format!("read {}: {e}", path.display()).into()))?;
    harvest_markdown_table(&text)
}

/// Aggregate mean of `primary_mean` by `verdict` (PASS / FAIL / …).
///
/// Eager-only (no lazy/streaming) so the optional `tables` feature stays lean.
pub fn mean_by_verdict(df: &DataFrame) -> PolarsResult<DataFrame> {
    let verdict = df.column("verdict")?.str()?;
    let means = df.column("primary_mean")?.f64()?;
    let mut sums: BTreeMap<String, (f64, u32)> = BTreeMap::new();
    for i in 0..df.height() {
        let v = verdict.get(i).unwrap_or("").to_string();
        if let Some(m) = means.get(i) {
            let e = sums.entry(v).or_insert((0.0, 0));
            e.0 += m;
            e.1 += 1;
        } else {
            sums.entry(v).or_insert((0.0, 0));
        }
    }
    let mut vs = Vec::new();
    let mut ms = Vec::new();
    let mut ns = Vec::new();
    for (v, (sum, n)) in sums {
        vs.push(v);
        ms.push(if n > 0 {
            Some(sum / f64::from(n))
        } else {
            None
        });
        ns.push(n as u32);
    }
    DataFrame::new(vec![
        Series::new("verdict".into(), vs).into(),
        Series::new("mean_primary".into(), ms).into(),
        Series::new("n_arms".into(), ns).into(),
    ])
}

/// Write a harvested frame to CSV (UTF-8).
pub fn write_csv(df: &mut DataFrame, path: &Path) -> PolarsResult<()> {
    let mut file = fs::File::create(path)
        .map_err(|e| PolarsError::ComputeError(format!("create {}: {e}", path.display()).into()))?;
    CsvWriter::new(&mut file).finish(df)?;
    Ok(())
}

fn parse_markdown_rows(md: &str) -> Vec<HarvestedRow> {
    let mut rows = Vec::new();
    let mut header: Option<Vec<String>> = None;
    for line in md.lines() {
        let line = line.trim();
        if !line.starts_with('|') {
            continue;
        }
        let cells: Vec<String> = line
            .trim_matches('|')
            .split('|')
            .map(|c| c.trim().to_string())
            .collect();
        if cells
            .iter()
            .all(|c| c.chars().all(|ch| ch == '-' || ch == ':' || ch == ' '))
        {
            continue;
        }
        if header.is_none() {
            let lower: Vec<String> = cells.iter().map(|c| c.to_ascii_lowercase()).collect();
            if lower
                .iter()
                .any(|c| c.contains("arm") || c.contains("protocol"))
                && lower.iter().any(|c| c.contains("hash"))
            {
                header = Some(lower);
            }
            continue;
        }
        let hdr = header.as_ref().unwrap();
        if cells.len() < hdr.len() {
            continue;
        }
        let get = |key: &str| -> String {
            hdr.iter()
                .position(|h| h.contains(key))
                .and_then(|i| cells.get(i).cloned())
                .unwrap_or_default()
        };
        let arm = get("arm");
        let arm = if arm.is_empty() { get("protocol") } else { arm };
        let hash = strip_ticks(&get("hash"));
        let verdict = strip_bold(&get("verdict"));
        if arm.is_empty() || hash.is_empty() {
            continue;
        }
        let primary = parse_f64_cell(&get("primary"))
            .or_else(|| parse_f64_cell(&get("local")))
            .or_else(|| parse_f64_cell(&get("mean")));
        let gap = parse_f64_cell(&get("gap"));
        let source = get("source");
        rows.push(HarvestedRow {
            arm,
            hash,
            verdict,
            primary_mean: primary,
            gap_lcb: gap,
            source,
        });
    }
    rows
}

fn strip_ticks(s: &str) -> String {
    s.trim().trim_matches('`').trim().to_string()
}

fn strip_bold(s: &str) -> String {
    s.replace("**", "").trim().to_string()
}

fn parse_f64_cell(s: &str) -> Option<f64> {
    let cleaned = s
        .replace("**", "")
        .replace(',', "")
        .replace('−', "-") // unicode minus
        .replace('–', "-") // en-dash
        .split_whitespace()
        .next()?
        .trim_matches(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '+')
        .to_string();
    if cleaned.is_empty() {
        return None;
    }
    cleaned.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harvests_paper_style_table() {
        let md = r#"
## Table A

| Arm | Hash | Verdict | Primary mean | Gap LCB | Source |
|---|---|---|---:|---:|---|
| Broadcast ±1 | `c1-match-5dc6822e71229e9e` | **FAIL** | 0.5000 | **0.0000** | [`c1_match.md`](c1_match.md) |
| DFA | `c1-dfa-c8c4fe0899908b84` | **PASS** | 0.9387 | **0.6894** | [`c1_dfa.md`](c1_dfa.md) |
"#;
        let df = harvest_markdown_table(md).unwrap();
        assert_eq!(df.height(), 2);
        let agg = mean_by_verdict(&df).unwrap();
        assert!(agg.height() >= 1);
    }

    /// Optional camp harvest (writes CSV next to RESULTS.md when present).
    #[test]
    fn harvest_dfa_live_size_results_note() {
        let path = Path::new("results/runs/2026-07-24-dfa-live-size/RESULTS.md");
        let path = if path.exists() {
            path.to_path_buf()
        } else {
            Path::new("../results/runs/2026-07-24-dfa-live-size/RESULTS.md").to_path_buf()
        };
        if !path.exists() {
            return;
        }
        let mut df = harvest_markdown_file(&path).expect("harvest RESULTS.md");
        assert_eq!(df.height(), 3, "expected 3 arm rows, got {}", df.height());
        let verdicts = df.column("verdict").unwrap().str().unwrap();
        let mut saw_accept = false;
        for i in 0..df.height() {
            if verdicts.get(i) == Some("Accept") {
                saw_accept = true;
            }
        }
        assert!(saw_accept, "expected Accept verdict in harvest");
        let out = path.with_file_name("arm_table.csv");
        write_csv(&mut df, &out).expect("write arm_table.csv");
    }
}
