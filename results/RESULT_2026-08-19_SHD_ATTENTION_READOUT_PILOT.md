# A time-axis attention read-out lifts the h128 arm by 0.1702 at a matched budget, and half of that is temporal order — preregistered pilot (rust arm, n=15 cells)

> ## SUPERSEDED 2026-08-19 — the headline does not replicate at convergence
>
> The converged run this pilot called for has landed:
> `RESULT_2026-08-19_W1_ATTENTION_AT_CONVERGENCE.md`, 60 cells, n=12 seeds, e400.
>
> **The +0.1702 below is +0.0421 at convergence — a quarter of it — and the
> registered primary hypothesis W1-1 is NOT SUPPORTED.** This pilot measured a
> budget effect, as its own §5.1 warned it might. **Do not cite +0.1702 as an
> architecture result.**
>
> Two findings did survive and strengthened. The mechanism result is emphatic at
> n=12: on bin-shuffled data the attention arm is *worse than its control* by
> −0.0492 in all twelve seeds, so its order-derived component is 7x what the base
> architecture can express. The capacity control also holds.
>
> §5.1 of this document is the part that aged well, and it is the reason the
> converged run existed to contradict the headline.

**Protocol:** `PREREG_2026-08-19_SHD_ATTENTION_READOUT.md`, registered before any
comparison cell was run.
**Artifacts:** `results/shd_attention_pilot_v1/` — 15 cells, 20 initialisation
files, `manifest.json` (binary + dataset + per-file sha256), `analyse.py`.
**Binary:** `637aaff4c88c9e76…`

```
claim_axis: architecture
object_under_test: Whether the instrument's near-blindness to temporal order is
  a property of its 46 ms membrane horizon and its permutation-invariant
  read-out, and whether a time-axis attention read-out recovers the missing
  structure.
may_claim: A budget-matched paired contrast at published-2ms / adjacent-sum-5 /
  h128 / e20 / full splits, over three seeds, between `ff+fixed`,
  `ff+fixed+attn`, and a parameter-matched `ff+fixed` at h192 — plus the same
  contrast run on bin-shuffled data. That the effect is not a capacity artefact
  and is roughly half temporal-order-derived. That the arm is numerically stable.
must_not_claim: Anything about the converged 0.7378 ceiling or the registered
  0.80 gate — this is a 20-epoch pilot and the attention arms are visibly
  UNDERTRAINED (§5). Any other contract, geometry, width or budget. That the
  four recorded base arms moved: they are bit-identical and Gate F passes 6/6.
  Cross-backend agreement: there is no python mirror, so Gate E is silent about
  these cells rather than satisfied by them.
```

---

## 1. Result

| arm | hidden | temporal | mean acc | sd | seeds |
|---|---:|---|---:|---:|---:|
| **A** `ff+fixed` | 128 | intact | **0.5807** | 0.0080 | 3 |
| **B** `ff+fixed+attn` | 128 | intact | **0.7509** | 0.0263 | 3 |
| **C** `ff+fixed` (capacity control) | 192 | intact | **0.5982** | 0.0038 | 3 |
| **D** `ff+fixed` | 128 | bin-shuffled | **0.5627** | 0.0049 | 3 |
| **E** `ff+fixed+attn` | 128 | bin-shuffled | **0.6461** | 0.0071 | 3 |

| ID | test | measured | threshold | verdict |
|---|---|---:|---|---|
| **H-A1** | mean(B) − mean(A) | **+0.1702**, per-seed +0.1378 / +0.2058 / +0.1670 | ≥ 0.05, all seeds positive | **SUPPORTED** |
| **H-A2** | mean(B) − mean(C) | **+0.1527**, per-seed +0.1330 / +0.1776 / +0.1475 | ≥ 0.02, ≥ 2/3 positive | **NOT A CAPACITY ARTEFACT** |
| **H-A3** | gain(intact) − gain(bin-shuffled) | **+0.0869** (0.1702 − 0.0833) | ≥ 0.02 | **MEMORY, not just capacity** |
| **H-A4** | attention-arm stability | 0 non-finite events, peak gradient norm **1.03e+01** | 0 and < 1e3 | **STABLE** |

Every validity gate in §5 of the prereg passes on all 15 cells: 20/20 classes
predicted everywhere, majority prediction 0.071–0.118 against a 0.30 bound,
zero non-finite events, and both shuffled arms report `counts_preserved` with a
relocated fraction above the bound.

## 2. H-A2 is the one that had to come out this way

Attention adds 8,832 parameters to a 20,500-parameter network (+43%). Arm C is
the same forward model widened to h192 — **30,740 parameters, more than the
treatment's 29,332** — and it buys **+0.0175** over arm A. The attention arm
buys **+0.1702** with fewer parameters, on the identical forward model, from
bit-identical base weights and bit-identical epoch orders.

That ratio is not close. Parameter count explains roughly a tenth of the effect,
which is what the recorded width axis already predicted: at the converged budget
the whole h128 → h1024 span buys +0.0346 and the final doubling +0.000883.

## 3. H-A3 is the one that says "memory"

The base arm's entire order sensitivity is **A − D = 0.0180**. That independently
reproduces the recorded converged figure of **0.0189**
(`RESULT_2026-08-03_SHD_TEMPORAL_INFORMATION_H1.md`) at a completely different
budget, width and epoch count — a corroboration this pilot did not set out to
produce and did not tune for.

Against that, attention's **order-derived** component is **0.0869**, which is
**4.8x the total amount of order information the base architecture can express.**

The remaining **0.0833** survives bin-shuffling. That is real and is not memory:
bin-shuffling preserves per-channel counts *and* within-bin synchrony, so a
pairwise, set-level read-out can still exploit which bins co-activate which
channels — structure the mean-rate read-out is blind to regardless of order. The
honest decomposition of the +0.1702 is therefore **roughly half temporal order,
half order-free pairwise structure**, and neither half is capacity.

## 4. H-A4 — the failure mode that blocks `rec+alif` did not appear

| arm | peak gradient norm | non-finite events | usable cells |
|---|---:|---:|---:|
| `rec+alif` h512 (recorded) | 3.08e10 – 3.93e33 | aborts | **0 / 3** |
| `ff+alif` healthy (recorded) | ~0.15 | 0 | — |
| `ff+fixed` here | 0.39 – 0.63 | 0 | 6 / 6 |
| **`ff+fixed+attn` here** | **5.79 – 1.03e+01** | **0** | **6 / 6** |

Attention runs ~15x hotter than the base arm and **32 orders of magnitude below**
the recurrent arm's worst cell (3.93e33 against 1.03e1). This is the predicted consequence of a gradient path that is
constant-depth in `T` rather than a product over `T` sequential steps, and it is
the practical reason this axis is measurable when `rec+alif` is not.

## 5. What this pilot does not establish — read before citing

1. **It is not a ceiling measurement, and arm B is undertrained.** Its
   `tail_loss_improvement` is **−0.149 to −0.157** against **−0.011** for the
   control: the attention arms were still learning fast when the 20-epoch budget
   ran out. By the registered convergence rule that is the UNDERTRAINED branch.
   Arm B's 0.7509 is therefore **not** comparable to the converged 0.7378, even
   though the numbers invite it. The width axis already taught that short-budget
   behaviour does not transfer (`SHD_BPTT_CEILING_NEGATIVE_RESULT.md` erratum
   E4), and the direction of the undertraining here is *toward* the treatment.
   **Do not quote 0.7509 against the 0.80 gate or against 0.7378.**
2. **Single backend.** No python mirror of the attention axis exists, so Gate E
   cannot cover these cells. A single-backend result is what this is.
3. **One point in the attention design space.** `d_model = 32`, one block,
   non-causal, normalised sinusoidal position, spikes as the input. Depth, width,
   causality and absolute-vs-normalised position are all untested.
4. **h128 only, `published-2ms` / `adjacent-sum-5` only.**

## 6. Registered next step

Per §6 of the prereg, H-A1 and H-A3 both being met makes the next step a
**separately registered converged-budget run** at h128/e400 against the recorded
0.7032 reference — not more cells appended here. The stopping rule was three
seeds, the verdict was computed once, and it is reported above whichever way it
fell.

The second obligation is the python mirror in `scripts/shd_calibration/arms.py`,
without which no attention cell can ever clear Gate E.

## 7. What was verified about the instrument itself

- **Gate F: 6/6** recorded rust cells reproduce **bit-identically** through the
  modified binary (`results/shd_instrument_v4/gate-f-rust/report.json`).
- The four base arms' forward and backward bit-pins are **unchanged** from their
  2026-08-03 constants; the attention arms carry their own 7-entry pins, whose
  membrane and spike hashes are **identical** to the base arms' — the read-out
  provably cannot perturb the spiking forward.
- Every attention parameter and the spike gradient `ds_attn` are
  **finite-difference checked**. Everything downstream of the spike threshold is
  smooth, so the reason `w_in` and `w_rec` are uncheckable does not apply, and
  the check is exact rather than indicative.
- `SHDWGT1` and `SHDWGT2` weight files are byte-unchanged; attention arms write a
  new `SHDWGT3` container.
- GC1–GC7 pass; `cargo fmt --all -- --check` clean; `binn-learn` clippy clean at
  `-D warnings`.
