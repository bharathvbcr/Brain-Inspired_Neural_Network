# A6 — ceiling health of the two surviving matched arms

**Date:** 2026-08-19
**Harness:** `binn-lab/experiments/a6_ceiling_health.rs` (`a6-ceiling-health`, protocol v1)
**Where run:** `c7g.8xlarge` (32 vCPU, `aarch64-unknown-linux-gnu`), us-east-2
**Status:** exploratory sensitivity sweep — **no frozen manifest, no `--config-hash` claim,
no preregistered threshold touched**
**Answers:** `TODO_2026-08-07_OPEN_WORK.md` §2, `WEEK_PLAN_2026-08-19.md` A6

---

## The question

Two arms hold a PASS and both clear the gradient reference they are mathematically
bounded by. On the DFA schedule the broadcast-graded **control** clears it too:

| schedule | arm | control | gradient reference |
|---|---|---|---|
| `c1-dfa-c8c4fe0899908b84` | 0.9387 | 0.9863 | 0.8963 |
| `c1-rl-42eddc9c801308e9` | 0.9200 | — | 0.8887 |

A6 asked: **are those references undertrained?**

## Answer

**Yes — but so is everything else, and that is the finding that matters.**

Raising the reference's budget alone inverts the ordering. Raising *both* budgets does
not: the arm stays at or above the reference at every budget tested, and the whole
schedule saturates at 1.0000. The published references are not a *uniquely*
undertrained ceiling; they are one symptom of a schedule that stops at 80 epochs
while every rule on it is still climbing.

**The "arm exceeds ceiling" anomaly is therefore not explained by reference
undertraining.** It survives matched compute. That question stays open.

## Evidence 1 — reference budget only (arms held at 80 epochs)

Forward, frozen splits, seed lineage (n=20), and every arm held fixed; only
`MatchedGradient` sees the swept budget. 24 budget points per suite.

`c1-dfa` — arm 0.9387, control 0.9863:

| reference budget | reference mean | SE | vs arm |
|---|---|---|---|
| `e80/lr0.05` *(canonical)* | 0.9013 | 0.0298 | arm above |
| `e160/lr0.1` | 0.9288 | 0.0297 | arm above |
| `e320/lr0.05` | 0.9700 | 0.0189 | **reference above** |
| `e640/lr0.05` | 0.9975 | 0.0025 | **reference above** |
| `e2560/lr0.02–0.2` | 1.0000 | 0.0000 | **reference above** |

`c1-rl` — arm 0.9200:

| reference budget | reference mean | SE | vs arm |
|---|---|---|---|
| `e80/lr0.05` *(canonical)* | 0.9188 | 0.0263 | arm above |
| `e160/lr0.05` | 0.9488 | 0.0220 | **reference above** |
| `e320/lr0.02` | 0.9838 | 0.0138 | **reference above** |
| `e640/lr0.05` | 1.0000 | 0.0000 | **reference above** |

The reference crosses the DFA arm at **4×** the canonical epoch budget and the RL arm at
**2×**, and saturates at 1.0000 by 32×. Read alone, this says the references were
undertrained.

## Evidence 2 — matched compute (both sides at the same budget)

The obvious objection to Evidence 1 is that the reference was simply handed more
compute. `--arm-epochs` puts both sides on the same budget:

| budget | DFA arm | DFA control | DFA reference | ordering |
|---|---|---|---|---|
| 80 *(canonical)* | 0.9387 | 0.9863 | 0.9013 | arm above |
| 320 | **1.0000** | **1.0000** | 0.9700 | arm above |
| 1280 | **1.0000** | **1.0000** | 0.9975 | arm above |
| 2560 | **1.0000** | **1.0000** | **1.0000** | tie |

| budget | RL arm | RL graded | RL reference | ordering |
|---|---|---|---|---|
| 80 *(canonical)* | 0.9200 | 0.5250 | 0.9188 | arm above (by 0.0012) |
| 320 | **0.9863** | 0.5500 | 0.9812 | arm above |
| 1280 | **1.0000** | 0.5500 | 0.9750 | arm above |
| 2560 | **1.0000** | 0.5500 | **1.0000** | tie |

Three things follow, and only the first was anticipated:

1. **Everything at 80 epochs is undertrained.** The DFA arm goes 0.9387 → 1.0000, the
   control 0.9863 → 1.0000, the reference 0.9013 → 1.0000. The canonical schedule
   measures *rate of learning*, not *quality of solution*.
2. **The anomaly survives matched compute.** At 320 and 1280 epochs the arm is at
   1.0000 while the reference is at 0.9700 / 0.9975. A local rule beating SuperSpike
   BPTT on the same substrate at the same budget is still unexplained.
3. **The task saturates and stops discriminating.** By 2560 epochs the arm, the
   control, and the reference are all exactly 1.0000. `gap_closed` is undefined in
   spirit there — every rule is perfect, so the metric has no headroom left.

`MatchedRlGraded` is the exception: it sits at 0.5250 → 0.5500 against a 0.5 chance
baseline at every budget. It is not learning this task at all, at any compute.

## What this means for the paper

- **`gap_closed` at the canonical budget is not a ceiling-normalised quantity.** It
  divides by a reference that is still climbing. With reference = 1.0000 the DFA arm's
  gap-closed would be `(0.9387 − 0.5)/(1.0 − 0.5) = 0.877`, not
  `(0.9387 − 0.5)/(0.8963 − 0.5) = 1.107`. The `> 1` values are an artifact of the
  denominator, and clamping them (as `runner.rs` does) hides the cause rather than
  fixing it.
- **Do not report this as "the reference was undertrained" full stop.** That was the
  single-sided reading and it does not survive Evidence 2. The defensible statement is:
  *the 80-epoch schedule undertrains every rule on it, and the ordering between local
  rules and BPTT at that budget is a statement about learning speed.*
- **The coincidence task cannot support a ceiling comparison at high budget.** Any
  future matched-architecture claim needs a task with headroom at convergence, not one
  where all arms reach 1.0000.
- **A6's stated acceptance is met** — a table of reference ceiling vs training budget,
  and an explicit verdict — but the verdict is not the one the TODO anticipated.

## Caveats, stated rather than buried

- **Platform.** Run on `aarch64-unknown-linux-gnu`. The gradient reference is not
  bit-reproducible across platforms: the canonical row reads 0.9013 here vs the
  published 0.8963 on macOS (DFA) and 0.9188 vs 0.8887 (RL). See
  `FINDING_2026-08-19_LIBM_PORTABILITY_OF_REPLAY.md`. **Every arm value in this
  document reproduced macOS exactly**, and the effects above (0.90 → 1.00) are an order
  of magnitude larger than the 0.005–0.030 drift, so the conclusions are robust to it.
  The absolute reference numbers here are not directly comparable to the published ones.
- **Bounded sweep.** Budgets stop at 2560 epochs and four learning rates
  (0.02/0.05/0.1/0.2). Saturation at 1.0000 is observed, not proven to be the limit.
- **n=20 seeds**, the canonical lineage. SEs are carried in Evidence 1; the matched-compute
  table reports means only.
- **Not a canonical run.** Exploratory, off-machine, no manifest. Nothing here may be
  cited as a `--config-hash` result.

## Reproducing

```bash
cargo build --locked --release -p binn-lab --bin a6-ceiling-health

# Evidence 1 — reference budget only (~31 min on 32 vCPU)
./target/release/a6-ceiling-health --suite both \
  --epochs 80,160,320,640,1280,2560 --lr 0.02,0.05,0.1,0.2 --out a6_report.md

# Evidence 2 — matched compute
for E in 80 320 1280 2560; do
  ./target/release/a6-ceiling-health --suite both --epochs $E --lr 0.05 \
    --arm-epochs $E --out matched_$E.md
done
```

The canonical budget row is always included as a harness self-check. On macOS it
reproduces the published reference exactly (drift 0.0000); a run whose self-check fails
must not be read as a sweep of the published substrate.
