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
mod nums {
    pub const BROADCAST_PM1: f64 = 0.5000;
    pub const BROADCAST_GRADED: f64 = 0.9863;
    pub const DFA: f64 = 0.9387;
    pub const RL_FB: f64 = 0.9200;
    pub const GRAD_MATCH: f64 = 0.8963;
    pub const XOR_BCAST: f64 = 0.5008;
    pub const XOR_DFA: f64 = 0.8267;
    pub const XOR_GRAD: f64 = 0.7733;
    pub const C1_LOCAL: f64 = 0.4912;
    pub const C1_DENSE: f64 = 0.5000;
    pub const C1_GRAD: f64 = 0.8938;
    pub const C1_ELIG: f64 = 1.0000;
    pub const C1_PC: f64 = 0.9488;
    pub const LIVE_RFB: f64 = 0.4900;
}

const W: u32 = 1400;
const H: u32 = 900;

/// Generate required camera-ready figures into `out_dir`.
pub fn generate_all(out_dir: &Path) -> Result<Vec<PathBuf>, DrawErr> {
    fs::create_dir_all(out_dir)?;
    let mut written = Vec::new();
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

fn draw_bar_row(
    root: &DrawingArea<SVGBackend<'_>, Shift>,
    x0: i32,
    y0: i32,
    width: i32,
    height: i32,
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
    Ok(())
}

fn draw_fig_m(root: &DrawingArea<SVGBackend<'_>, Shift>) -> Result<(), DrawErr> {
    label(
        root,
        (40, 20),
        "Figure M — Mechanism: richness × addressability",
        26,
        BLACK,
    )?;
    label(
        root,
        (40, 55),
        "Lead FAIL = broadcast ±1 three-factor (not every broadcast). XOR supplies locality evidence.",
        14,
        RGBColor(60, 60, 60),
    )?;

    label(
        root,
        (280, 100),
        "Low addressability (broadcast)",
        14,
        BLACK,
    )?;
    label(
        root,
        (780, 100),
        "High addressability (directed / local FB)",
        14,
        BLACK,
    )?;
    label(root, (40, 220), "Low richness (±1 / flat)", 13, BLACK)?;
    label(root, (40, 400), "High richness (graded)", 13, BLACK)?;

    let fail = RGBColor(220, 90, 90);
    let pass = RGBColor(90, 170, 110);
    let disclose = RGBColor(230, 200, 90);

    box_card(
        root,
        (200, 130, 620, 300),
        RGBColor(255, 235, 235),
        fail,
        "Broadcast ±1 three-factor  FAIL",
        &[
            &format!("acc {:.4}", nums::BROADCAST_PM1),
            "gap LCB 0.0000",
            "c1-match-5dc6822e71229e9e",
        ],
    )?;
    box_card(
        root,
        (700, 130, 1280, 300),
        RGBColor(230, 250, 235),
        pass,
        "REINFORCE × frozen B  PASS",
        &[
            &format!("acc {:.4}", nums::RL_FB),
            "gap LCB 0.6846",
            "c1-rl-42eddc9c801308e9",
        ],
    )?;
    box_card(
        root,
        (200, 330, 620, 500),
        RGBColor(255, 250, 220),
        disclose,
        "Broadcast-graded  (disclose)",
        &[
            &format!("acc {:.4} on DFA schedule", nums::BROADCAST_GRADED),
            "not a locality proof",
            "c1-dfa-c8c4fe0899908b84 contrast",
        ],
    )?;
    box_card(
        root,
        (700, 330, 1280, 500),
        RGBColor(230, 250, 235),
        pass,
        "Graded DFA  PASS",
        &[
            &format!("acc {:.4}", nums::DFA),
            "gap LCB 0.6894",
            "c1-dfa-c8c4fe0899908b84",
        ],
    )?;

    map_draw(root.draw(&Rectangle::new(
        [(40, 540), (1360, 860)],
        RGBColor(245, 245, 248).filled(),
    )))?;
    label(
        root,
        (60, 560),
        "Panel B — XOR locality flip (1-layer; deep_xor_thresh.json)",
        18,
        BLACK,
    )?;
    draw_bar_row(
        root,
        100,
        620,
        1200,
        200,
        &[
            ("Broadcast", nums::XOR_BCAST, fail),
            ("DFA", nums::XOR_DFA, pass),
            ("Gradient", nums::XOR_GRAD, RGBColor(90, 120, 200)),
        ],
    )?;
    label(
        root,
        (60, 840),
        "Caption: lead matched FAIL is broadcast ±1 three-factor; locality evidence is the XOR flip, not coincidence alone.",
        12,
        RGBColor(50, 50, 50),
    )?;
    Ok(())
}

fn draw_fig1(root: &DrawingArea<SVGBackend<'_>, Shift>) -> Result<(), DrawErr> {
    label(
        root,
        (40, 24),
        "Figure 1 — Matched rule-swap schematic",
        26,
        BLACK,
    )?;
    label(
        root,
        (40, 60),
        "Forward held fixed; only the update rule changes. Lead FAIL label: broadcast ±1 three-factor.",
        14,
        RGBColor(60, 60, 60),
    )?;

    box_card(
        root,
        (80, 140, 520, 420),
        RGBColor(235, 240, 255),
        RGBColor(70, 100, 180),
        "Dense-LIF coincidence forward",
        &[
            "shared win / wrec / wout",
            "continuous frames",
            &format!("SuperSpike ceiling {:.4}", nums::GRAD_MATCH),
        ],
    )?;

    let plugs = [
        (
            620,
            120,
            "Broadcast ±1 3F",
            format!("FAIL  {:.4}", nums::BROADCAST_PM1),
            RGBColor(220, 90, 90),
        ),
        (
            620,
            300,
            "DFA graded×B",
            format!("PASS  {:.4}", nums::DFA),
            RGBColor(90, 170, 110),
        ),
        (
            620,
            480,
            "RL reinforce_fb",
            format!("PASS  {:.4}", nums::RL_FB),
            RGBColor(90, 170, 110),
        ),
        (
            620,
            660,
            "BPTT / SuperSpike",
            format!("ceiling  {:.4}", nums::GRAD_MATCH),
            RGBColor(90, 120, 200),
        ),
    ];
    for (x, y, title, body, color) in plugs {
        map_draw(root.draw(&PathElement::new(
            vec![(520, 280), (x, y + 50)],
            color.stroke_width(2),
        )))?;
        box_card(
            root,
            (x, y, x + 640, y + 120),
            RGBColor(250, 250, 250),
            color,
            title,
            &[&body],
        )?;
    }
    Ok(())
}

fn draw_fig3(root: &DrawingArea<SVGBackend<'_>, Shift>) -> Result<(), DrawErr> {
    label(
        root,
        (40, 24),
        "Figure 3 — Engine C1 condition means",
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
        1000,
        520,
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
        "Same forward → broadcast ±1 three-factor fails; richer credit passes; live k-WTA transfer fails.",
        14,
        RGBColor(60, 60, 60),
    )?;

    box_card(
        root,
        (40, 120, 300, 360),
        RGBColor(235, 240, 255),
        RGBColor(70, 100, 180),
        "Dense-LIF forward",
        &["coincidence task", "shared architecture"],
    )?;

    let rules = [
        (
            340,
            "±1 3F",
            format!("FAIL {:.2}", nums::BROADCAST_PM1),
            RGBColor(220, 90, 90),
        ),
        (
            560,
            "DFA",
            format!("PASS {:.2}", nums::DFA),
            RGBColor(90, 170, 110),
        ),
        (
            780,
            "RL×B",
            format!("PASS {:.2}", nums::RL_FB),
            RGBColor(90, 170, 110),
        ),
    ];
    for (x, title, body, color) in rules {
        box_card(
            root,
            (x, 140, x + 200, 340),
            RGBColor(250, 250, 250),
            color,
            title,
            &[&body],
        )?;
    }

    map_draw(root.draw(&PathElement::new(
        vec![(1000, 250), (1120, 250)],
        BLACK.stroke_width(3),
    )))?;
    centered(root, (1060, 220), "transfer →", 14, BLACK)?;
    centered(
        root,
        (1060, 280),
        "live muted-θ / k-WTA",
        12,
        RGBColor(80, 80, 80),
    )?;

    box_card(
        root,
        (1140, 140, 1360, 360),
        RGBColor(255, 235, 235),
        RGBColor(220, 90, 90),
        "Live RFB → G2 FAIL",
        &[
            &format!("acc {:.4}", nums::LIVE_RFB),
            "gap-close still < 0.5",
            "v13–v24 package",
        ],
    )?;

    label(
        root,
        (40, 420),
        "Disclose elsewhere (Figure M): broadcast-graded 0.9863 on DFA schedule — does not erase the ±1 lead FAIL.",
        13,
        RGBColor(80, 80, 80),
    )?;
    label(
        root,
        (40, 460),
        "Avoid: brain icons, “solved,” Assembly Calculus branding, bare “broadcast credit topology.”",
        12,
        RGBColor(100, 100, 100),
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
