# SHD Architecture Ablation (C1-SHD-ARCH)

**Protocol version:** 141  
**Experiment:** shd-arch-ablation  
**Preregistration:** `results/PREREG_2026-07-25_SHD_ARCH_ABLATION.md`  
**Mode:** LR-SWEEP PILOT (DFA only, reduced schedule — a pilot, not a confirmatory run)  
**Question:** is DFA ≈ 0.234 on SHD a limit of local credit assignment, or of a feed-forward fixed-threshold forward model?  
**Schedule:** hidden=128, seeds=2, epochs=8, lrs=[0.005, 0.02, 0.08]  
**Data:** n_train=2000, n_test=500, n_in=700, T=100, classes=20, chance=0.0500, fixture=false  

## Ablation grid

Cells run in H1-critical order (`ff+fixed`, then `rec+alif`), so a truncated run still answers the preregistered contrast. Where several learning rates were run, H1/H2 use the **best non-degenerate cell per architecture**.

| Architecture | Rule | lr | Mean acc | SE | 95% CI (seeds) | Hidden activity | Modulator RMS | Wall (s) | Health |
|---|---|---:|---:|---:|---|---:|---:|---:|---|
| ff+fixed | SHD_ALIF_DFA | 0.0050 | 0.2030 | 0.0170 | [0.1697, 0.2363] | 0.0061 | 9.738e-3 | 38.9 | ok |
| ff+fixed | SHD_ALIF_DFA | 0.0200 | 0.0890 | 0.0070 | [0.0753, 0.1027] | 0.0272 | 1.262e-2 | 39.4 | ok |
| ff+fixed | SHD_ALIF_DFA | 0.0800 | 0.0720 | 0.0180 | [0.0367, 0.1073] | 0.0969 | 1.386e-2 | 37.6 | ok |
| rec+alif | SHD_ALIF_DFA | 0.0050 | 0.1970 | 0.0130 | [0.1715, 0.2225] | 0.0048 | 9.682e-3 | 60.2 | ok |
| rec+alif | SHD_ALIF_DFA | 0.0200 | 0.1230 | 0.0070 | [0.1093, 0.1367] | 0.0158 | 1.193e-2 | 60.9 | ok |
| rec+alif | SHD_ALIF_DFA | 0.0800 | 0.0650 | 0.0150 | [0.0356, 0.0944] | 0.0518 | 1.377e-2 | 58.9 | COLLAPSED (predicts a single class) |
| rec+fixed | SHD_ALIF_DFA | 0.0050 | 0.1940 | 0.0160 | [0.1626, 0.2254] | 0.0064 | 9.740e-3 | 61.3 | ok |
| rec+fixed | SHD_ALIF_DFA | 0.0200 | 0.0940 | 0.0120 | [0.0705, 0.1175] | 0.0281 | 1.263e-2 | 58.5 | ok |
| rec+fixed | SHD_ALIF_DFA | 0.0800 | 0.0790 | 0.0290 | [0.0222, 0.1358] | 0.1108 | 1.388e-2 | 58.4 | COLLAPSED (predicts a single class) |
| ff+alif | SHD_ALIF_DFA | 0.0050 | 0.1860 | 0.0160 | [0.1546, 0.2174] | 0.0046 | 9.688e-3 | 36.6 | ok |
| ff+alif | SHD_ALIF_DFA | 0.0200 | 0.0780 | 0.0020 | [0.0741, 0.0819] | 0.0154 | 1.191e-2 | 36.5 | ok |
| ff+alif | SHD_ALIF_DFA | 0.0800 | 0.0690 | 0.0190 | [0.0318, 0.1062] | 0.0496 | 1.377e-2 | 36.5 | NEAR-COLLAPSED (>95% of predictions in one class) |

### Degeneracy

**3 of 12 cells degenerate** — their accuracies are NOT interpretable as statements about the credit rule: `rec+alif / SHD_ALIF_DFA / lr=0.08` (COLLAPSED (predicts a single class)), `rec+fixed / SHD_ALIF_DFA / lr=0.08` (COLLAPSED (predicts a single class)), `ff+alif / SHD_ALIF_DFA / lr=0.08` (NEAR-COLLAPSED (>95% of predictions in one class))

A collapsed, silent or saturated arm scores near chance. Read as a bare number that is indistinguishable from "this credit rule does not work", which would invert the conclusion — hence the explicit health column.

## Ceiling health

| Architecture | DFA | e-prop ceiling | Modulator RMS ratio | Status |
|---|---:|---:|---:|---|
| — | — | — | — | no comparable pairs yet |

Parity tolerance 3.50; worst observed 0.00.

## Negative control (shuffled labels)

0.0390 (95% CI [0.0260, 0.0610]); chance 0.0500; threshold 0.1000; **ok**

## Preregistered hypotheses

| ID | Statement | Measured | Verdict |
|---|---|---|---|
| H1 | `rec+alif` DFA beats `ff+fixed` DFA by ≥ 0.10 with disjoint 95% CIs | gain -0.0060 (0.2030 → 0.1970), CIs disjoint = false | FAIL |
| H2 | Best-architecture DFA reaches ≥ 0.50 | 0.1970 | FAIL |

Validity gates: control ok, modulator parity ok, H1 cells present ok.

## Published reference points (same dataset, not run here)

| Method | Locality | SHD accuracy |
|---|---|---:|
| BPTT + learned delays (Hammouamri et al. 2023) | none | 0.951 |
| e-prop | local in time, non-local in space | 0.808 |
| ETLP (Quintana et al. 2024) | fully local three-factor | 0.746 |
| this run, best DFA architecture | fully local three-factor | 0.197 |

## Interpretation

**Architecture is NOT the binding constraint.** Recurrence and adaptation moved DFA by only -0.0060 (0.2030 → 0.1970). The feed-forward confound is ruled out and the limit lies in the credit pathway or the eligibility formulation. This is a legitimate negative result and the more interesting one to write up — but it may not be claimed until the two disclosed confounds are eliminated: the missing ALIF `ε_a` term (biases against this arm) and the un-swept learning rate. Run `--lr-sweep` before concluding.

## Non-claims

- **Not SOTA** and not a like-for-like ETLP comparison: different eligibility formulation, different schedule, capped splits unless `--full`.
- **Not Gate G2.**
- The eligibility trace omits the exact ALIF `ε_a` cross-term (`binn_learn::shd_alif` module docs). This biases **against** H1. If the adaptive axis shows any effect, implementing that term is the required follow-up.
- An `--lr-sweep` run is a **pilot**. It may inform which learning rate a confirmatory run uses; it may not itself be reported as the confirmatory result.
- `INVALID_HARNESS` blocks every H1/H2 claim; it is not a soft warning.
