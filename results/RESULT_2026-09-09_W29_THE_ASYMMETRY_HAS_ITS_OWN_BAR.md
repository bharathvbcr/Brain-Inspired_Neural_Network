# Wave 29 — the asymmetry has its own bar

**Registered:** [`PREREG_2026-09-07_W29_THE_ASYMMETRY_HAS_ITS_OWN_BAR.md`](PREREG_2026-09-07_W29_THE_ASYMMETRY_HAS_ITS_OWN_BAR.md),
committed 17:08 on 2026-09-07; the first cell finished at 22:09.
**Amended:** [`AMENDMENT_2026-09-07_WAVE_29_RUNS_ON_THE_LOCAL_PLATFORM.md`](AMENDMENT_2026-09-07_WAVE_29_RUNS_ON_THE_LOCAL_PLATFORM.md),
committed 21:58 — eleven minutes before that first cell, which is what makes it
an amendment rather than an explanation. §4 of this document is about what it
costs.
**Analyser:** `scripts/aws/analyse_wave29.py`, committed with the registration
and **not modified since** — the authority on every verdict below.
**Corpus:** `results/shd_attention_wave29_local/`, **48/48 cells, 0 failures,
0 INVALID**. Every one of the 48 registered ids is present and nothing
unplanned is beside it; the stored plan is byte-identical to what
`plan_cells.py --waves w29` emits today.
**Binary:** `fec6c40420953a811cfec284cce31929c565cd7f65c7f18acc7854bf937f86cd`,
built from source `a6acd8d` and verified against the amendment's pin at landing.
**Platform:** `aarch64-apple-darwin`, **not** the glibc fleet. This is the first
corpus in the campaign that is not fleet-produced.

---

## 1. What wave 28 left broken, and what this wave was for

Wave 28 registered `|DiD| < 0.03` for H28-2 and reported **NOT MET** at p90. The
band was two-sided and the question was not. With

```
DiD = (attn_intact − attn_X) − (rate_intact − rate_X)
```

a **positive** DiD is the read-out losing more than the substrate, which is what
H28-2 was written to detect. The measured values were **negative** — −0.0132 at
p70, −0.0382 at p90 — so the clause fired on the outcome nobody had named, and
its registered reading described the opposite of its own sign.

Rule §9.3 forbids re-reading wave 28's cells against a replacement bar. So the
bar was split into two one-sided clauses and pointed at a **new seed block**:
`5290001`–`5290012`, disjoint from every earlier wave. The analyser refuses wave
28's seeds by value, so the replacement cannot be evaluated on the cells that
motivated it.

## 2. The instrument can feel its own hand

**Sensitivity gate: SENSITIVE.** Wave 28's H28-1 bar, restated on the new block.

| | rate arm |
|---|---:|
| intact | 0.7068 (sd 0.0098) |
| p90 | 0.5956 (sd 0.0040) |
| **drop** | **+0.1112**, above the 0.03 bar in **12/12** |

The fleet measured +0.1080 on its own block. A null measured with this
instrument is a null, not a manipulation the substrate could not feel.

## 3. Both clauses MET

| | rate | attention | gain |
|---|---:|---:|---:|
| intact | 0.7068 | 0.8261 | 0.1193 |
| **p90** | 0.5956 (−0.1112) | 0.7615 (−0.0646) | **0.1659** |
| | | **DiD** | **−0.0467** |

**H29-1: MET.** The read-out is not hurt by count destruction.
One-sided; refuted only by DiD > +0.03. Satisfied in **12/12** seeds. Destroying
90% of spikes does not cost the read-out its advantage.

**H29-2: MET.** The advantage grows.
One-sided; requires DiD < −0.03. Satisfied in **11/12** seeds, against a
registered floor of 9/12. The one seed that misses is `5290002` at −0.0172 — it
misses the bar, not the sign; all twelve DiDs are negative, spanning −0.0795 to
−0.0172.

**This is the wave's substantive result.** The direction wave 28 measured
without a bar behind it now has one, on a seed block chosen before the bar was
written. The asymmetry is a result rather than an observation.

**H29-3 — monotone in rate: no verdict.** Registered as descriptive, and this
block carries no p70 cell, so no direction is reported. The analyser prints the
absence rather than interpolating one.

## 4. What the platform costs, stated first rather than last

**Registered platform gate: INSIDE THE BAR.** The wave's own rate-`intact` mean
is **0.7068** against the fleet's **0.7062** — a delta of **+0.0006**, fifty
times inside the 0.03 bar every clause is tested against.

That is a diagnostic, not a clause. It cannot void a cell or move a verdict, and
it did not.

**What it does not license.** The gate says this anchor moved less than the bar;
it does not say the platforms agree. Three constraints stand:

- **No wave-29 accuracy may be tabulated beside a fleet accuracy** without
  naming the platform of each. The comparable object is the DiD.
- **Every clause is a within-block paired contrast.** `did()` takes only seeds
  present in all four arms, and all four arms came from one binary on one box.
  No macOS number is subtracted from a fleet number anywhere in this wave.
- **A wave-29 verdict is a verdict on this platform.** Here the two platforms
  agree in sign and nearly in size — fleet DiD −0.0382, local −0.0467 — so the
  disagreement clause the amendment prepared for does not fire. Had they
  disagreed, "the platforms disagree" would have had to be carried alongside
  "the seed block was the artefact", and neither could have been chosen for
  being the more interesting.

## 5. Coverage

48 cells: `{intact, spike-dropout-p90} × {ff+fixed, ff+fixed+attn d32/L4} × 12
seeds`. 0 voided, 0 failed, 0 recovered — no cell in this wave was produced
twice, and no recovery run stands behind any number.

The intact arms were **re-run rather than reused** from `w26pos`. That was
registered before the platform question arose, and it is the reason this wave
survives the amendment: a design that had reused them would now be subtracting
fleet controls from macOS arms.

One disclosure the amendment already carries: a single `ff+fixed` intact cell at
seed `5290001` was run on this host *before* the amendment was committed, while
timing the box. It was **deleted** and the seed re-run as part of the wave, so
every cell the analyser reads was produced under a registration that already
existed. The computation is deterministic and the re-run reproduced it.

## 6. Cost

**223 cell-hours** — 219.9 in the attention arm (median 8.94 h/cell) and 3.3 in
the rate arm (median 0.13 h). 46.6 h wall, 2026-09-07 22:09 to 2026-09-09 20:43,
five concurrent cells at 3 threads.

The rate arm ran **first and alone**, deliberately: it is 1/70th the cost of the
attention arm, and both gates read only it. A platform failure would have cost
one hour instead of two days. Both gates passed on it before a single attention
cell was scheduled.

## 7. What this wave does not answer

- **Nothing about wave 28.** H28-2 stays NOT MET. This is a new block, and rule
  §9.3 governs: a wrong bar is reported as a wrong bar and the replacement
  governs the next wave's cells.
- **Not the shape of the effect in rate.** H29-3 has no verdict and this block
  has no p70 cell. Whether the advantage grows monotonically with dropout rate
  is unmeasured, not undecided.
- **Not why.** Both clauses are about what dropout costs each arm. The claim
  they support is that the read-out's advantage survives — and grows under —
  destruction of spike counts, which is a claim about **order**, not a mechanism
  for it.
- **Not a fleet result.** Everything here is `aarch64-apple-darwin`.
