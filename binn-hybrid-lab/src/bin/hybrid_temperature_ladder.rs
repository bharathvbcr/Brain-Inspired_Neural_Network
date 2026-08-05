use std::path::{Path, PathBuf};
use std::process::ExitCode;

use binn_hybrid_lab::{
    run_temperature_ladder, LadderArm, TemperatureLadderConfig, TemperatureLadderReport,
};

fn main() -> ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let quick = args.iter().any(|argument| argument == "--quick");
    let out_dir = args
        .windows(2)
        .find(|pair| pair[0] == "--out-dir")
        .map(|pair| PathBuf::from(&pair[1]))
        .unwrap_or_else(|| PathBuf::from("hybrid-results/temperature-ladder"));
    let config = if quick {
        TemperatureLadderConfig::quick()
    } else {
        TemperatureLadderConfig::full()
    };
    let report = run_temperature_ladder(&config);
    if let Err(error) = write_report(&out_dir, &report) {
        eprintln!("failed to write winner-temperature ladder evidence: {error}");
        return ExitCode::from(1);
    }
    println!(
        "BINN-Hybrid winner-temperature ladder: {}",
        report.protocol_hash
    );
    for (temperature, arm, depth) in &report.best_d_star {
        if *arm != LadderArm::DirectTerminal {
            continue;
        }
        println!(
            "direct-terminal T={} D*={}",
            temperature.as_str(),
            depth
                .map(|value| value.to_string())
                .unwrap_or_else(|| "none".to_string())
        );
    }
    match report.collapse_temperature {
        Some(temperature) => println!("collapse temperature: {}", temperature.as_str()),
        None => println!("collapse temperature: none"),
    }
    println!("results: {}", out_dir.display());
    ExitCode::SUCCESS
}

fn write_report(out_dir: &Path, report: &TemperatureLadderReport) -> std::io::Result<()> {
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
