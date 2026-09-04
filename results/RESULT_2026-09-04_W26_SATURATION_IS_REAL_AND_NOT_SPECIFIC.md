# Wave 26 — the saturation is real, and it is not specific to the collapse

**Registered:** [`PREREG_2026-09-03_W26_THE_SATURATION_AND_THE_TIMESCALE.md`](PREREG_2026-09-03_W26_THE_SATURATION_AND_THE_TIMESCALE.md),
committed before the first cell.
**Analyser:** `scripts/aws/analyse_wave26.py`, the authority on every verdict
below. Two amendments to it are disclosed in §5; both are committed, and one was
made after unblinding.
**Corpus:** 264/264 cells, **0 failures**, **0 INVALID**, `non_finite_forward: 0`
in every cell. 24/24 probe files, all parseable, both depths, all six registered
epochs.
**Binary:** one across the campaign, `434d38c2904e9ac5f75429fc1b2ef0a32ae17b26a9319bbf17c2aab14f0b160a`.

> **Reproduction gate, disclosed.** Cross-machine Gate F **FAILED on all six
> instances**. This is pre-existing and characterised, not new: the v2 fleet
> fails identically, and the divergence is exactly **0.0049** on accuracy,
> matching
> [`FINDING_2026-08-22_REPRODUCIBLE_ACROSS_ISA_UNDER_GLIBC.md`](FINDING_2026-08-22_REPRODUCIBLE_ACROSS_ISA_UNDER_GLIBC.md).
> Every number here is bit-reproducible *within* this fleet and binary, and is
> not bit-reproducible against the macOS build. No verdict below rests on a
> margin near 0.0049 except τ½, which is withheld for an unrelated registered
> reason.

## 1. H26-1 — the read-out saturates, and so does its control

| | d32l4 (collapsing) | d32l2 (control) |
|---|---:|---:|
| normalised entropy, e1 | 0.7125 | 0.6875 |
| e100 | 0.2138 | 0.1703 |
| e400 | **0.0021** | **0.0801** |
| drop e100→e400 | **0.2118** | 0.0902 |
| ‖q‖‖k‖ e1 → e400 | 46.4 → 8,587,447 (**185,179×**) | 39.8 → 914 (**22.97×**) |

n=12 at every entry.

**H26-1a: NOT MET.** The magnitude clause passed — the drop at `d32l4` is
0.2118 against a registered bar of ≥0.20. The **specificity** clause failed:
`d32l4` exceeds `d32l2` by 0.1216, against a registered margin of ≥0.15.

**H26-1b: NOT MET.** Again the magnitude clause passed, enormously — ‖q‖‖k‖
grows 185,179× at `d32l4` against a bar of ≥10×. The **control** clause failed:
`d32l2` grew 22.97×, where the bar required <3×.

**H26-1c** (secondary, reported not required): entropy at e400/`d32l4` is
0.0021, below the 0.30 threshold. The read-out is saturated in absolute terms.

### The registered reading does not fit this outcome, and is not used

§6 registers, for H26-1a NOT MET: *"The read-out does **not** saturate at
h1024/`d32l4`."* **That is not what happened, and writing it would be false.**
The read-out plainly does saturate: entropy falls from 0.71 to 0.0021 and the
named term grows five orders of magnitude. Both clauses failed on their
*control* half, not their magnitude half — a failure mode §6 did not anticipate
and therefore did not name.

What the data supports is narrower and is stated on its own terms: **saturation
is real at both depths, so it does not distinguish the configuration that
collapses from the one that does not.** A `d32l2` read-out that saturates to
0.0801 and grows its named term 23× is not a quiet control. Saturation is
therefore withdrawn *as an explanation of the h1024 collapse* — it fails to
separate the case from the control — while the observation that the read-out
saturates stands, and is new.

The QK-norm arm registered in §5 of the governing document loses the motivation
it was given here, which was to fix a *collapse-specific* saturation. It has not
acquired a prediction. This is the cheapest place to lose it: before any QK-norm
cell was run.

## 2. H26-2 — position is what the advantage runs through

| quantity | value | seeds |
|---|---:|---:|
| gain, default read-out, intact | 0.1258 | 12/12 positive |
| gain, no-position read-out, intact | 0.0506 | 12/12 positive |
| gain, default read-out, bin-shuffled | 0.0050 | 8/12 positive |
| **gain(default) − gain(no-position)** | **0.0752** | above 0.03 in **12/12** |

**H26-2a: MET.** Removing the positional code costs 0.0752 of the read-out's
advantage, above the campaign's 0.03 bar in every one of the twelve seed pairs.
Per §6: *position is what the read-out's advantage runs through, established
structurally rather than only by manipulating the data.* The arm keeps every
parameter — same width, same depth, same budget — and loses most of the gain.

**H26-2b (a registered question, not a prediction): the two ways of removing
order do not agree.** Removing the read-out's *access* to order leaves gain
0.0506; removing order from the *data* leaves 0.0050. |difference| = **0.0456**,
outside the ±0.03 band. Per §6, this is the paper's next question: either
something other than position carries order into the read-out, or shuffling
costs something other than order. A positionless read-out keeps a real advantage
(0.0506, 12/12 positive) that a shuffled input destroys (0.0050, 8/12).

Reported, not barred: DiD(attn vs no-position, `bin-shuffled`) = **0.1098**,
positive 12/12.

## 3. H26-3 — the ladder, and a τ½ withheld by 0.0008

| window | 2 | 4 | 8 | 16 | 32 | 64 | 128 | full shuffle |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| DiD | 0.0066 | −0.0028 | 0.0070 | 0.0206 | 0.0161 | 0.0245 | **0.0596** | **0.1208** |
| positive | 7/12 | 7/12 | 8/12 | 10/12 | 9/12 | 12/12 | 12/12 | 12/12 |

n=12 at every rung, each paired on a real seed quadruple.

**τ½ is WITHHELD**, by the registered rule *"the half-maximum is not reached on
the registered ladder"*. Half of the full effect is **0.0604**; the top rung
reaches **0.0596**. The ladder misses its own half-maximum by **0.0008**.

That margin is reported exactly as it fell. The rule was fixed in §1 of the
governing registration before any cell existed, and moving a ladder or a
threshold to capture a 0.0008 shortfall is precisely what preregistration exists
to prevent. It is not revisited.

What the curve says without τ½: permuting bins within 128-bin (256 ms) windows
recovers **49%** of the full shuffle's effect, and the ladder rises smoothly
across the whole range rather than turning over. So roughly half the read-out's
order-dependence lives at scales **longer than 256 ms** — longer than every rung
this ladder tested. The registered ladder was too short to find the scale, and
that is a finding about the ladder, not about the read-out.

## 4. Coverage

All 264 cells landed with `non_finite_forward: 0` and clean mechanical status;
0 failures, 0 INVALID. 73 cells are `CELL_PASS` and 191 `CELL_FAIL` under the
≥0.80 accuracy predicate — the expected split, since h128 rate arms sit near
0.70 and are not expected to pass it.

## 5. Two amendments to the frozen analyser

Both are committed with their reasoning, and neither moved a threshold, a bar,
or an estimator's algebra.

1. [`AMENDMENT_2026-09-03_A_CLAUSE_THAT_DID_NOT_RUN.md`](AMENDMENT_2026-09-03_A_CLAUSE_THAT_DID_NOT_RUN.md)
   — the analyser printed **NOT MET** for clauses whose inputs were absent, and
   for a fired void rule the registration says takes no verdict. Filed 20 minutes
   into the wave with zero probes written and every H26-1 statistic still `None`.
   Adds `NOT EVALUABLE` and exit 3.
2. [`AMENDMENT_2026-09-04_THE_LADDER_HAD_NO_BASELINE.md`](AMENDMENT_2026-09-04_THE_LADDER_HAD_NO_BASELINE.md)
   — `did()` drew `intact` from the manipulated wave, but §2 registers that H26-3
   *shares H26-2's* intact rungs, so all 168 `w26win` cells read as an absent
   effect. **Filed after the corpus completed and after H26-1 and H26-2 were
   read** — disclosed, and the weakest position from which to touch an analyser.
   The ladder itself was still blind when the change was made: every rung printed
   `NOT EVALUABLE` and no DiD existed for any window.

The second was findable only because of the first. Before it, the rungs printed
`DiD None` beside real numbers, and seven unevaluated rungs could have been
transcribed as seven measured nulls.

## 6. Cost and schedule, measured

| | |
|---|---:|
| slot-hours spent | 1,656 |
| fleet wall clock | 21.8 h × 6 `c7g.16xlarge` = 131 instance-hours |
| spot cost | **$90 – $94** at the measured $0.685–0.715/instance-hour |
| packing | 79% of purchased slot-hours used |

The registration estimated **509 slot-hours** and **$22.60**. The overrun is
**3.3×**, and the cause is measured rather than guessed: attention cells ran
about twice their archived median, while rate cells matched theirs exactly.

| configuration | archive (v2) | this wave |
|---|---:|---:|
| rate h128 e400 | 0.14 h | **0.15 h** |
| attn h128 d32l4 e400 | 5.69 h | **10.78 h** |
| attn h1024 d32l4 e400 | 3.46 h | **10.87 h** |

Only attention arms slowed, and Gate F is bit-identical on the local build, so
the arithmetic is unchanged — this is contention, not a regression: after the
first twenty minutes all 96 slots ran attention cells concurrently, where the
archived medians came from waves with rate arms interleaved. **Any future wave
that is predominantly attention should be costed at the concurrent-attention
median, not the archive's mixed-load median.**

## 7. What this wave does not answer

- **Why the read-out saturates at `d32l2` too.** The control was chosen to be a
  control; it is not one. Nothing here says whether `d32l2` saturation is benign.
- **The scale of the order-dependence.** The ladder stops at 256 ms and finds
  49% of the effect there. The remaining half is unlocated.
- **What carries order into the read-out besides position**, which is H26-2b's
  0.0456 gap and has no design.
- **The gain/DiD dissociation**, still the campaign's leading open problem and
  still without a design.
- **Nothing here is bit-reproducible off this fleet**, per the Gate F disclosure.
