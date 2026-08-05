//! C3 experiment harness entry (U15) — Gate credit-depth path.
//!
//! **Opt-in only.** Gate G2 FAIL under `c1-118207fbc3eaba53` still stands.
//! This binary refuses to run unless you explicitly override the v8 kill-gate:
//!
//! ```bash
//! cargo run -p binn-lab --bin c3 -- --enable-c3 --quick
//! cargo run -p binn-lab --bin c3 -- --override-g2-for c3 --quick
//! BINN_OVERRIDE_G2_FOR=c3 cargo run -p binn-lab --bin c3 -- --quick
//! ```
//!
//! Full scientific schedule (still exploratory):
//!
//! ```bash
//! cargo run -p binn-lab --release --bin c3 -- --enable-c3 --out results/c3_credit_depth.md
//! ```

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use binn_lab::{C3Config, C3Runner};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut quick = false;
    let mut hash: Option<String> = None;
    let mut out: Option<PathBuf> = None;
    let mut enable_c3 = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--quick" => quick = true,
            "--enable-c3" => enable_c3 = true,
            "--override-g2-for" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--override-g2-for requires a value (expected: c3)");
                    return ExitCode::from(2);
                }
                if args[i].eq_ignore_ascii_case("c3") {
                    enable_c3 = true;
                } else {
                    eprintln!(
                        "--override-g2-for {} is not supported (only `c3` in this binary)",
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

    if env_override_c3() {
        enable_c3 = true;
    }

    if !enable_c3 {
        eprintln!(
            "C3 is blocked by the v8 G2 kill-gate (FAIL under c1-118207fbc3eaba53).\n\
             \n\
             This is an exploratory post-kill-gate branch. To run C3 you must\n\
             explicitly override the gate:\n\
             \n\
               cargo run -p binn-lab --bin c3 -- --enable-c3 [--quick]\n\
               cargo run -p binn-lab --bin c3 -- --override-g2-for c3 [--quick]\n\
               BINN_OVERRIDE_G2_FOR=c3 cargo run -p binn-lab --bin c3 -- [--quick]\n\
             \n\
             See results/C3_OVERRIDE.md. Default program path (C1 / U-NEG) is unchanged."
        );
        return ExitCode::from(2);
    }

    let mut config = if let Some(h) = hash {
        match C3Config::from_hash(&h) {
            Some(c) => c,
            None => {
                eprintln!("unknown C3 config hash `{h}` — known presets:");
                for p in C3Config::known_presets() {
                    eprintln!("  {}  (quick={})", p.hash_string(), p.quick);
                }
                return ExitCode::from(2);
            }
        }
    } else if quick {
        C3Config::c3_quick()
    } else {
        C3Config::c3_default()
    };
    config.kill_gate_override = true;

    println!("C3 config hash: {}", config.hash_string());
    println!("protocol version: {}", binn_lab::C3_PROTOCOL_VERSION);
    println!("seeds: {:?}", config.seeds());
    println!("WARNING: kill-gate override active — G2 FAIL (c1-118207fbc3eaba53) still stands");

    let mut runner = C3Runner::new();
    let report = runner.run_c3(&config);
    let md = C3Runner::render_results_markdown(&report, &config);

    let out_path = out.unwrap_or_else(|| {
        let default_name = if config.quick {
            "results/c3_credit_depth_quick.md"
        } else {
            "results/c3_credit_depth.md"
        };
        let candidates = [
            PathBuf::from(default_name),
            PathBuf::from(format!("binn/{default_name}")),
            PathBuf::from(format!("binn-lab/{default_name}")),
        ];
        candidates
            .into_iter()
            .find(|p| p.parent().map(|d| d.exists()).unwrap_or(false))
            .unwrap_or_else(|| PathBuf::from(default_name))
    });
    if let Some(parent) = out_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Err(e) = fs::write(&out_path, &md) {
        eprintln!("failed to write {}: {e}", out_path.display());
        return ExitCode::from(1);
    }

    println!("verdict: {}", report.verdict.as_str());
    println!(
        "D* tabular-local={}  D* oracle-teacher-forced={}",
        report
            .d_star
            .map(|d| d.to_string())
            .unwrap_or_else(|| "none".into()),
        report
            .d_star_gradient
            .map(|d| d.to_string())
            .unwrap_or_else(|| "none".into())
    );
    for r in &report.depth_results {
        println!(
            "  depth {}: tabular-local={:.4} oracle={:.4}",
            r.depth, r.mean_accuracy_local, r.mean_accuracy_gradient
        );
    }
    println!("results note: {}", out_path.display());

    ExitCode::SUCCESS
}

fn env_override_c3() -> bool {
    match env::var("BINN_OVERRIDE_G2_FOR") {
        Ok(v) => v
            .split(',')
            .any(|p| matches!(p.trim().to_ascii_lowercase().as_str(), "c3" | "all")),
        Err(_) => false,
    }
}

fn print_help() {
    eprintln!(
        "Usage: c3 --enable-c3 [--quick] [--config-hash HASH] [--out PATH]\n\
         \n\
         Kill-gate override (required; pick one):\n\
           --enable-c3\n\
           --override-g2-for c3\n\
           BINN_OVERRIDE_G2_FOR=c3\n\
         \n\
         C3 v1 tabular credit-depth proxy (U15). Opt-in only.\n\
         This is not the production event-engine / ThreeFactor learner.\n\
         Does not reopen protocol-v2 kill-gate c1-118207fbc3eaba53.\n\
         --quick is PILOT only (never a scientific D* claim)."
    );
}
