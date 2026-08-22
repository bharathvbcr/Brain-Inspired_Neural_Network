# Preregistration — where the attention arm actually converges (wave 7)

**Registered:** 2026-08-20, **before any e5 or e10 cell existed** — zero in the
plan, zero in `results/`, zero in `claims/`, checked and recorded at registration
time.
**Adds to:** the campaign as wave 7. Changes no existing hypothesis or threshold.
**Named in advance by:** `RESULT_2026-08-20_W6_ATTENTION_IS_SAMPLE_EFFICIENCY.md`
§5, as "the obvious next question, deliberately unasked".

---

## 1. Why

Wave 6 established the campaign's surviving claim: the attention read-out reaches
**99.95%** of its 400-epoch accuracy within **20 epochs**, while the control
reaches 82.9% of its own. The claim is sample efficiency.

But **e20 is that ladder's floor, and the arm was already converged when it got
there.** So "20 epochs" is not a property of the arm — it is a property of where I
happened to stop measuring. The true figure is at or below 20 and unknown, and the
difference between "converges in 20 epochs" and "converges in 5" is most of the
claim's force.

A claim bounded by the measurement floor rather than by the phenomenon is a claim
that has not been measured. This wave lowers the floor.

## 2. Registered schedule

48 cells at the anchor (`published-2ms` / `adjacent-sum-5`), h128, 12 seeds:

| arm | epochs added here |
|---|---|
| `ff+fixed+attn` (d32, L1) | 5, 10 |
| `ff+fixed` | 5, 10 |

Assembled ladder, all on one machine, all seeds shared:
**e5 · e10 · e20 · e50 · e100 · e400 · e800**.

## 3. Hypotheses

| ID | statement | threshold |
|---|---|---|
| **W7-1** (primary) | The arm converges below the wave-6 floor | `attn(e10) ≥ 0.95 × attn(e400)` |
| **W7-2** | It converges below that too | `attn(e5) ≥ 0.95 × attn(e400)` |
| **W7-3** | The control does not, at either rung | `ff+fixed(e10) < 0.90 × ff+fixed(e400)` |
| **W7-4** (guard) | The floor is genuinely reached | if **W7-2 holds**, the ladder has *again* failed to bracket the convergence point, and the honest report is "at or below 5 epochs", **not** "5 epochs" |

**W7-4 is the point of registering this rather than just running it.** Wave 6
produced a number that reads like a measurement — "20 epochs" — but was really its
own floor. The same mistake is available here at e5, and naming it in advance is
what stops it being made twice. If W7-2 holds, the reported claim stays an upper
bound and a further wave is a *separate* registration.

## 4. What must not be claimed

- A convergence *point*, at any rung that is the ladder's lowest. Only an upper
  bound, unless a rung below it is measured and fails the 0.95 criterion.
- Any comparison against the macOS-recorded 0.7032 / 0.7378. The cross-machine
  gate FAILs on every instance.
- That fast convergence implies anything about local learning. BPTT references.

## 5. Stopping rule

**Twelve seeds, two new rungs, verdicts computed once.** No rung below e5 is
authorised here. If W7-2 holds, the correct response is a new registration, not
an extension of this one.
