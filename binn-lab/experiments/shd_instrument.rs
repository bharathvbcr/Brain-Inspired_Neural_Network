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
use binn_learn::{
    load_epoch_orders, one_cycle_lr, shd_matched_loss_and_gradient_arm, MatchedArm,
    MatchedShdSample, PortableRng, ShdArmGradient, ShdArmWeights, ShdMatchedWeights,
};
use binn_learn::shd_matched_arms::ArmAdam;
use binn_learn::{apply_temporal, TemporalAudit, TemporalCondition};

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
         Optional on init, recurrent arms only:\n\
           --w-rec-scale F   multiplies the Glorot recurrent draw (default 1.0)\n\
         Optional on train-cell:\n\
           --temporal intact|bin-shuffled|channel-shuffled|reversed  (default intact)\n\
           --temporal-seed N   (required unless --temporal intact)\n\
           --clip-grad-norm F  global-norm gradient clipping (default: off)"
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
    let json = format!(
        "{{\"frame_hash\":\"{:016x}\",\"valid_steps\":{},\"dt_ms\":{:.9},\
         \"loss\":{:.9},\"prediction\":{},\"membrane\":{},\"spikes\":{},\"rates\":{},\
         \"logits\":{},\"grad_w_in\":{},\"grad_w_out\":{},\"grad_b_out\":{},\
         \"updated_w_in\":{},\"updated_w_out\":{},\"updated_b_out\":{},\"arm\":\"{}\"{}}}\n",
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
    );
    atomic_write(&out, json.as_bytes())
}

fn init(args: &[String]) -> Result<(), String> {
    let n_inputs = require_positive(parse_usize(&required(args, "--n-inputs")?, "--n-inputs")?, "--n-inputs")?;
    let hidden = require_positive(parse_usize(&required(args, "--hidden")?, "--hidden")?, "--hidden")?;
    let n_classes = require_positive(parse_usize(&required(args, "--classes")?, "--classes")?, "--classes")?;
    let seed = parse_u64(&required(args, "--seed")?, "--seed")?;
    let epochs = require_positive(parse_usize(&required(args, "--epochs")?, "--epochs")?, "--epochs")?;
    let n_train = require_positive(parse_usize(&required(args, "--n-train")?, "--n-train")?, "--n-train")?;
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
            return Err(format!("--w-rec-scale must be finite and positive, got {scale}"));
        }
        let w_rec = if arm.recurrent {
            let mut rng = PortableRng::new(seed ^ 0x5245_4300_0000_0001);
            let limit = (6.0_f32 / (hidden + hidden) as f32).sqrt();
            (0..hidden * hidden).map(|_| rng.uniform(-limit, limit) * scale).collect()
        } else {
            Vec::new()
        };
        ShdArmWeights::new(base, arm, w_rec)?.save(&weights_path)?;
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
    let samples = require_positive(optional_usize(args, "--samples")?.unwrap_or(256), "--samples")?;
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

fn train_cell(args: &[String]) -> Result<(), String> {
    let train_events = required_path(args, "--train-events")?;
    let test_events = required_path(args, "--test-events")?;
    let contract = parse_contract(&required(args, "--contract")?)?;
    let geometry = parse_geometry(&required(args, "--geometry")?)?;
    let epochs = require_positive(parse_usize(&required(args, "--epochs")?, "--epochs")?, "--epochs")?;
    // Absent means no clipping and bit-identical behaviour to before the
    // flag existed. A non-positive or non-finite threshold is rejected
    // rather than silently disabling clipping on a run that asked for it.
    let clip_grad_norm = match optional_f32(args, "--clip-grad-norm")? {
        None => None,
        Some(value) if value.is_finite() && value > 0.0 => Some(f64::from(value)),
        Some(value) => {
            return Err(format!(
                "--clip-grad-norm must be finite and positive, got {value}"
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
        return Err("--temporal other than 'intact' requires --temporal-seed for reproducibility".into());
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
    if orders.iter().take(epochs).any(|order| order.len() != train.len()) {
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
        for batch in order.chunks(batch_size) {
            let mut gradient = ShdArmGradient::zeros_like(&weights);
            for &index in batch {
                let (forward, sample_gradient) =
                    shd_matched_loss_and_gradient_arm(&weights, &train[index])?;
                diagnostics.loss_sum += f64::from(forward.loss);
                diagnostics.samples += 1;
                if !forward.loss.is_finite() || !sample_gradient.all_finite() {
                    return Err(format!("non-finite training value at optimizer step {global_step}"));
                }
                gradient.add_assign(&sample_gradient);
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
            if let Some(threshold) = clip_grad_norm {
                let norm = f64::from(gradient_norm);
                if norm.is_finite() && norm > threshold {
                    gradient.scale((threshold / norm) as f32);
                    diagnostics.clipped_steps += 1;
                } else if !norm.is_finite() {
                    // An unrepresentable norm cannot be scaled into range by a
                    // ratio, so clipping cannot rescue this step. Count it and
                    // let the existing non-finite accounting report it.
                    diagnostics.unclippable_steps += 1;
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
        if earlier.abs() > 0.0 { (later - earlier) / earlier } else { 0.0 }
    } else {
        0.0
    };
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
        && diagnostics.non_finite_events == 0;
    let result = format!(
        "{{\"schema\":\"shd-cal-cell-v1\",\"backend\":\"rust\",\"arm\":\"{}\",\"contract\":\"{}\",\
         \"geometry\":\"{}\",\"hidden\":{},\"epochs\":{},\"n_train\":{},\"n_test\":{},\
         \"accuracy\":{:.9},\"classes_predicted\":{},\"majority_prediction\":{:.9},\
         \"mean_firing_rate\":{:.9},\"silent_fraction\":{:.9},\"saturated_fraction\":{:.9},\
         \"mean_loss\":{},\"mean_gradient_norm\":{},\"mean_update_rms\":{},\
         \"non_finite_events\":{},\"clip_grad_norm\":{},\"clipped_steps\":{},\"unclippable_steps\":{},\"temporal_condition\":\"{}\",\"temporal_audit\":{{\"samples\":{},\"counts_preserved\":{},\"relocated_fraction\":{:.9},\"mean_bin_displacement\":{:.9},\"occupied_bins_before\":{:.9},\"occupied_bins_after\":{:.9}}},\"epoch_mean_loss\":{},\"epoch_mean_gradient_norm\":{},\"epoch_max_gradient_norm\":{},\"epoch_max_gradient_step\":{},\"tail_loss_improvement\":{:.9},\"mechanical_status\":\"COMPLETE\",\
         \"scientific_status\":\"{}\",\"wall_secs\":{:.6}}}\n",
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
        clip_grad_norm.map_or("null".to_string(), |v| format!("{v:.9}")),
        diagnostics.clipped_steps,
        diagnostics.unclippable_steps,
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
    );
    atomic_write(&out, result.as_bytes())
}

struct Evaluation {
    accuracy: f64,
    classes_predicted: usize,
    majority_prediction: f64,
    mean_firing_rate: f64,
    silent_fraction: f64,
    saturated_fraction: f64,
}

fn evaluate(weights: &ShdArmWeights, samples: &[MatchedShdSample]) -> Result<Evaluation, String> {
    let mut correct = 0usize;
    let mut predictions = vec![0usize; weights.base.n_classes];
    let mut unit_rate = vec![0.0_f64; weights.base.hidden];
    for sample in samples {
        let (forward, _) = shd_matched_loss_and_gradient_arm(weights, sample)?;
        correct += usize::from(forward.prediction == sample.label as usize);
        predictions[forward.prediction] += 1;
        for (unit, rate) in unit_rate.iter_mut().zip(forward.rates) {
            *unit += f64::from(rate);
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
    if let Some(frame_ms) = value
        .strip_prefix("published-")
        .and_then(|value| value.strip_suffix("ms"))
    {
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
            args.get(index + 1).ok_or_else(|| format!("{flag} requires a value"))?,
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
            Ok(Some(raw.parse().map_err(|error| format!("{flag}: {error}"))?))
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
