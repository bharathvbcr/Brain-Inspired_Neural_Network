# Audit — the cross-repository review of September 4, read against the record

**Written:** 2026-09-05, after
[`RESULT_2026-09-05_W27_THE_TIMESCALE_IS_261_MS.md`](RESULT_2026-09-05_W27_THE_TIMESCALE_IS_261_MS.md)
landed and
[`PREREG_2026-09-05_W28_THE_RATE_AT_WHICH_DROPOUT_BITES.md`](PREREG_2026-09-05_W28_THE_RATE_AT_WHICH_DROPOUT_BITES.md)
was registered. **This document registers nothing, changes no verdict, and may not be
cited as a result.** It is a systematic re-reading of one external document against
this repository's own records, filed so that what an outside reader concluded about
BINN, and what has moved since, is on the record in the same place as the results.

---

## 0. What was reviewed, by whom, and what it could see

On 2026-09-04 a review titled *Architectural opportunities in MLSystemsLab and BINN*
(Codex / "GPT-6 Astra"; inventory timestamp 21:48 UTC) read a snapshot of the sibling
repository `MLSystemsLab` and, through it, BINN's report documents. It is preserved
there as `docs/architecture-review-2026-09-04/EVIDENCE_REVIEW.md`, with a consolidated
verdict in `docs/architecture-review-2026-09-04/README.md` and a proposed memory
protocol in `docs/architecture-review-2026-09-04/MEMORY_UPDATE_PROTOCOL.md` (sibling
repository; not bundled here, and none of its links are checked by this repository's
tooling). Its stated scope: "BINN supplies a related source of temporal-mechanism
hypotheses"; it did not review BINN's code, and it had BINN's *reports* but not its
*cells* — its own words are that the W26 raw cells "were unavailable for local
re-analysis". Everything below is therefore a check of a report-backed reading, not
of an independent recomputation.

The review's BINN conclusions were written before wave 27 existed. §3 records what
wave 27 did to them.

---

## 1. Every statement the review makes about BINN, checked

| the review says | the record says | verdict |
|---|---|---|
| W9's attention read-out gain "depends strongly on intact temporal structure" | [W9](RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md): at d32/L4, intact 0.8320, bin-shuffled 0.6983, cost of shuffling +0.1337, intact > shuffled in 12 of 12 seeds; the `ff+fixed` arm pays +0.0128 for the same shuffle. | **holds** |
| W25 "has an unevaluable recurrent comparison" | [W25](RESULT_2026-09-03_W25_THE_MECHANISM_WHERE_IT_WAS_UNMEASURED.md) H25-1 **NOT EVALUABLE**: 10 of 72 recurrent cells aborted on the instrument's own non-finite guard (`rec+alif` at `surrogate_scale` 0.4), 877 of 888 cells landed, and the `reversed` contrast reached 7 and 9 seed-paired quadruples against a registered floor of 9. | **holds** |
| those failures "are not evidence that a new learning rule succeeds or that recurrence is impossible" | W25 §2 says the same: the question "still has not been measured", and the failures are a stability finding about one surrogate scale. | **holds**, and is the record's own reading |
| W26 finds "a structural positional-code effect and a residual gap versus shuffling": 0.1258 → 0.0506 without position, 0.0050 under bin shuffling | [W26](RESULT_2026-09-04_W26_SATURATION_IS_REAL_AND_NOT_SPECIFIC.md) H26-2a **MET**: gain 0.1258 (default, intact), 0.0506 (no-position, intact), 0.0050 (default, bin-shuffled); the 0.0752 loss clears 0.03 in 12 of 12. | **holds** |
| removing positional features and shuffling raw input "are not equivalent interventions"; the residual gain "is an unanswered mechanism question, not proof of a new temporal code" | That is H26-2b, registered as a question: the two routes to removing order disagree by 0.0456, outside the ±0.03 band, and the write-up calls it "the paper's next question" with no design. | **holds**, and is the record's own reading |
| saturation "occurs in both the collapsing readout and its control, defeating the registered collapse-specific explanation" | H26-1a and H26-1b **NOT MET** on their *control* clauses: entropy drop 0.2118 at `d32l4` against 0.0902 at `d32l2` (margin 0.1216 < 0.15); ‖q‖‖k‖ grows 185,179× against 22.97× (control bar < 3×). Saturation is withdrawn as an explanation of the collapse and stands as an observation. | **holds** |
| "Saturation also does not logically rule out a useful QK-normalization intervention" | True on 2026-09-04. On 2026-09-05 W27 measured it: H27-4 **NOT MET** on both halves (accuracy −0.0692, DiD −0.0577, band ±0.03). QK-norm is a different read-out, not a bounded version of the same one. | **superseded** by W27 |
| W26's "264/264 coverage claim and exact numbers were not rederived" | Coverage is the analyser's own count; the cells are in this repository (§4). The review could not see them, so it could not rederive anything; that is a limit of its snapshot, not of the record. But see §4: this repository's *own* every-number sweep did not cover W26 either on the day of this audit. | **holds as a statement about the review's snapshot; §4 for the record's side** |
| "keep the BINN temporal branch exploratory until its cell artifacts and mechanism tests are available" | Artifacts: tracked, §4. Mechanism tests: W27's H27-1 exact-zero gate holds in production (12 of 12 byte-identical), and H27-3 gives the order-dependence a timescale. What the review means by "mechanism tests" is broader than either (§2). | **partly superseded**; the branch is still exploratory by this repository's own standards, for the reasons W26 §7 and W27 §8 list |
| the memory proposal "separate content, relative order, and synchrony in event memory" has the "weakest basis for an immediate general-purpose LM claim" | Nothing in this repository makes an LM claim, and no wave tests a general-purpose memory. The proposal is the reviewer's; it is not registered here and inherits nothing from BINN's verdicts. | **outside the record**; no position |

Nothing the review states about BINN's *results* is wrong. Its two forward-looking
sentences (QK-norm; "until artifacts and tests are available") were overtaken within a
day.

---

## 2. What the review asks of BINN, against what is registered

| the review recommends | nearest registered work | status |
|---|---|---|
| "separate temporal binding from storage with variable cue–value delays, overlapping entities, and distractors" | none. Every manipulation in waves 9–27 acts on the *data's* temporal structure (shuffles, windows, reversal, dropout) or on the read-out's *access* to it (no-position, hidden-shuffled). No wave varies a cue–value delay, an entity overlap or a distractor load as a factor. | **not registered**; a new instrument, not a new arm |
| "match count/order/coincidence tasks" | [W24](RESULT_2026-09-02_W24_ORDER_SYNCHRONY_AND_BUDGET.md) (order, synchrony and budget) and W25 H25-2, which answered that the synchrony term is a resolution effect at all three `fixed-tN` rungs. | **partly measured**, at one dataset and one anchor |
| "test bin-width and timescale transfer" | Timescale: W26 H26-3 (ladder to w128, τ½ withheld by 0.0008) and W27 H27-3 (τ½ = 130.33 bins = 260.7 ms, interpolated between w128 and w192; w256 indistinguishable from the full shuffle, 0.1260 vs 0.1208). Bin width: W26 §7 and W27 §8 list no transfer test, and W27's τ_m ladder is a hypothesis it "generated and did not test". | **timescale measured at one anchor; transfer across anchors and bin widths not** |
| "compare oracle and learned binding with the same downstream memory … a second dataset" | none; SHD only. Open problem 2 of [`PAPER_BINN_INSTRUMENT_2026-08-31.md`](PAPER_BINN_INSTRUMENT_2026-08-31.md) §11.1 (calibration; nothing here is comparable to an externally recorded number until criterion 5 is met) is the nearer blocker. | **not registered** |
| cell artifacts and mechanism tests "available" | §4 and §3. | see there |

---

## 3. What landed after the review's snapshot

- **Wave 27** ([result](RESULT_2026-09-05_W27_THE_TIMESCALE_IS_261_MS.md), 264/264 cells,
  same binary as wave 26): H27-1 MET (a rate read-out under `hidden-shuffled` pays
  exactly nothing, 12 of 12 byte-identical; reported DiD 0.1159, positive 12/12);
  H27-3 τ½ = 260.7 ms, about 36% of a 716 ms utterance; H27-2 NOT MET (dropout at p30
  costs the rate arm 0.0124, 41% of its bar, in 0 of 12 seeds — the instrument cannot
  feel its own hand, so nulls under it are unreadable); H27-4 NOT MET (QK-norm is a
  different read-out); H27-5 a τ_m trend reported as a question; H27-6 a speaker split
  registered as model selection only.
- **One amendment, disclosed after unblinding**
  ([`AMENDMENT_2026-09-05_THE_SAME_DEFECT_THROUGH_THE_OTHER_ARM.md`](AMENDMENT_2026-09-05_THE_SAME_DEFECT_THROUGH_THE_OTHER_ARM.md)):
  the wave-26 `intact_wave` fix had fixed the case, not the class, and the same
  estimator defect returned through the rate arms; every arm may now name its own wave.
- **Wave 28 registered** before any cell: the dropout rate at which the instrument
  becomes sensitive. It is in flight and is unaffected by anything in this audit.
- **The counterfactual arm** (`DESIGN_2026-09-05_THE_COUNTERFACTUAL_ARM.md`) exists as
  an instrument with no wave: it addresses open problem 4 of the instrument paper (the
  k-WTA transfer barrier), which the review did not discuss.

None of this touches the review's MLSystemsLab-side findings (a Gated DeltaNet
operator variant and a top-1 mixture-of-experts router with no task gradient). Those
concern `nanolab`, not this repository's code.

---

## 4. A gap in this repository's own gates, found while checking §1

The review said W26's cells were unavailable. They are here: the v3 corpus is tracked
in git under `shd_attention_campaign_v3/` — 265 wave-26 and 265 wave-27 cell files, three
plans ([`plan_w26.json`](shd_attention_campaign_v3/plan_w26.json), `plan_w27.json`,
`plan_w28.json`), 24 probe files; 555 tracked files in all. Anyone with the checkout can
rederive every number in W26 and W27 from them.

But the tooling that is supposed to do that automatically does not yet look there. Run
on 2026-09-05 at 15:30 UTC, on a clean tree at `ec7ea11`:

- [`../scripts/check_every_number.py`](../scripts/check_every_number.py): `CORPORA` lists
  `shd_attention_campaign_v1/cells`, `shd_attention_campaign_v2` and the Azure results;
  v3 is absent. Its document glob (`RESULT_*_W[0-9]*.md`) does include the two new
  results, so the sweep reports them as **FAIL — UNEXPLAINED**: five numbers in W26
  (0.0506, 0.0801, 0.1098, 0.1216, 0.2118) and three in W27 (0.7026, 0.7035, 0.7238)
  "the cells cannot produce", which is true of the cells it reads and false of the cells
  on disk. Every other number in both documents was matched by coincidence against the
  6,763 quantities the v1/v2 corpora generate, which the script itself warns accepts a
  random four-decimal value 37% of the time through `paired`. So W26 and W27 are
  currently *unchecked*, not checked-and-passing, and the sweep's verdict on them is
  noise in both directions.
- `check_verdicts_transcribed.py`: "48 verdicts cross-checked … 2 problem(s)": W26 and W27
  are "neither cross-checked nor declared in NO_VERDICTS". No `VERDICTS_W26.md` or
  `VERDICTS_W27.md` exists anywhere under `results/`; `analyse_wave26.py` and
  `analyse_wave27.py` print their verdicts rather than writing the file the check pairs
  against.

`record_checks.sh` runs both, so it is red on the two newest results. **This audit does
not fix it.** The fix-the-class version is: add the v3 corpus to `CORPORA` (and whatever
its cell schema needs), have every wave analyser write a `VERDICTS_W*.md` as the earlier
ones did, and make the "every wave result is one or the other" rule fail on a missing
verdict file rather than on a missing `PAIRS` entry. Until then, W26 and W27 rest on
their frozen analysers and on the hand-transcription discipline the checks exist to
replace — exactly the state the checks were written to end.

Cross-machine reproduction is a separate, already-disclosed limit: Gate F fails on
every fleet in this campaign by exactly 0.0049
([`FINDING_2026-08-22_REPRODUCIBLE_ACROSS_ISA_UNDER_GLIBC.md`](FINDING_2026-08-22_REPRODUCIBLE_ACROSS_ISA_UNDER_GLIBC.md)),
so a reader recomputing W26/W27 on macOS will not get these bytes, and both results say
so in their first lines.

---

## 5. What this audit does not do

- It does not register a wave, a bar, or a prediction; the delay/entity/distractor
  factorial of §2 stays a proposal until someone writes its preregistration.
- It does not change any verdict, threshold or number in any result.
- It does not fix the gate gap in §4; that is source work with its own tests.
- It does not verify the sibling repository's numbers, only what it says about this one.
