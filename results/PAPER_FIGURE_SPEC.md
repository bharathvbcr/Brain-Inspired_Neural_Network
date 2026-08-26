# BINN — figure / table specs (camera-ready)

Use numbers only from [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md).

---

## Graphical abstract (required)

**Message:** Same forward → ±1 × surrogate eligibility fails; every other rule tested passes against a reference at 1.0000; live k-WTA transfer fails. Disclose broadcast-graded 0.9975 elsewhere (Figure M), not as a PASS that erases the lead FAIL — and do not draw the passes as an ordering.

**Layout (left → right):**
1. Dense-LIF coincidence forward (shared box)
2. Three rule cards: broadcast ±1 3F → FAIL · DFA → PASS · RL×B → PASS
3. Arrow “transfer to live muted-θ / k-WTA C1”
4. Live RFB + gap-close → FAIL G2 (note: structured B clears acc floor only)

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

**Artwork target:** `runs/2026-07-23-paper-hard-both/figures/figM_mechanism_richness_addressability.{png,pdf}`  
**Draft cite:** [`PAPER_DRAFT.md`](PAPER_DRAFT.md) §3.1 / §4.1.

---

## Figure 1 — Matched rule-swap schematic

- One forward graph; three update plugs (broadcast ±1 3F / DFA / RL×B) + BPTT ceiling.
- Caption: “Forward held fixed; only the update rule changes. Lead FAIL label: broadcast ±1 three-factor.”

## Figure 2 — Matched means (bar or forest)

| Series | Values |
|---|---|
| Broadcast ±1 3F | 0.5000 |
| DFA | 0.9387 |
| RL reinforce_fb | 0.9200 |
| Gradient ceiling (match / dfa / rl) | 0.8963 / 0.8963 / 0.8887 |

Optional second panel: gap LCB 0.0000 / 0.6894 / 0.6846 with horizontal line at 0.5.

**Disclose in caption:** On the DFA schedule, broadcast-graded contrast = 0.9863 (shown primarily in Figure M); lead FAIL is **broadcast ±1 three-factor**.

## Figure 3 — Engine C1 condition means

From [`c1_g2.md`](c1_g2.md): local / dense / gradient / eligibility means + PC.  
Callout box: H1/H2/θ=∞/`project` unused.

## Figure 4 — Transfer ladder

Vertical or stepped:
1. Matched RL PASS (0.9200 / LCB 0.6846)
2. Live RFB FAIL (0.4900 / LCB 0.0737)
3. Gap-close strip: v14–v19 locals + LCBs (highlight v15 acc, v17 best LCB; v19 teach ≤ v15)
4. Break-it strip: v20–v24 (v20 best local 0.7325 still gap-short; v21 chance; v22 chance; v23 floor; v24 < v15)

Dashed line at acc 0.65 and gap LCB 0.5.

**Artwork:** `runs/2026-07-23-paper-hard-both/figures/fig4_transfer_ladder.{png,pdf}`

## Figure 0 — Claim-axis legend

Novel-CS / Brain-motif-under-test / Integrity cards.  
**Artwork:** `…/figures/fig0_claim_axis_legend.{png,pdf}`

## Figure D — Differential closure

Green/red/gray cells for D1–D22; zero empty.  
**Artwork:** `…/figures/figD_diff_closure.{png,pdf}`

## Figure 5 (optional) — XOR locality

Bar: broadcast 0.50 / DFA 0.83 / grad 0.77 from `xor_thresh`.  
Caption: “Locality flip on 1-layer XOR; not claimed for 2-layer depth. Supporting evidence for Figure M addressability axis.”  
**Artwork:** `…/figures/fig5_xor_locality.{png,pdf}`

---

## Table placement

| Paper table | Source sheet |
|---|---|
| Table 1 matched gate | Results Table A / PAPER_METRICS_FULL A |
| Table 2 C1 / integrity | Results Table B + Appendix A |
| Table 3 transfer / gap-close / break-it | Results Table C / PAPER_METRICS_FULL C |
| Table S1 XOR / depth | Results Table D |
| Table S2 methods footnotes | Results Table E |
| Table S3 dual-gap / seed diagnostics | PAPER_METRICS_FULL |
