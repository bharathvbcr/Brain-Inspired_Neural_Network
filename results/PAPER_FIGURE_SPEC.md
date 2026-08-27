# BINN — figure / table specs (camera-ready)

> ### RENUMBERED 2026-08-27 — the lead program now has figures
>
> The manuscript leads with the **SHD attention read-out** program and demotes
> the **matched-architecture kill gate** to secondary
> ([`PAPER_DRAFT.md`](PAPER_DRAFT.md), Abstract vs “Abstract —
> matched-architecture kill gate (secondary program)”). Until today this file
> held nine figure specs and a graphical abstract, **all** of them for the
> secondary program and **none** for the lead — a grep for `shd`, `attention` or
> `shuffle` returned nothing.
>
> **Figures 1–4 are new and belong to the lead program.** The former Figures
> 1, 2, 3, 4 and 5 become **5, 6, 7, 8 and 9**. Lettered figures (0, D, M) keep
> their letters and do not move.
>
> Two things this edit does **not** do, deliberately. It does not correct figure
> numbers cited in any other document — it touches this file only, so
> cross-references elsewhere in the record still use the old numbering. And it
> does not rename a single artwork file: the files on disk keep the `fig1_`,
> `fig2_`… names they were written with, which no longer match the spec numbers
> above them. Each `Artwork target:` line says so explicitly.

## Where numbers come from

The rule is still “use numbers only from [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md)”,
and as of 2026-08-27 that sheet can carry it: it now leads with **Tables
SHD-1 … SHD-7** for the SHD read-out program and keeps **Tables A–E** for the
matched program. Cite the sheet by those labels — a bare letter is ambiguous
against [`PAPER_METRICS_FULL.md`](PAPER_METRICS_FULL.md), which defines its own
A–F.

| block | sheet | wave document behind it |
|---|---|---|
| SHD headline and the 0.80 gate | **Table SHD-1** | [`RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md`](RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md) · [`RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md`](RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md) §6 |
| SHD bin-shuffle difference-in-differences | **Table SHD-2** | [`RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md`](RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md) · W15/17 §6 |
| SHD width ladder and the h1024 threshold | **Tables SHD-3, SHD-4** | W15/17 §5 and §2 |
| SHD geometry scope | **Table SHD-5** | [`RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md`](RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md) |
| SHD resolution ladder (`fixed-tN`) | **Table SHD-6** | [`RESULT_2026-08-22_W10_RESOLUTION_LADDER.md`](RESULT_2026-08-22_W10_RESOLUTION_LADDER.md) |
| matched dense-LIF arm means and gap LCBs | **Table A** | [`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md) · [`PAPER_DRAFT.md`](PAPER_DRAFT.md) §3.1 |
| Engine C1, live transfer, XOR | **Tables B, C, D** | as cited row-by-row in the sheet |
| literature positioning (frontier, baselines, prior destruction operators) | *not in any sheet* | [`PAPER_DRAFT.md`](PAPER_DRAFT.md) §0 — **and §0 flags every one of those citations as unchecked against its primary source** |

**One quantity a figure below needs is not published anywhere.** Table SHD-2
records the n = 32 bin-shuffled **cost** and prints `—` for the n = 32 shuffled
**accuracy**; only the n = 12 absolute means (0.6983 / 0.6934) exist. Any
absolute-accuracy shuffle panel is therefore n = 12 or `TODO(source needed)`.

**No artwork exists for any figure in the lead program.** Nothing in
[`runs/2026-07-23-paper-hard-both/figures/`](runs/2026-07-23-paper-hard-both/figures/)
is an SHD figure; every file in that directory is dated 24 July in the working
tree, which is before both the 2026-08-25 matched re-run and the 2026-08-27
waves, so the secondary program's artwork on disk is stale against its own
specs as well.

**A naming collision, flagged and not resolved here.**
[`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md) calls the SHD read-out
**“The read-out program (lead)”** and the matched gate **“The matched-gate program (secondary)”**;
[`PUBLISHABLE_CLAIMS.md`](PUBLISHABLE_CLAIMS.md) calls the SHD read-out
**“the read-out program”** and the matched gate **“the matched-gate program”**. The two schemes disagree
on which letter/number belongs to which program. **This file therefore names
neither**, and refers to them only as *the SHD attention read-out* and *the
matched-architecture kill gate*. One of the two sheets should be changed before
submission; that is not this file's edit to make.

---

# Lead program — SHD attention read-out

## Figure 1 — The conditional: a difference-in-differences on the *gain* (required MUST)

**Message:** Destroying temporal order costs the **attention read-out** almost
everything it was buying and costs the **rate read-out** almost nothing. The
quantity plotted is each arm’s own shuffle cost, and the comparison between
those two costs is the paper’s central claim. It is a statement about **which
component’s contribution is order-dependent**, not a statement about SHD.

**Layout:** two arms side by side (attention `d32/L4` left, rate `ff+fixed`
right), each drawn as an intact → bin-shuffled pair, with the two shuffle costs
carried into a single difference-in-differences strip beneath. `n = 32`,
`h128`, `published-2ms`, `adjacent-sum-5`, `e400`, seed-paired.

### Panel A — each arm’s shuffle cost (the plotted quantity), n = 32

| arm | pairs | intact − bin-shuffled | positive |
|---|---:|---:|---:|
| attention `d32/L4` | 32 | **+0.1347** | **32/32** |
| rate `ff+fixed` | 32 | **+0.0142** | — |

Ratio between the two costs: **9.5×**. Source:
[`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md) **Table SHD-2** ·
[`RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md`](RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md)
§6 (H17-2 **MET**); restated in [`PAPER_DRAFT.md`](PAPER_DRAFT.md) §3.5 item 2
and in the Abstract.

### Panel B — the same result read as the collapse of the advantage

| quantity | intact | bin-shuffled |
|---|---:|---:|
| attention advantage over rate, n = 32 | **+0.1275** | **+0.0070** |
| attention advantage over rate, n = 12 | **+0.1258** | **+0.0050** |

**94.5%** of the read-out’s advantage is contingent on temporal order at n = 32
(**96%** at n = 12). Source: [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md)
**Table SHD-2**, derived-quantity block · [`PAPER_DRAFT.md`](PAPER_DRAFT.md)
§3.5 item 2, §3.7, §3.8. The shuffled advantage **+0.0050** is recomputed from
cells and is **not** the difference of the two rounded means above it (that
arithmetic gives +0.0049); do not re-derive it on the figure.

### Panel C (optional) — the n = 12 measurement this replaces, in absolute accuracy

| arm | intact | bin-shuffled | cost |
|---|---:|---:|---:|
| `ff+fixed+attn` `d32/L4` | 0.8320 | 0.6983 | **+0.1337** (12/12) |
| `ff+fixed` | 0.7062 | 0.6934 | **+0.0128** |

Ratio **10×**. Every `w9shf` cell passes the temporal audit (counts preserved,
relocated fraction ≥ 0.5), so a shuffle that failed to shuffle would have been
voided rather than scored. Per seed at n = 12 the effect falls between
**+0.0967 and +0.1568** — no seed in which it is absent. Source:
[`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md) **Table SHD-2** ·
[`RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md`](RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md).
**Absolute bin-shuffled means at n = 32 are not published** — Table SHD-2 prints
`—` for them and only the two costs exist. If Panel C is drawn in absolute
accuracy it must be **labelled n = 12**; an n = 32 absolute-accuracy version is
`TODO(source needed)`.

**Four things this figure must not be allowed to say.**

1. **It must not read as “SHD is temporal.”** That is prior art and the figure
   is not entitled to it. The dataset’s own authors could not exceed **60%** on
   spike-count-only SHD (Cramer et al., IEEE TNNLS 33(7), 2022); the
   Neuromorphic Sequential Arena (IJCAI 2025) removes temporal processing
   model-side and reports SHD falling **86.48 → 68.51**; Yu et al.
   (arXiv:2507.16043, 2025) randomise spike times at fixed counts and separately
   reverse time. **Three independent destruction operators, one conclusion, all
   of it prior to this work** ([`PAPER_DRAFT.md`](PAPER_DRAFT.md) §0). Any
   encoding that puts *accuracy under shuffle* at the centre — a big
   0.8320 → 0.6983 arrow, a single-arm before/after — reproduces the prior
   result and loses the new one. **The centre of the figure is the pair of
   costs, and the rate arm must be as visually prominent as the attention arm.**
   The novelty claim is narrow and stated as such: no published equivalent for
   *a specific component’s marginal contribution* on any neuromorphic benchmark.
2. **It must not be drawn as an ablation of the mechanism.** Bin-shuffling is
   applied **independently per sample, in both the training and the test
   split**, so the task itself becomes rate-solvable; nothing is removed from
   the model. A panel that labels the shuffled condition “attention off” or
   places it on a model-component axis inverts what was done.
3. **It must not quote +0.1577.** The wave-17 analyser merged a `d32l1`
   archived shuffled control into the `d32l4` comparison for twelve of its
   pairs and inflated the shuffle cost from **+0.1347** to **+0.1577** — the
   verdict was MET either way, the effect size was **17% high**. The corrected
   value is the only one that may be drawn
   ([`AMENDMENT_2026-08-27_H17_2_MERGED_TWO_READOUT_DEPTHS.md`](AMENDMENT_2026-08-27_H17_2_MERGED_TWO_READOUT_DEPTHS.md)).
4. **It must not imply the n = 32 run rescued anything.** Both numbers cleared
   their bars at n = 12 and clear them at n = 32 by the same margin: twenty
   further seeds move the headline gain by **+0.0017** and the shuffle cost by
   **+0.0010**. If the n = 12 and n = 32 values are shown together, that
   near-identity is the message — not a widening effect.

**Caption (required wording):**
“The conditional. Bin-shuffling the input — independently per sample, in **both
the training and test splits**, so the task becomes rate-solvable — costs the
time-axis attention read-out **+0.1347** accuracy (positive in **32/32**
seed-paired runs) and costs the rate read-out **+0.0142**, a **9.5×** ratio. The
read-out’s advantage over the rate arm falls from **+0.1275** to **+0.0070**:
**94.5% of it is contingent on temporal order.** That SHD depends on temporal
order is **not** what this figure shows — that is established (Cramer et al.
2022, ≤60% on spike-count-only SHD; two 2025 studies with model-side and
spike-time operators). What is measured here is **which component’s
contribution is the order-dependent one**, a difference-in-differences on the
*gain* rather than on accuracy. `h128`, `published-2ms`, `d32/L4`, `e400`,
n = 32.”

**Artwork target:** **none — this figure has no artwork.** Nothing has been
drawn for it and no file in
[`runs/2026-07-23-paper-hard-both/figures/`](runs/2026-07-23-paper-hard-both/figures/)
corresponds to it. A new file must be produced; do not repoint an existing
`fig*` file at this spec.
**Draft cite:** [`PAPER_DRAFT.md`](PAPER_DRAFT.md) Abstract / §3.5 item 2 / §3.8.

**Avoid:** “attention makes the network temporal”, “SHD requires timing” as this
figure’s finding, a bare intact-vs-shuffled bar pair for the attention arm
alone, any brain or ear iconography, and any framing in which the rate arm is a
faint control rather than half the measurement.

---

## Figure 2 — Headline accuracy and the 0.80 clearance

**Message:** The read-out takes the instrument from **0.7057** to **0.8332**
with every seed positive and every seed at or above 0.80 — and that number is
**not competitive**, which the figure states rather than leaves to a reader.

**Layout:** a seed-paired accuracy panel (32 paired points, rate → attention)
with the 0.80 gate as a horizontal rule, beside a **positioning strip** placing
0.8332 against the published field on the same axis.

### Panel A — headline, n = 32 (and the n = 12 registration it confirms)

| | n | `ff+fixed` | `ff+fixed+attn` `d32/L4` | gain | positive | ≥ 0.80 |
|---|---:|---:|---:|---:|---:|---:|
| published | 12 | 0.7062 | 0.8320 | **+0.1258** | 12/12 | 12/12 |
| **this wave** | **32** | **0.7057** | **0.8332** | **+0.1275** | **32/32** | **32/32** |

Budget-stable: |e400 − e200| = **0.0002**. Twenty seeds beyond the registered
twelve move the gain by **+0.0017**. Sources: [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md) **Table SHD-1** ·
[`RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md`](RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md)
§6 (H17-1 **MET**) · [`RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md`](RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md).
**n = 12 is the registered measurement and n = 32 is the confirmation; neither
supersedes the other**, and the sheet says so.

### Panel B — positioning strip (all values from [`PAPER_DRAFT.md`](PAPER_DRAFT.md) §0)

| marker | value | note |
|---|---:|---|
| SHD frontier | **95–96.4%** | learned delays (DCLS, ICLR 2024), adaptation (SE-adLIF, Nat. Commun. 2025), spiking transformers |
| STSC-SNN (Yu et al., 2022) | 92.36% | temporal attention inside the synaptic connection |
| TA-SNN (Yao et al., ICCV 2021) | 91.08% | squeeze-and-excitation attention over the time axis |
| **no-delay recurrent baseline** (Cramer et al., 2022) | **83.2 ± 1.3%** | 1024 neurons, with augmentation — **this is the anchor** |
| **this instrument** | **0.8332** | no temporal kernel of any kind |
| this instrument, rate read-out | 0.7057 | |

### Panel C (optional) — the 0.80 clearance is geometry-specific

| geometry | n | `ff+fixed` | attention mean | gain | ≥ 0.80 | verdict |
|---|---:|---:|---:|---:|---:|---|
| `adjacent-sum-5` / `published-2ms` (anchor) | 12 | 0.7062 | **0.8320** | +0.1258 | **12/12** | clears the gate |
| `published-10ms` | 12 | 0.6734 | 0.8225 | **+0.1491** | 10/12 | S-4 SUPPORTED |
| `channels-700` (standard 700-channel input) | 12 | 0.6774 | 0.7864 | +0.1090 | **6/12** | **S-1 NOT SUPPORTED**; S-2 SUPPORTED — the gain survives |

Source: [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md) **Table SHD-5** ·
[`RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md`](RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md).
The sheet's reading is the one the panel must carry: **attention buys roughly
the same amount everywhere tested; the anchor geometry is the one where that is
enough to clear the bar.** The 0.80 clearance is geometry-specific; the gain is
not.

**Four things this figure must not be allowed to say.**

1. **It must not present 0.8332 as competitive.** The strip is not decoration:
   the frontier is 95–96.4% and the honest reading is that this instrument lands
   **where an architecture carrying no temporal kernel should land**, next to the
   dataset authors’ own no-delay recurrent baseline. Drawing the panel with an
   axis that starts at 0.65, or omitting the frontier marker, would make the bar
   read as a win.
2. **It must not resolve differences the benchmark cannot.** SHD ships **no
   validation set**; Baronig et al. (2025) report the same model at
   95.81 ± 0.56 validating on test and 93.79 ± 0.76 on a proper held-out split —
   a two-point gap. **Differences below ~1.5 points between published SHD
   numbers are not reliably meaningful**, which applies to this paper’s own
   comparisons. Any ordering drawn inside that band must be visually
   de-emphasised or annotated as unresolvable.
3. **It must not plot the excluded comparison-table numbers.** Pfa-SNN 96.26,
   Event-SSMA 95.90, SpikeSCR 95.60 and d-cAdLIF 94.85 came from a secondary
   comparison table rather than a primary source and are **deliberately excluded**
   from the paper’s claims ([`PAPER_DRAFT.md`](PAPER_DRAFT.md) §0). They must not
   appear on this axis. The frontier band is the only published-field marker.
4. **It must not present the strip as verified.** Every citation in Panel B was
   assembled by a 2026-08-27 search pass, is **not** machine-checked against
   cells on disk unlike every SHD number in this paper, and
   `scripts/check_every_number.py` does not sweep §0. Each must be checked
   against its primary source before submission.

**Caption (required wording):**
“Headline accuracy. The time-axis attention read-out reaches **0.8332** against
the rate read-out’s **0.7057** (gain **+0.1275**, positive in **32/32** seeds,
**32/32 at or above 0.80**, |e400 − e200| = 0.0002). **This is not
competitive.** The SHD frontier sits at **95–96.4%**, reached by learned delays,
adaptation and spiking transformers; this instrument carries **no temporal
kernel of any kind** and lands beside the dataset authors’ own best-effort
**no-delay recurrent baseline (83.2 ± 1.3% at 1024 neurons with augmentation)**,
which is where an architecture of this class should land. The 0.80 clearance is
geometry-specific: `channels-700` reaches 0.7864. SHD ships no validation set
and differences below ~1.5 points between published numbers are not reliably
meaningful. Literature values are from a search pass and are not machine-checked
against cells.”

**Artwork target:** **none — this figure has no artwork.** No file exists for
it in [`runs/2026-07-23-paper-hard-both/figures/`](runs/2026-07-23-paper-hard-both/figures/)
and none is renamed to serve as one.
**Draft cite:** [`PAPER_DRAFT.md`](PAPER_DRAFT.md) Abstract / §0 / §3.5 item 1 / §3.8.

**Avoid:** truncated y-axes, “state of the art”, “competitive with”, leaderboard
styling, a frontier marker rendered as a faint dashed line the eye skips, and
any arrangement in which the paper’s bar is the tallest thing on the page.

---

## Figure 3 — The width ladder and the threshold

**Message:** The gain survives five rungs of width and **inverts at h1024**. The
inversion is a **threshold** — a step located between h768 and h1024 — and the
rungs below it are **not** a monotone decay the reader may extrapolate.

**Layout:** six rungs on a width axis, gain on the vertical, with a break or
step glyph placed **between h768 and h1024** rather than a line drawn through
all six points. A separate lower strip carries the three failed rescue levers.

### Panel A — the six-rung ladder (n = 12 per rung, `d32/L4`, `e400`)

| width | pairs | `ff+fixed` | `d32/L4` | gain | positive |
|---|---:|---:|---:|---:|---:|
| h128 | 12 | 0.7062 | 0.8320 | **+0.1258** | 12/12 |
| h256 | 12 | 0.7240 | 0.8206 | **+0.0966** | 12/12 |
| h384 | 12 | 0.7336 | 0.8096 | **+0.0760** | 12/12 |
| h512 | 12 | 0.7357 | 0.8233 | **+0.0876** | 12/12 |
| h768 | 12 | 0.7386 | 0.7946 | **+0.0560** | 11/12 |
| **h1024** | 12 | 0.7386 | **0.5768** | **−0.1618** | **1/12** |

Adjacent gaps: **+0.0292, +0.0206, −0.0116, +0.0316**. The drop into h1024 is
**0.2178**, **6.9×** the largest gap below it (**0.0316**) and more than twice
the registered 3× bar (**0.0947**). Source:
[`RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md`](RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md)
§5 (H16-1 **NOT MET**, H16-2 **MET**) · [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md)
**Table SHD-3**, registered in
[`PREREG_2026-08-25_THE_H1024_COLLAPSE.md`](PREREG_2026-08-25_THE_H1024_COLLAPSE.md).

### Panel B — located, and not explained: three preregistered levers at h1024/`d32/L4`

| lever | pairs | gain | positive | median epoch-mean grad norm |
|---|---:|---:|---:|---:|
| surrogate scale 0.5 | 12 | **−0.2106** | 0/12 | 142.009 |
| surrogate scale 0.25 | 12 | **−0.2565** | 0/12 | 151.391 |
| clip-grad-norm 1000.0 | 12 | **−0.0904** | 1/12 | 11.660 |
| *(unclipped arm they were to rescue)* | 12 | −0.1618 | 1/12 | 55.494 |

Every lever is negative and **worse than the arm it was meant to rescue**.
Clipping bound on a median **96 of 12,800** optimiser steps per cell (**0.75%**,
range 2–192) touching a median **37 of 400** epochs (**9.2%**), with
`unclippable_steps` **0** in every cell — so it acted, and accuracy did not
follow. At h512 the same flag is **inert**: 12/12 cells byte-identical to the
archived unclipped cells. Source: [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md) **Table SHD-4**.
Surrogate scale is separately amended as the wrong lever for a feed-forward arm
([`AMENDMENT_2026-08-26_SURROGATE_SCALE_IS_THE_WRONG_LEVER_FOR_A_FEEDFORWARD_ARM.md`](AMENDMENT_2026-08-26_SURROGATE_SCALE_IS_THE_WRONG_LEVER_FOR_A_FEEDFORWARD_ARM.md)).

**Five things this panel must not be allowed to say.**

1. **It must not imply monotonic decay above the threshold.** H16-1 —
   “the gain decays monotonically with width up to the collapse” — is **NOT
   MET**. Seed-paired, gain(h384) − gain(h512) is **−0.0116, sd 0.0253,
   negative in only 7 of 12 seeds**: h384 and h512 **are not distinguishable at
   n = 12**. A smooth fitted curve, a trend arrow, or a monotone-looking
   connecting line through the first five rungs all assert an ordering the
   measurement cannot support. **Draw the rungs as points with their pairing
   visible; if any connector is used, h384–h512 must be drawn as
   indistinguishable.**
2. **It must not read as a dip at h384.** The registration demanded strict
   ordering with 0.005 separations over quantities inside their own noise floor.
   That is a defect in the registration, **not a finding about width**, and the
   figure must not manufacture one.
3. **It must not place the collapse between h512 and h1024.** The four-rung
   reading in [`RESULT_2026-08-22_AZURE_TRUNCATED_AT_95_OF_252.md`](RESULT_2026-08-22_AZURE_TRUNCATED_AT_95_OF_252.md)
   is **superseded**: h768 is still **+0.0560**, so the step sits between
   **h768 and h1024**.
4. **It must not offer a mechanism.** All three registered levers failed, and
   the known correlate — gradient norms leaving O(1), max norm **1.13e8** at
   h1024/L4 — is a correlate, not a cause. The figure’s own words are
   **“located but unexplained.”** An annotation reading “gradient pathology”
   would claim what H15-1 refuted. The parsimonious alternative — overfitting on
   8,156 training samples — is **not excluded by anything in this paper**
   ([`PAPER_DRAFT.md`](PAPER_DRAFT.md) §0) and must not be excluded by the
   drawing either.
5. **It must not show the h1024 depth result.** `d32/L2` at h1024 reaching
   **+0.0392** (12/12) is explicitly **not claimed** — it rests on three points
   with L3 missing and two archived, and is registered as its own wave
   ([`PREREG_2026-08-27_DEPTH_IS_NOT_MONOTONE_AT_H1024.md`](PREREG_2026-08-27_DEPTH_IS_NOT_MONOTONE_AT_H1024.md)).
   Keep it off this figure entirely.

**Caption (required wording):**
“The width ladder. Attention’s gain is positive on five rungs and **inverts at
h1024** (−0.1618, positive in 1/12). The inversion is a **threshold, not the
slope continuing**: the drop into h1024 is **0.2178**, **6.9×** the largest gap
below it, so the collapse is located **between h768 and h1024** and every rung
below it remains positive. **No monotonic decay is claimed above the
threshold** — h384 and h512 are not distinguishable at twelve seeds (paired
difference −0.0116, sd 0.0253, negative in 7 of 12). Three preregistered rescue
levers (surrogate scale 0.5 and 0.25, gradient clipping at 1000.0) are all
negative and all worse than the arm they were meant to rescue, though clipping
moves the median gradient norm from 55.494 to 11.660. **The collapse is located
and unexplained**; nothing in this paper offers a mechanism for it. n = 12 per
rung, `d32/L4`, `e400`.”

**Artwork target:** **none — this figure has no artwork.** Nothing has been
drawn for it and no existing `fig*` file is renamed to it.
**Draft cite:** [`PAPER_DRAFT.md`](PAPER_DRAFT.md) Abstract / §0 / §3.5 item 4.

**Avoid:** fitted trend lines, log-width axes that visually compress the step,
“scaling law” framing, “gradient explosion” as an explanation, colour ramps that
imply an ordering among h256–h768, and any extrapolation beyond h1024.

---

## Figure 4 — The resolution ladder (`fixed-tN`)

**Message:** With the analysis window held fixed at **1400 ms** and only the
number of frames varying, the read-out helps at **every** rung, clears 0.80 at
every rung, and its advantage **shrinks as bins get finer** — the opposite of
what the withdrawn S-5 hypothesis predicted, on the axis S-5 could not isolate.

**Layout:** three rungs on a bin-width axis (coarse → fine, left → right), each
a paired rate/attention point, gain annotated; a 0.80 gate rule across all
three; the baseline drift drawn as a secondary series so the reader can see the
confound was checked.

### Panel A — `fixed-tN`, 1400 ms window fixed, n = 12 per rung

| contract | bin | `ff+fixed` | `d32/L4` | gain | gain > 0 | ≥ 0.80 |
|---|---:|---:|---:|---:|---:|---:|
| `fixed-t100` | 14.0 ms | 0.6672 | 0.8599 | **+0.1927** | **12/12** | **12/12** |
| `fixed-t250` | 5.6 ms | 0.6844 | 0.8594 | **+0.1751** | **12/12** | **12/12** |
| `fixed-t500` | 2.8 ms | 0.7069 | 0.8543 | **+0.1474** | **12/12** | **12/12** |

gain(t500) − gain(t100) = **−0.0453** against a two-sided bar of **0.03**.
Baseline drift `ff+fixed` t500 − t100 = **+0.0397**, inside the **0.05**
confound bar. Source: [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md)
**Table SHD-6** · [`RESULT_2026-08-22_W10_RESOLUTION_LADDER.md`](RESULT_2026-08-22_W10_RESOLUTION_LADDER.md),
registered in [`PREREG_2026-08-22_SHD_ATTENTION_RESOLUTION_LADDER.md`](PREREG_2026-08-22_SHD_ATTENTION_RESOLUTION_LADDER.md).

**Three things this figure must not be allowed to say.**

1. **It must not be drawn on the `published-Nms` family.** That family moves bin
   width and sequence length **together**, so no single number can be attributed
   to either; the S-5 test built on it is **refuted and withdrawn**, and
   “temporal-resolution mechanism for attention” is a standing non-claim
   ([`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md) Table SHD-5 reading and
   its non-claims list). Only
   `fixed-tN` may be plotted, and the fixed 1400 ms window must be stated on the
   figure itself — it is the whole reason the axis means anything.
2. **It must not read as the substrate improving.** The rate arm rises **+0.0397**
   across the same ladder, which was checked against a preregistered **0.05**
   confound bar and cleared. If the baseline series is omitted, the falling gain
   can be misread as the attention arm degrading rather than as the rate arm
   catching up. **Draw both series.**
3. **It must not be given a mechanism or a direction of preference.** The result
   is that the advantage shrinks with finer resolution; the paper offers no
   account of why, and does not recommend an operating point. “Attention prefers
   coarse bins” is an interpretation the evidence does not carry.

**Caption (required wording):**
“The resolution ladder. Holding the 1400 ms analysis window fixed and varying
only the number of frames, the attention read-out helps at every rung —
**+0.1927** at 14.0 ms bins, **+0.1751** at 5.6 ms, **+0.1474** at 2.8 ms — with
**12/12** seeds positive and **12/12** at or above 0.80 on each. The advantage
**shrinks with finer resolution**: gain(t500) − gain(t100) = **−0.0453** against
a two-sided bar of 0.03, the opposite of the direction the withdrawn S-5
hypothesis predicted, now asked on an axis that isolates resolution from
sequence length. The rate baseline drifts **+0.0397** across the same ladder,
inside the 0.05 confound bar, so this is a property of the read-out and not of
the substrate beneath it. n = 12 per rung.”

**Artwork target:** **none — this figure has no artwork.** No file exists for it
and none is renamed to serve as one.
**Draft cite:** [`PAPER_DRAFT.md`](PAPER_DRAFT.md) §3.5 item 5.

**Avoid:** a bin-width axis without the fixed-window annotation, `published-Nms`
values on the same axes, “optimal resolution”, and dropping the baseline series
to make the gain trend look cleaner.

---

# Secondary program — matched-architecture kill gate

> **Artwork status for everything below, updated 2026-08-27.** Four of the nine
> files in
> [`runs/2026-07-23-paper-hard-both/figures/`](runs/2026-07-23-paper-hard-both/figures/)
> are **regenerated to the 2026-08-25 re-run**:
> `figM_mechanism_richness_addressability`, `fig1_matched_rule_swap`,
> `fig3_engine_c1_means` and `graphical_abstract`. They have one owner —
> `binn-lab/src/paper_figures.rs`, run as
> `cargo run --release -p binn-lab --features plots --bin paper-figures` — which
> until today hardcoded the superseded value block quoted under Figure 6 and
> reproduced the committed files byte-for-byte from it.
> `scripts/test_paper_figures_match_the_spec.py` now parses this sheet and fails
> if the generator and it disagree again.
>
> The other five files are still dated **24 July**, before the re-run. **No
> generator produces them**, so bringing them current is an authoring task
> rather than a re-run: `fig2_matched_means` (Figure 6) and `fig4_transfer_ladder`
> (Figure 8) are stale and named as such below; `fig0_claim_axis_legend`,
> `figD_diff_closure` and `fig5_xor_locality` are unaffected by the re-run.
>
> The filenames below are the files that exist; **they are not renamed to match
> this file’s new numbering**, so `fig1_matched_rule_swap` is the artwork for
> Figure 5, `fig2_matched_means` for Figure 6, and so on.

## Graphical abstract (required)

> **Scope note added 2026-08-27.** This abstract depicts the **secondary**
> program only. The manuscript now leads with the SHD read-out, and no
> graphical abstract has been specified for the lead program:
> `TODO(source needed)` — a lead-program graphical abstract is an open
> authoring task, not a missing number.

**Message:** Same forward → ±1 × surrogate eligibility fails; every other rule tested passes against a reference at 1.0000; live k-WTA transfer fails. Disclose broadcast-graded 0.9975 elsewhere (Figure M), not as a PASS that erases the lead FAIL — and do not draw the passes as an ordering.

**Layout (left → right):**
1. Dense-LIF coincidence forward (shared box)
2. Three rule cards: broadcast ±1 3F → FAIL · DFA → PASS · RL×B → PASS
3. Arrow “transfer to live muted-θ / k-WTA C1”
4. Live RFB + gap-close → FAIL G2 (note: structured B clears acc floor only)

**Artwork target:** `runs/2026-07-23-paper-hard-both/figures/graphical_abstract.{png,pdf}` — **current as of 2026-08-27**: regenerated from the 2026-08-25 re-run, with the two passes stacked rather than ranked and the ±1 broadcast REINFORCE figure disclosed beside the broadcast-graded one.

**Avoid:** brain icons, “solved,” Assembly Calculus branding, bare “broadcast credit topology.”

---

## Figure M — Mechanism: richness × addressability (required MUST)

**Message:** Lead FAIL is **broadcast ±1 three-factor**, not “any broadcast.” Richness and addressability are separable; XOR supplies locality evidence.

**Layout:** 2×2 panel + XOR locality-flip row beneath.

### Panel A — Coincidence (matched dense-LIF), richness × addressability

> **REDRAWN 2026-08-25.** The previous version of this panel was a graded
> surface — 0.5000, 0.9200, 0.9387, 0.9863 — and invited the reading that
> richness and addressability each buy accuracy. On the repaired instrument it
> is **a cliff with one cell below it**, and the figure must show that instead.
> Numbers are feed-forward / recurrent, n = 20 each
> ([`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md)).

|  | Low addressability (broadcast) | High addressability (directed / local feedback) |
|---|---|---|
| **Low richness (±1)** | **`MatchedLocal` ±1 × surrogate eligibility — FAIL, 0.5000 / 0.5100**<br>**`MatchedRlFlat` ±1 broadcast REINFORCE — 0.7775 / 0.7962** | **REINFORCE × frozen `B_i` — PASS, 0.9950 / 0.9812** |
| **High richness (graded)** | **Broadcast-graded — 0.9975** (DFA schedule contrast) | **Graded DFA — PASS, 0.9925 / 0.9875** |

Gradient ceiling callout, and it is now the point of the panel rather than an
aside: **SuperSpike BPTT = 1.0000 in every suite, on both graphs.**

**Two things this panel must not be allowed to say.**

1. **It is not a graded surface.** Six of the seven arms sit between 0.78 and
   1.00 against a ceiling of 1.0000; one sits at chance. Any visual encoding
   that maps accuracy to a continuous ramp will manufacture an ordering the
   task cannot support — with the reference at 1.0000 every pass reduces to
   "above 0.75". Encode **pass / fail / at-chance**, not a gradient.
2. **The low/low cell holds two different rules that disagree by 0.28.**
   `MatchedLocal` (±1 × surrogate eligibility) is at chance; `MatchedRlFlat`
   (±1 broadcast REINFORCE) reaches 0.78. Collapsing them into one "broadcast
   ±1" cell is exactly the overreach the lead claim's wording exists to avoid,
   and it would be a stronger version of the same error than the one the
   0.9863 disclosure was added to prevent. **Both must be drawn, labelled by
   rule and not by topology.**

### Panel B — XOR locality flip (supporting task)

| Arm | Accuracy | Reading |
|---|---:|---|
| Broadcast (err_broadcast) | 0.5008 | chance |
| DFA | 0.8267 | solves |
| Gradient | 0.7733 | ceiling |

Source: [`deep_xor_thresh.json`](deep_xor_thresh.json). Do **not** claim the same flip for mid-init depth locality (broadcast also solves there).

**Caption (required wording):**  
“Mechanism evidence for H\*: richness × addressability on a matched dense-LIF forward, n = 20 per cell. The lead matched FAIL is **±1 × surrogate eligibility** specifically — not a ban on every broadcast scalar (broadcast-graded reaches 0.9975) and not on every ±1 rule (±1 broadcast REINFORCE reaches 0.78). Against a SuperSpike BPTT reference at **1.0000**, every other rule tested clears the gate, so this panel shows **which single rule fails a task the rest saturate** and does not rank the rest. Locality / addressability as a necessary ingredient is the XOR flip (broadcast fails; DFA solves), not coincidence alone. Matched PASS still does not imply live muted-θ / k-WTA G2 PASS.”

**Artwork target:** `runs/2026-07-23-paper-hard-both/figures/figM_mechanism_richness_addressability.{png,pdf}` — **redrawn 2026-08-27** to this specification. Panel A encodes pass / fail / contrast as labelled chips with no accuracy-to-size mapping; the low-richness / low-addressability cell draws `MatchedLocal` and `MatchedRlFlat` as two rules with the 0.28 between them stated; the saturated reference leads the panel. Panel B carries an explicit chance line at 0.50, so the broadcast bar cannot read as “half as good” rather than “did not learn”. Not renamed.  
**Draft cite:** [`PAPER_DRAFT.md`](PAPER_DRAFT.md) §3.1 / §4.1.

---

## Figure 5 — Matched rule-swap schematic *(was Figure 1)*

- One forward graph; three update plugs (broadcast ±1 3F / DFA / RL×B) + BPTT ceiling.
- Caption: “Forward held fixed; only the update rule changes. Lead FAIL label: broadcast ±1 three-factor.”

**Artwork target:** `runs/2026-07-23-paper-hard-both/figures/fig1_matched_rule_swap.{png,pdf}` — **current as of 2026-08-27**: regenerated from the 2026-08-25 re-run; it previously drew the superseded block. The `fig1_` name is historical and is **not** renamed to `fig5_`.

## Figure 6 — Matched means (bar or forest) *(was Figure 2)*

> **VALUE BLOCK REPLACED 2026-08-27 — the previous one was superseded.** It read
> DFA **0.9387**, RL **0.9200**, gradient ceiling **0.8963 / 0.8963 / 0.8887**,
> and gap LCBs **0.0000 / 0.6894 / 0.6846**. None of those values appears in
> [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md) any longer: they are
> pre-repair figures from a forward pass that emitted **zero spikes at any
> seed**, and Table A of that sheet carries its own `SUPERSEDED IN PART` banner.
> The archived block is recorded here as **superseded and not for drawing**; the
> current figures are below. The `fig2_matched_means` artwork on disk still
> plots the superseded values.

Current values — 2026-08-25 re-run at `MATCHED_INPUT_SCALE = 2.0`, n = 20,
feed-forward / recurrent
([`PAPER_DRAFT.md`](PAPER_DRAFT.md) §3.1 · [`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md)):

| Series | Feed-forward | Recurrent | Verdict |
|---|---:|---:|---|
| Broadcast ±1 three-factor | **0.5000** | **0.5100** | **FAIL**, both |
| Graded DFA | 0.9925 | 0.9875 | PASS, both |
| REINFORCE × frozen `B_i` | 0.9950 | 0.9812 | PASS, both |
| Discrete EventProp-style spike-adjoint | 0.9450 | 0.8900 | PASS, both |
| Broadcast graded error | 0.9975 | 0.9975 | contrast |
| RL graded-reward broadcast | 0.8787 | 0.9100 | contrast |
| RL ±1 broadcast | 0.7775 | 0.7962 | contrast |
| **SuperSpike BPTT ceiling** | **1.0000** | **1.0000** | reference |

Optional second panel — gap LCB, feed-forward / recurrent, with a horizontal
line at 0.5 ([`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md) Table A):
broadcast ±1 **0.0000 / −0.0192**; DFA **0.9689 / 0.9509**; RL **0.9765 /
0.9079**; EventProp **0.7911 / 0.6494**.

**Disclose in caption:** the lead FAIL is **broadcast ±1 three-factor**
specifically; on the DFA schedule the broadcast-*graded* contrast reaches
**0.9975** (shown primarily in Figure M).

**This figure must not rank the passing arms.** With the reference at 1.0000
every PASS reduces to “the arm scored above 0.75”, and no ordering among them
may be claimed. Encode pass / fail / at-chance, as Figure M does.

**Artwork target:** `runs/2026-07-23-paper-hard-both/figures/fig2_matched_means.{png,pdf}` — **exists but is stale**: it plots the superseded value block above. **No generator produces this file**, so unlike Figure M it cannot be brought current by a re-run; it has to be authored. Not renamed.

## Figure 7 — Engine C1 condition means *(was Figure 3)*

From [`c1_g2.md`](c1_g2.md): local / dense / gradient / eligibility means + PC.  
Callout box: H1/H2/θ=∞/`project` unused.

**Artwork target:** `runs/2026-07-23-paper-hard-both/figures/fig3_engine_c1_means.{png,pdf}` — **exists**, and gained a chance line at 0.50 on 2026-08-27: local-assembly (0.4912) and dense-local (0.5000) are at chance on a two-class task, which bars from a zero baseline made look like half the reference rather than no learning. `fig3_` name is historical and not renamed.

## Figure 8 — Transfer ladder *(was Figure 4)*

Vertical or stepped:
1. Matched RL PASS (0.9950 ff / LCB 0.9765)
2. Live RFB FAIL (0.4900 / LCB 0.0737)
3. Gap-close strip: v14–v19 locals + LCBs (highlight v15 acc, v17 best LCB; v19 teach ≤ v15)
4. Break-it strip: v20–v24 (v20 best local 0.7325 still gap-short; v21 chance; v22 chance; v23 floor; v24 < v15)

Dashed line at acc 0.65 and gap LCB 0.5.

> **Rung 1 updated 2026-08-27.** It read “Matched RL PASS (0.9200 / LCB
> 0.6846)”, which is the same superseded pre-repair block as the old Figure 2.
> The current matched RL figures are **0.9950 ff / 0.9812 rec**, gap LCB
> **0.9765 ff / 0.9079 rec** ([`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md)
> Table A). Rungs 2–4 are live-transfer rows and are **unaffected** by the
> matched re-run — none of them runs on the matched dense-LIF forward.

**Artwork target:** `runs/2026-07-23-paper-hard-both/figures/fig4_transfer_ladder.{png,pdf}` — **exists but is stale at rung 1**. **No generator produces this file**; bringing rung 1 current is an authoring task. `fig4_` name is historical and not renamed.

## Figure 0 — Claim-axis legend

Novel-CS / Brain-motif-under-test / Integrity cards. Lettered; unaffected by the renumbering.  
**Artwork target:** `runs/2026-07-23-paper-hard-both/figures/fig0_claim_axis_legend.{png,pdf}` — **exists**.

## Figure D — Differential closure

Green/red/gray cells for D1–D22; zero empty. Lettered; unaffected by the renumbering.  
**Artwork target:** `runs/2026-07-23-paper-hard-both/figures/figD_diff_closure.{png,pdf}` — **exists**.

## Figure 9 (optional) — XOR locality *(was Figure 5)*

Bar: broadcast 0.5008 / DFA 0.8267 / gradient 0.7733 from `xor_thresh`
([`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md) Table D).  
Caption: “Locality flip on 1-layer XOR; not claimed for 2-layer depth. Supporting evidence for Figure M addressability axis.”  
**Artwork target:** `runs/2026-07-23-paper-hard-both/figures/fig5_xor_locality.{png,pdf}` — **exists**; the `fig5_` name now collides with **Figure 5** (matched rule-swap) in this spec and is **deliberately not renamed**. Resolve by filename, not by number.

---

## Table placement

Renumbered on 2026-08-27 alongside the figures: the SHD read-out tables lead.
The SHD program previously had **no row in this map at all**. The *paper*
tables are renumbered here; the *source sheet* labels (`SHD-1 … SHD-7`, `A`–`E`)
are the sheet's own and are cited unchanged, because
[`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md) states that renumbering its
letters would break the references in this file and in
[`PAPER_SKELETON.md`](PAPER_SKELETON.md).

| Paper table | Content | Source sheet |
|---|---|---|
| **Table 1 — SHD headline against the 0.80 gate** | n = 32 headline (0.8332 / 0.7057 / +0.1275, 32/32 ≥ 0.80) and the n = 12 registration it confirms | `PAPER_RESULTS_TABLE` **Table SHD-1** |
| **Table 2 — SHD bin-shuffle difference-in-differences** | the two shuffle costs (+0.1347 / +0.0142, 9.5×) and the advantage collapse (+0.1275 → +0.0070) | `PAPER_RESULTS_TABLE` **Table SHD-2** |
| **Table 3 — SHD scope limits** | six-rung width ladder and the h1024 threshold; the three failed rescue levers; geometry scope | `PAPER_RESULTS_TABLE` **Tables SHD-3, SHD-4, SHD-5** |
| **Table S0 — SHD resolution ladder** | `fixed-t100/250/500` at a fixed 1400 ms window | `PAPER_RESULTS_TABLE` **Table SHD-6** |
| **Table S0b — SHD substrate comparison** | `ff+fixed` / `ff+alif` / `rec+alif`; supports §3.7, no figure specified | `PAPER_RESULTS_TABLE` **Table SHD-7** |
| Table 4 — matched gate *(was Table 1)* | matched dense-LIF arm means and gap LCBs | `PAPER_RESULTS_TABLE` **Table A** + [`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md) / `PAPER_METRICS_FULL` A |
| Table 5 — C1 / integrity *(was Table 2)* | Engine C1 conditions | `PAPER_RESULTS_TABLE` **Table B** + Appendix A |
| Table 6 — transfer / gap-close / break-it *(was Table 3)* | v13–v24 | `PAPER_RESULTS_TABLE` **Table C** / `PAPER_METRICS_FULL` C |
| Table S1 — XOR / depth | supporting NumPy tasks | `PAPER_RESULTS_TABLE` **Table D** |
| Table S2 — methods footnotes | | `PAPER_RESULTS_TABLE` **Table E** |
| Table S3 — dual-gap / seed diagnostics | | [`PAPER_METRICS_FULL.md`](PAPER_METRICS_FULL.md) |

**Table SHD-7 has no figure in this spec.** The substrate comparison (§3.7 of
the draft) is specified as a table only; if it is later promoted to a figure it
needs its own spec, and the four load-bearing limits in §3.7 — the lower base,
that the recurrent substrate does not win, the numerical extremity, and reduced
survivorship — must travel with it. `TODO(source needed)` does not apply: the
numbers exist; the figure spec does not.
