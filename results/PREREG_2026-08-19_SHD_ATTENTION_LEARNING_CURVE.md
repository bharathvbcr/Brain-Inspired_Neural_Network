# Preregistration — same-machine learning curve for the attention arm (wave 6)

**Registered:** 2026-08-19, **before any e20 or e50 cell existed** — zero in the
plan, zero in `results/`, zero in `claims/`, checked and recorded at registration
time. The e100, e400 and e800 rungs already exist as waves 2, 1 and 5.
**Adds to:** `PREREG_2026-08-19_SHD_ATTENTION_CAMPAIGN.md` as wave 6. Changes no
existing hypothesis, threshold, arm or stopping rule.

---

## 1. Why — a comparison I am currently not licensed to make

The pilot measured `ff+fixed+attn` at **0.7509** (e20, h128) and the campaign
measures it at **0.7489** (e400, h128), which invites the reading that the
attention arm gains nothing from twenty times the budget while the control climbs
from 0.5807 to 0.7059 — i.e. that the pilot's +0.1702 was mostly *faster
convergence*, not *higher accuracy*.

**That comparison crosses machines.** The pilot ran on macOS/aarch64; the
campaign runs on Linux/aarch64, and
`MEASUREMENT_2026-08-19_CROSS_MACHINE_BIT_EXACTNESS.md` establishes those differ
by ~0.005 in accuracy and explicitly withholds licence for absolute comparison
between them. The observation is indicative and must not be reported as a result.

A learning curve measured **entirely on one machine** makes it a measurement
rather than an inference. That is the whole purpose of this wave.

## 2. Registered schedule

48 cells at the anchor (`published-2ms` / `adjacent-sum-5`), h128, 12 seeds:

| arm | epochs added here |
|---|---|
| `ff+fixed+attn` (d32, L1) | 20, 50 |
| `ff+fixed` | 20, 50 |

Together with the existing rungs the same-machine ladder is
**e20 · e50 · e100 · e400 · e800** for both arms, all seeds shared across rungs
and arms. Wave 2 supplies e100, wave 1 e400, wave 5 e800.

## 3. Hypotheses

| ID | statement | threshold |
|---|---|---|
| **W6-1** (primary) | The attention arm is sample-efficient rather than more accurate | `attn(e20) ≥ 0.95 × attn(e400)` — it reaches 95% of its converged accuracy within 20 epochs |
| **W6-2** | The control is not | `ff+fixed(e20) < 0.90 × ff+fixed(e400)` |
| **W6-3** | The advantage is a budget effect, not a ceiling effect | `[attn − ff+fixed]` at e20 is **at least 0.05 larger** than the same contrast at e400 |

**W6-1 and W6-3 together are the claim worth publishing if they hold**, and it is
a *different* claim from the one the pilot appeared to support: not "attention
raises the ceiling" but "attention reaches the same ceiling far sooner". If they
hold, the paper's contribution is sample efficiency on a spiking substrate, and
the ceiling language must be dropped from it entirely.

If **W6-3 fails** — the contrast is about as large at e400 as at e20 — then the
gain is not a budget effect and the wave-1 verdict stands on its own terms.

## 4. What must not be claimed

- That any of this bears on the recorded macOS ceiling of 0.7378. It does not,
  and cannot, for the reason this wave exists.
- That e20 is "where attention converges". The ladder's lowest rung is e20; a
  faster arm might converge sooner still, and this wave cannot see below its own
  floor.
- That sample efficiency implies anything about local learning. These are BPTT
  reference arms.

## 5. Stopping rule

**Twelve seeds, two new rungs, verdicts computed once from the assembled ladder.**
No rung below e20 and none between the registered points is authorised by this
document. If the curve is ambiguous, that is the finding.
