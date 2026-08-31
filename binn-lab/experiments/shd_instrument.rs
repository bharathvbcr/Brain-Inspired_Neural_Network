//! Native backend and parity probe for the SHD instrument calibration.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use binn_data::{
    frame_events, read_event_cache, FramedShdSample, FrequencyGeometry, ShdEventContract,
};
use binn_lab::{authorize_campaign, CampaignKind};
use rayon::prelude::*;

use binn_lab::gradient_clip::{clip_by_global_norm, ClipOutcome};
use binn_lab::timestamp::{iso8601_utc, unix_seconds};
use binn_learn::shd_attention::{AttentionConfig, AttentionParams};
use binn_learn::shd_matched_arms::ArmAdam;
use binn_learn::{apply_temporal, TemporalAudit, TemporalCondition};
use binn_learn::{
    load_epoch_orders, one_cycle_lr, shd_matched_loss_and_gradient_arm,
    shd_matched_loss_and_gradient_arm_scaled_prepared, MatchedArm, MatchedShdSample, PortableRng,
    ShdArmGradient, ShdArmWeightLayout, ShdArmWeights, ShdMatchedWeights,
};

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("shd-instrument: {error}");
            ExitCode::from(2)
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    let command = args.first().map(String::as_str).unwrap_or("help");
    match command {
        "fixture-hashes" => {
            authorize_campaign(CampaignKind::Parity)?;
            fixture_hashes(&args[1..])
        }
        "parity" => {
            authorize_campaign(CampaignKind::Parity)?;
            parity(&args[1..])
        }
        "init" => {
            authorize_campaign(CampaignKind::Calibration)?;
            init(&args[1..])
        }
        "train-cell" => {
            authorize_campaign(CampaignKind::Calibration)?;
            train_cell(&args[1..])
        }
        // Validity check on the instrument, not a campaign: it trains nothing
        // and makes no accuracy claim. See PREREG_2026-08-02 §5 positive control.
        "temporal-sensitivity" => {
            authorize_campaign(CampaignKind::HarnessValidation)?;
            temporal_sensitivity(&args[1..])
        }
        "help" | "-h" | "--help" => {
            print_help();
            Ok(())
        }
        other => Err(format!("unknown command: {other}")),
    }
}

fn print_help() {
    eprintln!(
        "shd-instrument COMMAND\n\n\
         Commands:\n\
           fixture-hashes --events FILE --contract ID --geometry ID\n\
           parity --events FILE --index N --contract ID --geometry ID --weights FILE --out FILE\n\
           init --n-inputs N --hidden N --classes N --seed N --epochs N --n-train N \\\n+                --weights FILE --orders FILE\n\
           train-cell --train-events FILE --test-events FILE --contract ID --geometry ID \\\n+                --weights FILE --orders FILE --epochs N --out FILE\n\n\
         Optional on parity/init/train-cell:\n\
           --arm ff+fixed|ff+alif|rec+fixed|rec+alif   (default ff+fixed)\n\
           any arm above with a +attn suffix adds the time-axis attention read-out\n\
         Optional on init, attention arms only:\n\
           --seed N          provenance label recorded in the cell (no effect on results)\n\
           --attn-dim N      attention width, even (default 32)\n\
           --attn-layers N   attention blocks (default 1)\n\
         Optional on init, recurrent arms only:\n\
           --w-rec-scale F   multiplies the Glorot recurrent draw (default 1.0)\n\
         Optional on train-cell:\n\
           --temporal intact|bin-shuffled|channel-shuffled|reversed  (default intact)\n\
           --temporal-seed N   (required unless --temporal intact)\n\
           --clip-grad-norm F  global-norm clipping of the batch gradient (default: off)\n\
           --clip-sample-grad-norm F  global-norm clipping of each sample gradient,\n\
                               before accumulation (default: off)\n\
           --surrogate-scale F multiplies the surrogate gain (default 1.0 = unchanged)"
    );
}

fn fixture_hashes(args: &[String]) -> Result<(), String> {
    let events = required_path(args, "--events")?;
    let contract = parse_contract(&required(args, "--contract")?)?;
    let geometry = parse_geometry(&required(args, "--geometry")?)?;
    for (index, sample) in read_event_cache(&events, None)?.iter().enumerate() {
        let framed = frame_events(sample, contract, geometry);
        println!("{index}\t{:016x}", framed.fingerprint());
    }
    Ok(())
}

fn parity(args: &[String]) -> Result<(), String> {
    let events = required_path(args, "--events")?;
    let index = parse_usize(&required(args, "--index")?, "--index")?;
    let contract = parse_contract(&required(args, "--contract")?)?;
    let geometry = parse_geometry(&required(args, "--geometry")?)?;
    let out = required_path(args, "--out")?;
    // `ArmWeights::load` dispatches on magic: SHDWGT1 yields ff+fixed with an
    // empty recurrent block, so existing fixtures keep working untouched.
    let weights = ShdArmWeights::load(&required_path(args, "--weights")?)?;
    if let Some(arm) = optional_arm(args)? {
        if arm != weights.arm {
            return Err(format!(
                "--arm {} disagrees with the weight file, which carries {}",
                arm.label(),
                weights.arm.label()
            ));
        }
    }
    let samples = read_event_cache(&events, Some(index + 1))?;
    let sample = samples
        .get(index)
        .ok_or_else(|| format!("fixture index {index} out of range"))?;
    let framed = frame_events(sample, contract, geometry);
    let matched = to_matched(&framed);
    let (forward, gradient) = shd_matched_loss_and_gradient_arm(&weights, &matched)?;
    let mut updated = weights.clone();
    let mut optimizer = ArmAdam::new(&updated);
    optimizer.update(&mut updated, &gradient, 1e-3, 1e-5);
    // `arm`, `grad_w_rec` and `updated_w_rec` are appended, never inserted, so
    // an ff+fixed payload stays a superset-compatible read for existing tools.
    let recurrent_fields = if weights.arm.recurrent {
        format!(
            ",\"grad_w_rec\":{},\"updated_w_rec\":{}",
            json_f32(&gradient.w_rec),
            json_f32(&updated.w_rec)
        )
    } else {
        String::new()
    };
    // Same append-only discipline as `recurrent_fields`.
    let attention_fields = match weights.attention_config() {
        Some(config) => format!(
            ",\"attn_dim\":{},\"attn_layers\":{}",
            config.d_model, config.layers
        ),
        None => String::new(),
    };
    let json = format!(
        "{{\"frame_hash\":\"{:016x}\",\"valid_steps\":{},\"dt_ms\":{:.9},\
         \"loss\":{:.9},\"prediction\":{},\"membrane\":{},\"spikes\":{},\"rates\":{},\
         \"logits\":{},\"grad_w_in\":{},\"grad_w_out\":{},\"grad_b_out\":{},\
         \"updated_w_in\":{},\"updated_w_out\":{},\"updated_b_out\":{},\"arm\":\"{}\"{}{}}}\n",
        framed.fingerprint(),
        framed.valid_steps(),
        framed.dt_ms,
        forward.loss,
        forward.prediction,
        json_f32(&forward.membrane),
        json_f32(&forward.spikes),
        json_f32(&forward.rates),
        json_f32(&forward.logits),
        json_f32(&gradient.base.w_in),
        json_f32(&gradient.base.w_out),
        json_f32(&gradient.base.b_out),
        json_f32(&updated.base.w_in),
        json_f32(&updated.base.w_out),
        json_f32(&updated.base.b_out),
        weights.arm.label(),
        recurrent_fields,
        attention_fields,
    );
    atomic_write(&out, json.as_bytes())
}

fn init(args: &[String]) -> Result<(), String> {
    reject_unknown_flags(args, INIT_FLAGS)?;
    let n_inputs = require_positive(
        parse_usize(&required(args, "--n-inputs")?, "--n-inputs")?,
        "--n-inputs",
    )?;
    let hidden = require_positive(
        parse_usize(&required(args, "--hidden")?, "--hidden")?,
        "--hidden",
    )?;
    let n_classes = require_positive(
        parse_usize(&required(args, "--classes")?, "--classes")?,
        "--classes",
    )?;
    let seed = parse_u64(&required(args, "--seed")?, "--seed")?;
    let epochs = require_positive(
        parse_usize(&required(args, "--epochs")?, "--epochs")?,
        "--epochs",
    )?;
    let n_train = require_positive(
        parse_usize(&required(args, "--n-train")?, "--n-train")?,
        "--n-train",
    )?;
    let weights_path = required_path(args, "--weights")?;
    let orders_path = required_path(args, "--orders")?;
    if let Some(parent) = weights_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    if let Some(parent) = orders_path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let arm = optional_arm(args)?.unwrap_or(MatchedArm::FF_FIXED);
    let base = ShdMatchedWeights::deterministic(n_inputs, hidden, n_classes, seed);
    if arm == MatchedArm::FF_FIXED {
        // Byte-identical to the shipped writer: existing initialization files
        // must stay reproducible (Gate F).
        base.save(&weights_path)?;
    } else {
        // Recurrent block drawn from the same PortableRng lineage, offset so it
        // cannot alias the readout stream. Glorot, then scaled.
        //
        // `--w-rec-scale` exists because the comment that used to sit here said
        // the registered scale "is set by the G8 pilot, not here" while offering
        // no way to vary it — so the pilot could not be run. It is needed:
        // measured 2026-08-03 at h128/published-2ms, unscaled Glorot gives
        // `rec+fixed` an epoch-1 mean gradient norm of 9.8e12 with a flat loss.
        // See MEASUREMENT_2026-08-03_RECURRENT_ARM_STABILITY.md.
        //
        // The default is 1.0, so every previously generated file is reproduced
        // byte-for-byte and the flag changes nothing unless passed. The draws
        // come from the same stream and are scaled after the fact, so a run at
        // scale s uses the same random lineage as one at scale 1.
        let scale = optional_f32(args, "--w-rec-scale")?.unwrap_or(1.0);
        if !(scale.is_finite() && scale > 0.0) {
            return Err(format!(
                "--w-rec-scale must be finite and positive, got {scale}"
            ));
        }
        let w_rec = if arm.recurrent {
            let mut rng = PortableRng::new(seed ^ 0x5245_4300_0000_0001);
            let limit = (6.0_f32 / (hidden + hidden) as f32).sqrt();
            (0..hidden * hidden)
                .map(|_| rng.uniform(-limit, limit) * scale)
                .collect()
        } else {
            Vec::new()
        };
        if arm.attention {
            // Its own `PortableRng` lineage, so an attention arm's base weights
            // are bit-identical to the same arm without attention at the same
            // seed. Any difference between the two is then the read-out, not a
            // different initialisation.
            let config = AttentionConfig::new(
                optional_usize(args, "--attn-dim")?.unwrap_or(binn_learn::DEFAULT_ATTENTION_DIM),
                optional_usize(args, "--attn-layers")?
                    .unwrap_or(binn_learn::DEFAULT_ATTENTION_LAYERS),
            )?;
            let attn = AttentionParams::deterministic(
                hidden,
                n_classes,
                config,
                seed ^ 0x4154_544E_0000_0000,
            )?;
            ShdArmWeights::new_attentive(base, arm, w_rec, attn)?.save(&weights_path)?;
        } else {
            ShdArmWeights::new(base, arm, w_rec)?.save(&weights_path)?;
        }
    }
    let mut rng = PortableRng::new(seed ^ 0x0D3E_45E5_51D0_0001);
    let mut orders = Vec::with_capacity(epochs);
    for _ in 0..epochs {
        let mut order: Vec<usize> = (0..n_train).collect();
        rng.shuffle(&mut order);
        orders.push(order);
    }
    binn_learn::save_epoch_orders(&orders_path, &orders)
}

/// One sample's evaluation: its index, its per-class logits, and whether those
/// logits were finite.
///
/// Named rather than written inline because `-D clippy::type_complexity`
/// rejects the tuple, and because the shape does not read for itself: the
/// boolean is a finiteness flag decided inside the evaluation closure, not a
/// class label.
type SampleEval = Result<(usize, Vec<f32>, bool), String>;

/// How many samples are in flight at once inside a batch.
///
/// Bounds peak memory to this many per-sample gradients. Larger than any core
/// count we run a single cell on, so it never limits parallelism; smaller than
/// `batch_size`, so it bounds memory. Changing it cannot change any result —
/// sub-chunks are consumed in batch order.
const PARALLEL_CHUNK: usize = 64;

#[derive(Default)]
struct TrainDiagnostics {
    loss_sum: f64,
    samples: usize,
    gradient_norm_sum: f64,
    update_rms_sum: f64,
    optimizer_steps: usize,
    non_finite_events: usize,
    /// Steps where clipping actually bound. Reported so a clipped run
    /// cannot be mistaken for an unclipped one.
    clipped_steps: usize,
    /// Steps whose norm was non-finite, which clipping cannot rescue.
    unclippable_steps: usize,
    /// Sample gradients where per-sample clipping actually bound. Reported for
    /// the same reason as `clipped_steps`: a clipped run must not be readable
    /// as an unclipped one.
    clipped_samples: usize,
}

/// Positive control for the temporal-information campaign.
///
/// # Why this has to exist
///
/// `PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION` concludes H1 from a *null*:
/// shuffling time does not move accuracy. A null is only interpretable if the
/// measurement could have detected an effect in the first place. Validity gate
/// 5.1 proves the manipulation changes timing and not rate — it says nothing
/// about whether timing ever reaches the loss.
///
/// The failure mode this guards against is concrete. Framing at 2 ms with an
/// `adjacent-sum-5` geometry could attenuate temporal structure before the
/// network sees it. Then every condition scores the same, H1 "passes" as an
/// artifact of the framing, and the same artifact would independently explain
/// the resolution invariance that motivated the campaign. The experiment would
/// be measuring its own preprocessing.
///
/// So: run the forward pass under each condition against the *untrained*
/// registered initialization and measure how far the hidden representation
/// moves. Untrained on purpose — this isolates "can timing reach the loss"
/// from "does the trained model choose to use it", which is the question the
/// campaign itself answers.
///
/// Reads a divergence near zero for `bin-shuffled` as: the pipeline destroys
/// temporal information, and the 24-cell campaign must not be run as designed.
fn temporal_sensitivity(args: &[String]) -> Result<(), String> {
    let test_events = required_path(args, "--test-events")?;
    let contract = parse_contract(&required(args, "--contract")?)?;
    let geometry = parse_geometry(&required(args, "--geometry")?)?;
    let weights_path = required_path(args, "--weights")?;
    let out = required_path(args, "--out")?;
    let samples = require_positive(
        optional_usize(args, "--samples")?.unwrap_or(256),
        "--samples",
    )?;
    let temporal_seed = optional_u64(args, "--temporal-seed")?.unwrap_or(5170001);
    let started = Instant::now();

    let weights = ShdArmWeights::load(&weights_path)?;
    let raw = read_event_cache(&test_events, Some(samples))?;
    if raw.is_empty() {
        return Err("empty test event cache".into());
    }
    // `read_event_cache` clamps silently to what the file holds, so asking for
    // 256 samples of a 100-sample cache returns 100 and nothing says so. Every
    // metric below is a mean over whatever arrived, and a write-up quoting the
    // requested count would then be wrong with no way to notice. Fail loudly
    // instead: a short cache is a setup error, not a smaller experiment.
    if raw.len() < samples {
        return Err(format!(
            "requested {samples} samples but {} holds only {}; refusing to \
             report a mean over a different number than was asked for",
            test_events.display(),
            raw.len(),
        ));
    }
    let framed: Vec<MatchedShdSample> = raw
        .iter()
        .map(|sample| to_matched(&frame_events(sample, contract, geometry)))
        .collect();

    let conditions = TemporalCondition::ALL;
    let arms = conditions.len();
    let mut report = Vec::new();

    // Sample-outer, condition-inner. The `intact` forward is the reference for
    // every condition, so computing it inside the condition loop recomputed it
    // four times per sample — 8 forward passes per sample where 5 suffice.
    //
    // Each condition's accumulators still receive samples in ascending index
    // order, exactly as before, so the reported means are bit-identical.
    let mut audits = vec![TemporalAudit::default(); arms];
    let mut spike_hamming = vec![0.0_f64; arms];
    let mut membrane_rel_l2 = vec![0.0_f64; arms];
    let mut loss_absolute = vec![0.0_f64; arms];
    let mut rate_rel_l1 = vec![0.0_f64; arms];
    let mut prediction_changed = vec![0_usize; arms];
    let mut counted = 0_usize;

    for (index, base) in framed.iter().enumerate() {
        let intact = shd_matched_loss_and_gradient_arm(&weights, base)?.0;
        for (slot, condition) in conditions.into_iter().enumerate() {
            let (
                audit,
                spike_hamming,
                membrane_rel_l2,
                loss_absolute,
                rate_rel_l1,
                prediction_changed,
            ) = (
                &mut audits[slot],
                &mut spike_hamming[slot],
                &mut membrane_rel_l2[slot],
                &mut loss_absolute[slot],
                &mut rate_rel_l1[slot],
                &mut prediction_changed[slot],
            );
            let mut moved = base.clone();
            // Seed per sample so two samples never share a permutation, matching
            // how the campaign derives its manipulation seeds.
            let audit_one = apply_temporal(
                &mut moved,
                condition,
                temporal_seed.wrapping_add(index as u64),
            )?;
            audit.merge(&audit_one);
            let shifted = shd_matched_loss_and_gradient_arm(&weights, &moved)?.0;

            let differing = intact
                .spikes
                .iter()
                .zip(shifted.spikes.iter())
                .filter(|(a, b)| a != b)
                .count();
            *spike_hamming += differing as f64 / intact.spikes.len().max(1) as f64;

            let numerator: f64 = intact
                .membrane
                .iter()
                .zip(shifted.membrane.iter())
                .map(|(a, b)| ((a - b) as f64).powi(2))
                .sum();
            let denominator: f64 = intact.membrane.iter().map(|a| (*a as f64).powi(2)).sum();
            if denominator > 0.0 {
                *membrane_rel_l2 += (numerator / denominator).sqrt();
            }

            let rate_numerator: f64 = intact
                .rates
                .iter()
                .zip(shifted.rates.iter())
                .map(|(a, b)| (a - b).abs() as f64)
                .sum();
            let rate_denominator: f64 = intact.rates.iter().map(|a| a.abs() as f64).sum();
            if rate_denominator > 0.0 {
                *rate_rel_l1 += rate_numerator / rate_denominator;
            }

            *loss_absolute += (intact.loss - shifted.loss).abs() as f64;
            if intact.prediction != shifted.prediction {
                *prediction_changed += 1;
            }
        }
        counted += 1;
    }

    let n = counted.max(1) as f64;
    for (slot, condition) in conditions.into_iter().enumerate() {
        let audit = &audits[slot];
        report.push(format!(
            "    {{\n      \"condition\": \"{}\",\n      \
             \"mean_spike_hamming\": {:.9},\n      \
             \"mean_membrane_rel_l2\": {:.9},\n      \
             \"mean_rate_rel_l1\": {:.9},\n      \
             \"mean_abs_loss_delta\": {:.9},\n      \
             \"prediction_changed_fraction\": {:.9},\n      \
             \"counts_preserved\": {},\n      \
             \"relocated_fraction\": {:.9}\n    }}",
            condition.label(),
            spike_hamming[slot] / n,
            membrane_rel_l2[slot] / n,
            rate_rel_l1[slot] / n,
            loss_absolute[slot] / n,
            prediction_changed[slot] as f64 / n,
            audit.counts_preserved,
            audit.relocated_fraction,
        ));
    }

    let payload = format!(
        "{{\n  \"schema\": \"shd-temporal-sensitivity-v1\",\n  \
         \"purpose\": \"positive control: does timing reach the hidden representation at all\",\n  \
         \"weights\": \"{}\",\n  \"weights_fingerprint\": \"{:016x}\",\n  \"samples\": {},\n  \
         \"temporal_seed\": {},\n  \"wall_secs\": {:.6},\n  \"conditions\": [\n{}\n  ]\n}}\n",
        weights_path.display(),
        weight_fingerprint(&weights),
        counted_samples(&framed),
        temporal_seed,
        started.elapsed().as_secs_f64(),
        report.join(",\n"),
    );
    fs::write(&out, payload).map_err(|error| error.to_string())?;
    println!("temporal-sensitivity -> {}", out.display());
    Ok(())
}

fn counted_samples(framed: &[MatchedShdSample]) -> usize {
    framed.len()
}

/// Every flag `init` accepts. One owner, so the subcommand and the test that
/// pins the real campaign invocations can never drift apart.
const INIT_FLAGS: &[&str] = &[
    "--n-inputs",
    "--hidden",
    "--classes",
    "--seed",
    "--epochs",
    "--n-train",
    "--weights",
    "--orders",
    "--arm",
    "--attn-dim",
    "--attn-layers",
    "--w-rec-scale",
];

/// Every flag `train-cell` accepts.
const TRAIN_CELL_FLAGS: &[&str] = &[
    "--train-events",
    "--test-events",
    "--contract",
    "--geometry",
    "--epochs",
    "--weights",
    "--orders",
    "--out",
    "--arm",
    "--surrogate-scale",
    "--clip-grad-norm",
    "--clip-sample-grad-norm",
    "--temporal",
    "--temporal-seed",
    "--seed",
    "--max-train",
    "--max-test",
    "--save-final-weights",
];

/// Refuse a flag this subcommand does not understand.
///
/// The parser finds flags by searching for them, so an unrecognised one used to
/// be **silently ignored**. A typo in `--surrogate-scale` or `--attn-dim` would
/// therefore run the cell at the default and emit a result that is wrong but
/// entirely plausible — no panic, no warning, and a cell JSON that looks exactly
/// like a healthy one. That is the "check that cannot fail" shape, applied to
/// the command line.
///
/// Values are skipped by position, so a value that happens to start with `--`
/// (a negative number never does here) is not mistaken for a flag.
fn reject_unknown_flags(args: &[String], allowed: &[&str]) -> Result<(), String> {
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if let Some(stripped) = arg.strip_prefix("--") {
            if !allowed.contains(&format!("--{stripped}").as_str()) {
                return Err(format!(
                    "unknown flag {arg}; this subcommand accepts: {}",
                    allowed.join(" ")
                ));
            }
            index += 2; // flag and its value
        } else {
            index += 1;
        }
    }
    Ok(())
}

fn train_cell(args: &[String]) -> Result<(), String> {
    reject_unknown_flags(args, TRAIN_CELL_FLAGS)?;
    // Provenance label only — it does NOT touch the computation. The seed that
    // determines the run was consumed by `init`, and the weights format carries
    // no seed, so until now a cell recorded every parameter *except* which seed
    // produced it: the seed lived only in the filename. Every paired statistic
    // in a campaign ("positive in 12 of 12 seeds") depends on that filename
    // being right, and nothing inside the cell could confirm it.
    //
    // Deliberately optional. Omitting it emits byte-identical output to before
    // this flag existed, so Gate F still regresses every recorded cell.
    let provenance_seed = optional_usize(args, "--seed")?;
    let train_events = required_path(args, "--train-events")?;
    let test_events = required_path(args, "--test-events")?;
    let contract = parse_contract(&required(args, "--contract")?)?;
    let geometry = parse_geometry(&required(args, "--geometry")?)?;
    let epochs = require_positive(
        parse_usize(&required(args, "--epochs")?, "--epochs")?,
        "--epochs",
    )?;
    // Absent means no clipping and bit-identical behaviour to before the
    // flag existed. A non-positive or non-finite threshold is rejected
    // rather than silently disabling clipping on a run that asked for it.
    // Multiplies MATCHED_SURROGATE_ALPHA. 1.0 (the default) is bit-identical to
    // before this flag existed; 0.4 gives a peak per-timestep backward gain of
    // exactly 1.0. Registered in
    // AMENDMENT_2026-08-05_SURROGATE_GAIN_FOR_RECURRENT.md, which records that a
    // gradient computed at any other scale is NOT comparable to the 216
    // recorded cells.
    let surrogate_scale = match optional_f32(args, "--surrogate-scale")? {
        None => 1.0_f32,
        Some(value) if value.is_finite() && value > 0.0 => value,
        Some(value) => {
            return Err(format!(
                "--surrogate-scale must be finite and positive, got {value}"
            ))
        }
    };
    let clip_grad_norm = match optional_f32(args, "--clip-grad-norm")? {
        None => None,
        Some(value) if value.is_finite() && value > 0.0 => Some(f64::from(value)),
        Some(value) => {
            return Err(format!(
                "--clip-grad-norm must be finite and positive, got {value}"
            ))
        }
    };
    // Per-sample clipping, and the reason it is separate from `--clip-grad-norm`.
    //
    // `AMENDMENT_2026-08-05_SURROGATE_GAIN_FOR_RECURRENT.md` §1 records batch
    // clipping as "never reached - abort fires on a per-sample gradient,
    // upstream", and the code was never changed to match that finding. The
    // recurrent failure compounds *inside* the per-sample backward: by the time
    // a batch gradient exists, a single sample has already produced non-finite
    // entries and the accumulation loop has returned an error. A threshold that
    // only ever sees the batch therefore cannot bind on the cells it was added
    // for.
    //
    // Clipping each sample before accumulation acts one level lower, where the
    // growth actually happens, and bounds every addend rather than their sum.
    // It does not replace `--clip-grad-norm`: the two compose, and a run may
    // pass both.
    //
    // **Off is bit-identical to before this existed.** With no flag the value
    // is `None`, no branch below is entered, and no arithmetic touches the
    // sample gradient. Gate F over the recorded cells is the binding check, as
    // it was for batch clipping.
    let clip_sample_grad_norm = match optional_f32(args, "--clip-sample-grad-norm")? {
        None => None,
        Some(value) if value.is_finite() && value > 0.0 => Some(f64::from(value)),
        Some(value) => {
            return Err(format!(
                "--clip-sample-grad-norm must be finite and positive, got {value}"
            ))
        }
    };
    let weights_path = required_path(args, "--weights")?;
    let orders_path = required_path(args, "--orders")?;
    let out = required_path(args, "--out")?;
    let train_limit = optional_usize(args, "--max-train")?;
    let test_limit = optional_usize(args, "--max-test")?;
    let started = Instant::now();

    let train_raw = read_event_cache(&train_events, train_limit)?;
    let test_raw = read_event_cache(&test_events, test_limit)?;
    if train_raw.is_empty() || test_raw.is_empty() {
        return Err("empty train/test event cache".into());
    }
    let train: Vec<MatchedShdSample> = train_raw
        .iter()
        .map(|sample| to_matched(&frame_events(sample, contract, geometry)))
        .collect();
    let test: Vec<MatchedShdSample> = test_raw
        .iter()
        .map(|sample| to_matched(&frame_events(sample, contract, geometry)))
        .collect();
    // --- temporal-information condition (PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION) ---
    let condition = optional_temporal(args)?.unwrap_or(TemporalCondition::Intact);
    let temporal_seed = optional_u64(args, "--temporal-seed")?;
    if !condition.is_identity() && temporal_seed.is_none() {
        return Err(
            "--temporal other than 'intact' requires --temporal-seed for reproducibility".into(),
        );
    }
    let temporal_seed = temporal_seed.unwrap_or(0);
    let mut train = train;
    let mut test = test;
    let mut audit = TemporalAudit::default();
    if !condition.is_identity() {
        // Train and test are manipulated with disjoint seed streams so a
        // coincidental shared permutation cannot leak structure between them.
        for (index, sample) in train.iter_mut().enumerate() {
            let per_sample = apply_temporal(
                sample,
                condition,
                temporal_seed ^ 0x1111_0000_0000_0000 ^ index as u64,
            )?;
            audit.merge(&per_sample);
        }
        for (index, sample) in test.iter_mut().enumerate() {
            let per_sample = apply_temporal(
                sample,
                condition,
                temporal_seed ^ 0x2222_0000_0000_0000 ^ index as u64,
            )?;
            audit.merge(&per_sample);
        }
        // Prereg gate 5.1 is enforced inside apply_temporal, which errors rather
        // than returning. A manipulation that relocated almost nothing would
        // pass that gate while doing nothing, so check it separately.
        if audit.relocated_fraction < 0.5 {
            return Err(format!(
                "temporal condition {} relocated only {:.4} of entries - the manipulation \
                 is not doing what it claims",
                condition.label(),
                audit.relocated_fraction
            ));
        }
    }
    let train = train;
    let test = test;

    let mut weights = ShdArmWeights::load(&weights_path)?;
    if let Some(arm) = optional_arm(args)? {
        if arm != weights.arm {
            return Err(format!(
                "--arm {} disagrees with the weight file, which carries {}",
                arm.label(),
                weights.arm.label()
            ));
        }
    }
    let orders = load_epoch_orders(&orders_path)?;
    if orders.len() < epochs {
        return Err(format!(
            "order file has {} epochs, requested {epochs}",
            orders.len()
        ));
    }
    if orders
        .iter()
        .take(epochs)
        .any(|order| order.len() != train.len())
    {
        return Err("order file n_train does not match loaded training set".into());
    }

    let batch_size = 256usize;
    let total_steps = epochs * train.len().div_ceil(batch_size);
    let mut optimizer = ArmAdam::new(&weights);
    let mut diagnostics = TrainDiagnostics::default();
    let mut global_step = 0usize;
    // Convergence telemetry. The completed matrix shows accuracy still rising
    // with width at h512, which leaves open whether 100 epochs is enough or the
    // 0.7151 ceiling is partly an undertraining artefact. Per-epoch traces make
    // that answerable from the cell record instead of by re-running.
    let mut epoch_loss: Vec<f64> = Vec::with_capacity(epochs);
    let mut epoch_gradient_norm: Vec<f64> = Vec::with_capacity(epochs);
    // Max alongside the mean, because the mean alone cannot distinguish the two
    // situations that call for opposite fixes: a gradient that is large on every
    // optimizer step (clip it) versus one that is fine on 31 steps out of 32 and
    // enormous on one (find out what is in that batch). Both produce the same
    // `epoch_mean_gradient_norm`, and `rec+fixed` produces values around 1e12
    // that no other field distinguishes from ordinary poor learning. See
    // MEASUREMENT_2026-08-03_RECURRENT_ARM_STABILITY.md §3.3.
    //
    // Additive: Gate F compares an explicit field list, so recorded cells are
    // unaffected, and the training loop is untouched — this reads a value it
    // was already computing.
    let mut epoch_max_gradient_norm: Vec<f64> = Vec::with_capacity(epochs);
    let mut epoch_max_gradient_step: Vec<f64> = Vec::with_capacity(epochs);
    for order in orders.iter().take(epochs) {
        let epoch_loss_start = diagnostics.loss_sum;
        let epoch_samples_start = diagnostics.samples;
        let epoch_norm_start = diagnostics.gradient_norm_sum;
        let epoch_steps_start = diagnostics.optimizer_steps;
        let mut epoch_peak_norm = 0.0_f64;
        // Which step held the peak, so the two remaining explanations can be
        // told apart. `order` is reshuffled every epoch, so if the peak is a
        // property of particular *samples* its step index should wander; if it
        // is a warmup transient it should pin to step 0 every epoch.
        let mut epoch_peak_step = 0_usize;
        let mut step_in_epoch = 0_usize;
        // Counter kept explicit rather than via `enumerate`: it is read and
        // reported alongside `global_step`, and this loop is a scientific
        // instrument where an off-by-one would be silent.
        #[allow(clippy::explicit_counter_loop)]
        for batch in order.chunks(batch_size) {
            let mut gradient = ShdArmGradient::zeros_like(&weights);
            // The optimiser mutates weights once per batch, so this layout is
            // valid for every sample gradient below and is rebuilt before the
            // next batch. Previously each sample recopied/transposed the same
            // input matrix (and recurrent matrix when present).
            let weight_layout = ShdArmWeightLayout::prepare(&weights);
            // Per-sample forward/backward runs in parallel; the *accumulation*
            // stays strictly in batch order.
            //
            // This is what lets a cell use more than one core. It is
            // bit-identical to the serial loop it replaces, and that is a
            // property of the reduction, not a hope: `par_iter().collect()` on
            // an indexed parallel iterator preserves order, so `add_assign` sees
            // exactly the addends it saw before, in exactly the same sequence.
            // Float addition is not associative, so anything less than that
            // ordering guarantee would silently change every recorded cell.
            // Gate F over the recorded cells is the binding check, and
            // `parallelism_is_bit_identical_to_serial` pins thread-count
            // independence directly.
            //
            // Chunked rather than one `par_iter` over the whole batch so peak
            // memory is bounded by `PARALLEL_CHUNK` gradients rather than by
            // `batch_size` of them — at h1024 with a recurrent block a single
            // sample gradient is ~4 MB, and 256 of them live at once would be
            // 1 GB per cell with several cells sharing a host. Chunking cannot
            // perturb the result: sub-chunks are consumed in order too.
            //
            // Only `loss` is carried out of the forward. Keeping the whole
            // `MatchedForward` would hold `membrane` and `spikes`
            // (`t_steps * hidden` each) alive for every in-flight sample.
            for chunk in batch.chunks(PARALLEL_CHUNK) {
                let computed = ordered_sample_gradients(
                    &weights,
                    &weight_layout,
                    &train,
                    chunk,
                    surrogate_scale,
                );
                for outcome in computed {
                    let (loss, mut sample_gradient) = outcome?;
                    diagnostics.loss_sum += f64::from(loss);
                    diagnostics.samples += 1;
                    if !loss.is_finite() || !sample_gradient.all_finite() {
                        return Err(format!(
                            "non-finite training value at optimizer step {global_step}"
                        ));
                    }
                    // Per-sample clipping. Placed after the finite check and
                    // before accumulation: scaling by `threshold / norm` cannot
                    // rescue a gradient that already holds inf or NaN, so the
                    // hard error stays as the last line of defence and this
                    // bounds the growth that would otherwise reach it.
                    if let Some(threshold) = clip_sample_grad_norm {
                        match clip_by_global_norm(&mut sample_gradient, threshold) {
                            ClipOutcome::Bound => diagnostics.clipped_samples += 1,
                            ClipOutcome::Untouched => {}
                            // Every entry is finite - checked above - but their
                            // sum of squares overflowed. `threshold / inf` is
                            // zero, so scaling here would silently delete the
                            // sample from the batch and leave a cell that looks
                            // trained. Refuse instead, and name the case.
                            //
                            // Deliberately not counted into a cell field: this
                            // branch returns, so no cell is ever emitted and any
                            // such field would be a constant zero masquerading
                            // as evidence. The failure log is the record.
                            ClipOutcome::NormOverflowed => {
                                return Err(format!(
                                    "sample gradient norm overflowed with all entries finite \
                                     at optimizer step {global_step}; per-sample clipping \
                                     cannot scale it and will not drop it"
                                ))
                            }
                        }
                    }
                    gradient.add_assign(&sample_gradient);
                }
            }
            gradient.scale(1.0 / batch.len() as f32);
            let gradient_norm = gradient.l2_norm();
            // Global-norm gradient clipping. **Off unless `--clip-grad-norm` is
            // passed**, and off is bit-identical to before this existed: with no
            // flag, `clip_grad_norm` is `None` and neither the branch below nor
            // `gradient` is touched. That is what keeps Gate F green over the
            // 216 recorded cells, none of which were trained with clipping.
            //
            // Why it exists: `rec+alif` at h512 — the width
            // PREREG_2026-08-02_SHD_BPTT_CEILING_RECALIF registers — produces
            // zero usable cells across three seeds, two aborting on non-finite
            // gradient *entries*. Every other lever was tried and failed:
            // rescaling the recurrent initialisation does not help
            // (non-monotonic across three orders of magnitude, and the scale
            // ranking does not survive reseeding), and the f64 norm fix
            // corrected the *record* without touching the dynamics. Clipping is
            // what remains.
            //
            // `clipped_steps` is counted and reported so a clipped run can
            // never be mistaken for an unclipped one, and so "how often did it
            // bind" is answerable from the cell rather than by re-running.
            // The scale is computed in f64: at h512 the norms reaching this
            // point are ~1e29, and `threshold / norm` in f32 would flush to
            // zero for norms above ~1e38.
            //
            // The rule itself lives in `binn_lab::gradient_clip`, shared with
            // the per-sample site below so the two cannot drift. Behaviour here
            // is unchanged: bind above the threshold, count an unrepresentable
            // norm, touch nothing otherwise. `gradient_norm` above is the
            // **pre-clip** value and stays the one reported.
            if let Some(threshold) = clip_grad_norm {
                match clip_by_global_norm(&mut gradient, threshold) {
                    ClipOutcome::Bound => diagnostics.clipped_steps += 1,
                    // An unrepresentable norm cannot be scaled into range by a
                    // ratio, so clipping cannot rescue this step. Count it and
                    // let the existing non-finite accounting report it.
                    ClipOutcome::NormOverflowed => diagnostics.unclippable_steps += 1,
                    ClipOutcome::Untouched => {}
                }
            }
            let lr = one_cycle_lr(global_step, total_steps, 1e-3, 5e-3);
            let update_rms = optimizer.update(&mut weights, &gradient, lr, 1e-5);
            // `non_finite_events` was declared, read by the pass predicate at
            // the bottom of this function, emitted into every cell — and never
            // incremented anywhere. The predicate's `non_finite_events == 0`
            // clause was therefore vacuous, and cells were written reporting
            // `"mean_gradient_norm":inf` beside `"non_finite_events":0`.
            //
            // The per-sample guard below already hard-errors on a non-finite
            // loss or gradient *entry*. What escaped it is the derived norm:
            // `l2_norm` squares in f32, so entries near 1e19 overflow to
            // infinity while every entry is individually finite. Seed 5170002,
            // `rec+fixed`, scale 1.0 does exactly this. See
            // MEASUREMENT_2026-08-03_RECURRENT_ARM_STABILITY.md.
            if !gradient_norm.is_finite() || !update_rms.is_finite() {
                diagnostics.non_finite_events += 1;
            }
            diagnostics.gradient_norm_sum += f64::from(gradient_norm);
            if f64::from(gradient_norm) > epoch_peak_norm {
                epoch_peak_norm = f64::from(gradient_norm);
                epoch_peak_step = step_in_epoch;
            }
            step_in_epoch += 1;
            diagnostics.update_rms_sum += f64::from(update_rms);
            diagnostics.optimizer_steps += 1;
            global_step += 1;
        }
        epoch_loss.push(
            (diagnostics.loss_sum - epoch_loss_start)
                / (diagnostics.samples - epoch_samples_start).max(1) as f64,
        );
        epoch_gradient_norm.push(
            (diagnostics.gradient_norm_sum - epoch_norm_start)
                / (diagnostics.optimizer_steps - epoch_steps_start).max(1) as f64,
        );
        epoch_max_gradient_norm.push(epoch_peak_norm);
        epoch_max_gradient_step.push(epoch_peak_step as f64);
    }
    // Convergence summary: fractional loss improvement over the final tenth of
    // training. Near zero means converged; materially negative means the cell
    // was still learning when the budget ran out, and the accuracy it reports
    // is a budget artefact rather than a ceiling.
    let tail = (epochs / 10).max(1);
    let tail_improvement = if epoch_loss.len() > tail {
        let earlier = epoch_loss[epoch_loss.len() - tail - 1];
        let later = epoch_loss[epoch_loss.len() - 1];
        if earlier.abs() > 0.0 {
            (later - earlier) / earlier
        } else {
            // Not computable: the reference epoch loss is exactly zero.
            f64::NAN
        }
    } else {
        // Not computable: fewer epochs than the tail window needs.
        f64::NAN
    };
    // Emitted as `null` when it could not be computed, not as `0.0`. The doc
    // comment above defines near-zero as *converged*, so a fabricated 0.0 was a
    // vote of confidence from a measurement that was never taken — and because
    // the consumer takes `min(tails)`, such a cell could never be the worst and
    // so was structurally incapable of reporting UNDERTRAINED. No archived cell
    // carries the fabricated value, so this changes no existing record.
    let tail_improvement = json_scalar(tail_improvement);
    // Optional, and deliberately after every training step: persisting the
    // trained weights lets diagnostics such as `temporal-sensitivity` run
    // against a trained network instead of only the initialization. It writes a
    // file and touches nothing the training loop reads, so cells stay
    // bit-reproducible with or without the flag.
    if let Some(path) = optional_path(args, "--save-final-weights")? {
        weights.save(&path)?;
    }
    let evaluation = evaluate(&weights, &test)?;
    let scientific = evaluation.accuracy >= 0.80
        && evaluation.classes_predicted == weights.base.n_classes
        && evaluation.majority_prediction < 0.30
        && evaluation.silent_fraction <= 0.95
        && evaluation.saturated_fraction <= 0.05
        && diagnostics.non_finite_events == 0
        // A forward that produced NaN or infinite logits cannot have produced a
        // meaningful accuracy, and `non_finite_events` above does not see it:
        // it counts gradient and update excursions during TRAINING.
        && evaluation.non_finite_forward == 0;
    // Appended, never inserted, and absent for the four base arms, so Gate F's
    // explicit `COMPARED_FIELDS` list and every recorded cell are unaffected.
    // Present when the arm is attentive so that a cell can never be read as a
    // plain arm, or compared against one, without the width and depth in hand.
    let provenance_fields = match provenance_seed {
        Some(seed) => format!(",\"seed\":{seed}"),
        None => String::new(),
    };
    let attention_fields = match weights.attention_config() {
        Some(config) => {
            format!(
                ",\"attn_dim\":{},\"attn_layers\":{}",
                config.d_model, config.layers
            )
        }
        None => String::new(),
    };
    let emitted = unix_seconds();
    let result = format!(
        "{{\"schema\":\"shd-cal-cell-v1\",\"backend\":\"rust\",\"arm\":\"{}\",\"contract\":\"{}\",\
         \"geometry\":\"{}\",\"hidden\":{},\"epochs\":{},\"n_train\":{},\"n_test\":{},\
         \"accuracy\":{:.9},\"classes_predicted\":{},\"majority_prediction\":{:.9},\
         \"mean_firing_rate\":{:.9},\"silent_fraction\":{:.9},\"saturated_fraction\":{:.9},\
         \"mean_loss\":{},\"mean_gradient_norm\":{},\"mean_update_rms\":{},\
         \"non_finite_events\":{},\"non_finite_forward\":{},\"surrogate_scale\":{:.9},\"clip_grad_norm\":{},\"clipped_steps\":{},\"unclippable_steps\":{},\"clip_sample_grad_norm\":{},\"clipped_samples\":{},\"temporal_condition\":\"{}\",\"temporal_audit\":{{\"samples\":{},\"counts_preserved\":{},\"relocated_fraction\":{:.9},\"mean_bin_displacement\":{:.9},\"occupied_bins_before\":{:.9},\"occupied_bins_after\":{:.9}}},\"epoch_mean_loss\":{},\"epoch_mean_gradient_norm\":{},\"epoch_max_gradient_norm\":{},\"epoch_max_gradient_step\":{},\"tail_loss_improvement\":{},\"mechanical_status\":\"COMPLETE\",\
         \"scientific_status\":\"{}\",\"wall_secs\":{:.6},\"emitted_unix_s\":{},\"emitted_utc\":\"{}\"{}{}}}\n",
        weights.arm.label(),
        contract.id(),
        geometry.id(),
        weights.base.hidden,
        epochs,
        train.len(),
        test.len(),
        evaluation.accuracy,
        evaluation.classes_predicted,
        evaluation.majority_prediction,
        evaluation.mean_firing_rate,
        evaluation.silent_fraction,
        evaluation.saturated_fraction,
        json_scalar(diagnostics.loss_sum / diagnostics.samples.max(1) as f64),
        json_scalar(diagnostics.gradient_norm_sum / diagnostics.optimizer_steps.max(1) as f64),
        json_scalar(diagnostics.update_rms_sum / diagnostics.optimizer_steps.max(1) as f64),
        diagnostics.non_finite_events,
        evaluation.non_finite_forward,
        surrogate_scale,
        clip_grad_norm.map_or("null".to_string(), |v| format!("{v:.9}")),
        diagnostics.clipped_steps,
        diagnostics.unclippable_steps,
        clip_sample_grad_norm.map_or("null".to_string(), |v| format!("{v:.9}")),
        diagnostics.clipped_samples,
        condition.label(),
        audit.samples,
        audit.counts_preserved,
        audit.relocated_fraction,
        audit.mean_bin_displacement,
        audit.occupied_bins_before,
        audit.occupied_bins_after,
        json_f64(&epoch_loss),
        json_f64(&epoch_gradient_norm),
        json_f64(&epoch_max_gradient_norm),
        json_f64(&epoch_max_gradient_step),
        tail_improvement,
        if scientific { "CELL_PASS" } else { "CELL_FAIL" },
        started.elapsed().as_secs_f64(),
        // WHEN the cell was produced. `wall_secs` above is a duration and says
        // nothing about when, so until these two existed the ordering that
        // carries the campaign's epistemic weight - rule registered before data
        // - could not be checked from the artefact at all, only from S3 upload
        // times and git. Never a measurement: it differs on every run by
        // construction and is excluded from Gate F's explicit field list, which
        // `binn-lab/tests/timestamp_is_not_compared.rs` pins.
        emitted,
        iso8601_utc(emitted),
        attention_fields,
        provenance_fields,
    );
    atomic_write(&out, result.as_bytes())
}

/// Compute independent sample gradients in parallel while preserving the
/// caller's index order for the subsequent floating-point reduction.
fn ordered_sample_gradients(
    weights: &ShdArmWeights,
    weight_layout: &ShdArmWeightLayout,
    train: &[MatchedShdSample],
    indices: &[usize],
    surrogate_scale: f32,
) -> Vec<Result<(f32, ShdArmGradient), String>> {
    indices
        .par_iter()
        .map(|&index| {
            shd_matched_loss_and_gradient_arm_scaled_prepared(
                weights,
                weight_layout,
                &train[index],
                surrogate_scale,
            )
            .map(|(forward, sample_gradient)| (forward.loss, sample_gradient))
        })
        .collect()
}

struct Evaluation {
    accuracy: f64,
    classes_predicted: usize,
    majority_prediction: f64,
    mean_firing_rate: f64,
    silent_fraction: f64,
    saturated_fraction: f64,
    /// Test samples whose forward produced a non-finite logit.
    ///
    /// # The hole this closes
    ///
    /// `AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md` §3 records it verbatim and it
    /// went unfixed until 2026-08-29: *"a cell whose logits are non-finite
    /// still reports a `prediction` and an `accuracy` as if meaningful.
    /// `non_finite_events` counts gradient/update excursions, not forward
    /// ones."*
    ///
    /// `argmax` orders by `total_cmp`, under which NaN sorts **above** every
    /// real, so a NaN logit does not crash and does not abstain — it wins, and
    /// its class is counted as a prediction. The fully-degenerate case was
    /// already caught downstream, because every sample then predicts one class
    /// and the pass predicate requires `classes_predicted == n_classes`. **The
    /// partial case was not caught by anything**: a minority of poisoned
    /// samples leaves the class histogram healthy and silently moves
    /// `accuracy`, which is the campaign's primary quantity, beside
    /// `non_finite_events: 0`.
    ///
    /// This is the register's dominant failure class — a guard that cannot fire
    /// — in the one field every published number is built from.
    non_finite_forward: usize,
}

fn evaluate(weights: &ShdArmWeights, samples: &[MatchedShdSample]) -> Result<Evaluation, String> {
    let mut correct = 0usize;
    let mut predictions = vec![0usize; weights.base.n_classes];
    let mut unit_rate = vec![0.0_f64; weights.base.hidden];
    let weight_layout = ShdArmWeightLayout::prepare(weights);
    // Same discipline as the training loop: evaluate in parallel, accumulate in
    // sample order. `correct` and `predictions` are integer counts and could not
    // care, but `unit_rate` is a float sum and would drift with thread count if
    // the order were not pinned. It feeds `mean_firing_rate`, `silent_fraction`
    // and `saturated_fraction` — three Gate F compared fields.
    let mut non_finite_forward = 0usize;
    for chunk in samples.chunks(PARALLEL_CHUNK) {
        // Finiteness is decided inside the closure, on the logits themselves,
        // and returned as a flag rather than by carrying `logits` out — the
        // vector is `n_classes` long per sample and cloning it would cost the
        // evaluation pass for a boolean. The count is an integer, so unlike
        // `unit_rate` it does not depend on accumulation order.
        let computed: Vec<SampleEval> = chunk
            .par_iter()
            .map(|sample| {
                shd_matched_loss_and_gradient_arm_scaled_prepared(
                    weights,
                    &weight_layout,
                    sample,
                    1.0,
                )
                .map(|(forward, _)| {
                    let finite = forward.logits.iter().all(|value| value.is_finite());
                    (forward.prediction, forward.rates, finite)
                })
            })
            .collect();
        for (sample, outcome) in chunk.iter().zip(computed) {
            let (prediction, rates, logits_finite) = outcome?;
            if !logits_finite {
                non_finite_forward += 1;
            }
            correct += usize::from(prediction == sample.label as usize);
            predictions[prediction] += 1;
            for (unit, rate) in unit_rate.iter_mut().zip(rates) {
                *unit += f64::from(rate);
            }
        }
    }
    for rate in &mut unit_rate {
        *rate /= samples.len() as f64;
    }
    let classes_predicted = predictions.iter().filter(|&&count| count > 0).count();
    let majority_prediction =
        predictions.iter().copied().max().unwrap_or(0) as f64 / samples.len() as f64;
    let mean_firing_rate = unit_rate.iter().sum::<f64>() / weights.base.hidden.max(1) as f64;
    let silent_fraction =
        unit_rate.iter().filter(|&&rate| rate <= 1e-6).count() as f64 / weights.base.hidden as f64;
    let saturated_fraction =
        unit_rate.iter().filter(|&&rate| rate >= 0.95).count() as f64 / weights.base.hidden as f64;
    Ok(Evaluation {
        accuracy: correct as f64 / samples.len() as f64,
        classes_predicted,
        majority_prediction,
        mean_firing_rate,
        silent_fraction,
        saturated_fraction,
        non_finite_forward,
    })
}

fn to_matched(sample: &FramedShdSample) -> MatchedShdSample {
    MatchedShdSample {
        label: sample.label,
        frames: sample
            .frames
            .iter()
            .map(|frame| frame.values.clone())
            .collect(),
        n_inputs: sample.n_inputs,
        dt_ms: sample.dt_ms,
    }
}

/// `--temporal intact|bin-shuffled|channel-shuffled|reversed`. Absent means intact.
fn optional_temporal(args: &[String]) -> Result<Option<TemporalCondition>, String> {
    match args.iter().position(|value| value == "--temporal") {
        Some(index) => {
            let value = args
                .get(index + 1)
                .ok_or_else(|| "--temporal requires a value".to_string())?;
            Ok(Some(TemporalCondition::parse(value)?))
        }
        None => Ok(None),
    }
}

fn optional_u64(args: &[String], flag: &str) -> Result<Option<u64>, String> {
    match args.iter().position(|value| value == flag) {
        Some(index) => {
            let value = args
                .get(index + 1)
                .ok_or_else(|| format!("{flag} requires a value"))?;
            Ok(Some(parse_u64(value, flag)?))
        }
        None => Ok(None),
    }
}

/// `--arm ff+fixed|ff+alif|rec+fixed|rec+alif`. Absent means ff+fixed, so every
/// existing invocation keeps its current behaviour.
fn optional_arm(args: &[String]) -> Result<Option<MatchedArm>, String> {
    match args.iter().position(|value| value == "--arm") {
        Some(index) => {
            let value = args
                .get(index + 1)
                .ok_or_else(|| "--arm requires a value".to_string())?;
            Ok(Some(MatchedArm::parse(value)?))
        }
        None => Ok(None),
    }
}

fn parse_contract(value: &str) -> Result<ShdEventContract, String> {
    if let Some(frame_ms) = value.strip_circumfix("published-", "ms") {
        return ShdEventContract::published(parse_u32(frame_ms, "published frame_ms")?);
    }
    if let Some(frames) = value.strip_prefix("fixed-t") {
        return ShdEventContract::fixed(parse_usize(frames, "fixed frames")?);
    }
    Err(format!("unknown contract: {value}"))
}

fn parse_geometry(value: &str) -> Result<FrequencyGeometry, String> {
    match value {
        "channels-700" | "700" => Ok(FrequencyGeometry::Channels700),
        "adjacent-sum-5" | "140" => Ok(FrequencyGeometry::AdjacentSum5),
        _ => Err(format!("unknown geometry: {value}")),
    }
}

fn required(args: &[String], flag: &str) -> Result<String, String> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .cloned()
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn required_path(args: &[String], flag: &str) -> Result<PathBuf, String> {
    Ok(PathBuf::from(required(args, flag)?))
}

fn optional_path(args: &[String], flag: &str) -> Result<Option<PathBuf>, String> {
    match args.iter().position(|arg| arg == flag) {
        Some(index) => Ok(Some(PathBuf::from(
            args.get(index + 1)
                .ok_or_else(|| format!("{flag} requires a value"))?,
        ))),
        None => Ok(None),
    }
}

fn optional_usize(args: &[String], flag: &str) -> Result<Option<usize>, String> {
    match args.iter().position(|arg| arg == flag) {
        Some(index) => Ok(Some(parse_usize(
            args.get(index + 1)
                .ok_or_else(|| format!("{flag} requires a value"))?,
            flag,
        )?)),
        None => Ok(None),
    }
}

fn optional_f32(args: &[String], flag: &str) -> Result<Option<f32>, String> {
    match args.iter().position(|arg| arg == flag) {
        Some(index) => {
            let raw = args
                .get(index + 1)
                .ok_or_else(|| format!("{flag} requires a value"))?;
            Ok(Some(
                raw.parse().map_err(|error| format!("{flag}: {error}"))?,
            ))
        }
        None => Ok(None),
    }
}

/// Reject a structural size of zero.
///
/// `init` accepted `--hidden 0`, `--classes 0`, `--n-inputs 0` and `--n-train 0`
/// without complaint, and `train-cell` accepted `--epochs 0`. None of them is a
/// smaller experiment; each is a different kind of nothing. `--epochs 0` was the
/// worst: the training loop simply did not execute, and the cell was written
/// with `mechanical_status: COMPLETE`, an empty `epoch_mean_loss`, a
/// `mean_gradient_norm` of exactly 0.0, and a plausible chance-level accuracy of
/// 0.0839 — an untrained model recorded as a finished result, with nothing in
/// the artifact to say so.
fn require_positive(value: usize, name: &str) -> Result<usize, String> {
    if value == 0 {
        return Err(format!(
            "{name} must be at least 1; {name}=0 does not run a smaller \
             experiment, it produces a cell that measured nothing"
        ));
    }
    Ok(value)
}

fn parse_usize(value: &str, name: &str) -> Result<usize, String> {
    value.parse().map_err(|error| format!("{name}: {error}"))
}

fn parse_u32(value: &str, name: &str) -> Result<u32, String> {
    value.parse().map_err(|error| format!("{name}: {error}"))
}

fn parse_u64(value: &str, name: &str) -> Result<u64, String> {
    value.parse().map_err(|error| format!("{name}: {error}"))
}

/// FNV-1a over every parameter, matching `FramedShdSample::fingerprint`.
///
/// This replaces a hardcoded `"trained": false` in the temporal-sensitivity
/// payload. That field was a literal, not a measurement — the probe loads a
/// weight file and has no way to know whether it came from `init` or from a
/// completed `train-cell`. It was therefore **false on every trained run**,
/// including the one §4b of
/// `MEASUREMENT_2026-08-03_TEMPORAL_SENSITIVITY_POSITIVE_CONTROL.md` is built
/// on, where the untrained/trained distinction is the entire point. Only the
/// file path distinguished them.
///
/// A fingerprint is something the instrument can actually establish: two runs
/// carrying the same value used the same weights, and a trained cell's
/// `--save-final-weights` output fingerprints differently from the
/// initialisation it started at.
fn weight_fingerprint(weights: &ShdArmWeights) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    let mut mix = |value: f32| {
        for byte in value.to_bits().to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }
    };
    for value in weights
        .base
        .w_in
        .iter()
        .chain(weights.base.w_out.iter())
        .chain(weights.base.b_out.iter())
        .chain(weights.w_rec.iter())
    {
        mix(*value);
    }
    hash
}

fn json_f64(values: &[f64]) -> String {
    let mut out = String::from("[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        // `{:.9}` renders a non-finite f64 as the bare token `inf`, which is not
        // valid JSON — so a diverging cell wrote a file no consumer could parse,
        // losing the very record that would explain the divergence. `json_f32`
        // already guarded this; `json_f64` did not. `null` matches that
        // convention and is what JSON has for "no value".
        if value.is_finite() {
            out.push_str(&format!("{value:.9}"));
        } else {
            out.push_str("null");
        }
    }
    out.push(']');
    out
}

/// Scalar counterpart of [`json_f64`]'s finiteness guard.
///
/// The cell payload interpolates several `f64` summaries with `{:.9}` directly.
/// `mean_gradient_norm` is the one observed to overflow, but any of them would
/// corrupt the file the same way.
fn json_scalar(value: f64) -> String {
    if value.is_finite() {
        format!("{value:.9}")
    } else {
        "null".into()
    }
}

fn json_f32(values: &[f32]) -> String {
    let body = values
        .iter()
        .map(|value| {
            if value.is_finite() {
                format!("{value:.9}")
            } else {
                "null".into()
            }
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{body}]")
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
    fs::rename(&temporary, path).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {

    /// Every flag a real campaign invocation uses must be accepted.
    ///
    /// When `reject_unknown_flags` was added it omitted `--arm`, which is parsed
    /// in a shared helper and so never appeared in a per-function grep of the
    /// flag literals. That would have refused **every attention cell** in the
    /// next campaign. Gate F did not catch it, because no recorded regression
    /// cell uses an attention arm — so this list is the only thing standing
    /// between a flag-parsing change and a dead campaign.
    #[test]
    fn every_real_campaign_invocation_is_accepted() {
        let owned = |v: &[&str]| -> Vec<String> { v.iter().map(|s| (*s).to_string()).collect() };

        // The exact shapes `scripts/aws/run_cell.py` emits.
        let init_attention = owned(&[
            "--n-inputs",
            "140",
            "--hidden",
            "128",
            "--classes",
            "20",
            "--seed",
            "5170001",
            "--epochs",
            "400",
            "--n-train",
            "8156",
            "--weights",
            "w.bin",
            "--orders",
            "o.bin",
            "--arm",
            "ff+fixed+attn",
            "--attn-dim",
            "32",
            "--attn-layers",
            "4",
        ]);
        let init_recurrent = owned(&[
            "--n-inputs",
            "700",
            "--hidden",
            "256",
            "--classes",
            "20",
            "--seed",
            "5170001",
            "--epochs",
            "100",
            "--n-train",
            "8156",
            "--weights",
            "w.bin",
            "--orders",
            "o.bin",
            "--arm",
            "rec+alif",
            "--w-rec-scale",
            "1.0",
        ]);
        for (label, args) in [
            ("init+attn", &init_attention),
            ("init+rec", &init_recurrent),
        ] {
            assert!(
                reject_unknown_flags(args, INIT_FLAGS).is_ok(),
                "{label} was refused: {:?}",
                reject_unknown_flags(args, INIT_FLAGS)
            );
        }

        let train_full = owned(&[
            "--train-events",
            "t.events",
            "--test-events",
            "e.events",
            "--contract",
            "published-2ms",
            "--geometry",
            "adjacent-sum-5",
            "--weights",
            "w.bin",
            "--orders",
            "o.bin",
            "--epochs",
            "400",
            "--out",
            "cell.json",
            "--arm",
            "ff+fixed+attn",
            "--temporal",
            "bin-shuffled",
            "--temporal-seed",
            "5170001",
            "--surrogate-scale",
            "0.4",
            "--clip-grad-norm",
            "1.0",
            "--max-train",
            "8156",
            "--max-test",
            "2264",
            "--seed",
            "5170001",
        ]);
        assert!(
            reject_unknown_flags(&train_full, TRAIN_CELL_FLAGS).is_ok(),
            "the full train-cell shape was refused: {:?}",
            reject_unknown_flags(&train_full, TRAIN_CELL_FLAGS)
        );
    }

    /// The guard must still refuse what it exists to refuse.
    #[test]
    fn a_misspelled_flag_is_refused_rather_than_ignored() {
        let args: Vec<String> = ["--attn-dims", "32"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let refused = reject_unknown_flags(&args, INIT_FLAGS);
        assert!(refused.is_err(), "a typo must not be silently ignored");
        let message = refused.unwrap_err();
        assert!(
            message.contains("--attn-dims"),
            "the message must name the flag"
        );
        assert!(message.contains("--attn-dim"), "and list what was expected");
    }
    use super::*;
    use rayon::ThreadPoolBuilder;

    fn sample(sample_index: usize) -> MatchedShdSample {
        let frames = (0..12)
            .map(|time| {
                (0..5)
                    .map(|event| (((sample_index * 13 + time * 7 + event * 11) % 40), 1.0))
                    .collect()
            })
            .collect();
        MatchedShdSample {
            label: (sample_index % 4) as u32,
            frames,
            n_inputs: 40,
            dt_ms: 2.0,
        }
    }

    fn result_bits(results: Vec<Result<(f32, ShdArmGradient), String>>) -> Vec<Vec<u32>> {
        results
            .into_iter()
            .map(|result| {
                let (loss, gradient) = result.expect("sample gradient");
                let mut bits = vec![loss.to_bits()];
                bits.extend(
                    gradient
                        .base
                        .w_in
                        .iter()
                        .chain(&gradient.base.w_out)
                        .chain(&gradient.base.b_out)
                        .chain(&gradient.w_rec)
                        .map(|value| value.to_bits()),
                );
                if let Some(attention) = &gradient.attn {
                    bits.extend(attention.iter_all().map(|value| value.to_bits()));
                }
                bits
            })
            .collect()
    }

    /// A forward that overflows must not be scored as if it had not.
    ///
    /// `AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md` §3 named this and it stayed
    /// open until 2026-08-29. The fixture is deliberately the REALISTIC shape —
    /// every weight finite, the logits overflowing during accumulation — rather
    /// than a NaN poked into a weight vector, because that is what h1024
    /// actually does at peak gradient norms of 4.9e32, and because a
    /// non-finite *weight* is already rejected at construction.
    ///
    /// Two assertions, and the second is the one that matters. `argmax` orders
    /// by `total_cmp`, under which NaN outranks every real: an overflowing
    /// forward does not crash and does not abstain, it wins and is counted. So
    /// the pre-fix code returned a well-formed accuracy here, and the only way
    /// to see the difference is to look at the new counter.
    #[test]
    fn a_forward_that_overflows_is_counted_rather_than_scored() {
        let samples: Vec<_> = (0..8).map(sample).collect();

        let healthy = ShdArmWeights::new(
            ShdMatchedWeights::deterministic(40, 8, 4, 7),
            MatchedArm::ALL[0],
            Vec::new(),
        )
        .expect("arm");
        let clean = evaluate(&healthy, &samples).expect("evaluate");
        assert_eq!(
            clean.non_finite_forward, 0,
            "the healthy fixture already overflows; it cannot show the defect"
        );

        // Every other field, pinned to the value the PRE-FIX code produced.
        //
        // These six numbers were captured by running this fixture through
        // `evaluate` at the commit before the guard existed, not by reading
        // them back off the new implementation. That is the whole point: the
        // guard reads `forward.logits`, which the forward already computed and
        // returned, and adds an integer count — so a healthy cell's
        // measurements must be untouched. Gate F would normally establish that
        // over archived cells, and it cannot run in a worktree without the
        // initialization artefacts, so the property is pinned here instead of
        // being argued from the diff.
        assert_eq!(clean.accuracy, 0.125);
        assert_eq!(clean.classes_predicted, 3);
        assert_eq!(clean.majority_prediction, 0.5);
        assert_eq!(clean.mean_firing_rate, 0.04817708465270698);
        assert_eq!(clean.silent_fraction, 0.375);
        assert_eq!(clean.saturated_fraction, 0.0);

        // Finite weights near the top of f32's range, in the READ-OUT only.
        //
        // Inflating `w_in` instead does not work, and the reason is worth
        // keeping: the membrane goes to `inf` on the first frame, the next step
        // computes `alpha * inf * (1.0 - 1.0)` and gets NaN, and `NaN >=
        // threshold` is false — so the neuron simply stops spiking and the
        // read-out stays finite. The spiking nonlinearity absorbs an input-side
        // overflow. It cannot absorb an output-side one: rates are healthy, in
        // [0, 1], and `b_out + sum(w_out * rate)` leaves f32's range.
        // `w_in` is raised only enough to make every unit fire every frame, so
        // the rates are ~1.0 and the read-out sum is eight terms of 3e38 rather
        // than eight terms of 3e38 times a small duty cycle. At the
        // deterministic init the rates are low enough that the sum stays inside
        // f32 and the fixture silently stops demonstrating anything — which it
        // did on the first attempt, so the firing rate is asserted below.
        let mut base = ShdMatchedWeights::deterministic(40, 8, 4, 7);
        base.w_in.fill(1.0);
        base.w_out.fill(3.0e38);
        assert!(
            base.w_in
                .iter()
                .chain(base.w_out.iter())
                .all(|v| v.is_finite()),
            "the fixture must overflow in the FORWARD, not in the weights"
        );
        let overflowing = ShdArmWeights::new(base, MatchedArm::ALL[0], Vec::new()).expect("arm");

        let poisoned = evaluate(&overflowing, &samples).expect("evaluate");
        // The fixture's own precondition: if the units stop firing, the
        // read-out sum shrinks back inside f32 and the assertion below would
        // pass or fail for a reason that has nothing to do with the guard.
        assert!(
            poisoned.mean_firing_rate > 0.5,
            "fixture fires at {:.4}; the read-out sum will not leave f32's range",
            poisoned.mean_firing_rate
        );
        assert_eq!(
            poisoned.non_finite_forward,
            samples.len(),
            "every sample's logits are non-finite and every one must be counted"
        );
        // The defect, stated as an assertion: the accuracy is still a
        // well-formed number in [0, 1]. Nothing about the value itself reveals
        // that it was computed over poisoned forwards, which is exactly why the
        // count has to exist and has to reach the pass predicate.
        assert!(
            (0.0..=1.0).contains(&poisoned.accuracy),
            "accuracy {} is not in [0, 1]",
            poisoned.accuracy
        );
    }

    /// Thread-count independence, for **every arm** and at prime thread counts.
    ///
    /// This previously ran one arm — `ff+fixed+attn` — at 1 thread against 4.
    /// Every cell in the record is produced by this reduction, and a
    /// thread-count dependence would change results silently rather than
    /// loudly: `par_iter().collect()` on an indexed parallel iterator preserves
    /// order, so `add_assign` must see identical addends in an identical
    /// sequence, and float addition is not associative enough to forgive
    /// anything less.
    ///
    /// Prime counts are included deliberately. A chunking bug that divides
    /// evenly into the batch is invisible at 1, 2, 4 and 8 and appears at 3, 5
    /// or 7 — and `PARALLEL_CHUNK` is 64, so an off-by-one in the sub-chunk
    /// walk would land exactly there.
    #[test]
    fn parallelism_is_bit_identical_to_serial_for_every_arm() {
        const THREADS: [usize; 6] = [1, 2, 3, 5, 8, 16];
        let train: Vec<_> = (0..16).map(sample).collect();
        let indices: Vec<_> = (0..train.len()).collect();

        for arm in MatchedArm::ALL.into_iter().chain(MatchedArm::ALL_ATTENTION) {
            let base = ShdMatchedWeights::deterministic(40, 8, 4, 7);
            let w_rec = if arm.recurrent {
                (0..8 * 8)
                    .map(|i| (((i % 13) as f32) - 6.0) * 1.3e-2)
                    .collect()
            } else {
                Vec::new()
            };
            let weights = if arm.attention {
                let config = AttentionConfig::new(32, 4).expect("attention config");
                let attention =
                    AttentionParams::deterministic(8, 4, config, 11).expect("attention parameters");
                ShdArmWeights::new_attentive(base, arm, w_rec, attention).expect("attention arm")
            } else {
                ShdArmWeights::new(base, arm, w_rec).expect("arm")
            };

            let run = |threads| {
                let weight_layout = ShdArmWeightLayout::prepare(&weights);
                ThreadPoolBuilder::new()
                    .num_threads(threads)
                    .build()
                    .expect("thread pool")
                    .install(|| {
                        result_bits(ordered_sample_gradients(
                            &weights,
                            &weight_layout,
                            &train,
                            &indices,
                            1.0,
                        ))
                    })
            };

            let serial = run(1);
            // A gradient of all zeros would be thread-count independent for
            // reasons that have nothing to do with the reduction.
            assert!(
                serial.iter().any(|bits| bits
                    .iter()
                    .any(|value| *value != 0 && *value != 0x8000_0000)),
                "{}: fixture produced an all-zero gradient; the comparison is vacuous",
                arm.label()
            );
            for threads in THREADS {
                assert_eq!(
                    serial,
                    run(threads),
                    "{} differs between 1 and {threads} threads",
                    arm.label()
                );
            }
        }
    }
}
