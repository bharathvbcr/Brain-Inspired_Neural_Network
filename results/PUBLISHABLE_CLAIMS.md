# BINN — publishable claim freeze

> ### SUPERSEDED IN PART — 2026-08-25 matched-architecture re-run
>
> **Amended 2026-08-27.** This banner used to say that every
> matched-architecture number here predated the 2026-08-22 silent-initialisation
> repair and that "none of them has been regenerated here". That was true when
> written and is now false: the 2026-08-27 restructure rebuilt §2 and §2b on the
> re-run's own figures (DFA 0.9925 / 0.9875, RL 0.9950 / 0.9812, EventProp
> 0.9450 / 0.8900 PASS). The banner is amended rather than deleted because a
> reader who saw the old one needs to know it moved.
>
> Matched-architecture numbers in this document are now the **post-repair**
> figures. The pre-repair values were produced on a forward pass that emitted
> **zero spikes at any seed**. The re-run is
> [`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md)
> and [`PAPER_DRAFT.md`](PAPER_DRAFT.md) §3.1 carries the same figures.
>
> **The lead negative survives** — broadcast ±1 three-factor is at chance on both
> forward graphs. **Three contrasts do not**: the discrete EventProp-style
> spike-adjoint goes 0.5000 FAIL to 0.9450 / 0.8900 PASS, and the two RL
> broadcast contrasts go 0.5250 to 0.9100 and 0.5113 to 0.7962. The gradient
> ceiling goes 0.8887-0.9150 to **1.0000**, so every `gap_closed` here divides by
> a different denominator than the instrument now produces.
>
> The `c1-*` config hashes cited below are **retired**: `MATCHED_INPUT_SCALE` was
> not part of them, so each named two experiments. They no longer resolve, by
> design. The retirement table is in the re-run document section 8.
>
> Rows concerning the SHD attention campaign, the live-transfer package and the
> XOR task are unaffected: none of them runs on the matched dense-LIF forward.

> **CITATION WARNING (added 2026-08-07).** This document cites the
> `track-b-rescue` **v130** row (`1.0000`, gap LCB `0.9988`, PASS matched) as a
> matched-substrate result. That report is stale: the source is **v131**, and the
> 130→131 bump is precisely the clamp-and-separation-gate fix for the defect the
> row exhibits. Under current code the arm **cannot be reported as PASS**. Do not
> cite it until `track-b-rescue` has been re-run. The DFA
> (`c1-dfa-c8c4fe0899908b84`) and RL (`c1-rl-42eddc9c801308e9`) matched PASSes are
> **not** affected by this defect; they ran through the clamped `runner.rs` path.
> See `AUDIT_2026-08-07_JULY_CAMPAIGN_SCORING_PATH.md` and
> `TODO_2026-08-07_OPEN_WORK.md` §1.
> **RESOLVED 2026-08-19.** The re-run landed. At v131 the arm reports
> **INVALID_HARNESS**, not PASS: the ceiling-inverted warning fires on 3 of 20
> learned-FB seeds and the code refuses to emit a PASS while it is present.
> The v130 PASS is **withdrawn**. See
> `RESULT_2026-08-19_TRACK_B_V130_PASS_WITHDRAWN.md` and
> `track_b_results_v131.md`.



Authority: Rust sources + on-disk result notes. Do not widen beyond this sheet for research packaging.

**Thesis (locked):**

> **Primary — SHD time-axis read-out (the read-out program).** What a time-axis attention
> read-out buys on SHD is **temporal order**, and the claim is a **conditional**:
> a difference-in-differences on the read-out's *gain*, not on accuracy.
> Destroying temporal order by permuting time bins — independently per sample,
> in **both the training and the test split**, so the task itself becomes
> rate-solvable — costs the attention read-out **+0.1347** of accuracy against
> the rate read-out's own **+0.0142**, on the same seeds and the same splits: a
> **9.5× ratio, positive in 32/32 seeds**. Neither "attention helps on SHD" nor
> "SHD depends on temporal order" is new, and neither may be claimed: both are
> prior art (Cramer et al., IEEE TNNLS 33(7) 2022, could not exceed **60%** on
> spike-count-only SHD; TA-SNN, ICCV 2021, **91.08%**; STSC-SNN, 2022,
> **92.36%**). What has not been measured is **which component's marginal
> contribution is the order-dependent one** — that, and not the accuracy, is the
> lead. Three scope limits travel with it and are not footnotes: the gain
> **inverts at h1024** and that inversion is a **threshold, located but
> unexplained**; **0.8332 is not competitive** against a 95–96.4% SHD frontier;
> and the instrument is **uncalibrated** (criterion 5 unmet).
>
> **Secondary — matched-architecture kill gate (the matched-gate program).** **Broadcast ±1 three-factor** credit (surrogate eligibility × ±1 reward) fails a preregistered accuracy/gap bar on a matched dense-LIF coincidence forward, **on both forward graphs, at n=20** — and it is the **only** rule tested that does. Every other rule now clears the gate against a reference at 1.0000, so the task isolates one failure from a field it can no longer rank; disclose that, and do not read the passes as an ordering. Lead FAIL is ±1 three-factor, not “any broadcast.” Locality evidence is XOR. The event-driven C1 / live k-WTA path fails its operationalized gate under broadcast and under live REINFORCE transfer (including structured-`B` / capacity / eligibility gap-close arms)—report as a substrate/pipeline transfer negative with integrity disclosed, not as a biological or Assembly-Calculus result.

---

## Claim strength ladder

| Rank | Claim | Strength | Object under test |
|---:|---|---|---|
| **1 (lead)** | On SHD, the **attention read-out's marginal contribution is the order-dependent one**: its bin-shuffle cost is **+0.1347** against the rate read-out's own **+0.0142** on the same seeds and splits — **9.5×**, **32/32** positive. A difference-in-differences on the **gain**, not on accuracy. **Measured at 9 of 21 operating points**, widths 128–1024 and both contracts and geometries, clearing its +0.03 bar at every one | **Strongest / measured (registered n=12, confirmed n=32, generalised at n=12 per point)** | Which component's contribution depends on temporal order |
| **1a (limit on the lead)** | The effect's **size is not the gain**: Spearman ρ over the six per-width rungs is **−0.1430** against a registered bar of **+0.829**. h768 carries the smallest gain (+0.0560) and the largest DiD (+0.1881) | **Measured — a registered NOT MET, and it bounds the lead** | Whether the gain *decomposes* into an order-dependent share |
| **1b (supporting)** | Same instrument, headline accuracy: `ff+fixed+attn` at `d32/L4` reaches **0.8332** against `ff+fixed` **0.7057** (gain **+0.1275**, **32/32** positive, **32/32** at or above 0.80) | **Measured — and explicitly *not* competitive** (frontier 95–96.4%) | Attention read-out on a LIF substrate |
| **2 (secondary)** | On an identical dense-LIF forward, **broadcast ±1 three-factor** (surrogate eligibility × ±1) does not clear the matched-arch gate vs SuperSpike BPTT, on **both** forward graphs at n=20 | **Strong / clean negative** | Rule topology only (±1 broadcast) |
| **2b (contrast)** | On the matched forward, graded DFA, REINFORCE×frozen-`B_i`, broadcast-graded **and a discrete spike-adjoint** all clear the gate — against a reference at 1.0000, so each pass reduces to "above 0.75" | **Weak — the task saturates; no ordering may be claimed** | Credit richness / locality |
| **3** | Under the coded C1 engine loop, local/dense three-factor miss `g2_min_*` while multi-epoch SurrogateLifReference on the same frozen splits succeeds | **Moderate — caveated** | Operationalized pipeline |
| **3b (transfer)** | Live C1 opt-in `ReinforceFeedback` (v13) and gap-close suite (v14–v19) fail G2; structured `B` / capacity clear accuracy floor but not gap LCB > 0.5 | **Moderate — new hashes** | Live k-WTA transfer |
| **4** | Exact-forward credit arms (`c1x-*`) fail G2-style bars on their separate hashes | Supporting methods notes | Exact-forward / hybrid credit |
| **5** | Checklist-closure protocols (`c1-spike-*`, `c1-spike-s-*`, `c1-project-*`, `c1x-eprop-true-*`, `c3-bptt-*`) | Supporting integrity / methods | New hash families only |
| **6** | 1-layer XOR / `xor_thresh`: broadcast ~chance, DFA solves (locality flip); mid-init depth does **not** show the same flip | Supporting task evidence | NumPy deep suite |
| — | **"Attention on SHD is new," "temporal attention on SHD is new," "96% of *accuracy* depends on temporal order," competitive SHD accuracy, any mechanism for the h1024 collapse, instrument calibration**, biology, neuromorphic HW, impossibility, brain model, live transfer from matched PASS, "any broadcast fails," continuous textbook EventProp equivalence, depth collapse (v134 withdrawn), online learned FB v130 (v131 withdrawn), **the discrete spike-adjoint FAIL (withdrawn 2026-08-25: 0.5000 → 0.9450/0.8900 PASS)**, **the two RL broadcast contrasts as chance results (0.5250 → 0.9100, 0.5113 → 0.7962)**, **any ranking among the passing matched arms** | **Do not claim** | — |

---

## 1. Primary claim — SHD time-axis read-out: whose contribution is order-dependent (Waves 1–17)

### May claim

1. **The conditional — this is the lead.** On SHD, bin-shuffling costs the
   attention read-out (`ff+fixed+attn`, `d32/L4`, `e400`) **+0.1347** of
   accuracy, positive in **32/32 seeds**, against the rate read-out
   (`ff+fixed`) losing **+0.0142** of its own on the same seeds, the same
   splits and the same destruction operator — a **9.5× factor**. Equivalently,
   the read-out's *advantage* falls from **+0.1275** to **+0.0070**, so
   **94.5% of the read-out's marginal contribution is contingent on temporal
   order**. The claimed object is the **gain**, not the accuracy.
   ([`RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md`](RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md);
   [`PAPER_DRAFT.md`](PAPER_DRAFT.md) §3.5)
2. **As registered, at n=12.** The same contrast was preregistered and measured
   at twelve seeds: attention shuffle cost **+0.1337** (0.8320 → 0.6983,
   **12/12**, per-seed range +0.0967 to +0.1568) against the plain arm's
   **+0.0128** (0.7062 → 0.6934) — a **10× factor**; advantage +0.1258 →
   +0.0050, **96%** order-contingent. **n=12 is the registered measurement;
   n=32 is the confirmation, and the two must be reported as such.** Twenty
   seeds beyond the registered twelve move the shuffle cost by **+0.0010**.
   ([`RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md`](RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md))
3. **The mechanism is not unique to the anchor.** The same operator, seeds and
   pinned binary at seven further operating points — 168 cells, **zero
   divergences, zero voided**, every point above the registered floor of nine
   seed-paired quadruples. DiD clears **+0.03** at **12/12** seeds at h256
   (**+0.0862**), h384 (**+0.0767**) and h512 (**+0.0968**), and at both
   alternative binnings — `channels-700` (**+0.1122**) and `published-10ms`
   (**+0.0959**). Coverage **2 → 9 of 21** operating points.
   **What may NOT be claimed from it:** that the DiD tracks the gain (ρ =
   **−0.1430** against **+0.829**, NOT MET), and anything about h1024 other than
   that the order-dependence persists there while the gain is negative.
   ([`RESULT_2026-08-29_W21_THE_MECHANISM_TRAVELS_BUT_ITS_SIZE_DOES_NOT.md`](RESULT_2026-08-29_W21_THE_MECHANISM_TRAVELS_BUT_ITS_SIZE_DOES_NOT.md))
4. **Headline accuracy — supporting, not the claim.** `ff+fixed+attn` at
   `d32/L4` / `e400` reaches **0.8332** against `ff+fixed`'s **0.7057**, gain
   **+0.1275**, positive in **32/32** seeds, **32/32 at or above 0.80**.
   Registered at n=12 as **0.8320** against **0.7062**, gain **+0.1258**,
   **12/12 ≥ 0.80**, budget-stable (|e400−e200| = 0.0002); twenty further seeds
   move the gain by **+0.0017**.
   ([`RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md`](RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md);
   [`RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md`](RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md);
   [`RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md`](RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md))
5. **Sample efficiency.** The attention read-out reaches **98.1%** of its e400
   accuracy by 10 epochs (0.7337), bracketing convergence at `(5, 10]`.
   **The denominator is the `d32/L1` arm at convergence (0.7483), not the
   `d32/L4` headline** — against the headline's 0.8320 the same cell is
   **88.2%**, and the L1 ladder's e400 gain is **+0.0421**, not +0.1258.
   ([`RESULT_2026-08-20_W7_CONVERGENCE_IS_BRACKETED.md`](RESULT_2026-08-20_W7_CONVERGENCE_IS_BRACKETED.md);
   [`SUMMARY_2026-08-20_ATTENTION_CAMPAIGN.md`](SUMMARY_2026-08-20_ATTENTION_CAMPAIGN.md))
6. **The read-out does not substitute for temporal state in the substrate.**
   Adaptation is inert at the anchor: the gain is **+0.1258** on `ff+fixed` and
   **+0.1285** on `ff+alif`, a difference of **+0.0027** against a two-sided bar
   of 0.03 and positive in **6 of 12** seeds. On the recurrent substrate the
   gain roughly doubles — **+0.2612** on `rec+alif` (10 pairs) against
   **+0.1201** on `ff+fixed` (12 pairs), a difference of **+0.1411** positive in
   **10 of 10**. Substitution is refuted on both axes.
   ([`RESULT_2026-08-23_W12_ATTENTION_DOES_NOT_SUBSTITUTE_FOR_ADAPTATION.md`](RESULT_2026-08-23_W12_ATTENTION_DOES_NOT_SUBSTITUTE_FOR_ADAPTATION.md);
   [`RESULT_2026-08-23_W14_ATTENTION_AND_RECURRENCE_ARE_COMPLEMENTARY.md`](RESULT_2026-08-23_W14_ATTENTION_AND_RECURRENCE_ARE_COMPLEMENTARY.md))

### Must disclose alongside it

- **The prior art, in the same breath as the claim.** A time-axis attention
  mechanism in a spiking network on SHD is **not new** (TA-SNN, ICCV 2021,
  91.08%; STSC-SNN, 2022, 92.36%), and that SHD depends on temporal order is
  **not new** (Cramer et al., IEEE TNNLS 33(7) 2022: spike-count-only variants
  could not exceed **60%**; Neuromorphic Sequential Arena, IJCAI 2025: SHD
  falls **86.48 → 68.51** with temporal processing removed model-side; Yu et
  al., arXiv:2507.16043, 2025). The paper's contribution is the
  **difference-in-differences on the gain**, and any phrasing that reads as
  "we show attention helps" or "we show SHD needs order" is a claim this
  package does not own. ([`PAPER_DRAFT.md`](PAPER_DRAFT.md) §0)
- **The 94.5% / 96% figure is about the *gain*, never about accuracy.** Under
  shuffling the attention arm still scores 0.6983 against the plain arm's
  0.6934 at n=12 — accuracy largely survives. "96% of *accuracy* depends on
  temporal order" is false and is an explicit non-claim.
- **The destruction operator.** Bins are permuted **per sample** and the
  manipulation is applied to **both the train and the test split**, with
  separate seed lineages — which is what removes the distribution-shift
  confound of a test-time-only probe, and what makes the shuffled task
  genuinely rate-solvable rather than merely corrupted.
  ([`PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION.md`](PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION.md) §5;
  [`PAPER_DRAFT.md`](PAPER_DRAFT.md) Abstract). Every `w9shf` cell passes the
  temporal audit (counts preserved, relocated fraction ≥ 0.5), so a "shuffle"
  that failed to shuffle would have been caught rather than scored.
- **Sample size discipline.** n=12 is the **registered** measurement against
  preregistered bars; n=32 is a **confirmation** run under its own
  registration. Both cleared their bars, and the extension was not run to
  rescue anything — report the pair, not the larger number alone.
- **Anchor.** Everything above is measured at **h128 / `published-2ms` /
  `adjacent-sum-5` / e400 / `d32/L4`** unless stated.

### Scope limitations (must disclose)

- **The width collapse is a THRESHOLD, and it is located but unexplained.** On
  a six-rung ladder the gain runs **+0.1258** (h128), **+0.0966** (h256),
  **+0.0760** (h384), **+0.0876** (h512), **+0.0560** (h768), **−0.1618**
  (h1024). The drop into h1024 is **0.2178**, **6.9×** the largest gap below it
  (0.0316), so the collapse is a threshold between **h768 and h1024** and not
  the slope continuing. Three preregistered rescue levers at h1024/d32/L4 are
  **all negative and all worse than the arm they were meant to rescue**:
  surrogate scale 0.5 → **−0.2106**, surrogate scale 0.25 → **−0.2565**,
  gradient clipping at 1000.0 → **−0.0904**. Clipping moved the median
  epoch-mean gradient norm from **55.494** to **11.660** — a real effect in the
  intended direction — and accuracy did not follow. **Nothing in this package
  offers a mechanism for the collapse**, and the parsimonious alternative
  (overfitting on 8,156 training samples) is not excluded — **and is not
  supported either**. A registered prediction said that if the collapse and the
  temporal-order account were the same phenomenon, the shuffle cost should
  vanish where the gain does. It **did not**: DiD(h1024) = **+0.1122** in 10 of
  12 seeds against a ceiling of **+0.02**, with the gain over those same twelve
  seeds at **−0.1618**. The read-out consumes temporal order while performing
  worse than no read-out at all. Per the preregistration this is the package's
  **leading open problem**, and the overfitting argument is left exactly where
  it was, because it was conditional on a collapse that did not occur.
  ([`RESULT_2026-08-29_W21_THE_MECHANISM_TRAVELS_BUT_ITS_SIZE_DOES_NOT.md`](RESULT_2026-08-29_W21_THE_MECHANISM_TRAVELS_BUT_ITS_SIZE_DOES_NOT.md))
  ([`RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md`](RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md);
  [`PREREG_2026-08-25_THE_H1024_COLLAPSE.md`](PREREG_2026-08-25_THE_H1024_COLLAPSE.md))
- **No monotonicity in width.** h384 and h512 are not distinguishable at twelve
  seeds (paired difference **−0.0116**, sd **0.0253**, negative in 7 of 12), so
  the decay above the collapse is **not** strictly ordered and no dip at h384 is
  claimed.
- **0.8332 is NOT competitive, and must be conceded in the text.** The SHD
  frontier is **95–96.4%** (learned delays, adaptation, spiking transformers).
  This instrument carries **no temporal kernel of any kind** and lands close to
  where the dataset's own authors put a no-delay recurrent baseline
  (**83.2 ± 1.3%** at 1024 neurons with augmentation). The pinned third-party
  calibration reference reaches **0.9390 / 0.9368 / 0.9371**; the **0.087**
  residual is attributed by elimination and code-reading — not by an ablation
  that added the kernel — to a 25-tap-per-synapse learned temporal kernel, and
  that is the package's weakest load-bearing inference.
  ([`PAPER_DRAFT.md`](PAPER_DRAFT.md) §0, §3.8;
  [`FINDING_2026-08-24_THE_FORWARD_PASSES_DIFFER_IN_KIND.md`](FINDING_2026-08-24_THE_FORWARD_PASSES_DIFFER_IN_KIND.md))
- **Benchmark caveat.** SHD ships no validation set; the same model validated on
  test reads **95.81 ± 0.56** against **93.79 ± 0.76** with a proper held-out
  split. Differences below ~1.5 points between published SHD numbers are not
  reliably meaningful, here included.
- **Geometry.** The gain is positive across geometries (**+0.1090** on
  `channels-700`, **+0.1491** on `published-10ms`), but the 0.80 clearance is
  geometry-specific (**0.7864**, 6/12, on `channels-700`).
- **Temporal resolution is a separate axis and points the other way.** The
  `published-Nms` test (S-5) is **refuted and withdrawn** — it moved bin width
  and sequence length together. Re-asked on `fixed-tN`, which holds a 1400 ms
  window fixed, the gain **shrinks** as resolution gets finer: **+0.1927**
  (14.0 ms) → **+0.1751** (5.6 ms) → **+0.1474** (2.8 ms), a difference of
  **−0.0453** against a two-sided bar of 0.03.
  ([`RESULT_2026-08-22_W10_RESOLUTION_LADDER.md`](RESULT_2026-08-22_W10_RESOLUTION_LADDER.md))
- **Recurrent-substrate limits.** `rec+alif` is measured from a lower base
  (headroom 0.4738 against 0.2912; headroom-normalised the ratio falls from
  2.2× to **1.34×**), does **not** win in absolute terms (0.7874 against
  `ff+fixed+attn`'s 0.8289), rests on **ten pairs — the registered minimum** —
  with peak gradient norms to 4.9e32, and carries residual survivorship
  exposure. Ten `rec+fixed` cells were voided by saturation.
- **NOT calibration.** Criterion 5 (Python mirror) remains **unmet**; the SHD
  instrument is uncalibrated. Every number above is a within-instrument,
  same-machine comparison — cross-machine Gate F FAILs macOS-vs-Linux by
  design, and no number here may be compared to a macOS-recorded one.

### Falsifier

A bin-shuffle contrast run on the same instrument, the same anchor, the same
seed lineage and the same both-splits destruction operator, in which **the
attention read-out's shuffle cost fails to exceed the rate read-out's** — or in
which the attention arm's intact-minus-shuffled falls below the registered
**+0.05** bar, or is positive in fewer than the registered **10 of 12** seeds —
overturns the lead claim. So does a destruction operator that leaves temporal
order intact while destroying something else (rate, synchrony, channel identity)
and reproduces the same collapse in the gain, since that would show the
difference-in-differences is not indexing order. Widening the seed pool,
switching anchor geometry, or substituting a test-time-only shuffle does **not**
count: the first two change the comparison, and the third reintroduces the
distribution-shift confound the both-splits operator exists to remove.
([`RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md`](RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md) §3, M-1/M-2)

### Non-claim

Do **not** claim novelty for temporal attention on SHD, or for the finding that
SHD depends on temporal order — both are prior art and the lead is the
conditional. Do **not** state the mechanism as "96% of accuracy depends on
temporal order"; the 96% / 94.5% is a fraction of the **gain**. Do **not**
present **0.8332** as competitive, or compare it to the 95–96.4% frontier as
though the gap were tuning. Do **not** offer any mechanism for the h1024
collapse, or extend the read-out claim past h768. Do **not** claim the
instrument is calibrated. Do **not** carry the S-5 temporal-*resolution* story:
it is withdrawn, and `fixed-tN` moves the opposite way.

---

## 2. Secondary claim — matched-arch broadcast ±1 three-factor insufficiency

**May claim:** Holding the dense-LIF forward fixed and swapping only the update rule — **broadcast ±1 three-factor** vs SuperSpike BPTT — the local arm fails the preregistered matched gate on **both** the feed-forward and recurrent graphs (0.5000 and 0.5100, n=20, against a reference at 1.0000). Mechanism label: **broadcast ±1 three-factor**, not "spiking failed," and not a bare "broadcast credit topology" ban that would misread the 0.9975 graded contrast.

**Must disclose alongside it:** every other rule tested clears this gate, and the reference sits at 1.0000 at the canonical budget, so `gap_closed` reduces each pass to "the arm scored above 0.75". The claim that survives is *one rule fails a task the rest saturate* — which is narrower and better evidenced than a graded richness ordering, and the ordering **may not** be claimed.

**Caveat (A6 ceiling health), now the binding limitation:** on the archived instrument the canonical 80-epoch schedule undertrained the reference (0.8963 / 0.9013 at e80, climbing to 1.0000 by e640; [`RESULT_2026-08-19_A6_CEILING_HEALTH.md`](RESULT_2026-08-19_A6_CEILING_HEALTH.md)), so the e80 comparison read as *learning speed*. On the repaired instrument the reference reaches **1.0000 at e80 itself**, so there is no budget at which this task separates the arms and no ceiling comparison on it survives. Every matched claim in this sheet is bounded by that.

**Code:**

- `binn-learn/src/matched_local_baseline.rs` — matched local + gradient arms; GC1-exempt baseline
- `binn-lab/src/match_config.rs` — protocol v4, `c1-match-*` hashes; asserts distinct from `c1-118207fbc3eaba53`
- `binn-lab/src/runner_match.rs` — harness; refuses to reopen protocol-v2
- `binn-lab/experiments/c1.rs` — `--matched-arch` entry

**On-disk anchors (do not invent):**

- Current figures: the 2026-08-25 re-run at `MATCHED_INPUT_SCALE = 2.0`, n = 20, both forward graphs — [`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md), reports in [`matched_rerun_2026-08-25/`](matched_rerun_2026-08-25/)
- Verdict **FAIL** on both graphs; matched-local mean **0.5000** (feed-forward) / **0.5100** (recurrent); gap LCB **0.0000** / **−0.0192** against a 0.5 bar; accuracy 0.51 against a 0.65 floor; reference **1.0000** on both
- Archived, pre-repair, **not citable as current**: `c1-match-5dc6822e71229e9e` — [`c1_match.md`](c1_match.md), [`MATCHED_ARCH_CONTROL.md`](MATCHED_ARCH_CONTROL.md); matched-gradient **0.8963**. The `c1-*` config hashes are **retired**: `MATCHED_INPUT_SCALE` was never mixed into them, so each named two experiments.

**Does not reopen** `c1-118207fbc3eaba53`.

---

## 2b. Contrast claims — DFA / REINFORCE / spike-adjoint clear the matched gate

**May claim:** On the same matched dense-LIF coincidence forward, replacing **broadcast ±1 three-factor** with (a) graded DFA, (b) directional REINFORCE × frozen per-neuron `B_i`, or (c) a discrete EventProp-style spike-adjoint clears the preregistered gate at the 80-epoch budget — on **both** forward graphs.

All figures are the 2026-08-25 re-run at `MATCHED_INPUT_SCALE = 2.0`, n = 20. "Archived" is the 2026-07-23/24 record, produced on a forward pass that emitted **zero spikes at every seed**, and is shown only so the movement is visible ([`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md)).

| Arm | archived | recurrent | feed-forward | verdict now |
|---|---:|---:|---:|---|
| **broadcast ±1 three-factor** (lead) | 0.5000 | **0.5100** | **0.5000** | **FAIL**, both |
| Matched graded DFA | 0.9387 | **0.9875** | **0.9925** | **PASS**, both |
| Broadcast-graded (honesty contrast) | 0.9863 | **0.9975** | **0.9975** | — (contrast) |
| Matched RL `rl_reinforce_fb` × frozen `B_i` | 0.9200 | **0.9812** | **0.9950** | **PASS**, both |
| RL graded-reward broadcast | 0.5250 | **0.9100** | **0.8787** | — (contrast; FAIL **withdrawn**) |
| RL ±1 broadcast | 0.5113 | **0.7962** | **0.7775** | — (contrast; FAIL **withdrawn**) |
| Discrete EventProp spike-adjoint (v28) | 0.5000 FAIL | **0.8900** | **0.9450** | **PASS**, both (gap LCB 0.6494 / 0.7911) — the FAIL is **withdrawn** |
| Matched RL Online Learned `B_i` (v130) | 1.0000 | — | — | **WITHDRAWN** — v131 reports `INVALID_HARNESS` ([`RESULT_2026-08-19_TRACK_B_V130_PASS_WITHDRAWN.md`](RESULT_2026-08-19_TRACK_B_V130_PASS_WITHDRAWN.md)) |
| SuperSpike BPTT ceiling | 0.8887–0.9150 | **1.0000** | **1.0000** | reference |
| Matched RL graded primary (v11) | 0.5900 | — | — | archived **FAIL** (gap LCB 0.0182); do not retune |

**The graphs disagree on two arms by more than the registered 0.02 bar** — EventProp by 0.0550 and RL graded-reward by 0.0313 — so these are **two comparisons**, and both must be reported. "On that same forward family" is false as written.

**Honesty (required):** On the DFA schedule, **broadcast-graded** contrast is also high (**0.9975**). Do **not** claim coincidence alone proves credit locality is required. The lead FAIL is specifically **broadcast ±1 three-factor** (±1 × eligibility). Locality-flip evidence is **1-layer XOR** ([`deep_xor_thresh.json`](deep_xor_thresh.json)). Mechanism figure: [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) Figure M, **redrawn** — it plotted richness × addressability as a graded surface and on the re-run it is a cliff with one cell below it, so it must encode pass/fail/at-chance rather than a ramp, and must draw the two low-richness broadcast rules separately.

**Falsifier:** A matched ±1 three-factor arm that clears accuracy floor and gap LCB under the same dense-LIF forward / Gate G2 thresholds overturns the secondary claim.

**Non-claim:** do **not** claim that matched PASS transfers to live k-WTA C1 (see §3b). Do **not** widen the lead FAIL to "any broadcast rule." Do **not** cite v131 / `live-transfer-rescue` as live-engine PASS (matched-only binary). Do **not** rank the passing arms.

---

## 2c. Discrete EventProp H2H (matched) — the FAIL is WITHDRAWN

**Withdrawn 2026-08-25.** This section previously claimed that a **discrete** EventProp-style spike-triggered adjoint on the matched dense-LIF forward **FAIL**s the preregistered gate (mean 0.5000, gap LCB 0.0000) against SuperSpike at 0.9150. On a forward pass that can spike it reaches **0.9450** feed-forward and **0.8900** recurrent and **PASS**es on both, gap LCB **0.7911** / **0.6494** against a 0.5 bar. The archived 0.5000 was a spike-adjoint method with **no spikes to differentiate through**: every other arm could still separate the classes by sub-threshold membrane rate, and the one method whose entire mechanism is the spike could not. ([`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md) §3)

**May claim:** nothing about EventProp from this arm in either direction. The old FAIL is withdrawn, and the new PASS is a saturated-task PASS against a reference at 1.0000 — "the arm scored above 0.75."

**Must disclose:** the previous framing — "discrete hard spike-gate adjoint ≠ continuous Wunderlich–Pehle (2021) hybrid EventProp" — was an explanation offered for a number with a different cause, and is retired with it. [`c1_eventprop.md`](c1_eventprop.md) carries the archived report and is **not** citable as current.

**Does not reopen** `c1-118207fbc3eaba53` / frozen match/dfa/rl hashes.

---

## 3. Engine C1 as operationalized pipeline negative

**May claim (with integrity appendix):** Under protocol-v2 hash `c1-118207fbc3eaba53`, Gate G2 FAIL — local/dense three-factor miss gap LCB and accuracy floor while gradient/eligibility references succeed on the same splits. Package as **this coded pipeline** fails its operationalized gate — not "local learning impossible."

**Anchors:** local **0.4912** / gap LCB **−0.0048** — [`c1_g2.md`](c1_g2.md), [`U-NEG_protocol_v2.md`](U-NEG_protocol_v2.md).

### Integrity caveats (must disclose)

| ID | Finding | Status after hardening |
|---|---|---|
| **H1** | `ThreeFactor.last_spike` never cleared across trials (v2) | **Fixed under `c1-iso` / v5**; **still true on canonical v2** |
| **H2** | Incomplete membrane reset vs C3 v2 (v2) | **Fixed under `c1-iso` / v5**; **still true on canonical v2** |
| **θ=∞ mute** | Hidden thresholds set to `f32::INFINITY` during integrate | **Removed under `c1-spike-*` / v6 and `c1-spike-s-*` / v9**; **still true on canonical v2 / iso** |
| **`project` unused** | Assembly Calculus `project` not on C1 crux | **Wired under `c1-project-*` / v7** (scientific **FAIL**); **still unused on canonical v2** |
| **Hybrid e-prop naming** | Exact-forward "e-prop/DFA" ≠ textbook e-prop | **True σ′ e-prop under `c1x-eprop-true-*`**; frozen hybrid remains hybrid |

Isolation / capacity / temporal-PC scientific: all **FAIL** G2 under new hashes ([`CAMPAIGN_2026-07-23_CLAIM_FREEZE.md`](CAMPAIGN_2026-07-23_CLAIM_FREEZE.md)).

---

## 3b. Live REINFORCE transfer + gap-close (v13–v19)

**May claim:** The neuromodulator family that PASSes matched dense-LIF (`rl_reinforce_fb`) **FAIL**s Gate G2 on live muted-θ / k-WTA C1 under an honest opt-in mapping. Separable gap-close hypotheses (epochs, structured `B`, capacity, eligibility timing, restored target teach) were run under **new hashes**; none clear gap LCB > 0.5.

| Protocol | Hash | Verdict | Local | Gap LCB |
|---|---|---|---:|---:|
| v13 live RFB | `c1-660401d74db3c88d` | **FAIL** | 0.4900 | 0.0737 |
| v14 epoch | `c1-714c115e14a3eeed` | **FAIL** | 0.4838 | −0.0100 |
| v15 structured B | `c1-493ddd56f8714fb6` | **FAIL** | **0.7262** | 0.2567 |
| v16 structured×epoch | `c1-677df7f7cbe4f8ec` | **FAIL** | 0.5200 | 0.0844 |
| v17 structured×capacity | `c1-983ee5303c00b147` | **FAIL** | **0.6825** | **0.3127** |
| v18 elig×REINFORCE | `c1-c7d2c86a2b1927f6` | **FAIL** | **0.7125** | 0.2351 |
| v19 structured×teach | `c1-dfab4a7ec19f17c2` | **FAIL** | **0.6700** | 0.2238 |

**Reading to cite:** structured `B` is the accuracy lever (floor cleared); capacity×structured is the best gap LCB still short of G2; epochs under structured B regress; eligibility co-design and restored target teach do not beat v15.

**Sources:** [`MATCHED_ARCH_LIVE_REINFORCE.md`](MATCHED_ARCH_LIVE_REINFORCE.md), [`GAP_CLOSE_RFB_TRANSFER.md`](GAP_CLOSE_RFB_TRANSFER.md).

**Stop:** no remassage of v13–v19 knobs without a new hypothesis + hash.

---

## 3c. Break-it wave (v20–v24) — no deferrals

```
claim_axis: see row
object_under_test: live / matched differentials D4–D8
may_claim: Named protocol FAIL under fixed G2 bars (dual gap reported on live)
must_not_claim: Transfer PASS; remassage v13–v19; biology
```

| Protocol | Hash | Axis | Verdict | Local | Gate LCB | Chance LCB |
|---|---|---|---|---:|---:|---:|
| v20 live DFA | `c1-4db53e645405fae0` | Novel-CS | **FAIL** | 0.7325 | 0.2601 | 0.3321 |
| v21 soft-WTA×SFB | `c1-f975db8fb3e5d569` | Novel-CS | **FAIL** | 0.5025 | 0.0406 | 0.0122 |
| v22 match 4×ep | `c1-match-b46b23549b37d90a` | Integrity | **FAIL** | 0.5000 | 0.0000 | — |
| v23 finite-θ SFB | `c1-4bbaf4b24c2d1da2` | Integrity | **FAIL** | 0.6638 | 0.2370 | 0.2370 |
| v24 continuous B | `c1-840f820b7c07b512` | Novel-CS | **FAIL** | 0.6437 | 0.1380 | 0.1163 |

**Reading:** Matched DFA does not transfer (v20 clears floor, misses gap). Soft winners under SFB regress. Undertraining does not rescue matched three-factor. Mute-off under SFB still fails gap. Continuous B does not beat v15. Closure: [`DIFF_CLOSURE.md`](DIFF_CLOSURE.md). Camp: [`runs/2026-07-23-paper-hard-both/`](runs/2026-07-23-paper-hard-both/).

---

## 4. Supporting notes (optional paper material)

- Exact-forward credit suite: [`credit_assignment.md`](credit_assignment.md) — separate `c1x-*` hashes; all listed arms FAIL.
- Spiking-path true DFA (P4): `c1x-dfa-spike-true-dfa-a911e793e590b0ed` **FAIL** (gap LCB 0.0733) — one honest attempt; stop.
- True e-prop (`c1x-eprop-true-*`): true-surrogate mean **0.7125** — methods contrast only.
- NumPy `xor_thresh`: broadcast **0.501** / DFA **0.827** / grad **0.773** — **locality flip** (cite 1-layer XOR, not depth).
- NumPy `depth_locality` mid: broadcast **0.816** / DFA **0.825** / rl_fb **0.803** — depth help ≠ locality flip (P7).
- Engine isolation / spike / project: [`RESEARCH_HARDENING.md`](RESEARCH_HARDENING.md).

---

## Explicit non-claims

Do **not** claim:

### SHD / attention read-out (the read-out program — the lead, so its non-claims come first)

1. **Novelty for temporal attention on SHD** — TA-SNN (ICCV 2021, 91.08%) and STSC-SNN (2022, 92.36%) precede this work. Only the *placement* (attention at the read-out alone) appears unoccupied, and a configuration gap is not a mechanism.
2. **Novelty for "SHD depends on temporal order"** — Cramer et al. (IEEE TNNLS 33(7), 2022) could not exceed **60%** spike-count-only; the Neuromorphic Sequential Arena (IJCAI 2025) reports 86.48 → 68.51; Yu et al. (arXiv:2507.16043, 2025) reach the same conclusion with two further operators. All prior.
3. **"96% (or 94.5%) of accuracy depends on temporal order"** — the fraction is of the read-out's **gain**, not of accuracy. Under shuffling the attention arm still reaches 0.6983 at n=12.
4. **Competitive SHD accuracy** — **0.8332** against a **95–96.4%** frontier. Do not present the 0.087 residual against the calibration reference as a tuning gap; the two forward passes differ in kind, and the kernel attribution rests on elimination and code-reading, not on an ablation that added the kernel.
5. **Any mechanism for the h1024 collapse** — located between h768 and h1024 (drop 0.2178, 6.9× the largest gap below it), and all three preregistered levers are negative (−0.2106, −0.2565, −0.0904). Overfitting is not excluded. Also do not claim a dip at h384: −0.0116 with sd 0.0253 is inside its own noise.
6. **SHD instrument calibration for attention** — criterion 5 (Python mirror) unmet; uncalibrated.
7. **Temporal-*resolution* mechanism for SHD attention** — S-5 is refuted and withdrawn (it moved bin width and sequence length together). On `fixed-tN`, which isolates the axis, the gain **shrinks** with finer resolution (−0.0453 across the ladder) — the opposite direction. Only temporal *order* is supported, via the shuffle inversion M-1 / M-2.
8. **`shd-scientific-sweep` claims** — withdrawn (synthetic 24-channel / 16-timestep data; never loaded SHD; [`DEFECT_2026-08-20_SHD_SWEEP_IS_SYNTHETIC.md`](DEFECT_2026-08-20_SHD_SWEEP_IS_SYNTHETIC.md)).
9. **Overnight SHD p27 / mac-probe ≡ proto-135 SHD sweep ≡ p29 full SuperSpike** — distinct protocols (20-way chance 0.05 capped e-prop vs 5-class chance 0.20 vs full-corpus SuperSpike BPTT `c1-shd-full-*`); do not mix.
10. **A recurrent-substrate win** — `rec+alif+attn` reaches 0.7874 against `ff+fixed+attn`'s 0.8289 at the same scale; the doubled gain is measured from a lower base (1.34× headroom-normalised), on ten pairs at the registered minimum, with residual survivorship exposure.

### Matched architecture / engine C1 (the matched-gate program)

11. **Biology / cortex** — motif-level substrate only.
12. **Assembly Calculus PASS** — `project` wired under `c1-project-*` and **FAIL**s G2.
13. **Natural-spiking G2 verdict** — scientific `c1-spike-*` / `c1-spike-s-*` are **INVALID_HARNESS** (PC below floor).
14. **True e-prop rescues broadcast insufficiency** — supporting methods note only.
15. **Neuromorphic hardware** — software harness only.
16. **Impossibility in principle** — scoped operationalized negatives only.
17. **Reopening protocol-v2 by threshold massage** — G2 thresholds and `c1-118207fbc3eaba53` stay frozen.
18. **Digital brain / brain equivalence**.
19. **Live-engine rescue from matched DFA / RL PASS** — transfer FAIL (v13–v24). Do **not** cite v131 / `live-transfer-rescue` as live PASS (matched-only).
20. **Structured `B` / capacity / eligibility / soft-WTA / continuous-B as G2 PASS** — floor cleared ≠ gate cleared.
21. **Coincidence-only proof that locality is required** — broadcast-graded also learns coincidence on the DFA schedule (**0.9975** on both graphs, archived 0.9863); cite XOR for locality flip. Do not replace "broadcast ±1 three-factor" with bare "broadcast credit topology" in a way that would ban graded broadcast.
22. **Undertraining as the matched ±1 three-factor FAIL cause** — v22 4× epochs still at chance.
23. **Any ranking among the passing matched arms** — the reference is at 1.0000 on both graphs, so every PASS reduces to "above 0.75" and five of seven arms sit between 0.88 and 1.00.
24. **The discrete spike-adjoint as a negative result** — withdrawn 2026-08-25 (0.5000 → 0.9450 / 0.8900 PASS). Equally, do **not** claim continuous / textbook EventProp equivalence or an EventProp rescue: the discrete H2H is not Wunderlich–Pehle's continuous adjoint, and its new PASS is a saturated-task PASS.
25. **The two RL broadcast contrasts as chance results** — withdrawn 2026-08-25 (0.5250 → 0.9100 / 0.8787; 0.5113 → 0.7962 / 0.7775). "Continuous magnitude without spatial directionality is insufficient on this gate" no longer has evidence behind it.
26. **A single matched forward family** — the two graphs disagree by more than the registered 0.02 bar on EventProp (0.0550) and RL graded-reward (0.0313); report two comparisons.
27. **Appendix G3 / G4 / hybrid H0 as reopening G2** — post-G2 harvest only; G4 NO-GO does not license remassaging the kill-gate.
28. **Hybrid T=2.0 collapse ≡ live v21 (T=1)** — separate protocols; do not equate.
29. **Online learned feedback alignment v130 PASS** — withdrawn under v131 (`INVALID_HARNESS` due to ceiling-inverted warnings; [`RESULT_2026-08-19_TRACK_B_V130_PASS_WITHDRAWN.md`](RESULT_2026-08-19_TRACK_B_V130_PASS_WITHDRAWN.md)). Under both repairs it reads 1.0000 against a ceiling of 1.0000 with zero variance — a **saturation** result, registered as explicitly not a credit-assignment one ([`RESULT_2026-08-23_TRACK_B_REREAD.md`](RESULT_2026-08-23_TRACK_B_REREAD.md)).
30. **Depth collapse / deep SNN scaling** — withdrawn under v134 (`INVALID_HARNESS`; all depth-matched gradient ceilings at chance; [`RESULT_2026-08-20_DEEP_SNN_V134_CEILING_IS_AT_CHANCE.md`](RESULT_2026-08-20_DEEP_SNN_V134_CEILING_IS_AT_CHANCE.md)).
31. **Retired `c1-*` config hashes as resolving** — `MATCHED_INPUT_SCALE` was never mixed into them, so each named two experiments either side of the repair. They no longer resolve, by design.

---

## Object-under-test checklist (honesty ledger)

| Item | Status |
|---|---|
| SHD attention read-out: gain is order-dependent (the lead) | **yes** — shuffle cost **+0.1347** (32/32) against the rate arm's **+0.0142**, **9.5×**; registered at n=12 as +0.1337 / +0.0128, 10× |
| SHD attention read-out headline | **yes** — `d32/L4` reaches **0.8332** against 0.7057, gain **+0.1275**, **32/32** positive, **32/32 ≥ 0.80** (registered n=12: 0.8320 / 0.7062 / +0.1258 / 12/12) |
| SHD headline competitive with the field | **no** — frontier 95–96.4%; this is a no-temporal-kernel instrument |
| SHD attention gain holds above h768 | **no** — threshold collapse into h1024 (drop 0.2178, 6.9×), unexplained; three levers negative |
| SHD attention temporal-*resolution* mechanism | **no** — S-5 refuted; on `fixed-tN` the gain shrinks with finer bins (−0.0453) |
| SHD instrument calibrated | **no** — criterion 5 (Python mirror) unmet |
| SHD shuffle applied to both splits per sample | **yes** — train and test, separate seed lineages; every `w9shf` cell passes the temporal audit |
| Read-out substitutes for substrate temporal state | **no** — refuted on both axes (adaptation inert, +0.0027; recurrent gain larger, +0.1411) |
| Event LIF + dendrites exist | yes |
| STDP eligibility + three-factor algebra live | yes |
| Natural hidden spiking during C1 integrate | **yes under `c1-spike-*` / `c1-spike-s-*`** (INVALID_HARNESS on PC); **no** under canonical v2 |
| Assembly Calculus `project` on C1 | **yes under `c1-project-*` (G2 FAIL)**; **no** under canonical v2 |
| Trial-isolated `last_spike` | **yes under `c1-iso` / spike/project**; **no** under canonical v2 |
| Matched DFA / RL reinforce_fb PASS | **yes** on both graphs in the 2026-08-25 re-run (DFA 0.9925 / 0.9875; RL 0.9950 / 0.9812) — against a reference at 1.0000, so "above 0.75" and no ordering |
| Matched RL online-learned FB PASS | **no** as a credit-assignment result (v130 withdrawn; v131 INVALID_HARNESS; 1.0000-vs-1.0000 is saturation) |
| Live RFB / gap-close G2 PASS | **no** (v13–v19 FAIL) |
| Live DFA / soft-WTA / finth / cont-B G2 PASS | **no** (v20–v24 FAIL) |
| v131 live-transfer-rescue = live Engine | **no** (matched-only; misnamed) |
| Discrete EventProp matched H2H FAIL | **no — withdrawn**; it PASSes on both graphs (0.9450 / 0.8900, gap LCB 0.7911 / 0.6494) |
| Matched undertrain rescues 3F | **no** (v22 FAIL @ chance) |
| Brain model | **no** |

New-protocol rows are closed with **new hashes**; they do not reinterpret v2 FAIL.
