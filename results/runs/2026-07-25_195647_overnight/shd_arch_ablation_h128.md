# SHD Architecture Ablation (C1-SHD-ARCH)

**Protocol version:** 141  
**Experiment:** shd-arch-ablation  
**Preregistration:** `results/PREREG_2026-07-25_SHD_ARCH_ABLATION.md`  
**Mode:** CAPPED scientific (2000/500)  
**Question:** is DFA ≈ 0.234 on SHD a limit of local credit assignment, or of a feed-forward fixed-threshold forward model?  
**Schedule:** hidden=128, seeds=3, epochs=15, lrs=[0.02]  
**Data:** n_train=2000, n_test=500, n_in=700, T=100, classes=20, chance=0.0500, fixture=false  

## Ablation grid

Cells run in H1-critical order (`ff+fixed`, then `rec+alif`), so a truncated run still answers the preregistered contrast. Where several learning rates were run, H1/H2 use the **best non-degenerate cell per architecture**.

| Architecture | Rule | lr | Mean acc | SE | 95% CI (seeds) | Hidden activity | Modulator RMS | Wall (s) | Health |
|---|---|---:|---:|---:|---|---:|---:|---:|---|
| ff+fixed | SHD_ALIF_DFA | 0.0200 | 0.0900 | 0.0012 | [0.0877, 0.0923] | 0.0509 | 1.312e-2 | 103.4 | ok |
| rec+alif | SHD_ALIF_DFA | 0.0200 | 0.1013 | 0.0077 | [0.0863, 0.1164] | 0.0269 | 1.262e-2 | 163.5 | ok |
| rec+fixed | SHD_ALIF_DFA | 0.0200 | 0.0640 | 0.0095 | [0.0455, 0.0825] | 0.0556 | 1.314e-2 | 162.8 | NEAR-COLLAPSED (>95% of predictions in one class) |
| ff+alif | SHD_ALIF_DFA | 0.0200 | 0.0980 | 0.0040 | [0.0902, 0.1058] | 0.0260 | 1.259e-2 | 103.0 | ok |
| ff+fixed | SHD_ALIF_EPROP_CEILING | 0.0200 | 0.2500 | 0.0191 | [0.2126, 0.2874] | 0.0022 | 1.456e-1 | 103.6 | ok |
| rec+alif | SHD_ALIF_EPROP_CEILING | 0.0200 | 0.2980 | 0.0100 | [0.2784, 0.3176] | 0.0022 | 1.507e-1 | 173.6 | ok |
| rec+fixed | SHD_ALIF_EPROP_CEILING | 0.0200 | 0.2640 | 0.0121 | [0.2404, 0.2876] | 0.0020 | 1.439e-1 | 176.8 | ok |
| ff+alif | SHD_ALIF_EPROP_CEILING | 0.0200 | 0.2653 | 0.0094 | [0.2469, 0.2838] | 0.0022 | 1.476e-1 | 107.3 | ok |

### Degeneracy

**1 of 8 cells degenerate** — their accuracies are NOT interpretable as statements about the credit rule: `rec+fixed / SHD_ALIF_DFA / lr=0.02` (NEAR-COLLAPSED (>95% of predictions in one class))

A collapsed, silent or saturated arm scores near chance. Read as a bare number that is indistinguishable from "this credit rule does not work", which would invert the conclusion — hence the explicit health column.

## Ceiling health

| Architecture | DFA | e-prop ceiling | Modulator RMS ratio | Status |
|---|---:|---:|---:|---|
| ff+fixed | 0.0900 | 0.2500 | 11.10 | ok |
| rec+alif | 0.1013 | 0.2980 | 11.94 | ok |
| ff+alif | 0.0980 | 0.2653 | 11.72 | ok |

Parity tolerance 3.50; worst observed 11.94.

## Negative control (shuffled labels)

0.0507 (95% CI [0.0341, 0.0728]); chance 0.0500; threshold 0.1000; **ok**

## Preregistered hypotheses

| ID | Statement | Measured | Verdict |
|---|---|---|---|
| H1 | `rec+alif` DFA beats `ff+fixed` DFA by ≥ 0.10 with disjoint 95% CIs | gain +0.0113 (0.0900 → 0.1013), CIs disjoint = false | INVALID_HARNESS |
| H2 | Best-architecture DFA reaches ≥ 0.50 | 0.1013 | INVALID_HARNESS |

Validity gates: control ok, modulator parity FAILED, H1 cells present ok.

## Published reference points (same dataset, not run here)

| Method | Locality | SHD accuracy |
|---|---|---:|
| BPTT + learned delays (Hammouamri et al. 2023) | none | 0.951 |
| e-prop | local in time, non-local in space | 0.808 |
| ETLP (Quintana et al. 2024) | fully local three-factor | 0.746 |
| this run, best DFA architecture | fully local three-factor | 0.101 |

## Interpretation

**INVALID_HARNESS.** A validity gate failed — the shuffled-label control, the modulator-parity check, or one of the two H1 cells being degenerate. No architecture or locality conclusion may be drawn. Fix the flagged gate and re-run before reading any number above.

## Non-claims

- **Not SOTA** and not a like-for-like ETLP comparison: different eligibility formulation, different schedule, capped splits unless `--full`.
- **Not Gate G2.**
- The eligibility trace omits the exact ALIF `ε_a` cross-term (`binn_learn::shd_alif` module docs). This biases **against** H1. If the adaptive axis shows any effect, implementing that term is the required follow-up.
- An `--lr-sweep` run is a **pilot**. It may inform which learning rate a confirmatory run uses; it may not itself be reported as the confirmatory result.
- `INVALID_HARNESS` blocks every H1/H2 claim; it is not a soft warning.
