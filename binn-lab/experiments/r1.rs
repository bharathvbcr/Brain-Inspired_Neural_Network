//! R1 experiment harness entry (U16) — multi-area composition.
//!
//! **Opt-in only.** Gate G2 FAIL under `c1-118207fbc3eaba53` still stands.
//!
//! ```bash
//! cargo run -p binn-lab --bin r1 -- --enable-r1 --quick
//! cargo run -p binn-lab --bin r1 -- --override-g2-for r1 --quick
//! BINN_OVERRIDE_G2_FOR=r1 cargo run -p binn-lab --bin r1 -- --quick
//! cargo run -p binn-lab --bin r1 -- --enable-r1 --quick --export-trace
//! ```

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use binn_lab::{R1Config, R1Runner};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut quick = false;
    let mut hash: Option<String> = None;
    let mut out: Option<PathBuf> = None;
    let mut export_trace: Option<PathBuf> = None;
    let mut enable_r1 = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--quick" => quick = true,
            "--enable-r1" => enable_r1 = true,
            "--override-g2-for" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--override-g2-for requires a value (expected: r1)");
                    return ExitCode::from(2);
                }
                if args[i].eq_ignore_ascii_case("r1") {
                    enable_r1 = true;
                } else {
                    eprintln!(
                        "--override-g2-for {} is not supported (only `r1` in this binary)",
                        args[i]
                    );
                    return ExitCode::from(2);
                }
            }
            "--config-hash" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--config-hash requires a value");
                    return ExitCode::from(2);
                }
                hash = Some(args[i].clone());
            }
            "--out" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--out requires a path");
                    return ExitCode::from(2);
                }
                out = Some(PathBuf::from(&args[i]));
            }
            "--export-trace" => {
                // Optional path; default when flag present with no following path.
                let default = PathBuf::from("results/r1_trace.jsonl");
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    i += 1;
                    export_trace = Some(PathBuf::from(&args[i]));
                } else {
                    export_trace = Some(default);
                }
            }
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("unknown arg: {other}");
                print_help();
                return ExitCode::from(2);
            }
        }
        i += 1;
    }

    if env_override_r1() {
        enable_r1 = true;
    }

    if !enable_r1 {
        eprintln!(
            "R1 is blocked by the v8 G2 kill-gate (FAIL under c1-118207fbc3eaba53).\n\
             \n\
             This is an exploratory post-kill-gate branch. To run R1 you must\n\
             explicitly override the gate:\n\
             \n\
               cargo run -p binn-lab --bin r1 -- --enable-r1 [--quick]\n\
               cargo run -p binn-lab --bin r1 -- --override-g2-for r1 [--quick]\n\
               BINN_OVERRIDE_G2_FOR=r1 cargo run -p binn-lab --bin r1 -- [--quick]\n\
             \n\
             See results/R1_OVERRIDE.md. Default program path (C1 / U-NEG) is unchanged."
        );
        return ExitCode::from(2);
    }

    let mut config = if let Some(h) = hash {
        match R1Config::from_hash(&h) {
            Some(c) => c,
            None => {
                eprintln!("unknown R1 config hash `{h}` — known presets:");
                for p in R1Config::known_presets() {
                    eprintln!("  {}  (quick={})", p.hash_string(), p.quick);
                }
                return ExitCode::from(2);
            }
        }
    } else if quick {
        R1Config::r1_quick()
    } else {
        R1Config::r1_default()
    };
    config.kill_gate_override = true;

    println!("R1 config hash: {}", config.hash_string());
    println!("protocol version: {}", binn_lab::R1_PROTOCOL_VERSION);
    println!("seeds: {:?}", config.seeds());
    println!("WARNING: kill-gate override active — G2 FAIL (c1-118207fbc3eaba53) still stands");

    let mut runner = R1Runner::new();
    let report = runner.run_r1(&config);
    let md = R1Runner::render_results_markdown(&report, &config);

    let out_path = out.unwrap_or_else(|| {
        let default_name = if config.quick {
            "results/r1_composition_quick.md"
        } else {
            "results/r1_composition.md"
        };
        resolve_results_path(default_name)
    });
    if let Some(parent) = out_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Err(e) = fs::write(&out_path, &md) {
        eprintln!("failed to write {}: {e}", out_path.display());
        return ExitCode::from(1);
    }

    if let Some(trace_arg) = export_trace {
        let seeds = config.seeds();
        let Some(&seed) = seeds.first() else {
            eprintln!("--export-trace requires at least one seed");
            return ExitCode::from(2);
        };
        let n_areas = config.max_areas;
        let tr = R1Runner::export_static_trace(&config, seed, n_areas);
        let trace_path = if trace_arg.is_absolute() {
            trace_arg
        } else {
            resolve_results_path(trace_arg.to_string_lossy().as_ref())
        };
        if let Some(parent) = trace_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Err(e) = tr.write_jsonl(&trace_path) {
            eprintln!("failed to write {}: {e}", trace_path.display());
            return ExitCode::from(1);
        }
        println!(
            "trace export: {} (n_areas={}, seed={}, lines={})",
            trace_path.display(),
            n_areas,
            seed,
            tr.lines().len()
        );
    }

    println!("verdict: {}", report.verdict.as_str());
    println!("compound_fraction: {:.3}", report.compound_fraction);
    for p in &report.points {
        println!(
            "  n={}: composed={:.4} additive={:.4} compounds={}",
            p.n_areas, p.mean_composed, p.mean_additive, p.compounds
        );
    }
    println!("results note: {}", out_path.display());

    ExitCode::SUCCESS
}

fn resolve_results_path(default_name: &str) -> PathBuf {
    let candidates = [
        PathBuf::from(default_name),
        PathBuf::from(format!("binn/{default_name}")),
        PathBuf::from(format!("binn-lab/{default_name}")),
    ];
    candidates
        .into_iter()
        .find(|p| p.parent().map(|d| d.exists()).unwrap_or(false))
        .unwrap_or_else(|| PathBuf::from(default_name))
}

fn env_override_r1() -> bool {
    match env::var("BINN_OVERRIDE_G2_FOR") {
        Ok(v) => v
            .split(',')
            .any(|p| matches!(p.trim().to_ascii_lowercase().as_str(), "r1" | "all")),
        Err(_) => false,
    }
}

fn print_help() {
    eprintln!(
        "Usage: r1 --enable-r1 [--quick] [--config-hash HASH] [--out PATH]\n\
         \n\
         Kill-gate override (required; pick one):\n\
           --enable-r1\n\
           --override-g2-for r1\n\
           BINN_OVERRIDE_G2_FOR=r1\n\
         \n\
         Optional:\n\
           --export-trace [PATH]  static topology/flow JSONL\n\
                                  (default: results/r1_trace.jsonl)\n\
         \n\
         Multi-area composition (U16). Opt-in only.\n\
         Does not reopen protocol-v2 kill-gate c1-118207fbc3eaba53.\n\
         --quick is PILOT only."
    );
}
