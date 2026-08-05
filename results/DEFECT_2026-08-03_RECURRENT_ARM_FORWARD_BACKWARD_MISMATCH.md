# Defect — the recurrent arms' gradient is not the gradient of their forward

**Found:** 2026-08-03, during the rust kernel optimisation.
**Severity:** was blocking for `rec+fixed` and `rec+alif`. **`ff+fixed` and
`ff+alif` were never affected — the 216 completed cells are safe.**
**Status: FIXED 2026-08-03** (§5), with the test that should have caught it
(§4b, §5.2). No recorded result changes: no recurrent cell has ever been run.

```
claim_axis: Integrity
object_under_test: Whether the recurrent forward in `loss_and_gradient_arm`
  computes the model its backward differentiates.
may_claim: It did not. The forward read partially-updated spikes while the
  backward differentiated the clean-previous-step model; verified by direct
  execution, fixed, and pinned by a test. No gradient check of any tolerance
  could have caught it, because the loss is piecewise constant in w_rec.
must_not_claim: Anything about ff+fixed or the completed matrix; any statement
  about how large the resulting error would have been after training, which was
  never measured; that rust and python recurrent forwards are now *measured* to
  agree — no cross-backend recurrent fixture exists yet.
```

---

## 1. The defect

`binn-learn/src/shd_matched_arms.rs`, inside the forward timestep loop:

- **:285** `previous_s` is declared once, outside the timestep loop.
- **:304** reads `previous_s[j]` for **all** `j in 0..hidden` — the recurrent drive.
- **:314** writes `previous_s[h] = spike` at the end of that same `h` iteration.

Both are inside a single `for h in 0..hidden` loop, and `previous_s` is never
cloned or shadowed between them. So when the loop reaches unit `h`:

| j | what `previous_s[j]` holds | correct? |
|---|---|---|
| `j < h` | **current**-timestep spike `s_j(t)` | **no** |
| `j == h` | irrelevant — `w_rec` diagonal is pinned to 0 | — |
| `j > h` | previous-timestep spike `s_j(t-1)` | yes |

Roughly half the off-diagonal recurrent terms use the wrong timestep, and *which*
half is an artifact of neuron indexing order.

The backward does not share this convention. **:378** accumulates
`gradient.w_rec[h,j] += du[h] * previous_spike_log[t*hidden + j]`, and
`previous_spike_log` is snapshotted at the **top** of each timestep (**:289**),
so it holds the true `s(t-1)`. Separately, **:357-362** routes unit `h`'s
influence only into `t+1` via `du_next`, whereas the aliased forward also feeds
`s_h(t)` into `u_g(t)` for every `g > h` at the *same* timestep — a path the
backward has no term for at all.

So the backward is the exact BPTT gradient of the documented model
`sum_j w_rec[h,j] * s_j(t-1)`. The forward runs something else.

Two nearby reads are **correct** and should not be "fixed": the adaptation trace
(**:290-294**) is a separate completed loop over `h`, and the detached-reset gate
(**:296**) reads `previous_s[h]` before that iteration's own write.

## 2. Verification

Executed against the real `loss_and_gradient_arm` from a throwaway crate outside
the repository — no project file was modified — with
`MatchedWeights::deterministic(40, 24, 20, 91)`, `w_rec[i] = ((i % 17) - 8) * 2e-2`,
zero diagonal, arm `rec+fixed`:

```
max |membrane_rust - membrane_true_previous_step| = 3.976696e-1   (threshold is 1.0)
spike-train mismatches                            = 1 / 720, first at (t=9, h=17)
rust forward reproduces the aliased convention bit-exactly = true
```

The divergence is order-1 against a unit threshold, not a rounding artifact.

## 3. The two backends disagree, and no gate would notice

`scripts/shd_calibration/arms.py:139` computes
`current = current + weights.w_rec @ previous_s` — one vectorised product against
the whole vector, evaluated before any spike for this timestep exists — and
**:151** *re-binds* `previous_s = spike` rather than mutating in place. Python
therefore uses the true `s(t-1)`, is self-consistent with its own backward
(**:185**), and matches the documented model.

**The rust and python recurrent forwards compute different functions.** Any
cross-backend parity comparison on `rec+*` would fail by ~4e-1 against a
registered forward tolerance of 1e-6.

Nothing currently in place would catch it:

| check | why it misses this |
|---|---|
| `arms.py::selftest()` | python-only, never touches rust |
| `arms.py::_naive_backward` | takes the forward's outputs *as inputs*; validates backward-vs-backward and cannot detect a forward/backward inconsistency by construction. It also uses the same previous-step convention on both sides |
| `ff_fixed_matches_shipped_reference` | recurrent branch never taken |
| `every_arm_changes_the_spike_train` | asserts the spike train *differs* from baseline; any recurrent term whatsoever satisfies that |
| `recurrent_gradient_has_zero_diagonal` | the diagonal is the one index the aliasing cannot affect |
| Gate E (cross-backend parity) | **not implemented** — raises "GATE E BLOCKED - no arm fixtures yet" |

There is no finite-difference or forward/backward consistency check anywhere in
the rust tree for this module.

## 4. Consequences

1. **`PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION` is blocked on its rec+alif
   half.** That is 12 of its 24 cells and ~30.5 h of its ~35 h budget, and its
   **H2** — "recurrence makes timing usable", the arm × condition interaction —
   is measured entirely on the broken arm. H1 (ff+fixed order-invariance, the
   novel result) is **not** affected and could run on its own; the prereg's §10
   stopping rule already orders cells so a truncated run answers H1 first.
2. **`PREREG_2026-08-02_SHD_BPTT_CEILING_RECALIF` is blocked outright** — its
   entire object is the rec+alif arm.
3. **Nothing already recorded is affected.** All 216 cells are `ff+fixed`
   (`arm.recurrent == false`), where both the aliased read at :304 and the
   recurrent gradient at :375-380 are skipped entirely and `w_rec` is empty.
4. **The G8 `W_rec`-scale pilot required by both preregs would have been run on
   the broken arm**, and would have produced a scale calibrated to the wrong
   forward.

## 4b. Why no gradient check could have caught this — the deeper reason

The obvious remedy is "add a finite-difference gradient check." **That remedy is
unavailable here, and understanding why is the useful part of this report.**

A finite-difference check on `w_in` or `w_rec` was written and it failed
immediately, with the numerical derivative coming back as *exactly* `0.0`:

```
ff+fixed w_in[0]: analytic 7.2298246e-3 vs numerical 0e0
```

That is not a bug in the test. The spike function is a **hard threshold**, so the
loss is piecewise constant in `w_in` and `w_rec`: the true derivative is zero
almost everywhere and undefined on the measure-zero set where a spike flips.
Everything this module produces for those blocks is a **surrogate** gradient —
deliberately *not* the true gradient. There is no numerical derivative for it to
agree with.

Consequences, in order of importance:

1. **No gradient check, of any tolerance, can detect a wrong-timestep recurrent
   forward.** The gradient that would expose it does not exist. The
   `GATE_EF_WORK.md` plan to catch arm bugs via gradient tolerances is
   structurally incapable of catching this class of defect.
2. **The forward has to be pinned directly** — reimplemented independently with
   explicit `s(t-1)` semantics and compared bit-for-bit. That is what
   `recurrent_drive_uses_previous_timestep_spikes` now does.
3. **Finite differencing remains valid for the readout only.** Given the spike
   train, the loss is smooth in `w_out`/`b_out`. That check now runs for all four
   arms; its tolerance is set by f32 central-difference roundoff
   (~`f32::EPSILON * |loss| / eps` ≈ 1e-4), not by gradient exactness.

This also explains how the defect survived review: every existing check compared
the arm path against itself, a serialization round-trip, or a backward against
another backward sharing the same convention. None pinned the forward.

## 5. Fix applied 2026-08-03

The recurrent drive now reads the clean snapshot that already existed —
`previous_spike_log[t * hidden + j]`, written at the top of each timestep —
instead of the live `previous_s`. The forward now matches both its own backward
and the python mirror, at no cost.

### 5.1 Why this is a bug fix and not a registered protocol change

An earlier draft of this document said the fix required registration. On
reflection that was over-cautious, and being over-cautious had a real cost: it
would have left a known-wrong gradient in place, blocking ~30.5 h of planned
compute, in exchange for no protection at all. The reasoning:

- **No recorded result changes.** No cell has ever been produced with a recurrent
  arm. All 216 completed cells are `ff+fixed`, where the changed lines are inside
  `if arm.recurrent` and are never executed.
- **The documented model is unchanged.** `sum_j w_rec[h,j] * s_j(t-1)` is what
  the module docstring, `arms.py:139`, and the prereg all specify. The rust code
  deviated from that specification through an in-place aliasing.
- **The fix restores the registered protocol rather than altering it.** What a
  prereg registers is hypotheses, thresholds, conditions and stopping rules — not
  the float semantics of an implementation defect.

The discipline that *does* apply is: prove nothing recorded moves, and land the
test that should have caught it. Both were done.

### 5.2 Verification

| check | result |
|---|---|
| `recurrent_drive_uses_previous_timestep_spikes` (new) | passes — independent forward reimplementation with explicit `s(t-1)`, bit-equal membrane trace across `rec+fixed` and `rec+alif` |
| `readout_gradient_matches_finite_difference_for_every_arm` (new) | passes for all four arms |
| `ff_fixed_matches_shipped_reference` | still passes |
| full `binn-learn` suite | 155 passed, 0 failed |
| rust Gate F, 13 recorded cells | **13/13 PASS** after the fix — `ff+fixed` untouched |

The Gate F evidence is stronger than a bare pass. `gate-f-rust/runs.jsonl` records
the same 13-cell suite passing under two *different* binaries —
`sha256 6f6dbbc9fd58…` before this fix and `10df998c491c…` after it. The binary
demonstrably changed and the `ff+fixed` output did not move by a single bit,
which is the property that matters for the 216 completed cells.

The first of those is the test that matters: per §4b it is the *only* form of
check that can detect this defect, because the gradient which would expose it
does not mathematically exist.

Still outstanding, and not addressed here: Gate E implemented far enough to
compare the rust and python recurrent forwards on a shared fixture (G7 in
`GATE_EF_WORK.md`). The two backends should now agree, but that is argued, not
measured — there is no cross-backend recurrent fixture to measure it with.

---

**Artifacts.**
`binn-learn/src/shd_matched_arms.rs` — :285, :289, :295-315 (esp. :304 / :314), :357-362, :378.
`scripts/shd_calibration/arms.py` — :139, :151, :185, :234-282.
`scripts/gates_ef.py` — :258-279, Gate E blocked.
`results/GATE_EF_WORK.md` — G7.
