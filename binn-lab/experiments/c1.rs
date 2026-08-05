//! C1 experiment harness entry (U13) — Gate G2.
//!
//! One command reproduces C1 from a config hash:
//!
//! ```bash
//! cargo run -p binn-lab --bin c1 -- --quick
//! cargo run -p binn-lab --bin c1 -- --config-hash c1-<hex>
//! cargo run -p binn-lab --bin c1 -- --isolate-condition local-assembly --seed 1 --quick
//! cargo run -p binn-lab --bin c1 -- --sensitivity temporal-pc --quick
//! cargo run -p binn-lab --bin c1 -- --sensitivity capacity --quick
//! cargo run -p binn-lab --bin c1 -- --isolation --quick
//! cargo run -p binn-lab --bin c1 -- --isolation --out results/c1_iso.md
//! cargo run -p binn-lab --bin c1 -- --matched-arch --quick
//! cargo run -p binn-lab --bin c1 -- --matched-arch --out results/c1_match.md
//! cargo run -p binn-lab --bin c1 -- --matched-dfa --quick
//! cargo run -p binn-lab --bin c1 -- --matched-dfa --out results/c1_dfa.md
//! cargo run -p binn-lab --bin c1 -- --matched-rl --quick
//! cargo run -p binn-lab --bin c1 -- --matched-rl --out results/c1_rl.md
//! cargo run -p binn-lab --bin c1 -- --matched-mech --quick
//! cargo run -p binn-lab --bin c1 -- --matched-mech --out results/c1_credit_mech.md
//! cargo run -p binn-lab --bin c1 -- --eventprop --quick
//! cargo run -p binn-lab --bin c1 -- --eventprop --out results/c1_eventprop.md
//! cargo run -p binn-lab --bin c1 -- --shd-cal --quick
//! cargo run -p binn-lab --bin c1 -- --shd-cal --out results/c1_shd.md
//! cargo run -p binn-lab --bin c1 -- --reinforce-fb --quick
//! cargo run -p binn-lab --bin c1 -- --reinforce-fb --out results/c1_rfb.md
//! cargo run -p binn-lab --bin c1 -- --reinforce-fb --config-hash c1-660401d74db3c88d
//! cargo run -p binn-lab --bin c1 -- --rfb-epoch --out results/c1_rfb_em.md
//! cargo run -p binn-lab --bin c1 -- --structured-fb --out results/c1_sfb.md
//! cargo run -p binn-lab --bin c1 -- --structured-fb-epoch --out results/c1_sfb_em.md
//! cargo run -p binn-lab --bin c1 -- --structured-fb-capacity --out results/c1_sfb_cap.md
//! cargo run -p binn-lab --bin c1 -- --elig-rfb --out results/c1_elig_rfb.md
//! cargo run -p binn-lab --bin c1 -- --structured-fb-teach --out results/c1_sfb_teach.md
//! ```

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use binn_lab::{
    scaled_k_wta, ConditionLabel, Config, DfaMatchConfig, DfaMatchRunner, EventPropMatchConfig,
    EventPropMatchRunner, MacProbeConfig, MacProbeMode, MatchConfig, MatchRunner, MechConfig,
    MechRunner, RlMatchConfig, RlMatchRunner, Runner, ShdCalConfig, ShdCalRunner,
    DFA_LIVE_SIZE_ACC_FLOOR, DFA_LIVE_SIZE_GAP_LCB_CLEAR, FOUNDATION_MICRO_NNZ_HI,
    FOUNDATION_MICRO_NNZ_LO, FOUNDATION_MICRO_TARGET_NNZ, FOUNDATION_MICRO_WALL_SECS_PER_SEED,
    MAC_PROBE_FULL_C1_REFUSE_N, MAC_PROBE_K_WTA, MICRO_MAX_FAN_OUT,
};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut quick = false;
    let mut hash: Option<String> = None;
    let mut out: Option<PathBuf> = None;
    let mut isolate_condition: Option<String> = None;
    let mut isolate_seed: Option<u64> = None;
    let mut match_nnz: Option<usize> = None;
    let mut sensitivity: Option<String> = None;
    let mut matched_arch = false;
    let mut matched_dfa = false;
    let mut matched_rl = false;
    let mut matched_mech = false;
    let mut matched_eventprop = false;
    let mut shd_cal = false;
    let mut shd_full = false;
    let mut shd_smoke = false;
    let mut isolation = false;
    let mut spike = false;
    let mut spike_s = false;
    let mut project = false;
    let mut reinforce_fb = false;
    let mut rfb_learned = false;
    let mut k_anneal = false;
    let mut rfb_epoch = false;
    let mut structured_fb = false;
    let mut structured_fb_epoch = false;
    let mut structured_fb_capacity = false;
    let mut elig_rfb = false;
    let mut structured_fb_teach = false;
    let mut dfa_live = false;
    let mut structured_fb_soft = false;
    let mut structured_fb_finth = false;
    let mut structured_fb_cont = false;
    let mut match_undertrain = false;
    let mut export_trace: Option<PathBuf> = None;
    let mut mac_probe = false;
    let mut micro_isolate = false;
    let mut foundation_micro = false;
    let mut dfa_live_size = false;
    let mut syn_matched = false;
    let mut mac_n_hidden: Option<usize> = None;
    let mut mac_max_fan_out: Option<usize> = None;
    let mut mac_k_wta: Option<usize> = None;
    let mut mac_mode = MacProbeMode::Pm1;
    let mut mac_mode_set = false;
    let mut shd_hidden: Option<usize> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--quick" => quick = true,
            "--matched-arch" => matched_arch = true,
            "--matched-dfa" | "--matched-arch-dfa" => matched_dfa = true,
            "--matched-rl" | "--matched-arch-rl" => matched_rl = true,
            "--matched-mech" | "--mech" | "--credit-mech" => matched_mech = true,
            "--eventprop" | "--matched-eventprop" | "--matched-arch-eventprop" => {
                matched_eventprop = true
            }
            "--shd-cal" | "--shd" => shd_cal = true,
            "--shd-full" | "--shd-corpus" => {
                shd_cal = true;
                shd_full = true;
            }
            "--smoke" | "--shd-smoke" => shd_smoke = true,
            "--isolation" | "--iso" => isolation = true,
            "--spike" | "--natural-spike" => spike = true,
            "--spike-s" | "--calibrated-spike" => spike_s = true,
            "--project" | "--ac-project" => project = true,
            "--reinforce-fb" | "--rfb" | "--live-reinforce-fb" => reinforce_fb = true,
            "--rfb-learned" | "--reinforce-fb-learned" => rfb_learned = true,
            "--k-anneal" | "--k-wta-anneal" => k_anneal = true,
            "--rfb-epoch" | "--reinforce-fb-epoch" | "--live-rfb-epoch" => rfb_epoch = true,
            "--structured-fb" | "--sfb" | "--structured-reinforce-fb" => structured_fb = true,
            "--structured-fb-epoch" | "--sfb-epoch" | "--sfb-em" => structured_fb_epoch = true,
            "--structured-fb-capacity" | "--sfb-capacity" | "--sfb-cap" => {
                structured_fb_capacity = true
            }
            "--elig-rfb" | "--eligibility-rfb" | "--elig-reinforce" => elig_rfb = true,
            "--structured-fb-teach" | "--sfb-teach" | "--structured-teach" => {
                structured_fb_teach = true
            }
            "--dfa-live" | "--live-dfa" | "--graded-dfa-live" => dfa_live = true,
            "--structured-fb-soft" | "--sfb-soft" | "--soft-wta-sfb" => structured_fb_soft = true,
            "--structured-fb-finth" | "--sfb-finth" | "--finite-theta-sfb" => {
                structured_fb_finth = true
            }
            "--structured-fb-cont" | "--sfb-cont" | "--continuous-sfb" => structured_fb_cont = true,
            "--match-undertrain" | "--matched-undertrain" | "--match-ep4" => {
                match_undertrain = true
            }
            "--mac-probe" | "--mac" => mac_probe = true,
            "--micro" | "--mac-micro" | "--micro-isolate" => {
                mac_probe = true;
                micro_isolate = true;
            }
            "--foundation-micro" | "--micro-foundation" | "--foundation-microcircuit" => {
                mac_probe = true;
                foundation_micro = true;
            }
            "--dfa-live-size" | "--mac-size" | "--h2-size" => {
                mac_probe = true;
                dfa_live_size = true;
            }
            "--syn-matched" | "--syn-match" => {
                mac_probe = true;
                syn_matched = true;
            }
            "--n-hidden" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--n-hidden requires a value");
                    return ExitCode::from(2);
                }
                match args[i].parse::<usize>() {
                    Ok(n) if n > 0 => mac_n_hidden = Some(n),
                    _ => {
                        eprintln!("--n-hidden must be a positive integer");
                        return ExitCode::from(2);
                    }
                }
            }
            "--shd-hidden" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--shd-hidden requires a value");
                    return ExitCode::from(2);
                }
                match args[i].parse::<usize>() {
                    Ok(n) if n > 0 => shd_hidden = Some(n),
                    _ => {
                        eprintln!("--shd-hidden must be a positive integer");
                        return ExitCode::from(2);
                    }
                }
            }
            "--max-fan-out" | "--fan-out" | "--fan" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--max-fan-out requires a value");
                    return ExitCode::from(2);
                }
                match args[i].parse::<usize>() {
                    Ok(n) if n > 0 => mac_max_fan_out = Some(n),
                    _ => {
                        eprintln!("--max-fan-out must be a positive integer");
                        return ExitCode::from(2);
                    }
                }
            }
            "--k-wta" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--k-wta requires a value");
                    return ExitCode::from(2);
                }
                match args[i].parse::<usize>() {
                    Ok(n) if n > 0 => mac_k_wta = Some(n),
                    _ => {
                        eprintln!("--k-wta must be a positive integer");
                        return ExitCode::from(2);
                    }
                }
            }
            "--mac-mode" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--mac-mode requires pm1|structured-fb|dfa-live");
                    return ExitCode::from(2);
                }
                match MacProbeMode::parse(&args[i]) {
                    Some(m) => {
                        mac_mode = m;
                        mac_mode_set = true;
                    }
                    None => {
                        eprintln!("--mac-mode must be pm1|structured-fb|dfa-live");
                        return ExitCode::from(2);
                    }
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
            "--sensitivity" | "--capacity" => {
                // `--capacity` is a shorthand for `--sensitivity capacity`.
                let name = if args[i] == "--capacity" {
                    "capacity".to_string()
                } else {
                    i += 1;
                    if i >= args.len() {
                        eprintln!("--sensitivity requires temporal-pc|capacity");
                        return ExitCode::from(2);
                    }
                    args[i].clone()
                };
                sensitivity = Some(name);
            }
            "--replay" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("--replay requires an output path (e.g. results/c1_replay.json)");
                    return ExitCode::from(2);
                }
                // Consumed by the runner (and inherited by isolated condition
                // children) via BINN_REPLAY_OUT; viz only, no effect on results.
                env::set_var(binn_lab::REPLAY_OUT_ENV, &args[i]);
            }
            "--export-trace" => {
                // Optional path: default results/c1_trace.jsonl when flag alone.
                let path = if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    i += 1;
                    PathBuf::from(&args[i])
                } else {
                    PathBuf::from("results/c1_trace.jsonl")
                };
                export_trace = Some(path);
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

    if matched_arch && matched_dfa {
        eprintln!("use either --matched-arch or --matched-dfa, not both");
        return ExitCode::from(2);
    }
    if matched_arch && matched_rl {
        eprintln!("use either --matched-arch or --matched-rl, not both");
        return ExitCode::from(2);
    }
    if matched_dfa && matched_rl {
        eprintln!("use either --matched-dfa or --matched-rl, not both");
        return ExitCode::from(2);
    }
    let exclusive_match_flags = u8::from(matched_arch)
        + u8::from(matched_dfa)
        + u8::from(matched_rl)
        + u8::from(matched_mech)
        + u8::from(matched_eventprop)
        + u8::from(shd_cal);
    if exclusive_match_flags > 1 {
        eprintln!(
            "use only one of --matched-arch / --matched-dfa / --matched-rl / --matched-mech / --eventprop / --shd-cal"
        );
        return ExitCode::from(2);
    }
    if shd_hidden.is_some() && !shd_cal {
        eprintln!("--shd-hidden requires --shd-cal");
        return ExitCode::from(2);
    }

    if matched_arch {
        return run_matched_arch(
            quick,
            hash.as_deref(),
            out,
            sensitivity.is_some()
                || isolation
                || spike
                || spike_s
                || project
                || reinforce_fb
                || rfb_epoch
                || structured_fb
                || structured_fb_epoch
                || structured_fb_capacity
                || elig_rfb
                || structured_fb_teach
                || dfa_live
                || structured_fb_soft
                || structured_fb_finth
                || structured_fb_cont,
            match_undertrain,
        );
    }
    if matched_dfa {
        return run_matched_dfa(
            quick,
            hash.as_deref(),
            out,
            sensitivity.is_some()
                || isolation
                || spike
                || spike_s
                || project
                || reinforce_fb
                || rfb_epoch
                || structured_fb
                || structured_fb_epoch
                || structured_fb_capacity
                || elig_rfb
                || structured_fb_teach
                || dfa_live
                || structured_fb_soft
                || structured_fb_finth
                || structured_fb_cont,
        );
    }
    if matched_rl {
        return run_matched_rl(
            quick,
            hash.as_deref(),
            out,
            sensitivity.is_some()
                || isolation
                || spike
                || spike_s
                || project
                || reinforce_fb
                || rfb_epoch
                || structured_fb
                || structured_fb_epoch
                || structured_fb_capacity
                || elig_rfb
                || structured_fb_teach
                || dfa_live
                || structured_fb_soft
                || structured_fb_finth
                || structured_fb_cont,
        );
    }
    if matched_mech {
        return run_matched_mech(
            quick,
            hash.as_deref(),
            out,
            sensitivity.is_some()
                || isolation
                || spike
                || spike_s
                || project
                || reinforce_fb
                || rfb_epoch
                || structured_fb
                || structured_fb_epoch
                || structured_fb_capacity
                || elig_rfb
                || structured_fb_teach
                || dfa_live
                || structured_fb_soft
                || structured_fb_finth
                || structured_fb_cont
                || match_undertrain,
        );
    }
    if matched_eventprop {
        return run_matched_eventprop(
            quick,
            hash.as_deref(),
            out,
            sensitivity.is_some()
                || isolation
                || spike
                || spike_s
                || project
                || reinforce_fb
                || rfb_epoch
                || structured_fb
                || structured_fb_epoch
                || structured_fb_capacity
                || elig_rfb
                || structured_fb_teach
                || dfa_live
                || structured_fb_soft
                || structured_fb_finth
                || structured_fb_cont
                || match_undertrain,
        );
    }
    if shd_cal {
        return run_shd_cal(
            quick,
            hash.as_deref(),
            out,
            shd_hidden,
            shd_full,
            shd_smoke,
            sensitivity.is_some()
                || isolation
                || spike
                || spike_s
                || project
                || reinforce_fb
                || rfb_epoch
                || structured_fb
                || structured_fb_epoch
                || structured_fb_capacity
                || elig_rfb
                || structured_fb_teach
                || dfa_live
                || structured_fb_soft
                || structured_fb_finth
                || structured_fb_cont
                || match_undertrain
                || matched_arch
                || matched_dfa
                || matched_rl
                || matched_mech
                || matched_eventprop,
        );
    }

    // Protocol flags may combine with --config-hash (matched-* style): the hash
    // must belong to that protocol family. Bare --config-hash alone still works.
    if sensitivity.is_some() && isolation {
        eprintln!("use either --sensitivity or --isolation, not both");
        return ExitCode::from(2);
    }
    if sensitivity.is_some() && spike {
        eprintln!("use either --sensitivity or --spike, not both");
        return ExitCode::from(2);
    }
    if sensitivity.is_some() && spike_s {
        eprintln!("use either --sensitivity or --spike-s, not both");
        return ExitCode::from(2);
    }
    if sensitivity.is_some() && project {
        eprintln!("use either --sensitivity or --project, not both");
        return ExitCode::from(2);
    }
    if sensitivity.is_some() && reinforce_fb {
        eprintln!("use either --sensitivity or --reinforce-fb, not both");
        return ExitCode::from(2);
    }
    if isolation && spike {
        eprintln!("use either --isolation or --spike, not both");
        return ExitCode::from(2);
    }
    if isolation && spike_s {
        eprintln!("use either --isolation or --spike-s, not both");
        return ExitCode::from(2);
    }
    if isolation && project {
        eprintln!("use either --isolation or --project, not both");
        return ExitCode::from(2);
    }
    if isolation && reinforce_fb {
        eprintln!("use either --isolation or --reinforce-fb, not both");
        return ExitCode::from(2);
    }
    if spike && spike_s {
        eprintln!("use either --spike or --spike-s, not both");
        return ExitCode::from(2);
    }
    if spike && project {
        eprintln!("use either --spike or --project, not both");
        return ExitCode::from(2);
    }
    if spike && reinforce_fb {
        eprintln!("use either --spike or --reinforce-fb, not both");
        return ExitCode::from(2);
    }
    if spike_s && project {
        eprintln!("use either --spike-s or --project, not both");
        return ExitCode::from(2);
    }
    if spike_s && reinforce_fb {
        eprintln!("use either --spike-s or --reinforce-fb, not both");
        return ExitCode::from(2);
    }
    if project && reinforce_fb {
        eprintln!("use either --project or --reinforce-fb, not both");
        return ExitCode::from(2);
    }
    if reinforce_fb && rfb_epoch {
        eprintln!("use either --reinforce-fb or --rfb-epoch, not both");
        return ExitCode::from(2);
    }
    if reinforce_fb && structured_fb {
        eprintln!("use either --reinforce-fb or --structured-fb, not both");
        return ExitCode::from(2);
    }
    if reinforce_fb && structured_fb_epoch {
        eprintln!("use either --reinforce-fb or --structured-fb-epoch, not both");
        return ExitCode::from(2);
    }
    if reinforce_fb && structured_fb_capacity {
        eprintln!("use either --reinforce-fb or --structured-fb-capacity, not both");
        return ExitCode::from(2);
    }
    if rfb_epoch && structured_fb {
        eprintln!("use either --rfb-epoch or --structured-fb, not both");
        return ExitCode::from(2);
    }
    if rfb_epoch && structured_fb_epoch {
        eprintln!("use either --rfb-epoch or --structured-fb-epoch, not both");
        return ExitCode::from(2);
    }
    if rfb_epoch && structured_fb_capacity {
        eprintln!("use either --rfb-epoch or --structured-fb-capacity, not both");
        return ExitCode::from(2);
    }
    if structured_fb && structured_fb_epoch {
        eprintln!("use either --structured-fb or --structured-fb-epoch, not both");
        return ExitCode::from(2);
    }
    if structured_fb && structured_fb_capacity {
        eprintln!("use either --structured-fb or --structured-fb-capacity, not both");
        return ExitCode::from(2);
    }
    if structured_fb_epoch && structured_fb_capacity {
        eprintln!("use either --structured-fb-epoch or --structured-fb-capacity, not both");
        return ExitCode::from(2);
    }
    if elig_rfb
        && (reinforce_fb
            || rfb_epoch
            || structured_fb
            || structured_fb_epoch
            || structured_fb_capacity
            || structured_fb_teach)
    {
        eprintln!(
            "use either --elig-rfb or another live-RFB family flag (--reinforce-fb/--rfb-epoch/--structured-fb*), not both"
        );
        return ExitCode::from(2);
    }
    if structured_fb_teach
        && (reinforce_fb
            || rfb_epoch
            || structured_fb
            || structured_fb_epoch
            || structured_fb_capacity
            || elig_rfb
            || dfa_live
            || structured_fb_soft
            || structured_fb_finth
            || structured_fb_cont)
    {
        eprintln!("use either --structured-fb-teach or another live-RFB family flag, not both");
        return ExitCode::from(2);
    }
    let break_it = [
        (dfa_live, "--dfa-live"),
        (structured_fb_soft, "--structured-fb-soft"),
        (structured_fb_finth, "--structured-fb-finth"),
        (structured_fb_cont, "--structured-fb-cont"),
    ];
    let live_family = reinforce_fb
        || rfb_epoch
        || structured_fb
        || structured_fb_epoch
        || structured_fb_capacity
        || elig_rfb
        || structured_fb_teach;
    for (i, (a, an)) in break_it.iter().enumerate() {
        if *a && live_family {
            eprintln!("use either {an} or another live-RFB family flag, not both");
            return ExitCode::from(2);
        }
        for (b, bn) in break_it.iter().skip(i + 1) {
            if *a && *b {
                eprintln!("use either {an} or {bn}, not both");
                return ExitCode::from(2);
            }
        }
    }
    if match_undertrain && !matched_arch {
        eprintln!("--match-undertrain requires --matched-arch");
        return ExitCode::from(2);
    }
    if match_undertrain
        && (matched_dfa
            || matched_rl
            || isolation
            || spike
            || spike_s
            || project
            || live_family
            || dfa_live
            || structured_fb_soft
            || structured_fb_finth
            || structured_fb_cont)
    {
        eprintln!("--match-undertrain cannot combine with other protocol flags");
        return ExitCode::from(2);
    }
    for (flag, name) in [
        (rfb_epoch, "--rfb-epoch"),
        (structured_fb, "--structured-fb"),
        (structured_fb_epoch, "--structured-fb-epoch"),
        (structured_fb_capacity, "--structured-fb-capacity"),
        (elig_rfb, "--elig-rfb"),
        (structured_fb_teach, "--structured-fb-teach"),
        (dfa_live, "--dfa-live"),
        (structured_fb_soft, "--structured-fb-soft"),
        (structured_fb_finth, "--structured-fb-finth"),
        (structured_fb_cont, "--structured-fb-cont"),
    ] {
        if !flag {
            continue;
        }
        if sensitivity.is_some() {
            eprintln!("use either --sensitivity or {name}, not both");
            return ExitCode::from(2);
        }
        if isolation {
            eprintln!("use either --isolation or {name}, not both");
            return ExitCode::from(2);
        }
        if spike || spike_s || project {
            eprintln!("use either --spike/--spike-s/--project or {name}, not both");
            return ExitCode::from(2);
        }
    }

    let config = if mac_probe
        || mac_n_hidden.is_some()
        || mac_max_fan_out.is_some()
        || syn_matched
        || micro_isolate
        || foundation_micro
        || dfa_live_size
    {
        // Mac-probe / micro-isolate / Foundation Micro / H2 size (new hashes; isolate-only at N≥2k).
        if let Some(h) = hash.as_deref() {
            if let Some(mp) = MacProbeConfig::from_hash(h) {
                let kind = if mp.is_foundation_micro() {
                    "foundation-micro"
                } else if mp.is_micro_isolate() {
                    "micro-isolate"
                } else if mp.size_protocol {
                    "mac-size"
                } else {
                    "mac-probe"
                };
                println!("{kind} config hash: {}", mp.hash_string());
                println!(
                    "geometry: N={} fan={} k={} mode={} regime={}",
                    mp.base.n_hidden,
                    mp.max_fan_out,
                    mp.base.k_wta,
                    mp.mode.as_str(),
                    mp.regime().as_str()
                );
                if mp.refuses_full_c1() && isolate_condition.is_none() {
                    eprintln!(
                        "mac/micro probe refuses full multi-condition C1 when n_hidden ≥ {MAC_PROBE_FULL_C1_REFUSE_N} (micro/foundation always) — use --isolate-condition local-assembly"
                    );
                    return ExitCode::from(2);
                }
                mp.to_config()
            } else if let Some(c) = Config::from_hash(h) {
                if !c.is_mac_probe_geometry() {
                    eprintln!("--mac-probe with --config-hash `{h}` is not a mac-probe geometry");
                    return ExitCode::from(2);
                }
                c
            } else {
                eprintln!("unknown mac-probe / config hash `{h}`");
                return ExitCode::from(2);
            }
        } else if foundation_micro {
            let mp = MacProbeConfig::foundation_micro(quick);
            let kn = mp.base.k_wta as f32 / mp.base.n_hidden.max(1) as f32;
            println!("foundation-micro config hash: {}", mp.hash_string());
            println!(
                "geometry: N={} fan={} k={} FIXED (k/N={:.4}) regime={} predicted_nnz≈{} target≈{}",
                mp.base.n_hidden,
                mp.max_fan_out,
                mp.base.k_wta,
                kn,
                mp.regime().as_str(),
                mp.predicted_nnz(),
                FOUNDATION_MICRO_TARGET_NNZ
            );
            println!(
                "Pass band: measured nnz ∈ [{FOUNDATION_MICRO_NNZ_LO}, {FOUNDATION_MICRO_NNZ_HI}]; RSS < 48GB; wall < {}s/seed",
                FOUNDATION_MICRO_WALL_SECS_PER_SEED
            );
            println!(
                "frame: Foundation Microcircuit ~1e6 syn — NOT overnight syn-matched-1e5 @ N=1e4; NOT G2; NOT biology; refuse dense+SurrogateLif"
            );
            if mp.refuses_full_c1() && isolate_condition.is_none() {
                eprintln!(
                    "foundation-micro requires --isolate-condition local-assembly (no full multi-condition C1 / SurrogateLif)"
                );
                return ExitCode::from(2);
            }
            mp.to_config()
        } else if dfa_live_size {
            let mode = if mac_mode_set {
                mac_mode
            } else if structured_fb {
                MacProbeMode::StructuredFb
            } else if dfa_live {
                MacProbeMode::DfaLive
            } else {
                // Default primary arm under size protocol is dfa-live.
                MacProbeMode::DfaLive
            };
            let mp = MacProbeConfig::dfa_live_size(mode, quick);
            println!("mac-size config hash: {}", mp.hash_string());
            println!(
                "geometry: N={} fan={} k={} mode={} size_protocol=true n_seeds={} regime={} predicted_nnz≈{}",
                mp.base.n_hidden,
                mp.max_fan_out,
                mp.base.k_wta,
                mp.mode.as_str(),
                mp.base.n_seeds,
                mp.regime().as_str(),
                mp.predicted_nnz()
            );
            println!(
                "floors: acc≥{DFA_LIVE_SIZE_ACC_FLOOR} · gap LCB vs pm1 > {DFA_LIVE_SIZE_GAP_LCB_CLEAR} → Accept; else Reject-floor / Reject-gap"
            );
            println!(
                "frame: H2 dfa-live width-transfer size protocol — NOT overnight quick H2; NOT frozen v20 remassage; NOT G2 reopen"
            );
            if mp.refuses_full_c1() && isolate_condition.is_none() {
                eprintln!(
                    "mac-size requires --isolate-condition local-assembly (no full multi-condition C1)"
                );
                return ExitCode::from(2);
            }
            mp.to_config()
        } else if micro_isolate {
            let n = mac_n_hidden.unwrap_or(1_000);
            if n != 1_000 && n != 10_000 && n != 100_000 {
                eprintln!("--micro --n-hidden must be 1000, 10000, or 100000");
                return ExitCode::from(2);
            }
            let mut mp = MacProbeConfig::micro_isolate(n, quick);
            if let Some(fan) = mac_max_fan_out {
                if fan > MICRO_MAX_FAN_OUT {
                    eprintln!(
                        "--max-fan-out {fan} exceeds WiringPrior::max_fan_out={MICRO_MAX_FAN_OUT}"
                    );
                    return ExitCode::from(2);
                }
                let k = mac_k_wta.unwrap_or_else(|| scaled_k_wta(n));
                mp = MacProbeConfig::geometry(n, fan, k, MacProbeMode::Pm1, quick);
                // Keep micro experiment / activity band / no-surrogate flags.
                mp.base.experiment = format!("c1-micro-n{n}-f{fan}");
                mp.base.activity_sparsity_min = binn_lab::MICRO_ACTIVITY_MIN;
                mp.base.activity_sparsity_max = binn_lab::MICRO_ACTIVITY_MAX;
                mp.base.use_surrogate_lif_reference = false;
                mp.base.matched_budget_repeat = false;
                mp.protocol_version = binn_lab::C1_MICRO_PROTOCOL_VERSION;
            } else if let Some(k) = mac_k_wta {
                mp = MacProbeConfig::geometry(n, MICRO_MAX_FAN_OUT, k, MacProbeMode::Pm1, quick);
                mp.base.experiment = format!("c1-micro-n{n}-f{MICRO_MAX_FAN_OUT}");
                mp.base.activity_sparsity_min = binn_lab::MICRO_ACTIVITY_MIN;
                mp.base.activity_sparsity_max = binn_lab::MICRO_ACTIVITY_MAX;
                mp.base.use_surrogate_lif_reference = false;
                mp.base.matched_budget_repeat = false;
                mp.protocol_version = binn_lab::C1_MICRO_PROTOCOL_VERSION;
            }
            let kn = mp.base.k_wta as f32 / mp.base.n_hidden.max(1) as f32;
            println!("micro-isolate config hash: {}", mp.hash_string());
            println!(
                "geometry: N={} fan={} k={} (k/N={:.4}) regime={} predicted_nnz≈{}",
                mp.base.n_hidden,
                mp.max_fan_out,
                mp.base.k_wta,
                kn,
                mp.regime().as_str(),
                mp.predicted_nnz()
            );
            println!(
                "frame: engineering capacity stress after G2 FAIL — not Foundation R0 unlock; no dense SurrogateLif / G2 verdict"
            );
            if mp.refuses_full_c1() && isolate_condition.is_none() {
                eprintln!(
                    "micro-isolate requires --isolate-condition local-assembly (no full multi-condition C1)"
                );
                return ExitCode::from(2);
            }
            mp.to_config()
        } else {
            let n = mac_n_hidden.unwrap_or(512);
            let k = mac_k_wta.unwrap_or(MAC_PROBE_K_WTA);
            // Mode from --mac-mode, or map legacy --structured-fb / --dfa-live.
            let mode = if mac_mode != MacProbeMode::Pm1 {
                mac_mode
            } else if structured_fb {
                MacProbeMode::StructuredFb
            } else if dfa_live {
                MacProbeMode::DfaLive
            } else {
                MacProbeMode::Pm1
            };
            let mp = if syn_matched {
                let mut c = MacProbeConfig::syn_matched(n, quick);
                if mode != MacProbeMode::Pm1 {
                    c = MacProbeConfig::geometry(n, c.max_fan_out, k, mode, quick);
                }
                if let Some(fan) = mac_max_fan_out {
                    c = MacProbeConfig::geometry(n, fan, k, mode, quick);
                }
                c
            } else {
                let fan = mac_max_fan_out.unwrap_or_else(|| {
                    if n >= 512 {
                        binn_lab::syn_matched_fan_out(n)
                    } else {
                        256
                    }
                });
                MacProbeConfig::geometry(n, fan, k, mode, quick)
            };
            println!("mac-probe config hash: {}", mp.hash_string());
            println!(
                "geometry: N={} fan={} k={} mode={} regime={} predicted_nnz≈{}",
                mp.base.n_hidden,
                mp.max_fan_out,
                mp.base.k_wta,
                mp.mode.as_str(),
                mp.regime().as_str(),
                mp.predicted_nnz()
            );
            println!(
                "init rescale: init_w_eff = init_w * sqrt(REF_MEAN_FAN_IN / mean_fan_in); REF={}",
                binn_lab::MAC_PROBE_REF_MEAN_FAN_IN
            );
            println!(
                "readout gain: boost * mean_readout_fan_in ≈ (1.15/0.15)*REF_RO_FAN_IN; REF={}",
                binn_lab::MAC_PROBE_REF_MEAN_READOUT_FAN_IN
            );
            if mp.refuses_full_c1() && isolate_condition.is_none() {
                eprintln!(
                    "mac-probe refuses full multi-condition C1 when n_hidden ≥ {MAC_PROBE_FULL_C1_REFUSE_N} — use --isolate-condition local-assembly"
                );
                return ExitCode::from(2);
            }
            mp.to_config()
        }
    } else if let Some(h) = hash {
        // Prefer mac-probe hash table when prefix matches.
        if let Some(mp) = MacProbeConfig::from_hash(&h) {
            println!("mac-probe config hash: {}", mp.hash_string());
            if mp.refuses_full_c1() && isolate_condition.is_none() {
                eprintln!(
                    "mac-probe refuses full multi-condition C1 when n_hidden ≥ {MAC_PROBE_FULL_C1_REFUSE_N} — use --isolate-condition local-assembly"
                );
                return ExitCode::from(2);
            }
            mp.to_config()
        } else {
            let Some(c) = Config::from_hash(&h) else {
                eprintln!("unknown config hash `{h}` — known presets:");
                for p in Config::known_presets() {
                    eprintln!("  {}  ({})", p.hash_string(), p.experiment);
                }
                return ExitCode::from(2);
            };
            if let Err(msg) = validate_protocol_flag_hash(
                &c,
                &h,
                ProtocolFlags {
                    reinforce_fb,
                    rfb_epoch,
                    structured_fb,
                    structured_fb_epoch,
                    structured_fb_capacity,
                    elig_rfb,
                    structured_fb_teach,
                    dfa_live,
                    structured_fb_soft,
                    structured_fb_finth,
                    structured_fb_cont,
                    project,
                    spike_s,
                    spike,
                    isolation,
                    sensitivity: sensitivity.as_deref(),
                },
            ) {
                eprintln!("{msg}");
                return ExitCode::from(2);
            }
            c
        }
    } else if structured_fb_cont {
        if quick {
            Config::c1_structured_fb_cont_quick()
        } else {
            Config::c1_structured_fb_cont()
        }
    } else if structured_fb_finth {
        if quick {
            Config::c1_structured_fb_finth_quick()
        } else {
            Config::c1_structured_fb_finth()
        }
    } else if structured_fb_soft {
        if quick {
            Config::c1_structured_fb_soft_quick()
        } else {
            Config::c1_structured_fb_soft()
        }
    } else if dfa_live {
        if quick {
            Config::c1_dfa_live_quick()
        } else {
            Config::c1_dfa_live()
        }
    } else if structured_fb_teach {
        if quick {
            Config::c1_structured_fb_teach_quick()
        } else {
            Config::c1_structured_fb_teach()
        }
    } else if elig_rfb {
        if quick {
            Config::c1_elig_rfb_quick()
        } else {
            Config::c1_elig_rfb()
        }
    } else if structured_fb_capacity {
        if quick {
            Config::c1_structured_fb_capacity_quick()
        } else {
            Config::c1_structured_fb_capacity()
        }
    } else if structured_fb_epoch {
        if quick {
            Config::c1_structured_fb_epoch_quick()
        } else {
            Config::c1_structured_fb_epoch()
        }
    } else if structured_fb {
        if quick {
            Config::c1_structured_fb_quick()
        } else {
            Config::c1_structured_fb()
        }
    } else if rfb_epoch {
        if quick {
            Config::c1_reinforce_fb_epoch_quick()
        } else {
            Config::c1_reinforce_fb_epoch()
        }
    } else if rfb_learned {
        if quick {
            Config::c1_reinforce_fb_learned_quick()
        } else {
            Config::c1_reinforce_fb_learned()
        }
    } else if k_anneal {
        if quick {
            Config::c1_k_anneal_quick()
        } else {
            Config::c1_k_anneal()
        }
    } else if reinforce_fb {
        if quick {
            Config::c1_reinforce_fb_quick()
        } else {
            Config::c1_reinforce_fb()
        }
    } else if project {
        if quick {
            Config::c1_project_quick()
        } else {
            Config::c1_project()
        }
    } else if spike_s {
        if quick {
            Config::c1_spike_s_quick()
        } else {
            Config::c1_spike_s()
        }
    } else if spike {
        if quick {
            Config::c1_spike_quick()
        } else {
            Config::c1_spike()
        }
    } else if isolation {
        if quick {
            Config::c1_isolation_quick()
        } else {
            Config::c1_isolation()
        }
    } else if let Some(name) = sensitivity {
        match (name.as_str(), quick) {
            ("temporal-pc" | "temporal_pc" | "tpc", true) => {
                Config::c1_temporal_pc_sensitivity_quick()
            }
            ("temporal-pc" | "temporal_pc" | "tpc", false) => Config::c1_temporal_pc_sensitivity(),
            ("capacity" | "cap", true) => Config::c1_capacity_sensitivity_quick(),
            ("capacity" | "cap", false) => Config::c1_capacity_sensitivity(),
            (unknown, _) => {
                eprintln!("unknown sensitivity `{unknown}` — expected temporal-pc|capacity");
                return ExitCode::from(2);
            }
        }
    } else if quick {
        Config::c1_quick()
    } else {
        Config::c1_default()
    };

    // Opt-in JSONL trace for offline viewer (viz only; one seed).
    // Inherited by isolate children via BINN_TRACE_OUT / BINN_TRACE_SEED.
    if let Some(path) = &export_trace {
        let trace_seed = isolate_seed.unwrap_or_else(|| {
            config
                .seeds()
                .into_iter()
                .next()
                .expect("config has ≥1 seed")
        });
        env::set_var(binn_lab::TRACE_OUT_ENV, path);
        env::set_var(binn_lab::TRACE_SEED_ENV, trace_seed.to_string());
        eprintln!(
            "trace export enabled: {} (seed {})",
            path.display(),
            trace_seed
        );
    }

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
    println!(
        "protocol version: {}{}",
        config.protocol_version(),
        if config.is_structured_fb_cont_protocol() {
            " (continuous structured B; does not remassage v15 / reopen v2)"
        } else if config.is_structured_fb_finth_protocol() {
            " (finite-theta under SFB; does not remassage v15 / reopen v2)"
        } else if config.is_structured_fb_soft_protocol() {
            " (soft-WTA x structured B; does not remassage v15 / reopen v2)"
        } else if config.is_dfa_live_protocol() {
            " (live graded-DFA transfer; does not remassage matched-dfa / reopen v2)"
        } else if config.is_structured_fb_teach_protocol() {
            " (structured B × target teach; does not remassage v15 / reopen v2)"
        } else if config.is_elig_rfb_protocol() {
            " (eligibility × REINFORCE; does not remassage v13–v17 / reopen v2)"
        } else if config.is_structured_fb_capacity_protocol() {
            " (structured B × capacity; does not remassage v15 / capacity-only / reopen v2)"
        } else if config.is_structured_fb_epoch_protocol() {
            " (structured B × epoch-matched; does not remassage v14/v15 / reopen v2)"
        } else if config.is_structured_fb_protocol() {
            " (structured frozen B under k-WTA; does not remassage v13 / reopen v2)"
        } else if config.is_reinforce_fb_epoch_protocol() {
            " (live RFB × epoch-matched; does not remassage v13 / reopen v2)"
        } else if config.is_reinforce_fb_protocol() {
            " (live ReinforceFeedback; does not reopen v2 kill-gate / flip default ±1)"
        } else if config.is_project_protocol() {
            " (Assembly-Calculus project; does not reopen v2 kill-gate)"
        } else if config.is_spike_s_protocol() {
            " (calibrated natural-spiking; does not reopen v2 / reinterpret v6)"
        } else if config.is_spike_protocol() {
            " (natural-hidden-spiking; does not reopen v2 kill-gate)"
        } else if config.is_isolation_protocol() {
            " (trial-isolation; does not reopen v2 kill-gate)"
        } else if config.is_sensitivity_protocol() {
            " (Tier-B sensitivity; does not reopen v2 kill-gate)"
        } else {
            ""
        }
    );
    println!("seeds: {:?}", config.seeds());

    let mut runner = Runner::new();
    let report = runner.run_c1(&config);
    let md = Runner::render_results_markdown(&report, &config);

    let out_path = out.unwrap_or_else(|| {
        let default_name = if config.is_structured_fb_cont_protocol() {
            if config.quick {
                "results/c1_sfb_cont_quick.md"
            } else {
                "results/c1_sfb_cont.md"
            }
        } else if config.is_structured_fb_finth_protocol() {
            if config.quick {
                "results/c1_sfb_finth_quick.md"
            } else {
                "results/c1_sfb_finth.md"
            }
        } else if config.is_structured_fb_soft_protocol() {
            if config.quick {
                "results/c1_sfb_soft_quick.md"
            } else {
                "results/c1_sfb_soft.md"
            }
        } else if config.is_dfa_live_protocol() {
            if config.quick {
                "results/c1_dfa_live_quick.md"
            } else {
                "results/c1_dfa_live.md"
            }
        } else if config.is_structured_fb_teach_protocol() {
            if config.quick {
                "results/c1_sfb_teach_quick.md"
            } else {
                "results/c1_sfb_teach.md"
            }
        } else if config.is_elig_rfb_protocol() {
            if config.quick {
                "results/c1_elig_rfb_quick.md"
            } else {
                "results/c1_elig_rfb.md"
            }
        } else if config.is_structured_fb_capacity_protocol() {
            if config.quick {
                "results/c1_sfb_cap_quick.md"
            } else {
                "results/c1_sfb_cap.md"
            }
        } else if config.is_structured_fb_epoch_protocol() {
            if config.quick {
                "results/c1_sfb_em_quick.md"
            } else {
                "results/c1_sfb_em.md"
            }
        } else if config.is_structured_fb_protocol() {
            if config.quick {
                "results/c1_sfb_quick.md"
            } else {
                "results/c1_sfb.md"
            }
        } else if config.is_reinforce_fb_epoch_protocol() {
            if config.quick {
                "results/c1_rfb_em_quick.md"
            } else {
                "results/c1_rfb_em.md"
            }
        } else if config.is_reinforce_fb_protocol() {
            if config.quick {
                "results/c1_rfb_quick.md"
            } else {
                "results/c1_rfb.md"
            }
        } else if config.is_project_protocol() {
            if config.quick {
                "results/c1_project_quick.md"
            } else {
                "results/c1_project.md"
            }
        } else if config.is_spike_s_protocol() {
            if config.quick {
                "results/c1_spike_s_quick.md"
            } else {
                "results/c1_spike_s.md"
            }
        } else if config.is_spike_protocol() {
            if config.quick {
                "results/c1_spike_quick.md"
            } else {
                "results/c1_spike.md"
            }
        } else if config.is_isolation_protocol() {
            if config.quick {
                "results/c1_iso_quick.md"
            } else {
                "results/c1_iso.md"
            }
        } else if config.is_sensitivity_protocol() {
            if config.uses_temporal_positive_control() {
                "results/c1_sens_temporal_pc.md"
            } else {
                "results/c1_sens_capacity.md"
            }
        } else {
            "results/c1_g2.md"
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

    println!("G2 verdict: {}", report.verdict.as_str());
    if config.is_project_protocol()
        || config.is_spike_protocol()
        || config.is_isolation_protocol()
        || config.is_sensitivity_protocol()
        || config.is_reinforce_fb_protocol()
        || config.is_reinforce_fb_epoch_protocol()
        || config.is_structured_fb_protocol()
        || config.is_structured_fb_epoch_protocol()
        || config.is_structured_fb_capacity_protocol()
        || config.is_elig_rfb_protocol()
        || config.is_structured_fb_teach_protocol()
        || config.is_dfa_live_protocol()
        || config.is_structured_fb_soft_protocol()
        || config.is_structured_fb_finth_protocol()
        || config.is_structured_fb_cont_protocol()
    {
        println!(
            "note: project/spike/isolation/sensitivity/reinforce-fb/rfb-epoch/sfb*/elig-rfb/dfa-live results do not reopen protocol-v2 hash c1-118207fbc3eaba53"
        );
    }
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

fn run_matched_arch(
    quick: bool,
    hash: Option<&str>,
    out: Option<PathBuf>,
    conflicting_flag: bool,
    undertrain: bool,
) -> ExitCode {
    if conflicting_flag {
        eprintln!(
            "--matched-arch cannot be combined with --sensitivity / --isolation / --spike / --spike-s / --project / --capacity"
        );
        return ExitCode::from(2);
    }
    let config = if let Some(h) = hash {
        // Accept only c1-match-* presets; refuse v2/v3 C1 hashes.
        if let Some(c) = MatchConfig::from_hash(h) {
            c
        } else if Config::from_hash(h).is_some() {
            eprintln!(
                "--matched-arch refuses protocol-v2/v3 config-hash `{h}` — \
                 use a c1-match-* hash or omit --config-hash"
            );
            return ExitCode::from(2);
        } else {
            eprintln!("unknown matched-arch hash `{h}` — known presets:");
            for p in MatchConfig::known_presets() {
                eprintln!("  {}  ({})", p.hash_string(), p.base.experiment);
            }
            return ExitCode::from(2);
        }
    } else if undertrain {
        if quick {
            MatchConfig::undertrain_epochs_quick()
        } else {
            MatchConfig::undertrain_epochs()
        }
    } else if quick {
        MatchConfig::quick()
    } else {
        MatchConfig::scientific()
    };

    println!("C1-MATCH config hash: {}", config.hash_string());
    println!(
        "protocol version: {} (matched-architecture control)",
        config.protocol_version
    );
    println!("does not reopen protocol-v2 hash c1-118207fbc3eaba53");
    println!("seeds: {:?}", config.seeds());

    let mut runner = MatchRunner::new();
    let report = runner.run(&config);
    let md = MatchRunner::render_markdown(&report, &config);

    let out_path = out.unwrap_or_else(|| {
        let default_name = if config.quick {
            "results/c1_match_quick.md"
        } else {
            "results/c1_match.md"
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

    println!("C1-MATCH verdict: {}", report.verdict.as_str());
    println!(
        "means: matched-local={:.4} matched-gradient={:.4}",
        report.mean_matched_local, report.mean_matched_gradient
    );
    println!(
        "gap_closed_matched={:.4}  lower-95={:.4}",
        report.mean_gap_closed_matched, report.gap_closed_matched_lower_95
    );
    println!("results note: {}", out_path.display());
    ExitCode::SUCCESS
}

fn run_matched_dfa(
    quick: bool,
    hash: Option<&str>,
    out: Option<PathBuf>,
    conflicting_flag: bool,
) -> ExitCode {
    if conflicting_flag {
        eprintln!(
            "--matched-dfa cannot be combined with --sensitivity / --isolation / --spike / --spike-s / --project / --capacity"
        );
        return ExitCode::from(2);
    }
    let config = if let Some(h) = hash {
        if let Some(c) = DfaMatchConfig::from_hash(h) {
            c
        } else if MatchConfig::from_hash(h).is_some() || Config::from_hash(h).is_some() {
            eprintln!(
                "--matched-dfa refuses non-c1-dfa config-hash `{h}` — \
                 use a c1-dfa-* hash or omit --config-hash"
            );
            return ExitCode::from(2);
        } else {
            eprintln!("unknown matched-dfa hash `{h}` — known presets:");
            for p in DfaMatchConfig::known_presets() {
                eprintln!("  {}  ({})", p.hash_string(), p.base.experiment);
            }
            return ExitCode::from(2);
        }
    } else if quick {
        DfaMatchConfig::quick()
    } else {
        DfaMatchConfig::scientific()
    };

    println!("C1-DFA config hash: {}", config.hash_string());
    println!(
        "protocol version: {} (matched-architecture DFA recipe)",
        config.protocol_version
    );
    println!("does not reopen protocol-v2 hash c1-118207fbc3eaba53");
    println!("seeds: {:?}", config.seeds());

    let mut runner = DfaMatchRunner::new();
    let report = runner.run(&config);
    let md = DfaMatchRunner::render_markdown(&report, &config);

    let out_path = out.unwrap_or_else(|| {
        let default_name = if config.quick {
            "results/c1_dfa_quick.md"
        } else {
            "results/c1_dfa.md"
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

    println!("C1-DFA verdict: {}", report.verdict.as_str());
    println!(
        "means: dfa={:.4} broadcast-err={:.4} gradient={:.4}",
        report.mean_matched_dfa, report.mean_matched_broadcast_err, report.mean_matched_gradient
    );
    println!(
        "gap_closed_dfa={:.4}  lower-95={:.4}",
        report.mean_gap_closed_dfa, report.gap_closed_dfa_lower_95
    );
    println!("results note: {}", out_path.display());
    ExitCode::SUCCESS
}

fn run_matched_rl(
    quick: bool,
    hash: Option<&str>,
    out: Option<PathBuf>,
    conflicting_flag: bool,
) -> ExitCode {
    if conflicting_flag {
        eprintln!(
            "--matched-rl cannot be combined with --sensitivity / --isolation / --spike / --spike-s / --project / --capacity"
        );
        return ExitCode::from(2);
    }
    let config = if let Some(h) = hash {
        if let Some(c) = RlMatchConfig::from_hash(h) {
            c
        } else if DfaMatchConfig::from_hash(h).is_some()
            || MatchConfig::from_hash(h).is_some()
            || Config::from_hash(h).is_some()
        {
            eprintln!(
                "--matched-rl refuses non-c1-rl config-hash `{h}` — \
                 use a c1-rl-* hash or omit --config-hash"
            );
            return ExitCode::from(2);
        } else {
            eprintln!("unknown matched-rl hash `{h}` — known presets:");
            for p in RlMatchConfig::known_presets() {
                eprintln!("  {}  ({})", p.hash_string(), p.base.experiment);
            }
            return ExitCode::from(2);
        }
    } else if quick {
        RlMatchConfig::quick()
    } else {
        RlMatchConfig::scientific()
    };

    println!("C1-RL config hash: {}", config.hash_string());
    println!(
        "protocol version: {} (matched-architecture RL; primary={})",
        config.protocol_version, config.primary_arm
    );
    println!("does not reopen protocol-v2 hash c1-118207fbc3eaba53");
    println!("does not retune failed v11 rl_graded (c1-rl-ef504db58916720d)");
    println!("seeds: {:?}", config.seeds());

    let mut runner = RlMatchRunner::new();
    let report = runner.run(&config);
    let md = RlMatchRunner::render_markdown(&report, &config);

    let out_path = out.unwrap_or_else(|| {
        let default_name = if config.quick {
            "results/c1_rl_quick.md"
        } else {
            "results/c1_rl.md"
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

    println!("C1-RL verdict: {}", report.verdict.as_str());
    println!(
        "means: rl_reinforce_fb={:.4} rl_graded={:.4} rl_flat={:.4} gradient={:.4}",
        report.mean_matched_rl_reinforce_fb,
        report.mean_matched_rl_graded,
        report.mean_matched_rl_flat,
        report.mean_matched_gradient
    );
    println!(
        "gap_closed_rl={:.4}  lower-95={:.4}",
        report.mean_gap_closed_rl, report.gap_closed_rl_lower_95
    );
    println!("results note: {}", out_path.display());
    ExitCode::SUCCESS
}

fn run_matched_mech(
    quick: bool,
    hash: Option<&str>,
    out: Option<PathBuf>,
    conflicting_flag: bool,
) -> ExitCode {
    if conflicting_flag {
        eprintln!(
            "--matched-mech cannot be combined with --sensitivity / --isolation / live protocol flags"
        );
        return ExitCode::from(2);
    }
    let config = if let Some(h) = hash {
        if let Some(c) = MechConfig::from_hash(h) {
            c
        } else {
            eprintln!("unknown matched-mech hash `{h}` — known presets:");
            for p in MechConfig::known_presets() {
                eprintln!("  {}  ({})", p.hash_string(), p.base.experiment);
            }
            return ExitCode::from(2);
        }
    } else if quick {
        MechConfig::quick()
    } else {
        MechConfig::scientific()
    };

    println!("C1-MECH config hash: {}", config.hash_string());
    println!(
        "protocol version: {} (mechanism diagnostic; recording only)",
        config.protocol_version
    );
    println!("does not reopen protocol-v2 / c1-match-* / c1-dfa-* / c1-rl-* hashes");
    println!("seeds: {:?}", config.seeds());

    let mut runner = MechRunner::new();
    let report = runner.run(&config);
    let md = MechRunner::render_markdown(&report, &config);

    let out_path = out.unwrap_or_else(|| {
        let default_name = if config.quick {
            "results/c1_credit_mech_quick.md"
        } else {
            "results/c1_credit_mech.md"
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

    println!(
        "C1-MECH schedule: {}",
        if report.pilot { "PILOT" } else { "SCIENTIFIC" }
    );
    for a in &report.mean_arms {
        println!(
            "  {}: loss_drop={:.6} rot={:.6} elig_cap={:.4}",
            a.arm, a.loss_drop, a.loss_drop_rotate, a.elig_energy_capture
        );
    }
    println!("results note: {}", out_path.display());
    ExitCode::SUCCESS
}

fn run_matched_eventprop(
    quick: bool,
    hash: Option<&str>,
    out: Option<PathBuf>,
    conflicting_flag: bool,
) -> ExitCode {
    if conflicting_flag {
        eprintln!(
            "--eventprop cannot be combined with --sensitivity / --isolation / live protocol flags"
        );
        return ExitCode::from(2);
    }
    let config = if let Some(h) = hash {
        if let Some(c) = EventPropMatchConfig::from_hash(h) {
            c
        } else if MatchConfig::from_hash(h).is_some()
            || DfaMatchConfig::from_hash(h).is_some()
            || Config::from_hash(h).is_some()
        {
            eprintln!(
                "--eventprop refuses non-c1-eventprop config-hash `{h}` — \
                 use a c1-eventprop-* hash or omit --config-hash"
            );
            return ExitCode::from(2);
        } else {
            eprintln!("unknown eventprop hash `{h}` — known presets:");
            for p in EventPropMatchConfig::known_presets() {
                eprintln!("  {}  ({})", p.hash_string(), p.base.experiment);
            }
            return ExitCode::from(2);
        }
    } else if quick {
        EventPropMatchConfig::quick()
    } else {
        EventPropMatchConfig::scientific()
    };

    println!("C1-EVENTPROP config hash: {}", config.hash_string());
    println!(
        "protocol version: {} (matched EventProp H2H; rule-only)",
        config.protocol_version
    );
    println!("does not reopen protocol-v2 / c1-match-* / c1-dfa-* / c1-rl-* hashes");
    println!("seeds: {:?}", config.seeds());

    let mut runner = EventPropMatchRunner::new();
    let report = runner.run(&config);
    let md = EventPropMatchRunner::render_markdown(&report, &config);

    let out_path = out.unwrap_or_else(|| {
        let default_name = if config.quick {
            "results/c1_eventprop_quick.md"
        } else {
            "results/c1_eventprop.md"
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

    println!("C1-EVENTPROP verdict: {}", report.verdict.as_str());
    println!(
        "means: eventprop={:.4} gradient={:.4}",
        report.mean_matched_eventprop, report.mean_matched_gradient
    );
    println!(
        "gap_closed_eventprop={:.4}  lower-95={:.4}",
        report.mean_gap_closed_eventprop, report.gap_closed_eventprop_lower_95
    );
    println!("results note: {}", out_path.display());
    ExitCode::SUCCESS
}

fn run_shd_cal(
    quick: bool,
    hash: Option<&str>,
    out: Option<PathBuf>,
    shd_hidden: Option<usize>,
    shd_full: bool,
    shd_smoke: bool,
    conflicting_flag: bool,
) -> ExitCode {
    if conflicting_flag {
        eprintln!(
            "--shd-cal/--shd-full cannot be combined with --sensitivity / --isolation / live protocol flags"
        );
        return ExitCode::from(2);
    }
    if shd_smoke && !shd_full && hash.is_none() {
        eprintln!("--smoke requires --shd-full (protocol-29 path-proof subset)");
        return ExitCode::from(2);
    }
    let mut config = if let Some(h) = hash {
        if let Some(c) = ShdCalConfig::from_hash(h) {
            c
        } else {
            eprintln!("unknown shd-cal/shd-full hash `{h}` — known presets:");
            for p in ShdCalConfig::known_presets() {
                eprintln!("  {}  ({})", p.hash_string(), p.base.experiment);
            }
            return ExitCode::from(2);
        }
    } else if quick && shd_full {
        ShdCalConfig::quick_full()
    } else if quick {
        ShdCalConfig::quick()
    } else if shd_smoke {
        ShdCalConfig::scientific_full_smoke()
    } else if shd_full {
        ShdCalConfig::scientific_full()
    } else if shd_hidden == Some(256) {
        ShdCalConfig::scientific_hidden256()
    } else if shd_hidden == Some(512) {
        ShdCalConfig::scientific_hidden512()
    } else {
        ShdCalConfig::scientific()
    };

    if let Some(h) = shd_hidden {
        if hash.is_some() && config.shd_hidden != h {
            eprintln!(
                "--shd-hidden {h} conflicts with --config-hash {} (hidden={})",
                config.hash_string(),
                config.shd_hidden
            );
            return ExitCode::from(2);
        }
        if config.protocol_version == binn_lab::C1_SHD_FULL_PROTOCOL_VERSION {
            if !quick && !shd_smoke && hash.is_none() && h != config.shd_hidden {
                eprintln!(
                    "warning: --shd-hidden {h} on --shd-full overrides geometry; hash will not round-trip"
                );
                config.shd_hidden = h;
            }
        } else if quick {
            config.shd_hidden = h;
        } else if h == 512 {
            config = ShdCalConfig::scientific_hidden512();
        } else if h == 256 {
            config = ShdCalConfig::scientific_hidden256();
        } else if h == 128 {
            config = ShdCalConfig::scientific();
        } else if hash.is_none() {
            // Geometry override on scientific base → new hashed config (not a known preset).
            config.shd_hidden = h;
            eprintln!(
                "warning: --shd-hidden {h} is not a known scientific preset; hash will not round-trip via from_hash"
            );
        }
    }

    println!("C1-SHD config hash: {}", config.hash_string());
    println!(
        "protocol version: {} (SHD {}; NOT Gate G2)",
        config.protocol_version,
        if config.protocol_version == binn_lab::C1_SHD_FULL_PROTOCOL_VERSION {
            "full-corpus / SuperSpike ceiling"
        } else {
            "calibration; capped e-prop ceiling"
        }
    );
    println!("chance baseline: {:.4} (1/20)", config.chance_baseline);
    println!("shd_hidden: {}", config.shd_hidden);
    println!(
        "ceiling: {}",
        if config.include_superspike {
            "SuperSpike BPTT"
        } else {
            "e-prop"
        }
    );
    println!("seeds: {:?}", config.seeds());

    let wall_start = std::time::Instant::now();
    let mut runner = ShdCalRunner::new();
    let report = match runner.run(&config) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("SHD calibration failed: {e}");
            return ExitCode::from(1);
        }
    };
    let wall_secs = wall_start.elapsed().as_secs_f64();
    let md = ShdCalRunner::render_markdown(&report, &config);
    let md = format!(
        "{md}\n## Compute disclosure\n\n\
         - wall_time_s: {:.1}\n\
         - n_train / n_test: {} / {}\n\
         - seeds × epochs × arms: {} × {} × {}\n\
         - feasibility: feed-forward SuperSpike BPTT is O(T·H·N_IN) per example; \
           full official splits (8156/2264) are runnable on a workstation CPU with \
           multi-hour wall time — disclose this number; do not claim free SOTA.\n",
        wall_secs,
        report.n_train,
        report.n_test,
        config.base.n_seeds,
        config.shd_epochs,
        if config.include_rl_fb { 4 } else { 3 },
    );

    let out_path = out.unwrap_or_else(|| {
        let default_name = if config.quick {
            if config.include_superspike {
                "results/c1_shd_full_quick.md"
            } else {
                "results/c1_shd_quick.md"
            }
        } else if config.protocol_version == binn_lab::C1_SHD_FULL_PROTOCOL_VERSION {
            if config.max_train > 0 {
                "results/c1_shd_full_smoke.md"
            } else {
                "results/c1_shd_full.md"
            }
        } else if config.shd_hidden == 512 {
            "results/c1_shd_h512.md"
        } else if config.shd_hidden == 256 {
            "results/c1_shd_h256.md"
        } else {
            "results/c1_shd.md"
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

    println!(
        "C1-SHD schedule: {}  fixture={}  wall_s={:.1}",
        if report.pilot { "PILOT" } else { "SCIENTIFIC" },
        report.fixture,
        wall_secs
    );
    if config.include_superspike {
        if config.include_rl_fb {
            println!(
                "means: broadcast_pm1={:.4} dfa={:.4} rl_reinforce_fb={:.4} superspike_ceiling={:.4} chance={:.4}",
                report.mean_broadcast_pm1,
                report.mean_dfa,
                report.mean_rl_reinforce_fb,
                report.mean_superspike_ceiling,
                report.chance_baseline
            );
        } else {
            println!(
                "means: broadcast_pm1={:.4} dfa={:.4} superspike_ceiling={:.4} chance={:.4}",
                report.mean_broadcast_pm1,
                report.mean_dfa,
                report.mean_superspike_ceiling,
                report.chance_baseline
            );
        }
    } else if config.include_rl_fb {
        println!(
            "means: broadcast_pm1={:.4} dfa={:.4} rl_reinforce_fb={:.4} eprop_ceiling={:.4} chance={:.4}",
            report.mean_broadcast_pm1,
            report.mean_dfa,
            report.mean_rl_reinforce_fb,
            report.mean_eprop_ceiling,
            report.chance_baseline
        );
    } else {
        println!(
            "means: broadcast_pm1={:.4} dfa={:.4} eprop_ceiling={:.4} chance={:.4}",
            report.mean_broadcast_pm1,
            report.mean_dfa,
            report.mean_eprop_ceiling,
            report.chance_baseline
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

fn print_help() {
    eprintln!(
        "Usage: c1 [--quick] [--config-hash HASH] [--sensitivity temporal-pc|capacity]\n\
         \n\
         Matched-architecture control (protocol v4; new c1-match-* hash):\n\
           c1 --matched-arch [--quick] [--out results/c1_match.md]\n\
           c1 --matched-arch --config-hash c1-match-<hex>\n\
         Refuse combo with --sensitivity / --isolation / v2–v3 --config-hash.\n\
         Does not reopen protocol-v2 kill-gate c1-118207fbc3eaba53.\n\
         \n\
         Matched-architecture DFA recipe (protocol v5; new c1-dfa-* hash):\n\
           c1 --matched-dfa [--quick] [--out results/c1_dfa.md]\n\
           c1 --matched-dfa --config-hash c1-dfa-<hex>\n\
         Directional graded error × fixed-random DFA on dense-LIF matched forward.\n\
         Does not reopen protocol-v2 or mutate c1-match-* / c1-iso*.\n\
         \n\
         Matched-architecture in-family RL recipe (protocol v12; new c1-rl-* hash):\n\
           c1 --matched-rl [--quick] [--out results/c1_rl.md]\n\
           c1 --matched-rl --config-hash c1-rl-<hex>\n\
         Primary rl_reinforce_fb (vs rl_graded / rl_flat contrasts) on dense-LIF matched forward.\n\
         Does not reopen protocol-v2, retune v11 rl_graded, or mutate c1-dfa-* / c1x-dfa-spike-*.\n\
         \n\
         Mechanism diagnostic (protocol v25; new c1-mech-* hash):\n\
           c1 --matched-mech [--quick] [--out results/c1_credit_mech.md]\n\
         One-step loss-drop + eligibility-energy on frozen matched feed-forward dense-LIF.\n\
         Recording only — does not mutate c1-match-* / c1-dfa-* / c1-rl-* / G2 hashes.\n\
         \n\
         Matched EventProp H2H (protocol v28; new c1-eventprop-* hash):\n\
           c1 --eventprop [--quick] [--out results/c1_eventprop.md]\n\
           c1 --eventprop --config-hash c1-eventprop-<hex>\n\
         Discrete EventProp-style spike adjoint vs SuperSpike on recurrent matched dense-LIF.\n\
         Rule-only; not neuromorphic HW; does not mutate c1-match-* / c1-dfa-* / c1-rl-*.\n\
         \n\
         SHD calibration (protocol v27 +RL×B; frozen v26 hash still accepted):\n\
           c1 --shd-cal --quick [--out results/c1_shd.md]   # CI fixture smoke\n\
           c1 --shd-cal [--out results/c1_shd.md]           # needs data/shd/{{train,test}}.bin\n\
           c1 --shd-cal --shd-hidden 256 [--out results/c1_shd_h256.md]  # p27 geometry=256\n\
           c1 --shd-cal --config-hash c1-shd-cal-bafa6835d8de7eb8         # same as --shd-hidden 256\n\
           c1 --shd-cal --config-hash c1-shd-cal-de44bb52bbd28fbc  # archived 3-arm scientific\n\
         20-way passthrough spikes; e-prop ceiling; NOT Gate G2; not neuromorphic SOTA.\n\
         Caps max_train=2000/max_test=500 — calibration, not full-corpus SOTA.\n\
         Frozen p27 default hash c1-shd-cal-eb3cb5d93417a638 (hidden=128) — do not remassage.\n\
         \n\
         SHD full-corpus + SuperSpike ceiling (protocol v29; NEW c1-shd-full-* hash):\n\
           c1 --shd-full --quick [--out results/c1_shd_full_quick.md]  # fixture + SuperSpike path\n\
           c1 --shd-full --smoke [--out results/c1_shd_full_smoke.md]  # 400/100 path-proof\n\
           c1 --shd-full [--out results/c1_shd_full.md]                # official 8156/2264\n\
           c1 --shd-full --config-hash c1-shd-full-2c93117075740ed0\n\
         Arms: pm1 / DFA / RL×B / true SuperSpike BPTT. Distinct from p27 and proto-135.\n\
         Convert via Rust only: cargo run -p binn-data --features shd-convert --bin convert-shd\n\
         \n\
         Causal mac-probe size science (c1-mac-probe-*; NOT Gate G2):\n\
           c1 --mac-probe --syn-matched --n-hidden 512 --quick --isolate-condition local-assembly\n\
           c1 --mac-probe --n-hidden 2000 --max-fan-out 32 --k-wta 8 --quick --isolate-condition local-assembly\n\
           c1 --mac-probe --syn-matched --n-hidden 2000 --mac-mode structured-fb --quick --isolate-condition local-assembly\n\
         Fixed k_wta; plumbed max_fan_out; init_w∝1/√fan_in; readout gain normalize.\n\
         Refuses full multi-condition C1 when n_hidden≥2000 (isolate-only).\n\
         \n\
         Mac/Micro isolate capacity stress (c1-micro-*; NOT Gate G2 / NOT R0 unlock):\n\
           c1 --micro --n-hidden 1000 --quick --isolate-condition local-assembly\n\
           c1 --micro --n-hidden 10000 --quick --isolate-condition local-assembly\n\
           c1 --micro --n-hidden 10000 --isolate-condition local-assembly --out results/c1_mac_probe.md\n\
         Activity-scaled k_wta (k/N∈[0.005,0.03]); max_fan_out=256; matched_budget_repeat=false.\n\
         Isolate local-assembly only — no dense SurrogateLif / G2 verdict.\n\
         \n\
         Foundation Microcircuit ~1e6 syn (c1-micro-foundation-*; fixed k; NOT syn-matched-1e5):\n\
           c1 --foundation-micro --isolate-condition local-assembly\n\
           c1 --foundation-micro --quick --isolate-condition local-assembly\n\
         N=10000 fan=100 k=8 FIXED; init/readout rescale; isolate-only; refuse dense+SurrogateLif.\n\
         Pass: measured nnz∈[8e5,1.2e6], RSS<48GB, wall<20min/seed. NOT G2 reopen / NOT biology.\n\
         \n\
         H2 dfa-live width-transfer size protocol (new c1-mac-probe-*-size hashes):\n\
           c1 --dfa-live-size --mac-mode dfa-live --isolate-condition local-assembly --seed N\n\
           c1 --dfa-live-size --mac-mode pm1 --isolate-condition local-assembly --seed N\n\
           c1 --dfa-live-size --mac-mode structured-fb --isolate-condition local-assembly --seed N\n\
         N=2000 syn-matched; scientific n_seeds=8; floor acc≥0.60; gap LCB vs pm1>0.\n\
         Accept / Reject-floor / Reject-gap. NOT overnight quick H2; NOT frozen v20 remassage.\n\
         \n\
         Trial-isolation integrity protocol (protocol v5; new c1-iso* hash):\n\
           c1 --isolation [--quick] [--out results/c1_iso.md]\n\
           c1 --isolation --config-hash c1-iso*<hex>\n\
         Clears ThreeFactor.last_spike + C3-style membrane reset at trial boundaries.\n\
         Does not reopen protocol-v2 kill-gate c1-118207fbc3eaba53.\n\
         \n\
         Natural-hidden-spiking protocol (protocol v6; historical INVALID_HARNESS):\n\
           c1 --spike [--quick] [--out results/c1_spike.md]\n\
         Finite hidden θ during integrate (no θ=∞ mute); membrane-score k-WTA.\n\
         Does not reopen protocol-v2 kill-gate c1-118207fbc3eaba53.\n\
         \n\
         Calibrated natural-spiking protocol (protocol v9; new c1-spike-s* hash):\n\
           c1 --spike-s [--quick] [--out results/c1_spike_s.md]\n\
         Finite θ + spike-count k-WTA + disclosed multi-frame PC; knobs calibrated.\n\
         Does not reopen v2 or reinterpret v6 c1-09442acdbdc0c752 (thresholds unchanged).\n\
         \n\
         Assembly-Calculus project protocol (protocol v7; new c1-project* hash):\n\
           c1 --project [--quick] [--out results/c1_project.md]\n\
         Wires binn_areas::project into hidden selection (not inline membrane k-WTA).\n\
         Does not reopen protocol-v2 kill-gate c1-118207fbc3eaba53.\n\
         \n\
         Live ReinforceFeedback neuromodulator (protocol v13; new c1-rfb* hash):\n\
           c1 --reinforce-fb [--quick] [--out results/c1_rfb.md]\n\
           c1 --reinforce-fb --config-hash c1-<hex>\n\
         Same k-WTA / single-pass C1 substrate; plasticity uses ReinforceFeedback × reinforce_term.\n\
         Does not flip default ±1 C1, remassage P4 spiking-DFA, or retune P5 rl_graded.\n\
         \n\
         Live RFB × epoch-matched (protocol v14; new c1-rfb-em* hash):\n\
           c1 --rfb-epoch [--quick] [--out results/c1_rfb_em.md]\n\
           c1 --rfb-epoch --config-hash c1-<hex>\n\
         Same neuromodulator as v13; local/dense train for disclosed multi-epoch exposure.\n\
         Does not remassage v13 c1-660401d74db3c88d.\n\
         \n\
         Structured frozen B under k-WTA (protocol v15; new c1-sfb* hash):\n\
           c1 --structured-fb [--quick] [--out results/c1_sfb.md]\n\
           c1 --structured-fb --config-hash c1-<hex>\n\
         Hidden B_i = sign(w→readout_1 − w→readout_0) after boost; single-pass.\n\
         Does not remassage v13 random-B FAIL.\n\
         \n\
         Structured B × epoch-matched (protocol v16; new c1-sfb-em* hash):\n\
           c1 --structured-fb-epoch [--quick] [--out results/c1_sfb_em.md]\n\
           c1 --structured-fb-epoch --config-hash c1-<hex>\n\
         v15 structured B + disclosed multi-epoch exposure (same schedule as v14).\n\
         Does not remassage v14/v15 FAILs in place.\n\
         \n\
         Structured B × capacity (protocol v17; new c1-sfb-cap* hash):\n\
           c1 --structured-fb-capacity [--quick] [--out results/c1_sfb_cap.md]\n\
           c1 --structured-fb-capacity --config-hash c1-<hex>\n\
         v15 structured B on Tier-B capacity substrate (richer k/N/train).\n\
         Does not remassage v15 or capacity-only c1-d38d7644d8afc84b.\n\
         \n\
         Eligibility × REINFORCE (protocol v18; new c1-elig-rfb* hash):\n\
           c1 --elig-rfb [--quick] [--out results/c1_elig_rfb.md]\n\
           c1 --elig-rfb --config-hash c1-<hex>\n\
         v15 structured B + τ_e=160 + mid-trial eligibility absorb before REINFORCE action.\n\
         Does not remassage v13–v17 FAILs in place.\n\
         \n\
         Structured B × target teach (protocol v19; new c1-sfb-teach* hash):\n\
           c1 --structured-fb-teach [--quick] [--out results/c1_sfb_teach.md]\n\
           c1 --structured-fb-teach --config-hash c1-<hex>\n\
         v15 structured B + secondary credit(+1) teach on incorrect trials (not observe-only).\n\
         Does not remassage v15 FAIL in place.\n\
         \n\
         Continuous structured B (protocol v24; new c1-sfb-cont* hash):\n\
           c1 --structured-fb-cont [--quick] [--out results/c1_sfb_cont.md]\n\
         L2-normalized B proportional to (w1-w0); one construction.\n\
         \n\
         Live graded-DFA transfer (protocol v20; new c1-dfa-live* hash):\n\
           c1 --dfa-live [--quick] [--out results/c1_dfa_live.md]\n\
         Graded error x FixedRandomFeedback on muted-theta/k-WTA C1.\n\
         \n\
         Soft-WTA x structured B (protocol v21; new c1-sfb-soft* hash):\n\
           c1 --structured-fb-soft [--quick] [--out results/c1_sfb_soft.md]\n\
         One disclosed temperature T=1.0; no grid.\n\
         \n\
         Matched undertrain adversarial (protocol v22):\n\
           c1 --matched-arch --match-undertrain [--quick] [--out results/c1_match_ep4.md]\n\
         4x epochs on matched broadcast three-factor; new c1-match-* hash.\n\
         \n\
         Finite-theta under SFB (protocol v23; new c1-sfb-finth* hash):\n\
           c1 --structured-fb-finth [--quick] [--out results/c1_sfb_finth.md]\n\
         Mute off under structured B; may INVALID_HARNESS.\n\
         \n\
         Optional Tier-B sensitivities (protocol v3; new hashes):\n\
           --sensitivity temporal-pc   coincidence-lag positive control\n\
           --sensitivity capacity      richer k_wta / n_train schedule\n\
           --capacity                  alias for --sensitivity capacity\n\
           c1 --sensitivity capacity --config-hash c1-<hex>\n\
         \n\
         Isolate one condition (peak-RSS child):\n\
           c1 --isolate-condition LABEL --seed N [--config-hash HASH] [--match-nnz N]\n\
         \n\
         Replay export (viz only; open viz/replay_viewer.html on the JSON):\n\
           c1 --quick --replay results/c1_replay.json\n\
         \n\
         Offline spike/assembly JSONL (viewer: results/viewer.html):\n\
           c1 --quick --isolate-condition local-assembly --seed 1 --export-trace\n\
           c1 --quick --export-trace results/c1_trace.jsonl\n\
         \n\
         Reproduces experiment C1 (Gate G2): local-assembly vs labeled\n\
         gradient / eligibility references and dense-local control.\n\
         --quick is PILOT only (never a scientific PASS/FAIL).\n\
         Protocol flags may pair with --config-hash when the hash matches that\n\
         family (matched-* style). Isolation/sensitivity runs do not reopen\n\
         protocol-v2 kill-gate c1-118207fbc3eaba53."
    );
}

/// When a protocol flag is set together with `--config-hash`, require the loaded
/// preset to belong to that family (same contract as `--matched-arch --config-hash`).
struct ProtocolFlags<'a> {
    reinforce_fb: bool,
    rfb_epoch: bool,
    structured_fb: bool,
    structured_fb_epoch: bool,
    structured_fb_capacity: bool,
    elig_rfb: bool,
    structured_fb_teach: bool,
    dfa_live: bool,
    structured_fb_soft: bool,
    structured_fb_finth: bool,
    structured_fb_cont: bool,
    project: bool,
    spike_s: bool,
    spike: bool,
    isolation: bool,
    sensitivity: Option<&'a str>,
}

fn validate_protocol_flag_hash(
    c: &Config,
    h: &str,
    flags: ProtocolFlags<'_>,
) -> Result<(), String> {
    if flags.reinforce_fb && !c.is_reinforce_fb_protocol() {
        return Err(format!(
            "--reinforce-fb refuses non-c1-rfb (v13) config-hash `{h}` — \
             use a live ReinforceFeedback hash or omit --config-hash"
        ));
    }
    if flags.rfb_epoch && !c.is_reinforce_fb_epoch_protocol() {
        return Err(format!(
            "--rfb-epoch refuses non-c1-rfb-em config-hash `{h}` — \
             use an epoch-matched RFB hash or omit --config-hash"
        ));
    }
    if flags.structured_fb && !c.is_structured_fb_protocol() {
        return Err(format!(
            "--structured-fb refuses non-c1-sfb (v15) config-hash `{h}` — \
             use a structured-feedback hash or omit --config-hash"
        ));
    }
    if flags.structured_fb_epoch && !c.is_structured_fb_epoch_protocol() {
        return Err(format!(
            "--structured-fb-epoch refuses non-c1-sfb-em config-hash `{h}` — \
             use a structured×epoch hash or omit --config-hash"
        ));
    }
    if flags.structured_fb_capacity && !c.is_structured_fb_capacity_protocol() {
        return Err(format!(
            "--structured-fb-capacity refuses non-c1-sfb-cap config-hash `{h}` — \
             use a structured×capacity hash or omit --config-hash"
        ));
    }
    if flags.elig_rfb && !c.is_elig_rfb_protocol() {
        return Err(format!(
            "--elig-rfb refuses non-c1-elig-rfb config-hash `{h}` — \
             use an eligibility×REINFORCE hash or omit --config-hash"
        ));
    }
    if flags.structured_fb_teach && !c.is_structured_fb_teach_protocol() {
        return Err(format!(
            "--structured-fb-teach refuses non-c1-sfb-teach config-hash `{h}` — \
             use a structured×teach hash or omit --config-hash"
        ));
    }
    if flags.dfa_live && !c.is_dfa_live_protocol() {
        return Err(format!(
            "--dfa-live refuses non-c1-dfa-live config-hash `{h}` — \
             use a live graded-DFA hash or omit --config-hash"
        ));
    }
    if flags.structured_fb_soft && !c.is_structured_fb_soft_protocol() {
        return Err(format!(
            "--structured-fb-soft refuses non-c1-sfb-soft config-hash `{h}` — \
             use a soft-WTA structured-B hash or omit --config-hash"
        ));
    }
    if flags.structured_fb_finth && !c.is_structured_fb_finth_protocol() {
        return Err(format!(
            "--structured-fb-finth refuses non-c1-sfb-finth config-hash `{h}` — \
             use a finite-theta SFB hash or omit --config-hash"
        ));
    }
    if flags.structured_fb_cont && !c.is_structured_fb_cont_protocol() {
        return Err(format!(
            "--structured-fb-cont refuses non-c1-sfb-cont config-hash `{h}` — \
             use a continuous structured-B hash or omit --config-hash"
        ));
    }
    if flags.project && !c.is_project_protocol() {
        return Err(format!(
            "--project refuses non-c1-project config-hash `{h}` — \
             use a c1-project* hash or omit --config-hash"
        ));
    }
    if flags.spike_s && !c.is_spike_s_protocol() {
        return Err(format!(
            "--spike-s refuses non-c1-spike-s config-hash `{h}` — \
             use a c1-spike-s* hash or omit --config-hash"
        ));
    }
    if flags.spike && !flags.spike_s {
        // `--spike` alone must not accept calibrated spike-s presets.
        if !c.is_spike_protocol() || c.is_spike_s_protocol() {
            return Err(format!(
                "--spike refuses non-c1-spike (v6) config-hash `{h}` — \
                 use a c1-spike* (not spike-s) hash or omit --config-hash"
            ));
        }
    }
    if flags.isolation && !c.is_isolation_protocol() {
        return Err(format!(
            "--isolation refuses non-c1-iso config-hash `{h}` — \
             use a c1-iso* hash or omit --config-hash"
        ));
    }
    if let Some(name) = flags.sensitivity {
        if !c.is_sensitivity_protocol() {
            return Err(format!(
                "--sensitivity refuses non-c1-sens config-hash `{h}` — \
                 use a c1-sens-* hash or omit --config-hash"
            ));
        }
        let exp = c.experiment.as_str();
        let ok = match name {
            "temporal-pc" | "temporal_pc" | "tpc" => exp.contains("temporal"),
            "capacity" | "cap" => exp.contains("capacity"),
            _ => true,
        };
        if !ok {
            return Err(format!(
                "--sensitivity {name} refuses config-hash `{h}` (experiment `{}`) — \
                 hash must match that sensitivity family",
                c.experiment
            ));
        }
    }
    Ok(())
}
