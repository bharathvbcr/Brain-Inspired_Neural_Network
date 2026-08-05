use std::path::{Path, PathBuf};
use std::process::ExitCode;

use binn_hybrid_lab::{
    run_production_diagnostics, ProductionDiagnosticConfig, ProductionDiagnosticReport,
};

fn main() -> ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let quick = args.iter().any(|argument| argument == "--quick");
    let out_dir = args
        .windows(2)
        .find(|pair| pair[0] == "--out-dir")
        .map(|pair| PathBuf::from(&pair[1]))
        .unwrap_or_else(|| PathBuf::from("hybrid-results/production-diagnostics"));
    let config = if quick {
        ProductionDiagnosticConfig::quick()
    } else {
        ProductionDiagnosticConfig::full()
    };
    let report = run_production_diagnostics(&config);
    if let Err(error) = write_report(&out_dir, &report) {
        eprintln!("failed to write production-event diagnostic evidence: {error}");
        return ExitCode::from(1);
    }
    println!(
        "BINN-Hybrid production diagnostic: {}",
        report.protocol_hash
    );
    for (arm, depth) in &report.best_d_star {
        println!(
            "{} D*={}",
            arm.as_str(),
            depth
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        );
    }
    println!("results: {}", out_dir.display());
    ExitCode::SUCCESS
}

fn write_report(out_dir: &Path, report: &ProductionDiagnosticReport) -> std::io::Result<()> {
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
