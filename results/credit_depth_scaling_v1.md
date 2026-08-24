# Credit-Depth Scaling Report

> **HARNESS DEFECT - do not interpret any comparison below.** At least one depth failed a validity gate: the ceiling did not clear chance, it was inverted, it saturated, or an arm's readout collapsed. Verdicts are `INVALID_HARNESS` for exactly this reason.

**Protocol Version:** 1  
**Experiment:** credit-depth-scaling  
**Preregistration:** `results/PREREG_2026-08-23_DEPTH_ON_A_TASK_WITH_HEADROOM.md`  
**Schedule:** FULL SCIENTIFIC (n=12, hidden=64, epochs=40)  
**Task:** `CreditDepthTask`, n_states=8, task depth 4 (fixed), chance 0.1250  

Treatment and ceiling share one forward graph, one initialisation, one optimiser and one step size, and differ only in whether the gradients are true or feedback-projected. **Task depth is held fixed**; the variable is network depth.

## Per network depth

| Depth | Hidden | Treatment | SE | Ceiling | SE | Gap | Headroom | Ceiling health | Verdict |
|---:|---|---:|---:|---:|---:|---:|---|---|---|
| 1 | 64 x 1 | 0.4742 | 0.0088 | 0.4692 | 0.0080 | +0.0050 | ok | INVERTED — ceiling below treatment; do not interpret | INVALID_HARNESS |
| 2 | 64 x 2 | 0.5708 | 0.0135 | 0.5822 | 0.0125 | -0.0114 | ok | ok | PASS |
| 3 | 64 x 3 | 0.5336 | 0.0116 | 0.5794 | 0.0085 | -0.0458 | ok | ok | PASS |
| 4 | 64 x 4 | 0.4647 | 0.0122 | 0.5119 | 0.0084 | -0.0472 | ok | ok | PASS |

`Gap` is treatment minus ceiling. `Headroom` fails when the ceiling exceeds 0.95, which voids the reading at that depth — the gate v136 did not have.

## Readout audit

An accuracy cannot distinguish a learner from a constant predictor. `Classes` is the mean number of distinct classes an arm actually predicted, out of 8; `Majority` is the share in the single most-predicted class.

| Depth | Arm | Classes | Majority | Defects |
|---:|---|---:|---:|---|
| 1 | treatment | 8.00 | 0.2158 | none |
| 1 | ceiling | 8.00 | 0.2119 | none |
| 2 | treatment | 8.00 | 0.2158 | none |
| 2 | ceiling | 8.00 | 0.2108 | none |
| 3 | treatment | 7.83 | 0.2350 | none |
| 3 | ceiling | 8.00 | 0.2281 | none |
| 4 | treatment | 6.92 | 0.2703 | none |
| 4 | ceiling | 7.50 | 0.2383 | none |

## Registered hypotheses

- **D-1** *(two-sided)* network depth changes the gap: |gap(4) - gap(1)| = **0.0522**, bar 0.05 -> **SUPPORTED**
- **D-2** *(two-sided)* network depth changes the ceiling: |ceiling(4) - ceiling(1)| = **0.0428**, bar 0.05 -> **NOT SUPPORTED**

**No hypothesis verdict may be read while the harness banner is present.**

## Interpretation caveat

This is a compositional symbolic task, not an input-rich sensory one. Whatever it finds is about credit assignment through composed transformations and transfers to SHD only as a hypothesis. Task depth 4 is one point; at task depth 8 the ceiling falls to 0.2750 and that regime is untested.
