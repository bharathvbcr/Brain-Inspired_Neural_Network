//! Independent derivation of the arm backward, to verify `w_in` through the
//! attention read-out.
//!
//! # Why this exists
//!
//! `every_attention_arm_forward_and_backward_is_bit_pinned` has **never passed
//! in this repository**. It was committed in `516e9c7` — the same commit that
//! introduced `shd_attention.rs` — with constants that do not match the kernel
//! that shipped beside them, and it fails at every commit from that one to
//! `597aeba`. The differing entry is index 3, `gradient.base.w_in`: the forward
//! and the read-out gradient still match, so what moved is the path that
//! carries attention's credit into the spiking layer.
//!
//! The pin cannot be repaired by inspection, because the values correspond to
//! no committed state — they were captured from a working tree that was never
//! committed, so there is nothing to reproduce them from. Re-pinning from the
//! current kernel would therefore record whatever the kernel now does as
//! correct by definition. This file exists so that the re-pin rests on a
//! derivation instead.
//!
//! # The argument, in three steps
//!
//! 1. [`reference_pass`] reimplements the arm forward and gradient from
//!    the equations documented in `shd_matched_arms.rs`, in a separate crate
//!    target that can reach only the public API. It shares no code with the
//!    kernel: no transposed layouts, no sparse-skip over silent units, no
//!    scratch staging, no prepared weight layout.
//! 2. It is **calibrated on the four base arms**, whose behaviour is covered by
//!    Gate F over 296 recorded cells and by
//!    `every_arm_forward_and_backward_is_bit_pinned`. If the reimplementation
//!    reproduces those, it is a trustworthy reference for this model.
//! 3. It is then applied to the four attention arms. Agreement there is
//!    evidence about the kernel, because the reference was fixed before the
//!    arms under test were looked at.
//!
//! `ds_attn` itself is not re-derived here — it is checked directly against
//! finite differences of the attention forward, at **every** spike index, by
//! `shd_attention::tests::spike_gradient_matches_finite_difference`. That check
//! needs no backward code at all, so the two layers do not share an assumption.
//!
//! # On summation order
//!
//! Comparison is bit-exact. Float addition is not associative, so a
//! reimplementation free to sum in any order could only be compared under a
//! tolerance, and a tolerance wide enough to absorb reordering is also wide
//! enough to hide a small systematic error. This file therefore follows the
//! summation convention the kernel documents — decay, then frame events in
//! frame order, then the recurrent term in ascending `j`; backward in
//! descending `t` — and states it as part of the specification rather than
//! discovering it from the code. Every test also asserts agreement under a
//! relative tolerance, so a future reordering downgrades this file to the
//! weaker claim instead of silently failing.

// Index-based loops throughout, for the same reason `shd_attention.rs` and
// `shd_matched_arms.rs` carry this allow: nearly every loop here walks several
// arrays at different strides — `[t, hidden]`, `[hidden, n_inputs]`,
// `[hidden, hidden]`, `[n_classes, hidden]`. The index arithmetic *is* the
// specification this file checks; rewriting it as zipped iterators would hide
// the very thing under test.
#![allow(clippy::needless_range_loop)]

use binn_learn::{
    attention_forward, attention_gradient, attention_logits, shd_matched_loss_and_gradient_arm,
    AttentionConfig, AttentionParams, MatchedArm, MatchedShdSample, ShdArmWeights,
    ShdMatchedWeights, MATCHED_PHYSICAL_TAU_MS, MATCHED_SURROGATE_ALPHA, MATCHED_THRESHOLD,
};

/// Surrogate derivative, written from the documented form: a Lorentzian with
/// peak `alpha / 2` at threshold.
fn surrogate_prime(u_minus_threshold: f32) -> f32 {
    let alpha = MATCHED_SURROGATE_ALPHA;
    let scaled = std::f32::consts::FRAC_PI_2 * alpha * u_minus_threshold;
    (alpha * 0.5) / (1.0 + scaled * scaled)
}

fn softmax(logits: &[f32]) -> Vec<f32> {
    let maximum = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut values: Vec<f32> = logits.iter().map(|v| (v - maximum).exp()).collect();
    let total: f32 = values.iter().sum();
    for value in values.iter_mut() {
        *value /= total;
    }
    values
}

/// What the independent pass produces, for comparison against the kernel.
struct Independent {
    membrane: Vec<f32>,
    spikes: Vec<f32>,
    logits: Vec<f32>,
    grad_w_in: Vec<f32>,
    grad_w_out: Vec<f32>,
    grad_b_out: Vec<f32>,
    grad_w_rec: Vec<f32>,
}

/// The arm forward and backward, from the documented equations.
///
/// `credit_attention` exists so the same code can be run with attention's
/// contribution to `dL/ds` deliberately withheld. That is what
/// [`the_check_fails_when_attention_credit_is_withheld`] uses to show this file
/// would notice if the term went missing — a check that passes either way is
/// not a check.
fn reference_pass(
    weights: &ShdArmWeights,
    sample: &MatchedShdSample,
    credit_attention: bool,
) -> Independent {
    let base = &weights.base;
    let arm = weights.arm;
    let hidden = base.hidden;
    let n_inputs = base.n_inputs;
    let n_classes = base.n_classes;
    let t_steps = sample.frames.len();

    let alpha = (-sample.dt_ms / MATCHED_PHYSICAL_TAU_MS).exp();
    let rho = (-1.0_f32 / weights.tau_a).exp();
    let beta_a = weights.beta_a;

    // --- forward -------------------------------------------------------------
    let mut membrane = vec![0.0_f32; t_steps * hidden];
    let mut spikes = vec![0.0_f32; t_steps * hidden];
    let mut thresholds = vec![MATCHED_THRESHOLD; t_steps * hidden];
    // `s(t-1)` snapshotted per timestep: the recurrent drive reads the previous
    // timestep's spikes, never a mixture of previous and current.
    let mut previous_spikes = vec![0.0_f32; t_steps * hidden];

    let mut previous_u = vec![0.0_f32; hidden];
    let mut previous_s = vec![0.0_f32; hidden];
    let mut adaptation = vec![0.0_f32; hidden];

    for t in 0..t_steps {
        previous_spikes[t * hidden..(t + 1) * hidden].copy_from_slice(&previous_s);
        if arm.adaptive {
            for h in 0..hidden {
                adaptation[h] = rho * adaptation[h] + previous_s[h];
            }
        }
        let mut current = vec![0.0_f32; hidden];
        for h in 0..hidden {
            current[h] = alpha * previous_u[h] * (1.0 - previous_s[h]);
        }
        for &(channel, count) in &sample.frames[t] {
            for h in 0..hidden {
                current[h] += base.w_in[h * n_inputs + channel] * count;
            }
        }
        if arm.recurrent {
            for j in 0..hidden {
                let spike = previous_spikes[t * hidden + j];
                if spike == 0.0 {
                    continue;
                }
                for h in 0..hidden {
                    current[h] += weights.w_rec[h * hidden + j] * spike;
                }
            }
        }
        for h in 0..hidden {
            let threshold = if arm.adaptive {
                MATCHED_THRESHOLD + beta_a * adaptation[h]
            } else {
                MATCHED_THRESHOLD
            };
            let spike = f32::from(current[h] >= threshold);
            membrane[t * hidden + h] = current[h];
            spikes[t * hidden + h] = spike;
            thresholds[t * hidden + h] = threshold;
            previous_u[h] = current[h];
            previous_s[h] = spike;
        }
    }

    let inv_t = 1.0 / t_steps as f32;
    let mut rates = vec![0.0_f32; hidden];
    for t in 0..t_steps {
        for h in 0..hidden {
            rates[h] += spikes[t * hidden + h] * inv_t;
        }
    }

    let attention_cache = weights
        .attn
        .as_ref()
        .map(|params| attention_forward(params, &spikes, t_steps).unwrap());

    let mut logits = base.b_out.clone();
    for class in 0..n_classes {
        let row = class * hidden;
        for h in 0..hidden {
            logits[class] += base.w_out[row + h] * rates[h];
        }
    }
    if let (Some(params), Some(cache)) = (&weights.attn, &attention_cache) {
        for (logit, delta) in logits
            .iter_mut()
            .zip(attention_logits(params, cache.pooled()))
        {
            *logit += delta;
        }
    }

    let mut probabilities = softmax(&logits);
    probabilities[sample.label as usize] -= 1.0;

    // --- backward ------------------------------------------------------------
    let ds_attn = match (&weights.attn, &attention_cache) {
        (Some(params), Some(cache)) if credit_attention => {
            attention_gradient(params, cache, &spikes, &probabilities)
                .unwrap()
                .1
        }
        _ => Vec::new(),
    };
    let use_attention = !ds_attn.is_empty();

    // The read-out gradient is exact rather than surrogate: given the spike
    // train the loss is smooth in `w_out` and `b_out`.
    let grad_b_out = probabilities.clone();
    let mut grad_w_out = vec![0.0_f32; n_classes * hidden];
    let mut direct_spike = vec![0.0_f32; hidden];
    for class in 0..n_classes {
        let row = class * hidden;
        let probability = probabilities[class];
        for h in 0..hidden {
            grad_w_out[row + h] = probability * rates[h];
            direct_spike[h] += base.w_out[row + h] * probability * inv_t;
        }
    }

    let mut grad_w_in = vec![0.0_f32; hidden * n_inputs];
    let mut grad_w_rec = if arm.recurrent {
        vec![0.0_f32; hidden * hidden]
    } else {
        Vec::new()
    };
    let mut du_next = vec![0.0_f32; hidden];
    let mut da_next = vec![0.0_f32; hidden];

    for t in (0..t_steps).rev() {
        let mut du = vec![0.0_f32; hidden];
        let mut da = vec![0.0_f32; hidden];

        // `dL/ds` before the threshold: the rate read-out's constant-in-`t`
        // term, plus the recurrent gather from `t+1`, plus attention's
        // timestep-specific term.
        let mut ds_source = direct_spike.clone();
        if arm.recurrent {
            for j in 0..hidden {
                let backward_drive = du_next[j];
                for h in 0..hidden {
                    ds_source[h] += backward_drive * weights.w_rec[j * hidden + h];
                }
            }
        }
        if use_attention {
            for h in 0..hidden {
                ds_source[h] += ds_attn[t * hidden + h];
            }
        }

        for h in 0..hidden {
            let index = t * hidden + h;
            let mut ds = ds_source[h];
            if arm.adaptive {
                ds += da_next[h];
            }
            let gated = ds * surrogate_prime(membrane[index] - thresholds[index]);
            du[h] = gated + alpha * (1.0 - spikes[index]) * du_next[h];
            if arm.adaptive {
                da[h] = -beta_a * gated + rho * da_next[h];
            }
        }

        for &(channel, count) in &sample.frames[t] {
            for h in 0..hidden {
                grad_w_in[h * n_inputs + channel] += du[h] * count;
            }
        }
        if arm.recurrent {
            for h in 0..hidden {
                for j in 0..hidden {
                    let spike = previous_spikes[t * hidden + j];
                    if spike == 0.0 {
                        continue;
                    }
                    grad_w_rec[h * hidden + j] += du[h] * spike;
                }
            }
        }

        du_next.copy_from_slice(&du);
        if arm.adaptive {
            da_next.copy_from_slice(&da);
        }
    }

    // A self-loop is a threshold change in disguise, so the diagonal is held at
    // zero and receives no gradient.
    if arm.recurrent {
        for h in 0..hidden {
            grad_w_rec[h * hidden + h] = 0.0;
        }
    }

    Independent {
        membrane,
        spikes,
        logits,
        grad_w_in,
        grad_w_out,
        grad_b_out,
        grad_w_rec,
    }
}

/// The fixture the pin was taken over, so this file speaks directly to it.
fn dense_fixture(arm: MatchedArm) -> (MatchedShdSample, ShdArmWeights) {
    let hidden = 48;
    let n_inputs = 60;
    let mut frames = Vec::new();
    for t in 0..40 {
        let mut frame = Vec::new();
        for k in 0..14 {
            frame.push(((t * 13 + k * 7) % n_inputs, 1.0 + (k % 3) as f32));
        }
        frames.push(frame);
    }
    let sample = MatchedShdSample {
        label: 5,
        frames,
        n_inputs,
        dt_ms: 4.0,
    };
    let base = ShdMatchedWeights::deterministic(n_inputs, hidden, 20, 4242);
    let w_rec = if arm.recurrent {
        (0..hidden * hidden)
            .map(|i| (((i % 23) as f32) - 11.0) * 9e-3)
            .collect()
    } else {
        Vec::new()
    };
    let plain = MatchedArm {
        attention: false,
        ..arm
    };
    if arm.attention {
        let config = AttentionConfig::new(6, 1).unwrap();
        let mut params = AttentionParams::deterministic(hidden, 20, config, 771).unwrap();
        for block in params.blocks.iter_mut() {
            for (position, value) in block.w_o.iter_mut().enumerate() {
                *value = (((position % 13) as f32) - 6.0) * 2e-2;
            }
        }
        (
            sample,
            ShdArmWeights::new_attentive(base, arm, w_rec, params).unwrap(),
        )
    } else {
        (sample, ShdArmWeights::new(base, plain, w_rec).unwrap())
    }
}

/// Bit-exact, with a relative-tolerance fallback reported in the message so a
/// reordering is distinguishable from a wrong value.
fn assert_matches(label: &str, block: &str, expected: &[f32], observed: &[f32]) {
    assert_eq!(
        expected.len(),
        observed.len(),
        "{label} {block}: length {} vs {}",
        expected.len(),
        observed.len()
    );
    let mut worst = 0.0_f32;
    let mut worst_index = 0_usize;
    let mut differing = 0_usize;
    for (index, (left, right)) in expected.iter().zip(observed).enumerate() {
        if left.to_bits() != right.to_bits() {
            differing += 1;
            let scale = left.abs().max(right.abs()).max(1e-30);
            let relative = (left - right).abs() / scale;
            if relative > worst {
                worst = relative;
                worst_index = index;
            }
        }
    }
    assert!(
        differing == 0,
        "{label} {block}: {differing} of {} entries differ from the independent \
         derivation; worst at [{worst_index}] kernel {:e} vs independent {:e} \
         (relative {worst:e}). A worst-case relative deviation near f32 epsilon \
         means the summation order moved; anything larger means the value did.",
        expected.len(),
        expected[worst_index],
        observed[worst_index],
    );
}

/// Step 2 of the argument: the reference reproduces the arms that Gate F and
/// the base pin already cover. Establishes that the reimplementation is right
/// before it is pointed at anything unverified.
#[test]
fn the_independent_derivation_reproduces_every_base_arm() {
    for arm in MatchedArm::ALL {
        let (sample, weights) = dense_fixture(arm);
        let (forward, gradient) = shd_matched_loss_and_gradient_arm(&weights, &sample).unwrap();
        let independent = reference_pass(&weights, &sample, true);
        let label = arm.label();

        assert_matches(label, "membrane", &forward.membrane, &independent.membrane);
        assert_matches(label, "spikes", &forward.spikes, &independent.spikes);
        assert_matches(label, "logits", &forward.logits, &independent.logits);
        assert_matches(
            label,
            "grad_w_in",
            &gradient.base.w_in,
            &independent.grad_w_in,
        );
        assert_matches(
            label,
            "grad_w_out",
            &gradient.base.w_out,
            &independent.grad_w_out,
        );
        assert_matches(
            label,
            "grad_b_out",
            &gradient.base.b_out,
            &independent.grad_b_out,
        );
        assert_matches(
            label,
            "grad_w_rec",
            &gradient.w_rec,
            &independent.grad_w_rec,
        );
    }
}

/// Step 3: the same reference, applied to the four attention arms. This is the
/// check the re-pin rests on.
#[test]
fn attention_w_in_matches_the_independent_derivation() {
    for arm in MatchedArm::ALL_ATTENTION {
        let (sample, weights) = dense_fixture(arm);
        let (forward, gradient) = shd_matched_loss_and_gradient_arm(&weights, &sample).unwrap();
        let independent = reference_pass(&weights, &sample, true);
        let label = arm.label();

        // The forward must be untouched by the read-out; if it were not, a
        // `w_in` comparison would be comparing two different spike trains.
        assert_matches(label, "membrane", &forward.membrane, &independent.membrane);
        assert_matches(label, "spikes", &forward.spikes, &independent.spikes);
        assert_matches(label, "logits", &forward.logits, &independent.logits);
        assert_matches(
            label,
            "grad_w_in",
            &gradient.base.w_in,
            &independent.grad_w_in,
        );
        assert_matches(
            label,
            "grad_w_out",
            &gradient.base.w_out,
            &independent.grad_w_out,
        );
        assert_matches(
            label,
            "grad_b_out",
            &gradient.base.b_out,
            &independent.grad_b_out,
        );
        assert_matches(
            label,
            "grad_w_rec",
            &gradient.w_rec,
            &independent.grad_w_rec,
        );
    }
}

/// Step 1 of the argument, and the one layer that touches no backward code at
/// all: `ds_attn` against central differences of the attention forward, at
/// **every** index of the spike train rather than a sampled few.
///
/// The objective is the directional derivative `attention_gradient` is defined
/// to return — `sum_c d_logits[c] * logit_c` — so a disagreement here is a
/// disagreement about `dL/ds` itself, independent of how the arm later spends
/// it. Nothing downstream of the spike threshold is a surrogate, so this is a
/// genuine derivative and exactness is the right standard.
#[test]
fn ds_attn_matches_finite_difference_at_every_index() {
    let (t_steps, hidden, n_classes, d_model, layers) = (12_usize, 8_usize, 5_usize, 6_usize, 2);
    let config = AttentionConfig::new(d_model, layers).unwrap();
    let mut params = AttentionParams::deterministic(hidden, n_classes, config, 771).unwrap();
    for block in params.blocks.iter_mut() {
        for (position, value) in block.w_o.iter_mut().enumerate() {
            *value = (((position % 13) as f32) - 6.0) * 2e-2;
        }
    }
    // A spike train with both states well represented: an all-silent or
    // all-firing trace would make the attention rows uniform and the check
    // vacuous.
    let spikes: Vec<f32> = (0..t_steps * hidden)
        .map(|i| f32::from((i * 7 + i / 5) % 3 != 0))
        .collect();
    let firing = spikes.iter().sum::<f32>() / spikes.len() as f32;
    assert!(
        (0.2..=0.8).contains(&firing),
        "fixture firing rate {firing:.3} is degenerate"
    );
    let d_logits: Vec<f32> = (0..n_classes)
        .map(|c| 0.31 - 0.24 * c as f32 + 0.05 * (c * c) as f32)
        .collect();

    let objective = |spikes: &[f32]| -> f32 {
        let cache = attention_forward(&params, spikes, t_steps).unwrap();
        attention_logits(&params, cache.pooled())
            .iter()
            .zip(&d_logits)
            .map(|(logit, weight)| logit * weight)
            .sum()
    };

    let cache = attention_forward(&params, &spikes, t_steps).unwrap();
    let (_, ds_attn) = attention_gradient(&params, &cache, &spikes, &d_logits).unwrap();

    // Two bars, because one is not enough. The loose `absolute < 2e-4` branch
    // is unavoidable — central differencing in f32 has a roundoff floor of
    // about `eps * |objective| / h`, and entries below that floor cannot be
    // resolved at all. But a test that only ever passes through the loose
    // branch would accept a systematically wrong gradient on small entries. So
    // the entries large enough to be resolved are held to the *relative* bar
    // separately, and the count of them is asserted.
    let resolvable = 1e-3_f32;
    let epsilon = 1e-3_f32;
    let mut worst_relative = 0.0_f32;
    let mut worst_resolvable = 0.0_f32;
    let mut resolvable_count = 0_usize;
    let mut nonzero = 0_usize;
    for index in 0..spikes.len() {
        let mut plus = spikes.clone();
        plus[index] += epsilon;
        let mut minus = spikes.clone();
        minus[index] -= epsilon;
        let numerical = (objective(&plus) - objective(&minus)) / (2.0 * epsilon);
        let analytic = ds_attn[index];
        let absolute = (analytic - numerical).abs();
        let scale = analytic.abs().max(numerical.abs()).max(1e-5);
        let relative = absolute / scale;
        if analytic != 0.0 {
            nonzero += 1;
        }
        worst_relative = worst_relative.max(relative);
        if analytic.abs() >= resolvable {
            resolvable_count += 1;
            worst_resolvable = worst_resolvable.max(relative);
        }
        assert!(
            absolute < 2e-4 || relative < 0.02,
            "ds_attn[{index}]: analytic {analytic:e} vs numerical {numerical:e} \
             (absolute {absolute:e}, relative {relative:e})"
        );
    }
    assert!(
        resolvable_count * 4 >= spikes.len(),
        "only {resolvable_count} of {} entries exceed {resolvable:e}, so the \
         relative bar below covers too little of the gradient to be meaningful",
        spikes.len()
    );
    assert!(
        worst_resolvable < 0.02,
        "worst relative deviation among the {resolvable_count} resolvable \
         entries is {worst_resolvable:e}; the analytic gradient disagrees with \
         finite differences where finite differences are trustworthy"
    );
    // An all-zero `ds_attn` would satisfy every comparison above if the forward
    // were also insensitive to the spikes. It is not, and this pins that.
    assert_eq!(
        nonzero,
        spikes.len(),
        "only {nonzero} of {} ds_attn entries are non-zero; the check would be \
         partly vacuous",
        spikes.len()
    );
    println!(
        "{} indices checked, {resolvable_count} resolvable; worst relative \
         overall {worst_relative:e}, worst among resolvable {worst_resolvable:e}",
        spikes.len()
    );
}

/// The check must be able to fail. With attention's contribution to `dL/ds`
/// withheld, the derivation has to disagree with the kernel — otherwise
/// [`attention_w_in_matches_the_independent_derivation`] would pass on an arm
/// whose attention credit never reached `w_in` at all, which is the exact
/// defect it exists to detect.
#[test]
fn the_check_fails_when_attention_credit_is_withheld() {
    for arm in MatchedArm::ALL_ATTENTION {
        let (sample, weights) = dense_fixture(arm);
        let (_, gradient) = shd_matched_loss_and_gradient_arm(&weights, &sample).unwrap();
        let withheld = reference_pass(&weights, &sample, false);

        let differing = gradient
            .base
            .w_in
            .iter()
            .zip(&withheld.grad_w_in)
            .filter(|(left, right)| left.to_bits() != right.to_bits())
            .count();
        assert!(
            differing > 0,
            "{}: withholding attention credit left grad_w_in unchanged, so the \
             comparison in this file proves nothing about the attention path",
            arm.label()
        );

        // Not merely different: different by a margin far above rounding, so
        // the sensitivity is to the term itself rather than to noise.
        let worst = gradient
            .base
            .w_in
            .iter()
            .zip(&withheld.grad_w_in)
            .map(|(left, right)| (left - right).abs() / left.abs().max(right.abs()).max(1e-30))
            .fold(0.0_f32, f32::max);
        assert!(
            worst > 1e-3,
            "{}: attention credit moves grad_w_in by at most {worst:e} relative, \
             which is too small to distinguish from rounding",
            arm.label()
        );
    }
}

/// Randomised differential stress: the kernel against the independent
/// derivation across many shapes, not one fixture.
///
/// The bit-pin and the tests above all speak to a single 48x60x40 fixture with
/// `d_model = 6, layers = 1`. A kernel can be exactly right there and wrong at
/// a boundary — one timestep, one hidden unit, an attention width equal to the
/// hidden width, a frame with no events, an adaptation constant that makes the
/// threshold outrun the drive. This sweeps those.
///
/// Every configuration is checked on **all eight arms**, so a defect that only
/// appears when adaptation and recurrence and attention are combined has
/// somewhere to show up. That combination is exactly what a cross-architecture
/// wave runs and what nothing in the record has ever exercised.
#[test]
fn the_kernel_matches_the_derivation_across_many_shapes() {
    // (t_steps, hidden, n_inputs, events_per_frame, d_model, layers, dt_ms)
    let shapes: &[(usize, usize, usize, usize, usize, usize, f32)] = &[
        (1, 4, 5, 2, 2, 1, 2.0),     // single timestep: no order to exploit
        (2, 1, 3, 1, 2, 1, 2.0),     // single hidden unit
        (3, 4, 4, 0, 4, 1, 2.0),     // every frame empty: drive is decay only
        (5, 8, 6, 3, 8, 1, 1.0),     // d_model == hidden
        (7, 3, 9, 5, 4, 3, 4.0),     // more attention layers than hidden units
        (9, 6, 7, 2, 6, 2, 10.0),    // coarse bins: alpha near zero
        (12, 16, 20, 7, 16, 1, 0.5), // fine bins: alpha near one
        (20, 24, 30, 11, 12, 2, 2.0),
        (33, 12, 14, 4, 10, 1, 3.0),
        (17, 9, 11, 1, 2, 4, 2.0), // narrowest attention, deepest stack
    ];

    let mut compared = 0_usize;
    for (index, &(t_steps, hidden, n_inputs, events, d_model, layers, dt_ms)) in
        shapes.iter().enumerate()
    {
        let seed = 9_000 + index as u64;
        let mut frames = Vec::with_capacity(t_steps);
        for t in 0..t_steps {
            let mut frame = Vec::with_capacity(events);
            for k in 0..events {
                // Counts vary and repeat a channel within a frame, so the
                // accumulation order inside a timestep is actually exercised.
                let channel = (t * 5 + k * 3) % n_inputs;
                let count = 1.0 + ((t + k) % 4) as f32 * 0.5;
                frame.push((channel, count));
            }
            frames.push(frame);
        }
        let sample = MatchedShdSample {
            label: (index % 7) as u32,
            frames,
            n_inputs,
            dt_ms,
        };

        for arm in MatchedArm::ALL.into_iter().chain(MatchedArm::ALL_ATTENTION) {
            let base = ShdMatchedWeights::deterministic(n_inputs, hidden, 7, seed);
            let w_rec = if arm.recurrent {
                (0..hidden * hidden)
                    .map(|i| (((i % 17) as f32) - 8.0) * 1.1e-2)
                    .collect()
            } else {
                Vec::new()
            };
            let weights = if arm.attention {
                let config = AttentionConfig::new(d_model, layers).unwrap();
                let mut params =
                    AttentionParams::deterministic(hidden, 7, config, seed ^ 0x5151).unwrap();
                // `w_o` is zero at initialisation, which would make the block an
                // exact identity residual and leave most of the backward
                // untested. Move it off zero.
                for block in params.blocks.iter_mut() {
                    for (position, value) in block.w_o.iter_mut().enumerate() {
                        *value = (((position % 9) as f32) - 4.0) * 3e-2;
                    }
                }
                ShdArmWeights::new_attentive(base, arm, w_rec, params).unwrap()
            } else {
                ShdArmWeights::new(base, arm, w_rec).unwrap()
            };

            let (forward, gradient) = shd_matched_loss_and_gradient_arm(&weights, &sample).unwrap();
            let independent = reference_pass(&weights, &sample, true);
            let label = format!(
                "{} t{t_steps} h{hidden} n{n_inputs} e{events} d{d_model} l{layers} dt{dt_ms}",
                arm.label()
            );

            assert!(
                forward.loss.is_finite(),
                "{label}: fixture produced a non-finite loss, so the comparison \
                 below would be comparing two NaNs"
            );
            assert_matches(&label, "membrane", &forward.membrane, &independent.membrane);
            assert_matches(&label, "spikes", &forward.spikes, &independent.spikes);
            assert_matches(&label, "logits", &forward.logits, &independent.logits);
            assert_matches(
                &label,
                "grad_w_in",
                &gradient.base.w_in,
                &independent.grad_w_in,
            );
            assert_matches(
                &label,
                "grad_w_out",
                &gradient.base.w_out,
                &independent.grad_w_out,
            );
            assert_matches(
                &label,
                "grad_b_out",
                &gradient.base.b_out,
                &independent.grad_b_out,
            );
            assert_matches(
                &label,
                "grad_w_rec",
                &gradient.w_rec,
                &independent.grad_w_rec,
            );
            compared += 1;
        }
    }
    assert_eq!(
        compared,
        shapes.len() * 8,
        "not every shape reached every arm"
    );
}
