use std::path::{Path, PathBuf};
use std::process::ExitCode;

use binn_hybrid_lab::{run_temperature_ablation, TemperatureAblationConfig};

fn main() -> ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let quick = args.iter().any(|a| a == "--quick");
    let out_dir = args
        .windows(2)
        .find(|pair| pair[0] == "--out-dir")
        .map(|pair| PathBuf::from(&pair[1]))
        .unwrap_or_else(|| PathBuf::from("hybrid-results/temperature-ablation"));
    let config = if quick {
        TemperatureAblationConfig::quick()
    } else {
        TemperatureAblationConfig::scientific()
    };
    let report = run_temperature_ablation(&config);
    if let Err(error) = write_report(&out_dir, &report) {
        eprintln!("failed to write T-ablation evidence: {error}");
        return ExitCode::from(1);
    }
    println!(
        "BINN-Hybrid winner-temperature ablation: {}",
        report.protocol_hash
    );
    for row in &report.collapses {
        println!(
            "variant={} soft_D*={} T2_D*={} collapse={}",
            row.variant,
            row.soft_d_star
                .map(|d| d.to_string())
                .unwrap_or_else(|| "none".into()),
            row.t2_d_star
                .map(|d| d.to_string())
                .unwrap_or_else(|| "none".into()),
            row.collapse_temperature
                .map(|t| t.as_str())
                .unwrap_or_else(|| "none".into()),
        );
    }
    println!("results: {}", out_dir.display());
    ExitCode::SUCCESS
}

fn write_report(
    out_dir: &Path,
    report: &binn_hybrid_lab::TemperatureAblationReport,
) -> std::io::Result<()> {
    std::fs::create_dir_all(out_dir)?;
    let stem = &report.protocol_hash;
    std::fs::write(out_dir.join(format!("{stem}.md")), report.render_markdown())?;
    std::fs::write(
        out_dir.join(format!("{stem}-sweep.csv")),
        report.render_sweep_csv(),
    )?;
    Ok(())
}
