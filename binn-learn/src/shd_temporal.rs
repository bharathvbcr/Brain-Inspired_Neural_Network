//! Temporal-information manipulations for `PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION`.
//!
//! | Condition | Destroys | Preserves |
//! |---|---|---|
//! | `intact` | - | - |
//! | `bin-shuffled` | temporal order | per-channel counts, within-bin synchrony |
//! | `channel-shuffled` | order **and** cross-channel synchrony | per-channel counts |
//! | `reversed` | direction | order magnitude, synchrony, counts |
//! | `window-shuffled-wN` | order **below N bins** | counts, synchrony, structure above N |
//! | `spike-dropout-pN` | N% of the spikes | order, synchrony, sequence length |
//! | `hidden-shuffled` | order in the **hidden** train | the input, the substrate, every rate |
//!
//! The `bin-shuffled` / `channel-shuffled` contrast is what separates *order*
//! from *synchrony*; most published shuffle controls conflate the two.
//! `window-shuffled-wN` turns the order axis into a ladder — `bin-shuffled` is
//! its `N >= steps` rung — and `spike-dropout-pN` is the one operator that
//! moves rate, which is what makes a null under the others interpretable.
//! `hidden-shuffled` is the only one that acts after the substrate has run, and
//! it is the only one whose effect on the rate arm is **exactly** zero rather
//! than measured to be small — which makes every rate cell it produces a
//! positive control on the instrument that produced it.
//!
//! # Manipulation check is code, not hope
//!
//! Prereg §5.1 makes count preservation a blocking gate: if a manipulation
//! changed per-channel totals it would be altering *rate*, and the whole
//! experiment would be measuring the wrong thing. [`apply_temporal`] therefore
//! recomputes per-channel totals before and after and refuses to return a
//! sample that fails. [`TemporalAudit`] carries the evidence into the cell
//! record so a passing run can be checked after the fact.
//!
//! # One gate per operator, not one gate for all of them
//!
//! That paragraph was true of all four original conditions at once, and so was
//! the call site's second check — `relocated_fraction >= 0.5`. Neither survives
//! contact with the two operators added since: dropout changes counts by design
//! and relocates nothing, and `window-shuffled-w2` relocates about half the
//! dataset and would sit on the old floor's boundary.
//!
//! Widening the shared gate to admit them would have made it a check that
//! cannot fail — the failure shape this campaign has removed from the command
//! line, the pass predicate and the forward pass in turn. Instead each operator
//! declares its own [`OperatorInvariants`], and the declaration is as specific
//! as the operator: a window shuffle's displacement bound is exact, dropout's
//! retention is checked against a binomial band whose width comes from the
//! number of spikes it actually saw, and the three original operators keep
//! precisely the teeth they had plus the displacement floor that separates a
//! full shuffle from a narrow one.
//!
//! `scripts/cell_validity.py` carries the same table for the recorded artefact,
//! and `scripts/test_campaign_tooling.py` pins the two together.

use crate::shd_matched::{MatchedShdSample, PortableRng};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemporalCondition {
    Intact,
    BinShuffled,
    ChannelShuffled,
    Reversed,
    /// Permute bins only **within disjoint windows** of `window` bins.
    ///
    /// This is the timescale ladder. Structure coarser than `window` survives
    /// untouched, structure finer than it is destroyed, and `bin-shuffled` is
    /// the `window >= steps` end of the same family. Sweeping `window` is what
    /// turns "the read-out uses order" into "the read-out uses order at *this*
    /// scale".
    ///
    /// Displacement is bounded by `window - 1` **by construction**, which is
    /// the invariant [`OperatorInvariants`] checks: a window shuffle that moved
    /// an entry further than its own window would be a different manipulation
    /// wearing this label.
    WindowShuffled { window: usize },
    /// Delete each individual spike independently with probability
    /// `percent / 100`.
    ///
    /// The **only** registered operator that changes rate, and it is here
    /// precisely because every other one preserves it. A null under shuffling
    /// can always be read as "the read-out only ever used rate"; that reading
    /// predicts dropout is costly. A null under *both* says the measurement is
    /// insensitive, and no amount of shuffling could have told us.
    ///
    /// Per **spike**, not per (bin, channel) entry: `adjacent-sum-5` leaves
    /// entries carrying counts above one, so deleting entries would delete a
    /// variable and count-dependent number of spikes. Per-spike deletion makes
    /// the surviving count exactly `Binomial(count, 1 - p)`, which is what the
    /// registered retention band is written against.
    SpikeDropout { percent: u32 },
    /// Permute the **hidden** spike train's time axis, leaving the input alone.
    ///
    /// Every other operator manipulates what the network is shown and then asks
    /// what the read-out could still recover. This one manipulates what the
    /// *read-out* is shown, after the substrate has already run, and so
    /// separates two things the other operators necessarily confound: how much
    /// of the read-out's advantage comes from temporal structure the substrate
    /// built, and how much from the substrate having been driven by ordered
    /// input in the first place.
    ///
    /// Its defining property is that the **rate read-out's cost is exactly
    /// zero**, not approximately zero — a mean over timesteps is a sum over a
    /// set and a permutation is a bijection on it. That makes it the only
    /// operator in this file with a positive control built into its own design:
    /// a rate arm whose accuracy moves by a single ULP under `hidden-shuffled`
    /// is evidence of a defect, not of an effect.
    HiddenShuffled,
}

impl TemporalCondition {
    /// The four conditions registered by
    /// `PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION`.
    ///
    /// This stays four elements. Every recorded cell is one of these, and the
    /// tests that say "every condition" were written before the timescale and
    /// dropout operators existed — widening it here would silently redefine
    /// what they claim to cover. New operators are opted into by name.
    pub const ALL: [Self; 4] = [
        Self::Intact,
        Self::BinShuffled,
        Self::ChannelShuffled,
        Self::Reversed,
    ];

    pub fn label(self) -> String {
        match self {
            Self::Intact => "intact".to_string(),
            Self::BinShuffled => "bin-shuffled".to_string(),
            Self::ChannelShuffled => "channel-shuffled".to_string(),
            Self::Reversed => "reversed".to_string(),
            Self::WindowShuffled { window } => format!("window-shuffled-w{window}"),
            Self::SpikeDropout { percent } => format!("spike-dropout-p{percent}"),
            Self::HiddenShuffled => "hidden-shuffled".to_string(),
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "intact" => return Ok(Self::Intact),
            "bin-shuffled" => return Ok(Self::BinShuffled),
            "channel-shuffled" => return Ok(Self::ChannelShuffled),
            "reversed" => return Ok(Self::Reversed),
            "hidden-shuffled" => return Ok(Self::HiddenShuffled),
            _ => {}
        }
        if let Some(rest) = value.strip_prefix("window-shuffled-w") {
            let window: usize = rest
                .parse()
                .map_err(|_| format!("window-shuffled needs an integer width, got {rest:?}"))?;
            return Self::window_shuffled(window);
        }
        if let Some(rest) = value.strip_prefix("spike-dropout-p") {
            let percent: u32 = rest
                .parse()
                .map_err(|_| format!("spike-dropout needs an integer percent, got {rest:?}"))?;
            return Self::spike_dropout(percent);
        }
        Err(format!(
            "unknown temporal condition {value:?}; expected intact, bin-shuffled, \
             channel-shuffled, reversed, hidden-shuffled, window-shuffled-w<N> or \
             spike-dropout-p<N>"
        ))
    }

    /// A window of 1 permutes nothing and would report as a manipulation while
    /// being the identity — the failure the `relocated_fraction` gate exists to
    /// catch, arriving here instead as a parse error before any cell runs.
    pub fn window_shuffled(window: usize) -> Result<Self, String> {
        if window < 2 {
            return Err(format!(
                "window-shuffled needs a window of at least 2 bins, got {window}; \
                 a window of 1 is the identity"
            ));
        }
        Ok(Self::WindowShuffled { window })
    }

    /// 0% deletes nothing and 100% deletes the dataset. Both would produce a
    /// cell that looks like a dropout arm and is not one.
    pub fn spike_dropout(percent: u32) -> Result<Self, String> {
        if percent == 0 || percent >= 100 {
            return Err(format!(
                "spike-dropout percent must be in 1..=99, got {percent}"
            ));
        }
        Ok(Self::SpikeDropout { percent })
    }

    pub const fn is_identity(self) -> bool {
        matches!(self, Self::Intact)
    }

    /// The invariants this operator promises, registered in code.
    ///
    /// # Why this replaced a single global gate
    ///
    /// Until the timescale and dropout operators existed, every registered
    /// manipulation preserved per-channel counts exactly and relocated most of
    /// the dataset, so **one** pair of checks — bit-exact counts inside
    /// [`apply_temporal`], and `relocated_fraction >= 0.5` at the call site —
    /// covered all four. Both are wrong for the new pair, and wrong in the
    /// direction that voids valid runs rather than passing invalid ones:
    /// dropout relocates nothing and reduces counts *by design*, and a window
    /// shuffle at `window = 2` relocates about half the dataset and would sit
    /// on the old gate's boundary.
    ///
    /// The repair is not to loosen the gate. A loosened global gate is the
    /// shape this campaign keeps finding and removing — a check that cannot
    /// fail. Each operator instead declares what *it* must be true of, and the
    /// declaration is as specific as the operator: dropout's retention is
    /// checked against a binomial band derived from the number of spikes it
    /// actually saw, and window-shuffle's displacement bound is exact.
    pub fn invariants(self) -> OperatorInvariants {
        match self {
            Self::Intact => OperatorInvariants {
                relocated: (0.0, 0.0),
                max_displacement: Some(0.0),
                min_mean_displacement_fraction: None,
                hidden_permutation_relocated: (0.0, 0.0),
                retention: RetentionRule::Exact,
            },
            // The three original shuffles keep exactly the teeth they had: the
            // 0.5 floor is `shd_instrument`'s old global gate, moved here
            // unchanged so no recorded cell changes verdict.
            Self::BinShuffled | Self::ChannelShuffled | Self::Reversed => OperatorInvariants {
                relocated: (0.5, 1.0),
                max_displacement: None,
                min_mean_displacement_fraction: Some(FULL_SHUFFLE_DISPLACEMENT_FLOOR),
                hidden_permutation_relocated: (0.0, 0.0),
                retention: RetentionRule::Exact,
            },
            Self::WindowShuffled { window } => OperatorInvariants {
                // A uniform permutation of `w` items leaves exactly one fixed
                // point in expectation, so the relocated fraction concentrates
                // on `1 - 1/w`. The lower edge is slack enough for the short
                // final window and for occupancy that is not uniform across
                // windows; it is still far above what "did nothing" (0.0)
                // would produce, which is the failure it has to catch. For
                // large `w` this band cannot separate a window shuffle from a
                // full shuffle — `max_displacement` is what does that, and it
                // is exact.
                relocated: (relocated_floor(window), 1.0),
                max_displacement: Some((window - 1) as f64),
                // Deliberately none: the whole point of the low rungs is that
                // they displace by very little.
                min_mean_displacement_fraction: None,
                hidden_permutation_relocated: (0.0, 0.0),
                retention: RetentionRule::Exact,
            },
            Self::HiddenShuffled => OperatorInvariants {
                // The input is untouched, so every input-side statistic reads
                // exactly as `intact` does. That is the point and it is also
                // the danger: without the clause below, a `hidden-shuffled`
                // cell whose permutation was never built would produce an audit
                // indistinguishable from an intact one and pass every gate.
                relocated: (0.0, 0.0),
                max_displacement: Some(0.0),
                min_mean_displacement_fraction: None,
                retention: RetentionRule::Exact,
                hidden_permutation_relocated: (0.5, 1.0),
            },
            Self::SpikeDropout { percent } => OperatorInvariants {
                relocated: (0.0, 0.0),
                max_displacement: Some(0.0),
                min_mean_displacement_fraction: None,
                hidden_permutation_relocated: (0.0, 0.0),
                retention: RetentionRule::Binomial {
                    survival: 1.0 - f64::from(percent) / 100.0,
                    sigmas: DROPOUT_BAND_SIGMAS,
                },
            },
        }
    }
}

/// Lower edge of the registered `relocated_fraction` band for a window shuffle.
///
/// `1 - 1/w` is the expectation; the 0.15 of slack absorbs the short final
/// window and non-uniform occupancy across windows. Named rather than inlined
/// because `scripts/cell_validity.py` carries the same rule and
/// `test_campaign_tooling.py` pins the two together.
fn relocated_floor(window: usize) -> f64 {
    (1.0 - 1.0 / window as f64 - 0.15).max(0.0)
}

/// Registered floor on a full shuffle's mean displacement, as a fraction of the
/// sequence length. Half the `T/3` a uniform permutation gives in expectation.
pub const FULL_SHUFFLE_DISPLACEMENT_FLOOR: f64 = 1.0 / 6.0;

/// Width of the binomial band on dropout retention, in standard deviations.
///
/// Five, not three. This band is a **validity gate on a manipulation**, not a
/// hypothesis test: a false rejection voids a legitimate multi-hour cell, and
/// the thing it is built to catch — a mask that was never applied, applied
/// twice, or redrawn per epoch — misses by a margin far larger than five
/// sigmas. At the anchor's spike count the band is still tighter than 0.003.
pub const DROPOUT_BAND_SIGMAS: f64 = 5.0;

/// How much of the spike count an operator promises to leave behind.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RetentionRule {
    /// Every spike survives; per-channel totals are compared bit-exactly
    /// inside [`apply_temporal`] and the ratio here must be exactly 1.
    Exact,
    /// Each spike survives independently with probability `survival`. Checked
    /// against a `sigmas`-wide band whose width is computed from the number of
    /// spikes the operator actually saw, so the same rule is meaningful on a
    /// six-spike fixture and on the 8,156-sample training set.
    Binomial { survival: f64, sigmas: f64 },
}

/// What one operator promises, and the only thing that decides whether a
/// manipulated run is valid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OperatorInvariants {
    /// Inclusive band on [`TemporalAudit::relocated_fraction`].
    pub relocated: (f64, f64),
    /// Inclusive bound on [`TemporalAudit::max_bin_displacement`], where the
    /// operator bounds it. `None` means unbounded — not unchecked; the
    /// displacement floor below is what carries the weight there.
    pub max_displacement: Option<f64>,
    /// Floor on [`TemporalAudit::mean_bin_displacement`], as a fraction of
    /// [`TemporalAudit::mean_steps`].
    ///
    /// # Why an operator needs a floor as well as a ceiling
    ///
    /// Without it, `bin-shuffled`'s invariants accept a `window-shuffled-w4`
    /// audit: a window shuffle preserves counts and relocates three quarters of
    /// the dataset, so it satisfies everything a full shuffle promises. The two
    /// operators sit at opposite ends of one ladder and the registry has to be
    /// able to tell them apart, or a mislabelled cell — a plan typo, a reused
    /// weight file — enters the corpus as the wrong rung and the timescale
    /// estimator reads a curve that was never run.
    ///
    /// A uniform permutation of `T` bins displaces by `(T^2 - 1) / 3T ~ T/3` in
    /// expectation. The registered floor is half of that, so it is slack
    /// against occupancy that clusters in time and still an order of magnitude
    /// above what any small window can reach.
    pub min_mean_displacement_fraction: Option<f64>,
    /// Inclusive band on [`TemporalAudit::hidden_permutation_relocated`].
    ///
    /// `(0.0, 0.0)` for every operator that works on the input, which is not a
    /// vacuous clause: it is what stops a read-out permutation from riding
    /// along unnoticed on a cell labelled `bin-shuffled`.
    pub hidden_permutation_relocated: (f64, f64),
    pub retention: RetentionRule,
}

impl OperatorInvariants {
    /// Check a completed dataset-level audit. `Err` voids the run.
    ///
    /// Every clause names the operator, the observed value and the registered
    /// bound, because this error is the only thing a campaign operator sees
    /// when a wave refuses to start.
    pub fn check(&self, label: &str, audit: &TemporalAudit) -> Result<(), String> {
        if audit.samples == 0 {
            return Err(format!(
                "temporal condition {label} audited zero samples - the manipulation \
                 never ran and nothing below could have failed"
            ));
        }
        let (low, high) = self.relocated;
        if audit.relocated_fraction < low || audit.relocated_fraction > high {
            return Err(format!(
                "temporal condition {label} relocated {:.4} of entries, outside its \
                 registered band [{low:.4}, {high:.4}] - the manipulation is not doing \
                 what it claims",
                audit.relocated_fraction
            ));
        }
        let (low, high) = self.hidden_permutation_relocated;
        if audit.hidden_permutation_relocated < low || audit.hidden_permutation_relocated > high {
            return Err(format!(
                "temporal condition {label} permuted {:.4} of the read-out's timesteps, \
                 outside its registered band [{low:.4}, {high:.4}]",
                audit.hidden_permutation_relocated
            ));
        }
        if let Some(fraction) = self.min_mean_displacement_fraction {
            let floor = fraction * audit.mean_steps;
            if audit.mean_bin_displacement < floor {
                return Err(format!(
                    "temporal condition {label} displaced entries by {:.3} bins on a \
                     {:.1}-bin sequence, below its registered floor of {floor:.3} - this \
                     is what a narrower manipulation wearing this label looks like",
                    audit.mean_bin_displacement, audit.mean_steps
                ));
            }
        }
        if let Some(bound) = self.max_displacement {
            if audit.max_bin_displacement > bound {
                return Err(format!(
                    "temporal condition {label} moved an entry {:.1} bins, past its \
                     registered bound of {bound:.1}",
                    audit.max_bin_displacement
                ));
            }
        }
        match self.retention {
            RetentionRule::Exact => {
                if !audit.counts_preserved || audit.count_mismatches > 0 {
                    return Err(format!(
                        "temporal condition {label} registers exact counts and changed \
                         them on {} samples - prereg gate 5.1 voids this run",
                        audit.count_mismatches
                    ));
                }
                if audit.count_after.to_bits() != audit.count_before.to_bits() {
                    return Err(format!(
                        "temporal condition {label} registers exact counts; total spikes \
                         went from {} to {}",
                        audit.count_before, audit.count_after
                    ));
                }
            }
            RetentionRule::Binomial { survival, sigmas } => {
                if audit.count_before <= 0.0 {
                    return Err(format!(
                        "temporal condition {label} saw no spikes to delete"
                    ));
                }
                let observed = audit.count_after / audit.count_before;
                // Standard error of the mean of `n` Bernoulli(survival) draws.
                let deviation =
                    (survival * (1.0 - survival) / audit.count_before).sqrt() * sigmas;
                if (observed - survival).abs() > deviation {
                    return Err(format!(
                        "temporal condition {label} retained {observed:.6} of {} spikes; \
                         the registered band is {survival:.6} +/- {deviation:.6} \
                         ({sigmas} sigma). A mask that was never applied, applied twice, \
                         or redrawn per epoch lands outside this.",
                        audit.count_before
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Evidence that the manipulation did what it claims, emitted per cell.
#[derive(Clone, Debug, PartialEq)]
pub struct TemporalAudit {
    pub samples: usize,
    /// Prereg §5.1. False here voids the run.
    pub counts_preserved: bool,
    /// Samples whose per-channel totals changed. Must be 0.
    pub count_mismatches: usize,
    /// Mean |new_bin - old_bin| over relocated entries, in bins.
    ///
    /// A manipulation that reports `counts_preserved` but near-zero
    /// displacement has silently done nothing - the failure mode that would
    /// make a null result meaningless.
    pub mean_bin_displacement: f64,
    /// Fraction of (bin, channel) entries that changed bin. ~0 is a red flag
    /// for every condition except `intact`.
    pub relocated_fraction: f64,
    /// Mean occupied bins per sample, before and after. Equal for bin-shuffled
    /// and reversed; may fall for channel-shuffled as entries collide into
    /// fewer distinct bins.
    pub occupied_bins_before: f64,
    pub occupied_bins_after: f64,
    /// Largest |new_bin - old_bin| over every entry of every sample, in bins.
    ///
    /// The mean cannot express a *bound*. `window-shuffled-w8` and a full
    /// shuffle can share a mean displacement while differing in exactly the
    /// property the window operator is named for, so the timescale ladder needs
    /// the max and the max is what [`OperatorInvariants`] checks against.
    pub max_bin_displacement: f64,
    /// Mean bins per sample.
    ///
    /// Carried so a displacement bound can be expressed as a *fraction of the
    /// sequence*, which is the only form in which it is comparable across the
    /// `published-2ms` (~358 bins) and `fixed-t100` (100 bins) contracts. A
    /// bound in absolute bins would be two different gates wearing one number.
    pub mean_steps: f64,
    /// Fraction of read-out timesteps whose presentation position changed.
    ///
    /// Zero for every operator that works on the input — including `intact` —
    /// and the only field that distinguishes a `hidden-shuffled` audit from an
    /// intact one, because `hidden-shuffled` does not touch the input at all.
    pub hidden_permutation_relocated: f64,
    /// Total spikes before and after, summed over channels, bins and samples.
    ///
    /// Counts, not means. The dropout retention band is binomial, so its width
    /// is a function of how many spikes were actually drawn on; a per-sample
    /// mean would have thrown that number away.
    pub count_before: f64,
    pub count_after: f64,
}

impl Default for TemporalAudit {
    /// `counts_preserved` defaults to **true**, not `false`.
    ///
    /// [`TemporalAudit::merge`] folds it with `&=`, so the default value is the
    /// identity element of a conjunction over zero samples - vacuously true,
    /// nothing has violated it yet. Deriving `Default` gives `false`, which is
    /// absorbing rather than identity: every fold starting from a derived
    /// default would report `counts_preserved: false` no matter what the samples
    /// actually did, writing a false gate violation into every cell record.
    fn default() -> Self {
        Self {
            samples: 0,
            counts_preserved: true,
            count_mismatches: 0,
            mean_bin_displacement: 0.0,
            relocated_fraction: 0.0,
            occupied_bins_before: 0.0,
            occupied_bins_after: 0.0,
            // Identity elements again: 0 for a running max over a non-negative
            // quantity, and 0 for two running sums.
            max_bin_displacement: 0.0,
            mean_steps: 0.0,
            hidden_permutation_relocated: 0.0,
            count_before: 0.0,
            count_after: 0.0,
        }
    }
}

/// Per-channel totals, summed in bin order.
///
/// # Why the bit-comparison against these is safe
///
/// `apply_temporal` compares before/after totals with `to_bits() == to_bits()`,
/// and shuffling changes the order the addends arrive in. Float addition is not
/// associative, so in general that comparison could fail on a manipulation that
/// preserved every count — a spurious hard failure voiding a valid run, on the
/// prereg's blocking gate 5.1.
///
/// It is safe here because framed counts are **integer-valued**: `frame_events`
/// counts events per (bin, channel), and `adjacent-sum-5` sums integers. f32
/// represents every integer below `2^24` exactly and their sums are therefore
/// order-independent, and per-channel totals over an SHD sample are in the
/// thousands at most.
///
/// This is an assumption, not an invariant the type system carries. If counts
/// ever become non-integral — normalisation, weighting, a float-valued
/// geometry — the bit-comparison must become an exact-sum or tolerance
/// comparison at the same time. `integer_counts_survive_reordering` pins it.
fn channel_totals(sample: &MatchedShdSample) -> Vec<f32> {
    let mut totals = vec![0.0_f32; sample.n_inputs];
    for frame in &sample.frames {
        for &(channel, count) in frame {
            totals[channel] += count;
        }
    }
    totals
}

fn occupied(sample: &MatchedShdSample) -> usize {
    sample
        .frames
        .iter()
        .filter(|frame| !frame.is_empty())
        .count()
}

/// Rebuild `frames` from `(bin, channel, count)` triples, restoring the framing
/// invariant that each bin is sorted ascending by channel and holds one entry
/// per channel.
fn rebuild(steps: usize, entries: Vec<(usize, usize, f32)>) -> Vec<Vec<(usize, f32)>> {
    let mut frames: Vec<Vec<(usize, f32)>> = vec![Vec::new(); steps];
    for (bin, channel, count) in entries {
        frames[bin].push((channel, count));
    }
    for frame in frames.iter_mut() {
        frame.sort_by_key(|&(channel, _)| channel);
        // Independent per-channel permutation can land two entries for the same
        // channel in one bin; merging keeps counts exact and the invariant true.
        let mut merged: Vec<(usize, f32)> = Vec::with_capacity(frame.len());
        for &(channel, count) in frame.iter() {
            match merged.last_mut() {
                Some((last_channel, last_count)) if *last_channel == channel => {
                    *last_count += count;
                }
                _ => merged.push((channel, count)),
            }
        }
        *frame = merged;
    }
    frames
}

/// Running evidence that a manipulation moved what it claims to move.
///
/// Threaded through the per-operator arms rather than recomputed per arm, so
/// the accounting cannot drift between operators — the `relocated_fraction`
/// of a window shuffle has to mean exactly what a bin shuffle's does for the
/// two to sit on the same ladder.
#[derive(Default)]
struct MoveStats {
    entries: usize,
    relocated: usize,
    displacement_sum: f64,
    max_displacement: f64,
}

impl MoveStats {
    fn saw(&mut self, old_bin: usize, new_bin: usize) {
        self.entries += 1;
        if new_bin != old_bin {
            self.relocated += 1;
            let distance = (new_bin as f64 - old_bin as f64).abs();
            self.displacement_sum += distance;
            if distance > self.max_displacement {
                self.max_displacement = distance;
            }
        }
    }
}

/// Move every entry by one bin permutation shared across channels.
///
/// `bin-shuffled` and `window-shuffled-wN` differ **only** in how the
/// permutation is drawn: one over the whole bin axis, one within disjoint
/// windows. Sharing the application is what makes that the only difference,
/// rather than a claim in a comment.
fn permute_bins(
    sample: &mut MatchedShdSample,
    steps: usize,
    permutation: &[usize],
    stats: &mut MoveStats,
) {
    let mut entries = Vec::new();
    for (old_bin, frame) in sample.frames.iter().enumerate() {
        let new_bin = permutation[old_bin];
        for &(channel, count) in frame {
            stats.saw(old_bin, new_bin);
            entries.push((new_bin, channel, count));
        }
    }
    sample.frames = rebuild(steps, entries);
}

/// Total spikes in a sample, summed in (bin, channel) order.
///
/// f64 rather than f32: this is a dataset-scale accumulator behind the dropout
/// retention band, and SHD's training set carries millions of spikes — well
/// past the 2^24 where an f32 running sum stops counting by ones. Counts
/// themselves stay f32 and integral; only the total widens.
fn total_count(sample: &MatchedShdSample) -> f64 {
    sample
        .frames
        .iter()
        .flat_map(|frame| frame.iter())
        .map(|&(_, count)| f64::from(count))
        .sum()
}

/// Apply `condition` in place. `seed` must be derived from the cell seed so the
/// manipulation is reproducible and identical across backends.
///
/// # The mask is frozen here, and that is the point
///
/// Every operator draws from a [`PortableRng`] seeded only by `seed`, and the
/// campaign calls this **once per sample before the first epoch**. So the
/// realisation a cell trains against is fixed for the whole run.
///
/// For the shuffles that is incidental. For [`TemporalCondition::SpikeDropout`]
/// it is the difference between two experiments. Redrawn each epoch, dropout is
/// *noise augmentation* — a regulariser that Cramer et al. (2022) report
/// **improves** SHD generalisation — and it would lift both arms while wearing
/// the label of an information-removal control. Frozen, the network sees one
/// impoverished dataset and the question is whether the read-out can still use
/// it. Nothing in this function can be called per epoch without the caller
/// re-deriving a seed, and `dropout_is_a_frozen_mask_not_per_epoch_noise` pins
/// the realisation itself.
pub fn apply_temporal(
    sample: &mut MatchedShdSample,
    condition: TemporalCondition,
    seed: u64,
) -> Result<TemporalAudit, String> {
    let steps = sample.frames.len();
    let before_totals = channel_totals(sample);
    let before_occupied = occupied(sample);
    let count_before = total_count(sample);

    let mut stats = MoveStats::default();
    // Set only by `hidden-shuffled`; zero everywhere else, which is exactly
    // what every other operator's invariants require.
    let mut hidden_moved = 0_usize;

    match condition {
        TemporalCondition::Intact => {}
        TemporalCondition::Reversed => {
            sample.frames.reverse();
            for (new_bin, frame) in sample.frames.iter().enumerate() {
                let old_bin = steps - 1 - new_bin;
                for _ in frame {
                    stats.saw(old_bin, new_bin);
                }
            }
        }
        TemporalCondition::BinShuffled => {
            // One permutation shared by every channel: order dies, within-bin
            // synchrony survives.
            let mut rng = PortableRng::new(seed);
            let mut permutation: Vec<usize> = (0..steps).collect();
            rng.shuffle(&mut permutation);
            permute_bins(sample, steps, &permutation, &mut stats);
        }
        TemporalCondition::WindowShuffled { window } => {
            // Disjoint windows, permuted independently. A bin never leaves its
            // own window, so displacement is bounded by `window - 1` and every
            // structure coarser than the window survives intact — which is what
            // makes sweeping `window` a timescale measurement rather than a
            // second shuffle.
            //
            // The final window is short when `steps` is not a multiple of
            // `window`. It is permuted at its own length rather than padded:
            // padding would let a bin move to an index that does not exist, and
            // dropping it would leave a tail of the utterance unmanipulated.
            let mut rng = PortableRng::new(seed);
            let mut permutation: Vec<usize> = (0..steps).collect();
            for start in (0..steps).step_by(window) {
                let end = (start + window).min(steps);
                let mut local: Vec<usize> = (0..end - start).collect();
                rng.shuffle(&mut local);
                for (offset, &target) in local.iter().enumerate() {
                    permutation[start + offset] = start + target;
                }
            }
            permute_bins(sample, steps, &permutation, &mut stats);
        }
        TemporalCondition::ChannelShuffled => {
            // One permutation per channel: order and cross-channel synchrony
            // both die. Seeds are offset by channel so no two channels share a
            // permutation.
            let mut permutations: Vec<Vec<usize>> = Vec::with_capacity(sample.n_inputs);
            for channel in 0..sample.n_inputs {
                let mut rng = PortableRng::new(
                    seed ^ ((channel as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15)),
                );
                let mut permutation: Vec<usize> = (0..steps).collect();
                rng.shuffle(&mut permutation);
                permutations.push(permutation);
            }
            let mut entries = Vec::new();
            for (old_bin, frame) in sample.frames.iter().enumerate() {
                for &(channel, count) in frame {
                    let new_bin = permutations[channel][old_bin];
                    stats.saw(old_bin, new_bin);
                    entries.push((new_bin, channel, count));
                }
            }
            sample.frames = rebuild(steps, entries);
        }
        TemporalCondition::HiddenShuffled => {
            // The frames are not touched. The permutation is attached to the
            // sample and consumed by the read-out, after the membrane loop has
            // already produced a spike train identical to the intact twin's.
            let mut rng = PortableRng::new(seed);
            let mut permutation: Vec<usize> = (0..steps).collect();
            rng.shuffle(&mut permutation);
            hidden_moved = permutation
                .iter()
                .enumerate()
                .filter(|&(position, &source)| position != source)
                .count();
            sample.hidden_time_permutation = Some(permutation);
        }
        TemporalCondition::SpikeDropout { percent } => {
            // Nothing moves; individual spikes disappear. Draws are taken in
            // (bin, channel) order — bins ascending, and within a bin the
            // channels ascending, which `rebuild` and `frame_events` both
            // guarantee — so the mask is a pure function of `seed` and the
            // sample, on any backend and any thread count.
            let mut rng = PortableRng::new(seed);
            let deleted_below = u64::from(percent);
            for (bin, frame) in sample.frames.iter_mut().enumerate() {
                for entry in frame.iter_mut() {
                    let count = entry.1;
                    // The whole operator rests on counts being whole spikes.
                    // If a geometry ever produced fractional or negative
                    // counts, `trials` below would silently truncate and the
                    // retention band would be checking a quantity that no
                    // longer means what it says.
                    if !count.is_finite() || count < 0.0 || count.fract() != 0.0 {
                        return Err(format!(
                            "spike-dropout needs integral non-negative counts; bin {bin} \
                             channel {} carries {count}",
                            entry.0
                        ));
                    }
                    if count > MAX_EXACT_INTEGER_F32 {
                        return Err(format!(
                            "spike-dropout count {count} at bin {bin} channel {} exceeds \
                             2^24, past which f32 no longer counts by ones",
                            entry.0
                        ));
                    }
                    let trials = count as u64;
                    let mut kept = 0.0_f32;
                    for _ in 0..trials {
                        // Integer comparison, not `uniform(0.0, 1.0) < p`.
                        // Modulo bias against 2^64 is ~1e-17 and the result is
                        // identical on every backend, where a float threshold
                        // would depend on rounding at the boundary.
                        if rng.next_u64() % 100 >= deleted_below {
                            kept += 1.0;
                        }
                    }
                    entry.1 = kept;
                    stats.saw(bin, bin);
                }
                // An entry whose every spike was deleted is not an entry with
                // a zero count: `frame_events` never emits one, and leaving it
                // would make `occupied_bins_after` and every downstream
                // sparsity statistic describe a frame shape the pipeline
                // cannot otherwise produce.
                frame.retain(|&(_, count)| count != 0.0);
            }
        }
    }

    let after_totals = channel_totals(sample);
    let mismatched = before_totals
        .iter()
        .zip(after_totals.iter())
        .any(|(before, after)| before.to_bits() != after.to_bits());
    // Gate 5.1, and it stays a hard error rather than a recorded field for the
    // operators that register exact counts. Deferring it to
    // `OperatorInvariants::check` would let a count-changing shuffle run the
    // whole dataset before anything complained, and would put the campaign's
    // oldest blocking gate behind a table that is easier to edit than this
    // line is. Dropout is exempt because it registers a binomial retention,
    // not an exact one — `RetentionRule` is the authority on which is which.
    if mismatched && matches!(condition.invariants().retention, RetentionRule::Exact) {
        return Err(format!(
            "temporal condition {} changed per-channel spike counts - prereg gate 5.1 voids this run",
            condition.label()
        ));
    }

    Ok(TemporalAudit {
        samples: 1,
        counts_preserved: !mismatched,
        count_mismatches: usize::from(mismatched),
        mean_bin_displacement: if stats.relocated == 0 {
            0.0
        } else {
            stats.displacement_sum / stats.relocated as f64
        },
        relocated_fraction: if stats.entries == 0 {
            0.0
        } else {
            stats.relocated as f64 / stats.entries as f64
        },
        occupied_bins_before: before_occupied as f64,
        occupied_bins_after: occupied(sample) as f64,
        max_bin_displacement: stats.max_displacement,
        mean_steps: steps as f64,
        hidden_permutation_relocated: if steps == 0 {
            0.0
        } else {
            hidden_moved as f64 / steps as f64
        },
        count_before,
        count_after: total_count(sample),
    })
}

/// Largest integer f32 represents exactly. See [`channel_totals`].
const MAX_EXACT_INTEGER_F32: f32 = 16_777_216.0;


impl TemporalAudit {
    /// Fold a per-sample audit into a running dataset-level audit.
    pub fn merge(&mut self, other: &TemporalAudit) {
        let total = self.samples + other.samples;
        let weight = |value_a: f64, count_a: usize, value_b: f64, count_b: usize| {
            if count_a + count_b == 0 {
                0.0
            } else {
                (value_a * count_a as f64 + value_b * count_b as f64) / (count_a + count_b) as f64
            }
        };
        self.mean_bin_displacement = weight(
            self.mean_bin_displacement,
            self.samples,
            other.mean_bin_displacement,
            other.samples,
        );
        self.relocated_fraction = weight(
            self.relocated_fraction,
            self.samples,
            other.relocated_fraction,
            other.samples,
        );
        self.occupied_bins_before = weight(
            self.occupied_bins_before,
            self.samples,
            other.occupied_bins_before,
            other.samples,
        );
        self.occupied_bins_after = weight(
            self.occupied_bins_after,
            self.samples,
            other.occupied_bins_after,
            other.samples,
        );
        self.mean_steps = weight(
            self.mean_steps,
            self.samples,
            other.mean_steps,
            other.samples,
        );
        self.hidden_permutation_relocated = weight(
            self.hidden_permutation_relocated,
            self.samples,
            other.hidden_permutation_relocated,
            other.samples,
        );
        self.counts_preserved &= other.counts_preserved;
        self.count_mismatches += other.count_mismatches;
        // A bound is a max, not a mean. Folding this the way the fractions
        // above are folded would let one sample's out-of-window move be
        // averaged away by ten thousand well-behaved ones, and the window
        // operator's only exact invariant would stop being checkable.
        if other.max_bin_displacement > self.max_bin_displacement {
            self.max_bin_displacement = other.max_bin_displacement;
        }
        // Sums, so the retention band keeps its dependence on how many spikes
        // were actually drawn on.
        self.count_before += other.count_before;
        self.count_after += other.count_after;
        self.samples = total;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shd_attention::{AttentionConfig, AttentionParams};
    use crate::shd_matched::MatchedWeights;
    use crate::shd_matched_arms::{loss_and_gradient_arm, ArmWeights, MatchedArm};

    /// Gate 5.1 compares per-channel totals bit-exactly, and shuffling changes
    /// the order they are summed in. That is only sound because counts are
    /// integer-valued — see [`channel_totals`]. This pins both halves: large
    /// integer counts survive reordering exactly, and non-integral ones do not.
    ///
    /// The second assertion is the point. It documents that the current gate
    /// would start failing spuriously if counts ever stopped being integers,
    /// so the failure arrives here rather than as a voided campaign run.
    #[test]
    fn integer_counts_survive_reordering() {
        let integral: Vec<f32> = (1..=64).map(|k| (k * 7 % 23) as f32).collect();
        let forward: f32 = integral.iter().sum();
        let backward: f32 = integral.iter().rev().sum();
        assert_eq!(
            forward.to_bits(),
            backward.to_bits(),
            "integer-valued counts must sum order-independently in f32",
        );

        // The bound is real, and this is where it bites: 2^24 is the largest
        // integer f32 represents exactly, so above it addition stops being
        // order-independent. Summing high-to-low loses both ones; low-to-high
        // keeps them.
        //
        // An earlier version of this test tried to demonstrate the same point
        // with fractional counts `k * 0.1` and asserted the two orders differ.
        // They did not — that particular sequence reassociates exactly — which
        // is a good reminder that "floats are inexact" is not a property you
        // can assert with an arbitrary example. The 2^24 boundary is the
        // documented claim, so it is the one worth pinning.
        let past_the_bound = [16_777_216.0_f32, 1.0, 1.0];
        let high_first: f32 = past_the_bound.iter().sum();
        let low_first: f32 = past_the_bound.iter().rev().sum();
        assert_ne!(
            high_first.to_bits(),
            low_first.to_bits(),
            "2^24 is the stated bound on order-independence; if this stops \
             holding the comment on channel_totals is wrong",
        );
        assert_eq!(low_first, 16_777_218.0, "low-to-high keeps both increments");
    }

    fn sample() -> MatchedShdSample {
        let mut frames = Vec::new();
        for t in 0..40 {
            let mut frame = Vec::new();
            for k in 0..5 {
                frame.push((((t * 3 + k * 7) % 20) as usize, 1.0_f32));
            }
            frame.sort_by_key(|&(channel, _)| channel);
            frame.dedup_by_key(|entry| entry.0);
            frames.push(frame);
        }
        MatchedShdSample {
            label: 3,
            frames,
            n_inputs: 20,
            dt_ms: 10.0,
            hidden_time_permutation: None,
        }
    }

    /// PREREG GATE 5.1. Every condition must preserve per-channel totals exactly.
    #[test]
    fn every_condition_preserves_channel_counts() {
        for condition in TemporalCondition::ALL {
            let original = sample();
            let before = channel_totals(&original);
            let mut manipulated = original.clone();
            apply_temporal(&mut manipulated, condition, 77)
                .unwrap_or_else(|e| panic!("{}: {e}", condition.label()));
            let after = channel_totals(&manipulated);
            assert_eq!(
                before,
                after,
                "condition {} changed counts",
                condition.label()
            );
        }
    }

    /// A manipulation that silently does nothing would make a null meaningless.
    #[test]
    fn non_identity_conditions_actually_move_spikes() {
        for condition in [
            TemporalCondition::BinShuffled,
            TemporalCondition::ChannelShuffled,
            TemporalCondition::Reversed,
        ] {
            let mut manipulated = sample();
            let audit = apply_temporal(&mut manipulated, condition, 77).unwrap();
            assert!(
                audit.relocated_fraction > 0.5,
                "condition {} relocated only {:.3} of entries",
                condition.label(),
                audit.relocated_fraction
            );
            assert!(audit.mean_bin_displacement > 1.0, "{}", condition.label());
            assert_ne!(manipulated.frames, sample().frames, "{}", condition.label());
        }
    }

    #[test]
    fn intact_is_a_no_op() {
        let mut manipulated = sample();
        let audit = apply_temporal(&mut manipulated, TemporalCondition::Intact, 77).unwrap();
        assert_eq!(manipulated.frames, sample().frames);
        assert_eq!(audit.relocated_fraction, 0.0);
    }

    /// bin-shuffled keeps whole bins together; channel-shuffled does not.
    /// That difference is the order/synchrony decomposition the prereg rests on.
    #[test]
    fn bin_shuffle_preserves_within_bin_synchrony_channel_shuffle_does_not() {
        let mut binned = sample();
        apply_temporal(&mut binned, TemporalCondition::BinShuffled, 5).unwrap();
        let original_bins: Vec<Vec<(usize, f32)>> = sample().frames;
        for frame in &binned.frames {
            if frame.is_empty() {
                continue;
            }
            assert!(
                original_bins.contains(frame),
                "bin-shuffled must permute intact bins, found a bin that never existed"
            );
        }
        let mut channelled = sample();
        apply_temporal(&mut channelled, TemporalCondition::ChannelShuffled, 5).unwrap();
        let novel = channelled
            .frames
            .iter()
            .filter(|frame| !frame.is_empty() && !original_bins.contains(frame))
            .count();
        assert!(
            novel > 0,
            "channel-shuffled should create bins that never co-occurred"
        );
    }

    #[test]
    fn reversed_is_its_own_inverse() {
        let mut once = sample();
        apply_temporal(&mut once, TemporalCondition::Reversed, 0).unwrap();
        apply_temporal(&mut once, TemporalCondition::Reversed, 0).unwrap();
        assert_eq!(once.frames, sample().frames);
    }

    /// Regression: a derived `Default` made `counts_preserved` absorbing under
    /// the `&=` fold, so every merged audit reported a gate violation that had
    /// not happened.
    #[test]
    fn default_audit_is_the_identity_of_the_merge_fold() {
        let empty = TemporalAudit::default();
        assert!(
            empty.counts_preserved,
            "default must be the conjunction identity"
        );
        assert_eq!(empty.samples, 0);

        let mut folded = TemporalAudit::default();
        let mut manipulated = sample();
        let single = apply_temporal(&mut manipulated, TemporalCondition::BinShuffled, 1).unwrap();
        folded.merge(&single);
        assert_eq!(
            folded, single,
            "folding one audit into the default must be a no-op"
        );
    }

    #[test]
    fn audit_merge_averages_by_sample_count() {
        let mut total = TemporalAudit::default();
        for seed in 0..4 {
            let mut manipulated = sample();
            let audit =
                apply_temporal(&mut manipulated, TemporalCondition::BinShuffled, seed).unwrap();
            total.merge(&audit);
        }
        assert_eq!(total.samples, 4);
        assert!(total.counts_preserved);
        assert!(total.mean_bin_displacement > 1.0);
    }

    /// A denser, longer fixture than [`sample`]: 97 bins, counts above one, and
    /// uneven occupancy. The window and dropout operators are both defined by
    /// bounds that a 40-bin all-ones fixture cannot exercise.
    fn dense_sample(salt: u64) -> MatchedShdSample {
        let mut frames = Vec::new();
        for t in 0..97 {
            let mut frame = Vec::new();
            for c in 0..11_usize {
                if (t * 7 + c * 13 + salt as usize) % 5 < 2 {
                    frame.push((c, ((t + c) % 4 + 1) as f32));
                }
            }
            frames.push(frame);
        }
        MatchedShdSample {
            label: 3,
            frames,
            n_inputs: 11,
            dt_ms: 2.0,
            hidden_time_permutation: None,
        }
    }

    fn fingerprint(sample: &MatchedShdSample) -> u64 {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for (bin, frame) in sample.frames.iter().enumerate() {
            for &(channel, count) in frame {
                for byte in (bin as u64)
                    .to_le_bytes()
                    .iter()
                    .chain((channel as u64).to_le_bytes().iter())
                    .chain(count.to_bits().to_le_bytes().iter())
                {
                    hash ^= u64::from(*byte);
                    hash = hash.wrapping_mul(0x100_0000_01b3);
                }
            }
        }
        hash
    }

    fn pinned(condition: TemporalCondition) -> u64 {
        let mut accumulator = 0_u64;
        for seed in 1_u64..=6 {
            let mut manipulated = dense_sample(seed);
            let audit = apply_temporal(
                &mut manipulated,
                condition,
                seed.wrapping_mul(0x9E37_79B9_7F4A_7C15),
            )
            .unwrap();
            accumulator ^= fingerprint(&manipulated)
                .wrapping_mul(31)
                .wrapping_add(audit.relocated_fraction.to_bits())
                .wrapping_add(audit.mean_bin_displacement.to_bits())
                .wrapping_add(audit.occupied_bins_after.to_bits());
        }
        accumulator
    }

    /// The three original operators are byte-frozen.
    ///
    /// # Why a hash and not a property
    ///
    /// 784 archived cells were produced by these three manipulations, and the
    /// campaign's reproduction gate re-runs them and demands every recorded
    /// digit. A refactor that changed which permutation a seed produces — or
    /// merely the order entries are pushed in before `rebuild` sorts them —
    /// would not fail any property test in this file, would not fail to
    /// compile, and would silently retire the corpus.
    ///
    /// These three values were measured against the code as it stood before
    /// `permute_bins` and `MoveStats` were extracted, and they did not move.
    /// If one of them moves again, the change is not a refactor.
    #[test]
    fn the_three_original_operators_are_byte_frozen() {
        assert_eq!(
            pinned(TemporalCondition::BinShuffled),
            0x1a53_b7ca_2ca1_7daa,
            "bin-shuffled",
        );
        assert_eq!(
            pinned(TemporalCondition::ChannelShuffled),
            0x3d41_1940_07bd_aef5,
            "channel-shuffled",
        );
        assert_eq!(
            pinned(TemporalCondition::Reversed),
            0x8d9a_010a_0236_1144,
            "reversed",
        );
    }

    #[test]
    fn labels_round_trip_through_parse() {
        let conditions = [
            TemporalCondition::Intact,
            TemporalCondition::BinShuffled,
            TemporalCondition::ChannelShuffled,
            TemporalCondition::Reversed,
            TemporalCondition::WindowShuffled { window: 2 },
            TemporalCondition::WindowShuffled { window: 64 },
            TemporalCondition::SpikeDropout { percent: 1 },
            TemporalCondition::SpikeDropout { percent: 50 },
        ];
        for condition in conditions {
            let label = condition.label();
            assert_eq!(
                TemporalCondition::parse(&label).unwrap(),
                condition,
                "{label} did not survive a round trip"
            );
        }
    }

    /// Both degenerate parameterisations are refused at parse time, before a
    /// cell exists — the identity wearing a manipulation's label is exactly
    /// what the relocated gate was added to catch, and catching it here costs
    /// nothing instead of a multi-hour run.
    #[test]
    fn degenerate_operator_parameters_are_refused() {
        assert!(TemporalCondition::parse("window-shuffled-w1").is_err());
        assert!(TemporalCondition::parse("window-shuffled-w0").is_err());
        assert!(TemporalCondition::parse("spike-dropout-p0").is_err());
        assert!(TemporalCondition::parse("spike-dropout-p100").is_err());
        assert!(TemporalCondition::parse("spike-dropout-p150").is_err());
        assert!(TemporalCondition::parse("window-shuffled-wide").is_err());
    }

    /// The bound the window operator is named for, checked at every window on a
    /// sequence whose length is not a multiple of any of them.
    #[test]
    fn a_window_shuffle_never_moves_an_entry_out_of_its_window() {
        for window in [2_usize, 3, 8, 25, 64] {
            let condition = TemporalCondition::window_shuffled(window).unwrap();
            let mut audit = TemporalAudit::default();
            for seed in 1_u64..=8 {
                let mut manipulated = dense_sample(seed);
                audit.merge(&apply_temporal(&mut manipulated, condition, seed * 7).unwrap());
            }
            assert!(
                audit.max_bin_displacement <= (window - 1) as f64,
                "w{window}: moved {} bins",
                audit.max_bin_displacement
            );
            assert!(audit.counts_preserved, "w{window} changed counts");
            condition
                .invariants()
                .check(&condition.label(), &audit)
                .unwrap_or_else(|error| panic!("w{window}: {error}"));
        }
    }

    /// The ladder has to actually be a ladder: a wider window must move entries
    /// further on average, or sweeping it measures nothing.
    #[test]
    fn a_wider_window_displaces_further() {
        let mut previous = 0.0_f64;
        for window in [2_usize, 4, 8, 16, 32] {
            let condition = TemporalCondition::window_shuffled(window).unwrap();
            let mut audit = TemporalAudit::default();
            for seed in 1_u64..=8 {
                let mut manipulated = dense_sample(seed);
                audit.merge(&apply_temporal(&mut manipulated, condition, seed * 11).unwrap());
            }
            assert!(
                audit.mean_bin_displacement > previous,
                "w{window} displaced {:.3}, no further than w/2's {previous:.3}",
                audit.mean_bin_displacement
            );
            previous = audit.mean_bin_displacement;
        }
    }

    /// Dropout deletes spikes at the registered rate and moves nothing.
    ///
    /// The retention assertion is the operator's own registered band, not a
    /// hand-picked tolerance — the same code path a campaign run is gated on.
    #[test]
    fn dropout_deletes_at_the_registered_rate_and_relocates_nothing() {
        for percent in [5_u32, 10, 25, 50, 90] {
            let condition = TemporalCondition::spike_dropout(percent).unwrap();
            let mut audit = TemporalAudit::default();
            for seed in 1_u64..=40 {
                let mut manipulated = dense_sample(seed % 5);
                audit.merge(&apply_temporal(&mut manipulated, condition, seed * 13).unwrap());
            }
            assert_eq!(audit.relocated_fraction, 0.0, "p{percent} relocated entries");
            assert_eq!(audit.max_bin_displacement, 0.0, "p{percent} moved an entry");
            assert!(
                !audit.counts_preserved,
                "p{percent} preserved counts - it deleted nothing"
            );
            condition
                .invariants()
                .check(&condition.label(), &audit)
                .unwrap_or_else(|error| panic!("p{percent}: {error}"));
        }
    }

    /// The mask is a pure function of the seed and the sample.
    ///
    /// # What this is really pinning
    ///
    /// Redrawing dropout every epoch turns an information-removal control into
    /// noise augmentation, which is a *regulariser* on SHD — it would lift both
    /// arms and the cell would still look exactly like a dropout cell. Nothing
    /// downstream could tell the two apart from the record.
    ///
    /// The structural defence is that `apply_temporal` takes a seed and is
    /// called once per sample before training; this asserts the consequence,
    /// that the same seed reproduces the same realisation exactly, and that a
    /// different sample index does not.
    #[test]
    fn dropout_is_a_frozen_mask_not_per_epoch_noise() {
        let condition = TemporalCondition::spike_dropout(30).unwrap();
        let mut first = dense_sample(2);
        let mut second = dense_sample(2);
        apply_temporal(&mut first, condition, 0xABCD).unwrap();
        apply_temporal(&mut second, condition, 0xABCD).unwrap();
        assert_eq!(
            fingerprint(&first),
            fingerprint(&second),
            "the same seed must reproduce the same mask exactly"
        );

        let mut other_index = dense_sample(2);
        apply_temporal(&mut other_index, condition, 0xABCE).unwrap();
        assert_ne!(
            fingerprint(&first),
            fingerprint(&other_index),
            "two samples in one split must not share a mask"
        );
    }

    /// Dropout draws from its own stream and cannot perturb initialisation.
    ///
    /// Seed-paired estimators are the campaign's whole statistical basis: an
    /// `attn` cell and its `rate` twin at seed *s* must start from the same
    /// weights, and a `spike-dropout` cell must start from the same weights as
    /// its `intact` twin. If the manipulation consumed draws from the stream
    /// that initialises weights, every pairing in every wave would be against a
    /// different network and the difference-in-differences would be measuring
    /// initialisation.
    ///
    /// Structurally it cannot: `apply_temporal` constructs a local
    /// [`PortableRng`] from its own argument and there is no global stream. The
    /// assertion is here because "structurally it cannot" is what this campaign
    /// keeps discovering was untrue.
    #[test]
    fn no_manipulation_perturbs_the_initialisation_stream() {
        // Exactly the three streams `shd_instrument init` draws from, in the
        // order it draws them: the base weights, the attention read-out, and
        // the per-epoch presentation orders.
        let lineage = || {
            let base = MatchedWeights::deterministic(20, 17, 8, 0x5EED_0001);
            let attn = AttentionParams::deterministic(
                17,
                8,
                AttentionConfig::DEFAULT,
                0x5EED_0001 ^ 0x4154_544E_0000_0000,
            )
            .unwrap();
            let mut rng = PortableRng::new(0x5EED_0001 ^ 0x0D3E_45E5_51D0_0001);
            let mut order: Vec<usize> = (0..64).collect();
            rng.shuffle(&mut order);
            (base, attn, order)
        };
        let reference = lineage();
        let conditions = [
            TemporalCondition::BinShuffled,
            TemporalCondition::ChannelShuffled,
            TemporalCondition::Reversed,
            TemporalCondition::WindowShuffled { window: 8 },
            TemporalCondition::SpikeDropout { percent: 30 },
        ];
        for condition in conditions {
            let mut manipulated = dense_sample(1);
            apply_temporal(&mut manipulated, condition, 0x5EED_0001).unwrap();
            let after = lineage();
            assert!(
                reference.0 == after.0 && reference.1 == after.1 && reference.2 == after.2,
                "{} moved the initialisation stream",
                condition.label()
            );
        }
    }

    /// Every operator satisfies its own invariants and violates its neighbours'.
    ///
    /// The second half is the load-bearing one. A registry whose entries all
    /// accept every operator's audit is the global gate again with more
    /// ceremony, and it would pass this file's first half unchanged.
    #[test]
    fn each_operator_is_separated_by_its_own_invariants() {
        let conditions = [
            TemporalCondition::BinShuffled,
            TemporalCondition::WindowShuffled { window: 4 },
            TemporalCondition::SpikeDropout { percent: 40 },
        ];
        let mut audits = Vec::new();
        for condition in conditions {
            let mut audit = TemporalAudit::default();
            for seed in 1_u64..=12 {
                let mut manipulated = dense_sample(seed % 5);
                audit.merge(&apply_temporal(&mut manipulated, condition, seed * 17).unwrap());
            }
            audits.push(audit);
        }
        for (own, condition) in conditions.iter().enumerate() {
            condition
                .invariants()
                .check(&condition.label(), &audits[own])
                .unwrap_or_else(|error| panic!("{} rejected its own audit: {error}", condition.label()));
            for (other, audit) in audits.iter().enumerate() {
                if other == own {
                    continue;
                }
                assert!(
                    condition.invariants().check(&condition.label(), audit).is_err(),
                    "{}'s invariants accepted {}'s audit",
                    condition.label(),
                    conditions[other].label(),
                );
            }
        }
    }

    /// A dropout mask applied twice retains `(1-p)^2` and must be rejected.
    ///
    /// This is the concrete accident the binomial band exists for: a caller
    /// that manipulates train and test in one loop and then again in another,
    /// or a resumed run that re-applies. At p=30 the doubled retention is 0.49
    /// against a registered 0.70, which is thousands of sigmas out.
    #[test]
    fn a_mask_applied_twice_is_rejected_by_the_retention_band() {
        let condition = TemporalCondition::spike_dropout(30).unwrap();
        let mut audit = TemporalAudit::default();
        for seed in 1_u64..=12 {
            let mut manipulated = dense_sample(seed % 5);
            let before = apply_temporal(&mut manipulated, condition, seed * 19).unwrap();
            let mut second = apply_temporal(&mut manipulated, condition, seed * 23).unwrap();
            // The second pass sees the already-thinned sample, so its own
            // `count_before` is the first pass's `count_after`. Splice the two
            // into the audit a caller would actually accumulate.
            second.count_before = before.count_before;
            audit.merge(&second);
        }
        let error = condition
            .invariants()
            .check(&condition.label(), &audit)
            .expect_err("a doubled mask must not pass its own retention band");
        assert!(error.contains("retained"), "{error}");
    }

    /// An operator that ran on nothing must not report a clean audit.
    ///
    /// `TemporalAudit::default()` is deliberately the identity of the merge
    /// fold — `counts_preserved: true`, every fraction zero — so a caller that
    /// never invoked the manipulation holds a value that looks like a passing
    /// one. The band on `relocated_fraction` catches that for the shuffles by
    /// accident; the explicit zero-sample clause catches it for every operator,
    /// including dropout, whose registered relocated fraction *is* zero.
    #[test]
    fn a_manipulation_that_never_ran_is_not_a_passing_audit() {
        for condition in [
            TemporalCondition::BinShuffled,
            TemporalCondition::WindowShuffled { window: 8 },
            TemporalCondition::SpikeDropout { percent: 30 },
        ] {
            let error = condition
                .invariants()
                .check(&condition.label(), &TemporalAudit::default())
                .expect_err("an empty audit must not pass");
            assert!(error.contains("zero samples"), "{error}");
        }
    }

    /// Dropout refuses a fractional count rather than truncating it.
    #[test]
    fn dropout_refuses_non_integral_counts() {
        let mut sample = dense_sample(1);
        sample.frames[3].push((7, 2.5));
        sample.frames[3].sort_by_key(|&(channel, _)| channel);
        let error = apply_temporal(
            &mut sample,
            TemporalCondition::SpikeDropout { percent: 20 },
            9,
        )
        .expect_err("2.5 spikes is not a number of spikes");
        assert!(error.contains("integral"), "{error}");
    }

    /// The claim that makes `hidden-shuffled` a control rather than a second
    /// shuffle: a rate arm's forward and backward are **bit-identical** under it.
    ///
    /// Not "close", not "within tolerance". The campaign's estimator is a
    /// difference of differences read against a ±0.03 bar, and the design says
    /// this particular difference is exactly zero. If it were merely small, a
    /// reader could not tell a real rate-arm effect from accumulated rounding,
    /// and the wave's own positive control would be the thing generating the
    /// signal.
    ///
    /// It holds by construction — `rates` is accumulated from the unpermuted
    /// buffer and the spiking loop never sees the permutation — and this is the
    /// assertion that keeps it holding.
    #[test]
    fn a_hidden_shuffle_costs_the_rate_read_out_exactly_nothing() {
        for arm in MatchedArm::ALL {
            let (mut sample, weights) = permuted_fixture(arm, false);
            let intact = sample.clone();
            let mut rng = PortableRng::new(0xFEED_0007);
            let mut permutation: Vec<usize> = (0..sample.frames.len()).collect();
            rng.shuffle(&mut permutation);
            sample.hidden_time_permutation = Some(permutation);

            let (intact_forward, intact_gradient) =
                loss_and_gradient_arm(&weights, &intact).unwrap();
            let (shuffled_forward, shuffled_gradient) =
                loss_and_gradient_arm(&weights, &sample).unwrap();

            assert_eq!(
                intact_forward.loss.to_bits(),
                shuffled_forward.loss.to_bits(),
                "{} loss moved under hidden-shuffled",
                arm.label()
            );
            assert_eq!(
                intact_forward.logits, shuffled_forward.logits,
                "{} logits moved under hidden-shuffled",
                arm.label()
            );
            assert_eq!(
                intact_gradient, shuffled_gradient,
                "{} gradient moved under hidden-shuffled",
                arm.label()
            );
        }
    }

    /// And it costs an attention arm something, or the control is vacuous.
    #[test]
    fn a_hidden_shuffle_moves_every_attention_arm() {
        for arm in MatchedArm::ALL_ATTENTION {
            let (mut sample, weights) = permuted_fixture(arm, true);
            let intact = sample.clone();
            let mut rng = PortableRng::new(0xFEED_0009);
            let mut permutation: Vec<usize> = (0..sample.frames.len()).collect();
            rng.shuffle(&mut permutation);
            sample.hidden_time_permutation = Some(permutation);

            let (intact_forward, _) = loss_and_gradient_arm(&weights, &intact).unwrap();
            let (shuffled_forward, _) = loss_and_gradient_arm(&weights, &sample).unwrap();
            let moved: f32 = intact_forward
                .logits
                .iter()
                .zip(&shuffled_forward.logits)
                .map(|(a, b)| (a - b).abs())
                .sum();
            assert!(
                moved > 1e-4,
                "{} did not notice the permutation (moved {moved:e})",
                arm.label()
            );
        }
    }

    /// A fixture whose spiking is dense enough that a permutation has something
    /// to permute, on every arm.
    fn permuted_fixture(arm: MatchedArm, attentive: bool) -> (MatchedShdSample, ArmWeights) {
        let (n_inputs, hidden, n_classes, t_steps) = (24_usize, 12_usize, 5_usize, 17_usize);
        let frames: Vec<Vec<(usize, f32)>> = (0..t_steps)
            .map(|t| {
                (0..n_inputs)
                    .filter(|channel| (t * 5 + channel * 3) % 4 < 2)
                    .map(|channel| (channel, ((t + channel) % 3 + 1) as f32))
                    .collect()
            })
            .collect();
        let sample = MatchedShdSample::new(2, frames, n_inputs, 2.0);
        let base = MatchedWeights::deterministic(n_inputs, hidden, n_classes, 0x51E6_0001);
        let w_rec = if arm.recurrent {
            let mut rng = PortableRng::new(0x51E6_0002);
            (0..hidden * hidden).map(|_| rng.uniform(-0.1, 0.1)).collect()
        } else {
            Vec::new()
        };
        let weights = if attentive {
            let mut attn = AttentionParams::deterministic(
                hidden,
                n_classes,
                AttentionConfig::DEFAULT,
                0x51E6_0003,
            )
            .unwrap();
            // `deterministic` zeroes `w_o`, which would make the attention
            // stream identical at every position and hide the permutation
            // behind a structural zero rather than a measured one.
            let mut rng = PortableRng::new(0x51E6_0004);
            for block in attn.blocks.iter_mut() {
                for value in block.w_o.iter_mut() {
                    *value = rng.uniform(-0.3, 0.3);
                }
            }
            for value in attn.w_a.iter_mut() {
                *value = rng.uniform(-0.3, 0.3);
            }
            ArmWeights::new_attentive(base, arm, w_rec, attn).unwrap()
        } else {
            ArmWeights::new(base, arm, w_rec).unwrap()
        };
        (sample, weights)
    }

    /// `hidden-shuffled` leaves the input byte-identical and is visible only in
    /// the field that exists for it.
    #[test]
    fn hidden_shuffled_touches_no_frame_and_is_still_auditable() {
        let before = dense_sample(3);
        let mut after = dense_sample(3);
        let audit =
            apply_temporal(&mut after, TemporalCondition::HiddenShuffled, 0x1234).unwrap();
        assert_eq!(before.frames, after.frames, "the input must be untouched");
        assert_eq!(audit.relocated_fraction, 0.0);
        assert_eq!(audit.count_before, audit.count_after);
        assert!(audit.counts_preserved);
        assert!(
            audit.hidden_permutation_relocated > 0.5,
            "permuted only {:.3} of timesteps",
            audit.hidden_permutation_relocated
        );
        assert!(after.hidden_time_permutation.is_some());
        TemporalCondition::HiddenShuffled
            .invariants()
            .check("hidden-shuffled", &audit)
            .unwrap();
    }

    /// The clause that stops a `hidden-shuffled` cell whose permutation was
    /// never built from passing as one. Without it, its audit is an intact
    /// audit and every other gate agrees.
    #[test]
    fn a_hidden_shuffle_that_never_permuted_is_rejected() {
        let mut untouched = dense_sample(3);
        let intact_audit =
            apply_temporal(&mut untouched, TemporalCondition::Intact, 0x1234).unwrap();
        let error = TemporalCondition::HiddenShuffled
            .invariants()
            .check("hidden-shuffled", &intact_audit)
            .expect_err("an intact audit must not pass as hidden-shuffled");
        assert!(error.contains("read-out's timesteps"), "{error}");
    }

    /// And the converse: a read-out permutation riding along on an input
    /// operator is caught by that operator's own band.
    #[test]
    fn a_stray_read_out_permutation_is_caught_by_the_input_operators() {
        let mut sample = dense_sample(3);
        let mut audit =
            apply_temporal(&mut sample, TemporalCondition::BinShuffled, 0x1234).unwrap();
        audit.hidden_permutation_relocated = 0.97;
        let error = TemporalCondition::BinShuffled
            .invariants()
            .check("bin-shuffled", &audit)
            .expect_err("bin-shuffled must not carry a read-out permutation");
        assert!(error.contains("read-out's timesteps"), "{error}");
    }
}
