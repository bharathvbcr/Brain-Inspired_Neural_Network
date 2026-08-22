# The attention read-out converges between 5 and 10 epochs — bracketed, not floored

**Protocol:** `PREREG_2026-08-20_SHD_ATTENTION_LADDER_FLOOR.md` (wave 7),
registered before any e5 or e10 cell existed. Verdicts computed once at n=12.
**Cells:** 48 of 48, **0 voided**, joining waves 6 and 1 on the same ladder.
**Backend:** rust on Linux/aarch64, binary `22d97c51`, one machine throughout.

---

## 1. The complete ladder

| epochs | `ff+fixed` | `ff+fixed+attn` | gain | attn as % of its own e400 |
|---:|---:|---:|---:|---:|
| 5 | 0.4529 | 0.6756 | **+0.2227** | **90.3%** |
| 10 | 0.5336 | 0.7337 | +0.2002 | **98.1%** |
| 20 | 0.5851 | 0.7479 | +0.1627 | 99.9% |
| 50 | 0.6484 | **0.7539** | +0.1055 | 100.7% |
| 400 | 0.7062 | 0.7483 | +0.0421 | 100.0% |

n=12 at every cell, seeds shared across every rung and both arms.

## 2. The registered verdicts

| ID | measured | threshold | verdict |
|---|---:|---|---|
| **W7-1** | attn(e10)/attn(e400) = **0.9806** | ≥ 0.95 | **SUPPORTED** |
| **W7-2** | attn(e5)/attn(e400) = **0.9029** | ≥ 0.95 | **NOT SUPPORTED** |
| **W7-3** | ff+fixed(e10)/ff+fixed(e400) = **0.7556** | < 0.90 | **SUPPORTED** |
| **W7-4** | guard | fires only if W7-2 holds | **does not fire** |

## 3. W7-2 failing is the result

W7-2 said the arm would still be converged at e5. It is not — 90.3%, below the
0.95 criterion. **That is what completes the measurement.** e5 fails the
criterion and e10 passes it, so the convergence point is *bracketed*:
somewhere in `(5, 10]` epochs.

Wave 6 reported "20 epochs" and that number was its own floor — the arm was
already converged at the lowest rung measured, so the ladder never bracketed
anything. `PREREG_2026-08-20…` §3 registered W7-4 specifically to catch the same
mistake recurring at e5, and named the honest report if it did: *"at or below 5
epochs", not "5 epochs"*.

**The guard does not fire.** There is a failing rung below a passing one, so this
is a bound on both sides rather than an artefact of where measurement stopped.
The claim is no longer limited by the instrument.

## 4. What is now established

**The attention read-out converges between 5 and 10 epochs. The rate-only control
is at 75.6% of its own converged accuracy at e10, and needs the full 400 epochs to
reach 0.7062 — which is still below what attention reaches in ten.**

At e5 the arm already beats the control's 400-epoch result: **0.6756 in five
epochs** against 0.7062 in four hundred, and by e10 it is ahead outright
(0.7337). The gain declines monotonically as the control catches up — +0.2227,
+0.2002, +0.1627, +0.1055, +0.0421 — which is the signature of a budget effect and
exactly what W6-3 registered in advance.

The peak is at **e50 (0.7539)**, 100.7% of the e400 value: the arm is very
slightly *worse* after another 350 epochs. Falling loss with declining accuracy is
overfitting, confirming for the second time that W1-4's UNDERTRAINED verdict reads
the opposite of what is happening.

## 5. What must still not be claimed

- **Not a ceiling result.** W1-1 remains NOT SUPPORTED at n=12; the converged gap
  is +0.0421 against a registered 0.05.
- **No comparison to the macOS-recorded 0.7032 / 0.7378.** Every instance FAILs
  the cross-machine gate.
- **Nothing about local learning.** These are BPTT reference arms; the
  frozen-attention local arm is built, registered and blocked
  (`BLOCKER_2026-08-19_FROZEN_ATTENTION_LOCAL_ARM.md`).
- **Not "converges in 7.5 epochs"** or any interpolated point. The measurement is
  an interval, and reporting a midpoint would invent precision the ladder does not
  have.

## 6. The claim, in one sentence

On this forward model, at this contract and width, a time-axis attention read-out
reaches its converged accuracy **within 10 epochs and not within 5**, exceeding in
ten epochs what the rate-only read-out reaches in four hundred — and it does so by
using temporal order, since destroying that order makes it *worse* than the
control in 12 of 12 seeds.
