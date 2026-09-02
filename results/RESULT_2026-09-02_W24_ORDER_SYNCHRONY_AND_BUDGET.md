# Result — the shuffle cost is about order, it survives a second budget, and at one point synchrony matters too

**Registered:** [`PREREG_2026-09-01_ORDER_SYNCHRONY_AND_BUDGET.md`](PREREG_2026-09-01_ORDER_SYNCHRONY_AND_BUDGET.md),
committed at `f679d71` **before any cell of this wave existed**.
**Analyser:** `scripts/aws/analyse_wave24.py`, frozen in that same commit and the
authority on every verdict below.
**Cells:** 336 of 336, every one valid, every one `non_finite_forward: 0`.

---

## 1. What was at stake

Every difference-in-differences in this campaign — all 21 operating points after
wave 22 — was built from **one** destruction operator. `bin-shuffled` destroys
temporal order *and* displaces almost every spike, and nothing separated the
two. On that evidence "attention reads temporal order" was indistinguishable
from "attention is more brittle than the rate read-out to having its input moved
about." A reviewer could delete the paper's mechanism claim without disputing a
number in it.

Separately, **no `bin-shuffled` cell had ever run at any budget but `e400`**, at
any point, in any wave.

## 2. Verdicts

| hypothesis | verdict |
|---|---|
| **H24-1** — the cost is not generic perturbation brittleness | **MET** at all three points |
| **H24-2** — does cross-channel synchrony contribute? *(question)* | **order alone at two points; synchrony also read at the third** |
| **H24-3** — the contrast survives a second budget | **MET** at both points |
| **H24-4** — the order control holds at the second budget too | **MET** at both points |
| **H24-5** — cells | **MET**, 336 of 336, 0 voided |

### H24-1 — the null costs nothing

| point | DiD(`bin-shuffled`) | DiD(`reversed`) | positive seeds |
|---|---:|---:|---:|
| h128 / `published-2ms` / `d32L2` | +0.1145 | **−0.0222** | 2/12 |
| h1024 / `published-2ms` / `d32L1` | +0.0675 | **−0.0040** | 4/12 |
| h128 / `fixed-t250` / `d32L4` | +0.1119 | **−0.0099** | 3/12 |

Every DiD under reversal is **negative**, and positive in only 2–4 seeds of 12.
The bar registered was `< +0.03`; the measurement is not near it.

### The control was stronger than registered, and the audit says so

The preregistration called `reversed` *displacement-matched*. The manipulation
audit, recorded per cell, shows it is **displacement-exceeding**:

| condition | relocated fraction | mean bin displacement |
|---|---:|---:|
| `bin-shuffled` | 0.9972 | 109.2 bins |
| `channel-shuffled` | 0.9969 | 105.3 bins |
| **`reversed`** | **0.9986** | **145.2 bins** |

Reversal moves spikes **a third further** than shuffling does, relocates a
larger fraction of them, and costs the attention read-out nothing above the rate
read-out. Per-channel counts are preserved on every manipulated cell in the wave
— checked, not assumed, by `apply_temporal`'s own gate.

That is the finding. **The cost is not in the displacement; it is in what the
displacement destroys.**

### H24-3 and H24-4 — the mechanism is not an artefact of `e400`

| point | budget | DiD(`bin-shuffled`) | positive | DiD(`reversed`) |
|---|---|---:|---:|---:|
| h128 / `published-2ms` / `d32L2` | `e100` | **+0.1183** | 12/12 | −0.0068 |
| h1024 / `published-2ms` / `d32L1` | `e100` | **+0.0633** | 12/12 | −0.0307 |

At a quarter of the budget the contrast is **as large as at `e400`** (+0.1183
against +0.1145; +0.0633 against +0.0675), positive in every seed pair, and the
order control is still flat. The manuscript's "all of it is `e400`" scope limit
is retired by measurement.

## 3. The new finding: synchrony, at one point and not the others

H24-2 was registered as a **question**, with no directional prediction, because
the campaign had never separated order from synchrony. `bin-shuffled` applies
one permutation to every channel — order dies, within-bin synchrony survives.
`channel-shuffled` permutes each channel independently — both die.

| point | DiD(`bin`) | DiD(`channel`) | difference | reading |
|---|---:|---:|---:|---|
| h128 / `published-2ms` / `d32L2` | +0.1145 | +0.1028 | −0.0117 | order alone |
| h1024 / `published-2ms` / `d32L1` | +0.0675 | +0.0495 | −0.0181 | order alone |
| **h128 / `fixed-t250` / `d32L4`** | +0.1119 | **+0.2157** | **+0.1038** | **synchrony also read** |

At the two anchor-contract points, destroying synchrony on top of order costs
nothing beyond order — inside the ±0.03 band, and slightly *negative* at both.
At the `fixed-t250` rung it **doubles the cost**.

The two points that answered "order alone" are `published-2ms`, where a bin is
2 ms. The point that did not is `fixed-t250`, where each utterance is forced
into 250 bins and a bin is therefore ~2.9 ms on the mean utterance and varies
per sample. This wave does not establish *why* they differ, and the honest
statement is the one the registration named in advance: **attention reads
cross-channel synchrony as well as order, at at least one operating point, and
the mechanism is broader than the paper currently claims.** One point is not a
resolution axis, and the paper must not present it as one.

This is a new result and it is not a comfortable one. It says the single-word
summary "order" is wrong somewhere in the design space, and it says so from a
control the paper did not previously run.

## 4. Provenance

- **One binary.** Every cell of this wave and every wave-22 arm it pairs against
  was produced by `3afd4434431a75a26cc9d5fa46831341fc2f1dd0ef08dc308e18ca139b576364`.
  All ten gate reports in the campaign bucket name it, checked by
  `collect.py --require-binary-sha256`, which exits non-zero rather than
  printing a warning. The launch itself was gated: `launch.py` reads the
  bucket's pin and refuses to provision on a mismatch, because the default
  bucket is the **v1** one at `22d97c51…` and a cell record carries no binary at
  all.
- **The `e400` intact and `bin-shuffled` arms are reused, not re-run**, per
  preregistration §5. The analyser fetches every arm from a **named** wave and
  prints which wave supplied it, so a silent fallback cannot look like a clean
  contrast.
- **The reuse reproduces.** `check_reproduction.py` finds all 456 duplicated
  cells across the corpus byte-identical over 687,528 compared values —
  including wave 24's `e100` rate cells against wave 23's, an independent
  cross-wave reproduction at h1024.
- **Corpus.** 1,437 → 1,773 cells, 1,427 → 1,763 valid, both **+336**. The ten
  invalid cells are the same ten `w13rec` cells as before, enumerated rather
  than inferred from the count. An addition, not a re-scoring.
- **Cost.** 587 slot-hours, which is what the estimate said to four significant
  figures before the wave ran. 14:52 to 21:22 UTC on a fleet scaled from four to
  six `c7g.16xlarge` mid-run; ≈$26 of spot.

## 5. What this does not establish

- **Three of 21 points at `e400`, two at `e100`.** H24-1 is a claim about the
  points measured, not a licence for "the mechanism is order everywhere."
- **Two budgets, not a curve.**
- **Reversal cannot separate order from direction.** A read-out using interval
  structure while ignoring the arrow of time would satisfy H24-1 without reading
  order in the strong sense. This supports **"attention reads temporal
  structure"**; it does not support "reads the direction of time."
- **The synchrony result is n = 1 point.** It is reported as a finding and not
  as an axis.
- **`rec+alif` is untouched.**
