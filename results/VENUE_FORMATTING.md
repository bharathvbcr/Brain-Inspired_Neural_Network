# Venue formatting notes (skeleton)

Status: **skeleton + bibliography stubs** (REPRO §G partially checked). Final camera-ready style pass still open once venue is chosen.

Companion: [`references.bib`](references.bib) · [`PAPER_DRAFT.md`](PAPER_DRAFT.md) · [`PAPER_SKELETON.md`](PAPER_SKELETON.md)

---

## Target venue classes (pick one)

| Class | Fit | Style notes |
|---|---|---|
| Negative-results / ML methods | **Best fit** for matched ±1 FAIL + transfer FAIL | Often NeurIPS/ICML workshop or TMLR-like; anonymous PDF; ~8–12 pp |
| Neuromorphic / SNN methods | Possible if claim language stays software-harness | Emphasize SuperSpike ceiling + discrete EventProp FAIL; refuse biology |
| “Brain-like AI” venues | **Avoid** unless claims rewritten down | Would overclaim Assembly Calculus / cortex |

**Working default:** methods / negative-result track; anonymous submission; single-column or venue template TBD.

---

## Formatting checklist

- [x] Claim ladder frozen ([`PUBLISHABLE_CLAIMS.md`](PUBLISHABLE_CLAIMS.md))
- [x] Cite-every-number table ([`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md))
- [x] Prose draft with §3.1 ↔ §4.2 honesty ([`PAPER_DRAFT.md`](PAPER_DRAFT.md))
- [x] Bibliography stubs ([`references.bib`](references.bib))
- [ ] Venue template applied (NeurIPS/ICML/TMLR/… `.sty` / Overleaf)
- [ ] Anonymous PDF build
- [x] Figure artwork complete (figM, fig1, fig3, graphical abstract — camp `figures/`)
- [ ] Caption pass against [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md)
- [ ] Page budget / appendix split (G3/G4/H0 appendix-only)
- [ ] Final copy-edit for “broadcast ±1 three-factor” terminology consistency

---

## Required main-text disclosures (do not drop)

1. Lead FAIL = **broadcast ±1 three-factor**, not “any broadcast” (disclose DFA-schedule graded **0.9863**).
2. Live transfer = **v13–v24 FAIL**; v131 is matched-only.
3. Discrete EventProp H2H **FAIL** `c1-eventprop-5bb083d5e88d0ad2` ≠ continuous Wunderlich–Pehle.
4. Integrity appendix for canonical C1 (H1/H2/θ=∞/`project`).
5. F1 / F2 / F5 efficiency honesty.
6. Non-claims: biology, AC PASS, impossibility, neuromorphic HW.

---

## Figure inventory vs camp artwork

| Spec ID | Camp path | Status |
|---|---|---|
| fig0 | `runs/2026-07-23-paper-hard-both/figures/fig0_*` | present |
| fig1 | `…/fig1_matched_rule_swap.*` | present |
| fig2 | `…/fig2_matched_means.*` | present |
| fig3 | `…/fig3_engine_c1_means.*` | present |
| fig4 | `…/fig4_transfer_ladder.*` | present |
| fig5 | `…/fig5_xor_locality.*` | present |
| figD | `…/figD_diff_closure.*` | present |
| **figM** | `…/figM_mechanism_*` | present |
| **graphical abstract** | `…/graphical_abstract.*` | present |

Orphans under `results/fig1_ladder.png` etc. are **not** camp MUST artwork.

---

## Bibliography ownership

Primary stubs live in [`references.bib`](references.bib). Expand DOIs / venue pages when locking the template. Prefer citing on-disk hashes in Methods over inventing external “BINN” papers.
