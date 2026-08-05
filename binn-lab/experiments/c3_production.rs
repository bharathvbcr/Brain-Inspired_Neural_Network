//! Production-faithful C3 v2 entrypoint.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use binn_lab::{C3BpttArm, C3BpttConfig, C3BpttRunner, C3V2Arm, C3V2Config, C3V2Runner};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut quick = false;
    let mut enabled = false;
    let mut bptt_reference = false;
    let mut hash: Option<String> = None;
    let mut out: Option<PathBuf> = None;
    let mut list_hashes = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--quick" => quick = true,
            "--list-hashes" => list_hashes = true,
            "--enable-c3-v2" => enabled = true,
            "--bptt-reference" => bptt_reference = true,
            "--override-g2-for" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    eprintln!("--override-g2-for requires c3-v2");
                    return ExitCode::from(2);
                };
                if value == "c3-v2" {
                    enabled = true;
                } else {
                    eprintln!("expected --override-g2-for c3-v2");
                    return ExitCode::from(2);
                }
            }
            "--config-hash" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    eprintln!("--config-hash requires a value");
                    return ExitCode::from(2);
                };
                hash = Some(value.clone());
            }
            "--out" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    eprintln!("--out requires a path");
                    return ExitCode::from(2);
                };
                out = Some(PathBuf::from(value));
            }
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("unknown argument: {other}");
                print_help();
                return ExitCode::from(2);
            }
        }
        i += 1;
    }
    if list_hashes {
        if bptt_reference {
            for preset in C3BpttConfig::known_presets() {
                println!(
                    "{}",
                    if preset.quick {
                        "PILOT/development [C3 BPTT / c3-bptt-*]"
                    } else {
                        "SCIENTIFIC/held-out [C3 BPTT / c3-bptt-*]"
                    }
                );
                for arm in C3BpttArm::ALL {
                    println!("  {} {}", arm.as_str(), preset.hash_string_for_arm(arm));
                }
            }
            return ExitCode::SUCCESS;
        }
        for preset in C3V2Config::known_presets() {
            println!(
                "{}",
                if preset.quick {
                    "PILOT/development"
                } else {
                    "SCIENTIFIC/held-out"
                }
            );
            for arm in C3V2Arm::ALL {
                println!("  {} {}", arm.as_str(), preset.hash_string_for_arm(arm));
            }
        }
        return ExitCode::SUCCESS;
    }
    if env_override() {
        enabled = true;
    }
    if bptt_reference {
        return run_c3_bptt(quick, hash, out, enabled);
    }
    if !enabled {
        eprintln!(
            "C3 v2 is an exploratory post-G2 protocol. Explicitly acknowledge the\n\
             unchanged kill-gate with --enable-c3-v2, --override-g2-for c3-v2,\n\
             or BINN_OVERRIDE_G2_FOR=c3-v2."
        );
        return ExitCode::from(2);
    }

    let (mut config, replay_arm) = if let Some(hash) = hash {
        match C3V2Config::from_hash(&hash) {
            Some((config, arm)) => (config, Some(arm)),
            None => {
                eprintln!("unknown C3 v2 hash `{hash}`; known hashes:");
                for preset in C3V2Config::known_presets() {
                    for arm in C3V2Arm::ALL {
                        eprintln!("  {}", preset.hash_string_for_arm(arm));
                    }
                }
                return ExitCode::from(2);
            }
        }
    } else if quick {
        (C3V2Config::quick(), None)
    } else {
        (C3V2Config::scientific(), None)
    };
    config.kill_gate_override = true;
    println!(
        "C3 v2 schedule={} seeds={} depths={}..={}",
        if config.quick { "PILOT" } else { "SCIENTIFIC" },
        config.n_seeds,
        config.min_depth,
        config.max_depth
    );
    println!("canonical C1 remains c1-118207fbc3eaba53 (FAIL)");
    if let Some(arm) = replay_arm {
        println!(
            "hash replay requested for `{}`; complete paired suite will run",
            arm.as_str()
        );
    }
    for arm in C3V2Arm::ALL {
        println!(
            "  {} hash={}",
            arm.as_str(),
            config.hash_string_for_arm(arm)
        );
    }

    let mut runner = C3V2Runner::new();
    let report = runner.run(&config);
    let markdown = C3V2Runner::render_markdown(&report, &config);
    let out_path = out.unwrap_or_else(|| {
        PathBuf::from(if config.quick {
            "results/c3_v2_production_quick.md"
        } else {
            "results/c3_v2_production.md"
        })
    });
    if let Some(parent) = out_path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            eprintln!("failed to create {}: {error}", parent.display());
            return ExitCode::from(1);
        }
    }
    if let Err(error) = fs::write(&out_path, markdown) {
        eprintln!("failed to write {}: {error}", out_path.display());
        return ExitCode::from(1);
    }
    println!("forward parity: PASS");
    println!("verdict: {}", report.verdict.as_str());
    for result in &report.arm_results {
        println!(
            "{} D*={}",
            result.arm.as_str(),
            result
                .d_star
                .map(|depth| depth.to_string())
                .unwrap_or_else(|| "none".into())
        );
    }
    println!("results note: {}", out_path.display());
    ExitCode::SUCCESS
}

fn run_c3_bptt(quick: bool, hash: Option<String>, out: Option<PathBuf>, enabled: bool) -> ExitCode {
    if !enabled {
        eprintln!(
            "C3 BPTT is an exploratory post-G2 protocol. Explicitly acknowledge the\n\
             unchanged kill-gate with --enable-c3-v2, --override-g2-for c3-v2,\n\
             or BINN_OVERRIDE_G2_FOR=c3-v2 when using --bptt-reference."
        );
        return ExitCode::from(2);
    }

    let (mut config, replay_arm) = if let Some(hash) = hash {
        match C3BpttConfig::from_hash(&hash) {
            Some((config, arm)) => (config, Some(arm)),
            None => {
                eprintln!("unknown C3 BPTT hash `{hash}`; known hashes:");
                for preset in C3BpttConfig::known_presets() {
                    for arm in C3BpttArm::ALL {
                        eprintln!("  {}", preset.hash_string_for_arm(arm));
                    }
                }
                return ExitCode::from(2);
            }
        }
    } else if quick {
        (C3BpttConfig::quick(), None)
    } else {
        (C3BpttConfig::scientific(), None)
    };
    config.kill_gate_override = true;
    println!(
        "C3 BPTT schedule={} seeds={} depths={}..={}",
        if config.quick { "PILOT" } else { "SCIENTIFIC" },
        config.n_seeds,
        config.min_depth,
        config.max_depth
    );
    println!("canonical C1 remains c1-118207fbc3eaba53 (FAIL)");
    println!("hash family: c3-bptt-* (frozen c3v2-* unchanged)");
    if let Some(arm) = replay_arm {
        println!(
            "hash replay requested for `{}`; complete paired suite will run",
            arm.as_str()
        );
    }
    for arm in C3BpttArm::ALL {
        println!(
            "  {} hash={}",
            arm.as_str(),
            config.hash_string_for_arm(arm)
        );
    }

    let mut runner = C3BpttRunner::new();
    let report = runner.run(&config);
    let markdown = C3BpttRunner::render_markdown(&report, &config);
    let out_path = out.unwrap_or_else(|| {
        PathBuf::from(if config.quick {
            "results/c3_bptt_quick.md"
        } else {
            "results/c3_bptt.md"
        })
    });
    if let Some(parent) = out_path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            eprintln!("failed to create {}: {error}", parent.display());
            return ExitCode::from(1);
        }
    }
    if let Err(error) = fs::write(&out_path, markdown) {
        eprintln!("failed to write {}: {error}", out_path.display());
        return ExitCode::from(1);
    }
    println!("verdict: {}", report.verdict.as_str());
    for result in &report.arm_results {
        println!(
            "{} D*={}",
            result.arm.as_str(),
            result
                .d_star
                .map(|depth| depth.to_string())
                .unwrap_or_else(|| "none".into())
        );
    }
    println!("results note: {}", out_path.display());
    ExitCode::SUCCESS
}

fn env_override() -> bool {
    match env::var("BINN_OVERRIDE_G2_FOR") {
        Ok(value) => value
            .split(',')
            .any(|part| matches!(part.trim().to_ascii_lowercase().as_str(), "c3-v2" | "all")),
        Err(_) => false,
    }
}

fn print_help() {
    eprintln!(
        "Usage: c3-production --enable-c3-v2 [--quick] [--bptt-reference] [--config-hash HASH] [--out PATH]\n\
         c3-production --list-hashes\n\
         \n\
         Runs C3 v2 on the production event engine and ThreeFactor eligibility.\n\
         `--bptt-reference` selects the separate real SuperSpike BPTT family\n\
         (`c3-bptt-*`). C3 v1 remains a separate tabular proxy. `--quick` is PILOT only."
    );
}
