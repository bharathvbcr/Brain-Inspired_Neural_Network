# Audit 2026-09-29 — the manuscript read as a reviewer would

**Scope.** `PAPER_DRAFT.md` as built for TMLR, read the way a hostile referee
reads it. Two passes ran independently: a referee-style audit of the claims
against the cells and the code, and a literature search for prior work that
pre-empts, overlaps or contradicts them. Every finding below was re-checked
before being recorded. **Verified** means recomputed from the cells, read in
the code, or read in the primary source. **Inferred** means reasoned from
verified facts. **Unverified** means neither.

**Verdict.** The anchor result is correct and not pre-empted. The lead claim
is stated more broadly than the cells support, the comparator is weaker than
the prose implies, and the document reads as a lab record rather than a paper.
Predicted outcome at TMLR: **major revision**. At a Neural Networks-class
journal, as written: **reject**. Worth submitting to TMLR once the writing
items in §3 are done, with the experiment in §4 run either before submission or
for the revision.

---

## 1. Corrected in the draft on 2026-09-29 (verified factual errors)

| Where | Was | Now | Evidence |
|---|---|---|---|
| Abstract (and `PAPER_SKELETON.md`) | "the read-out's advantage … collapses by **+0.1347**" | the attention arm loses +0.1347, the rate arm +0.0142, so the advantage collapses by **+0.1205** | +0.1347 is the attention arm's own shuffle cost (`PAPER_RESULTS_TABLE.md:149`); the DiD is +0.1205 (`RESULT_2026-08-29_W21_…:36`), recomputed by `verify_published_numbers.py` `w21_point` |
| §0 | attention "replacing the field's default unweighted Σₜ softmax(u[t])" | added *alongside a mean spike-rate read-out*, which is permutation-invariant by construction and is not that default | `binn-learn/src/shd_matched_arms.rs:816-850`: logits = rate term + attention term |
| §1 | "Section 4.1 states the four scope limits" | §4.6 | the limits are in §4.6 |
| §2.3 | depth $L \in \{1,2,4\}$ | $L \in \{1,2,3,4\}$ at $d \in \{32,64\}$ | wave 22 ran `d32l3` and `d64l4` |
| §4.8 | kernel attribution "by elimination, not by an ablation" | the 0.087 is a net of offsetting terms: K = +0.3111, M = +0.1666, 0.0378 unidentified | §3.8 reports that ablation |
| Appendix J | DFA 0.9387, RL 0.9200 | 0.9925, 0.9950 | those were the *archived* values from the zero-spike forward pass (`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md` §1) |
| PDF | headings printed "0.2 1. Introduction" | numbered once, by the manuscript | `build_paper.py` emitted `##` as a LaTeX-numbered `\subsection` |

`check_every_number.py` now declares 0.1205 in `KNOWN_COINCIDENCE`, in the
same class as W22's 0.0893 and W21's 0.0959: a four-arm DiD that the `paired`
generator reaches only by arithmetic accident.

## 2. The lead claim is broader than the cells (verified; the author's decision)

"The control clears its +0.03 bar at all twenty-one operating points" is true
of the DiD. It does not mean the read-out *buys* something order-dependent at
all twenty-one. DiD = gain(intact) − gain(shuffled). Where the intact gain is
nil, a positive DiD comes entirely from the attention arm training *worse than
the rate arm* once order is destroyed. Recomputed from the wave-21/22 cells,
seed-paired:

| operating point | gain, intact | gain, shuffled | DiD | min(gain, DiD) |
|---|---:|---:|---:|---:|
| h1024 / L1 (w22) | −0.0159 | −0.0834 | +0.0675 | **−0.0159** |
| h512 / L1 (w22) | +0.0043 | −0.0850 | +0.0893 | **+0.0043** |
| h256 / L1 (w22) | +0.0150 | −0.0608 | +0.0758 | **+0.0150** |
| h128 / `channels-700` / L1 (w22) | +0.0243 | −0.1127 | +0.1369 | **+0.0243** |
| h1024 / L4 (w21) | −0.1618 | −0.2740 | +0.1122 | **−0.1618** |
| h128 anchor / L4, n = 32 | +0.1275 | +0.0070 | +0.1205 | +0.1205 |

- **At five of 21 points the read-out has no gain to lose, measured against the
  +0.03 bar.** Only h1024/L4 is already disclosed, as a scope limit. **(Verified.)**
- §3.5 cites h512/L1 and h1024/L1 as the mechanism recurring "where nothing
  collapses". Their intact gains are +0.0043 and −0.0159. **(Verified.)**
- The "leading open problem" (§4.8, the gain/DiD dissociation and the failed
  Spearman test) follows largely from the shuffled gain varying widely across
  points. **(Inferred.)**
- One candidate cause: a fixed per-sample permutation lets the positional code
  memorise noise. The cells' loss trajectories could test this. **(Unverified.)**

**Recommended restatement:** a shuffled-gain column beside every DiD, and the
claim as "order-dependent wherever the read-out has a gain to lose: 16 of 21
points". This changes what the paper asserts, so it needs an amendment the
author owns. It is not edited here.

**Adopted the same day** with the author's approval, sharpened by the
intervals (all 21 points at `e100`, 16 of 21 at `e400`):
[`AMENDMENT_2026-09-29_THE_CLAIM_IS_WHERE_THERE_IS_A_GAIN_TO_LOSE.md`](AMENDMENT_2026-09-29_THE_CLAIM_IS_WHERE_THERE_IS_A_GAIN_TO_LOSE.md).

## 3. Writing-only items before submission (the author's decisions)

1. **Synchrony increment.** The abstract quotes +0.1308 / +0.1038 / +0.0575
   without qualification. At `fixed-t100/t250`, `channel-shuffled` raises the
   rate arm's firing from about 0.18 to 0.42–0.45, with saturation 0.016–0.035.
   `TODO_2026-08-07_OPEN_WORK.md:218` already calls the increment an upper
   bound. **(Verified that the TODO says so; firing rates as reported by the
   referee pass.)**
2. **Promote `hidden-shuffled`.** It leaves the substrate bit-identical and
   permutes only the read-out's view of time. DiD **0.1159**, positive 12/12;
   the rate arm pays exactly 0.0000. It is the cleanest statement of what the
   read-out consumes, and the main text omits it
   (`RESULT_2026-09-05_W27_THE_TIMESCALE_IS_261_MS.md` §1). **(Verified.)**
3. **Test-set selection.** Cell accuracy and the 0.80 gate are test-set
   accuracy (`binn-lab/experiments/shd_instrument.rs:1108`). The anchor was
   carried forward on clearing that gate. §0 applies SHD's
   missing-validation-split caveat to others' numbers; it should apply it to
   the 0.8332 headline too. The paired DiD is largely unaffected. **(Verified.)**
4. **§0 literature framing.**
   - Yu et al. (2025) reverse time at test only. This paper reverses train and
     test, so theirs is not "an independent prediction" of the flat reversal
     DiD. Yu also ran a with/without-delay comparison and per-spike vs
     per-neuron jitter, which partly anticipates the synchrony term.
   - Nowotny, Turner & Knight (arXiv:2212.01232) already replace Σₜ with a
     time-weighted sum on SHD.
   - Cramer et al. put counts-only classifiers at ≤60%, yet the shuffled arms
     reach 0.69; the shuffled task is "bag-of-frames-solvable", not
     "rate-solvable".
   - Name the DiD as the read-out × shuffle interaction of a 2×2 factorial.

   **(Verified by reading the primary sources.)**
5. **Missing citations.** Nowotny et al. 2022; Perez-Nieves et al. 2021;
   Zenke & Vogels 2021; Iyer et al. 2021; Hindman & Cantley 2026 (TEMPO, same
   operator family); Zeng et al. 2023 (attention with positional codes ignoring
   shuffles — engage with it); Sinha et al. 2021; Hewitt & Liang 2019;
   Bertinetto et al. 2021 (preregistration in ML). Update Yu 2025 to its
   published DOI, 10.1088/2634-4386/ae46d5.
6. **Structure.** The matched-architecture programme is about half the
   document. §4.1 is titled "Lead claim" but calls the matched negative "the
   cleanest publishable claim". Split that programme into a separate note.
7. **Readability.** Internal vocabulary dominates: waves, contracts, operating
   points, kill-gates, twins, cells, the instrument. Paragraphs narrate their
   own revision history. Record filenames and script paths appear in the body.
   Hyperparameters (optimiser, learning rate, batch, surrogate,
   initialisation, attention dimensions) are only in the repository. A TMLR
   reviewer must be able to reproduce the method from the paper.
8. **Statistics.** No confidence intervals on any DiD, and no treatment of
   multiplicity across 21 points × 2 budgets. Add paired bootstrap CIs; the
   anchor survives any correction (32/32).

## 4. The experiment that answers the central critique

**The critique.** The control is order-blind by construction and the substrate
cannot hold order beyond a few bins, so "the only component that can read
order is the one that depends on it" is close to design. **(Inferred; a
referee will say it.)**

**The answer.** The same DiD against order-*sensitive* read-outs: Σₜ
softmax(u[t]), max-over-time, and last step. Bittar & Garner (2022) report all
of them on SHD. Cost at the anchor with 12 seeds: roughly 300–400 cell-hours.
That is about two weeks of wall time on the local Mac at 8–11 h per attention
cell, and the Mac is shared with wave 29's platform. **(Cost inferred from
`TODO_2026-08-07_OPEN_WORK.md:189-190`.)**

**Later:**
- SSC as a second dataset (needs the Python mirror, ~100 h per attention cell).
- A competitive substrate (DCLS delays or adLIF).
- Synchrony at matched activity (~800 cell-hours).

## 5. What is new, after the literature pass

**Not pre-empted (inferred — no one can prove a negative):**
- A seed-paired, component-level measure of how much of one read-out's
  marginal gain on SHD survives destroying temporal order.
- The bin-shuffle vs channel-shuffle split measuring the synchrony term.
- The reversal control.
- The 261 ms timescale.
- The `hidden-shuffled` operator.

**Already known, and conceded by §0:**
- Temporal attention in SNNs on SHD.
- That SHD needs temporal information.
- That 0.83 is far from the ~0.95–0.96 frontier.

**Search tooling.** ScholarLM ran degraded on every call (2–4 of 7 providers
erroring, arXiv timeouts, HTTP 429s). The pass fell back to web search and read
primary text where it could.
