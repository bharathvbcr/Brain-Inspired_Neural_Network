# Wave 27 — the read-out's order-dependence has a timescale, and it is 261 ms

**Registered:** [`PREREG_2026-09-04_W27_THE_INSTRUMENTS_THAT_WERE_NEVER_RUN.md`](PREREG_2026-09-04_W27_THE_INSTRUMENTS_THAT_WERE_NEVER_RUN.md),
committed before the first cell.
**Analyser:** `scripts/aws/analyse_wave27.py`, the authority on every verdict
below. One amendment to it is disclosed in §6.
**Corpus:** 264/264 cells, **0 failures**, **0 INVALID**. The v3 corpus now
holds 528 cells across waves 26 and 27, all valid.
**Binary:** `434d38c2904e9ac5f75429fc1b2ef0a32ae17b26a9319bbf17c2aab14f0b160a`
— wave 26's binary, unchanged, verified by `--require-binary-sha256` at launch.

> **Reproduction gate, disclosed.** Cross-machine Gate F **FAILED on all six
> instances**, as on every fleet in this campaign. Pre-existing and
> characterised: divergence exactly **0.0049** on accuracy, matching
> [`FINDING_2026-08-22_REPRODUCIBLE_ACROSS_ISA_UNDER_GLIBC.md`](FINDING_2026-08-22_REPRODUCIBLE_ACROSS_ISA_UNDER_GLIBC.md).
> Every number here is bit-reproducible *within* this fleet and binary and is
> not bit-reproducible against the macOS build.

## 1. H27-1 — the campaign's only exact zero holds in production

**MET.** All **12 of 12** seed pairs are byte-identical across every scientific
field: a rate read-out under `hidden-shuffled` pays **exactly nothing**, not
"nothing within a tolerance".

This is the gate that voids the entire wave if it fails, which is why it was
scheduled first. It had existed only as a unit test since it was registered.
It now holds on the real binary, in production, at the anchor — so the operator
does separate *read-out access to temporal structure* from *temporal structure
in the input*, and every other result in this wave stands on a valid instrument.

Reported, not barred: DiD(`hidden-shuffled`) = **0.1159**, positive **12/12**.
Registered as a question, and it has an answer: denying the read-out access to
the hidden train's time axis costs about what destroying order in the data costs
(0.1208). The two routes to the same deprivation agree.

## 2. H27-3 — the timescale, at last

| window | 2 | 4 | 8 | 16 | 32 | 64 | 128 | **192** | **256** | full |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| DiD | 0.0066 | −0.0028 | 0.0070 | 0.0206 | 0.0161 | 0.0245 | 0.0596 | **0.0814** | **0.1260** | 0.1208 |
| positive | 7/12 | 7/12 | 8/12 | 10/12 | 9/12 | 12/12 | 12/12 | 12/12 | 12/12 | 12/12 |

**τ½ = 130.33 bins = 260.7 ms**, interpolated between w128 and the new w192
rung, on a ladder monotone within the registered ±0.02.

Wave 26 could not produce this number: its ladder stopped at w128 and reached
0.0596 against a half-maximum of 0.0604, missing by 0.0008. One rung at w192
settles it. The estimator and every withholding rule are unchanged from the
registration that predates both waves.

**What it means.** At this anchor an utterance is 357.9 bins ≈ **716 ms**, so
the read-out's order-dependence has a half-life at about **36% of the
utterance**. Order below ~260 ms is worth about half of what order across the
whole utterance is worth; the rest lives at longer scales.

The ladder saturates where it should: at w256 the window covers 71% of a
715.8 ms utterance, and its DiD (0.1260) is indistinguishable from the full
shuffle's (0.1208, a gap of +0.0052 on 12 seeds). A manipulation that is nearly
a full shuffle costs nearly what a full shuffle costs, which is the consistency
check the top of the ladder exists to provide.

## 3. H27-2 — the dropout instrument does not work, and that is the finding

**NOT MET.** At `p30` the rate arm loses **0.0124** of accuracy from `intact`
— **41% of the registered 0.03 bar** — and clears that bar in **0 of 12** seeds.

The bar was registered as *a check on the manipulation, not on the read-out*,
and the manipulation failed it. Per §3 of the registration this is a finding
about the **instrument**: dropout at p30 does not cost the substrate enough to
be sensitive, so **no null measured under it is interpretable**, and §2's
ambiguity — does a null mean *order does not matter* or *the measurement cannot
see it* — **is not resolved and remains open**.

Reported, not barred: DiD(`spike-dropout-p30`) = 0.0015, positive 7/12. That
null is exactly the kind this wave has just established it cannot read.

The operator itself is correct: the audit shows 57,845,244 of 82,625,050 spikes
retained, **70.01%**, from a mask frozen before the first epoch. It deletes what
it says it deletes. It simply does not hurt enough at 30% to measure with.

## 4. H27-4 — QK-norm is a different read-out, on both halves

**NOT MET**, and not marginally:

| half | value | band |
|---|---:|---:|
| accuracy(qk-norm) − accuracy(default), intact | **−0.0692** | ±0.03 |
| DiD(bin-shuffled) under qk-norm − under default | **−0.0577** | ±0.03 |

(qk-norm DiD 0.0632 against the default's 0.1208, n=12 on both.)

Per §5: qk-norm is a **different read-out**, and nothing measured with it
transfers to the paper's headline. It is not the same instrument with the score
term bounded — it is worse at the task *and* it consumes order differently,
losing about half the order-dependence the default read-out has.

**This was already a read-out check and not a test of the collapse
hypothesis.** Wave 26 withdrew the collapse-specific motivation by refuting the
specificity of saturation, and the registration said in advance that a MET here
would not have resurrected it. A NOT MET does not either. What it settles is
narrower and still useful: the arm §5 proposed as a fix is not a drop-in
replacement, and the h1024 half that §5 made conditional on this bar **must not
be run against the default arm's numbers**.

## 5. H27-5 and H27-6 — questions and thresholds, carrying no verdict

**The τ_m ladder ran clean: no rung voided.** Every rung had
`saturated_fraction` **0.0** and **0 of 12** cells over the 0.05 gate, so the
standing void rule never fired and no rung was lost to survivorship.

| τ (ms) | 2.5 | 5.0 | 10.05 | 20.0 | 40.0 |
|---|---:|---:|---:|---:|---:|
| rate accuracy | 0.7041 | 0.7026 | 0.7062 | 0.7149 | 0.7238 |
| DiD vs the calibrated point | 0.0076 | 0.0073 | 0.0 | 0.0156 | 0.0275 |
| positive | 9/12 | 7/12 | 0/12 | 8/12 | 11/12 |

The 10.05 rung reads exactly 0.0 in 0/12 because it *is* the calibrated point:
DiD against itself is identically zero. That is the ladder's own zero and it
validates the estimator rather than measuring anything.

Read as a question, not a verdict: a longer membrane constant helps the **rate**
arm more than the attention arm — rate accuracy climbs 0.7041 → 0.7238 across
the ladder while the DiD grows to 0.0275 at τ=40 ms, approaching but not
crossing the campaign's 0.03 bar. Some of what the read-out supplies may also be
obtainable by integrating longer in the substrate. **That is a hypothesis this
wave generated and did not test**, and it needs its own registration.

**Cross-wave reproduction, obtained free from a registered rung: 12/12 rate and
12/12 attention byte-identical** at τ=10.05 against wave 26's `intact` arms.
`--tau-m 10.05` is the default path, and it reproduces wave 26 exactly, cell for
cell, on the same fleet and binary.

**H27-6, reported and not scored.** Rate: test 0.6786, val **0.7035**.
Attention: test 0.7926, val **0.6661**. The attention arm falls 0.127 on
speakers it never trained on while the rate arm *rises*. §7 registered this
split as **model selection only — no headline number, no DiD, no bar** — and
these cells train on 6,987 samples rather than 8,156, so they are not comparable
to the corpus. **The asymmetry is suggestive and is not a result.** It cannot
become one without its own registration, and this document does not give it one.

## 6. One amendment, and it is the same defect twice

[`AMENDMENT_2026-09-05_THE_SAME_DEFECT_THROUGH_THE_OTHER_ARM.md`](AMENDMENT_2026-09-05_THE_SAME_DEFECT_THROUGH_THE_OTHER_ARM.md)
— `did()` assumed the rate arms live in the same wave as the attention arms.
`w27qk` carries only attention cells by design, so half of a registered two-part
bar could not be computed with all 24 of its cells present and valid.

**This is wave 26's ladder defect returning.** That amendment added
`intact_wave`, fixing the instance; the shape survived as "all four arms share
two waves" and came back through the arm the first fix did not touch. The
standing rule is *fix the class, not the case*, and the first amendment did not.
The class is fixed now: every arm may name its own wave.

No threshold and no estimator algebra changed. A regression test pins the
dropout DiD to its exact value and passes against the pre-fix analyser too, to
catch a fix that silently re-points an arm that was already right.

**The failure direction was the safe one.** Before the fix H27-4 read
NOT EVALUABLE, yet the accuracy half alone (−0.0692) already established
NOT MET. The analyser withheld a verdict it could have reached rather than
announcing one it could not — which is the direction this discipline is built to
fail in.

## 7. Cost, and a prediction that held

| | registered | actual |
|---|---:|---:|
| slot-hours | 1,573 | **1,553** |
| spot cost | $86 – $110 | **$90 – $94** |
| makespan | 20 – 24 h | **21.9 h** |

Within **1.3%** on slot-hours. Wave 26 overran its registration 3.3× because it
was costed at the archive's mixed-load medians while running an all-attention
fleet; wave 27 was costed at wave 26's own measured medians — attention h128
`d32l4` e400 at 10.78 h — and the wave came back at a 10.84 h median. The
correction worked, and the lesson generalises: **cost an attention-heavy wave at
concurrent-attention medians.**

## 8. What this wave does not answer

- **Whether order matters below the dropout instrument's sensitivity.** H27-2
  leaves §2's ambiguity open. A higher dropout rate is the obvious next step and
  is not registered.
- **Where the other half of the order-dependence lives.** τ½ is 261 ms; the
  ladder says nothing about the structure above it beyond that it saturates.
- **Whether a longer τ substitutes for the read-out.** §5's trend is a
  hypothesis this wave generated, at one substrate, feed-forward only.
- **Why `d32l2` saturates**, wave 26's open question, still has no design.
- **The gain/DiD dissociation**, the campaign's leading open problem, still has
  no design.
- **Nothing here is bit-reproducible off this fleet.**
