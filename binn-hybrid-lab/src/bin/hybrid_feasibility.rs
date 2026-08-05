use std::path::{Path, PathBuf};
use std::process::ExitCode;

use binn_hybrid_lab::{run_feasibility, HybridProtocol};

fn main() -> ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let quick = args.iter().any(|argument| argument == "--quick");
    let out_dir = args
        .windows(2)
        .find(|pair| pair[0] == "--out-dir")
        .map(|pair| PathBuf::from(&pair[1]))
        .unwrap_or_else(|| PathBuf::from("hybrid-results"));
    let protocol = if quick {
        HybridProtocol::quick()
    } else {
        HybridProtocol::scientific()
    };
    let report = run_feasibility(&protocol);
    if let Err(error) = write_report(&out_dir, &report, &protocol) {
        eprintln!("failed to write BINN-Hybrid feasibility evidence: {error}");
        return ExitCode::from(1);
    }
    println!(
        "BINN-Hybrid {}: {}",
        report.protocol_hash,
        report.decision.as_str()
    );
    println!("results: {}", out_dir.display());
    ExitCode::SUCCESS
}

fn write_report(
    out_dir: &Path,
    report: &binn_hybrid_lab::FeasibilityReport,
    protocol: &HybridProtocol,
) -> std::io::Result<()> {
    std::fs::create_dir_all(out_dir)?;
    std::fs::write(
        out_dir.join(format!("{}.md", report.protocol_hash)),
        report.render_markdown(protocol),
    )?;
    if !report.artifacts.is_empty() {
        let artifact_dir = out_dir.join(format!("artifacts-{}", report.protocol_hash));
        std::fs::create_dir_all(&artifact_dir)?;
        for (name, artifact) in &report.artifacts {
            let encoded = artifact
                .encode()
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            std::fs::write(artifact_dir.join(name), encoded)?;
        }
    }
    Ok(())
}
