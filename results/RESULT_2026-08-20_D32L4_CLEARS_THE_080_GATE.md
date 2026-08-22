# d32/L4 reaches 0.8320 at e400 — 12 of 12 seeds above the registered 0.80 gate, and budget-stable

> **SCOPE MEASURED 2026-08-21.** Every scope limit this document inherited was
> measured at d32/**L1**. Wave 8 measured them at d32/**L4**, and two of them
> changed: the 0.80 clearance does **not** hold on `channels-700` (0.7864, 6/12),
> and the width inversion is far worse at L4 than at L1 (**−0.1618** vs −0.0159 at
> h1024). The *gain* transfers everywhere tested (12/12 seeds positive on both
> other axes). A registered mechanistic prediction (S-5) also **failed**, which
> removes the temporal-resolution reading from the mechanism claim.
> See [`RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md`](RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md).


**Protocol:** `PREREG_2026-08-20_SHD_ATTENTION_D32L4_AT_E400.md`, registered
before any e200 cell and before any d32/L4 cell above e100 existed. Verdicts
computed once at n=12.
**Cells:** 36 of 36, **0 voided, 0 diverged**, on the pinned binary `22d97c51` —
the same binary that produced the control cells being compared against.
**Backend:** rust on Linux/aarch64, h128, `published-2ms` / `adjacent-sum-5`.

---

## 1. The ladder

| epochs | `ff+fixed` | `ff+fixed+attn` d32/L4 | gain | seeds ≥ 0.80 |
|---:|---:|---:|---:|---:|
| 100 | 0.6659 | 0.8209 | +0.1550 | 11/12 |
| 200 | 0.6868 | 0.8322 | +0.1454 | **12/12** |
| **400** | 0.7062 | **0.8320** | **+0.1258** | **12/12** |

Per-seed at e400: 0.8083, 0.8193, 0.8238, 0.8264, 0.8282, 0.8348, 0.8366,
0.8375, 0.8383, 0.8388, 0.8450, 0.8472. **The lowest seed is 0.8083.**

## 2. The registered verdicts

| ID | measured | threshold | verdict |
|---|---:|---|---|
| **R-1** | mean **0.8320**, **12/12** seeds ≥ 0.80 | ≥ 0.80 and ≥ 9 of 12 | **SUPPORTED** |
| **R-2** | \|e400 − e200\| = **0.0002** | < 0.01 | **BUDGET-STABLE** |
| **R-3** | gain **+0.1258** | ≥ 0.05 | **SUPPORTED** |
| **R-4** | 0 validity-gate failures of 12 | all pass | **PASS** |

**R-1 and R-2 both hold**, which is what the preregistration required for the
accuracy to be reportable as an architecture result rather than a budget artefact.
0.0002 between adjacent doublings is as stable as this instrument measures
anything.

## 3. What changed, and what did not

**W1-1 measured +0.0421 and was NOT SUPPORTED. R-3 measures +0.1258 against the
same 0.05 bound and is SUPPORTED — three times the effect.** Nothing about the
method changed. The only difference is the attention configuration: W1 registered
**d32/L1**, which wave 2 later showed to be the *weakest* of the six settings it
swept. The campaign tested one point and reported the negative honestly; the sweep
found the point was badly chosen.

This is the value of having run wave 2 at all. Had the campaign stopped at W1's
negative — which was correctly computed, at n=12, with t=8.21 — the conclusion
would have been "attention does not raise the ceiling", and it would have been
wrong about the arm rather than about the configuration.

**The pilot's +0.1702 is still an artefact.** It was measured at d32/L1/e20, where
the control had not caught up. R-3's +0.1258 is at e400 with the control at its
own converged-ish value, and is budget-stable. These are different quantities that
happen to be similar in size.

## 4. What this does NOT establish

**Not calibration.** The SHD instrument's criterion 4 asks for three clean seeds
at ≥ 0.80 and this delivers twelve — but **criterion 5 requires a matched
Python/Rust configuration**, and no Python mirror of the attention axis exists
(`scripts/shd_calibration/arms.py`). No accuracy at any level satisfies criterion
5. `SHD_INSTRUMENT_STATE` remains `Uncalibrated`, the local-learning family stays
blocked, and the frozen-attention local arm
(`BLOCKER_2026-08-19_FROZEN_ATTENTION_LOCAL_ARM.md`) still cannot run.

**Not a comparison with the recorded 0.7378 ceiling.** The cross-machine gate
FAILs on every instance
(`MEASUREMENT_2026-08-19_CROSS_MACHINE_BIT_EXACTNESS.md`). The gate cleared here
is the **0.80 constant**, which is a registered threshold rather than a measured
macOS quantity — that comparison is licensed, and the one against 0.7378 is not.

**Not scope beyond h128 / `adjacent-sum-5`.** Wave 3 measured the d32/L1 gain
inverting by h1024 and losing seed-consistency at `channels-700`. Whether d32/L4
behaves differently at width or geometry is **unmeasured**.

**Not optimality.** d32/L4 is the best of six configurations swept at e100. L8,
d64/L4, d128/L4 and the rest are untested — and the sweep was monotone in both
axes at h128, so there is no reason to think L4 is the top.

## 5. Where this leaves the project

The registered 0.80 accuracy gate, which this instrument has never cleared and
against which its converged feed-forward reference sits at 0.7378, is cleared by
**every one of twelve seeds** by a spiking forward model whose only addition is a
frozen-architecture attention read-out over the time axis — at h128, on the
anchor contract, budget-stable between e200 and e400.

The single thing standing between that and a calibrated instrument is the Python
mirror. It is not a compute problem, it has never been a compute problem, and
~$40 of spot capacity has now made that unusually clear.
