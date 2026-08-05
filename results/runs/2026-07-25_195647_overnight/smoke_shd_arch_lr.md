# SHD Architecture Ablation (C1-SHD-ARCH)

**Protocol version:** 141  
**Experiment:** shd-arch-ablation  
**Preregistration:** `results/PREREG_2026-07-25_SHD_ARCH_ABLATION.md`  
**Mode:** LR-SWEEP PILOT (DFA only, reduced schedule — a pilot, not a confirmatory run)  
**Question:** is DFA ≈ 0.234 on SHD a limit of local credit assignment, or of a feed-forward fixed-threshold forward model?  
**Schedule:** hidden=128, seeds=2, epochs=8, lrs=[0.00125, 0.005, 0.02]  
**Data:** n_train=200, n_test=100, n_in=700, T=100, classes=20, chance=0.0500, fixture=false  

## Ablation grid

Cells run in H1-critical order (`ff+fixed`, then `rec+alif`), so a truncated run still answers the preregistered contrast. Where several learning rates were run, H1/H2 use the **best non-degenerate cell per architecture**.

| Architecture | Rule | lr | Mean acc | SE | 95% CI (seeds) | Hidden activity | Modulator RMS | Wall (s) | Health |
|---|---|---:|---:|---:|---|---:|---:|---:|---|
| ff+fixed | SHD_ALIF_DFA | 0.0012 | 0.0850 | 0.0050 | [0.0752, 0.0948] | 0.0033 | 9.857e-3 | 3.9 | ok |
| ff+fixed | SHD_ALIF_DFA | 0.0050 | 0.0900 | 0.0100 | [0.0704, 0.1096] | 0.0033 | 9.827e-3 | 3.9 | ok |
| ff+fixed | SHD_ALIF_DFA | 0.0200 | 0.1050 | 0.0550 | [0.0000, 0.2128] | 0.0038 | 1.038e-2 | 4.0 | ok |
| rec+alif | SHD_ALIF_DFA | 0.0012 | 0.0900 | 0.0100 | [0.0704, 0.1096] | 0.0029 | 9.863e-3 | 7.0 | ok |
| rec+alif | SHD_ALIF_DFA | 0.0050 | 0.0900 | 0.0400 | [0.0116, 0.1684] | 0.0029 | 9.820e-3 | 6.8 | ok |
| rec+alif | SHD_ALIF_DFA | 0.0200 | 0.1000 | 0.0500 | [0.0020, 0.1980] | 0.0032 | 9.994e-3 | 6.3 | ok |
| rec+fixed | SHD_ALIF_DFA | 0.0012 | 0.0950 | 0.0050 | [0.0852, 0.1048] | 0.0034 | 9.860e-3 | 6.6 | ok |
| rec+fixed | SHD_ALIF_DFA | 0.0050 | 0.1050 | 0.0150 | [0.0756, 0.1344] | 0.0034 | 9.832e-3 | 6.3 | ok |
| rec+fixed | SHD_ALIF_DFA | 0.0200 | 0.1100 | 0.0100 | [0.0904, 0.1296] | 0.0040 | 1.041e-2 | 6.4 | ok |
| ff+alif | SHD_ALIF_DFA | 0.0012 | 0.0750 | 0.0050 | [0.0652, 0.0848] | 0.0028 | 9.862e-3 | 4.1 | ok |
| ff+alif | SHD_ALIF_DFA | 0.0050 | 0.1000 | 0.0300 | [0.0412, 0.1588] | 0.0028 | 9.813e-3 | 3.9 | ok |
| ff+alif | SHD_ALIF_DFA | 0.0200 | 0.1000 | 0.0400 | [0.0216, 0.1784] | 0.0031 | 9.981e-3 | 4.0 | ok |

### Degeneracy

No cell was degenerate.

A collapsed, silent or saturated arm scores near chance. Read as a bare number that is indistinguishable from "this credit rule does not work", which would invert the conclusion — hence the explicit health column.

## Ceiling health

| Architecture | DFA | e-prop ceiling | Modulator RMS ratio | Status |
|---|---:|---:|---:|---|
| — | — | — | — | no comparable pairs yet |

Parity tolerance 3.50; worst observed 0.00.

## Negative control (shuffled labels)

0.0850 (95% CI [0.0481, 0.1623]); chance 0.0500; threshold 0.1000; **ok**

## Preregistered hypotheses

| ID | Statement | Measured | Verdict |
|---|---|---|---|
| H1 | `rec+alif` DFA beats `ff+fixed` DFA by ≥ 0.10 with disjoint 95% CIs | gain -0.0050 (0.1050 → 0.1000), CIs disjoint = false | FAIL |
| H2 | Best-architecture DFA reaches ≥ 0.50 | 0.1000 | FAIL |

Validity gates: control ok, modulator parity ok, H1 cells present ok.

## Published reference points (same dataset, not run here)

| Method | Locality | SHD accuracy |
|---|---|---:|
| BPTT + learned delays (Hammouamri et al. 2023) | none | 0.951 |
| e-prop | local in time, non-local in space | 0.808 |
| ETLP (Quintana et al. 2024) | fully local three-factor | 0.746 |
| this run, best DFA architecture | fully local three-factor | 0.100 |

## Interpretation

**Architecture is NOT the binding constraint.** Recurrence and adaptation moved DFA by only -0.0050 (0.1050 → 0.1000). The feed-forward confound is ruled out and the limit lies in the credit pathway or the eligibility formulation. This is a legitimate negative result and the more interesting one to write up — but it may not be claimed until the two disclosed confounds are eliminated: the missing ALIF `ε_a` term (biases against this arm) and the un-swept learning rate. Run `--lr-sweep` before concluding.

## Non-claims

- **Not SOTA** and not a like-for-like ETLP comparison: different eligibility formulation, different schedule, capped splits unless `--full`.
- **Not Gate G2.**
- The eligibility trace omits the exact ALIF `ε_a` cross-term (`binn_learn::shd_alif` module docs). This biases **against** H1. If the adaptive axis shows any effect, implementing that term is the required follow-up.
- An `--lr-sweep` run is a **pilot**. It may inform which learning rate a confirmatory run uses; it may not itself be reported as the confirmatory result.
- `INVALID_HARNESS` blocks every H1/H2 claim; it is not a soft warning.
