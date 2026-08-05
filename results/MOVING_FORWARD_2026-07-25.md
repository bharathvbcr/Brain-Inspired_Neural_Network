# Moving forward — 2026-07-25

Companion to `HARD_AUDIT_v12_2026-07-25.md` (what was wrong) and
`HARDENING_v12_2026-07-25.md` (what was fixed). This document is about **what to
do next and why**.

---

## 1. Where the project actually stands

| Claim | Status |
|---|---|
| DFA beats chance on SHD (0.234 vs 0.05) | Real, but ~3× below the nearest comparable local rule |
| Feedback alignment degrades with depth | Reproduces known literature, on a task (`N_IN = 2`) that cannot support the claim |
| Preregistration / claim-axis discipline | Genuinely unusual for this field — but partially occupied by others as of mid-2026 |
| Matched dense-LIF → live k-WTA gap (1.0000 → 0.5188) | **The one observation that does not reproduce something known** |
| Multi-area / neuromod / E-I suites | Property tests, not results |
| GPU throughput | Withdrawn — no GPU code ever ran |

The competitive picture on SHD:

| Method | Locality | Accuracy |
|---|---|---:|
| BPTT + learned delays | none | 0.951 |
| e-prop | local in time only | 0.808 |
| **ETLP** — event-based three-factor, hardware-targeted | **full** | **0.746** |
| this project | full | **0.234** |

ETLP occupies exactly this project's claim space. Any reviewer opens with "how
does this compare to ETLP?"

## 2. The one question that changes everything

ETLP's own conclusion is that recurrence and threshold adaptation are *necessary*
for local plasticity on SHD. `ShdArch` has **neither**. So the 0.234 figure may
be measuring a feed-forward rate readout's ceiling on a temporal task, and
attributing it to locality.

That is now testable: `shd-arch-ablation` crosses
`{feed-forward, recurrent} × {fixed θ, adaptive θ} × {DFA, e-prop ceiling}`
with a shuffled-label control, under the preregistration in
`PREREG_2026-07-25_SHD_ARCH_ABLATION.md`.

**Everything downstream depends on this answer**, which is why it runs first
tonight and gets the longest timeout.

## 3. Decision tree

```
                    shd-arch-ablation
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
   INVALID_HARNESS    H1 PASS            H1 FAIL
        │                  │                  │
   fix the gate,      architecture       architecture is
   re-run. No claim.  was the confound   NOT the constraint
                           │                  │
                  ┌────────┴────────┐         │
              H2 PASS           H2 FAIL       │
                  │                 │         │
        Rebuild the SHD    Implement exact    │
        story on rec+alif. ALIF ε_a term,     │
        Re-run width/depth then LR sweep,     │
        sweeps. Retract    then re-assess     │
        old SHD numbers.                      │
                                              │
                                    Real negative result.
                                    But eliminate the two
                                    confounds (§7 of prereg)
                                    before writing it up.
```

Regardless of branch: **the transfer-gap decomposition
(`DESIGN_TRANSFER_GAP_DECOMPOSITION.md`) is the main line of work afterwards.**
It is the only angle here that isn't already occupied.

## 4. Publication paths, ranked

**1. The transfer gap (best odds).**
"Local rules validated in dense simulation do not survive event-driven
execution." Needs: a non-saturated task, the 16-rung substrate ladder, mechanism
instrumentation, and a rescue arm. Multi-week. Genuinely useful to anyone
deploying to Loihi/SpiNNaker. Do the literature search first — fifteen minutes
now beats three weeks of building something already published.

**2. A rigorous negative result on architecture vs locality (medium odds).**
Only available if H1 FAILs *and* both confounds are eliminated. "Recurrence and
adaptation do not rescue three-factor DFA on SHD; the binding constraint is X."
Workshop-tier unless the mechanism is crisp.

**3. Beating a baseline (currently closed).**
Reopens only if the ablation lands near ETLP's 0.746. Do not plan around this.

**4. Methodology (partially occupied).**
`arXiv 2606.21126` (scale stability / reference validity / depth utility, June
2026) and NeuroTrain's open benchmarking framework already cover much of it. The
`guards.rs` + prereg-lint machinery is still worth keeping — it is now the reason
this project can trust its own numbers — but it is infrastructure, not a paper.

## 5. The process problem, stated plainly

Fifteen reports in one night. Four could not fail by construction. Eight of ten
headline claims in the summary were wrong. Nobody asked why the ceiling was
*below* the treatment, even though that was printed in plain text in the h128
report and invalidated the entire SHD comparison.

That is throughput outrunning validation. Three load-bearing experiments per
night, each with a preregistration and a negative control, is strictly better
science than fifteen unvalidated ones — and would have caught the ceiling bug in
week one.

Three habits, in order of value:

1. **Every accuracy gets a negative control.** Shuffled labels, or a constant
   predictor baseline, in the same report. `guards.rs` now computes the second
   automatically; `shuffle_labels()` provides the first.
2. **Every comparison gets a scale check.** If two arms' credit signals differ in
   magnitude, you are measuring learning rate. `ModulatorScale` makes this a
   one-line assertion.
3. **Read the ceiling row first.** If a reference arm loses to the thing it is
   supposed to bound, stop and fix the harness before reading anything else.

## 6. Tonight

```bash
cd binn
./scripts/overnight.sh --smoke     # ~15 min, verify everything works
./scripts/overnight.sh             # full run, resumable
```

Order of operations: preflight → build/test gate → smokes → **LR pilot** →
**confirmatory ablation** → supporting re-runs → low-value suites.

Estimated ~6 hours at the measured throughput of the previous h128 run
(≈1.7 GOPS effective; the ablation's dominant cost is the dense `h × n_in`
eligibility decay, so recurrence adds only ~35%, not the 4× it looks like).

**Ways the night can no longer be wasted:**

| Failure | Guard |
|---|---|
| Broken build | `cargo check` + `cargo test` gate, aborts in ~20 min |
| No SHD cache | preflight checks `train.bin`/`test.bin` before anything expensive |
| Silently running on the smoke fixture | ablation exits 3 outside `--quick` |
| One job hangs | per-job wall-clock timeout |
| Crash loses everything | `.done` markers; re-run resumes |
| Ablation times out mid-grid | report rewritten after **every cell**; H1-critical cells run **first** |
| A learning rate diverges | reported as `DIVERGED`, not panicked — the rest of the sweep survives |
| Recurrent net dies or saturates | per-cell `SILENT` / `SATURATED` / `COLLAPSED` flags; degenerate cells excluded from selection |
| Machine sleeps | `caffeinate` |

**Not run:** the h256/h512 width sweep. 512 vs 128 measured +0.0056 with SE
0.0243 (t = 0.23) — there is no width effect, and it costs ~6.6 hours. Re-run it
only on whichever architecture the ablation selects, and only if the ablation
shows a real gain. `deep-snn-scaling` is demoted to Tier 4: it runs on
`CoincidenceTask` (`N_IN = 2`), so the full run adds little over the smoke.

### First thing in the morning

```bash
RUN=$(ls -1d results/runs/*_overnight | tail -1)
cat "$RUN/SUMMARY.md"
grep -rn 'INVALID_HARNESS\|DEGENERATE\|SATURATED\|SILENT\|COLLAPSED\|DIVERGED\|INVERTED\|LEAK DETECTED' "$RUN"/*.md
sed -n '/## Preregistered hypotheses/,/## Published/p' "$RUN/shd_arch_ablation_h128.md"
```

Read in this order:

1. **Health column of the ablation grid.** If `rec+alif` is `SATURATED` or
   `SILENT`, **H1 was not tested** — the fix is `rec_scale` in
   `ShdAlifArch::new`, not the hypothesis. Do not read the accuracy.
2. **Shuffled-label control.** If it leaks, every number is void.
3. **Ceiling health** (modulator parity, inversions).
4. **Only then** the H1/H2 verdicts.
5. **The LR pilot**, especially if H1 failed. If `rec+alif` peaks at a different
   learning rate than `ff+fixed`, the confirmatory run at the fixed `lr = 0.02`
   understated the architecture effect and needs re-running at the better rate.

A `readout arm ... is degenerate` panic from the *other* experiments is the guard
working, not a flake. Investigate the stimulus path before touching the guard.

## 7. Am I heading in the right direction?

The field is right and active. The engineering is strong. The instinct toward
preregistration is better than most of this literature.

What needs to change is not the direction but the ratio: fewer experiments, each
load-bearing, each with a control that could have come out the other way. The
audit found four experiments that were incapable of failing — the fix for that is
not more tests, it is fewer claims per night.

The transfer gap is worth betting on. Verify nobody else has published it, then
build the ladder properly.
