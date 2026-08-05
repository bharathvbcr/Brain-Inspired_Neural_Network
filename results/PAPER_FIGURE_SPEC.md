# BINN — figure / table specs (camera-ready)

Use numbers only from [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md).

---

## Graphical abstract (required)

**Message:** Same forward → broadcast ±1 three-factor fails; richer credit (DFA / REINFORCE×B) passes; live k-WTA transfer fails. Disclose broadcast-graded 0.9863 elsewhere (Figure M), not as a PASS that erases the lead FAIL.

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

|  | Low addressability (broadcast) | High addressability (directed / local feedback) |
|---|---|---|
| **Low richness (±1 / flat)** | **Broadcast ±1 three-factor FAIL** — 0.5000 / gap LCB 0.0000 (`c1-match-5dc6822e71229e9e`) | **REINFORCE × frozen B PASS** — 0.9200 / gap LCB 0.6846 (`c1-rl-42eddc9c801308e9`) |
| **High richness (graded)** | **Broadcast-graded** — **0.9863** on DFA schedule (`c1-dfa-c8c4fe0899908b84` contrast; disclose) | **Graded DFA PASS** — 0.9387 / gap LCB 0.6894 (`c1-dfa-c8c4fe0899908b84`) |

Optional callout on gradient ceiling: SuperSpike BPTT 0.8963 (match/dfa) / 0.8887 (rl).

### Panel B — XOR locality flip (supporting task)

| Arm | Accuracy | Reading |
|---|---:|---|
| Broadcast (err_broadcast) | 0.5008 | chance |
| DFA | 0.8267 | solves |
| Gradient | 0.7733 | ceiling |

Source: [`deep_xor_thresh.json`](deep_xor_thresh.json). Do **not** claim the same flip for mid-init depth locality (broadcast also solves there).

**Caption (required wording):**  
“Mechanism evidence for H\*: richness × addressability. The lead matched FAIL is **broadcast ±1 three-factor** (eligibility × ±1), not a ban on every broadcast scalar—broadcast-graded reaches **0.9863** on the coincidence DFA schedule. Locality / addressability as a necessary ingredient is the XOR flip (broadcast fails; DFA solves), not coincidence alone. Matched PASS still does not imply live muted-θ / k-WTA G2 PASS.”

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
