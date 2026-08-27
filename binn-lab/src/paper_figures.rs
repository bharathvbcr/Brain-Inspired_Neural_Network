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
