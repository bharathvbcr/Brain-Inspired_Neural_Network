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
    /// against cells, unlike every other number in this module -- these come
    /// from published papers, not from this instrument. They ARE checked
    /// against their primary sources: every one was read on 2026-09-03 and
    /// recorded, with the source hashes, in
    /// `results/CITATIONS_2026-09-03_SECTION_0_AGAINST_PRIMARIES.md`. The
    /// figure says on its face that they are not cell-derived.
    ///
    /// **`FIELD_FRONTIER_HI` was 0.964 until that read, and 0.964 was wrong.**
    /// No source §0 cited reported it, and the sources it named topped out at
    /// 0.9581; 96.4 turns up in the literature as an S-MNIST result. The
    /// verified band runs from DCLS's own 95.07 +/- 0.24 (Hammouamri et al.,
    /// ICLR 2024) to S7's 96.3 (arXiv:2410.03464) -- and the top of it is a
    /// state-space model with continuously-valued output and augmentation,
    /// not a spiking network, which is why the caption no longer attributes
    /// the whole band to delays, adaptation and "spiking transformers". That
    /// last attribution had no citation at all and is withdrawn.
    pub const FIELD_FRONTIER_LO: f64 = 0.9507;
    pub const FIELD_FRONTIER_HI: f64 = 0.963;
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
        (
            "adjacent-sum-5 / published-2ms (anchor)",
            0.7062,
            0.8320,
            0.1258,
            "12/12",
        ),
        ("published-10ms", 0.6734, 0.8225, 0.1491, "10/12"),
        (
            "channels-700 (standard 700-channel input)",
            0.6774,
            0.7864,
            0.1090,
            "6/12",
        ),
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
    /// Coverage as MEASURED. Wave 22 closed the remaining twelve points, so
    /// this is 21 of 21 as of 2026-09-01 (H22-2, `analyse_wave22.py` and
    /// `mechanism_coverage.py`). Ban 4 SURVIVES the change and is why the
    /// caption still refuses "every width": 21 of 21 is every point this
    /// campaign MEASURED, not every width or depth that exists. The BUDGET
    /// caveat is gone as of 2026-09-03: wave 24 re-measured two of the
    /// twenty-one at e100 and wave 25 the other nineteen, so all twenty-one
    /// carry the contrast at two budgets and the caption says so. What
    /// replaces it is DEPTH -- H25-3 is NOT MET at h768/d32l2, 0.1271 from its
    /// d32l4 twin against a 0.10 bar -- and the caption must carry that
    /// instead. Two budgets is still not "every budget"; the caption claims
    /// two, not all.
    pub const COVERAGE_COVERED: u32 = 21;
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
        (
            "(the unclipped arm they were to rescue)",
            -0.1618,
            "1/12",
            55.494,
        ),
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
        (
            "ff+alif",
            "threshold adaptation",
            1.0,
            12,
            0.7018,
            0.8303,
            0.1285,
        ),
        (
            "rec+alif",
            "recurrent + adaptation",
            0.4,
            10,
            0.5262,
            0.7874,
            0.2612,
        ),
        (
            "ff+fixed",
            "scale-matched control",
            0.4,
            12,
            0.7088,
            0.8289,
            0.1201,
        ),
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

/// Type sizes for the seven figures the manuscript typesets.
///
/// # Why these exist
///
/// The manuscript includes every figure at `\linewidth` — 6.5 in, 468 pt — so
/// a `W`-wide canvas is scaled by 468/1400 = 0.334 on the page. plotters' SVG
/// backend also writes `font-size` as the requested size / 1.24 (measured on
/// the PDFs this module writes: a request of 26 is set at 20.97 pt). A request
/// of `s` therefore prints at `s / 1.24 × 468 / 1400` pt.
///
/// Until 2026-09-29 those figures were drawn as posters: every annotation was
/// a request of 9–16, which is 2.4–4.3 pt in print, and a reviewer reading
/// the typeset PDF could not read them. These tiers are the replacement, and
/// `tests::paper_figures_print_at_seven_points_or_more` renders each of the
/// seven figures and fails on any `font-size` below `NOTE`.
///
/// The remaining figures (5, 7, 8 and both graphical abstracts) are not in
/// the manuscript and keep their poster sizes.
mod ty {
    /// 7.0 pt on the page. The floor: nothing in a paper figure is smaller.
    pub const NOTE: i32 = 26;
    /// 7.5 pt: annotation a reader is meant to read with the plot.
    pub const BODY: i32 = 28;
    /// 8.1 pt: plotted values and category labels.
    pub const VALUE: i32 = 30;
    /// 8.6 pt: panel headings.
    pub const HEAD: i32 = 32;

    /// The printed size, in points, of a request of `size` at `\linewidth`.
    #[cfg(test)]
    pub fn printed_pt(size: i32) -> f64 {
        size as f64 / 1.24 * 468.0 / super::W as f64
    }

    /// Distance between baselines of consecutive wrapped lines.
    pub fn line_step(size: i32) -> i32 {
        (size as f64 * 1.12).round() as i32
    }
}

/// Left and right margins of the paper figures.
const LEFT: i32 = 36;
const RIGHT: i32 = W as i32 - 36;

/// How wide `text` will be drawn at `size`, in canvas units.
///
/// Uses plotters' own font metrics. If no system face resolves, it falls back
/// to an average advance of 0.46 × size, deliberately generous for sans-serif
/// so a wrap errs toward an extra line rather than a clipped one.
fn text_width(root: &DrawingArea<SVGBackend<'_>, Shift>, text: &str, size: i32) -> i32 {
    let style = TextStyle::from(("sans-serif", size).into_font());
    match root.estimate_text_size(text, &style) {
        Ok((w, _)) => w as i32,
        Err(_) => (text.chars().count() as f64 * size as f64 * 0.46).round() as i32,
    }
}

/// `text` broken into lines no wider than `max_w`, breaking only at spaces.
fn wrap(
    root: &DrawingArea<SVGBackend<'_>, Shift>,
    text: &str,
    size: i32,
    max_w: i32,
) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split(' ') {
        let candidate = if line.is_empty() {
            word.to_string()
        } else {
            format!("{line} {word}")
        };
        if !line.is_empty() && text_width(root, &candidate, size) > max_w {
            lines.push(std::mem::replace(&mut line, word.to_string()));
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// A wrapped paragraph with its top-left at `(x, y)`. Returns the y just
/// below its last line, so the next element is placed from where this one
/// ended rather than from a literal that a longer sentence would overrun.
fn para(
    root: &DrawingArea<SVGBackend<'_>, Shift>,
    (x, y): (i32, i32),
    max_w: i32,
    text: &str,
    size: i32,
    color: RGBColor,
) -> Result<i32, DrawErr> {
    let mut y = y;
    for line in wrap(root, text, size, max_w) {
        label(root, (x, y), &line, size, color)?;
        y += ty::line_step(size);
    }
    Ok(y)
}

/// One run of text inside a `boxed` block: `(text, size, color)`.
type Run<'a> = (&'a str, i32, RGBColor);

/// Paragraphs on a filled, bordered block spanning `x0..x1` from `y0`.
/// Returns the y just below the block.
fn boxed(
    root: &DrawingArea<SVGBackend<'_>, Shift>,
    (x0, y0, x1): (i32, i32, i32),
    fill: RGBColor,
    border: RGBColor,
    runs: &[Run<'_>],
) -> Result<i32, DrawErr> {
    let pad = 12;
    let inner = x1 - x0 - 2 * pad;
    let height: i32 = runs
        .iter()
        .map(|(text, size, _)| wrap(root, text, *size, inner).len() as i32 * ty::line_step(*size))
        .sum::<i32>()
        + 2 * pad
        - 6;
    map_draw(root.draw(&Rectangle::new(
        [(x0, y0), (x1, y0 + height)],
        fill.filled(),
    )))?;
    map_draw(root.draw(&Rectangle::new(
        [(x0, y0), (x1, y0 + height)],
        border.stroke_width(2),
    )))?;
    let mut y = y0 + pad;
    for (text, size, color) in runs {
        y = para(root, (x0 + pad, y), inner, text, *size, *color)?;
    }
    Ok(y0 + height)
}

/// Canvas heights of the seven manuscript figures, at `ty` sizes.
///
/// Each is where that figure's content ends plus a margin; the content itself
/// is placed from computed y positions (`para`, `boxed`), so a longer sentence
/// pushes what follows down instead of drawing over it. The manuscript sets
/// every figure at `\linewidth` with a caption of six to nine lines, so a
/// figure taller than about 1600 would not fit a 9 in text block with its
/// caption: `tests::paper_figures_fit_a_page_with_their_caption` holds that.
const H_LEAD1: u32 = 1380;
const H_LEAD2: u32 = 960;
const H_LEAD3: u32 = 1070;
const H_LEAD4: u32 = 870;
const H_FIG_S: u32 = 1440;
const H_FIG_M: u32 = 1060;
const H_FIG6: u32 = 1375;
/// Figure 8 carries twelve live-engine rows under a substrate break that must
/// not be compressed away: the break is the figure's first ban, and a divider
/// squeezed to fit reads as a section heading rather than as a change of system.
const H_FIG8: u32 = 1020;

/// A figure body: draws onto a `W`-wide canvas.
type Draw = fn(&DrawingArea<SVGBackend<'_>, Shift>) -> Result<(), DrawErr>;

/// The seven figures the manuscript typesets, with their canvas heights.
///
/// These are drawn at `ty` sizes; every other figure here is poster artwork
/// the manuscript does not include. The table is the one list both
/// `generate_all` and the legibility tests read, so a figure cannot be added
/// to the paper without being held to seven points.
///
/// Figure S is lettered, beside Figure M: it belongs to the LEAD program and
/// a fifth lead figure would renumber the secondary program 5-9 -> 6-10 one
/// day after the 2026-08-27 renumber, for one figure. `fig2_matched_means` is
/// Figure 6: the file existed on disk from 24 July with NO generator and
/// plotted the superseded value block until it was authored here on
/// 2026-08-29; the `fig2_` stem is historical and deliberately not renamed.
const PAPER_FIGURES: [(&str, Draw, u32); 7] = [
    ("leadfig1_the_conditional", draw_lead_fig1, H_LEAD1),
    ("leadfig2_headline_accuracy", draw_lead_fig2, H_LEAD2),
    ("leadfig3_width_ladder", draw_lead_fig3, H_LEAD3),
    ("leadfig4_resolution_ladder", draw_lead_fig4, H_LEAD4),
    ("figS_substrate", draw_fig_s_substrate, H_FIG_S),
    (
        "figM_mechanism_richness_addressability",
        draw_fig_m,
        H_FIG_M,
    ),
    ("fig2_matched_means", draw_fig6_matched_means, H_FIG6),
];

/// Generate required camera-ready figures into `out_dir`.
pub fn generate_all(out_dir: &Path) -> Result<Vec<PathBuf>, DrawErr> {
    fs::create_dir_all(out_dir)?;
    let mut written = Vec::new();
    // The manuscript's figures first: until 2026-08-27 nothing had been drawn
    // for any of the four lead figures.
    for (stem, draw, h) in PAPER_FIGURES {
        written.extend(write_pair_sized(out_dir, stem, draw, h)?);
    }
    // Deliberately NOT `graphical_abstract`: that file exists and depicts the
    // SECONDARY program. This one was `TODO(source needed)` until 2026-08-29 --
    // an authoring task rather than a missing number, since every value it
    // draws was already published.
    written.extend(write_pair(
        out_dir,
        "lead_graphical_abstract",
        draw_lead_graphical_abstract,
    )?);
    written.extend(write_pair(out_dir, "fig1_matched_rule_swap", draw_fig1)?);
    written.extend(write_pair(out_dir, "fig3_engine_c1_means", draw_fig3)?);
    // Figure 8. Its file existed on disk from 24 July with NO generator, so the
    // 2026-08-27 re-run could not touch it and it was stale at rung 1.
    // Authored 2026-08-29; the `fig4_` stem is historical, as `fig2_` is.
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

fn write_pair(out_dir: &Path, stem: &str, draw: Draw) -> Result<Vec<PathBuf>, DrawErr> {
    write_pair_sized(out_dir, stem, draw, H)
}

fn write_pair_sized(
    out_dir: &Path,
    stem: &str,
    draw: Draw,
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
    face: &BarFace,
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
            (bx + bar_w / 2, top - face.value_dy),
            &format!("{val:.4}"),
            face.value,
            BLACK,
        )?;
        centered(
            root,
            (bx + bar_w / 2, base + face.name_dy),
            name,
            face.name,
            BLACK,
        )?;
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
            (x0 + width + 6, y - face.chance_dy),
            &format!("chance {level:.2}"),
            face.chance,
            RGBColor(120, 120, 120),
        )?;
    }
    Ok(())
}

/// Type sizes and offsets for `draw_bar_row`, which Figure 7 also draws at
/// its original poster sizes.
struct BarFace {
    value: i32,
    value_dy: i32,
    name: i32,
    name_dy: i32,
    chance: i32,
    chance_dy: i32,
}

const BAR_POSTER: BarFace = BarFace {
    value: 14,
    value_dy: 16,
    name: 14,
    name_dy: 18,
    chance: 12,
    chance_dy: 8,
};

const BAR_PAPER: BarFace = BarFace {
    value: ty::VALUE,
    value_dy: 20,
    name: ty::BODY,
    name_dy: 22,
    chance: ty::NOTE,
    chance_dy: 12,
};

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
    face: &CostFace,
) -> Result<(), DrawErr> {
    let h = ((cost / scale) * height as f64).round() as i32;
    let top = y_base - h;
    map_draw(root.draw(&Rectangle::new(
        [(x0, top), (x0 + width, y_base)],
        color.filled(),
    )))?;
    centered(
        root,
        (x0 + width / 2, top - face.value_dy),
        &format!("+{cost:.4}"),
        face.value,
        BLACK,
    )?;
    centered(
        root,
        (x0 + width / 2, y_base + face.arm_dy),
        arm,
        face.arm,
        BLACK,
    )?;
    centered(
        root,
        (x0 + width / 2, y_base + face.detail_dy),
        detail,
        face.detail,
        RGBColor(90, 90, 90),
    )?;
    centered(
        root,
        (x0 + width / 2, y_base + face.pairs_dy),
        pairs,
        face.detail,
        RGBColor(90, 90, 90),
    )?;
    Ok(())
}

/// Type sizes and offsets for `cost_bar`, which the lead graphical abstract
/// also draws at its original poster sizes. Both arms of one figure always
/// share one face: equal weight is the first ban.
struct CostFace {
    value: i32,
    value_dy: i32,
    arm: i32,
    arm_dy: i32,
    detail: i32,
    detail_dy: i32,
    pairs_dy: i32,
}

const COST_POSTER: CostFace = CostFace {
    value: 26,
    value_dy: 22,
    arm: 18,
    arm_dy: 26,
    detail: 13,
    detail_dy: 50,
    pairs_dy: 72,
};

const COST_PAPER: CostFace = CostFace {
    value: ty::HEAD,
    value_dy: 24,
    arm: ty::VALUE,
    arm_dy: 26,
    detail: ty::NOTE,
    detail_dy: 58,
    pairs_dy: 88,
};

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
    let grey = RGBColor(100, 100, 100);
    let attn = RGBColor(70, 100, 180);
    let rate = RGBColor(200, 140, 60);

    // The prior-art banner. Without it the figure invites the reading it is
    // least entitled to.
    let top = boxed(
        root,
        (LEFT, 8, RIGHT),
        RGBColor(238, 238, 242),
        RGBColor(170, 170, 178),
        &[(
            "NOT SHOWN HERE, AND NOT CLAIMED: that SHD depends on temporal order. That is established: Cramer et al. 2022 (≤60% on spike-count-only SHD), the Neuromorphic Sequential Arena 2025 (86.48 → 68.51 with temporal processing removed model-side), Yu et al. 2025 (randomised spike times, reversed time).",
            ty::NOTE,
            RGBColor(60, 60, 60),
        )],
    )?;

    // Panels A (left) and B / C (right) share a row: two bars do not need the
    // full width, and stacking all three put Panel D off the page.
    let (a_x0, a_x1) = (LEFT, 690);
    let (c_x0, c_x1) = (730, RIGHT);

    // --- Panel A ------------------------------------------------------------
    let mut y = para(
        root,
        (a_x0, top + 18),
        a_x1 - a_x0,
        "Panel A: each arm's own shuffle cost. n = 32, seed-paired, h128 / published-2ms / e400.",
        ty::HEAD,
        BLACK,
    )?;
    y = para(
        root,
        (a_x0, y),
        a_x1 - a_x0,
        "Bin-shuffling permutes time bins independently per sample in BOTH the training and test splits, so the task itself must be solved without temporal order. Nothing is removed from the model.",
        ty::NOTE,
        grey,
    )?;

    let axis_top = y + 44;
    let base = axis_top + 250;
    let scale = 0.15;
    // Axis, so the 9.5x is readable off the drawing rather than only asserted.
    map_draw(root.draw(&PathElement::new(
        vec![(110, axis_top), (110, base)],
        RGBColor(120, 120, 120).stroke_width(2),
    )))?;
    for step in 0..=3 {
        let value = 0.05 * step as f64;
        let y = base - ((value / scale) * (base - axis_top) as f64).round() as i32;
        map_draw(root.draw(&PathElement::new(
            vec![(104, y), (a_x1, y)],
            RGBColor(224, 224, 228).stroke_width(1),
        )))?;
        label(
            root,
            (a_x0, y - 11),
            &format!("{value:.2}"),
            ty::NOTE,
            RGBColor(120, 120, 120),
        )?;
    }
    map_draw(root.draw(&PathElement::new(
        vec![(110, base), (a_x1, base)],
        BLACK.stroke_width(2),
    )))?;

    let height = base - axis_top;
    cost_bar(
        root,
        (170, base, 160, height),
        scale,
        nums::SHUFFLE_COST_ATTN_32,
        attn,
        "time-axis read-out",
        "d32/L4, ff+fixed+attn",
        "32 pairs · positive in 32/32",
        &COST_PAPER,
    )?;
    cost_bar(
        root,
        (470, base, 160, height),
        scale,
        nums::SHUFFLE_COST_RATE_32,
        rate,
        "rate read-out",
        "ff+fixed, the control",
        "32 pairs",
        &COST_PAPER,
    )?;
    centered(
        root,
        (550, axis_top + 70),
        &format!("{:.1}×", nums::SHUFFLE_COST_RATIO_32),
        40,
        RGBColor(50, 50, 50),
    )?;
    centered(
        root,
        (550, axis_top + 112),
        "ratio of the two costs",
        ty::NOTE,
        RGBColor(90, 90, 90),
    )?;
    let panel_a_end = base + COST_PAPER.pairs_dy + 20;

    // --- Panel B ------------------------------------------------------------
    let width_c = c_x1 - c_x0;
    let mut y = para(
        root,
        (c_x0, top + 18),
        width_c,
        "Panel B: the same result as the collapse of the advantage over the rate arm.",
        ty::HEAD,
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
            "n = 12",
            nums::ADVANTAGE_INTACT_12,
            nums::ADVANTAGE_SHUFFLED_12,
            nums::CONTINGENT_PCT_12,
        ),
    ];
    for (name, intact, shuffled, pct) in rows {
        y += 8;
        label(root, (c_x0, y), name, ty::BODY, BLACK)?;
        label(
            root,
            (c_x0 + 100, y),
            &format!("intact +{intact:.4}"),
            ty::VALUE,
            attn,
        )?;
        label(
            root,
            (c_x0 + 300, y),
            "→",
            ty::VALUE,
            RGBColor(120, 120, 120),
        )?;
        label(
            root,
            (c_x0 + 340, y),
            &format!("bin-shuffled +{shuffled:.4}"),
            ty::VALUE,
            rate,
        )?;
        y += ty::line_step(ty::VALUE);
        y = para(
            root,
            (c_x0 + 100, y),
            width_c - 100,
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
            ty::NOTE,
            BLACK,
        )?;
    }
    y = para(
        root,
        (c_x0, y + 8),
        width_c,
        "n = 12 is the registration; twenty further seeds move the gain by +0.0017 and the shuffle cost by +0.0010. The larger run confirms the registration; it did not rescue it.",
        ty::NOTE,
        grey,
    )?;

    // --- Panel C ------------------------------------------------------------
    // Absolute accuracy, deliberately labelled. Table SHD-2 prints `—` for the
    // n = 32 shuffled means, so this panel is n = 12 or nothing.
    y = para(
        root,
        (c_x0, y + 22),
        width_c,
        "Panel C: the n = 12 measurement, in absolute accuracy. LABELLED n = 12: absolute bin-shuffled means are not published at n = 32.",
        ty::HEAD,
        BLACK,
    )?;
    for (arm, color, intact, shuffled, cost) in [
        (
            "time-axis read-out",
            attn,
            nums::ABS_ATTN_INTACT_12,
            nums::ABS_ATTN_SHUFFLED_12,
            nums::SHUFFLE_COST_ATTN_12,
        ),
        (
            "rate read-out",
            rate,
            nums::ABS_RATE_INTACT_12,
            nums::ABS_RATE_SHUFFLED_12,
            nums::SHUFFLE_COST_RATE_12,
        ),
    ] {
        y += 6;
        label(root, (c_x0, y), arm, ty::BODY, color)?;
        label(
            root,
            (c_x0 + 220, y),
            &format!("{intact:.4} → {shuffled:.4}"),
            ty::VALUE,
            color,
        )?;
        label(
            root,
            (c_x0 + 440, y),
            &format!("cost +{cost:.4}"),
            ty::VALUE,
            color,
        )?;
        y += ty::line_step(ty::VALUE);
    }
    y = para(
        root,
        (c_x0, y + 8),
        width_c,
        &format!(
            "Attention's cost is positive in 12/12; per seed it falls between +{:.4} and +{:.4}, and every w9shf cell passes the temporal audit.",
            nums::PER_SEED_MIN_12,
            nums::PER_SEED_MAX_12
        ),
        ty::NOTE,
        grey,
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
    //   4. coverage stated as a MEASURED count, never "every width".
    let d_head = y.max(panel_a_end) + 24;
    let y = para(
        root,
        (LEFT, d_head),
        RIGHT - LEFT,
        "Panel D: the same contrast at eight operating points. n = 12 per point, n = 32 at h128, seed-paired.",
        ty::HEAD,
        BLACK,
    )?;

    const ROW: i32 = 36;
    let d_left = 310;
    let d_right = 1180;
    let d_top = y + 40;
    let d_bottom = d_top + 8 * ROW;
    let d_scale = 0.20_f64;
    let x_of = |v: f64| d_left + ((v / d_scale) * (d_right - d_left) as f64).round() as i32;

    // Gridlines and the registered bar. The bar is drawn once, labelled, and
    // every point sits to the right of it -- which is the panel's whole claim.
    for step in 0..=4 {
        let value = 0.05 * step as f64;
        let x = x_of(value);
        map_draw(root.draw(&PathElement::new(
            vec![(x, d_top - 6), (x, d_bottom)],
            RGBColor(228, 228, 232).stroke_width(1),
        )))?;
        centered(
            root,
            (x, d_bottom + 18),
            &format!("{value:.2}"),
            ty::NOTE,
            RGBColor(120, 120, 120),
        )?;
    }
    let bar_x = x_of(nums::DID_BAR);
    map_draw(root.draw(&PathElement::new(
        vec![(bar_x, d_top - 6), (bar_x, d_bottom)],
        RGBColor(200, 90, 40).stroke_width(2),
    )))?;
    label(
        root,
        (bar_x + 8, d_top - 36),
        &format!("registered bar +{:.2}", nums::DID_BAR),
        ty::NOTE,
        RGBColor(200, 90, 40),
    )?;

    for (i, (name, did, positive, of, gain_negative)) in nums::DID_LADDER.iter().enumerate() {
        let y = d_top + i as i32 * ROW;
        label(root, (LEFT, y + 2), name, ty::BODY, BLACK)?;
        let fill = if *gain_negative {
            RGBColor(200, 90, 40)
        } else {
            RGBColor(46, 106, 150)
        };
        map_draw(root.draw(&Rectangle::new(
            [(d_left, y + 6), (x_of(*did), y + 28)],
            fill.filled(),
        )))?;
        let value = format!("+{did:.4}   {positive}/{of}");
        label(
            root,
            (x_of(*did) + 10, y + 3),
            &value,
            ty::NOTE,
            RGBColor(60, 60, 60),
        )?;
        // Ban 3: the one point where the read-out consumes temporal order and
        // harms accuracy is named on the figure, not left to the caption.
        if *gain_negative {
            label(
                root,
                (x_of(*did) + 30 + text_width(root, &value, ty::NOTE), y + 3),
                "gain is NEGATIVE here",
                ty::NOTE,
                RGBColor(200, 90, 40),
            )?;
        }
    }

    // Ban 2 and ban 4 in text, beneath the points they constrain.
    let mut y = d_bottom + 46;
    y = para(
        root,
        (LEFT, y),
        RIGHT - LEFT,
        &format!(
            "The mechanism travels: every point clears the registered bar at BOTH budgets, and coverage is {} of {} operating points, NOT every width and NOT every read-out depth: h768/d32l2 is 0.1271 from its d32l4 twin, outside the 0.10 bar.",
            nums::COVERAGE_COVERED, nums::COVERAGE_TOTAL
        ),
        ty::NOTE,
        RGBColor(60, 60, 60),
    )?;
    y = para(
        root,
        (LEFT, y + 4),
        RIGHT - LEFT,
        &format!(
            "Its SIZE is not the gain. Spearman rho over the six widths is {:.4} against a registered bar of +{:.3} (NOT MET): h768 carries the smallest positive gain on the ladder ({:+.4}) and the largest DiD here ({:+.4}).",
            nums::DID_RHO, nums::DID_RHO_BAR, nums::GAIN_H768, nums::DID_H768
        ),
        ty::NOTE,
        RGBColor(60, 60, 60),
    )?;
    para(
        root,
        (LEFT, y + 4),
        RIGHT - LEFT,
        "Points are in ladder order and are deliberately NOT connected and NOT sorted by effect size: either would assert a relationship the rank correlation rejects.",
        ty::NOTE,
        grey,
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
    let grey = RGBColor(100, 100, 100);
    let rate = RGBColor(200, 140, 60);
    let attn = RGBColor(70, 100, 180);
    let field = RGBColor(90, 90, 100);
    let frontier_ink = RGBColor(60, 80, 140);

    // --- Panel A: this instrument ------------------------------------------
    let mut y = para(
        root,
        (LEFT, 8),
        RIGHT - LEFT,
        "Panel A: this instrument, seed-paired. THIS IS NOT COMPETITIVE, and the axis below says so.",
        ty::HEAD,
        BLACK,
    )?;
    y = para(
        root,
        (LEFT, y + 4),
        RIGHT - LEFT,
        &format!(
            "n = 32 (confirmation):  {:.4} → {:.4},  gain +{:.4},  positive 32/32,  ≥ 0.80 in 32/32,  |e400 − e200| = {:.4}",
            nums::HEAD_RATE_32, nums::HEAD_ATTN_32, nums::HEAD_GAIN_32, nums::HEAD_BUDGET_DELTA
        ),
        ty::BODY,
        BLACK,
    )?;
    y = para(
        root,
        (LEFT, y),
        RIGHT - LEFT,
        &format!(
            "n = 12 (registered):  {:.4} → {:.4},  gain +{:.4},  positive 12/12,  ≥ 0.80 in 12/12.  Neither supersedes the other.",
            nums::HEAD_RATE_12, nums::HEAD_ATTN_12, nums::HEAD_GAIN_12
        ),
        ty::BODY,
        RGBColor(80, 80, 80),
    )?;

    // --- Panel B: the published field, on the same axis --------------------
    let y = para(
        root,
        (LEFT, y + 18),
        RIGHT - LEFT,
        "Panel B: the published field, on the same axis. The instrument carries no temporal kernel of any kind.",
        ty::HEAD,
        BLACK,
    )?;

    // --- the single shared axis --------------------------------------------
    let (x0, x1) = (150, 1330);
    let (lo, hi) = (0.50, 1.00);
    let at = |v: f64| x0 + (((v - lo) / (hi - lo)) * (x1 - x0) as f64).round() as i32;
    // Label rows above the axis, top to bottom. Every label sits in a row of
    // its own wherever its span crosses another marker's line.
    let row = |k: i32| y + 6 + k * 32;
    let axis_y = row(8) + 20;
    let marker_y = axis_y - 44;

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
        [(lo_b, row(6)), (hi_b, axis_y)],
        RGBColor(232, 232, 236).filled(),
    )))?;
    centered(
        root,
        ((lo_b + hi_b) / 2, row(3) + 12),
        "no-delay recurrent baseline",
        ty::NOTE,
        BLACK,
    )?;
    centered(
        root,
        ((lo_b + hi_b) / 2, row(4) + 12),
        "Cramer et al. 2022: 83.2 ± 1.3%",
        ty::NOTE,
        RGBColor(70, 70, 70),
    )?;

    // The frontier band.
    let (f_lo, f_hi) = (at(nums::FIELD_FRONTIER_LO), at(nums::FIELD_FRONTIER_HI));
    map_draw(root.draw(&Rectangle::new(
        [(f_lo, row(2)), (f_hi, axis_y)],
        RGBColor(208, 218, 240).filled(),
    )))?;
    centered(
        root,
        ((f_lo + f_hi) / 2, row(0) + 12),
        &format!(
            "{:.2}–{:.1}%",
            nums::FIELD_FRONTIER_LO * 100.0,
            nums::FIELD_FRONTIER_HI * 100.0
        ),
        ty::BODY,
        frontier_ink,
    )?;
    centered(
        root,
        ((f_lo + f_hi) / 2, row(1) + 12),
        "SHD frontier",
        ty::NOTE,
        frontier_ink,
    )?;

    // Two published attention SNNs, 1.3 points apart: one row each, labelled
    // to the left of their own line.
    for (v, name, k) in [
        (
            nums::FIELD_TA_SNN,
            "TA-SNN 91.08: attention over the time axis (ICCV 2021)",
            2,
        ),
        (
            nums::FIELD_STSC_SNN,
            "STSC-SNN 92.36: attention inside the synapse (2022)",
            1,
        ),
    ] {
        map_draw(root.draw(&PathElement::new(
            vec![(at(v), row(k) + 4), (at(v), axis_y)],
            field.stroke_width(2),
        )))?;
        let style = TextStyle::from(("sans-serif", ty::NOTE).into_font())
            .color(&field)
            .pos(Pos::new(HPos::Right, VPos::Top));
        map_draw(root.draw(&Text::new(name.to_string(), (at(v) - 10, row(k)), style)))?;
    }

    map_draw(root.draw(&PathElement::new(
        vec![(x0, axis_y), (x1, axis_y)],
        BLACK.stroke_width(2),
    )))?;
    let mut tick = 50;
    while tick <= 100 {
        let v = tick as f64 / 100.0;
        let major = tick % 10 == 0;
        map_draw(root.draw(&PathElement::new(
            vec![
                (at(v), axis_y),
                (at(v), axis_y + if major { 10 } else { 6 }),
            ],
            BLACK.stroke_width(1),
        )))?;
        // Labelled every 0.10: at a readable size the 0.05 labels collide.
        if major {
            centered(
                root,
                (at(v), axis_y + 26),
                &format!("{v:.2}"),
                ty::NOTE,
                RGBColor(90, 90, 90),
            )?;
        }
        tick += 5;
    }
    label(
        root,
        (LEFT, axis_y - 36),
        "accuracy",
        ty::NOTE,
        RGBColor(90, 90, 90),
    )?;

    // The 0.80 gate, labelled beneath its own tick.
    let mut gy = row(6);
    while gy < axis_y {
        map_draw(root.draw(&PathElement::new(
            vec![(at(0.80), gy), (at(0.80), (gy + 10).min(axis_y))],
            RGBColor(120, 120, 120).stroke_width(2),
        )))?;
        gy += 20;
    }
    centered(
        root,
        (at(0.80), axis_y + 56),
        "0.80 gate",
        ty::NOTE,
        RGBColor(110, 110, 110),
    )?;

    // --- this instrument's two markers, on that axis ------------------------
    for (v, color, name) in [
        (nums::HEAD_RATE_32, rate, "rate read-out  ff+fixed"),
        (nums::HEAD_ATTN_32, attn, "time-axis read-out  d32/L4"),
    ] {
        map_draw(root.draw(&PathElement::new(
            vec![(at(v), marker_y), (at(v), axis_y)],
            color.stroke_width(2),
        )))?;
        map_draw(root.draw(&Circle::new((at(v), marker_y), 11, color.filled())))?;
        centered(
            root,
            (at(v), row(5) + 12),
            name,
            ty::NOTE,
            RGBColor(60, 60, 60),
        )?;
        centered(
            root,
            (at(v), row(6) + 14),
            &format!("{v:.4}"),
            ty::VALUE,
            BLACK,
        )?;
    }
    map_draw(root.draw(&PathElement::new(
        vec![
            (at(nums::HEAD_RATE_32), marker_y),
            (at(nums::HEAD_ATTN_32), marker_y),
        ],
        BLACK.stroke_width(2),
    )))?;
    centered(
        root,
        (
            (at(nums::HEAD_RATE_32) + at(nums::HEAD_ATTN_32)) / 2 - 20,
            row(6) + 14,
        ),
        &format!("gain +{:.4}", nums::HEAD_GAIN_32),
        ty::BODY,
        BLACK,
    )?;

    // Ban 2, drawn to scale on this axis rather than only asserted in words.
    let band = at(lo + nums::FIELD_UNRESOLVABLE) - x0;
    let band_y = axis_y + 84;
    map_draw(root.draw(&Rectangle::new(
        [(x0, band_y + 4), (x0 + band, band_y + 24)],
        RGBColor(190, 190, 200).filled(),
    )))?;
    let y = para(
        root,
        (x0 + band + 14, band_y),
        x1 - x0 - band - 14,
        &format!(
            "{:.1} points, to scale. SHD ships no validation set: Baronig et al. 2025 report one model at 95.81 validating on test and 93.79 on a held-out split. Differences narrower than this between published SHD numbers are not reliably meaningful, including this paper's own.",
            nums::FIELD_UNRESOLVABLE * 100.0
        ),
        ty::NOTE,
        grey,
    )?;

    // --- Panel C: the clearance is geometry-specific ------------------------
    let mut y = para(
        root,
        (LEFT, y + 18),
        RIGHT - LEFT,
        "Panel C: the 0.80 clearance is geometry-specific. The gain is not: attention buys roughly the same amount everywhere tested.",
        ty::HEAD,
        BLACK,
    )?;
    for (name, r, a, gain, clears) in nums::GEOM {
        y += 6;
        label(root, (LEFT + 12, y), name, ty::BODY, BLACK)?;
        label(root, (580, y), &format!("rate {r:.4}"), ty::BODY, rate)?;
        label(root, (760, y), &format!("attention {a:.4}"), ty::BODY, attn)?;
        label(
            root,
            (1000, y),
            &format!("gain +{gain:.4}"),
            ty::BODY,
            BLACK,
        )?;
        label(
            root,
            (1180, y),
            &format!("≥ 0.80 in {clears}"),
            ty::BODY,
            if clears == "12/12" {
                BLACK
            } else {
                RGBColor(170, 90, 60)
            },
        )?;
        y += ty::line_step(ty::BODY);
    }

    y = para(
        root,
        (LEFT, y + 14),
        RIGHT - LEFT,
        "This instrument lands INSIDE the no-delay recurrent band, which is where an architecture carrying no temporal kernel of any kind should land, and is the whole reading.",
        ty::NOTE,
        RGBColor(60, 60, 60),
    )?;
    // The literature strip's status. It read "NOT MACHINE-CHECKED ... must be
    // checked against its primary source before submission" until 2026-09-29;
    // that check was done on 2026-09-03
    // (`CITATIONS_2026-09-03_SECTION_0_AGAINST_PRIMARIES.md`), and
    // `check_section0_citations.py` now gates the `FIELD_*` constants drawn
    // here against that record. What remains true is that no cell on disk
    // produced them, so they are not checked against cells.
    para(
        root,
        (LEFT, y + 4),
        RIGHT - LEFT,
        "Literature values in Panel B are published figures, checked against their primary sources on 2026-09-03, before submission. They are NOT MACHINE-CHECKED against cells: no cell on disk produced them.",
        ty::NOTE,
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
    let grey = RGBColor(100, 100, 100);
    let warn = RGBColor(160, 80, 60);

    let mut y = para(
        root,
        (LEFT, 8),
        RIGHT - LEFT,
        "Panel A: attention's gain on six widths. n = 12 per rung, e400, seed-paired. Rungs are points: no curve is fitted and none may be.",
        ty::HEAD,
        BLACK,
    )?;
    y = para(
        root,
        (LEFT, y + 2),
        RIGHT - LEFT,
        "EVERY RUNG IS d32/L4. The gain inverts at h1024 as a property of THIS read-out depth, not of the width: at h1024, d32/L2 gains +0.0405 in 20/20 seeds.",
        ty::NOTE,
        RGBColor(70, 70, 70),
    )?;

    // --- Panel A ------------------------------------------------------------
    let (lo, hi) = (-0.20, 0.15);
    let (top, bottom) = (y + 44, y + 380);
    let x0 = 220;
    let step_x = 175;
    let at_y = |v: f64| bottom - (((v - lo) / (hi - lo)) * (bottom - top) as f64).round() as i32;
    let at_x = |i: usize| x0 + (i as i32) * step_x;

    for k in 0..8 {
        let v = -0.20 + 0.05 * k as f64;
        let y = at_y(v);
        map_draw(root.draw(&PathElement::new(
            vec![(x0 - 70, y), (RIGHT, y)],
            RGBColor(232, 232, 236).stroke_width(1),
        )))?;
        label(
            root,
            (70, y - 11),
            &format!("{v:+.2}"),
            ty::NOTE,
            RGBColor(120, 120, 120),
        )?;
    }
    let zero = at_y(0.0);
    map_draw(root.draw(&PathElement::new(
        vec![(x0 - 70, zero), (RIGHT, zero)],
        BLACK.stroke_width(2),
    )))?;
    label(
        root,
        (LEFT - 20, zero - 44),
        "gain",
        ty::NOTE,
        RGBColor(90, 90, 90),
    )?;

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
    label(
        root,
        (break_x + 12, top),
        "the step sits HERE,",
        ty::NOTE,
        RGBColor(170, 90, 60),
    )?;
    label(
        root,
        (break_x + 12, top + ty::line_step(ty::NOTE)),
        "between h768 and h1024",
        ty::NOTE,
        RGBColor(170, 90, 60),
    )?;

    for (i, (name, _rate, _attn, gain, positive)) in nums::LADDER.into_iter().enumerate() {
        let (x, y) = (at_x(i), at_y(gain));
        let color = if gain > 0.0 {
            RGBColor(70, 100, 180)
        } else {
            RGBColor(170, 90, 60)
        };
        map_draw(root.draw(&Circle::new((x, y), 12, color.filled())))?;
        centered(root, (x, y - 32), &format!("{gain:+.4}"), ty::VALUE, BLACK)?;
        centered(root, (x, bottom + 24), name, ty::VALUE, BLACK)?;
        centered(
            root,
            (x, bottom + 56),
            positive,
            ty::NOTE,
            RGBColor(110, 110, 110),
        )?;
    }

    // h384 and h512 are not distinguishable, and the figure has to say so where
    // a reader would otherwise read a dip.
    let (b0, b1) = (at_x(2), at_x(3));
    let bracket_y = at_y(0.0) + 22;
    map_draw(root.draw(&PathElement::new(
        vec![
            (b0, bracket_y - 8),
            (b0, bracket_y),
            (b1, bracket_y),
            (b1, bracket_y - 8),
        ],
        RGBColor(120, 120, 130).stroke_width(2),
    )))?;
    // Left-aligned under the bracket: centred, it ran into the step line.
    label(
        root,
        (x0 - 20, bracket_y + 10),
        &format!(
            "not distinguishable at n = 12: paired {:+.4}, sd {:.4}, negative in 7 of 12",
            nums::LADDER_H384_H512,
            nums::LADDER_H384_H512_SD
        ),
        ty::NOTE,
        RGBColor(100, 100, 110),
    )?;

    let mut y = para(
        root,
        (LEFT, bottom + 88),
        RIGHT - LEFT,
        &format!(
            "The drop into h1024 is {:.4}, {:.1}× the largest gap below it ({:.4}), and more than twice the registered 3× bar. That is what makes it a threshold rather than the slope continuing.",
            nums::LADDER_DROP, nums::LADDER_DROP_RATIO, nums::LADDER_LARGEST_GAP_BELOW
        ),
        ty::BODY,
        BLACK,
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
    y = para(
        root,
        (LEFT, y + 6),
        RIGHT - LEFT,
        &format!(
            "THE MECHANISM DOES NOT TRACK THIS CURVE. Spearman rho between these rungs and their difference-in-differences is {:.4}, against a registered bar of +{:.3}: NOT MET. h768 carries the smallest positive gain on this ladder ({:+.4}) and the largest DiD in the campaign ({:+.4}). The DiD ladder is Figure 1 Panel D, and is deliberately not superimposed here.",
            nums::DID_RHO, nums::DID_RHO_BAR, nums::GAIN_H768, nums::DID_H768
        ),
        ty::NOTE,
        warn,
    )?;

    // --- Panel B ------------------------------------------------------------
    let mut y = para(
        root,
        (LEFT, y + 20),
        RIGHT - LEFT,
        "Panel B: located, and not explained. Three preregistered rescue levers at h1024 / d32/L4, n = 12 each.",
        ty::HEAD,
        BLACK,
    )?;
    for (lever, gain, positive, norm) in nums::LEVERS {
        y += 4;
        let rescued = lever.starts_with('(');
        let ink = if rescued {
            RGBColor(110, 110, 110)
        } else {
            BLACK
        };
        label(root, (LEFT + 12, y), lever, ty::BODY, ink)?;
        label(
            root,
            (500, y),
            &format!("gain {gain:+.4}"),
            ty::BODY,
            RGBColor(170, 90, 60),
        )?;
        label(
            root,
            (690, y),
            &format!("positive {positive}"),
            ty::BODY,
            ink,
        )?;
        label(
            root,
            (890, y + 2),
            &format!("median epoch-mean gradient norm {norm:.3}"),
            ty::NOTE,
            ink,
        )?;
        y += ty::line_step(ty::BODY);
    }
    y = para(
        root,
        (LEFT, y + 10),
        RIGHT - LEFT,
        "At h512 the clipping flag is inert: 12/12 cells byte-identical to the archived unclipped cells.",
        ty::NOTE,
        grey,
    )?;
    para(
        root,
        (LEFT, y + 4),
        RIGHT - LEFT,
        "LOCATED BUT UNEXPLAINED. Gradient norms leaving O(1) are a correlate, not a cause, and overfitting on 8,156 training samples is not excluded by anything in this paper.",
        ty::NOTE,
        RGBColor(60, 60, 60),
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
    let grey = RGBColor(100, 100, 100);
    let y = para(
        root,
        (LEFT, 8),
        RIGHT - LEFT,
        "fixed-tN: the 1400 ms analysis window is HELD FIXED and only the number of frames varies, which isolates resolution from sequence length. n = 12 per rung.",
        ty::BODY,
        BLACK,
    )?;

    let (lo, hi) = (0.60, 0.90);
    let (top, bottom) = (y + 60, y + 440);
    let x0 = 300;
    let step_x = 380;
    let at_y = |v: f64| bottom - (((v - lo) / (hi - lo)) * (bottom - top) as f64).round() as i32;
    let at_x = |i: usize| x0 + (i as i32) * step_x;

    for k in 0..7 {
        let v = 0.60 + 0.05 * k as f64;
        let y = at_y(v);
        map_draw(root.draw(&PathElement::new(
            vec![(200, y), (RIGHT, y)],
            RGBColor(234, 234, 238).stroke_width(1),
        )))?;
        label(
            root,
            (130, y - 11),
            &format!("{v:.2}"),
            ty::NOTE,
            RGBColor(120, 120, 120),
        )?;
    }
    label(
        root,
        (LEFT, top - 44),
        "accuracy",
        ty::NOTE,
        RGBColor(90, 90, 90),
    )?;
    // The 0.80 gate, cleared 12/12 at every rung.
    let gate = at_y(0.80);
    let mut x = 200;
    while x < RIGHT {
        map_draw(root.draw(&PathElement::new(
            vec![(x, gate), ((x + 14).min(RIGHT), gate)],
            RGBColor(120, 120, 120).stroke_width(2),
        )))?;
        x += 28;
    }

    let attn = RGBColor(70, 100, 180);
    let rate = RGBColor(200, 140, 60);
    // Both series, joined within themselves only. The comparison the figure
    // makes is vertical — the gap at each rung — not across rungs.
    for (series, color, pick) in [("attention", attn, 1usize), ("rate", rate, 0usize)] {
        let _ = series;
        let points: Vec<(i32, i32)> = nums::RESOLUTION
            .iter()
            .enumerate()
            .map(|(i, (_, _, r, a, _))| (at_x(i), at_y(if pick == 1 { *a } else { *r })))
            .collect();
        map_draw(root.draw(&PathElement::new(points.clone(), color.stroke_width(3))))?;
        for point in points {
            map_draw(root.draw(&Circle::new(point, 11, color.filled())))?;
        }
    }

    for (i, (contract, bin, r, a, gain)) in nums::RESOLUTION.into_iter().enumerate() {
        let x = at_x(i);
        centered(
            root,
            (x, at_y(a) - 32),
            &format!("{a:.4}"),
            ty::VALUE,
            BLACK,
        )?;
        centered(
            root,
            (x, at_y(r) + 32),
            &format!("{r:.4}"),
            ty::VALUE,
            BLACK,
        )?;
        map_draw(root.draw(&PathElement::new(
            vec![(x, at_y(a)), (x, at_y(r))],
            RGBColor(140, 140, 148).stroke_width(1),
        )))?;
        let mid = (at_y(a) + at_y(r)) / 2;
        centered(
            root,
            (x + 100, mid),
            &format!("gain +{gain:.4}"),
            ty::BODY,
            BLACK,
        )?;
        centered(
            root,
            (x + 100, mid + 30),
            "12/12 positive",
            ty::NOTE,
            RGBColor(110, 110, 110),
        )?;
        centered(root, (x, bottom + 26), contract, ty::VALUE, BLACK)?;
        centered(
            root,
            (x, bottom + 58),
            &format!("{bin} bins"),
            ty::NOTE,
            RGBColor(110, 110, 110),
        )?;
    }
    centered(
        root,
        (at_x(0) - 150, bottom + 26),
        "coarse",
        ty::NOTE,
        RGBColor(120, 120, 120),
    )?;
    centered(
        root,
        (at_x(2) + 150, bottom + 26),
        "fine",
        ty::NOTE,
        RGBColor(120, 120, 120),
    )?;
    // Below the bin labels, not level with them.
    label(
        root,
        (LEFT, bottom + 84),
        "● time-axis read-out",
        ty::BODY,
        attn,
    )?;
    label(root, (330, bottom + 84), "● rate read-out", ty::BODY, rate)?;
    // The gate's key sits in the legend row: on the plot it ran through the
    // t500 connector and its gain label.
    label(
        root,
        (580, bottom + 86),
        "– – 0.80 gate, cleared 12/12 at every rung",
        ty::NOTE,
        RGBColor(110, 110, 110),
    )?;

    let mut y = para(
        root,
        (LEFT, bottom + 128),
        RIGHT - LEFT,
        &format!(
            "The advantage SHRINKS as bins get finer: gain(t500) − gain(t100) = {:.4} against a two-sided bar of 0.03.",
            nums::RESOLUTION_TREND
        ),
        ty::BODY,
        BLACK,
    )?;
    y = para(
        root,
        (LEFT, y + 4),
        RIGHT - LEFT,
        &format!(
            "The rate baseline rises +{:.4} across the same ladder, inside the preregistered 0.05 confound bar. Both series are drawn because without it the falling gain reads as the attention arm degrading, when it is the rate arm catching up.",
            nums::RESOLUTION_BASELINE_DRIFT
        ),
        ty::NOTE,
        RGBColor(70, 70, 70),
    )?;
    y = para(
        root,
        (LEFT, y + 4),
        RIGHT - LEFT,
        "NOT DRAWN, and not drawable here: the published-Nms family, which moves bin width and sequence length together so no number from it can be attributed to either.",
        ty::NOTE,
        grey,
    )?;
    para(
        root,
        (LEFT, y + 4),
        RIGHT - LEFT,
        "NO MECHANISM AND NO PREFERENCE. The paper offers no account of why the advantage shrinks and recommends no operating point. \"Attention prefers coarse bins\" is not carried by this.",
        ty::NOTE,
        grey,
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
    face: &CardFace,
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
    let chip = face.chip_w;
    map_draw(root.draw(&Rectangle::new(
        [(x0 + 10, y0 + 10), (x0 + 10 + chip, y0 + 10 + face.chip_h)],
        verdict.border().filled(),
    )))?;
    centered(
        root,
        (x0 + 10 + chip / 2, y0 + 10 + face.chip_h / 2),
        verdict.tag(),
        face.tag,
        WHITE,
    )?;
    label(
        root,
        (x0 + 10 + chip + 14, y0 + face.rule_dy),
        rule,
        face.rule,
        BLACK,
    )?;
    label(
        root,
        (x0 + 12, y0 + face.mechanism_dy),
        mechanism,
        face.mechanism,
        RGBColor(90, 90, 90),
    )?;
    match face.value_at {
        ValueAt::Below(dy) => label(root, (x0 + 12, y0 + dy), &both(value), face.value, BLACK)?,
        ValueAt::RightOfRule => {
            let style = TextStyle::from(("sans-serif", face.value).into_font())
                .color(&BLACK)
                .pos(Pos::new(HPos::Right, VPos::Top));
            map_draw(root.draw(&Text::new(both(value), (x1 - 14, y0 + face.rule_dy), style)))?
        }
    }
    Ok(())
}

/// Where a `rule_card` prints its `ff / rec` value.
#[derive(Clone, Copy)]
enum ValueAt {
    /// On its own line, this far below the card's top edge.
    Below(i32),
    /// Right-aligned on the chip row: for a full-width card, where a third
    /// line would cost height the figure does not have.
    RightOfRule,
}

/// Type sizes and geometry for `rule_card`. Figures 5 and 8 and the secondary
/// graphical abstract keep `CARD_POSTER`.
struct CardFace {
    chip_w: i32,
    chip_h: i32,
    tag: i32,
    rule: i32,
    rule_dy: i32,
    mechanism: i32,
    mechanism_dy: i32,
    value: i32,
    value_at: ValueAt,
}

const CARD_POSTER: CardFace = CardFace {
    chip_w: 128,
    chip_h: 22,
    tag: 12,
    rule: 16,
    rule_dy: 12,
    mechanism: 12,
    mechanism_dy: 44,
    value: 15,
    value_at: ValueAt::Below(66),
};

/// A card with its value on a third line. 128 tall.
const CARD_PAPER_STACKED: CardFace = CardFace {
    chip_w: 224,
    chip_h: 36,
    tag: ty::NOTE,
    rule: ty::BODY,
    rule_dy: 14,
    mechanism: ty::NOTE,
    mechanism_dy: 56,
    value: ty::BODY,
    value_at: ValueAt::Below(90),
};

/// A full-width card with its value on the chip row. 92 tall.
const CARD_PAPER_ROW: CardFace = CardFace {
    value_at: ValueAt::RightOfRule,
    ..CARD_PAPER_STACKED
};

fn draw_fig_m(root: &DrawingArea<SVGBackend<'_>, Shift>) -> Result<(), DrawErr> {
    let grey = RGBColor(100, 100, 100);
    // The ceiling is the point of the panel, not an aside: with the reference
    // saturated, every passing arm reduces to "above 0.75" and no ordering
    // among them may be drawn or claimed.
    let reference = Verdict::Reference;
    let banner = format!(
        "SuperSpike BPTT reference saturates: {:.4} ff / {:.4} rec, every suite, both graphs. Every other rule tested clears the gate, so this panel shows WHICH SINGLE RULE FAILS a task the rest saturate; it does not rank the rest. Pass / fail / at-chance, not a graded surface.",
        nums::CEILING.0,
        nums::CEILING.1
    );
    let y = boxed(
        root,
        (LEFT, 8, RIGHT),
        reference.fill(),
        reference.border(),
        &[(&banner, ty::NOTE, BLACK)],
    )?;

    // Two columns of cards beside a column of row labels.
    let (c1_x0, c1_x1) = (190, 772);
    let (c2_x0, c2_x1) = (792, RIGHT);
    const CARD: i32 = 128;
    let head_y = y + 16;
    centered(
        root,
        ((c1_x0 + c1_x1) / 2, head_y + 16),
        "Low addressability (broadcast)",
        ty::BODY,
        BLACK,
    )?;
    centered(
        root,
        ((c2_x0 + c2_x1) / 2, head_y + 16),
        "High addressability (directed / local feedback)",
        ty::BODY,
        BLACK,
    )?;

    // --- low richness ------------------------------------------------------
    // Two rules, one cell. Drawn apart, with the distance between them stated,
    // because that distance is the reason the lead claim says "±1 three-factor"
    // and not "broadcast".
    let low = head_y + 44;
    let pm1_y = low + 8;
    let flat_y = pm1_y + CARD + 8;
    let cell_note = flat_y + CARD + 6;
    let note_lines = wrap(
        root,
        "One cell, two rules, 0.28 apart. Collapsing them into one “broadcast ±1” box is the overreach the lead claim’s wording exists to avoid.",
        ty::NOTE,
        c1_x1 - c1_x0 - 12,
    );
    let low_end = cell_note + note_lines.len() as i32 * ty::line_step(ty::NOTE) + 6;
    map_draw(root.draw(&Rectangle::new(
        [(c1_x0 - 10, low), (c1_x1 + 10, low_end)],
        RGBColor(246, 246, 249).filled(),
    )))?;
    rule_card(
        root,
        (c1_x0, pm1_y, c1_x1, pm1_y + CARD),
        Verdict::Fail,
        "±1 × surrogate eligibility",
        "MatchedLocal: the lead FAIL, at chance",
        nums::BROADCAST_PM1,
        &CARD_PAPER_STACKED,
    )?;
    rule_card(
        root,
        (c1_x0, flat_y, c1_x1, flat_y + CARD),
        Verdict::Contrast,
        "±1 broadcast REINFORCE",
        "MatchedRlFlat: same reward and topology",
        nums::RL_FLAT,
        &CARD_PAPER_STACKED,
    )?;
    for (i, line) in note_lines.iter().enumerate() {
        label(
            root,
            (c1_x0, cell_note + i as i32 * ty::line_step(ty::NOTE)),
            line,
            ty::NOTE,
            grey,
        )?;
    }

    rule_card(
        root,
        (c2_x0, pm1_y, c2_x1, pm1_y + CARD),
        Verdict::Pass,
        "REINFORCE × frozen B_i",
        "MatchedRlReinforceFb: directed, same ±1 reward",
        nums::RL_FB,
        &CARD_PAPER_STACKED,
    )?;
    // The reading of the grid, in the space the taller low-richness cell
    // leaves beside it.
    let mut ny = pm1_y + CARD + 16;
    for (text, ink) in [
        (
            "Richness alone does not decide it: a graded BROADCAST rule reaches 0.9975.",
            grey,
        ),
        (
            "Addressability alone does not either: ±1 through a frozen B_i passes.",
            grey,
        ),
        ("What fails is the one rule that has neither.", BLACK),
    ] {
        ny = para(root, (c2_x0, ny), c2_x1 - c2_x0, text, ty::NOTE, ink)? + 4;
    }

    // --- high richness -----------------------------------------------------
    let high = low_end.max(ny) + 16;
    rule_card(
        root,
        (c1_x0, high, c1_x1, high + CARD),
        Verdict::Contrast,
        "broadcast graded error",
        "MatchedBroadcastGradedError: not a locality proof",
        nums::BROADCAST_GRADED,
        &CARD_PAPER_STACKED,
    )?;
    rule_card(
        root,
        (c2_x0, high, c2_x1, high + CARD),
        Verdict::Pass,
        "graded error × DFA",
        "MatchedDfaGradedError",
        nums::DFA,
        &CARD_PAPER_STACKED,
    )?;

    for (text, detail, cy) in [
        ("Low richness", "(±1)", (low + low_end) / 2),
        ("High richness", "(graded)", high + CARD / 2),
    ] {
        label(root, (LEFT - 20, cy - 30), text, ty::BODY, BLACK)?;
        label(root, (LEFT - 20, cy + 2), detail, ty::BODY, grey)?;
    }

    // --- Panel B -----------------------------------------------------------
    let b_top = high + CARD + 30;
    let b_head = wrap(
        root,
        "Panel B: XOR locality flip (1-layer xor_thresh). Addressability evidence: broadcast fails a task DFA solves.",
        ty::HEAD,
        RIGHT - LEFT - 32,
    );
    let chart_top = b_top + 12 + b_head.len() as i32 * ty::line_step(ty::HEAD);
    let chart_h = 250;
    let foot = chart_top + chart_h + 28;
    let foot_lines = wrap(
        root,
        "Not claimed for 2-layer mid-init depth locality, where broadcast also solves. Matched PASS does not imply live muted-θ / k-WTA G2 PASS.",
        ty::NOTE,
        RIGHT - LEFT - 32,
    );
    let b_bottom = foot + foot_lines.len() as i32 * ty::line_step(ty::NOTE) + 8;
    map_draw(root.draw(&Rectangle::new(
        [(LEFT, b_top), (RIGHT, b_bottom)],
        RGBColor(245, 245, 248).filled(),
    )))?;
    for (i, line) in b_head.iter().enumerate() {
        label(
            root,
            (LEFT + 16, b_top + 10 + i as i32 * ty::line_step(ty::HEAD)),
            line,
            ty::HEAD,
            BLACK,
        )?;
    }
    draw_bar_row(
        root,
        90,
        chart_top,
        1060,
        chart_h,
        Some(0.5),
        &[
            (
                "Broadcast: at chance",
                nums::XOR_BCAST,
                Verdict::Fail.border(),
            ),
            ("DFA: solves", nums::XOR_DFA, Verdict::Pass.border()),
            (
                "Gradient: ceiling",
                nums::XOR_GRAD,
                Verdict::Reference.border(),
            ),
        ],
        &BAR_PAPER,
    )?;
    for (i, line) in foot_lines.iter().enumerate() {
        label(
            root,
            (LEFT + 16, foot + i as i32 * ty::line_step(ty::NOTE)),
            line,
            ty::NOTE,
            grey,
        )?;
    }
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
            "MatchedLocal: scalar reward, no addressing",
            nums::BROADCAST_PM1,
        ),
        (
            300,
            Verdict::Pass,
            "graded error × DFA",
            "MatchedDfaGradedError: graded error, fixed random feedback",
            nums::DFA,
        ),
        (
            480,
            Verdict::Pass,
            "REINFORCE × frozen B_i",
            "MatchedRlReinforceFb: ±1 reward through directed feedback",
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
        rule_card(
            root,
            (560, y, 1300, y + 110),
            verdict,
            title,
            mechanism,
            value,
            &CARD_POSTER,
        )?;
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
    let grey = RGBColor(100, 100, 100);
    let y = para(
        root,
        (LEFT, 8),
        RIGHT - LEFT,
        "h128 / published-2ms / e400 / d32-L4, seed-paired. Does the read-out ADD temporal structure no such substrate represents, or SUBSTITUTE for the adaptation and recurrence ff+fixed happens not to have?",
        ty::NOTE,
        RGBColor(70, 70, 70),
    )?;

    // --- Panel A ------------------------------------------------------------
    let y = para(
        root,
        (LEFT, y + 14),
        RIGHT - LEFT,
        "Panel A: one shared accuracy axis, in the row order of Table SHD-7. NOT sorted by gain.",
        ty::HEAD,
        BLACK,
    )?;

    let (lo, hi) = (0.45, 0.90);
    let (x0, x1) = (520, 1080);
    let at_x = |v: f64| x0 + (((v - lo) / (hi - lo)) * (x1 - x0) as f64).round() as i32;
    let rate_col = RGBColor(200, 140, 60);
    let attn_col = RGBColor(70, 100, 180);
    label(root, (x0, y + 4), "● rate read-out", ty::NOTE, rate_col)?;
    label(
        root,
        (x0 + 220, y + 4),
        "● + attention d32/L4",
        ty::NOTE,
        attn_col,
    )?;
    const ROW: i32 = 66;
    let top = y + 44;
    let bottom = top + 4 * ROW;

    for k in 0..=9 {
        let v = lo + 0.05 * k as f64;
        let x = at_x(v);
        map_draw(root.draw(&PathElement::new(
            vec![(x, top), (x, bottom)],
            RGBColor(232, 232, 238).stroke_width(1),
        )))?;
        if k % 2 == 0 {
            centered(
                root,
                (x, bottom + 18),
                &format!("{v:.2}"),
                ty::NOTE,
                RGBColor(120, 120, 120),
            )?;
        }
    }

    for (i, (substrate, note, scale, pairs, rate, attn, gain)) in
        nums::SUBSTRATE.into_iter().enumerate()
    {
        let y = top + 30 + (i as i32) * ROW;
        label(root, (LEFT, y - 28), substrate, ty::VALUE, BLACK)?;
        label(
            root,
            (LEFT + 150, y - 25),
            &format!("scale {scale:.1} · n = {pairs} pairs"),
            ty::NOTE,
            RGBColor(80, 80, 80),
        )?;
        label(root, (LEFT, y + 4), note, ty::NOTE, RGBColor(110, 110, 110))?;
        // Rate -> attention on one axis. The connector is within a row only:
        // the comparison this figure makes is horizontal, and joining rows
        // would draw the substrate ordering ban 1 refuses.
        map_draw(root.draw(&PathElement::new(
            vec![(at_x(rate), y), (at_x(attn), y)],
            RGBColor(160, 160, 168).stroke_width(2),
        )))?;
        map_draw(root.draw(&Circle::new((at_x(rate), y), 10, rate_col.filled())))?;
        map_draw(root.draw(&Circle::new((at_x(attn), y), 10, attn_col.filled())))?;
        // The values sit in a column of their own: printed beside the dots,
        // three of the four attention labels ran through the dashed marker.
        label(
            root,
            (1110, y - 28),
            &format!("{rate:.4} → {attn:.4}"),
            ty::NOTE,
            RGBColor(80, 80, 80),
        )?;
        label(
            root,
            (1110, y + 2),
            &format!("gain +{gain:.4}"),
            ty::BODY,
            BLACK,
        )?;
    }

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
    let y = para(
        root,
        (LEFT, bottom + 48),
        RIGHT - LEFT,
        &format!(
            "THE RECURRENT SUBSTRATE DOES NOT WIN. rec+alif+attn reaches {rec_attn:.4} against ff+fixed+attn's {ff_attn:.4} at the same scale: attention closes most of the gap the substrate gives away, and not all of it. NO VERDICT IS ISSUED ON THAT ORDERING, and nothing here is sorted by gain.",
        ),
        ty::BODY,
        BLACK,
    )?;

    // --- Panel B ------------------------------------------------------------
    let y = para(
        root,
        (LEFT, y + 20),
        RIGHT - LEFT,
        "Panel B: the registered contrasts, each against its two-sided bar",
        ty::HEAD,
        BLACK,
    )?;
    let contrasts = [
        (
            "A-1  gain(ff+alif) − gain(ff+fixed)",
            nums::SUBSTRATE_A1,
            "6 of 12",
            Verdict::Fail,
            "a coin flip: adaptation makes no difference to the gain HERE",
        ),
        (
            "M-2  gain(rec+alif) − gain(ff+fixed), scale 0.4",
            nums::SUBSTRATE_M2,
            "10 of 10",
            Verdict::Pass,
            "SUPPORTED: the gain roughly doubles on the recurrent substrate",
        ),
        (
            "M-4  ff+fixed at scale 0.4 vs archived 1.0",
            nums::SUBSTRATE_M4,
            "—",
            Verdict::Contrast,
            "the scale is not doing the work",
        ),
    ];
    let (bx0, bx1) = (600, 820);
    let at_b = |v: f64| bx0 + ((v / 0.20) * (bx1 - bx0) as f64).round() as i32;
    let bar_x = at_b(nums::SUBSTRATE_BAR);
    let b_top = y + 34;
    const B_ROW: i32 = 64;
    for (i, (name, value, positive, verdict, reading)) in contrasts.into_iter().enumerate() {
        let y = b_top + 22 + (i as i32) * B_ROW;
        label(root, (LEFT, y - 14), name, ty::BODY, BLACK)?;
        map_draw(root.draw(&Circle::new(
            (at_b(value), y),
            10,
            verdict.border().filled(),
        )))?;
        // The value and its seed count sit in a column of their own: beside
        // the dot, a value short of the bar ran through the dashed bar line.
        label(
            root,
            (850, y - 14),
            &format!("+{value:.4}   {positive}"),
            ty::BODY,
            RGBColor(60, 60, 60),
        )?;
        para(root, (1040, y - 26), RIGHT - 1040, reading, ty::NOTE, grey)?;
    }
    let b_bottom = b_top + 3 * B_ROW;
    let mut by = b_top;
    while by < b_bottom {
        map_draw(root.draw(&PathElement::new(
            vec![(bar_x, by), (bar_x, (by + 10).min(b_bottom))],
            RGBColor(120, 120, 120).stroke_width(2),
        )))?;
        by += 20;
    }
    centered(
        root,
        (bar_x, b_top - 16),
        &format!("bar ±{:.2}", nums::SUBSTRATE_BAR),
        ty::NOTE,
        RGBColor(100, 100, 100),
    )?;

    // Ban 2. Both readings, and the status of the second stated on its face.
    let first = format!(
        "BOTH READINGS OF “THE GAIN DOUBLES”, because neither may be quoted alone. rec+alif starts 0.18 LOWER, with {:.4} of headroom against {:.4}.",
        nums::HEADROOM_REC,
        nums::HEADROOM_FF
    );
    let second = format!(
        "Gain / headroom is {:.3} against {:.3}, so the ratio falls {:.1}× → {:.2}×. The ORDERING SURVIVES; most of its apparent size does not.",
        nums::HEADROOM_RATIO_REC,
        nums::HEADROOM_RATIO_FF,
        nums::RAW_RATIO,
        nums::NORMALISED_RATIO
    );
    let y = boxed(
        root,
        (LEFT, b_bottom + 16, RIGHT),
        RGBColor(253, 246, 224),
        RGBColor(200, 160, 60),
        &[
            (&first, ty::BODY, BLACK),
            (&second, ty::NOTE, RGBColor(70, 70, 70)),
            (
                "That normalisation is POST-HOC AND NOT REGISTERED, and saying so is not licence to prefer whichever of the two numbers is more convenient, in either direction.",
                ty::NOTE,
                RGBColor(110, 110, 110),
            ),
        ],
    )?;

    // --- Panel C ------------------------------------------------------------
    let y = para(
        root,
        (LEFT, y + 22),
        RIGHT - LEFT,
        "Panel C: usability, which had to be measured before any row of Panel A could be. This is why the recurrent row exists at one scale only.",
        ty::HEAD,
        BLACK,
    )?;
    let head = RGBColor(110, 110, 110);
    let cols = [LEFT, 200, 290, 450, 560];
    for (x, name) in cols
        .iter()
        .zip(["arm", "scale", "completed", "voided", "diverged"])
    {
        label(root, (*x, y + 4), name, ty::NOTE, head)?;
    }
    let t_top = y + 4 + ty::line_step(ty::NOTE);
    for (i, (arm, scale, completed, voided, diverged)) in nums::USABILITY.into_iter().enumerate() {
        let y = t_top + (i as i32) * ty::line_step(ty::BODY);
        let flag = |n: u32| if n > 0 { Verdict::Fail.border() } else { BLACK };
        label(root, (cols[0], y), arm, ty::BODY, BLACK)?;
        label(root, (cols[1], y), &format!("{scale:.1}"), ty::BODY, BLACK)?;
        label(
            root,
            (cols[2], y),
            &format!("{completed} / 12"),
            ty::BODY,
            BLACK,
        )?;
        label(
            root,
            (cols[3], y),
            &format!("{voided}"),
            ty::BODY,
            flag(voided),
        )?;
        label(
            root,
            (cols[4], y),
            &format!("{diverged}"),
            ty::BODY,
            flag(diverged),
        )?;
    }
    let t_bottom = t_top + 4 * ty::line_step(ty::BODY);
    let side = 700;
    let mut sy = para(
        root,
        (side, y + 4),
        RIGHT - side,
        &format!(
            "rec+fixed's ten voids are ALL SATURATION (up to {:.1}% of hidden units pinned at maximum firing), and NONE is divergence at scale 0.4. So on the recurrent substrate",
            nums::SATURATED_FRACTION_MAX * 100.0
        ),
        ty::NOTE,
        RGBColor(70, 70, 70),
    )?;
    sy = para(
        root,
        (side, sy),
        RIGHT - side,
        "ADAPTATION IS STABILISING, the opposite of the sign that wave's own hypothesis asserted.",
        ty::NOTE,
        BLACK,
    )?;
    sy = para(
        root,
        (side, sy),
        RIGHT - side,
        "A-1 above is therefore inert AT THIS OPERATING POINT, on this substrate, not in general.",
        ty::NOTE,
        grey,
    )?;

    // Ban 3, last, where a reader leaves the figure.
    para(
        root,
        (LEFT, sy.max(t_bottom) + 16),
        RIGHT - LEFT,
        // The count comes from the row it describes rather than from the
        // sentence: "TEN" and an array that says 9 would disagree silently.
        &format!(
            "TEN PAIRS ({} of 12), THE REGISTERED MINIMUM. The two recurrent arms lost different seeds and one further loss on either would have made M-2 unreportable. Pairing REDUCES survivorship rather than removing it: the surviving pairs are those that did not diverge, and divergence is not random. The feed-forward rows carry no such exposure at 12/12.",
            nums::SUBSTRATE[2].3
        ),
        ty::NOTE,
        grey,
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
/// is **not competitive** against a 95.07-96.3% frontier, and the gain **inverts
/// at h1024**. Without them the image reads as a results claim about SHD.
fn draw_lead_graphical_abstract(root: &DrawingArea<SVGBackend<'_>, Shift>) -> Result<(), DrawErr> {
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
        "NOT SHOWN HERE, AND NOT CLAIMED: that SHD depends on temporal order. That is established: Cramer et al. 2022 could not exceed 60% on spike-count-only SHD;",
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
        "The operation, on the INPUT",
        &[
            "permute time bins,",
            "independently per sample,",
            "in BOTH the training and",
            "test splits",
            "",
            "the task must be solved",
            "without temporal order",
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
        "Each arm's OWN shuffle cost: the pair is the measurement",
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
        &COST_POSTER,
    )?;
    cost_bar(
        root,
        (660, base, 150, height),
        scale,
        nums::SHUFFLE_COST_RATE_32,
        RGBColor(200, 140, 60),
        "rate read-out",
        "ff+fixed, half the measurement,",
        "not a faint control",
        &COST_POSTER,
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
        "That is the collapse of the ADVANTAGE, which is not the gain and not a share of it; see the strip below.",
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
            "all at e400 AND e100;",
            "depth is a scope limit: h768",
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
            "Accuracy, for scale and no more: the read-out reaches {:.4} against the rate arm's {:.4}. It is NOT COMPETITIVE: the SHD frontier is {:.2}–{:.1}% via learned delays,",
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
        "adaptation and a state-space model, and this instrument carries no temporal kernel of any kind. No accuracy claim is made and none is drawn larger than the costs above.",
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
    let grey = RGBColor(100, 100, 100);
    let y = para(
        root,
        (LEFT, 8),
        RIGHT - LEFT,
        "Panel A: primary mean, grouped by verdict. Every value is feed-forward / recurrent.",
        ty::HEAD,
        BLACK,
    )?;
    // The saturated reference is the reason the passing arms cannot be
    // ordered, so it is said before any card rather than after them.
    let y = para(
        root,
        (LEFT, y),
        RIGHT - LEFT,
        "Against a SuperSpike BPTT reference at 1.0000 every PASS reduces to “the arm scored above 0.75”, so this figure does not rank them: rows are in the order of Table A. Accuracy is printed, never encoded as a length or a position.",
        ty::NOTE,
        RGBColor(70, 70, 70),
    )?;

    // Grouped by verdict, and the contrast group is labelled as ungated in the
    // group header itself rather than only in the chip.
    let groups: [VerdictGroup<'_>; 4] = [
        (
            Verdict::Fail,
            "Does not clear the gate: the lead negative, and the only one",
            &[(
                "broadcast ±1 three-factor",
                "MatchedLocal: ±1 reward × surrogate eligibility. At chance on both graphs.",
                nums::BROADCAST_PM1,
            )],
        ),
        (
            Verdict::Pass,
            "Clears the gate: unordered, and not orderable against a saturated reference",
            &[
                (
                    "graded DFA",
                    "MatchedDfaGradedError: graded error through fixed random feedback",
                    nums::DFA,
                ),
                (
                    "REINFORCE × frozen B_i",
                    "MatchedRlReinforceFb: the same ±1 reward, addressed",
                    nums::RL_FB,
                ),
                (
                    "discrete EventProp-style spike-adjoint",
                    "MatchedEventProp: the archived FAIL is WITHDRAWN; it had no spikes to differentiate through",
                    nums::EVENTPROP,
                ),
            ],
        ),
        (
            Verdict::Contrast,
            "Measured and NOT gated: neither passes to be counted nor failures to be explained away",
            &[
                (
                    "broadcast graded error",
                    "MatchedBroadcastGradedError: why the lead negative says “±1 three-factor” and not “any broadcast”",
                    nums::BROADCAST_GRADED,
                ),
                (
                    "RL graded-reward broadcast",
                    "MatchedRlGraded: richness without addressing",
                    nums::RL_GRADED,
                ),
                (
                    "RL ±1 broadcast",
                    "MatchedRlFlat: same reward and topology as the lead FAIL, 0.28 above it",
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

    // `CARD_PAPER_ROW` puts the value on the chip row and the mechanism line
    // at y0 + 56 in a 26 request, so 92 is the shortest card that keeps that
    // line off its own bottom border.
    const CARD_H: i32 = 92;
    let mut y = y + 10;
    for (verdict, heading, arms) in groups {
        label(root, (LEFT, y), heading, ty::NOTE, verdict.border())?;
        y += ty::line_step(ty::NOTE) + 2;
        for (rule, mechanism, value) in arms {
            rule_card(
                root,
                (LEFT, y, RIGHT, y + CARD_H),
                verdict,
                rule,
                mechanism,
                *value,
                &CARD_PAPER_ROW,
            )?;
            y += CARD_H + 6;
        }
        y += 10;
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
        (LEFT, panel_b),
        "Panel B: gap LCB against the > 0.5 gate: the second half of the gate, same row order.",
        ty::HEAD,
        BLACK,
    )?;

    let (lcb_lo, lcb_hi) = (-0.25, 1.05);
    let (lx0, lx1) = (450, 1150);
    let at_lcb =
        |v: f64| lx0 + (((v - lcb_lo) / (lcb_hi - lcb_lo)) * (lx1 - lx0) as f64).round() as i32;

    // Zero, then the gate. Zero is drawn because the broadcast recurrent value
    // is below it and a reader needs to see which side of nothing it is on.
    const LCB_ROW: i32 = 40;
    let strip_top = panel_b + 48;
    let strip_bottom = strip_top + 4 * LCB_ROW;
    let zero = at_lcb(0.0);
    map_draw(root.draw(&PathElement::new(
        vec![(zero, strip_top), (zero, strip_bottom)],
        RGBColor(170, 170, 176).stroke_width(1),
    )))?;
    centered(
        root,
        (zero, strip_bottom + 18),
        "0",
        ty::NOTE,
        RGBColor(120, 120, 120),
    )?;
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
        (gate, strip_bottom + 18),
        &format!("gate: gap LCB > {:.1}", nums::GATE_LCB),
        ty::NOTE,
        RGBColor(100, 100, 100),
    )?;
    // The two values are printed in a column of their own. Beside their dots
    // they ran into the gate line on the one row whose LCBs straddle it.
    let value_x = 1180;
    label(
        root,
        (value_x, strip_top - 30),
        "ff / rec",
        ty::NOTE,
        RGBColor(110, 110, 110),
    )?;

    for (i, (arm, ff, rec)) in nums::GAP_LCB.into_iter().enumerate() {
        let y = strip_top + 20 + (i as i32) * LCB_ROW;
        let verdict = if ff > nums::GATE_LCB {
            Verdict::Pass
        } else {
            Verdict::Fail
        };
        label(root, (LEFT, y - 15), arm, ty::BODY, BLACK)?;
        map_draw(root.draw(&PathElement::new(
            vec![(at_lcb(ff.min(rec)), y), (at_lcb(ff.max(rec)), y)],
            verdict.border().stroke_width(3),
        )))?;
        map_draw(root.draw(&Circle::new((at_lcb(ff), y), 9, verdict.border().filled())))?;
        map_draw(root.draw(&Circle::new(
            (at_lcb(rec), y),
            9,
            verdict.border().stroke_width(3),
        )))?;
        label(
            root,
            (value_x, y - 13),
            &format!("{ff:.4} / {rec:.4}"),
            ty::NOTE,
            RGBColor(60, 60, 60),
        )?;
    }
    label(
        root,
        (value_x - 110, strip_bottom + 46),
        "● feed-forward   ○ recurrent",
        ty::NOTE,
        RGBColor(110, 110, 110),
    )?;
    para(
        root,
        (LEFT, strip_bottom + 46),
        value_x - 150 - LEFT,
        &format!(
            "Broadcast ±1 is BELOW ZERO on the recurrent graph ({:.4}), not merely short of the gate.",
            nums::GAP_LCB[0].2
        ),
        ty::NOTE,
        grey,
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
        &BAR_POSTER,
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
        "A rule that clears the matched gate does not clear it on the live engine. Gate: local acc ≥ 0.65 AND gap LCB > 0.5, both, which is why both axes are drawn.",
        13,
        RGBColor(70, 70, 70),
    )?;

    // --- Rung 1: a different substrate, and therefore a card and not a point.
    label(
        root,
        (36, 88),
        "Rung 1: matched dense-LIF forward, n = 20",
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
        "gap LCB 0.9765 ff / 0.9079 rec: clears both halves of the gate on both forward graphs",
        nums::RL_FB,
        &CARD_POSTER,
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

    label(
        root,
        (36, y - 30),
        "Rung 2: live REINFORCE transfer",
        13,
        RGBColor(90, 90, 90),
    )?;
    row(root, y, "v13 live RFB", nums::LIVE_RFB, nums::LIVE_RFB_LCB)?;
    y += 46;

    label(
        root,
        (36, y - 22),
        "Rung 3: gap-close family, in protocol order (v14–v19)",
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
        "Rung 4: break-it family, in protocol order (v20–v24)",
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
    map_draw(root.draw(&Circle::new(
        (400, legend_y),
        7,
        RGBColor(200, 160, 60).filled(),
    )))?;
    label(
        root,
        (414, legend_y - 8),
        "cleared the 0.65 accuracy floor",
        12,
        RGBColor(110, 110, 110),
    )?;
    map_draw(root.draw(&Circle::new(
        (660, legend_y),
        7,
        Verdict::Fail.border().filled(),
    )))?;
    label(
        root,
        (674, legend_y - 8),
        "did not",
        12,
        RGBColor(110, 110, 110),
    )?;
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
            "Floor cleared is not gate cleared, and the two axes exist so that cannot be read off one. The best local below the break is v15's {:.4}; on accuracy alone it looks like a near miss.",
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
        "NO MECHANISM. Four suspects for the transfer gap (sticky last_spike, partial membrane reset, θ=∞ muting, hard k-WTA) have never been tested individually, and this figure does not attribute the gap to any of them.",
        12,
        RGBColor(120, 120, 120),
    )?;
    Ok(())
}

fn draw_graphical_abstract(root: &DrawingArea<SVGBackend<'_>, Shift>) -> Result<(), DrawErr> {
    label(root, (40, 20), "BINN: graphical abstract", 26, BLACK)?;
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
            "MatchedLocal: scalar reward, no addressing",
            nums::BROADCAST_PM1,
        ),
        (
            Verdict::Pass,
            "graded error × DFA",
            "MatchedDfaGradedError: richer error, addressed",
            nums::DFA,
        ),
        (
            Verdict::Pass,
            "REINFORCE × frozen B_i",
            "MatchedRlReinforceFb: same ±1 reward, addressed",
            nums::RL_FB,
        ),
    ];
    for (i, (verdict, title, mechanism, value)) in rules.into_iter().enumerate() {
        let y = 130 + (i as i32) * 170;
        rule_card(
            root,
            (330, y, 970, y + 150),
            verdict,
            title,
            mechanism,
            value,
            &CARD_POSTER,
        )?;
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

    /// One manuscript figure rendered to SVG text, as `write_pair_sized` draws it.
    fn render(draw: Draw, h: u32) -> String {
        let mut svg = String::new();
        {
            let root = SVGBackend::with_string(&mut svg, (W, h)).into_drawing_area();
            root.fill(&WHITE).expect("fill");
            draw(&root).expect("draw");
            root.present().expect("present");
        }
        svg
    }

    /// Every `font-size` the SVG sets, in SVG units.
    fn font_sizes(svg: &str) -> Vec<f64> {
        svg.split("font-size=\"")
            .skip(1)
            .map(|rest| {
                let value = &rest[..rest.find('"').expect("unterminated font-size")];
                value
                    .trim_end_matches("px")
                    .parse::<f64>()
                    .unwrap_or_else(|e| panic!("font-size {value:?}: {e}"))
            })
            .collect()
    }

    #[test]
    fn the_type_floor_is_seven_points_at_linewidth() {
        assert!(
            ty::printed_pt(ty::NOTE) >= 7.0,
            "{}",
            ty::printed_pt(ty::NOTE)
        );
        assert!(
            ty::printed_pt(ty::NOTE - 1) < 7.0,
            "NOTE is larger than it needs to be"
        );
        assert!(ty::NOTE < ty::BODY && ty::BODY < ty::VALUE && ty::VALUE < ty::HEAD);
    }

    /// The manuscript sets each of these at `\linewidth`, 468 pt for a
    /// `W`-wide canvas. Before 2026-09-29 their annotations printed at
    /// 2.4-4.3 pt; this renders each figure and refuses any smaller than 7.
    #[test]
    fn paper_figures_print_at_seven_points_or_more() {
        for (stem, draw, h) in PAPER_FIGURES {
            let sizes = font_sizes(&render(draw, h));
            assert!(
                sizes.len() > 20,
                "{stem}: parsed only {} font sizes",
                sizes.len()
            );
            let smallest = sizes.iter().cloned().fold(f64::INFINITY, f64::min);
            let printed = smallest * 468.0 / W as f64;
            assert!(
                printed >= 7.0,
                "{stem}: smallest text prints at {printed:.2} pt at \\linewidth"
            );
            // The SVG unit this parses must be the one `ty::printed_pt`
            // assumes, or the floor above is checking a different quantity.
            assert!(
                (printed - ty::printed_pt(ty::NOTE)).abs() < 0.05,
                "{stem}: smallest text {printed:.2} pt is not the NOTE tier"
            );
        }
    }

    /// LaTeX numbers the figures and the caption carries the title. A title
    /// drawn into the artwork disagrees with both the moment the manuscript
    /// orders them differently, and did: the artwork said "Figure 6" for a
    /// figure the manuscript typesets under another number.
    #[test]
    fn paper_figures_carry_no_figure_title_of_their_own() {
        for (stem, draw, h) in PAPER_FIGURES {
            let svg = render(draw, h);
            // Both separators: the figure labels moved from dashes to colons on
            // 2026-09-29, so a title would now most likely arrive as the second.
            for tag in ["1", "2", "3", "4", "5", "6", "7", "8", "9", "S", "M", "D"] {
                for title in [format!("Figure {tag} —"), format!("Figure {tag}:")] {
                    assert!(!svg.contains(&title), "{stem} draws the title {title:?}");
                }
            }
        }
    }

    /// Height at `\linewidth` plus the longest manuscript caption (about nine
    /// lines, 1.4 in) must fit the 9 in text block.
    #[test]
    fn paper_figures_fit_a_page_with_their_caption() {
        for (stem, _, h) in PAPER_FIGURES {
            let inches = h as f64 * 6.5 / W as f64;
            assert!(
                inches + 1.4 <= 9.0,
                "{stem}: {inches:.2} in tall at \\linewidth"
            );
        }
    }
}
