# Preregistration — the recurrent comparison at n=32, and whether survivorship is inflating it

**Registered:** 2026-08-27, **before any cell of wave 20 exists and before any
instance for it is launched.** Authorised by the maintainer, who asked for
additional hypotheses and larger samples and for the fleet not to sit idle.

**Protocol:** `shd-attention-campaign-v2`, wave `w20rec`.
**Binary:** the campaign's existing pinned binary
(`22d97c51ab0204702ce44661683ff8c759c29d7f3379e2f6606b048f4f032104`). One binary
across all twenty waves; reused controls stay bit-comparable.
**Analyser:** `scripts/aws/analyse_wave20.py`, frozen in the same commit as this
document and before the first cell lands.

---

## 1. Why this wave exists

[`PAPER_DRAFT.md`](PAPER_DRAFT.md) §3.7 states four limits on its recurrent
result and calls them load-bearing. Two of them are about the evidence being
thin rather than about the science:

> **3.** The recurrent arms are numerically extreme … and the comparison rests
> on **ten pairs, the registered minimum**. The two arms lost different seeds;
> one further loss on either would have made the comparison unreportable.
>
> **4.** Survivorship is reduced, not removed. … the surviving recurrent pairs
> are those that did not diverge, and divergence is not random.

A claim one cell-loss away from being unreportable is the most fragile thing in
the manuscript, and the paper says so itself. Twenty more seeds retires limit 3
outright and makes limit 4 measurable instead of acknowledged.

The claim at stake, at h128 / `published-2ms` / `adjacent-sum-5` / e400 / d32/L4,
every arm at surrogate scale 0.4:

| substrate | pairs | rate read-out | + attention | gain |
|---|---:|---:|---:|---:|
| `rec+alif` | 10 | 0.5262 | 0.7874 | **+0.2612** |
| `ff+fixed` | 12 | 0.7088 | 0.8289 | **+0.1201** |

Difference of gains **+0.1391** seed-paired over the ten, positive in 10 of 10.

## 2. The statistic that is already failing, stated before it is registered

Wave 18's preregistration recorded three candidate bars that died against cells
already in hand. This one records a bar that **the pilot already fails**, so
that its failure cannot later be presented as a surprise or its threshold as
post-hoc.

Over the ten surviving recurrent pairs, **Spearman ρ between the paired gain and
log₁₀ of the cell's peak gradient norm is −0.648.** Among the cells that
completed, the ones that came closest to diverging show the *smaller* gains. The
two cells that did not complete had higher norms still.

If that relationship is real, the +0.2612 is measured on a subsample selected
for the property that predicts a large gain, and §3.7's limit 4 is not a caveat
but a defect. At n=10 the correlation is not resolved — this is exactly the
sample size at which a rank correlation is least trustworthy — which is why the
question is put to thirty-two seeds rather than argued from ten.

**H20-3 below is registered against this pilot value**, predicting it is
small-sample noise. It is a real prediction with a named consequence in both
directions.

## 3. Design

Twenty new seeds on four arms at h128 / e400 / `published-2ms` /
`adjacent-sum-5`, every arm at surrogate scale **0.4** so substrate and scale
cannot be confounded — the constraint wave 14 established and this wave inherits
unchanged:

- `rec+alif` and `rec+alif+attn` (d32/L4) — the arms under test
- `ff+fixed` and `ff+fixed+attn` (d32/L4) — the reference substrate, extended to
  the same count so the difference of gains stays seed-paired at n=32 rather
  than pairing thirty-two against twelve

The archived twelve of each arm are reused: same pinned binary, deterministic
instrument, and `w1`/`w3wid` are 12/12 byte-identical where they overlap, which
`scripts/test_campaign_tooling.py` now asserts.

**80 new cells, ≈277 slot-hours, ≈17 h on a 16-slot fleet.** Calibrated from
measured `wall_secs` on the arms themselves (`rec+alif+attn` at scale 0.4 runs
6.75 h, nearly double the scale-1.0 attention arms) and **not** from
`estimate_cost.py`, which over-predicts by a median 3.08× and now prints that
ratio with every run.

## 4. Hypotheses

| ID | statement | criterion |
|---|---|---|
| **H20-1** | The recurrent substrate's larger gain survives a tripled sample | seed-paired difference of gains (`rec+alif` − `ff+fixed`) **≥ +0.03**, positive in **≥ 24/32** pairs |
| **H20-2** | The comparison is no longer one loss from unreportable | the `rec+alif` / `rec+alif+attn` comparison yields **≥ 24 usable pairs** of 32 |
| **H20-3** | Survivorship is not shaping the gain | Spearman ρ(paired gain, log₁₀ peak gradient norm) over completing recurrent pairs is **≥ −0.30** |
| **H20-4** | The advantage survives headroom normalisation | (recurrent gain / recurrent headroom) ÷ (feed-forward gain / feed-forward headroom) **> 1.0** |

H20-1 through H20-4 are independent. Any one passing rescues none of the others.

**H20-2 is not a formality.** `rec+alif` completed 11 of 12 at this scale and
`rec+alif+attn` 11 of 12, losing *different* seeds — that is how ten pairs came
from twelve. If the completion rate holds, 32 seeds give roughly 27 pairs. If
fewer than 24 land, the recurrent claim does not become reportable at a larger
sample and the honest reading is that this operating point cannot support it.

**H20-4 makes a post-hoc number registered.** §3.7's limit 1 already computes
the headroom-normalised ratio as **1.34×** and labels it "post-hoc, not
registered". Registering it here is what turns it into a result. Its bar is 1.0
— that the ordering survives normalisation at all — not 1.34, because
registering the pilot's own value as the bar would be fitting to it.

## 5. Named outcomes, in every direction

| observed | reading |
|---|---|
| H20-1 **MET**, H20-3 **MET** | §3.7 stands and limits 3 and 4 are retired. The recurrent result is the campaign's best-evidenced contrast. |
| H20-1 **MET**, H20-3 **NOT MET** | The gain is real *and* survivorship-shaped. §3.7 must report it as measured on a subsample selected for low gradient norm, and limit 4 is promoted from caveat to finding. This is the outcome the pilot points at. |
| H20-1 **NOT MET** | The tripled sample does not support the doubling. §3.7's table is withdrawn to n=12 with its provenance stated, and the recurrent claim leaves the paper's abstract. |
| H20-2 **NOT MET** | The operating point cannot support the claim at any sample size reachable here. Reported as such; H20-1's verdict is then **not** licensed regardless of its arithmetic, and the analyser suppresses it. |
| H20-4 **NOT MET** | The advantage is an artefact of the lower base. Limit 1 becomes the headline caveat on §3.7. |
| ρ **≥ +0.30** | The opposite correlation. Not predicted by anything and would need its own explanation before any of this is read. |

## 6. Stopping rule

Every cell runs. No arm is extended, truncated or re-seeded on the basis of what
it shows. **A comparison below 24 pairs carries no numbers at all** — not a
mean, not a direction.

Validity is `scripts/cell_validity.py` unmodified. **Gradient magnitude never
voids a cell**, here least of all: it is the covariate H20-3 is measured on, and
voiding on it would decide the survivorship question by construction.

## 7. What may not be claimed

- **Nothing against macOS-recorded numbers.** Cross-machine Gate F FAILs
  macOS-vs-Linux on every node of this campaign, by design.
- **No claim that the recurrent substrate wins.** §3.7's limit 2 stands
  untouched: `rec+alif+attn` reaches 0.7874 against `ff+fixed+attn`'s 0.8289,
  and this wave issues no verdict on that ordering.
- **No claim about surrogate scale 1.0.** Every arm here is at 0.4 and the
  completion rate at 1.0 was worse (8 of 12). That is a different wave.
- **No causal claim from H20-3 either way.** A correlation between gain and
  gradient norm among survivors bounds how much the selection could be doing;
  it does not establish that it is doing it.
