//! C1 experiment harness entry (U13) — Gate G2.
//!
//! One command reproduces C1 from a config hash:
//!
//! ```bash
//! cargo run -p binn-lab --bin c1 -- --quick
//! cargo run -p binn-lab --bin c1 -- --config-hash c1-<hex>
//! cargo run -p binn-lab --bin c1 -- --isolate-condition local-assembly --seed 1 --quick
//! ```

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use binn_lab::{ConditionLabel, Config, Runner};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut quick = false;
    let mut hash: Option<String> = None;
    let mut out: Option<PathBuf> = None;
    let mut isolate_condition: Option<String> = None;
    let mut isolate_seed: Option<u64> = None;
    let mut match_nnz: Option<usize> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--quick" => quick = true,
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
            "--isolate-condition" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--isolate-condition requires a value");
                    return ExitCode::from(2);
                }
                isolate_condition = Some(args[i].clone());
            }
            "--seed" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--seed requires a value");
                    return ExitCode::from(2);
                }
                match args[i].parse::<u64>() {
                    Ok(s) => isolate_seed = Some(s),
                    Err(_) => {
                        eprintln!("--seed must be an integer");
                        return ExitCode::from(2);
                    }
                }
            }
            "--match-nnz" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--match-nnz requires a value");
                    return ExitCode::from(2);
                }
                match args[i].parse::<usize>() {
                    Ok(n) => match_nnz = Some(n),
                    Err(_) => {
                        eprintln!("--match-nnz must be an integer");
                        return ExitCode::from(2);
                    }
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

    let config = if let Some(h) = hash {
        match Config::from_hash(&h) {
            Some(c) => c,
            None => {
                eprintln!(
                    "unknown config hash `{h}` — known presets: {}",
                    Config::c1_default().hash_string()
                );
                eprintln!(
                    "tip: use --quick for the CI preset ({})",
                    Config::c1_quick().hash_string()
                );
                return ExitCode::from(2);
            }
        }
    } else if quick {
        Config::c1_quick()
    } else {
        Config::c1_default()
    };

    if let Some(cond_s) = isolate_condition {
        let Some(label) = ConditionLabel::parse(&cond_s) else {
            eprintln!("unknown condition `{cond_s}`");
            return ExitCode::from(2);
        };
        let seed = isolate_seed.unwrap_or_else(|| {
            config
                .seeds()
                .into_iter()
                .next()
                .expect("config has ≥1 seed")
        });
        println!(
            "{}",
            Runner::condition_json(&config, seed, label, match_nnz)
        );
        return ExitCode::SUCCESS;
    }

    println!("C1 config hash: {}", config.hash_string());
    println!("seeds: {:?}", config.seeds());

    let mut runner = Runner::new();
    let report = runner.run_c1(&config);
    let md = Runner::render_results_markdown(&report, &config);

    let out_path = out.unwrap_or_else(|| {
        let candidates = [
            PathBuf::from("results/c1_g2.md"),
            PathBuf::from("binn/results/c1_g2.md"),
            PathBuf::from("binn-lab/results/c1_g2.md"),
        ];
        candidates
            .into_iter()
            .find(|p| p.parent().map(|d| d.exists()).unwrap_or(false))
            .unwrap_or_else(|| PathBuf::from("results/c1_g2.md"))
    });
    if let Some(parent) = out_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Err(e) = fs::write(&out_path, &md) {
        eprintln!("failed to write {}: {e}", out_path.display());
        return ExitCode::from(1);
    }

    println!("G2 verdict: {}", report.verdict.as_str());
    println!(
        "means: local={:.4} dense-local={:.4} gradient-reference={:.4} eligibility-reference={:.4}",
        report.summary.mean_local,
        report.summary.mean_dense,
        report.summary.mean_gradient_reference,
        report.summary.mean_eligibility_reference
    );
    println!(
        "normalized-gap-closed={:.4}  lower-95={:.4}  |local-dense|={:.4}",
        report.summary.mean_gap_closed,
        report.summary.gap_closed_lower_95,
        report.summary.mean_dist_to_dense
    );
    println!(
        "positive_control={:.4}  activity_sparsity={:.4}  required_n_seeds={}",
        report.positive_control_mean,
        report.mean_activity_sparsity,
        report.required_scientific_n_seeds
    );
    println!("results note: {}", out_path.display());

    ExitCode::SUCCESS
}

fn print_help() {
    eprintln!(
        "Usage: c1 [--quick] [--config-hash HASH] [--out PATH]\n\
         \n\
         Isolate one condition (peak-RSS child):\n\
           c1 --isolate-condition LABEL --seed N [--config-hash HASH] [--match-nnz N]\n\
         \n\
         Reproduces experiment C1 (Gate G2): local-assembly vs labeled\n\
         gradient / eligibility references and dense-local control.\n\
         --quick is PILOT only (never a scientific PASS/FAIL)."
    );
}
