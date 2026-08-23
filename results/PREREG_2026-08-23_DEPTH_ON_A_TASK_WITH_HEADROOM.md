# Preregistration — network depth on a task whose reference can fall

**Registered:** 2026-08-23, before the suite is built and before any accuracy at
any network depth exists.

**Why:** v136 answered the depth question on `CoincidenceTask`, where the ceiling
saturates at exactly 1.0000 and "the treatment tracks its ceiling" is close to
"both arms solved an easy task"
(`RESULT_2026-08-22_DEEP_PATH_AND_TRANSPORT_SCALE.md` §6). The same disease
governs the matched-arch schedule (`RESULT_2026-08-23_TRACK_B_REREAD.md` §2).
SHD is the obvious remedy and is refused at the calibration gate; that refusal is
respected, not routed around.

**Feasibility:** `MEASUREMENT_2026-08-23_A_TASK_WITH_HEADROOM.md`.

---

## 1. The task and the operating point

`binn_data::credit_depth::CreditDepthTask` — compositional, order-sensitive,
terminal-reward-only, and currently unwired. Difficulty is set by `n_states` and
task depth.

**Operating point chosen by a rule stated before it was applied**: the cell
maximising `min(ceiling − chance, 0.95 − ceiling)` — furthest from *both*
saturation and chance — over the swept grid.

Applied to §3 of the measurement, that selects **`n_states = 8`, task depth 4**
(ceiling 0.4600, chance 0.1250; margin 0.3350), narrowly over `n_states = 4`,
depth 4 (0.3300).

**Disclosure.** The sweep reported the treatment as well as the ceiling and I have
seen both. **The selection rule reads the ceiling only**, exactly as v136's
step-size rule did, so the arm under test did not influence the operating point.
That is a weaker guarantee than not having seen it, and it is stated rather than
glossed.

**Task depth is fixed at 4 for the whole suite.** The variable under test is
*network* depth. Confounding the two is the error this design exists to avoid.

## 2. Design

`shared_bptt`, exactly as v136 uses it — one shared forward, one initialisation,
one optimiser, differing only in whether gradients are true or feedback-projected.

| | |
|---|---|
| network depths | 1, 2, 3, 4 |
| width | 64 |
| ceiling | `train_bptt` (Adam) |
| treatment | `train_learned_feedback_adam` |
| seeds | 12 |
| epochs | 40 |
| chance | 0.1250 (`1 / n_states`) |
| harness validity | `binn_lab::guards::CeilingHealth`, and a readout audit per arm |

The readout audit is **mandatory**, not optional: §3 of the measurement flagged
one cell where ceiling and treatment agreed to four decimals, which is the shape
of two arms agreeing because both are degenerate.
`readout_audit_coverage.rs` requires it and this binary will not be added to
`KNOWN_UNAUDITED`.

## 3. Hypotheses, registered two-sided

There is no directional theory about feedback alignment and depth, so none is
assumed.

| id | hypothesis | bar |
|---|---|---|
| **V-1** | the harness is usable at all | `CeilingHealth::Ok` at every depth: ceiling above chance by the 0.05 margin, and not inverted |
| **V-2** | the reference has headroom where it is read | mean ceiling ≤ 0.95 at every depth; a saturated ceiling **voids** the reading at that depth |
| **V-3** | no arm is degenerate | readout audit reports no fatal defect for either arm at any depth |
| **D-1** *(two-sided)* | network depth changes the gap | \|gap(depth 4) − gap(depth 1)\| ≥ 0.05, where gap = treatment − ceiling |
| **D-2** *(two-sided)* | network depth changes the ceiling | \|ceiling(4) − ceiling(1)\| ≥ 0.05 |

## 4. Named outcomes

- **V-1…V-3 hold and D-1 fires with a growing negative gap** → feedback alignment
  degrades with network depth on a task with headroom. This is the result v136
  could not produce, and the first one on this axis that is not bounded by a
  saturated ceiling.
- **V-1…V-3 hold and D-1 does not fire** → no depth penalty is detected at this
  operating point, **with the ceiling demonstrably unsaturated**. A genuine
  negative, unlike v136's.
- **D-2 fires and D-1 does not** → depth moves what the architecture can do at
  all, and the credit rule tracks it. Reported as a statement about the
  architecture, not the rule.
- **V-1 fails at any depth** → that depth is reported `INVALID_HARNESS` and
  contributes to no verdict, as v136 does.
- **V-2 fails** → the operating point saturated once network depth was varied.
  The suite is **not** re-tuned to a new cell: that would be selecting an
  operating point on the outcome. It is reported as a failed feasibility
  assumption and the rule in §1 is re-applied to a fresh sweep in a separate
  registration.
- **V-3 fails** → the two suspicious cells in the measurement were the same
  degeneracy, now diagnosed. No depth verdict is issued.

## 5. What this may not claim

- **It is not SHD.** A compositional symbolic task is not an input-rich sensory
  one. Whatever this finds is about credit assignment through composed
  transformations, and transfers to SHD only as a hypothesis.
- **It does not supersede v136.** v136 stands as the `CoincidenceTask` result with
  its saturation caveat. This is a second task, not a correction.
- **It does not unblock anything.** `SHD_INSTRUMENT_STATE` stays `Uncalibrated`
  and the SHD depth suite stays refused. This experiment exists *because* that
  refusal is respected.
- **Task depth 4 is one point.** Whether the finding holds at task depth 8, where
  the ceiling falls to 0.2750, is untested and is not implied.
