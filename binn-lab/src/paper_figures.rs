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
    /// SuperSpike BPTT. Saturates in every suite on both graphs, which is what
    /// makes the passing arms unrankable: every one of them reduces to
    /// "above 0.75" against a reference at 1.
    pub const CEILING: Both = (1.0000, 1.0000);

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

fn both(v: nums::Both) -> String {
    format!("{:.4} ff  /  {:.4} rec", v.0, v.1)
}

const W: u32 = 1400;
const H: u32 = 900;

/// Generate required camera-ready figures into `out_dir`.
pub fn generate_all(out_dir: &Path) -> Result<Vec<PathBuf>, DrawErr> {
    fs::create_dir_all(out_dir)?;
    let mut written = Vec::new();
    // Lead program first: it is what the manuscript leads with, and until
    // 2026-08-27 nothing had been drawn for any of its four figures.
    written.extend(write_pair(
        out_dir,
        "leadfig1_the_conditional",
        draw_lead_fig1,
    )?);
    written.extend(write_pair(out_dir, "leadfig2_headline_accuracy", draw_lead_fig2)?);
    written.extend(write_pair(out_dir, "leadfig3_width_ladder", draw_lead_fig3)?);
    written.extend(write_pair(out_dir, "leadfig4_resolution_ladder", draw_lead_fig4)?);
    written.extend(write_pair(
        out_dir,
        "figM_mechanism_richness_addressability",
        draw_fig_m,
    )?);
    written.extend(write_pair(out_dir, "fig1_matched_rule_swap", draw_fig1)?);
    written.extend(write_pair(out_dir, "fig3_engine_c1_means", draw_fig3)?);
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
    let png = out_dir.join(format!("{stem}.png"));
    let pdf = out_dir.join(format!("{stem}.pdf"));
    let svg_path = out_dir.join(format!("{stem}.svg.tmp"));

    {
        let root = SVGBackend::new(&svg_path, (W, H)).into_drawing_area();
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
    let mut pixmap = resvg::tiny_skia::Pixmap::new(W, H).ok_or("pixmap alloc")?;
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

    // --- Panel B ------------------------------------------------------------
    label(
        root,
        (36, 606),
        "Panel B — located, and not explained. Three preregistered rescue levers at h1024 / d32/L4, n = 12 each.",
        16,
        BLACK,
    )?;
    label(
        root,
        (36, 630),
        "Every lever is negative, and every one is worse than the arm it was meant to rescue.",
        12,
        RGBColor(110, 110, 110),
    )?;
    for (i, (lever, gain, positive, norm)) in nums::LEVERS.into_iter().enumerate() {
        let y = 664 + (i as i32) * 34;
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
        (36, 810),
        "Clipping moved the median gradient norm from 55.494 to 11.660 and accuracy did not follow. At h512 the same flag is inert: 12/12 cells byte-identical to the archived unclipped cells.",
        12,
        RGBColor(110, 110, 110),
    )?;
    label(
        root,
        (36, 840),
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
        assert!(written.len() >= 8);
        for stem in [
            "figM_mechanism_richness_addressability",
            "fig1_matched_rule_swap",
            "fig3_engine_c1_means",
            "graphical_abstract",
        ] {
            assert!(dir.join(format!("{stem}.png")).is_file(), "{stem}.png");
            assert!(dir.join(format!("{stem}.pdf")).is_file(), "{stem}.pdf");
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
