//! Camera-ready paper figures from PAPER_FIGURE_SPEC / PAPER_RESULTS_TABLE.
//!
//! Numbers are hardcoded from those sheets — never remassaged. Behind `plots`.

use std::fs;
use std::path::{Path, PathBuf};

use plotters::coord::Shift;
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};

type DrawErr = Box<dyn std::error::Error>;

/// Numbers from `results/PAPER_FIGURE_SPEC.md` / `PAPER_RESULTS_TABLE.md` only.
///
/// # The block below replaced a superseded one on 2026-08-27
///
/// This module used to carry `DFA = 0.9387`, `RL_FB = 0.9200` and
/// `GRAD_MATCH = 0.8963`, with `gap LCB 0.6894 / 0.6846` written inline in the
/// figure bodies. `PAPER_FIGURE_SPEC.md` §"Figure 6" now names that exact set
/// as **"superseded and not for drawing"**: they are pre-repair figures from a
/// forward pass that emitted zero spikes at any seed, and none of them appears
/// in `PAPER_RESULTS_TABLE.md` any longer.
///
/// The generator went on drawing them, and reproduced the committed artwork
/// byte-for-byte, so the figures on disk were the superseded block rendered at
/// camera-ready quality. Numbers here are now the **2026-08-25 re-run** at
/// `MATCHED_INPUT_SCALE = 2.0`, n = 20, feed-forward / recurrent
/// (`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`), and
/// `paper_figures_match_the_spec` parses the spec's own table and fails if the
/// two ever disagree again.
mod nums {
    /// `(feed-forward, recurrent)` from the 2026-08-25 re-run.
    pub type Both = (f64, f64);

    // --- matched dense-LIF, the 2026-08-25 re-run ---------------------------
    /// `MatchedLocal` — ±1 reward × surrogate eligibility. The lead FAIL, and
    /// at chance on both graphs.
    pub const BROADCAST_PM1: Both = (0.5000, 0.5100);
    /// `MatchedRlFlat` — ±1 broadcast REINFORCE. Shares the low-richness,
    /// low-addressability cell with `BROADCAST_PM1` and disagrees with it by
    /// 0.28, which is why the cell must be drawn as two rules.
    pub const RL_FLAT: Both = (0.7775, 0.7962);
    /// `MatchedRlReinforceFb` — REINFORCE × frozen `B_i`.
    pub const RL_FB: Both = (0.9950, 0.9812);
    /// `MatchedDfaGradedError` — graded error × DFA.
    pub const DFA: Both = (0.9925, 0.9875);
    /// `MatchedBroadcastGradedError` — the contrast the honesty note is about.
    pub const BROADCAST_GRADED: Both = (0.9975, 0.9975);
    /// `MatchedEventProp` — discrete EventProp-style spike-adjoint. **PASS on
    /// both graphs, and the archived FAIL is WITHDRAWN**: at
    /// `MATCHED_INPUT_SCALE = 0.5` a spike-adjoint method had no spikes to
    /// differentiate through, and the 0.5000 that produced was a property of
    /// the forward rather than of the rule.
    pub const EVENTPROP: Both = (0.9450, 0.8900);
    /// `MatchedRlGraded` — RL graded-reward broadcast. Measured, not gated.
    pub const RL_GRADED: Both = (0.8787, 0.9100);
    /// SuperSpike BPTT. Saturates in every suite on both graphs, which is what
    /// makes the passing arms unrankable: every one of them reduces to
    /// "above 0.75" against a reference at 1.
    pub const CEILING: Both = (1.0000, 1.0000);

    /// Figure 6 Panel B — the gate quantity, `(arm, ff, rec)`, Table A.
    ///
    /// Not `Both`, deliberately: the broadcast row is **negative on the
    /// recurrent graph** and a `Both` here would be silently excluded from the
    /// spec-parity check, which only reads unsigned pairs. Ban 3 requires that
    /// value be drawn rather than clipped at zero, so it gets a shape that
    /// carries a sign.
    ///
    /// Row order is Table A's, not sorted: ban 1 forbids an ordering among the
    /// passing arms and a sorted second panel would supply one.
    pub const GAP_LCB: [(&str, f64, f64); 4] = [
        ("broadcast ±1 three-factor", 0.0000, -0.0192),
        ("graded DFA", 0.9689, 0.9509),
        ("REINFORCE × frozen B_i", 0.9765, 0.9079),
        ("discrete EventProp spike-adjoint", 0.7911, 0.6494),
    ];
    /// The gate: primary mean ≥ `GATE_FLOOR` **and** gap LCB > `GATE_LCB`.
    /// Both are drawn, on both figures that report against them, because
    /// "cleared the floor" and "cleared the gate" are different sentences and
    /// this package keeps them apart everywhere else.
    pub const GATE_FLOOR: f64 = 0.65;
    pub const GATE_LCB: f64 = 0.5;

    // --- Figure 8: the transfer ladder (Table A rung 1, Table C rungs 2-4) ---
    //
    // Rung 1 is the matched dense-LIF forward. Rungs 2-4 are the live
    // event-driven muted-θ / k-WTA engine. They are NOT one system at four
    // settings, and ban 1 turns on that.

    /// `(protocol, local, gap LCB)` in **protocol order**, from Table C.
    ///
    /// v14-v19, the gap-close family. Sorting these would assert a ranking over
    /// a sequential exploratory family that has no multiplicity-corrected
    /// claim, which is ban 3.
    pub const GAP_CLOSE: [(&str, f64, f64); 6] = [
        ("v14 epoch", 0.4838, -0.0100),
        ("v15 structured B", 0.7262, 0.2567),
        ("v16 structured × epoch", 0.5200, 0.0844),
        ("v17 structured × capacity", 0.6825, 0.3127),
        ("v18 elig × REINFORCE", 0.7125, 0.2351),
        ("v19 structured × teach", 0.6700, 0.2238),
    ];
    /// v20-v24, the break-it family, protocol order, from Table C.
    pub const BREAK_IT: [(&str, f64, f64); 5] = [
        ("v20 live DFA", 0.7325, 0.2601),
        ("v21 soft-WTA × SFB", 0.5025, 0.0406),
        ("v22 match 4× epochs", 0.5000, 0.0000),
        ("v23 finite-θ SFB", 0.6638, 0.2370),
        ("v24 continuous B", 0.6437, 0.1380),
    ];
    /// Rung 2, the arm the ladder descends to: v13 live RFB.
    pub const LIVE_RFB_LCB: f64 = 0.0737;
    /// The two landmarks the spec names, and the only two. v15 carries the best
    /// local below rung 1 among the gap-close family; v17 the best gap LCB
    /// anywhere below rung 1 — and 0.3127 against a bar of 0.5 is not close.
    pub const BEST_LOCAL_GAP_CLOSE: f64 = 0.7262;
    pub const BEST_LCB_ANYWHERE: f64 = 0.3127;

    // --- LEAD PROGRAM: the SHD read-out, Table SHD-2 -------------------------
    //
    // The manuscript leads with these and, until 2026-08-27, nothing had been
    // drawn for any of the four lead figures. Note what is NOT here: absolute
    // bin-shuffled means at n = 32. Table SHD-2 prints `—` for them, so only
    // the two costs exist at that sample size and only the costs may be
    // plotted. The absolute intact→shuffled pair exists at n = 12 alone.

    /// Each arm's own shuffle cost, n = 32. **This pair is the figure.**
    pub const SHUFFLE_COST_ATTN_32: f64 = 0.1347;
    pub const SHUFFLE_COST_RATE_32: f64 = 0.0142;
    /// Not `+0.1577`. The wave-17 analyser merged a `d32l1` archived shuffled
    /// control into the `d32l4` comparison for twelve pairs and inflated the
    /// cost by 17%. The verdict was MET either way; the corrected value is the
    /// only one that may be drawn.
    pub const SHUFFLE_COST_RATIO_32: f64 = 9.5;

    /// The same result read as the collapse of the advantage, n = 32.
    pub const ADVANTAGE_INTACT_32: f64 = 0.1275;
    pub const ADVANTAGE_SHUFFLED_32: f64 = 0.0070;
    pub const CONTINGENT_PCT_32: f64 = 94.5;

    /// n = 12, the registration this confirms. Drawn beside n = 32 because the
    /// near-identity is the message: twenty further seeds move the gain by
    /// +0.0017 and the cost by +0.0010. Nothing was rescued by the larger run.
    pub const ADVANTAGE_INTACT_12: f64 = 0.1258;
    pub const ADVANTAGE_SHUFFLED_12: f64 = 0.0050;
    pub const CONTINGENT_PCT_12: f64 = 96.0;

    /// n = 12 absolute accuracy — the only sample size at which the intact and
    /// shuffled means both exist. Drawn small and labelled, because an
    /// accuracy-under-shuffle encoding at the centre of the figure would
    /// reproduce prior art and lose the new result.
    pub const ABS_ATTN_INTACT_12: f64 = 0.8320;
    pub const ABS_ATTN_SHUFFLED_12: f64 = 0.6983;
    pub const ABS_RATE_INTACT_12: f64 = 0.7062;
    pub const ABS_RATE_SHUFFLED_12: f64 = 0.6934;
    pub const SHUFFLE_COST_ATTN_12: f64 = 0.1337;
    pub const SHUFFLE_COST_RATE_12: f64 = 0.0128;
    /// Per seed at n = 12 the effect falls in this interval: no seed in which
    /// it is absent.
    pub const PER_SEED_MIN_12: f64 = 0.0967;
    pub const PER_SEED_MAX_12: f64 = 0.1568;

    // --- Figure 2: headline accuracy and the 0.80 clearance (Table SHD-1) ----
    /// n = 32 confirmation and the n = 12 registration. Neither supersedes the
    /// other and the sheet says so, so both are drawn.
    pub const HEAD_RATE_32: f64 = 0.7057;
    pub const HEAD_ATTN_32: f64 = 0.8332;
    pub const HEAD_GAIN_32: f64 = 0.1275;
    pub const HEAD_RATE_12: f64 = 0.7062;
    pub const HEAD_ATTN_12: f64 = 0.8320;
    pub const HEAD_GAIN_12: f64 = 0.1258;
    /// Budget stability: |e400 − e200|.
    pub const HEAD_BUDGET_DELTA: f64 = 0.0002;

    /// The published field, from `PAPER_DRAFT.md` §0. **Not machine-checked**
    /// against cells, unlike every other number in this module: assembled by a
    /// 2026-08-27 search pass, and `check_every_number.py` does not sweep §0.
    /// The figure says so on its face.
    pub const FIELD_FRONTIER_LO: f64 = 0.95;
    pub const FIELD_FRONTIER_HI: f64 = 0.964;
    pub const FIELD_STSC_SNN: f64 = 0.9236;
    pub const FIELD_TA_SNN: f64 = 0.9108;
    /// The anchor: the dataset authors' own no-delay recurrent baseline, 1024
    /// neurons, with augmentation.
    pub const FIELD_NO_DELAY: f64 = 0.832;
    pub const FIELD_NO_DELAY_SD: f64 = 0.013;
    /// SHD ships no validation set. Baronig et al. 2025 report the same model
    /// at 95.81 validating on test and 93.79 on a held-out split, so
    /// differences below about this much between published numbers are not
    /// reliably meaningful — including this paper's own.
    pub const FIELD_UNRESOLVABLE: f64 = 0.015;

    /// Figure 2 Panel C, the geometry ladder (Table SHD-5). The 0.80 clearance
    /// is geometry-specific; the gain is not.
    pub const GEOM: [(&str, f64, f64, f64, &str); 3] = [
        ("adjacent-sum-5 / published-2ms (anchor)", 0.7062, 0.8320, 0.1258, "12/12"),
        ("published-10ms", 0.6734, 0.8225, 0.1491, "10/12"),
        ("channels-700 (standard 700-channel input)", 0.6774, 0.7864, 0.1090, "6/12"),
    ];

    // --- Figure 3: the width ladder (Table SHD-3, Table SHD-4) ---------------
    /// `(width, rate, attention, gain, positive)`, n = 12 per rung.
    pub const LADDER: [(&str, f64, f64, f64, &str); 6] = [
        ("h128", 0.7062, 0.8320, 0.1258, "12/12"),
        ("h256", 0.7240, 0.8206, 0.0966, "12/12"),
        ("h384", 0.7336, 0.8096, 0.0760, "12/12"),
        ("h512", 0.7357, 0.8233, 0.0876, "12/12"),
        ("h768", 0.7386, 0.7946, 0.0560, "11/12"),
        ("h1024", 0.7386, 0.5768, -0.1618, "1/12"),
    ];
    /// Wave 21's difference-in-differences across the design space, in LADDER
    /// order then the two alternative binnings. Figure 1 Panel D.
    /// `(point, DiD, positive, quadruples, gain_is_negative)`.
    ///
    /// The last field exists because a DiD-only column at h1024 reads as "the
    /// mechanism is healthy at every width", and that point is the one place
    /// where the read-out consumes temporal order while HARMING accuracy.
    /// `PAPER_FIGURE_SPEC.md` Figure 1 Panel D ban 3 requires it be marked.
    pub const DID_LADDER: [(&str, f64, u32, u32, bool); 8] = [
        ("h128", 0.1205, 32, 32, false),
        ("h256", 0.0862, 12, 12, false),
        ("h384", 0.0767, 12, 12, false),
        ("h512", 0.0968, 12, 12, false),
        ("h768", 0.1881, 12, 12, false),
        ("h1024", 0.1122, 10, 12, true),
        ("h128 / channels-700", 0.1122, 12, 12, false),
        ("h128 / published-10ms", 0.0959, 12, 12, false),
    ];
    /// The registered bar every point above clears.
    pub const DID_BAR: f64 = 0.03;
    /// H21-3: Spearman rho between the six per-width gains and their DiDs,
    /// against the n=6 one-tailed critical value at alpha=0.05. NOT MET, and
    /// Panel D ban 2 and Figure 3 ban 6 both exist because of it.
    pub const DID_RHO: f64 = -0.1430;
    pub const DID_RHO_BAR: f64 = 0.829;
    /// Coverage as MEASURED. Ban 4: the caption says 9 of 21, never "every
    /// width" -- twelve operating points still carry no bin-shuffled twin.
    pub const COVERAGE_COVERED: u32 = 9;
    pub const COVERAGE_TOTAL: u32 = 21;
    /// h768 carries the smallest positive gain on LADDER and the largest DiD
    /// in the campaign. It is the single clearest refutation of H21-3 and
    /// Figure 3's required annotation names it.
    pub const DID_H768: f64 = 0.1881;
    pub const GAIN_H768: f64 = 0.0560;

    /// The step into h1024, and the largest gap below it. 6.9x, against a
    /// registered 3x bar of 0.0947.
    pub const LADDER_DROP: f64 = 0.2178;
    pub const LADDER_LARGEST_GAP_BELOW: f64 = 0.0316;
    pub const LADDER_DROP_RATIO: f64 = 6.9;
    /// h384 − h512, seed-paired. H16-1 NOT MET: the two rungs are **not
    /// distinguishable** at twelve seeds, so no connector may assert an order.
    pub const LADDER_H384_H512: f64 = -0.0116;
    pub const LADDER_H384_H512_SD: f64 = 0.0253;

    /// The three preregistered rescue levers at h1024/d32/L4. Every one is
    /// negative and worse than the arm it was meant to rescue.
    /// `(lever, gain, positive, median epoch-mean gradient norm)`.
    pub const LEVERS: [(&str, f64, &str, f64); 4] = [
        ("surrogate scale 0.5", -0.2106, "0/12", 142.009),
        ("surrogate scale 0.25", -0.2565, "0/12", 151.391),
        ("clip-grad-norm 1000.0", -0.0904, "1/12", 11.660),
        ("(the unclipped arm they were to rescue)", -0.1618, "1/12", 55.494),
    ];

    // --- Figure 4: the resolution ladder (Table SHD-6) -----------------------
    /// `(contract, bin, rate, attention, gain)`. The 1400 ms analysis window is
    /// held FIXED across all three; only the frame count varies. That is the
    /// whole reason the axis means anything, and it is stated on the figure.
    pub const RESOLUTION: [(&str, &str, f64, f64, f64); 3] = [
        ("fixed-t100", "14.0 ms", 0.6672, 0.8599, 0.1927),
        ("fixed-t250", "5.6 ms", 0.6844, 0.8594, 0.1751),
        ("fixed-t500", "2.8 ms", 0.7069, 0.8543, 0.1474),
    ];
    /// gain(t500) − gain(t100), against a two-sided bar of 0.03.
    pub const RESOLUTION_TREND: f64 = -0.0453;
    /// The rate arm rises across the same ladder, inside a 0.05 confound bar.
    /// Drawn, because omitting it lets the falling gain be misread as the
    /// attention arm degrading rather than the rate arm catching up.
    pub const RESOLUTION_BASELINE_DRIFT: f64 = 0.0397;

    // --- Figure S: the substrate panel (Table SHD-7) -------------------------
    //
    // The anchor campaign's 720 cells all sat on ONE substrate, so its gain had
    // two readings it could not separate: the read-out ADDS temporal structure
    // no such substrate represents, or it SUBSTITUTES for the adaptation and
    // recurrence `ff+fixed` happens not to have. Three waves settle it.

    /// `(substrate, note, surrogate scale, pairs, rate, attention, gain)`.
    ///
    /// Row order is Table SHD-7's, which is also the order the waves ran in.
    /// Ban 1 forbids sorting by gain: that would put `rec+alif` last as a
    /// climax and assert the ordering the record explicitly refuses.
    pub const SUBSTRATE: [(&str, &str, f64, u32, f64, f64, f64); 4] = [
        ("ff+fixed", "the anchor", 1.0, 12, 0.7062, 0.8320, 0.1258),
        ("ff+alif", "threshold adaptation", 1.0, 12, 0.7018, 0.8303, 0.1285),
        ("rec+alif", "recurrent + adaptation", 0.4, 10, 0.5262, 0.7874, 0.2612),
        ("ff+fixed", "scale-matched control", 0.4, 12, 0.7088, 0.8289, 0.1201),
    ];
    /// A-1: gain(`ff+alif`) − gain(`ff+fixed`), against a two-sided 0.03 bar.
    /// Positive in 6 of 12 — a coin flip, and inert AT THIS OPERATING POINT
    /// only, which is ban 4.
    pub const SUBSTRATE_A1: f64 = 0.0027;
    /// M-2: gain(`rec+alif`) − gain(`ff+fixed`) at matched scale 0.4.
    pub const SUBSTRATE_M2: f64 = 0.1411;
    /// M-4: `ff+fixed` at scale 0.4 against the same arm archived at 1.0. The
    /// scale is not doing the work.
    pub const SUBSTRATE_M4: f64 = 0.0026;
    pub const SUBSTRATE_BAR: f64 = 0.03;
    /// Ban 2. `rec+alif` starts 0.18 lower and has this much room to recover;
    /// gain/headroom is 0.551 against 0.412 and the ratio falls 2.2x -> 1.34x.
    /// **Post-hoc and not registered**, and the figure says so — it is not a
    /// licence to prefer whichever number is more convenient in either
    /// direction, so both go on.
    pub const HEADROOM_REC: f64 = 0.4738;
    pub const HEADROOM_FF: f64 = 0.2912;
    pub const HEADROOM_RATIO_REC: f64 = 0.551;
    pub const HEADROOM_RATIO_FF: f64 = 0.412;
    pub const RAW_RATIO: f64 = 2.2;
    pub const NORMALISED_RATIO: f64 = 1.34;

    /// Panel C: `(arm, scale, completed, voided, diverged)` of 12.
    ///
    /// This had to be measured before any of Panel A could be, and dropping it
    /// would turn A-1's scoped null into a general one: `rec+fixed`'s ten voids
    /// are all SATURATION, so on the recurrent substrate adaptation is
    /// stabilising — the opposite of the sign that wave's hypothesis asserted.
    pub const USABILITY: [(&str, f64, u32, u32, u32); 4] = [
        ("rec+alif", 0.4, 11, 0, 1),
        ("rec+alif", 1.0, 8, 0, 4),
        ("rec+fixed", 0.4, 7, 5, 0),
        ("rec+fixed", 1.0, 5, 5, 2),
    ];
    pub const SATURATED_FRACTION_MAX: f64 = 0.523;

    // --- XOR locality flip, 1-layer `xor_thresh` (Table D) -------------------
    pub const XOR_BCAST: f64 = 0.5008;
    pub const XOR_DFA: f64 = 0.8267;
    pub const XOR_GRAD: f64 = 0.7733;

    // --- engine C1 / Gate G2, hash c1-118207fbc3eaba53 (Figure 7) ------------
    // Unaffected by the matched re-run: none of these runs on the matched
    // dense-LIF forward.
    pub const C1_LOCAL: f64 = 0.4912;
    pub const C1_DENSE: f64 = 0.5000;
    pub const C1_GRAD: f64 = 0.8938;
    pub const C1_ELIG: f64 = 1.0000;
    pub const C1_PC: f64 = 0.9488;
    pub const LIVE_RFB: f64 = 0.4900;
}

/// How a cell is encoded. Accuracy is never mapped to a size or a ramp in
/// Panel A: `PAPER_FIGURE_SPEC.md` requires pass / fail / at-chance, because
/// with the reference at 1.0000 every pass reduces to "above 0.75" and a
/// gradient would manufacture an ordering the task cannot support.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Verdict {
    Pass,
    /// Gated and failed. Drawn identically whether the arm is at chance or not;
    /// the accuracy is printed, not encoded.
    Fail,
    /// Measured and reported, but not a gated arm — so it is neither a pass to
    /// be counted nor a failure to be explained away.
    Contrast,
    Reference,
}

impl Verdict {
    fn tag(self) -> &'static str {
        match self {
            Verdict::Pass => "PASS",
            Verdict::Fail => "FAIL",
            Verdict::Contrast => "contrast (not gated)",
            Verdict::Reference => "reference",
        }
    }

    fn border(self) -> RGBColor {
        match self {
            Verdict::Pass => RGBColor(90, 170, 110),
            Verdict::Fail => RGBColor(220, 90, 90),
            Verdict::Contrast => RGBColor(200, 160, 60),
            Verdict::Reference => RGBColor(90, 120, 200),
        }
    }

    fn fill(self) -> RGBColor {
        match self {
            Verdict::Pass => RGBColor(233, 248, 237),
            Verdict::Fail => RGBColor(253, 234, 234),
            Verdict::Contrast => RGBColor(253, 246, 224),
            Verdict::Reference => RGBColor(235, 240, 253),
        }
    }
}

/// One verdict group of Figure 6: the verdict, its heading, and its arms as
/// `(rule, mechanism, ff/rec)`.
///
/// Figure 6 groups by verdict rather than listing arms flat, because the spec
/// forbids any encoding that orders the passing arms and grouping is the
/// categorical alternative it names.
type VerdictGroup<'a> = (Verdict, &'a str, &'a [(&'a str, &'a str, nums::Both)]);

fn both(v: nums::Both) -> String {
    format!("{:.4} ff  /  {:.4} rec", v.0, v.1)
}

const W: u32 = 1400;
const H: u32 = 900;
/// Figure 1 carries a fourth panel -- wave 21's eight-point DiD ladder -- and
/// Panels A-C already reach y=872 of the standard 900. Rather than compress
/// three finished panels to fit a fourth, this figure alone is taller.
const H_LEAD1: u32 = 1380;
/// Figure 3 gained wave 21's two-line "the mechanism does not track this curve"
/// annotation, which pushed Panel B and its computed base down by 60px. At the
/// standard 900 the last line was drawn at y=900 and was invisible.
const H_LEAD3: u32 = 980;
/// Figure 6 groups eight arms by verdict and then draws the gap-LCB half of the
/// gate beneath them. Grouping is the requirement -- the spec bans any encoding
/// that orders the passing arms -- and eight cards plus four group headings plus
/// a second panel do not fit 900.
const H_FIG6: u32 = 1390;
/// Figure S stacks three panels: four substrate rows on a shared accuracy axis,
/// three registered contrasts against their bar, and the usability table that
/// had to be measured before any of Panel A could be. Panel C is a ban, not a
/// footnote, so it is not the panel that gets compressed to fit 900.
const H_FIG_S: u32 = 1000;
/// Figure 8 carries twelve live-engine rows under a substrate break that must
/// not be compressed away: the break is the figure's first ban, and a divider
/// squeezed to fit reads as a section heading rather than as a change of system.
const H_FIG8: u32 = 1020;

/// Generate required camera-ready figures into `out_dir`.
pub fn generate_all(out_dir: &Path) -> Result<Vec<PathBuf>, DrawErr> {
    fs::create_dir_all(out_dir)?;
    let mut written = Vec::new();
    // Lead program first: it is what the manuscript leads with, and until
    // 2026-08-27 nothing had been drawn for any of its four figures.
    written.extend(write_pair_sized(
        out_dir,
        "leadfig1_the_conditional",
        draw_lead_fig1,
        H_LEAD1,
    )?);
    written.extend(write_pair(out_dir, "leadfig2_headline_accuracy", draw_lead_fig2)?);
    written.extend(write_pair_sized(
        out_dir,
        "leadfig3_width_ladder",
        draw_lead_fig3,
        H_LEAD3,
    )?);
    written.extend(write_pair(out_dir, "leadfig4_resolution_ladder", draw_lead_fig4)?);
    // Deliberately NOT `graphical_abstract`: that file exists and depicts the
    // SECONDARY program. This one was `TODO(source needed)` until 2026-08-29 --
    // an authoring task rather than a missing number, since every value it
    // draws was already published.
    // Figure S is lettered, beside Figure M: it belongs to the LEAD program and
    // a fifth lead figure would renumber the secondary program 5-9 -> 6-10 one
    // day after the 2026-08-27 renumber, for one figure.
    written.extend(write_pair_sized(
        out_dir,
        "figS_substrate",
        draw_fig_s_substrate,
        H_FIG_S,
    )?);
    written.extend(write_pair(
        out_dir,
        "lead_graphical_abstract",
        draw_lead_graphical_abstract,
    )?);
    written.extend(write_pair(
        out_dir,
        "figM_mechanism_richness_addressability",
        draw_fig_m,
    )?);
    written.extend(write_pair(out_dir, "fig1_matched_rule_swap", draw_fig1)?);
    // Figures 6 and 8. Both files existed on disk from 24 July with NO
    // generator, so the 2026-08-27 re-run that brought the rest of the package
    // current could not touch them: fig2_ plotted the superseded value block and
    // fig4_ was stale at rung 1. Authored 2026-08-29; the `fig2_`/`fig4_` stems
    // are historical and are deliberately not renamed to `fig6_`/`fig8_`.
    written.extend(write_pair_sized(
        out_dir,
        "fig2_matched_means",
        draw_fig6_matched_means,
        H_FIG6,
    )?);
    written.extend(write_pair(out_dir, "fig3_engine_c1_means", draw_fig3)?);
    written.extend(write_pair_sized(
        out_dir,
        "fig4_transfer_ladder",
        draw_fig8_transfer_ladder,
        H_FIG8,
    )?);
    written.extend(write_pair(
        out_dir,
        "graphical_abstract",
        draw_graphical_abstract,
    )?);
    Ok(written)
}

fn write_pair(
    out_dir: &Path,
    stem: &str,
    draw: fn(&DrawingArea<SVGBackend<'_>, Shift>) -> Result<(), DrawErr>,
) -> Result<Vec<PathBuf>, DrawErr> {
    write_pair_sized(out_dir, stem, draw, H)
}

fn write_pair_sized(
    out_dir: &Path,
    stem: &str,
    draw: fn(&DrawingArea<SVGBackend<'_>, Shift>) -> Result<(), DrawErr>,
    h: u32,
) -> Result<Vec<PathBuf>, DrawErr> {
    let png = out_dir.join(format!("{stem}.png"));
    let pdf = out_dir.join(format!("{stem}.pdf"));
    let svg_path = out_dir.join(format!("{stem}.svg.tmp"));

    {
        let root = SVGBackend::new(&svg_path, (W, h)).into_drawing_area();
        root.fill(&WHITE).map_err(|e| format!("svg fill: {e}"))?;
        draw(&root)?;
        root.present().map_err(|e| format!("svg present: {e}"))?;
    }

    let svg = fs::read_to_string(&svg_path)?;
    let mut opt = usvg::Options::default();
    opt.fontdb_mut().load_system_fonts();
    let tree = usvg::Tree::from_str(&svg, &opt)?;
    let pdf_bytes = svg2pdf::to_pdf(
        &tree,
        svg2pdf::ConversionOptions::default(),
        svg2pdf::PageOptions::default(),
    )
    .map_err(|e| format!("svg2pdf: {e:?}"))?;
    fs::write(&pdf, pdf_bytes)?;

    // Rasterize SVG → PNG via resvg (same tree, system fonts loaded).
    let mut pixmap = resvg::tiny_skia::Pixmap::new(W, h).ok_or("pixmap alloc")?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );
    pixmap.save_png(&png)?;

    let _ = fs::remove_file(&svg_path);
    Ok(vec![png, pdf])
}

fn map_draw<E: std::fmt::Debug>(r: Result<(), E>) -> Result<(), DrawErr> {
    r.map_err(|e| format!("draw: {e:?}").into())
}

fn label(
    root: &DrawingArea<SVGBackend<'_>, Shift>,
    (x, y): (i32, i32),
    text: &str,
    size: i32,
    color: RGBColor,
) -> Result<(), DrawErr> {
    let style = TextStyle::from(("sans-serif", size).into_font())
        .color(&color)
        .pos(Pos::new(HPos::Left, VPos::Top));
    map_draw(root.draw(&Text::new(text.to_string(), (x, y), style)))
}

fn centered(
    root: &DrawingArea<SVGBackend<'_>, Shift>,
    (cx, cy): (i32, i32),
    text: &str,
    size: i32,
    color: RGBColor,
) -> Result<(), DrawErr> {
    let style = TextStyle::from(("sans-serif", size).into_font())
        .color(&color)
        .pos(Pos::new(HPos::Center, VPos::Center));
    map_draw(root.draw(&Text::new(text.to_string(), (cx, cy), style)))
}

fn box_card(
    root: &DrawingArea<SVGBackend<'_>, Shift>,
    (x0, y0, x1, y1): (i32, i32, i32, i32),
    fill: RGBColor,
    border: RGBColor,
    title: &str,
    body: &[&str],
) -> Result<(), DrawErr> {
    map_draw(root.draw(&Rectangle::new([(x0, y0), (x1, y1)], fill.filled())))?;
    map_draw(root.draw(&Rectangle::new(
        [(x0, y0), (x1, y1)],
        border.stroke_width(2),
    )))?;
    centered(root, ((x0 + x1) / 2, y0 + 28), title, 18, BLACK)?;
    let mut y = y0 + 58;
    for line in body {
        centered(root, ((x0 + x1) / 2, y), line, 15, BLACK)?;
        y += 22;
    }
    Ok(())
}

/// A bar row with an explicit chance line.
///
/// Without it a bar at 0.5008 on a two-class task reads as "half as good as
/// 1.0" rather than "did not learn", which is the same manufactured ordering
/// Panel A is forbidden to draw.
fn draw_bar_row(
    root: &DrawingArea<SVGBackend<'_>, Shift>,
    x0: i32,
    y0: i32,
    width: i32,
    height: i32,
    chance: Option<f64>,
    bars: &[(&str, f64, RGBColor)],
) -> Result<(), DrawErr> {
    let n = bars.len() as i32;
    let gap = 40;
    let bar_w = (width - gap * (n + 1)) / n;
    let base = y0 + height - 30;
    map_draw(root.draw(&PathElement::new(
        vec![(x0, base), (x0 + width, base)],
        BLACK.stroke_width(2),
    )))?;
    for (i, (name, val, color)) in bars.iter().enumerate() {
        let i = i as i32;
        let bx = x0 + gap + i * (bar_w + gap);
        let h = ((*val) * (height - 50) as f64) as i32;
        let top = base - h;
        map_draw(root.draw(&Rectangle::new(
            [(bx, top), (bx + bar_w, base)],
            color.filled(),
        )))?;
        centered(
            root,
            (bx + bar_w / 2, top - 16),
            &format!("{val:.4}"),
            14,
            BLACK,
        )?;
        centered(root, (bx + bar_w / 2, base + 18), name, 14, BLACK)?;
    }
    if let Some(level) = chance {
        let y = base - (level * (height - 50) as f64) as i32;
        let dash = 12;
        let mut x = x0;
        while x < x0 + width {
            map_draw(root.draw(&PathElement::new(
                vec![(x, y), ((x + dash).min(x0 + width), y)],
                RGBColor(120, 120, 120).stroke_width(2),
            )))?;
            x += dash * 2;
        }
        label(
            root,
            (x0 + width + 6, y - 8),
            &format!("chance {level:.2}"),
            12,
            RGBColor(120, 120, 120),
        )?;
    }
    Ok(())
}

/// One arm's shuffle cost, drawn on a shared axis.
///
/// Both arms get the same bar width, the same label size and the same weight.
/// `PAPER_FIGURE_SPEC.md` is explicit that the rate arm must be as visually
/// prominent as the attention arm: the pair of costs IS the measurement, and a
/// figure in which the control is a faint reference reads as "shuffling hurts
/// the attention model", which is prior art.
fn cost_bar(
    root: &DrawingArea<SVGBackend<'_>, Shift>,
    (x0, y_base, width, height): (i32, i32, i32, i32),
    scale: f64,
    cost: f64,
    color: RGBColor,
    arm: &str,
    detail: &str,
    pairs: &str,
) -> Result<(), DrawErr> {
    let h = ((cost / scale) * height as f64).round() as i32;
    let top = y_base - h;
    map_draw(root.draw(&Rectangle::new(
        [(x0, top), (x0 + width, y_base)],
        color.filled(),
    )))?;
    centered(
        root,
        (x0 + width / 2, top - 22),
        &format!("+{cost:.4}"),
        26,
        BLACK,
    )?;
    centered(root, (x0 + width / 2, y_base + 26), arm, 18, BLACK)?;
    centered(
        root,
        (x0 + width / 2, y_base + 50),
        detail,
        13,
        RGBColor(90, 90, 90),
    )?;
    centered(
        root,
        (x0 + width / 2, y_base + 72),
        pairs,
        13,
        RGBColor(90, 90, 90),
    )?;
    Ok(())
}

/// Figure 1 of the lead program — the difference-in-differences on the gain.
///
/// Four things `PAPER_FIGURE_SPEC.md` forbids, and where each is handled:
///
/// 1. It must not read as "SHD is temporal" — that is prior art. The centre of
///    the figure is the PAIR of costs, drawn at equal weight, and the banner
///    across the top states the prior work by name so the figure cannot be
///    mistaken for claiming it.
/// 2. It must not be drawn as an ablation. Bin-shuffling is applied to the
///    DATA, independently per sample, in BOTH splits — nothing is removed from
///    the model — so no panel carries a model-component axis or an
///    "attention off" label, and the operation is spelled out where it is
///    named.
/// 3. It must not quote +0.1577, the value the wave-17 analyser inflated by
///    merging two read-out depths. Only +0.1347 appears.
/// 4. It must not imply n = 32 rescued anything. Panel B draws n = 12 beside
///    n = 32 precisely so the near-identity is visible.
///
/// Panel A plots costs and not accuracies because Table SHD-2 prints `—` for
/// the absolute bin-shuffled means at n = 32: at that sample size the costs are
/// the only quantities that exist. The spec's layout line asks for "an intact →
/// bin-shuffled pair" per arm, which is drawable at n = 12 only, and that is
/// what Panel C is.
fn draw_lead_fig1(root: &DrawingArea<SVGBackend<'_>, Shift>) -> Result<(), DrawErr> {
    label(
        root,
        (36, 14),
        "Figure 1 — The conditional: a difference-in-differences on the gain",
        26,
        BLACK,
    )?;
    label(
        root,
        (36, 48),
        "Destroying temporal order costs the time-axis read-out almost everything it was buying, and costs the rate read-out almost nothing. The plotted quantity is each arm's own shuffle cost.",
        13,
        RGBColor(70, 70, 70),
    )?;

    // The prior-art banner. Without it the figure invites the reading it is
    // least entitled to.
    let prior = RGBColor(238, 238, 242);
    map_draw(root.draw(&Rectangle::new([(36, 72), (1364, 122)], prior.filled())))?;
    map_draw(root.draw(&Rectangle::new(
        [(36, 72), (1364, 122)],
        RGBColor(170, 170, 178).stroke_width(1),
    )))?;
    label(
        root,
        (48, 80),
        "NOT SHOWN HERE, AND NOT CLAIMED: that SHD depends on temporal order. That is established — Cramer et al. 2022 (≤60% on spike-count-only SHD),",
        12,
        RGBColor(60, 60, 60),
    )?;
    label(
        root,
        (48, 100),
        "the Neuromorphic Sequential Arena 2025 (86.48 → 68.51 with temporal processing removed model-side), Yu et al. 2025 (randomised spike times, reversed time).",
        12,
        RGBColor(60, 60, 60),
    )?;

    // --- Panel A ------------------------------------------------------------
    label(
        root,
        (36, 138),
        "Panel A — each arm's own shuffle cost. n = 32, seed-paired, h128 / published-2ms / adjacent-sum-5 / e400.",
        16,
        BLACK,
    )?;
    label(
        root,
        (36, 160),
        "Bin-shuffling permutes time bins independently per sample in BOTH the training and test splits, so the task itself becomes rate-solvable. Nothing is removed from the model.",
        12,
        RGBColor(110, 110, 110),
    )?;

    let base = 470;
    let axis_top = 200;
    let scale = 0.15;
    // Axis, so the 9.5x is readable off the drawing rather than only asserted.
    map_draw(root.draw(&PathElement::new(
        vec![(150, axis_top), (150, base)],
        RGBColor(120, 120, 120).stroke_width(2),
    )))?;
    for step in 0..=3 {
        let value = 0.05 * step as f64;
        let y = base - ((value / scale) * (base - axis_top) as f64).round() as i32;
        map_draw(root.draw(&PathElement::new(
            vec![(144, y), (1300, y)],
            RGBColor(224, 224, 228).stroke_width(1),
        )))?;
        label(
            root,
            (96, y - 9),
            &format!("{value:.2}"),
            12,
            RGBColor(120, 120, 120),
        )?;
    }
    map_draw(root.draw(&PathElement::new(
        vec![(150, base), (1300, base)],
        BLACK.stroke_width(2),
    )))?;

    let height = base - axis_top;
    cost_bar(
        root,
        (280, base, 300, height),
        scale,
        nums::SHUFFLE_COST_ATTN_32,
        RGBColor(70, 100, 180),
        "time-axis read-out  d32/L4",
        "ff+fixed+attn",
        "32 pairs · cost positive in 32/32",
    )?;
    cost_bar(
        root,
        (870, base, 300, height),
        scale,
        nums::SHUFFLE_COST_RATE_32,
        RGBColor(200, 140, 60),
        "rate read-out  ff+fixed",
        "the control, and half the measurement",
        "32 pairs",
    )?;
    centered(
        root,
        (725, 300),
        &format!("{:.1}×", nums::SHUFFLE_COST_RATIO_32),
        34,
        RGBColor(50, 50, 50),
    )?;
    centered(root, (725, 336), "ratio of the two costs", 13, RGBColor(90, 90, 90))?;

    // --- Panel B ------------------------------------------------------------
    label(
        root,
        (36, 566),
        "Panel B — the same result read as the collapse of the advantage over the rate arm.",
        16,
        BLACK,
    )?;
    let rows = [
        (
            "n = 32",
            nums::ADVANTAGE_INTACT_32,
            nums::ADVANTAGE_SHUFFLED_32,
            nums::CONTINGENT_PCT_32,
        ),
        (
            "n = 12  (the registration this confirms)",
            nums::ADVANTAGE_INTACT_12,
            nums::ADVANTAGE_SHUFFLED_12,
            nums::CONTINGENT_PCT_12,
        ),
    ];
    for (i, (name, intact, shuffled, pct)) in rows.into_iter().enumerate() {
        let y = 600 + (i as i32) * 52;
        label(root, (48, y), name, 14, BLACK)?;
        label(
            root,
            (390, y),
            &format!("intact  +{intact:.4}"),
            16,
            RGBColor(70, 100, 180),
        )?;
        label(root, (600, y), "→", 16, RGBColor(120, 120, 120))?;
        label(
            root,
            (650, y),
            &format!("bin-shuffled  +{shuffled:.4}"),
            16,
            RGBColor(200, 140, 60),
        )?;
        label(
            root,
            (940, y),
            // Table SHD-2 prints 96%, not 96.0%. A figure that adds a decimal
            // the sheet does not carry is quoting a number nobody computed.
            &format!(
                "{}% of the advantage is contingent on temporal order",
                if (pct - pct.round()).abs() < 1e-9 {
                    format!("{pct:.0}")
                } else {
                    format!("{pct:.1}")
                }
            ),
            14,
            BLACK,
        )?;
    }
    label(
        root,
        (48, 706),
        "Twenty further seeds move the gain by +0.0017 and the shuffle cost by +0.0010. The larger run confirms the registration; it did not rescue it.",
        12,
        RGBColor(110, 110, 110),
    )?;

    // --- Panel C ------------------------------------------------------------
    // Absolute accuracy, deliberately small and labelled. Table SHD-2 prints
    // `—` for the n = 32 shuffled means, so this panel is n = 12 or nothing.
    map_draw(root.draw(&Rectangle::new(
        [(36, 736), (1364, 872)],
        RGBColor(247, 247, 250).filled(),
    )))?;
    label(
        root,
        (48, 744),
        "Panel C — the n = 12 measurement this confirms, in absolute accuracy. LABELLED n = 12: absolute bin-shuffled means are not published at n = 32.",
        13,
        BLACK,
    )?;
    label(
        root,
        (48, 772),
        &format!(
            "time-axis read-out   {:.4} → {:.4}   cost +{:.4}, positive in 12/12",
            nums::ABS_ATTN_INTACT_12,
            nums::ABS_ATTN_SHUFFLED_12,
            nums::SHUFFLE_COST_ATTN_12
        ),
        14,
        RGBColor(70, 100, 180),
    )?;
    label(
        root,
        (48, 798),
        &format!(
            "rate read-out        {:.4} → {:.4}   cost +{:.4}",
            nums::ABS_RATE_INTACT_12,
            nums::ABS_RATE_SHUFFLED_12,
            nums::SHUFFLE_COST_RATE_12
        ),
        14,
        RGBColor(200, 140, 60),
    )?;
    label(
        root,
        (48, 828),
        &format!(
            "Per seed the effect falls between +{:.4} and +{:.4} — no seed in which it is absent. Every w9shf cell passes the temporal audit (counts preserved, relocated fraction ≥ 0.5).",
            nums::PER_SEED_MIN_12,
            nums::PER_SEED_MAX_12
        ),
        12,
        RGBColor(110, 110, 110),
    )?;
    label(
        root,
        (48, 850),
        "What is measured here is which component's contribution is the order-dependent one: a difference-in-differences on the gain, not on accuracy.",
        12,
        RGBColor(60, 60, 60),
    )?;

    // --- Panel D ------------------------------------------------------------
    //
    // Wave 21's eight-point DiD ladder. Specified in `PAPER_FIGURE_SPEC.md`
    // Figure 1 Panel D with four bans, and every one of them is a constraint
    // on how the points are ARRANGED rather than on which are drawn:
    //
    //   1. ladder order, never sorted by DiD -- sorting manufactures the trend
    //      H21-3 refuted;
    //   2. no gain on this axis and no connector between points -- rho is
    //      -0.1430 against a bar of +0.829, so a line asserts a NOT MET result;
    //   3. h1024 marked as the arm whose gain is negative;
    //   4. coverage stated as 9 of 21, never "every width".
    label(
        root,
        (36, 900),
        "Panel D — the same contrast at eight operating points. n = 12 per point, n = 32 at the anchor, seed-paired.",
        16,
        BLACK,
    )?;
    label(
        root,
        (36, 922),
        "Only the bin-shuffled halves are new; intact halves are reused from the corpus at the same seeds and the same pinned binary.",
        12,
        RGBColor(110, 110, 110),
    )?;

    let d_left = 300;
    let d_right = 1180;
    let d_top = 962;
    let d_scale = 0.20_f64;
    let x_of = |v: f64| d_left + ((v / d_scale) * (d_right - d_left) as f64).round() as i32;

    // Gridlines and the registered bar. The bar is drawn once, labelled, and
    // every point sits to the right of it -- which is the panel's whole claim.
    for step in 0..=4 {
        let value = 0.05 * step as f64;
        let x = x_of(value);
        map_draw(root.draw(&PathElement::new(
            vec![(x, d_top - 6), (x, d_top + 8 * 40)],
            RGBColor(228, 228, 232).stroke_width(1),
        )))?;
        label(
            root,
            (x - 14, d_top + 8 * 40 + 8),
            &format!("{value:.2}"),
            12,
            RGBColor(120, 120, 120),
        )?;
    }
    let bar_x = x_of(nums::DID_BAR);
    map_draw(root.draw(&PathElement::new(
        vec![(bar_x, d_top - 6), (bar_x, d_top + 8 * 40)],
        RGBColor(200, 90, 40).stroke_width(2),
    )))?;
    label(
        root,
        (bar_x + 6, d_top - 26),
        &format!("registered bar +{:.2}", nums::DID_BAR),
        12,
        RGBColor(200, 90, 40),
    )?;

    for (i, (name, did, positive, of, gain_negative)) in nums::DID_LADDER.iter().enumerate() {
        let y = d_top + i as i32 * 40;
        label(root, (36, y + 4), name, 13, BLACK)?;
        // Ban 3: the one point where the read-out consumes temporal order and
        // harms accuracy is named on the figure, not left to the caption.
        if *gain_negative {
            label(
                root,
                (150, y + 5),
                "gain is NEGATIVE here",
                11,
                RGBColor(200, 90, 40),
            )?;
        }
        let fill = if *gain_negative {
            RGBColor(200, 90, 40)
        } else {
            RGBColor(46, 106, 150)
        };
        map_draw(root.draw(&Rectangle::new(
            [(d_left, y + 6), (x_of(*did), y + 24)],
            fill.filled(),
        )))?;
        label(
            root,
            (x_of(*did) + 10, y + 6),
            &format!("+{did:.4}   {positive}/{of}"),
            12,
            RGBColor(60, 60, 60),
        )?;
    }

    // Ban 2 and ban 4 in text, beside the points they constrain.
    let note_y = d_top + 8 * 40 + 34;
    label(
        root,
        (36, note_y),
        &format!(
            "The mechanism travels: every point clears the registered bar, and coverage is {} of {} operating points — NOT every width. Twelve carry intact arms with no bin-shuffled twin and claim nothing.",
            nums::COVERAGE_COVERED, nums::COVERAGE_TOTAL
        ),
        13,
        RGBColor(60, 60, 60),
    )?;
    label(
        root,
        (36, note_y + 24),
        &format!(
            "Its SIZE is not the gain. Spearman rho over the six widths is {:.4} against a registered bar of +{:.3} (NOT MET): h768 carries the smallest positive gain on the ladder ({:+.4}) and the largest DiD here ({:+.4}).",
            nums::DID_RHO, nums::DID_RHO_BAR, nums::GAIN_H768, nums::DID_H768
        ),
        13,
        RGBColor(60, 60, 60),
    )?;
    label(
        root,
        (36, note_y + 48),
        "Points are in ladder order and are deliberately NOT connected and NOT sorted by effect size: either would assert a relationship the rank correlation rejects.",
        12,
        RGBColor(120, 120, 120),
    )?;

    Ok(())
}

/// Figure 2 of the lead program — headline accuracy, and that it is not
/// competitive.
///
/// The spec's first ban is the whole construction: "Drawing the panel with an
/// axis that starts at 0.65, or omitting the frontier marker, would make the
/// bar read as a win." So there is exactly ONE accuracy axis in this figure,
/// it runs 0.50 to 1.00, and the instrument's markers and the published field
/// sit on it together. `0.8332` cannot look like a frontier number on an axis
/// that shows where the frontier is.
///
/// Ban 3 is enforced by absence: Pfa-SNN 96.26, Event-SSMA 95.90, SpikeSCR
/// 95.60 and d-cAdLIF 94.85 came from a secondary comparison table rather than
/// a primary source and are excluded from the paper's claims. No constant for
/// them exists in `nums`, so they cannot be drawn by accident.
fn draw_lead_fig2(root: &DrawingArea<SVGBackend<'_>, Shift>) -> Result<(), DrawErr> {
    label(
        root,
        (36, 14),
        "Figure 2 — Headline accuracy, and the 0.80 clearance",
        26,
        BLACK,
    )?;
    label(
        root,
        (36, 48),
        "The read-out takes the instrument from 0.7057 to 0.8332, every seed positive and every seed at or above 0.80. THIS IS NOT COMPETITIVE, and the axis below says so.",
        13,
        RGBColor(70, 70, 70),
    )?;

    // --- the single shared axis --------------------------------------------
    let (x0, x1) = (150, 1330);
    let (lo, hi) = (0.50, 1.00);
    let axis_y = 470;
    let at = |v: f64| x0 + (((v - lo) / (hi - lo)) * (x1 - x0) as f64).round() as i32;

    // Both bands are drawn FIRST, so every marker sits over them. Painted after
    // Panel A they hid the instrument's own point, which is the one thing on
    // this axis that must be visible.
    //
    // The anchor is a BAND, not a marker, and the instrument's marker falls
    // inside it. That is the comparison the caption makes: an architecture
    // carrying no temporal kernel lands where the dataset authors' own no-delay
    // recurrent baseline lands.
    let lo_b = at(nums::FIELD_NO_DELAY - nums::FIELD_NO_DELAY_SD);
    let hi_b = at(nums::FIELD_NO_DELAY + nums::FIELD_NO_DELAY_SD);
    map_draw(root.draw(&Rectangle::new(
        [(lo_b, 330), (hi_b, axis_y)],
        RGBColor(232, 232, 236).filled(),
    )))?;
    // Kept short so it fits over a band this narrow. The reading it supports is
    // in the footer, where there is room for a sentence.
    centered(root, ((lo_b + hi_b) / 2, 298), "no-delay recurrent baseline", 12, BLACK)?;
    centered(
        root,
        ((lo_b + hi_b) / 2, 316),
        "Cramer et al. 2022 — 83.2 ± 1.3%",
        11,
        RGBColor(70, 70, 70),
    )?;

    // The frontier band.
    map_draw(root.draw(&Rectangle::new(
        [(at(nums::FIELD_FRONTIER_LO), 296), (at(nums::FIELD_FRONTIER_HI), axis_y)],
        RGBColor(226, 232, 244).filled(),
    )))?;
    centered(
        root,
        ((at(nums::FIELD_FRONTIER_LO) + at(nums::FIELD_FRONTIER_HI)) / 2, 236),
        "SHD frontier",
        13,
        RGBColor(60, 80, 140),
    )?;
    centered(
        root,
        ((at(nums::FIELD_FRONTIER_LO) + at(nums::FIELD_FRONTIER_HI)) / 2, 218),
        &format!(
            "{:.0}–{:.1}%",
            nums::FIELD_FRONTIER_LO * 100.0,
            nums::FIELD_FRONTIER_HI * 100.0
        ),
        15,
        RGBColor(60, 80, 140),
    )?;

    map_draw(root.draw(&PathElement::new(
        vec![(x0, axis_y), (x1, axis_y)],
        BLACK.stroke_width(2),
    )))?;
    let mut tick = 50;
    while tick <= 100 {
        let v = tick as f64 / 100.0;
        map_draw(root.draw(&PathElement::new(
            vec![(at(v), axis_y), (at(v), axis_y + 8)],
            BLACK.stroke_width(1),
        )))?;
        centered(root, (at(v), axis_y + 22), &format!("{v:.2}"), 12, RGBColor(90, 90, 90))?;
        tick += 5;
    }
    label(root, (36, axis_y - 8), "accuracy", 13, RGBColor(90, 90, 90))?;

    // The 0.80 gate.
    let mut y = 296;
    while y < axis_y {
        map_draw(root.draw(&PathElement::new(
            vec![(at(0.80), y), (at(0.80), (y + 10).min(axis_y))],
            RGBColor(120, 120, 120).stroke_width(2),
        )))?;
        y += 20;
    }
    label(root, (at(0.80) + 8, 446), "0.80 gate", 12, RGBColor(110, 110, 110))?;

    // --- Panel A: this instrument, on that axis -----------------------------
    label(root, (36, 92), "Panel A — this instrument, n = 32 seed-paired", 16, BLACK)?;
    let rate = RGBColor(200, 140, 60);
    let attn = RGBColor(70, 100, 180);
    for (v, color, name, detail) in [
        (
            nums::HEAD_RATE_32,
            rate,
            "rate read-out  ff+fixed",
            format!("{:.4}", nums::HEAD_RATE_32),
        ),
        (
            nums::HEAD_ATTN_32,
            attn,
            "time-axis read-out  d32/L4",
            format!("{:.4}", nums::HEAD_ATTN_32),
        ),
    ] {
        map_draw(root.draw(&Circle::new((at(v), 420), 9, color.filled())))?;
        map_draw(root.draw(&PathElement::new(
            vec![(at(v), 420), (at(v), axis_y)],
            color.stroke_width(1),
        )))?;
        centered(root, (at(v), 392), &detail, 15, BLACK)?;
        centered(root, (at(v), 376), name, 12, RGBColor(80, 80, 80))?;
    }
    map_draw(root.draw(&PathElement::new(
        vec![(at(nums::HEAD_RATE_32), 420), (at(nums::HEAD_ATTN_32), 420)],
        BLACK.stroke_width(2),
    )))?;
    centered(
        root,
        ((at(nums::HEAD_RATE_32) + at(nums::HEAD_ATTN_32)) / 2, 438),
        &format!("gain +{:.4}", nums::HEAD_GAIN_32),
        14,
        BLACK,
    )?;

    label(
        root,
        (36, 118),
        &format!(
            "n = 32 (confirmation):  {:.4} → {:.4},  gain +{:.4},  positive 32/32,  ≥ 0.80 in 32/32,  |e400 − e200| = {:.4}",
            nums::HEAD_RATE_32, nums::HEAD_ATTN_32, nums::HEAD_GAIN_32, nums::HEAD_BUDGET_DELTA
        ),
        14,
        BLACK,
    )?;
    label(
        root,
        (36, 142),
        &format!(
            "n = 12 (registered):     {:.4} → {:.4},  gain +{:.4},  positive 12/12,  ≥ 0.80 in 12/12",
            nums::HEAD_RATE_12, nums::HEAD_ATTN_12, nums::HEAD_GAIN_12
        ),
        14,
        RGBColor(80, 80, 80),
    )?;
    label(
        root,
        (36, 166),
        "n = 12 is the registered measurement and n = 32 the confirmation. Neither supersedes the other.",
        12,
        RGBColor(120, 120, 120),
    )?;

    // --- Panel B: the published field, same axis ----------------------------
    label(
        root,
        (36, 196),
        "Panel B — the published field, on the same axis. The instrument carries no temporal kernel of any kind.",
        16,
        BLACK,
    )?;
    let field = RGBColor(90, 90, 100);
    // Staggered: at this scale the two markers are 1.3 points apart and their
    // labels collide on one line.
    for (v, name, text_y) in [
        (nums::FIELD_TA_SNN, "TA-SNN 91.08 — attention over the time axis (ICCV 2021)", 250),
        (nums::FIELD_STSC_SNN, "STSC-SNN 92.36 — attention inside the synapse (2022)", 272),
    ] {
        map_draw(root.draw(&PathElement::new(
            vec![(at(v), text_y + 12), (at(v), axis_y)],
            field.stroke_width(1),
        )))?;
        label(root, (at(v) - 340, text_y), name, 11, field)?;
    }
    // Ban 2, drawn to scale on this axis rather than only asserted in words.
    let band = at(lo + nums::FIELD_UNRESOLVABLE) - x0;
    map_draw(root.draw(&Rectangle::new(
        [(x0, 512), (x0 + band, 528)],
        RGBColor(206, 206, 214).filled(),
    )))?;
    label(
        root,
        (x0 + band + 12, 506),
        &format!(
            "{:.1} points, to scale. SHD ships no validation set: Baronig et al. 2025 report one model at 95.81 validating on test and 93.79 on a held-out split.",
            nums::FIELD_UNRESOLVABLE * 100.0
        ),
        12,
        RGBColor(90, 90, 90),
    )?;
    label(
        root,
        (x0 + band + 12, 526),
        "Differences narrower than this between published SHD numbers are not reliably meaningful — including this paper's own.",
        12,
        RGBColor(90, 90, 90),
    )?;

    // --- Panel C: the clearance is geometry-specific ------------------------
    label(
        root,
        (36, 568),
        "Panel C — the 0.80 clearance is geometry-specific. The gain is not: attention buys roughly the same amount everywhere tested.",
        16,
        BLACK,
    )?;
    for (i, (name, r, a, gain, clears)) in nums::GEOM.into_iter().enumerate() {
        let y = 604 + (i as i32) * 46;
        label(root, (48, y), name, 13, BLACK)?;
        label(root, (560, y), &format!("rate {r:.4}"), 13, rate)?;
        label(root, (720, y), &format!("attention {a:.4}"), 13, attn)?;
        label(root, (920, y), &format!("gain +{gain:.4}"), 13, BLACK)?;
        label(
            root,
            (1090, y),
            &format!("≥ 0.80 in {clears}"),
            13,
            if clears == "12/12" { BLACK } else { RGBColor(170, 90, 60) },
        )?;
    }

    label(
        root,
        (36, 742),
        "This instrument lands INSIDE the no-delay recurrent band — which is where an architecture carrying no temporal kernel of any kind should land, and is the whole reading.",
        12,
        RGBColor(70, 70, 70),
    )?;
    label(
        root,
        (36, 770),
        "Excluded from this axis on purpose: Pfa-SNN 96.26, Event-SSMA 95.90, SpikeSCR 95.60, d-cAdLIF 94.85 — a secondary comparison table, not a primary source.",
        12,
        RGBColor(120, 120, 120),
    )?;
    label(
        root,
        (36, 800),
        "NOT MACHINE-CHECKED: every literature value in Panel B comes from a 2026-08-27 search pass. Unlike every SHD number in this paper it is not verified against cells on disk,",
        12,
        RGBColor(160, 80, 60),
    )?;
    label(
        root,
        (36, 822),
        "and check_every_number.py does not sweep the section it lives in. Each must be checked against its primary source before submission.",
        12,
        RGBColor(160, 80, 60),
    )?;
    Ok(())
}

/// Figure 3 of the lead program — the width ladder and the threshold.
///
/// Five of this figure's bans are about lines that are not there:
///
/// 1. No connector runs through the first five rungs, and none may. H16-1 —
///    "the gain decays monotonically with width up to the collapse" — is NOT
///    MET: seed-paired, gain(h384) − gain(h512) is −0.0116 with sd 0.0253,
///    negative in only 7 of 12 seeds. A fitted curve or a monotone-looking
///    connector asserts an ordering the measurement cannot support, so the
///    rungs are points and h384–h512 carries an explicit bracket saying they
///    are indistinguishable.
/// 2. No dip at h384 is drawn. The registration demanded strict ordering with
///    0.005 separations over quantities inside their own noise floor; that is a
///    defect in the registration, not a finding about width.
/// 3. The step is placed between h768 and h1024, not between h512 and h1024 —
///    the four-rung reading is superseded and h768 is still +0.0560.
/// 4. No mechanism is offered. All three registered levers failed, and the
///    known correlate — gradient norms leaving O(1) — is a correlate. The
///    figure's own words are "located but unexplained", and the parsimonious
///    alternative (overfitting on 8,156 training samples) is not excluded by
///    anything in this paper, so it must not be excluded by the drawing.
/// 5. The h1024 `d32/L2` result is absent entirely: +0.0392 rests on three
///    points with L3 missing and is registered as its own wave.
///
/// The width axis is categorical and evenly spaced. A log-width axis would
/// visually compress the step, which is the one feature the figure exists for.
fn draw_lead_fig3(root: &DrawingArea<SVGBackend<'_>, Shift>) -> Result<(), DrawErr> {
    label(
        root,
        (36, 14),
        "Figure 3 — The width ladder, and a threshold rather than a slope",
        26,
        BLACK,
    )?;
    label(
        root,
        (36, 48),
        "EVERY RUNG IS d32/L4. The gain is positive on five rungs and inverts at h1024 — a property of THIS read-out depth, not of the width: at h1024, d32/L2 gains +0.0405 in 20/20 seeds.",
        13,
        RGBColor(70, 70, 70),
    )?;

    // --- Panel A ------------------------------------------------------------
    let (lo, hi) = (-0.20, 0.15);
    let (top, bottom) = (110, 470);
    let x0 = 200;
    let step_x = 175;
    let at_y =
        |v: f64| bottom - (((v - lo) / (hi - lo)) * (bottom - top) as f64).round() as i32;
    let at_x = |i: usize| x0 + (i as i32) * step_x;

    for k in 0..8 {
        let v = -0.20 + 0.05 * k as f64;
        let y = at_y(v);
        map_draw(root.draw(&PathElement::new(
            vec![(x0 - 60, y), (1330, y)],
            RGBColor(232, 232, 236).stroke_width(1),
        )))?;
        label(root, (110, y - 9), &format!("{v:+.2}"), 12, RGBColor(120, 120, 120))?;
    }
    let zero = at_y(0.0);
    map_draw(root.draw(&PathElement::new(
        vec![(x0 - 60, zero), (1330, zero)],
        BLACK.stroke_width(2),
    )))?;
    label(root, (36, zero - 9), "gain", 13, RGBColor(90, 90, 90))?;

    // The step, between h768 and h1024 and nowhere else.
    let break_x = (at_x(4) + at_x(5)) / 2;
    let mut y = top;
    while y < bottom {
        map_draw(root.draw(&PathElement::new(
            vec![(break_x, y), (break_x, (y + 12).min(bottom))],
            RGBColor(170, 90, 60).stroke_width(2),
        )))?;
        y += 24;
    }
    label(root, (break_x + 10, top + 4), "the step sits HERE,", 12, RGBColor(170, 90, 60))?;
    label(root, (break_x + 10, top + 22), "between h768 and h1024", 12, RGBColor(170, 90, 60))?;

    for (i, (name, _rate, _attn, gain, positive)) in nums::LADDER.into_iter().enumerate() {
        let (x, y) = (at_x(i), at_y(gain));
        let color = if gain > 0.0 {
            RGBColor(70, 100, 180)
        } else {
            RGBColor(170, 90, 60)
        };
        map_draw(root.draw(&Circle::new((x, y), 10, color.filled())))?;
        centered(
            root,
            (x, y - 26),
            &format!("{gain:+.4}"),
            16,
            BLACK,
        )?;
        centered(root, (x, bottom + 20), name, 15, BLACK)?;
        centered(root, (x, bottom + 42), positive, 12, RGBColor(110, 110, 110))?;
    }

    // h384 and h512 are not distinguishable, and the figure has to say so where
    // a reader would otherwise read a dip.
    let (b0, b1) = (at_x(2), at_x(3));
    let bracket_y = at_y(0.0) + 26;
    map_draw(root.draw(&PathElement::new(
        vec![(b0, bracket_y - 8), (b0, bracket_y), (b1, bracket_y), (b1, bracket_y - 8)],
        RGBColor(120, 120, 130).stroke_width(2),
    )))?;
    centered(
        root,
        ((b0 + b1) / 2, bracket_y + 14),
        &format!(
            "not distinguishable at n = 12: paired {:+.4}, sd {:.4}, negative in 7 of 12",
            nums::LADDER_H384_H512,
            nums::LADDER_H384_H512_SD
        ),
        12,
        RGBColor(100, 100, 110),
    )?;

    label(
        root,
        (36, 540),
        &format!(
            "The drop into h1024 is {:.4} — {:.1}× the largest gap below it ({:.4}), and more than twice the registered 3× bar. That is what makes it a threshold rather than the slope continuing.",
            nums::LADDER_DROP, nums::LADDER_DROP_RATIO, nums::LADDER_LARGEST_GAP_BELOW
        ),
        13,
        BLACK,
    )?;
    label(
        root,
        (36, 562),
        "No monotonic decay is claimed above the threshold, and no dip at h384: the registration asked for 0.005 separations over quantities inside their own noise floor.",
        12,
        RGBColor(110, 110, 110),
    )?;
    label(
        root,
        (36, 584),
        "n = 12 per rung, d32/L4, e400, seed-paired. Rungs are points: no curve is fitted and none may be.",
        12,
        RGBColor(110, 110, 110),
    )?;

    // Required annotation, `PAPER_FIGURE_SPEC.md` Figure 3, added 2026-08-29.
    //
    // This ladder plots the GAIN. A reader arriving from Figure 1 will assume
    // the order-dependence rises and falls with it, and wave 21 measured that
    // it does not. Ban 6 forbids drawing the DiD here as a second series --
    // precisely BECAUSE the two are uncorrelated, superimposing them invites
    // the eye to find the relationship the rank correlation rejects. So it is
    // said in words instead, and it sits with the other statements about what
    // this figure does NOT show.
    //
    // Two lines, not one: at 12pt the single-line version ran off the 1400px
    // canvas and lost the sentence naming where the DiD ladder actually is.
    label(
        root,
        (36, 610),
        &format!(
            "THE MECHANISM DOES NOT TRACK THIS CURVE. Spearman rho between these rungs and their difference-in-differences is {:.4}, against a registered bar of +{:.3} — NOT MET.",
            nums::DID_RHO, nums::DID_RHO_BAR
        ),
        12,
        RGBColor(160, 80, 60),
    )?;
    label(
        root,
        (36, 630),
        &format!(
            "h768 carries the smallest positive gain on this ladder ({:+.4}) and the largest DiD in the campaign ({:+.4}). The DiD ladder is Figure 1 Panel D, and is deliberately not superimposed here.",
            nums::GAIN_H768, nums::DID_H768
        ),
        12,
        RGBColor(160, 80, 60),
    )?;

    // --- Panel B ------------------------------------------------------------
    label(
        root,
        (36, 666),
        "Panel B — located, and not explained. Three preregistered rescue levers at h1024 / d32/L4, n = 12 each.",
        16,
        BLACK,
    )?;
    label(
        root,
        (36, 690),
        "Every lever is negative, and every one is worse than the arm it was meant to rescue.",
        12,
        RGBColor(110, 110, 110),
    )?;
    for (i, (lever, gain, positive, norm)) in nums::LEVERS.into_iter().enumerate() {
        let y = 724 + (i as i32) * 34;
        let rescued = lever.starts_with('(');
        let ink = if rescued { RGBColor(110, 110, 110) } else { BLACK };
        label(root, (48, y), lever, 14, ink)?;
        label(root, (560, y), &format!("gain {gain:+.4}"), 14, RGBColor(170, 90, 60))?;
        label(root, (760, y), &format!("positive {positive}"), 13, ink)?;
        label(
            root,
            (930, y),
            &format!("median epoch-mean gradient norm {norm:.3}"),
            13,
            ink,
        )?;
    }
    label(
        root,
        (36, 870),
        "Clipping moved the median gradient norm from 55.494 to 11.660 and accuracy did not follow. At h512 the same flag is inert: 12/12 cells byte-identical to the archived unclipped cells.",
        12,
        RGBColor(110, 110, 110),
    )?;
    label(
        root,
        (36, 900),
        "LOCATED BUT UNEXPLAINED. Gradient norms leaving O(1) are a correlate, not a cause, and overfitting on 8,156 training samples is not excluded by anything in this paper.",
        13,
        RGBColor(70, 70, 70),
    )?;
    Ok(())
}

/// Figure 4 of the lead program — the resolution ladder.
///
/// Only `fixed-tN` may be plotted: the `published-Nms` family moves bin width
/// and sequence length together, so no number from it can be attributed to
/// either, and the S-5 test built on it is refuted and withdrawn. The fixed
/// 1400 ms window is stated on the figure because it is the whole reason the
/// axis means anything.
///
/// Both series are drawn. The rate arm rises +0.0397 across the same ladder —
/// checked against a preregistered 0.05 confound bar and cleared — and without
/// it the falling gain reads as the attention arm degrading rather than as the
/// rate arm catching up.
fn draw_lead_fig4(root: &DrawingArea<SVGBackend<'_>, Shift>) -> Result<(), DrawErr> {
    label(
        root,
        (36, 14),
        "Figure 4 — The resolution ladder, with the analysis window held fixed",
        26,
        BLACK,
    )?;
    label(
        root,
        (36, 48),
        "fixed-tN: the 1400 ms analysis window is HELD FIXED and only the number of frames varies. This is the axis that isolates resolution from sequence length. n = 12 per rung.",
        13,
        RGBColor(70, 70, 70),
    )?;

    let (lo, hi) = (0.60, 0.90);
    let (top, bottom) = (130, 560);
    let x0 = 300;
    let step_x = 380;
    let at_y =
        |v: f64| bottom - (((v - lo) / (hi - lo)) * (bottom - top) as f64).round() as i32;
    let at_x = |i: usize| x0 + (i as i32) * step_x;

    for k in 0..7 {
        let v = 0.60 + 0.05 * k as f64;
        let y = at_y(v);
        map_draw(root.draw(&PathElement::new(
            vec![(200, y), (1330, y)],
            RGBColor(234, 234, 238).stroke_width(1),
        )))?;
        label(root, (146, y - 9), &format!("{v:.2}"), 12, RGBColor(120, 120, 120))?;
    }
    // The 0.80 gate, cleared 12/12 at every rung.
    let gate = at_y(0.80);
    let mut x = 200;
    while x < 1330 {
        map_draw(root.draw(&PathElement::new(
            vec![(x, gate), ((x + 14).min(1330), gate)],
            RGBColor(120, 120, 120).stroke_width(2),
        )))?;
        x += 28;
    }
    label(root, (1210, gate - 22), "0.80 gate — cleared 12/12 at every rung", 12, RGBColor(110, 110, 110))?;
    label(root, (36, at_y(0.75) - 9), "accuracy", 13, RGBColor(90, 90, 90))?;

    let attn = RGBColor(70, 100, 180);
    let rate = RGBColor(200, 140, 60);
    // Both series, joined within themselves only. The comparison the figure
    // makes is vertical — the gap at each rung — not across rungs.
    for (series, color, pick) in [
        ("attention", attn, 1usize),
        ("rate", rate, 0usize),
    ] {
        let _ = series;
        let points: Vec<(i32, i32)> = nums::RESOLUTION
            .iter()
            .enumerate()
            .map(|(i, (_, _, r, a, _))| (at_x(i), at_y(if pick == 1 { *a } else { *r })))
            .collect();
        map_draw(root.draw(&PathElement::new(points.clone(), color.stroke_width(2))))?;
        for point in points {
            map_draw(root.draw(&Circle::new(point, 9, color.filled())))?;
        }
    }

    for (i, (contract, bin, r, a, gain)) in nums::RESOLUTION.into_iter().enumerate() {
        let x = at_x(i);
        centered(root, (x, at_y(a) - 28), &format!("{a:.4}"), 15, BLACK)?;
        centered(root, (x, at_y(r) + 20), &format!("{r:.4}"), 15, BLACK)?;
        map_draw(root.draw(&PathElement::new(
            vec![(x, at_y(a)), (x, at_y(r))],
            RGBColor(140, 140, 148).stroke_width(1),
        )))?;
        centered(
            root,
            (x + 76, (at_y(a) + at_y(r)) / 2),
            &format!("gain +{gain:.4}"),
            15,
            BLACK,
        )?;
        centered(
            root,
            (x + 76, (at_y(a) + at_y(r)) / 2 + 20),
            "12/12 positive",
            12,
            RGBColor(110, 110, 110),
        )?;
        centered(root, (x, bottom + 24), contract, 16, BLACK)?;
        centered(root, (x, bottom + 46), &format!("{bin} bins"), 13, RGBColor(110, 110, 110))?;
    }
    centered(root, (at_x(0) - 130, bottom + 24), "coarse", 13, RGBColor(120, 120, 120))?;
    centered(root, (at_x(2) + 130, bottom + 24), "fine", 13, RGBColor(120, 120, 120))?;
    // Below the bin labels, not level with them.
    label(root, (36, 636), "time-axis read-out", 13, attn)?;
    label(root, (220, 636), "rate read-out", 13, rate)?;

    label(
        root,
        (36, 664),
        &format!(
            "The advantage SHRINKS as bins get finer: gain(t500) − gain(t100) = {:.4} against a two-sided bar of 0.03.",
            nums::RESOLUTION_TREND
        ),
        15,
        BLACK,
    )?;
    label(
        root,
        (36, 692),
        "That is the opposite of the direction the withdrawn S-5 hypothesis predicted — now asked on an axis that isolates resolution from sequence length.",
        13,
        RGBColor(90, 90, 90),
    )?;
    label(
        root,
        (36, 728),
        &format!(
            "The rate baseline rises +{:.4} across the same ladder, inside the preregistered 0.05 confound bar. Both series are drawn because without the baseline the falling gain",
            nums::RESOLUTION_BASELINE_DRIFT
        ),
        13,
        RGBColor(90, 90, 90),
    )?;
    label(
        root,
        (36, 750),
        "reads as the attention arm degrading, when what it shows is the rate arm catching up. So this is a property of the read-out, not of the substrate beneath it.",
        13,
        RGBColor(90, 90, 90),
    )?;
    label(
        root,
        (36, 792),
        "NOT DRAWN, and not drawable here: the published-Nms family, which moves bin width and sequence length together so no number from it can be attributed to either.",
        12,
        RGBColor(120, 120, 120),
    )?;
    label(
        root,
        (36, 822),
        "NO MECHANISM AND NO PREFERENCE. The paper offers no account of why the advantage shrinks and recommends no operating point. \"Attention prefers coarse bins\" is not carried by this.",
        12,
        RGBColor(120, 120, 120),
    )?;
    Ok(())
}

/// One rule, drawn by rule rather than by topology.
///
/// `PAPER_FIGURE_SPEC.md` requires this: the low-richness / low-addressability
/// cell holds `MatchedLocal` at chance and `MatchedRlFlat` at 0.78, and
/// collapsing them into one "broadcast ±1" box is a stronger version of the
/// overreach the lead claim's wording exists to avoid.
fn rule_card(
    root: &DrawingArea<SVGBackend<'_>, Shift>,
    (x0, y0, x1, y1): (i32, i32, i32, i32),
    verdict: Verdict,
    rule: &str,
    mechanism: &str,
    value: nums::Both,
) -> Result<(), DrawErr> {
    map_draw(root.draw(&Rectangle::new(
        [(x0, y0), (x1, y1)],
        verdict.fill().filled(),
    )))?;
    map_draw(root.draw(&Rectangle::new(
        [(x0, y0), (x1, y1)],
        verdict.border().stroke_width(2),
    )))?;
    // The verdict is a labelled chip, not a colour alone: the categorical
    // encoding has to survive a greyscale print.
    let chip = 128;
    map_draw(root.draw(&Rectangle::new(
        [(x0 + 10, y0 + 10), (x0 + 10 + chip, y0 + 32)],
        verdict.border().filled(),
    )))?;
    centered(root, (x0 + 10 + chip / 2, y0 + 21), verdict.tag(), 12, WHITE)?;
    label(root, (x0 + 10 + chip + 14, y0 + 12), rule, 16, BLACK)?;
    label(root, (x0 + 12, y0 + 44), mechanism, 12, RGBColor(90, 90, 90))?;
    label(root, (x0 + 12, y0 + 66), &both(value), 15, BLACK)?;
    Ok(())
}

fn draw_fig_m(root: &DrawingArea<SVGBackend<'_>, Shift>) -> Result<(), DrawErr> {
    label(
        root,
        (36, 16),
        "Figure M — Mechanism: richness × addressability",
        26,
        BLACK,
    )?;
    label(
        root,
        (36, 50),
        "Matched dense-LIF forward, n = 20 per cell, 2026-08-25 re-run at MATCHED_INPUT_SCALE = 2.0. Pass / fail / at-chance — not a graded surface.",
        13,
        RGBColor(70, 70, 70),
    )?;

    // The ceiling is the point of the panel, not an aside: with the reference
    // saturated, every passing arm reduces to "above 0.75" and no ordering
    // among them may be drawn or claimed.
    let reference = Verdict::Reference;
    map_draw(root.draw(&Rectangle::new(
        [(36, 74), (1364, 112)],
        reference.fill().filled(),
    )))?;
    map_draw(root.draw(&Rectangle::new(
        [(36, 74), (1364, 112)],
        reference.border().stroke_width(2),
    )))?;
    label(
        root,
        (48, 84),
        &format!(
            "SuperSpike BPTT reference saturates: {:.4} ff / {:.4} rec, every suite, both graphs. Every other rule tested clears the gate, so this panel shows WHICH SINGLE RULE FAILS a task the rest saturate — it does not rank the rest.",
            nums::CEILING.0,
            nums::CEILING.1
        ),
        13,
        BLACK,
    )?;

    label(root, (352, 126), "Low addressability (broadcast)", 15, BLACK)?;
    label(
        root,
        (930, 126),
        "High addressability (directed / local feedback)",
        15,
        BLACK,
    )?;
    label(root, (16, 236), "Low richness", 14, BLACK)?;
    label(root, (16, 256), "(±1)", 14, RGBColor(90, 90, 90))?;
    label(root, (16, 470), "High richness", 14, BLACK)?;
    label(root, (16, 490), "(graded)", 14, RGBColor(90, 90, 90))?;

    // --- low richness ------------------------------------------------------
    // Two rules, one cell. Drawn apart, with the distance between them stated,
    // because that distance is the reason the lead claim says "±1 three-factor"
    // and not "broadcast".
    map_draw(root.draw(&Rectangle::new(
        [(186, 148), (798, 372)],
        RGBColor(248, 248, 250).filled(),
    )))?;
    rule_card(
        root,
        (196, 156, 788, 254),
        Verdict::Fail,
        "±1 × surrogate eligibility",
        "MatchedLocal — the lead FAIL, and at chance on both graphs",
        nums::BROADCAST_PM1,
    )?;
    rule_card(
        root,
        (196, 262, 788, 360),
        Verdict::Contrast,
        "±1 broadcast REINFORCE",
        "MatchedRlFlat — same reward, same topology, well above chance",
        nums::RL_FLAT,
    )?;
    label(
        root,
        (196, 366),
        "One cell, two rules, 0.28 apart. Collapsing them into one “broadcast ±1” box is the overreach the lead claim’s wording exists to avoid.",
        11,
        RGBColor(110, 110, 110),
    )?;

    rule_card(
        root,
        (818, 156, 1364, 254),
        Verdict::Pass,
        "REINFORCE × frozen B_i",
        "MatchedRlReinforceFb — directed feedback, same ±1 reward",
        nums::RL_FB,
    )?;

    // --- high richness -----------------------------------------------------
    rule_card(
        root,
        (196, 400, 788, 498),
        Verdict::Contrast,
        "broadcast graded error",
        "MatchedBroadcastGradedError — on the DFA schedule; not a locality proof",
        nums::BROADCAST_GRADED,
    )?;
    rule_card(
        root,
        (818, 400, 1364, 498),
        Verdict::Pass,
        "graded error × DFA",
        "MatchedDfaGradedError",
        nums::DFA,
    )?;
    label(
        root,
        (818, 512),
        "Richness alone does not decide it: a graded BROADCAST rule reaches 0.9975.",
        12,
        RGBColor(110, 110, 110),
    )?;
    label(
        root,
        (818, 534),
        "Addressability alone does not either: ±1 through a frozen B_i passes.",
        12,
        RGBColor(110, 110, 110),
    )?;
    label(
        root,
        (818, 556),
        "What fails is the one rule that has neither.",
        12,
        RGBColor(60, 60, 60),
    )?;

    // --- Panel B -----------------------------------------------------------
    map_draw(root.draw(&Rectangle::new(
        [(36, 622), (1364, 872)],
        RGBColor(245, 245, 248).filled(),
    )))?;
    label(
        root,
        (52, 632),
        "Panel B — XOR locality flip (1-layer xor_thresh). Addressability evidence: broadcast fails a task DFA solves.",
        15,
        BLACK,
    )?;
    draw_bar_row(
        root,
        90,
        664,
        1180,
        170,
        Some(0.5),
        &[
            ("Broadcast — at chance", nums::XOR_BCAST, Verdict::Fail.border()),
            ("DFA — solves", nums::XOR_DFA, Verdict::Pass.border()),
            (
                "Gradient — ceiling",
                nums::XOR_GRAD,
                Verdict::Reference.border(),
            ),
        ],
    )?;
    label(
        root,
        (52, 848),
        "Not claimed for 2-layer mid-init depth locality, where broadcast also solves. Matched PASS does not imply live muted-θ / k-WTA G2 PASS.",
        12,
        RGBColor(90, 90, 90),
    )?;
    Ok(())
}

fn draw_fig1(root: &DrawingArea<SVGBackend<'_>, Shift>) -> Result<(), DrawErr> {
    label(
        root,
        (40, 24),
        "Figure 5 — Matched rule-swap schematic",
        26,
        BLACK,
    )?;
    label(
        root,
        (40, 60),
        "Forward held fixed; only the update rule changes. Lead FAIL label: broadcast ±1 three-factor. Feed-forward / recurrent, 2026-08-25 re-run.",
        14,
        RGBColor(60, 60, 60),
    )?;

    box_card(
        root,
        (60, 200, 480, 470),
        RGBColor(235, 240, 255),
        RGBColor(70, 100, 180),
        "Dense-LIF coincidence forward",
        &[
            "shared win / wrec / wout",
            "continuous frames",
            "both graphs: wrec = 0 and wrec live",
            &format!("SuperSpike ceiling {:.4}", nums::CEILING.0),
        ],
    )?;

    let plugs = [
        (
            120,
            Verdict::Fail,
            "±1 × surrogate eligibility",
            "MatchedLocal — scalar reward, no addressing",
            nums::BROADCAST_PM1,
        ),
        (
            300,
            Verdict::Pass,
            "graded error × DFA",
            "MatchedDfaGradedError — graded error, fixed random feedback",
            nums::DFA,
        ),
        (
            480,
            Verdict::Pass,
            "REINFORCE × frozen B_i",
            "MatchedRlReinforceFb — ±1 reward through directed feedback",
            nums::RL_FB,
        ),
        (
            660,
            Verdict::Reference,
            "SuperSpike BPTT",
            "the ceiling every other arm is measured against",
            nums::CEILING,
        ),
    ];
    for (y, verdict, title, mechanism, value) in plugs {
        map_draw(root.draw(&PathElement::new(
            vec![(480, 335), (560, y + 55)],
            verdict.border().stroke_width(2),
        )))?;
        rule_card(root, (560, y, 1300, y + 110), verdict, title, mechanism, value)?;
    }
    Ok(())
}

/// Figure S — the substrate panel.
///
/// # Why this had no figure until 2026-08-29
///
/// §3.7 of `PAPER_DRAFT.md` is a lead-program section with three waves behind
/// it, and no sheet specified a figure for it. The omission was hidden behind a
/// wrong label: `PAPER_SKELETON.md`'s figure map called this the "Fig. 4
/// substrate panel", and Figure 4 is the resolution ladder, so the map looked
/// complete while naming something that did not exist. Correcting Figure 4's
/// identity is what surfaced it.
///
/// Lettered rather than numbered, beside Figure M: a fifth lead figure would
/// renumber the secondary program 5-9 -> 6-10 one day after the 2026-08-27
/// renumber, for one figure.
///
/// # Four bans, all of them limits the draft already calls load-bearing
///
/// 1. **The recurrent substrate must not read as a win.** `rec+alif+attn`
///    reaches 0.7874 against `ff+fixed+attn`'s 0.8289 at the same scale and the
///    paper issues NO VERDICT on that ordering. So: Table SHD-7's row order, no
///    sort by gain, and both attention accuracies on one shared axis where a
///    reader can see which is higher.
/// 2. **Neither ratio may be quoted alone.** The recurrent gain is measured
///    from a base 0.18 lower; headroom-normalised the ratio falls 2.2x -> 1.34x,
///    and that normalisation is POST-HOC and NOT REGISTERED. Both go on the
///    figure with the status labelled.
/// 3. **Ten pairs, the registered minimum**, and one further loss on either arm
///    would have made the comparison unreportable. `n` is printed per row.
/// 4. **Adaptation is inert AT THIS OPERATING POINT, not in general.** Panel C
///    is why: on the recurrent substrate adaptation is what prevents
///    saturation, so dropping it would turn a scoped null into a general one.
fn draw_fig_s_substrate(root: &DrawingArea<SVGBackend<'_>, Shift>) -> Result<(), DrawErr> {
    label(
        root,
        (36, 16),
        "Figure S — Substrate: the read-out does not substitute for temporal state",
        26,
        BLACK,
    )?;
    label(
        root,
        (36, 50),
        "h128 / published-2ms / adjacent-sum-5 / e400 / d32-L4, seed-paired. The anchor campaign's 720 cells all sat on ff+fixed, so its gain had two readings it could not separate.",
        13,
        RGBColor(70, 70, 70),
    )?;
    label(
        root,
        (36, 72),
        "ADDS temporal structure no such substrate represents, or SUBSTITUTES for the adaptation and recurrence ff+fixed happens not to have. ETLP makes the second reading the live one.",
        13,
        RGBColor(110, 110, 110),
    )?;

    // --- Panel A ------------------------------------------------------------
    label(
        root,
        (36, 108),
        "Panel A — one shared accuracy axis, in the row order of Table SHD-7. NOT sorted by gain.",
        15,
        BLACK,
    )?;

    let (lo, hi) = (0.45, 0.90);
    let (x0, x1) = (430, 1180);
    let at_x = |v: f64| x0 + (((v - lo) / (hi - lo)) * (x1 - x0) as f64).round() as i32;
    let (top, bottom) = (150, 380);

    for k in 0..=9 {
        let v = lo + 0.05 * k as f64;
        let x = at_x(v);
        map_draw(root.draw(&PathElement::new(
            vec![(x, top), (x, bottom)],
            RGBColor(238, 238, 242).stroke_width(1),
        )))?;
        centered(root, (x, bottom + 14), &format!("{v:.2}"), 11, RGBColor(140, 140, 140))?;
    }

    let rate_col = RGBColor(200, 140, 60);
    let attn_col = RGBColor(70, 100, 180);
    for (i, (substrate, note, scale, pairs, rate, attn, gain)) in
        nums::SUBSTRATE.into_iter().enumerate()
    {
        let y = top + 26 + (i as i32) * 54;
        label(root, (36, y - 18), substrate, 16, BLACK)?;
        label(
            root,
            (36, y + 2),
            &format!("{note} · scale {scale:.1} · n = {pairs} pairs"),
            12,
            RGBColor(110, 110, 110),
        )?;
        // Rate -> attention on one axis. The connector is within a row only:
        // the comparison this figure makes is horizontal, and joining rows
        // would draw the substrate ordering ban 1 refuses.
        map_draw(root.draw(&PathElement::new(
            vec![(at_x(rate), y), (at_x(attn), y)],
            RGBColor(160, 160, 168).stroke_width(2),
        )))?;
        map_draw(root.draw(&Circle::new((at_x(rate), y), 8, rate_col.filled())))?;
        map_draw(root.draw(&Circle::new((at_x(attn), y), 8, attn_col.filled())))?;
        label(
            root,
            (at_x(rate) - 56, y - 8),
            &format!("{rate:.4}"),
            12,
            RGBColor(80, 80, 80),
        )?;
        label(
            root,
            (at_x(attn) + 14, y - 8),
            &format!("{attn:.4}"),
            12,
            RGBColor(80, 80, 80),
        )?;
        label(root, (1224, y - 8), &format!("gain +{gain:.4}"), 14, BLACK)?;
    }
    // Below the header, not level with it: the header line runs past x=430.
    label(root, (430, 128), "● rate read-out", 12, rate_col)?;
    label(root, (560, 128), "● + attention d32/L4", 12, attn_col)?;

    // Ban 1, drawn where the eye already is: a vertical marker at the
    // feed-forward attention arm, so the recurrent one is visibly BELOW it.
    let ff_attn = nums::SUBSTRATE[3].5;
    let rec_attn = nums::SUBSTRATE[2].5;
    let mark = at_x(ff_attn);
    let mut my = top;
    while my < bottom {
        map_draw(root.draw(&PathElement::new(
            vec![(mark, my), (mark, (my + 10).min(bottom))],
            RGBColor(120, 120, 120).stroke_width(2),
        )))?;
        my += 20;
    }
    label(
        root,
        (36, bottom + 40),
        &format!(
            "THE RECURRENT SUBSTRATE DOES NOT WIN. rec+alif+attn reaches {rec_attn:.4} against ff+fixed+attn's {ff_attn:.4} at the same scale — attention closes most of the gap the",
        ),
        14,
        BLACK,
    )?;
    label(
        root,
        (36, bottom + 62),
        "substrate gives away, and not all of it. NO VERDICT IS ISSUED ON THAT ORDERING, and nothing here is sorted by gain.",
        14,
        BLACK,
    )?;

    // --- Panel B ------------------------------------------------------------
    label(
        root,
        (36, 496),
        "Panel B — the registered contrasts, each against its two-sided bar",
        15,
        BLACK,
    )?;
    let contrasts = [
        (
            "A-1  gain(ff+alif) − gain(ff+fixed)",
            nums::SUBSTRATE_A1,
            "6 of 12",
            Verdict::Fail,
            "a coin flip — adaptation makes no difference to the gain HERE",
        ),
        (
            "M-2  gain(rec+alif) − gain(ff+fixed), both at scale 0.4",
            nums::SUBSTRATE_M2,
            "10 of 10",
            Verdict::Pass,
            "SUPPORTED — the gain roughly doubles on the recurrent substrate",
        ),
        (
            "M-4  ff+fixed at scale 0.4 vs archived at 1.0",
            nums::SUBSTRATE_M4,
            "—",
            Verdict::Contrast,
            "the scale is not doing the work",
        ),
    ];
    let (bx0, bx1) = (700, 1000);
    let at_b = |v: f64| bx0 + ((v / 0.20) * (bx1 - bx0) as f64).round() as i32;
    let bar_x = at_b(nums::SUBSTRATE_BAR);
    for (i, (name, value, positive, verdict, reading)) in contrasts.into_iter().enumerate() {
        let y = 534 + (i as i32) * 46;
        label(root, (36, y - 8), name, 13, BLACK)?;
        map_draw(root.draw(&Circle::new(
            (at_b(value), y),
            7,
            verdict.border().filled(),
        )))?;
        // A value short of the bar sits within ~45px of the axis origin, and
        // a right-hand label then runs straight through the dashed bar. Label
        // those on the left.
        let text = format!("+{value:.4}   {positive}");
        let left = value < nums::SUBSTRATE_BAR;
        label(
            root,
            (if left { at_b(value) - 132 } else { at_b(value) + 14 }, y - 8),
            &text,
            12,
            RGBColor(80, 80, 80),
        )?;
        label(root, (1080, y - 8), reading, 12, RGBColor(110, 110, 110))?;
    }
    let mut by = 520;
    while by < 640 {
        map_draw(root.draw(&PathElement::new(
            vec![(bar_x, by), (bar_x, (by + 10).min(640))],
            RGBColor(120, 120, 120).stroke_width(2),
        )))?;
        by += 20;
    }
    centered(
        root,
        (bar_x, 508),
        &format!("bar ±{:.2}", nums::SUBSTRATE_BAR),
        12,
        RGBColor(100, 100, 100),
    )?;

    // Ban 2. Both readings, and the status of the second stated on its face.
    map_draw(root.draw(&Rectangle::new(
        [(36, 664), (1364, 752)],
        RGBColor(253, 246, 224).filled(),
    )))?;
    map_draw(root.draw(&Rectangle::new(
        [(36, 664), (1364, 752)],
        RGBColor(200, 160, 60).stroke_width(2),
    )))?;
    label(
        root,
        (48, 674),
        &format!(
            "BOTH READINGS OF “THE GAIN DOUBLES”, because neither may be quoted alone. rec+alif starts 0.18 LOWER, with {:.4} of headroom against {:.4}.",
            nums::HEADROOM_REC,
            nums::HEADROOM_FF
        ),
        14,
        BLACK,
    )?;
    label(
        root,
        (48, 698),
        &format!(
            "Gain / headroom is {:.3} against {:.3}, so the ratio falls {:.1}× → {:.2}×. The ORDERING SURVIVES; most of its apparent size does not.",
            nums::HEADROOM_RATIO_REC,
            nums::HEADROOM_RATIO_FF,
            nums::RAW_RATIO,
            nums::NORMALISED_RATIO
        ),
        13,
        RGBColor(70, 70, 70),
    )?;
    label(
        root,
        (48, 722),
        "That normalisation is POST-HOC AND NOT REGISTERED, and saying so is not licence to prefer whichever of the two numbers is more convenient, in either direction.",
        12,
        RGBColor(120, 120, 120),
    )?;

    // --- Panel C ------------------------------------------------------------
    label(
        root,
        (36, 782),
        "Panel C — usability, which had to be measured before any row of Panel A could be. This is why the recurrent row exists at one scale only.",
        15,
        BLACK,
    )?;
    label(root, (36, 812), "arm", 12, RGBColor(110, 110, 110))?;
    label(root, (230, 812), "scale", 12, RGBColor(110, 110, 110))?;
    label(root, (330, 812), "completed", 12, RGBColor(110, 110, 110))?;
    label(root, (460, 812), "voided", 12, RGBColor(110, 110, 110))?;
    label(root, (570, 812), "diverged", 12, RGBColor(110, 110, 110))?;
    for (i, (arm, scale, completed, voided, diverged)) in nums::USABILITY.into_iter().enumerate()
    {
        let y = 836 + (i as i32) * 24;
        label(root, (36, y), arm, 13, BLACK)?;
        label(root, (230, y), &format!("{scale:.1}"), 13, BLACK)?;
        label(root, (330, y), &format!("{completed} / 12"), 13, BLACK)?;
        label(
            root,
            (460, y),
            &format!("{voided}"),
            13,
            if voided > 0 { Verdict::Fail.border() } else { BLACK },
        )?;
        label(
            root,
            (570, y),
            &format!("{diverged}"),
            13,
            if diverged > 0 { Verdict::Fail.border() } else { BLACK },
        )?;
    }
    label(
        root,
        (700, 836),
        &format!(
            "rec+fixed's ten voids are ALL SATURATION — up to {:.1}% of hidden units pinned at",
            nums::SATURATED_FRACTION_MAX * 100.0
        ),
        13,
        RGBColor(70, 70, 70),
    )?;
    label(
        root,
        (700, 858),
        "maximum firing — and NONE is divergence at scale 0.4. So on the recurrent substrate",
        13,
        RGBColor(70, 70, 70),
    )?;
    label(
        root,
        (700, 880),
        "ADAPTATION IS STABILISING, the opposite of the sign that wave's own hypothesis asserted.",
        13,
        BLACK,
    )?;
    label(
        root,
        (700, 906),
        "A-1 above is therefore inert AT THIS OPERATING POINT, on this substrate — not in general.",
        12,
        RGBColor(120, 120, 120),
    )?;

    // Ban 3, last line, where a reader leaves the figure.
    label(
        root,
        (36, 948),
        // The count comes from the row it describes rather than from the
        // sentence: "TEN" and an array that says 9 would disagree silently.
        &format!(
            "TEN PAIRS ({} of 12), THE REGISTERED MINIMUM. The two recurrent arms lost different seeds and one further loss on either would have made M-2 unreportable. Pairing REDUCES",
            nums::SUBSTRATE[2].3
        ),
        13,
        RGBColor(120, 120, 120),
    )?;
    label(
        root,
        (36, 970),
        "survivorship rather than removing it: the surviving pairs are those that did not diverge, and divergence is not random. The feed-forward rows carry no such exposure at 12/12.",
        13,
        RGBColor(120, 120, 120),
    )?;
    Ok(())
}

/// The lead program's graphical abstract.
///
/// # What was open here, and why it was not a missing number
///
/// This was `TODO(source needed)` from 2026-08-27 and
/// `VENUE_FORMATTING.md` called it "the last *unspecified* piece of figure work
/// in the package". Every quantity it draws already existed; what was missing
/// was a decision about what the paper's front image says, and that decision is
/// written into `PAPER_FIGURE_SPEC.md` §"Graphical abstract — lead program"
/// rather than into this function, so it can be argued with.
///
/// # Four bans, three of them Figure 1's
///
/// An abstract is a compression of the figure, not a licence to say something
/// the figure may not, so bans 1-3 carry over unchanged: not "SHD is temporal"
/// (prior art, named on the image, rate arm at equal weight), not an ablation
/// (the shuffle is done to the DATA, in both splits), and never +0.1577.
///
/// The fourth is this figure's own and is the reason the headline accuracy is
/// **not** the largest number on it. rho = -0.1430 against a bar of +0.829: the
/// difference-in-differences is not the gain and not a share of it, so there is
/// no 0.8332-to-shuffled arrow and no percentage-of-gain framing. The one
/// percentage drawn is the 94.5% collapse of the *advantage*, which is a
/// different quantity and is labelled as one.
///
/// Two disclosures the spec requires because an abstract travels alone: 0.8332
/// is **not competitive** against a 95-96.4% frontier, and the gain **inverts
/// at h1024**. Without them the image reads as a results claim about SHD.
fn draw_lead_graphical_abstract(
    root: &DrawingArea<SVGBackend<'_>, Shift>,
) -> Result<(), DrawErr> {
    label(
        root,
        (36, 16),
        "What a time-axis read-out buys is temporal order",
        30,
        BLACK,
    )?;
    label(
        root,
        (36, 56),
        "A difference-in-differences on the GAIN, on SHD. Seed-paired, n = 32 at the anchor, h128 / published-2ms / d32-L4 / e400.",
        14,
        RGBColor(70, 70, 70),
    )?;

    // Ban 1, first and largest, because it is the reading the image is least
    // entitled to and the one a reader arrives with.
    map_draw(root.draw(&Rectangle::new(
        [(36, 88), (1364, 162)],
        RGBColor(245, 245, 248).filled(),
    )))?;
    label(
        root,
        (48, 96),
        "NOT SHOWN HERE, AND NOT CLAIMED: that SHD depends on temporal order. That is established — Cramer et al. 2022 could not exceed 60% on spike-count-only SHD;",
        13,
        RGBColor(90, 90, 90),
    )?;
    label(
        root,
        (48, 118),
        "the Neuromorphic Sequential Arena removes temporal processing model-side and reports 86.48 → 68.51; Yu et al. 2025 randomise spike times at fixed counts.",
        13,
        RGBColor(90, 90, 90),
    )?;
    label(
        root,
        (48, 140),
        "Three independent destruction operators, one conclusion, all of it prior to this work. What is measured here is WHICH COMPONENT'S contribution is the order-dependent one.",
        13,
        RGBColor(60, 60, 60),
    )?;

    // --- 1. the operation, on the data --------------------------------------
    box_card(
        root,
        (36, 216, 356, 496),
        RGBColor(246, 243, 236),
        RGBColor(180, 150, 70),
        "The operation — on the INPUT",
        &[
            "permute time bins,",
            "independently per sample,",
            "in BOTH the training and",
            "test splits",
            "",
            "the task becomes",
            "rate-solvable",
        ],
    )?;
    label(
        root,
        (36, 510),
        "Nothing is removed from the model.",
        13,
        RGBColor(60, 60, 60),
    )?;
    label(
        root,
        (36, 532),
        "This is not an ablation of the read-out,",
        12,
        RGBColor(110, 110, 110),
    )?;
    label(
        root,
        (36, 552),
        "and no arrow here points at a component.",
        12,
        RGBColor(110, 110, 110),
    )?;

    // --- 2. the pair of costs, the centre ------------------------------------
    label(
        root,
        (410, 196),
        "Each arm's OWN shuffle cost — the pair is the measurement",
        16,
        BLACK,
    )?;
    let base = 496;
    let height = 200;
    let scale = 0.16;
    cost_bar(
        root,
        (440, base, 150, height),
        scale,
        nums::SHUFFLE_COST_ATTN_32,
        RGBColor(70, 100, 180),
        "time-axis attention",
        "d32 / L4 read-out",
        "32 / 32 seed pairs positive",
    )?;
    cost_bar(
        root,
        (660, base, 150, height),
        scale,
        nums::SHUFFLE_COST_RATE_32,
        RGBColor(200, 140, 60),
        "rate read-out",
        "ff+fixed — half the measurement,",
        "not a faint control",
    )?;
    centered(
        root,
        (625, base - 80),
        &format!("{:.1}×", nums::SHUFFLE_COST_RATIO_32),
        30,
        BLACK,
    )?;
    centered(
        root,
        (625, base + 94),
        &format!(
            "The read-out's advantage over the rate arm falls +{:.4} → +{:.4}: {:.1}% of it is contingent on temporal order.",
            nums::ADVANTAGE_INTACT_32, nums::ADVANTAGE_SHUFFLED_32, nums::CONTINGENT_PCT_32
        ),
        13,
        RGBColor(70, 70, 70),
    )?;
    centered(
        root,
        (625, base + 116),
        "That is the collapse of the ADVANTAGE, which is not the gain and not a share of it — see the strip below.",
        12,
        RGBColor(120, 120, 120),
    )?;

    // --- 3. coverage ---------------------------------------------------------
    box_card(
        root,
        (900, 216, 1364, 496),
        RGBColor(233, 248, 237),
        RGBColor(90, 170, 110),
        "Measured across the design space",
        &[
            &format!(
                "{} of {} operating points carry",
                nums::COVERAGE_COVERED,
                nums::COVERAGE_TOTAL
            ),
            "the control, and EVERY ONE",
            &format!("clears its +{:.2} bar", nums::DID_BAR),
            "",
            "widths 128 → 1024,",
            "both contracts, both geometries",
            "",
            &format!(
                "{} points carry intact arms with",
                nums::COVERAGE_TOTAL - nums::COVERAGE_COVERED
            ),
            "no shuffled twin, and claim nothing",
        ],
    )?;

    // --- ban 4: the DiD is not the gain -------------------------------------
    map_draw(root.draw(&Rectangle::new(
        [(36, 634), (1364, 716)],
        RGBColor(253, 246, 224).filled(),
    )))?;
    map_draw(root.draw(&Rectangle::new(
        [(36, 634), (1364, 716)],
        RGBColor(200, 160, 60).stroke_width(2),
    )))?;
    label(
        root,
        (48, 644),
        &format!(
            "THE EFFECT'S SIZE IS NOT THE GAIN. Spearman ρ between the six per-width gains and their difference-in-differences is {:.4}, against a registered bar of +{:.3}.",
            nums::DID_RHO, nums::DID_RHO_BAR
        ),
        14,
        BLACK,
    )?;
    label(
        root,
        (48, 668),
        &format!(
            "h768 carries the SMALLEST positive gain on the ladder (+{:.4}) and the LARGEST difference-in-differences in the campaign (+{:.4}).",
            nums::GAIN_H768, nums::DID_H768
        ),
        13,
        RGBColor(90, 90, 90),
    )?;
    label(
        root,
        (48, 690),
        "The claim is that the read-out's contribution IS order-dependent. It is NOT that the gain decomposes into an order-dependent share and a remainder.",
        13,
        RGBColor(90, 90, 90),
    )?;

    // --- the two disclosures an abstract cannot travel without ---------------
    label(
        root,
        (36, 734),
        &format!(
            "Accuracy, for scale and no more: the read-out reaches {:.4} against the rate arm's {:.4}. It is NOT COMPETITIVE — the SHD frontier is {:.0}–{:.1}% via learned delays,",
            nums::HEAD_ATTN_32,
            nums::HEAD_RATE_32,
            nums::FIELD_FRONTIER_LO * 100.0,
            nums::FIELD_FRONTIER_HI * 100.0
        ),
        13,
        RGBColor(70, 70, 70),
    )?;
    label(
        root,
        (36, 756),
        "adaptation and spiking transformers, and this instrument carries no temporal kernel of any kind. No accuracy claim is made and none is drawn larger than the costs above.",
        13,
        RGBColor(70, 70, 70),
    )?;
    label(
        root,
        (36, 786),
        &format!(
            "SCOPE, on the image because an abstract travels alone: the gain INVERTS at width h1024 ({:.4} at d32/L4), the collapse there is LOCATED BUT UNEXPLAINED, and 0.80",
            nums::LADDER[5].3
        ),
        13,
        RGBColor(120, 120, 120),
    )?;
    label(
        root,
        (36, 808),
        "clearance is geometry-specific. Three preregistered rescue levers at h1024 all failed and every one is worse than the arm it was meant to rescue.",
        13,
        RGBColor(120, 120, 120),
    )?;
    label(
        root,
        (36, 846),
        "No biology, no cortex, no neuromorphic-hardware claim, and no claim that attention makes a network temporal.",
        12,
        RGBColor(140, 140, 140),
    )?;
    Ok(())
}

/// Figure 6 of the secondary program — the matched means, by verdict.
///
/// # Why this function did not exist until 2026-08-29
///
/// `fig2_matched_means.{png,pdf}` has been on disk since 24 July and **nothing
/// generated it**. It plots the value block `PAPER_FIGURE_SPEC.md` names as
/// "superseded and not for drawing" — DFA 0.9387, RL 0.9200, a gradient ceiling
/// at 0.8963 — pre-repair figures from a forward pass that emitted zero spikes
/// at any seed. Figure M, Figure 5, Figure 7 and the graphical abstract were
/// brought current by re-running this binary on 2026-08-27; this one could not
/// be, because it had no owner. It is authored here rather than re-run.
///
/// # Four things the spec forbids, and where each is handled
///
/// 1. **No ranking of the passing arms.** With the reference at 1.0000 every
///    PASS reduces to "above 0.75". Nothing here maps accuracy to a length, a
///    position or a sort: the arms are cards grouped by verdict, in the row
///    order of Table A, and the panel says so on its face.
/// 2. **The contrasts are not a fourth verdict.** They were measured and not
///    gated, so they get their own labelled group rather than a pass colour
///    (which would make six passing arms out of three) or a fail colour (which
///    would turn the 0.9975 disclosure into evidence for the FAIL it qualifies).
/// 3. **The recurrent column never goes missing.** Every card carries
///    `ff / rec`, because the lead FAIL is a FAIL on BOTH graphs at n = 20 and
///    that is the claim's strength. Panel B draws the broadcast gap LCB at
///    −0.0192 with its sign rather than clipping it at zero.
/// 4. **The gate is not one bar.** Panel A is the accuracy half and Panel B the
///    gap-LCB half, each with its own threshold drawn, because "cleared the
///    floor" and "cleared the gate" are different sentences everywhere else in
///    this package.
fn draw_fig6_matched_means(root: &DrawingArea<SVGBackend<'_>, Shift>) -> Result<(), DrawErr> {
    label(root, (36, 16), "Figure 6 — Matched means, by verdict", 26, BLACK)?;
    label(
        root,
        (36, 50),
        "Matched dense-LIF forward, n = 20 per arm, 2026-08-25 re-run at MATCHED_INPUT_SCALE = 2.0. Every value is feed-forward / recurrent. Gate: primary mean ≥ 0.65 AND gap LCB > 0.5.",
        13,
        RGBColor(70, 70, 70),
    )?;

    // The reference leads, as it does in Figure M, because it is the reason the
    // passing arms cannot be ordered rather than an aside about the ceiling.
    let reference = Verdict::Reference;
    map_draw(root.draw(&Rectangle::new(
        [(36, 74), (1364, 112)],
        reference.fill().filled(),
    )))?;
    map_draw(root.draw(&Rectangle::new(
        [(36, 74), (1364, 112)],
        reference.border().stroke_width(2),
    )))?;
    label(
        root,
        (48, 84),
        &format!(
            "SuperSpike BPTT reference saturates at {:.4} ff / {:.4} rec. Against it every PASS below reduces to “the arm scored above 0.75”, so this figure does not rank them: rows are in the order of Table A.",
            nums::CEILING.0,
            nums::CEILING.1
        ),
        13,
        BLACK,
    )?;

    label(
        root,
        (36, 128),
        "Panel A — primary mean, grouped by verdict. Accuracy is printed, never encoded as a length or a position.",
        15,
        BLACK,
    )?;

    // Grouped by verdict, and the contrast group is labelled as ungated in the
    // group header itself rather than only in the chip.
    let groups: [VerdictGroup<'_>; 4] = [
        (
            Verdict::Fail,
            "Does not clear the gate — the lead negative, and the only one",
            &[(
                "broadcast ±1 three-factor",
                "MatchedLocal — ±1 reward × surrogate eligibility. At chance on both graphs.",
                nums::BROADCAST_PM1,
            )],
        ),
        (
            Verdict::Pass,
            "Clears the gate — unordered, and not orderable against a saturated reference",
            &[
                (
                    "graded DFA",
                    "MatchedDfaGradedError — graded error through fixed random feedback",
                    nums::DFA,
                ),
                (
                    "REINFORCE × frozen B_i",
                    "MatchedRlReinforceFb — the same ±1 reward, addressed",
                    nums::RL_FB,
                ),
                (
                    "discrete EventProp-style spike-adjoint",
                    "MatchedEventProp — the archived FAIL is WITHDRAWN; it had no spikes to differentiate through",
                    nums::EVENTPROP,
                ),
            ],
        ),
        (
            Verdict::Contrast,
            "Measured and NOT gated — neither passes to be counted nor failures to be explained away",
            &[
                (
                    "broadcast graded error",
                    "MatchedBroadcastGradedError — why the lead negative says “±1 three-factor” and not “any broadcast”",
                    nums::BROADCAST_GRADED,
                ),
                (
                    "RL graded-reward broadcast",
                    "MatchedRlGraded — richness without addressing",
                    nums::RL_GRADED,
                ),
                (
                    "RL ±1 broadcast",
                    "MatchedRlFlat — same reward and topology as the lead FAIL, 0.28 above it",
                    nums::RL_FLAT,
                ),
            ],
        ),
        (
            Verdict::Reference,
            "The ceiling every arm above is measured against",
            &[(
                "SuperSpike BPTT",
                "saturates in every suite, on both forward graphs",
                nums::CEILING,
            )],
        ),
    ];

    // `rule_card` writes its value line at y0 + 66 in a 15px face, so a card
    // shorter than 92 puts that line on its own bottom border. It did, at 76.
    const CARD_H: i32 = 92;
    let mut y = 152;
    for (verdict, heading, arms) in groups {
        label(root, (36, y), heading, 13, verdict.border())?;
        y += 22;
        for (rule, mechanism, value) in arms {
            rule_card(root, (36, y, 1364, y + CARD_H), verdict, rule, mechanism, *value)?;
            y += CARD_H + 8;
        }
        y += 14;
    }

    // --- Panel B — the gate quantity ---------------------------------------
    //
    // Every coordinate below is derived from where the group loop ENDED, not
    // written down. The 2026-08-29 lead-figure pass lost a line to exactly this:
    // a shift moved the literal label positions and left a computed base behind,
    // and the panel it belonged to landed on top of the row above it.
    let panel_b = y + 20;
    label(
        root,
        (36, panel_b),
        "Panel B — gap LCB against the > 0.5 gate. Same row order; the second half of the gate, drawn because clearing the floor and clearing the gate are different sentences.",
        15,
        BLACK,
    )?;

    let (lcb_lo, lcb_hi) = (-0.25, 1.05);
    let (lx0, lx1) = (470, 1330);
    let at_lcb = |v: f64| {
        lx0 + (((v - lcb_lo) / (lcb_hi - lcb_lo)) * (lx1 - lx0) as f64).round() as i32
    };

    // Zero, then the gate. Zero is drawn because the broadcast recurrent value
    // is below it and a reader needs to see which side of nothing it is on.
    let strip_top = panel_b + 48;
    let strip_bottom = strip_top + 4 * 34;
    let zero = at_lcb(0.0);
    map_draw(root.draw(&PathElement::new(
        vec![(zero, strip_top), (zero, strip_bottom)],
        RGBColor(170, 170, 176).stroke_width(1),
    )))?;
    label(root, (zero - 8, strip_bottom + 4), "0", 12, RGBColor(140, 140, 140))?;
    let gate = at_lcb(nums::GATE_LCB);
    let mut gy = strip_top;
    while gy < strip_bottom {
        map_draw(root.draw(&PathElement::new(
            vec![(gate, gy), (gate, (gy + 12).min(strip_bottom))],
            RGBColor(120, 120, 120).stroke_width(2),
        )))?;
        gy += 24;
    }
    centered(
        root,
        (gate, strip_top - 14),
        &format!("gate: gap LCB > {:.1}", nums::GATE_LCB),
        13,
        RGBColor(100, 100, 100),
    )?;

    for (i, (arm, ff, rec)) in nums::GAP_LCB.into_iter().enumerate() {
        let y = strip_top + 22 + (i as i32) * 34;
        let verdict = if ff > nums::GATE_LCB {
            Verdict::Pass
        } else {
            Verdict::Fail
        };
        label(root, (36, y - 8), arm, 14, BLACK)?;
        map_draw(root.draw(&PathElement::new(
            vec![(at_lcb(ff.min(rec)), y), (at_lcb(ff.max(rec)), y)],
            verdict.border().stroke_width(2),
        )))?;
        map_draw(root.draw(&Circle::new(
            (at_lcb(ff), y),
            7,
            verdict.border().filled(),
        )))?;
        map_draw(root.draw(&Circle::new((at_lcb(rec), y), 7, verdict.border().stroke_width(2))))?;
        let (left, right) = if ff <= rec { (ff, rec) } else { (rec, ff) };
        label(
            root,
            (at_lcb(left) - 66, y - 8),
            &format!("{left:.4}"),
            12,
            RGBColor(80, 80, 80),
        )?;
        label(
            root,
            (at_lcb(right) + 14, y - 8),
            &format!("{right:.4}"),
            12,
            RGBColor(80, 80, 80),
        )?;
    }
    label(
        root,
        (940, strip_bottom + 28),
        "● feed-forward   ○ recurrent",
        12,
        RGBColor(110, 110, 110),
    )?;
    label(
        root,
        (36, strip_bottom + 28),
        &format!(
            "Broadcast ±1 is BELOW ZERO on the recurrent graph ({:.4}), not merely short of the gate.",
            nums::GAP_LCB[0].2
        ),
        12,
        RGBColor(120, 120, 120),
    )?;
    label(
        root,
        (36, strip_bottom + 52),
        &format!(
            "Disclose with this figure: the broadcast-GRADED contrast reaches {:.4} and ±1 broadcast REINFORCE reaches {:.4}. Neither is a PASS and neither erases the ±1 × eligibility FAIL.",
            nums::BROADCAST_GRADED.0,
            nums::RL_FLAT.0
        ),
        12,
        RGBColor(120, 120, 120),
    )?;
    Ok(())
}

fn draw_fig3(root: &DrawingArea<SVGBackend<'_>, Shift>) -> Result<(), DrawErr> {
    label(
        root,
        (40, 24),
        "Figure 7 — Engine C1 condition means",
        26,
        BLACK,
    )?;
    label(
        root,
        (40, 58),
        "Hash c1-118207fbc3eaba53 · G2 FAIL · PC (positive-control proxy) disclosed",
        14,
        RGBColor(60, 60, 60),
    )?;

    draw_bar_row(
        root,
        80,
        140,
        940,
        520,
        Some(0.5),
        &[
            ("local-assembly", nums::C1_LOCAL, RGBColor(220, 90, 90)),
            ("dense-local", nums::C1_DENSE, RGBColor(200, 140, 80)),
            ("gradient-ref", nums::C1_GRAD, RGBColor(90, 120, 200)),
            ("eligibility-ref", nums::C1_ELIG, RGBColor(90, 170, 110)),
        ],
    )?;

    box_card(
        root,
        (1120, 160, 1360, 420),
        RGBColor(255, 250, 230),
        RGBColor(180, 140, 40),
        "Integrity callouts",
        &[
            &format!("PC ≈ {:.4}", nums::C1_PC),
            "H1 sticky last_spike",
            "H2 partial reset",
            "θ=∞ mute",
            "project unused on v2",
        ],
    )?;
    Ok(())
}

/// Figure 8 of the secondary program — the transfer ladder.
///
/// # Why this function did not exist until 2026-08-29
///
/// Like Figure 6, `fig4_transfer_ladder.{png,pdf}` had no generator and was
/// stale at rung 1: it drew matched RL at 0.9200 with a gap LCB of 0.6846, the
/// superseded pre-repair block. The current figures are 0.9950 ff / 0.9812 rec
/// and 0.9765 / 0.9079. Rungs 2-4 are **unaffected** by the matched re-run —
/// none of them runs on the matched dense-LIF forward — which is also the whole
/// point of ban 1.
///
/// # Three things the spec forbids, and where each is handled
///
/// 1. **Not one substrate.** Rung 1 is the matched dense-LIF forward; rungs 2-4
///    are the live event-driven muted-θ / k-WTA engine. Rung 1 is a card with
///    its own values; the axes below it carry only live-engine arms, and the
///    break between them is drawn and named. Nothing connects across it.
/// 2. **Not accuracy alone.** Two axes, each with its own gate: acc ≥ 0.65 and
///    gap LCB > 0.5. On accuracy alone v15, v18 and v20 read as near-misses of
///    one bar; the best gap LCB anywhere below rung 1 is v17's 0.3127.
/// 3. **No ranking of v14-v24.** They are a sequential exploratory family with
///    no family-wise claim, so they are drawn in protocol order and the only
///    two called out are the two the spec names as landmarks.
fn draw_fig8_transfer_ladder(root: &DrawingArea<SVGBackend<'_>, Shift>) -> Result<(), DrawErr> {
    label(root, (36, 16), "Figure 8 — The transfer ladder", 26, BLACK)?;
    label(
        root,
        (36, 50),
        "A rule that clears the matched gate does not clear it on the live engine. Gate: local acc ≥ 0.65 AND gap LCB > 0.5 — both, which is why both axes are drawn.",
        13,
        RGBColor(70, 70, 70),
    )?;

    // --- Rung 1: a different substrate, and therefore a card and not a point.
    label(
        root,
        (36, 88),
        "Rung 1 — matched dense-LIF forward, n = 20",
        14,
        Verdict::Pass.border(),
    )?;
    // 92, not 76: `rule_card` writes its value line at y0 + 66 in a 15px face
    // and a shorter card puts that line on its own bottom border.
    rule_card(
        root,
        (36, 110, 1364, 202),
        Verdict::Pass,
        "matched REINFORCE × frozen B_i (v12)",
        "gap LCB 0.9765 ff / 0.9079 rec — clears both halves of the gate on both forward graphs",
        nums::RL_FB,
    )?;

    // --- The break. Everything below runs on a different engine.
    map_draw(root.draw(&Rectangle::new(
        [(36, 222), (1364, 284)],
        RGBColor(246, 243, 236).filled(),
    )))?;
    map_draw(root.draw(&Rectangle::new(
        [(36, 222), (1364, 284)],
        RGBColor(180, 150, 70).stroke_width(2),
    )))?;
    label(
        root,
        (48, 232),
        "THE SUBSTRATE CHANGES HERE. Everything below runs on the live event-driven muted-θ / k-WTA engine, not on the matched dense-LIF forward above.",
        14,
        BLACK,
    )?;
    label(
        root,
        (48, 256),
        "This ladder is NOT one system at twelve settings, and nothing is drawn across the break. Rungs 2-4 are unaffected by the 2026-08-25 matched re-run: none of them runs on that forward.",
        12,
        RGBColor(110, 110, 110),
    )?;

    // --- Rungs 2-4, on two axes, one substrate ------------------------------
    let (acc_lo, acc_hi) = (0.40, 0.80);
    let (ax0, ax1) = (400, 800);
    let at_acc =
        |v: f64| ax0 + (((v - acc_lo) / (acc_hi - acc_lo)) * (ax1 - ax0) as f64).round() as i32;
    let (lcb_lo, lcb_hi) = (-0.05, 0.55);
    let (bx0, bx1) = (940, 1340);
    let at_lcb =
        |v: f64| bx0 + (((v - lcb_lo) / (lcb_hi - lcb_lo)) * (bx1 - bx0) as f64).round() as i32;

    let (top, bottom) = (338, 808);
    centered(root, ((ax0 + ax1) / 2, 308), "local accuracy", 15, BLACK)?;
    centered(root, ((bx0 + bx1) / 2, 308), "gap LCB", 15, BLACK)?;

    for (x, level, caption) in [
        (at_acc(nums::GATE_FLOOR), nums::GATE_FLOOR, "floor 0.65"),
        (at_lcb(nums::GATE_LCB), nums::GATE_LCB, "gate 0.5"),
    ] {
        let _ = level;
        let mut y = top - 6;
        while y < bottom {
            map_draw(root.draw(&PathElement::new(
                vec![(x, y), (x, (y + 12).min(bottom))],
                RGBColor(120, 120, 120).stroke_width(2),
            )))?;
            y += 24;
        }
        centered(root, (x, 328), caption, 12, RGBColor(100, 100, 100))?;
    }

    // One row per protocol, in protocol order. `at_acc`/`at_lcb` place the two
    // dots; nothing is sorted and nothing is joined between rows.
    let mut y = top + 18;
    let row = |root: &DrawingArea<SVGBackend<'_>, Shift>,
                   y: i32,
                   name: &str,
                   local: f64,
                   lcb: f64|
     -> Result<(), DrawErr> {
        // Colour is the GATE outcome, and every one of these is a FAIL. The
        // floor-cleared arms are marked separately so "cleared the floor" stays
        // visible without being mistaken for "cleared the gate".
        let cleared_floor = local >= nums::GATE_FLOOR;
        label(root, (36, y - 9), name, 14, BLACK)?;
        map_draw(root.draw(&Circle::new(
            (at_acc(local), y),
            7,
            if cleared_floor {
                RGBColor(200, 160, 60).filled()
            } else {
                Verdict::Fail.border().filled()
            },
        )))?;
        label(
            root,
            (at_acc(local) + 14, y - 8),
            &format!("{local:.4}"),
            12,
            RGBColor(80, 80, 80),
        )?;
        map_draw(root.draw(&Circle::new(
            (at_lcb(lcb), y),
            7,
            Verdict::Fail.border().filled(),
        )))?;
        label(
            root,
            (at_lcb(lcb) + 14, y - 8),
            &format!("{lcb:.4}"),
            12,
            RGBColor(80, 80, 80),
        )?;
        label(root, (1352, y - 8), "FAIL", 12, Verdict::Fail.border())?;
        Ok(())
    };

    label(root, (36, y - 30), "Rung 2 — live REINFORCE transfer", 13, RGBColor(90, 90, 90))?;
    row(root, y, "v13 live RFB", nums::LIVE_RFB, nums::LIVE_RFB_LCB)?;
    y += 46;

    label(
        root,
        (36, y - 22),
        "Rung 3 — gap-close family, in protocol order (v14–v19)",
        13,
        RGBColor(90, 90, 90),
    )?;
    y += 4;
    for (name, local, lcb) in nums::GAP_CLOSE {
        row(root, y, name, local, lcb)?;
        y += 32;
    }

    y += 16;
    label(
        root,
        (36, y - 22),
        "Rung 4 — break-it family, in protocol order (v20–v24)",
        13,
        RGBColor(90, 90, 90),
    )?;
    y += 4;
    for (name, local, lcb) in nums::BREAK_IT {
        row(root, y, name, local, lcb)?;
        y += 32;
    }

    // Drawn as circles rather than written as "●", which was the first version
    // and carried no colour at all: the whole distinction the legend exists for
    // is the fill, and a text bullet renders in the label's own grey.
    let legend_y = bottom + 28;
    map_draw(root.draw(&Circle::new((400, legend_y), 7, RGBColor(200, 160, 60).filled())))?;
    label(root, (414, legend_y - 8), "cleared the 0.65 accuracy floor", 12, RGBColor(110, 110, 110))?;
    map_draw(root.draw(&Circle::new(
        (660, legend_y),
        7,
        Verdict::Fail.border().filled(),
    )))?;
    label(root, (674, legend_y - 8), "did not", 12, RGBColor(110, 110, 110))?;
    label(
        root,
        (880, legend_y - 8),
        "Every gap-LCB dot is a FAIL, so that column carries no second colour.",
        12,
        RGBColor(110, 110, 110),
    )?;
    label(
        root,
        (36, legend_y + 30),
        // Counted, never written down. A literal "six" here would go on reading
        // "six" if an arm were added, corrected or withdrawn from either array
        // -- and the arrays are the same ones the dots come from, so the prose
        // and the picture would then disagree with nothing to notice it.
        &format!(
            "EVERY ARM BELOW THE BREAK FAILS THE GATE. {} of {} clear the {:.2} accuracy floor; the best gap LCB anywhere below rung 1 is v17's {:.4}, against a bar of {:.1}.",
            std::iter::once(nums::LIVE_RFB)
                .chain(nums::GAP_CLOSE.iter().map(|(_, local, _)| *local))
                .chain(nums::BREAK_IT.iter().map(|(_, local, _)| *local))
                .filter(|local| *local >= nums::GATE_FLOOR)
                .count(),
            1 + nums::GAP_CLOSE.len() + nums::BREAK_IT.len(),
            nums::GATE_FLOOR,
            nums::BEST_LCB_ANYWHERE,
            nums::GATE_LCB
        ),
        15,
        BLACK,
    )?;
    label(
        root,
        (36, legend_y + 58),
        &format!(
            "Floor cleared is not gate cleared, and the two axes exist so that cannot be read off one. The best local below the break is v15's {:.4} — on accuracy alone it looks like a near miss.",
            nums::BEST_LOCAL_GAP_CLOSE
        ),
        13,
        RGBColor(90, 90, 90),
    )?;
    label(
        root,
        (36, legend_y + 94),
        "NOT A RANKING. v14–v24 are a SEQUENTIAL EXPLORATORY family: each new hypothesis minted a new protocol version and hash, and there is no multiplicity-corrected family-wise claim over them.",
        12,
        RGBColor(120, 120, 120),
    )?;
    label(
        root,
        (36, legend_y + 118),
        "v15 and v17 are named as landmarks because the specification names them, not as winners. Rows are in protocol order and are never sorted by either quantity.",
        12,
        RGBColor(120, 120, 120),
    )?;
    label(
        root,
        (36, legend_y + 154),
        "NO MECHANISM. Four suspects for the transfer gap — sticky last_spike, partial membrane reset, θ=∞ muting, hard k-WTA — have never been tested individually, and this figure does not attribute the gap to any of them.",
        12,
        RGBColor(120, 120, 120),
    )?;
    Ok(())
}

fn draw_graphical_abstract(root: &DrawingArea<SVGBackend<'_>, Shift>) -> Result<(), DrawErr> {
    label(root, (40, 20), "BINN — graphical abstract", 26, BLACK)?;
    label(
        root,
        (40, 54),
        "Same forward → ±1 × surrogate eligibility fails; every other rule tested passes against a reference at 1.0000; live k-WTA transfer fails.",
        14,
        RGBColor(60, 60, 60),
    )?;
    // Scope, because this abstract depicts the SECONDARY program. The
    // manuscript leads with the SHD read-out and has no graphical abstract of
    // its own yet -- an open authoring task, recorded in PAPER_FIGURE_SPEC.md
    // rather than papered over here.
    label(
        root,
        (40, 78),
        "Secondary program (matched-architecture kill gate). The lead SHD read-out program has no graphical abstract specified.",
        12,
        RGBColor(120, 120, 120),
    )?;

    box_card(
        root,
        (36, 130, 300, 640),
        RGBColor(235, 240, 255),
        RGBColor(70, 100, 180),
        "Dense-LIF forward",
        &[
            "coincidence task",
            "shared architecture",
            "wrec = 0 and wrec live",
            "",
            &format!("BPTT reference {:.4}", nums::CEILING.0),
        ],
    )?;

    let rules = [
        (
            Verdict::Fail,
            "±1 × surrogate eligibility",
            "MatchedLocal — scalar reward, no addressing",
            nums::BROADCAST_PM1,
        ),
        (
            Verdict::Pass,
            "graded error × DFA",
            "MatchedDfaGradedError — richer error, addressed",
            nums::DFA,
        ),
        (
            Verdict::Pass,
            "REINFORCE × frozen B_i",
            "MatchedRlReinforceFb — same ±1 reward, addressed",
            nums::RL_FB,
        ),
    ];
    for (i, (verdict, title, mechanism, value)) in rules.into_iter().enumerate() {
        let y = 130 + (i as i32) * 170;
        rule_card(root, (330, y, 970, y + 150), verdict, title, mechanism, value)?;
    }
    // The passes are stacked, not ranked: with the reference saturated there is
    // no ordering among them to draw.
    label(
        root,
        (330, 648),
        "The two passes are not ordered: against a reference at 1.0000 each reduces to “above 0.75”.",
        12,
        RGBColor(110, 110, 110),
    )?;

    map_draw(root.draw(&PathElement::new(
        vec![(1000, 385), (1090, 385)],
        BLACK.stroke_width(3),
    )))?;
    centered(root, (1045, 356), "transfer →", 14, BLACK)?;
    centered(
        root,
        (1045, 412),
        "live muted-θ / k-WTA",
        12,
        RGBColor(80, 80, 80),
    )?;

    box_card(
        root,
        (1110, 130, 1364, 640),
        RGBColor(253, 234, 234),
        RGBColor(220, 90, 90),
        "Live RFB → G2 FAIL",
        &[
            &format!("acc {:.4}", nums::LIVE_RFB),
            "gap-close still < 0.5",
            "v13–v24 package",
            "",
            "structured B clears the",
            "accuracy floor only",
        ],
    )?;

    label(
        root,
        (36, 700),
        &format!(
            "Disclose (Figure M): a broadcast-GRADED rule reaches {:.4}, and ±1 broadcast REINFORCE reaches {:.4}. Neither erases the ±1 × eligibility FAIL, and neither is a PASS.",
            nums::BROADCAST_GRADED.0,
            nums::RL_FLAT.0
        ),
        14,
        RGBColor(70, 70, 70),
    )?;
    label(
        root,
        (36, 730),
        "Matched PASS does not imply live muted-θ / k-WTA G2 PASS. No claim of biology, Assembly Calculus, impossibility, or neuromorphic hardware.",
        13,
        RGBColor(110, 110, 110),
    )?;
    label(
        root,
        (36, 760),
        "Avoid: brain icons, “solved,” Assembly Calculus branding, bare “broadcast credit topology.”",
        12,
        RGBColor(140, 140, 140),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn generates_required_stems() {
        let dir = temp_dir().join(format!("binn_paper_figs_{}", std::process::id()));
        let written = generate_all(&dir).expect("generate");
        assert!(written.len() >= 12);
        for stem in [
            "figS_substrate",
            "lead_graphical_abstract",
            "figM_mechanism_richness_addressability",
            "fig1_matched_rule_swap",
            "fig2_matched_means",
            "fig3_engine_c1_means",
            "fig4_transfer_ladder",
            "graphical_abstract",
        ] {
            assert!(dir.join(format!("{stem}.png")).is_file(), "{stem}.png");
            assert!(dir.join(format!("{stem}.pdf")).is_file(), "{stem}.pdf");
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
