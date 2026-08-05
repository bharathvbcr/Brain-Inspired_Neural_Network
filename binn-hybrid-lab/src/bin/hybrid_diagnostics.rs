use std::path::{Path, PathBuf};
use std::process::ExitCode;

use binn_hybrid_lab::{run_diagnostics, DiagnosticConfig, DiagnosticReport};

fn main() -> ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let quick = args.iter().any(|argument| argument == "--quick");
    let out_dir = args
        .windows(2)
        .find(|pair| pair[0] == "--out-dir")
        .map(|pair| PathBuf::from(&pair[1]))
        .unwrap_or_else(|| PathBuf::from("hybrid-results/diagnostics"));
    let config = if quick {
        DiagnosticConfig::quick()
    } else {
        DiagnosticConfig::full()
    };
    let report = run_diagnostics(&config);
    if let Err(error) = write_report(&out_dir, &report) {
        eprintln!("failed to write BINN-Hybrid diagnostic evidence: {error}");
        return ExitCode::from(1);
    }
    println!("BINN-Hybrid diagnostic: {}", report.protocol_hash);
    println!("results: {}", out_dir.display());
    ExitCode::SUCCESS
}

fn write_report(out_dir: &Path, report: &DiagnosticReport) -> std::io::Result<()> {
    std::fs::create_dir_all(out_dir)?;
    let stem = &report.protocol_hash;
    std::fs::write(out_dir.join(format!("{stem}.md")), report.render_markdown())?;
    std::fs::write(
        out_dir.join(format!("{stem}-sweep.csv")),
        report.render_sweep_csv(),
    )?;
    std::fs::write(
        out_dir.join(format!("{stem}-mechanisms.csv")),
        report.render_mechanism_csv(),
    )?;
    Ok(())
}
