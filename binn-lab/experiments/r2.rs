//! R2 experiment harness entry (U17) — scaling curve / Gate G4 DECISION,
//! plus opt-in directed-credit mitigation probe (`--credit` / `r2-credit-*`).
//!
//! **Opt-in only.** Gate G2 FAIL under `c1-118207fbc3eaba53` still stands.
//! G4 is DECISION not kill — a healthy curve does **not** prove 10⁴–10⁶ areas.
//! `--credit` does **not** reopen frozen G4 NO-GO (`r2-afafa0fa6f43e3fc`).
//!
//! ```bash
//! cargo run -p binn-lab --bin r2 -- --enable-r2 --quick
//! cargo run -p binn-lab --bin r2 -- --enable-r2 --credit --quick
//! cargo run -p binn-lab --bin r2 -- --override-g2-for r2 --quick
//! BINN_OVERRIDE_G2_FOR=r2 cargo run -p binn-lab --bin r2 -- --quick
//! ```

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use binn_lab::{R2Config, R2CreditConfig, R2CreditRunner, R2Runner};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut quick = false;
    let mut credit = false;
    let mut hash: Option<String> = None;
    let mut out: Option<PathBuf> = None;
    let mut enable_r2 = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--quick" => quick = true,
            "--credit" | "--r2-credit" => credit = true,
            "--enable-r2" => enable_r2 = true,
            "--override-g2-for" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--override-g2-for requires a value (expected: r2)");
                    return ExitCode::from(2);
                }
                if args[i].eq_ignore_ascii_case("r2") {
                    enable_r2 = true;
                } else {
                    eprintln!(
                        "--override-g2-for {} is not supported (only `r2` in this binary)",
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

    if env_override_r2() {
        enable_r2 = true;
    }

    if !enable_r2 {
        eprintln!(
            "R2 is blocked by the v8 G2 kill-gate (FAIL under c1-118207fbc3eaba53).\n\
             \n\
             This is an exploratory post-kill-gate branch. To run R2 you must\n\
             explicitly override the gate:\n\
             \n\
               cargo run -p binn-lab --bin r2 -- --enable-r2 [--quick] [--credit]\n\
               cargo run -p binn-lab --bin r2 -- --override-g2-for r2 [--quick] [--credit]\n\
               BINN_OVERRIDE_G2_FOR=r2 cargo run -p binn-lab --bin r2 -- [--quick] [--credit]\n\
             \n\
             See results/R2_OVERRIDE.md. Default program path (C1 / U-NEG) is unchanged."
        );
        return ExitCode::from(2);
    }

    if credit {
        return run_credit(quick, hash, out);
    }
    run_frozen_r2(quick, hash, out)
}

fn run_frozen_r2(quick: bool, hash: Option<String>, out: Option<PathBuf>) -> ExitCode {
    let mut config = if let Some(h) = hash {
        if h.starts_with(binn_lab::R2_CREDIT_HASH_PREFIX) {
            eprintln!(
                "hash `{h}` is an r2-credit-* preset; pass --credit to run the mitigation probe"
            );
            return ExitCode::from(2);
        }
        match R2Config::from_hash(&h) {
            Some(c) => c,
            None => {
                eprintln!("unknown R2 config hash `{h}` — known presets:");
                for p in R2Config::known_presets() {
                    eprintln!("  {}  (quick={})", p.hash_string(), p.quick);
                }
                return ExitCode::from(2);
            }
        }
    } else if quick {
        R2Config::r2_quick()
    } else {
        R2Config::r2_default()
    };
    config.kill_gate_override = true;

    println!("R2 config hash: {}", config.hash_string());
    println!("protocol version: {}", binn_lab::R2_PROTOCOL_VERSION);
    println!("seeds: {:?}", config.seeds());
    println!("area counts: {:?}", config.area_counts());
    println!("WARNING: kill-gate override active — G2 FAIL (c1-118207fbc3eaba53) still stands");

    let mut runner = R2Runner::new();
    let report = runner.run_r2(&config);
    let md = R2Runner::render_results_markdown(&report, &config);

    let out_path = out.unwrap_or_else(|| {
        let default_name = if config.quick {
            "results/r2_scaling_quick.md"
        } else {
            "results/r2_scaling.md"
        };
        resolve_out(default_name)
    });
    if let Err(code) = write_note(&out_path, &md) {
        return code;
    }

    println!("G4 decision: {}", report.decision.as_str());
    println!("curve shape: {}", report.shape.as_str());
    println!(
        "fit: slope={:.4} intercept={:.4} R²={:.3}",
        report.fit.slope, report.fit.intercept, report.fit.r_squared
    );
    for p in &report.points {
        println!(
            "  n={}: capability={:.4} nnz={:.0}",
            p.n_areas, p.mean_capability, p.mean_nnz
        );
    }
    println!("results note: {}", out_path.display());

    ExitCode::SUCCESS
}

fn run_credit(quick: bool, hash: Option<String>, out: Option<PathBuf>) -> ExitCode {
    let mut config = if let Some(h) = hash {
        if h.starts_with(binn_lab::R2_HASH_PREFIX)
            && !h.starts_with(binn_lab::R2_CREDIT_HASH_PREFIX)
        {
            eprintln!(
                "hash `{h}` is frozen R2 / G4; do not remassage it. Omit --credit for frozen R2, \
                 or pass an r2-credit-* hash."
            );
            return ExitCode::from(2);
        }
        match R2CreditConfig::from_hash(&h) {
            Some(c) => c,
            None => {
                eprintln!("unknown R2-credit config hash `{h}` — known presets:");
                for p in R2CreditConfig::known_presets() {
                    eprintln!("  {}  (quick={})", p.hash_string(), p.quick);
                }
                return ExitCode::from(2);
            }
        }
    } else if quick {
        R2CreditConfig::quick()
    } else {
        R2CreditConfig::scientific()
    };
    config.kill_gate_override = true;

    println!("R2-credit config hash: {}", config.hash_string());
    println!("protocol version: {}", binn_lab::R2_CREDIT_PROTOCOL_VERSION);
    println!("directed arms: {:?}", config.directed_arms());
    println!("seeds: {:?}", config.seeds());
    println!("area counts: {:?}", config.area_counts());
    println!(
        "WARNING: kill-gate override active — does NOT reopen G4 NO-GO (r2-afafa0fa6f43e3fc) \
         or G2 FAIL (c1-118207fbc3eaba53)"
    );

    let mut runner = R2CreditRunner::new();
    let report = runner.run(&config);
    let md = R2CreditRunner::render_results_markdown(&report, &config);

    let out_path = out.unwrap_or_else(|| {
        let default_name = if config.quick {
            "results/r2_credit_scaling_quick.md"
        } else {
            "results/r2_credit_scaling.md"
        };
        resolve_out(default_name)
    });
    if let Err(code) = write_note(&out_path, &md) {
        return code;
    }

    println!("mitigation reading: {}", report.mitigation_reading);
    for curve in &report.directed {
        println!(
            "  arm={}: shape={} slope={:.4} R²={:.3}",
            curve.arm.as_str(),
            curve.shape.as_str(),
            curve.fit.slope,
            curve.fit.r_squared
        );
        for p in &curve.points {
            println!("    n={}: capability={:.4}", p.n_areas, p.mean_capability);
        }
    }
    if let Some(smoke) = &report.pm1_smoke {
        println!(
            "  pm1-smoke: shape={} slope={:.4}",
            smoke.shape.as_str(),
            smoke.fit.slope
        );
    }
    println!("results note: {}", out_path.display());

    ExitCode::SUCCESS
}

fn resolve_out(default_name: &str) -> PathBuf {
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

fn write_note(out_path: &PathBuf, md: &str) -> Result<(), ExitCode> {
    if let Some(parent) = out_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Err(e) = fs::write(out_path, md) {
        eprintln!("failed to write {}: {e}", out_path.display());
        return Err(ExitCode::from(1));
    }
    Ok(())
}

fn env_override_r2() -> bool {
    match env::var("BINN_OVERRIDE_G2_FOR") {
        Ok(v) => v
            .split(',')
            .any(|p| matches!(p.trim().to_ascii_lowercase().as_str(), "r2" | "all")),
        Err(_) => false,
    }
}

fn print_help() {
    eprintln!(
        "Usage: r2 --enable-r2 [--quick] [--credit] [--config-hash HASH] [--out PATH]\n\
         \n\
         Kill-gate override (required; pick one):\n\
           --enable-r2\n\
           --override-g2-for r2\n\
           BINN_OVERRIDE_G2_FOR=r2\n\
         \n\
         Default: frozen R2 / Gate G4 DECISION (U17). Opt-in only.\n\
         Does not reopen protocol-v2 kill-gate c1-118207fbc3eaba53.\n\
         --quick is PILOT only (never a scientific GO/NO-GO).\n\
         \n\
         --credit / --r2-credit: directed-credit mitigation probe (r2-credit-*).\n\
         Same #areas grid; arms = graded DFA + REINFORCE×frozen B (+ optional\n\
         1-seed ±1 smoke). Does NOT reopen frozen G4 NO-GO r2-afafa0fa6f43e3fc."
    );
}
