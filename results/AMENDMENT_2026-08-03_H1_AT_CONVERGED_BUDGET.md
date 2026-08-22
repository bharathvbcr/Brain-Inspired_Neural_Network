# Amendment: repeat H1 at the converged budget, and test the remaining geometry

> ## OUTCOME: both extensions ran clean, 27/27 cells, no failures.
> ## A: H1 is NOT SUPPORTED at e400 — the same verdict as e100. Outcome 4 of 4.
> ## B: `channels-700` is 0.0283 **worse**; the ceiling figure is unchanged.
> ## See §4.

**Registered:** 2026-08-03, before any cell of either extension ran.
**Amends:** `PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION.md` §3 (budget), and
extends `RESULT_2026-08-03_BUDGET_CONVERGENCE_CEILING_RESTORED.md` (geometry).

---

## 1. Extension A — H1 at e400

### Why

`RESULT_2026-08-03_SHD_TEMPORAL_INFORMATION_H1.md` §6.1 names this as the most
likely reviewer objection, and it is now quantified rather than vague: **e100 is
undertrained by 0.021 accuracy** against the converged value (0.7164 → 0.7378).
H1's contrast is internally valid — every condition shares the budget — but
whether the order effect *survives, grows or vanishes* at convergence is
untested.

The e400 budget is SUFFICIENT by the amended convergence rule (final doubling
buys 0.000294).

### Design

Identical to the settled H1 in every respect except budget: `ff+fixed`,
`published-2ms / adjacent-sum-5 / h512`, **e400**, 4 conditions × **6 seeds** =
**24 cells**, same `--temporal-seed` lineage.

Six seeds, not three, deliberately. The 3-seed run of this campaign returned
SUPPORTED and the 6-seed run returned NOT SUPPORTED on the CI criterion; n=3 is
known to be too coarse to resolve an effect of this size, and running it again
would repeat a mistake already documented.

### Registered thresholds — unchanged

`|intact − bin-shuffled| ≤ 0.02` **and** overlapping 95% CIs, exactly as in
prereg §4. **No threshold moves.** H3's bound stays at 0.02 and its conditional
status stays as registered.

### Stopping rule

**24 cells, one verdict, reported whichever way it falls. No seed extension.**
If any cell aborts, the affected condition is reported as incomplete and no
verdict is issued for the extension — the same rule that fired on H2, for the
same reason.

### The four outcomes, named in advance

| e400 result | reading |
|---|---|
| effect **grows** past 0.02 | order matters more with training; the e100 result understated it |
| effect **similar** (~0.019, CIs disjoint) | H1's NOT SUPPORTED verdict is budget-robust — the strongest available outcome |
| effect **shrinks**, CIs overlap | H1 becomes SUPPORTED at convergence and the e100 verdict was a budget artefact |
| effect shrinks but CIs stay disjoint | order effect is real and small at every budget |

All four are publishable. Naming them now removes the temptation to find the
third one more interesting than the second after the fact.

## 2. Extension B — geometry at convergence

### Why

With budget and width both closed
(`RESULT_2026-08-03_BUDGET_CONVERGENCE_CEILING_RESTORED.md`), the ceiling claim's
only remaining scope qualifier is **contract and geometry**. The campaign has two
geometries and only `adjacent-sum-5` has been run to convergence.

### Design

`ff+fixed`, `published-2ms`, **`channels-700`**, h512, **e400**, 3 seeds — the
same points at which `adjacent-sum-5` gives 0.736896 ± 0.002087.

`channels-700` presents 700 input channels rather than 140, so it is the
higher-resolution frequency geometry: if anything is being discarded by the
`adjacent-sum-5` binning, this is where it would show.

### Pre-specified reading

- **Within noise of 0.7369** → geometry does not move the ceiling, and the
  qualifier can be dropped from "one geometry" to "both geometries tested".
- **Materially higher** → `adjacent-sum-5` was discarding usable information and
  the ceiling figure must be restated at the better geometry.
- **Materially lower** → confirms `adjacent-sum-5` as the stronger framing;
  ceiling unchanged.

No threshold is registered for "materially", because this is descriptive scope
work rather than a hypothesis test, and pretending otherwise would be false
precision. It will be reported as a measured difference with CIs.

## 3. What neither extension does

- Neither touches H2, which remains NOT RUN
  (`AMENDMENT_2026-08-03_H2_AT_H256.md` §7).
- Neither revisits H1 at e100, which is settled and stands as reported. If e400
  disagrees, **both are reported**, with the budget stated for each — the e100
  result is not retracted for being at a shorter budget than a later run.


## 4. OUTCOME

### 4A. H1 at e400 — NOT SUPPORTED, and the e100 verdict is budget-robust

24 cells, 6 seeds, zero aborts, `non_finite_events` = 0.

| condition | mean | sd | 95% CI (t=2.571) |
|---|---:|---:|---|
| `intact` | 0.7374 | 0.0021 | [0.7352, 0.7396] |
| `bin-shuffled` | 0.7247 | 0.0028 | [0.7217, 0.7276] |
| `channel-shuffled` | 0.5911 | 0.0026 | [0.5884, 0.5938] |
| `reversed` | 0.7283 | 0.0053 | [0.7228, 0.7338] |

| criterion | value | verdict |
|---|---|---|
| \|intact − bin-shuffled\| ≤ 0.02 | **0.012736** | passes |
| 95% CIs overlap | disjoint | **fails** |

**H1 NOT SUPPORTED at e400** — the same verdict as e100, reached the same way:
the effect is comfortably inside the practical-negligibility bound and still
statistically resolvable.

This is **outcome 4 of the four named in §1**: *"effect shrinks but CIs stay
disjoint → order effect is real and small at every budget."* Naming the outcomes
in advance is what makes that statement worth anything.

**§6.1 of the H1 result is discharged.** The e100 budget caveat — the most
likely reviewer objection — is answered: the verdict does not depend on the
budget.

### 4A.1 What changed with training, and it is not nothing

| quantity | e100 | e400 | change |
|---|---:|---:|---:|
| order effect | 0.018919 | **0.012736** | **−0.006183** |
| synchrony increment | 0.124779 | **0.133613** | **+0.008834** |
| reversal cost | 0.003165 | 0.009128 | +0.005963 |
| **synchrony ÷ order** | **6.6×** | **10.5×** | — |

**Training moves the solution further toward synchrony and away from order.** The
order effect shrinks by a third while the synchrony increment grows, so the ratio
goes from 6.6× to 10.5×. The headline decomposition is not merely preserved at
convergence — it sharpens.

This is descriptive and was not registered. It is a hypothesis for future work,
not a result, and the direction should be checked at a third budget before
anyone leans on it.

### 4B. Geometry — `channels-700` is worse, and the ceiling is unchanged

3 cells at `published-2ms / channels-700 / h512 / e400`, against the matching
`adjacent-sum-5` cells:

| geometry | n | mean | sd | 95% CI |
|---|---:|---:|---:|---|
| `adjacent-sum-5` | 3 | **0.736896** | 0.002087 | [0.7317, 0.7421] |
| `channels-700` | 3 | 0.708628 | 0.004104 | [0.6984, 0.7188] |

**`channels-700` is 0.0283 worse, CIs disjoint.** Per the reading pre-specified
in §2, this is the third case: *"materially lower → confirms `adjacent-sum-5` as
the stronger framing; ceiling unchanged."*

The ceiling figure stands at **0.7378** (h1024/e400, `adjacent-sum-5`), shortfall
**0.0622**. Presenting all 700 channels separately does not recover information
that the `adjacent-sum-5` binning discards — it loses accuracy, which is the
opposite of the concern this extension was run to rule out.

**Both geometries are now measured at convergence**, so the ceiling claim's
scope qualifier narrows from "one contract, one geometry" to "one contract".

> **Correction, 2026-08-22 — the sentence above is withdrawn.** The numbers in
> this section all reproduce exactly from the cells; this conclusion does not.
> `channels-700` is measured at **one budget and one width**, so both of its
> axes are still open, and `PREREG_2026-08-04_SHD_GEOMETRY_AT_CONVERGENCE.md`
> section 6 forbids the word "convergence" for this geometry until its Stage 2
> (6 cells at h512/e800 and h1024/e400) has run. **Zero of those 6 exist.**
>
> The claim also fails on the preregistration's own arithmetic: two doublings at
> the maximum increment recorded here (+0.018551 each) give
> `0.708628 + 0.037102 = 0.745730`, which is **above** `adjacent-sum-5`'s
> 0.736896. Nothing measured excludes `channels-700` converging higher.
>
> The scope qualifier stays at "one contract, one geometry". This does not
> affect G1, whose CI upper bound (0.718824) is far below its 0.80 threshold.
> See `RESULT_2026-08-22_GEOMETRY_AT_CONVERGENCE.md`, which also records that
> these same three cells **falsify G3** — registered the day after this
> amendment was written, against this very data, and never checked against it.
