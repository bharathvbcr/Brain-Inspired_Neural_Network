# Measurement — wave 20's plain recurrent arm is complete, and H20-2 is now decidable in advance

**This is a measurement, not a verdict.** Wave 20 is unfinished. No hypothesis is
evaluated here, no number from the frozen analyser is quoted, and nothing in
[`PREREG_2026-08-27_THE_RECURRENT_CLAIM_AT_THIRTY_TWO_SEEDS.md`](PREREG_2026-08-27_THE_RECURRENT_CLAIM_AT_THIRTY_TWO_SEEDS.md)
is amended. The prereg's bars stand exactly as registered.

**Why it is written now.** At the time of writing, `rec+alif+attn` has **0 of 20
completed** — 15 in flight, 5 unclaimed. The plain arm it must pair against is
finished. That makes H20-2's outcome a function of one number nobody has yet
seen, and the honest moment to write down the boundary is before the number
exists rather than after. Recorded post hoc, everything below would be a story
fitted to whatever landed.

---

## 1. What is measured

Wave `w20rec`, h128 / e400 / `published-2ms` / `adjacent-sum-5` / d32L4,
every arm at surrogate scale 0.4. Counts read from the campaign bucket.

| arm | planned | completed | diverged | in flight | unclaimed |
|---|---:|---:|---:|---:|---:|
| `ff+fixed` | 20 | **20** | **0** | 0 | 0 |
| `rec+alif` | 20 | **16** | **4** | 0 | 0 |
| `ff+fixed+attn` | 20 | 0 | 0 | 0 | 20 |
| `rec+alif+attn` | 20 | 0 | 0 | 15 | 5 |

The four losses are `shd-instrument: non-finite training value`, seeds
5170013, 5170017, 5170020 and 5170029, at optimiser steps 4306, 5028, 7271 and
9051 of 12,800 — **34%, 39%, 57% and 71% of the way through training**, not at
initialisation. Their claims are consumed; per the stopping rule they are not
re-seeded.

**These are the instrument diverging, not the tooling failing.** Each log opens
with `run_cell: … is absent from cells.json but present in the published queue;
using the published entry` and then runs to a mid-training divergence. That line
is the boot-copy repair working: the cells reached the trainer, which is what
distinguishes these four from the eighty that died instantly on 2026-08-27.

**Divergence is a property of the substrate, not of the wave.** `ff+fixed`
completed 20 of 20 at the same scale, on the same hosts, from the same binary.

## 2. The rate did not measurably rise

The archive completed 11 of 12 on this arm. Wave 20 completed 16 of 20.

| | diverged | rate |
|---|---:|---:|
| archived twelve | 1 / 12 | 0.083 |
| wave 20 | 4 / 20 | 0.200 |
| pooled | 5 / 32 | 0.156 |

**Fisher exact, two-sided: p = 0.626.** Twenty per cent against eight is not an
increase; it is what 12 and 20 seeds look like when drawn from one rate near
0.16. Nothing here licenses a claim that the arm got less stable, and §3.7's
limit 4 is not strengthened by it.

## 3. Where H20-2 now stands, stated before the deciding cells exist

H20-2 requires **≥ 24 usable `rec+alif` / `rec+alif+attn` pairs of 32**. The
archived ten are fixed and already counted, so **the new twenty seeds must yield
at least 14 pairs.** The plain arm has lost 4 of them permanently. Let *m* be the
number the attention arm loses.

| *m* | pairs if losses are disjoint | pairs if losses overlap maximally | outcome |
|---:|---:|---:|---|
| 0–2 | 14–16 | 16 | **H20-2 MET either way** |
| 3–6 | 10–13 | 14–16 | **MET only if the losses overlap** |
| ≥ 7 | ≤ 9 | ≤ 13 | **H20-2 NOT MET either way** |

Two things make the middle row the one to watch. The attention arm's own archive
lost 1 of 12, so *m* ≤ 2 is not out of reach. But where the archive's two arms
both lost a seed, **they lost different ones** — the prereg says so and the
corpus confirms it — which is the disjoint column, the unfavourable one.

**No prediction is registered here and no bar is moved.** H20-2's floor is 24
because the prereg set it at 24. This note only records that the floor is now
one arm away from being decided, and which side of it each value of *m* falls on.

## 4. What this does not say

- **Nothing about H20-1.** Its arithmetic is not computed here, and under the
  prereg it is not licensed at all unless H20-2 passes. `analyse_wave20.py`
  suppresses it in that case; `test_h20_2_suppresses_h20_1_at_ten_pairs` and
  `test_a_thin_recurrent_arm_suppresses_the_headline` exercise that path, and
  the file's sixteen tests pass.
- **Nothing about H20-3.** The gradient-norm correlation is measured over
  completing *pairs*, and there are none yet.
- **Nothing about which substrate wins.** §3.7's limit 2 stands untouched.
- **Nothing against the macOS record.** Cross-machine Gate F FAILs
  macOS-vs-Linux on every node of this campaign, by design.
