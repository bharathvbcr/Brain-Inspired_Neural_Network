//! Exact-forward C1 credit-assignment repreregistration entrypoint.
//!
//! This is separate from the canonical `c1` binary and cannot mutate or
//! reinterpret protocol-v2 hash `c1-118207fbc3eaba53`.
//!
//! `--isolation` selects the `c1x-iso-*` trial-isolation family (new hashes);
//! `--isolation-calibrated` selects sparsity-calibrated `c1x-iso-s-*`.
//! Frozen non-isolated `c1x-*` remain byte-stable.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use binn_lab::{
    CreditArm, CreditConfig, CreditRunner, DfaSpikeArm, DfaSpikeConfig, DfaSpikeRunner,
    EpropTrueArm, EpropTrueConfig, EpropTrueRunner, DFA_SPIKE_PROTOCOL_VERSION,
    EPROP_TRUE_PROTOCOL_VERSION,
};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut quick = false;
    let mut isolation = false;
    let mut isolation_calibrated = false;
    let mut true_eprop = false;
    let mut dfa_spike = false;
    let mut hash: Option<String> = None;
    let mut out: Option<PathBuf> = None;
    let mut list_hashes = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--quick" => quick = true,
            "--isolation" => isolation = true,
            "--isolation-calibrated" | "--iso-s" => isolation_calibrated = true,
            "--true-eprop" => true_eprop = true,
            "--dfa-spike" => dfa_spike = true,
            "--list-hashes" => list_hashes = true,
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
    if isolation && isolation_calibrated {
        eprintln!("use either --isolation or --isolation-calibrated, not both");
        return ExitCode::from(2);
    }
    if true_eprop && dfa_spike {
        eprintln!("use either --true-eprop or --dfa-spike, not both");
        return ExitCode::from(2);
    }
    if (true_eprop || dfa_spike) && (isolation || isolation_calibrated) {
        eprintln!(
            "--true-eprop / --dfa-spike cannot combine with --isolation or --isolation-calibrated"
        );
        return ExitCode::from(2);
    }
    if list_hashes {
        if true_eprop {
            for preset in EpropTrueConfig::known_presets() {
                println!(
                    "{}",
                    if preset.quick {
                        "PILOT/development [true surrogate e-prop / c1x-eprop-true-*]"
                    } else {
                        "SCIENTIFIC/held-out [true surrogate e-prop / c1x-eprop-true-*]"
                    }
                );
                for arm in EpropTrueArm::ALL {
                    println!(
                        "  {} protocol={} {}",
                        arm.as_str(),
                        EPROP_TRUE_PROTOCOL_VERSION,
                        preset.hash_string_for_arm(arm)
                    );
                }
            }
            return ExitCode::SUCCESS;
        }
        if dfa_spike {
            for preset in DfaSpikeConfig::known_presets() {
                println!(
                    "{}",
                    if preset.quick {
                        "PILOT/development [spiking DFA rescue / c1x-dfa-spike-*]"
                    } else {
                        "SCIENTIFIC/held-out [spiking DFA rescue / c1x-dfa-spike-*]"
                    }
                );
                for arm in DfaSpikeArm::ALL {
                    println!(
                        "  {} protocol={} {}",
                        arm.as_str(),
                        DFA_SPIKE_PROTOCOL_VERSION,
                        preset.hash_string_for_arm(arm)
                    );
                }
            }
            return ExitCode::SUCCESS;
        }
        for preset in CreditConfig::known_presets() {
            let family = if preset.is_isolation_calibrated_protocol() {
                " [sparsity-calibrated isolation / c1x-iso-s-*]"
            } else if preset.is_isolation_protocol() {
                " [trial-isolation / c1x-iso-*]"
            } else {
                ""
            };
            println!(
                "{}{}",
                if preset.quick {
                    "PILOT/development"
                } else {
                    "SCIENTIFIC/held-out"
                },
                family
            );
            for arm in CreditArm::ALL {
                println!(
                    "  {} protocol={} {}",
                    arm.as_str(),
                    preset.protocol_version_for(arm),
                    preset.hash_string_for_arm(arm)
                );
            }
        }
        return ExitCode::SUCCESS;
    }

    if true_eprop {
        return run_true_eprop(quick, hash, out);
    }
    if dfa_spike {
        return run_dfa_spike(quick, hash, out);
    }

    let (config, replay_arm) = if let Some(hash) = hash {
        match CreditConfig::from_hash(&hash) {
            Some((config, arm)) => (config, Some(arm)),
            None => {
                eprintln!("unknown credit-assignment hash `{hash}`; known hashes:");
                for preset in CreditConfig::known_presets() {
                    for arm in CreditArm::ALL {
                        eprintln!("  {}", preset.hash_string_for_arm(arm));
                    }
                }
                return ExitCode::from(2);
            }
        }
    } else if isolation_calibrated && quick {
        (CreditConfig::quick_isolation_calibrated(), None)
    } else if isolation_calibrated {
        (CreditConfig::scientific_isolation_calibrated(), None)
    } else if isolation && quick {
        (CreditConfig::quick_isolation(), None)
    } else if isolation {
        (CreditConfig::scientific_isolation(), None)
    } else if quick {
        (CreditConfig::quick(), None)
    } else {
        (CreditConfig::scientific(), None)
    };

    println!(
        "schedule={} seeds={} matched_epochs={} trial_isolation={} kwta_all_finite={}",
        if config.quick { "PILOT" } else { "SCIENTIFIC" },
        config.base.n_seeds,
        config.matched_epochs,
        config.is_isolation_protocol(),
        config.kwta_all_finite
    );
    println!("canonical C1 remains c1-118207fbc3eaba53 (protocol v2)");
    if config.is_isolation_calibrated_protocol() {
        println!("hash family: c1x-iso-s-* (frozen c1x-* / prior c1x-iso-* unchanged)");
    } else if config.is_isolation_protocol() {
        println!("hash family: c1x-iso-* (frozen c1x-* unchanged)");
    } else {
        println!(
            "hash family: c1x-* (non-isolated; sticky last_spike / incomplete membrane reset)"
        );
    }
    if let Some(arm) = replay_arm {
        println!(
            "config-hash replay requested for arm `{}`; the full paired suite is rerun",
            arm.as_str()
        );
    }
    for arm in CreditArm::ALL {
        println!(
            "  {} protocol={} hash={}",
            arm.as_str(),
            config.protocol_version_for(arm),
            config.hash_string_for_arm(arm)
        );
    }

    let mut runner = CreditRunner::new();
    let report = runner.run(&config);
    let markdown = CreditRunner::render_markdown(&report, &config);
    let out_path = out.unwrap_or_else(|| {
        PathBuf::from(if config.is_isolation_calibrated_protocol() {
            if config.quick {
                "results/credit_assignment_iso_s_quick.md"
            } else {
                "results/credit_assignment_iso_s.md"
            }
        } else if config.is_isolation_protocol() {
            if config.quick {
                "results/credit_assignment_iso_quick.md"
            } else {
                "results/credit_assignment_iso.md"
            }
        } else if config.quick {
            "results/credit_assignment_quick.md"
        } else {
            "results/credit_assignment.md"
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

    println!(
        "forward parity: {}",
        if report.parity.all_pass() {
            "PASS"
        } else {
            "FAIL"
        }
    );
    println!(
        "activity_sparsity={:.4} positive_control={:.4}",
        report.mean_activity_sparsity, report.positive_control_mean
    );
    for summary in &report.summaries {
        println!(
            "{} accuracy={:.4} gap_lcb={:.4} verdict={}",
            summary.arm.as_str(),
            summary.mean_accuracy,
            summary.gap_closed_lower_95,
            summary.verdict.as_str()
        );
    }
    println!("results note: {}", out_path.display());
    ExitCode::SUCCESS
}

fn run_true_eprop(quick: bool, hash: Option<String>, out: Option<PathBuf>) -> ExitCode {
    let (config, replay_arm) = if let Some(hash) = hash {
        match EpropTrueConfig::from_hash(&hash) {
            Some((config, arm)) => (config, Some(arm)),
            None => {
                eprintln!("unknown true-eprop hash `{hash}`; known hashes:");
                for preset in EpropTrueConfig::known_presets() {
                    for arm in EpropTrueArm::ALL {
                        eprintln!("  {}", preset.hash_string_for_arm(arm));
                    }
                }
                return ExitCode::from(2);
            }
        }
    } else if quick {
        (EpropTrueConfig::quick(), None)
    } else {
        (EpropTrueConfig::scientific(), None)
    };

    println!(
        "schedule={} seeds={} matched_epochs={} protocol=true-surrogate-eprop",
        if config.quick { "PILOT" } else { "SCIENTIFIC" },
        config.base.n_seeds,
        config.matched_epochs
    );
    println!("canonical C1 remains c1-118207fbc3eaba53 (protocol v2)");
    println!("hash family: c1x-eprop-true-* (frozen c1x-eprop-exact-forward-* unchanged)");
    if let Some(arm) = replay_arm {
        println!(
            "config-hash replay requested for arm `{}`; full paired suite reruns",
            arm.as_str()
        );
    }
    for arm in EpropTrueArm::ALL {
        println!(
            "  {} hash={}",
            arm.as_str(),
            config.hash_string_for_arm(arm)
        );
    }

    let mut runner = EpropTrueRunner::new();
    let report = runner.run(&config);
    let markdown = EpropTrueRunner::render_markdown(&report, &config);
    let out_path = out.unwrap_or_else(|| {
        PathBuf::from(if config.quick {
            "results/credit_assignment_eprop_true_quick.md"
        } else {
            "results/credit_assignment_eprop_true.md"
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

    for summary in &report.summaries {
        println!(
            "{} accuracy={:.4}",
            summary.arm.as_str(),
            summary.mean_accuracy
        );
    }
    println!("results note: {}", out_path.display());
    ExitCode::SUCCESS
}

fn run_dfa_spike(quick: bool, hash: Option<String>, out: Option<PathBuf>) -> ExitCode {
    let (config, replay_arm) = if let Some(hash) = hash {
        match DfaSpikeConfig::from_hash(&hash) {
            Some((config, arm)) => (config, Some(arm)),
            None => {
                eprintln!("unknown dfa-spike hash `{hash}`; known hashes:");
                for preset in DfaSpikeConfig::known_presets() {
                    for arm in DfaSpikeArm::ALL {
                        eprintln!("  {}", preset.hash_string_for_arm(arm));
                    }
                }
                return ExitCode::from(2);
            }
        }
    } else if quick {
        (DfaSpikeConfig::quick(), None)
    } else {
        (DfaSpikeConfig::scientific(), None)
    };

    println!(
        "schedule={} seeds={} matched_epochs={} protocol=spiking-dfa-rescue burst={}×{}",
        if config.quick { "PILOT" } else { "SCIENTIFIC" },
        config.base.n_seeds,
        config.matched_epochs,
        config.burst_count,
        config.burst_stride
    );
    println!("canonical C1 remains c1-118207fbc3eaba53 (protocol v2)");
    println!("hash family: c1x-dfa-spike-* (frozen c1x-dfa-exact-forward-* / c1-dfa-* unchanged)");
    if let Some(arm) = replay_arm {
        println!(
            "config-hash replay requested for arm `{}`; full paired suite reruns",
            arm.as_str()
        );
    }
    for arm in DfaSpikeArm::ALL {
        println!(
            "  {} hash={}",
            arm.as_str(),
            config.hash_string_for_arm(arm)
        );
    }

    let mut runner = DfaSpikeRunner::new();
    let report = runner.run(&config);
    let markdown = DfaSpikeRunner::render_markdown(&report, &config);
    let out_path = out.unwrap_or_else(|| {
        PathBuf::from(if config.quick {
            "results/credit_dfa_spike_quick.md"
        } else {
            "results/credit_dfa_spike.md"
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

    println!(
        "verdict={} true_dfa={:.4} gap_lcb={:.4} surrogate={:.4} hybrid={:.4}",
        report.verdict.as_str(),
        report.mean_true_dfa,
        report.gap_closed_dfa_lower_95,
        report.mean_surrogate_gradient,
        report.mean_hybrid_stdp_dfa
    );
    println!(
        "activity_sparsity={:.4} positive_control={:.4}",
        report.mean_activity_sparsity, report.positive_control_mean
    );
    for summary in &report.summaries {
        println!(
            "{} accuracy={:.4}",
            summary.arm.as_str(),
            summary.mean_accuracy
        );
    }
    println!("results note: {}", out_path.display());
    ExitCode::SUCCESS
}

fn print_help() {
    eprintln!(
        "Usage: credit-assignment [--quick] [--true-eprop | --dfa-spike] \
         [--isolation | --isolation-calibrated] [--config-hash HASH] [--out PATH]\n\
         credit-assignment --list-hashes\n\
         credit-assignment --true-eprop --list-hashes\n\
         credit-assignment --dfa-spike --list-hashes\n\
         \n\
         Runs the exact-forward matched C1 credit suite. `--true-eprop` selects the\n\
         separate true-surrogate e-prop family (`c1x-eprop-true-*`). `--dfa-spike`\n\
         selects the spiking-path DFA rescue family (`c1x-dfa-spike-*`). `--quick` is a\n\
         development-only PILOT. `--isolation` selects the `c1x-iso-*` trial-\n\
         isolation family (clear last_spike + full membrane reset; new hashes).\n\
         `--isolation-calibrated` / `--iso-s` selects sparsity-calibrated\n\
         `c1x-iso-s-*` (same isolation + winner-floor k-WTA; G2 thresholds\n\
         unchanged). Any arm hash replays its complete paired preset so matched\n\
         comparisons remain intact. Canonical C1 protocol-v2 and\n\
         c1-118207fbc3eaba53 are never modified; frozen non-isolated c1x-* and\n\
         prior c1x-iso-* hashes stay byte-stable."
    );
}
