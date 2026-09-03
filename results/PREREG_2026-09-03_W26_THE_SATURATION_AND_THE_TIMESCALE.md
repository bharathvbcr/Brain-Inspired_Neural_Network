# Preregistration — wave 26: the collapse's mechanism, the structural null, and the timescale

**Registered:** 2026-09-03, **before any cell of `w26sat`, `w26pos` or `w26win`
exists and before any instance for them is launched.** Attested by git history:
this file, `scripts/aws/analyse_wave26.py`, `scripts/test_wave26_analyser.py`
and the `wave26_the_saturation_and_the_timescale` plan function are committed
together, and the first cell is produced afterwards.

**Analyser:** `scripts/aws/analyse_wave26.py`, frozen in the same commit and the
authority on every verdict below. Twelve tests in
`scripts/test_wave26_analyser.py`.

**Binary:** new, and that is the wave's defining constraint. See §1.

The campaign's convention is to name the **fleet** (aarch64/AL2023) hash, and it
does not exist yet: the first instance builds it from the uploaded source and
publishes the pin, exactly as `3afd4434…` was created for wave 22. What is
already measured is the **local** macOS/arm64 build,
`fe7904a48dd4160948876d94eeb3f9b0571a03f316d35c26bef0802a77085669`, which is the
binary the reproduction gate in §1 ran against. The two are different builds of
the same source and will not share a hash; the fleet hash is recorded per
instance in `gates/` and is transcribed into the result document.

**Bucket:** `s3://binn-campaign-v3-511192439661-us-east-1`, **new**, and for the
same reason. `binn-campaign-v2` pins `3afd4434…`, the binary all 1,790 of its
archived results came from, and `bootstrap.sh` treats a published pin as
mandatory — an instance there would download the old binary and reject every one
of this wave's flags. Replacing the pin in place would overwrite the artefact
that those 1,790 results' provenance points at. A new bucket per binary
generation is the campaign's existing pattern (`binn-campaign` → `-v2`), it is
non-destructive, and it makes "self-contained on one binary" true at the storage
layer rather than only by convention.

Results are collected into `results/shd_attention_campaign_v3/`, and the frozen
analyser is pointed at it with `--results`.

**Governing registration:**
[`PREREG_2026-09-03_THE_INSTRUMENT_BEFORE_THE_WAVE.md`](PREREG_2026-09-03_THE_INSTRUMENT_BEFORE_THE_WAVE.md),
which fixed the estimators and bars this wave inherits. §7 below records the two
places this document **supersedes** it, and why each is a repair rather than a
choice.

---

## 1. Why this wave re-runs cells the corpus already has

Every instrument here is new code, so the binary is new. §0 of the governing
registration forbids pairing a cell from it against an archived half: an
`intact` cell from `w22cov` and a `window-shuffled-w8` cell from this binary are
not a pair, and a difference-in-differences over them would be measuring the
compiler alongside the manipulation.

So the rate arms and the intact attention arms are re-run here even though the
corpus holds hundreds of them. That is 88 of the 264 cells and it is not
redundancy; it is the difference between a self-contained experiment and a
comparison across two binaries.

**The reproduction gate has run.** `scripts/gate_f_rust.py` on this binary:
**15 of 15 cells bit-identical** against the macOS reference corpus, including
`rust__fixed-t100__adjacent-sum-5__h128__e20__s5170001__ff-fixed-attn__d32l1`,
which is the one attention cell in that corpus and exercises the whole changed
path. Each instance re-runs `--cheapest 3` on aarch64 at boot and records PASS
or FAIL into `gates/`, as every wave since 15 has.

## 2. Design

| group | cells | what runs |
|---|---:|---|
| **H26-1** `w26sat` | 24 | `ff+fixed+attn` h1024 `d32l4` and `d32l2`, e400, anchor, **probed** at epochs 1/25/50/100/200/400 × 12 seeds |
| **H26-2** `w26pos` | 72 | rate, `attn` default, `attn` **no-position**, each × `intact`/`bin-shuffled` × 12 seeds, h128 `d32l4` e400 anchor |
| **H26-3** `w26win` | 168 | rate and `attn` × **7 window rungs** {2,4,8,16,32,64,128} × 12 seeds, same operating point |
| **total** | **264** | |

H26-3 shares H26-2's `intact` and `bin-shuffled` rungs — the same cells, in the
same wave, on the same binary — which is why they are not restated.

**Cost, from the archive's own measured medians** at each configuration, not
from an estimator: **509.1 slot-hours**, **$22.60** at the calibrated
$0.0444/slot-hour. Longest single cell 3.36 h, which is the makespan floor. Six
`c7g.16xlarge` give 5.3 h of ideal packing against that floor.

## 3. H26-1 — does the h1024 collapse saturate the read-out?

[`RESULT_2026-08-30_W23_THE_COLLAPSE_IS_LATE.md`](RESULT_2026-08-30_W23_THE_COLLAPSE_IS_LATE.md)
established **where**: the collapse is late, and truncating to e100 avoids it —
**+0.0827** against e400's **−0.1318**, while `d32l2` moves only **+0.0149**
between the two budgets.
[`FINDING_2026-08-29_THE_H1024_COLLAPSE_IS_A_LOST_FIT.md`](FINDING_2026-08-29_THE_H1024_COLLAPSE_IS_A_LOST_FIT.md)
established **what kind**: a lost fit, 63 of 68 cells ending 56× above their own
best training loss. Neither says **why**, because no cell has ever recorded
anything about the read-out's internal state.

**H26-1a — the read-out saturates where the fit is lost, and not in the control.**
Mean normalised attention-row entropy at `d32l4` falls by **≥ 0.20** between
epoch 100 and epoch 400, and that fall exceeds `d32l2`'s over the same interval
by **≥ 0.15**.

**H26-1b — the named term is what grows.** Median `mean(‖q‖) × mean(‖k‖)` rises
by **≥ 10×** between epoch 1 and epoch 400 at `d32l4`, and by **< 3×** at
`d32l2`.

**Both are required.** Entropy collapsing without the norms growing is a
different mechanism and is reported as a **new** open problem, not as
confirmation — the discipline H15-2 established, for the same reason.

**H26-1c — the absolute scale.** Mean normalised entropy at e400/`d32l4` is
below **0.30**. *Reported, not required.* Normalised entropy has never been
measured on this instrument, so an absolute threshold on it is a guess in a way
the comparative clauses are not.

**Void condition.** If mean normalised entropy at `d32l4` is already below 0.30
at **epoch 1**, the probe is measuring initialisation rather than training, the
epoch grid is wrong, and **no verdict is taken from H26-1**.

## 4. H26-2 — the structural null for the whole mechanism claim

Every order result in this paper removes order from the **data**. This removes
the read-out's ability to **use** it: without the positional code, mean-pooled
attention is permutation-invariant, so the arm keeps every parameter, every
pairwise interaction and the identical initialisation, and loses the only path
to order.

**H26-2a — position is what the advantage runs through.** Seed-paired,
`gain(default) − gain(no-position)` at `intact` is **> +0.03**, and above that
bar in **≥ 9 of 12** seeds. `gain` is attention minus the rate arm, the paper's
own quantity.

**H26-2b — do the two ways of removing order agree?** *Registered as a question,
not a prediction.* Is `|gain(no-position, intact) − gain(default, bin-shuffled)|`
inside ±0.03? If position is exactly the read-out's path to order, removing the
code and removing the order should cost the same thing. The campaign has no
basis for predicting exact agreement and does not claim one.

## 5. H26-3 — at what timescale does the read-out use order?

`bin-shuffled` destroys order at every scale at once and cannot say which scale
mattered. Seven windows turn one number into a curve.

**Registered as a question.** The estimator is **τ½**, fixed in §1 of the
governing registration: the smallest window at which `DiD(N)` reaches half
`DiD(bin-shuffled)`, linearly interpolated between bracketing rungs, reported in
bins and converted at the anchor's 2 ms.

**τ½ is withheld** — not estimated — if `DiD(bin-shuffled) ≤ +0.03` (no order
effect to locate), if the ladder is non-monotone beyond ±0.02 (a half-maximum on
a non-monotone curve is an artefact of which crossing you take), or if the
half-maximum is never reached on the registered ladder. The analyser prints
which of these fired.

## 6. Named outcomes, in every direction

| outcome | reading |
|---|---|
| **H26-1a and H26-1b MET** | The collapse has a mechanism. §3.5's "located but unexplained" is retired, and the QK-norm arm registered in §5 of the governing document acquires a prediction rather than a hope. |
| **H26-1a MET, H26-1b NOT** | The read-out saturates and the proposed cause is wrong. Reported as a **new** open problem, narrower than the current one. |
| **H26-1a NOT MET** | The read-out does **not** saturate at h1024/`d32l4`. The saturation account is withdrawn, and QK-norm's registration loses its motivation before any QK-norm cell is run — which is the cheapest place to lose it. |
| **H26-2a MET** | Position is what the read-out's advantage runs through, established structurally rather than only by manipulating the data. |
| **H26-2a NOT MET** | Removing the positional code does **not** cost the read-out its advantage. The paper's account of *what* the read-out consumes is wrong, and that is a finding about the mechanism claim, not about this wave. It would be the most consequential result here. |
| **H26-2b inside ±0.03** | The two ways of removing order agree, and §3.5's shuffle result and this null are one finding rather than two. |
| **H26-2b outside** | They do not agree, and the gap is the paper's next question: something other than position carries order into the read-out, or shuffling costs something other than order. |
| **τ½ estimated** | The order the read-out uses has a scale, in milliseconds, and the paper can say which. |
| **τ½ withheld** | The reason is printed and reported. A withheld τ½ on a non-monotone ladder is a finding about the ladder. |

## 7. Where this supersedes the governing registration, and why

Both repairs are made **before any cell exists**. The rule this campaign holds
is that no threshold moves after a cell lands; nothing has landed.

**§4's H26-2 bar was backwards.** It read: *"`DiD` computed with the
`no-position` arm in place of the rate arm is ≥ +0.03 **smaller** than the
standard `DiD`."* That is the wrong sign. If the no-position arm is order-blind,
shuffling costs it roughly nothing, so `(nopos_intact − nopos_shuffled) ≈ 0` and
that DiD would come out slightly **larger** than the standard one, not smaller —
the bar as written would have been failed by the very result it was meant to
confirm. §4 above replaces it with a statement about the **gain**, which is what
the sentence beside the bar actually described. The DiD form is still computed
and printed; it is not barred.

**§6's H26-1 clause 1 was an absolute threshold on a quantity never measured.**
It required entropy `< 0.30` at e400 and `> 0.60` at the controls. Normalised
entropy has no prior measurement on this instrument, so those numbers are
guesses, and a wave whose primary clause rests on a guessed scale can fail for
having guessed the scale rather than for the mechanism being absent. §3 makes
the primary clauses **comparative** — a drop, and a drop larger than the
control's — which is robust to the absolute scale being anything, and keeps the
absolute threshold as a reported secondary.

Neither repair loosens a bar. The first fixes a sign; the second replaces one
guessed threshold with two comparisons and demotes the guess.

## 8. What this wave will not answer

- **No QK-norm cell runs here.** §5 of the governing registration requires its
  h128 bar and its h1024 arm in one wave, and that wave is only worth running if
  H26-1 supports the mechanism it tests. This wave decides whether that one
  should exist.
- **No `spike-dropout`, `hidden-shuffled`, `tau_m` or speaker-split cell runs
  here.** Those instruments are built, registered and tested, and unrun.
- **H26-1 is one width and one contract** — h1024, `published-2ms`,
  `adjacent-sum-5`. It says nothing about whether a read-out saturates anywhere
  it does not collapse.
- **The ladder is one operating point.** h128, `d32l4`, e400, anchor. τ½ at one
  point is not a timescale of the task.
- **Nothing here addresses the gain/DiD dissociation**, which remains the
  paper's leading open problem and still has no design.
