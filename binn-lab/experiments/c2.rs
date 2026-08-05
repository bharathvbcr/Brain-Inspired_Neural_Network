//! C2 experiment harness entry (U14) — Gate G3 path.
//!
//! **Opt-in only.** Gate G2 FAIL under `c1-118207fbc3eaba53` still stands.
//! This binary refuses to run unless you explicitly override the v8 kill-gate:
//!
//! ```bash
//! cargo run -p binn-lab --bin c2 -- --enable-c2 --quick
//! cargo run -p binn-lab --bin c2 -- --override-g2-for c2 --quick
//! BINN_OVERRIDE_G2_FOR=c2 cargo run -p binn-lab --bin c2 -- --quick
//! ```
//!
//! Full scientific schedule (still exploratory):
//!
//! ```bash
//! cargo run -p binn-lab --release --bin c2 -- --enable-c2 --out results/c2_g3.md
//! ```

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use binn_lab::{C2Config, C2Runner};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut quick = false;
    let mut hash: Option<String> = None;
    let mut out: Option<PathBuf> = None;
    let mut enable_c2 = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--quick" => quick = true,
            "--enable-c2" => enable_c2 = true,
            "--override-g2-for" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--override-g2-for requires a value (expected: c2)");
                    return ExitCode::from(2);
                }
                if args[i].eq_ignore_ascii_case("c2") {
                    enable_c2 = true;
                } else {
                    eprintln!(
                        "--override-g2-for {} is not supported (only `c2` in this binary)",
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

    // Env override (same kill-gate acknowledgment).
    if env_override_c2() {
        enable_c2 = true;
    }

    if !enable_c2 {
        eprintln!(
            "C2 is blocked by the v8 G2 kill-gate (FAIL under c1-118207fbc3eaba53).\n\
             \n\
             This is an exploratory post-kill-gate branch. To run C2 you must\n\
             explicitly override the gate:\n\
             \n\
               cargo run -p binn-lab --bin c2 -- --enable-c2 [--quick]\n\
               cargo run -p binn-lab --bin c2 -- --override-g2-for c2 [--quick]\n\
               BINN_OVERRIDE_G2_FOR=c2 cargo run -p binn-lab --bin c2 -- [--quick]\n\
             \n\
             See results/C2_OVERRIDE.md. Default program path (C1 / U-NEG) is unchanged."
        );
        return ExitCode::from(2);
    }

    let mut config = if let Some(h) = hash {
        match C2Config::from_hash(&h) {
            Some(c) => c,
            None => {
                eprintln!("unknown C2 config hash `{h}` — known presets:");
                for p in C2Config::known_presets() {
                    eprintln!("  {}  (quick={})", p.hash_string(), p.quick);
                }
                return ExitCode::from(2);
            }
        }
    } else if quick {
        C2Config::c2_quick()
    } else {
        C2Config::c2_default()
    };
    config.kill_gate_override = true;

    println!("C2 config hash: {}", config.hash_string());
    println!("protocol version: {}", binn_lab::C2_PROTOCOL_VERSION);
    println!("seeds: {:?}", config.seeds());
    println!("WARNING: kill-gate override active — G2 FAIL (c1-118207fbc3eaba53) still stands");

    let mut runner = C2Runner::new();
    let report = runner.run_c2(&config);
    let md = C2Runner::render_results_markdown(&report, &config);

    let out_path = out.unwrap_or_else(|| {
        let default_name = if config.quick {
            "results/c2_g3_quick.md"
        } else {
            "results/c2_g3.md"
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

    println!("G3 verdict: {}", report.verdict.as_str());
    println!(
        "forgetting: local={:.4} baseline={:.4}  below_baseline={}",
        report.mean_forgetting_local, report.mean_forgetting_baseline, report.below_baseline
    );
    println!(
        "overlap intervention: high={:.4} low={:.4} shuffle={:.4}  direction_ok={}",
        report.mean_forgetting_high,
        report.mean_forgetting_low,
        report.mean_forgetting_shuffle,
        report.intervention_direction_ok
    );
    println!("results note: {}", out_path.display());

    ExitCode::SUCCESS
}

fn env_override_c2() -> bool {
    match env::var("BINN_OVERRIDE_G2_FOR") {
        Ok(v) => v.split(',').any(|p| p.trim().eq_ignore_ascii_case("c2")),
        Err(_) => false,
    }
}

fn print_help() {
    eprintln!(
        "Usage: c2 --enable-c2 [--quick] [--config-hash HASH] [--out PATH]\n\
         \n\
         Kill-gate override (required; pick one):\n\
           --enable-c2\n\
           --override-g2-for c2\n\
           BINN_OVERRIDE_G2_FOR=c2\n\
         \n\
         Class-incremental continual learning (U14 / Gate G3). Opt-in only.\n\
         Does not reopen protocol-v2 kill-gate c1-118207fbc3eaba53.\n\
         --quick is PILOT only (never a scientific PASS/FAIL)."
    );
}
