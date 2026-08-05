# SHD Architecture Ablation (C1-SHD-ARCH)

**Protocol version:** 142  
**Experiment:** shd-arch-ablation  
**Preregistration:** `results/PREREG_2026-07-25_SHD_ARCH_ABLATION_V142.md`  
**Mode:** QUICK smoke  
**Question:** is DFA ≈ 0.234 on SHD a limit of local credit assignment, or of a feed-forward fixed-threshold forward model?  
**Schedule:** hidden=128, seeds=2, epochs=3, lrs=[0.005]  
**Data:** n_train=200, n_test=100, n_in=700, T=100, classes=20, chance=0.0500, fixture=false  

## Ablation grid

Cells run in H1-critical order (`ff+fixed`, then `rec+alif`), so a truncated run still answers the preregistered contrast. Where several learning rates were run, H1/H2 use the **best non-degenerate cell per architecture**.

| Architecture | Rule | lr | Mean acc | SE | 95% CI (seeds) | Hidden activity | Modulator RMS | Wall (s) | Health |
|---|---|---:|---:|---:|---|---:|---:|---:|---|
| ff+fixed | SHD_ALIF_DFA | 0.0050 | 0.0650 | 0.0050 | [0.0552, 0.0748] | 0.0033 | 9.973e-3 | 2.5 | ok |
| rec+alif | SHD_ALIF_DFA | 0.0050 | 0.0650 | 0.0050 | [0.0552, 0.0748] | 0.0029 | 9.945e-3 | 3.8 | ok |
| rec+fixed | SHD_ALIF_DFA | 0.0050 | 0.0600 | 0.0100 | [0.0404, 0.0796] | 0.0034 | 9.975e-3 | 3.8 | ok |
| ff+alif | SHD_ALIF_DFA | 0.0050 | 0.0700 | 0.0100 | [0.0504, 0.0896] | 0.0028 | 9.943e-3 | 2.7 | ok |
| ff+fixed | SHD_ALIF_EPROP_CEILING | 0.0050 | 0.0600 | 0.0000 | [0.0600, 0.0600] | 0.0032 | 9.969e-3 | 2.6 | ok |
| rec+alif | SHD_ALIF_EPROP_CEILING | 0.0050 | 0.0600 | 0.0000 | [0.0600, 0.0600] | 0.0030 | 9.949e-3 | 4.0 | ok |
| rec+fixed | SHD_ALIF_EPROP_CEILING | 0.0050 | 0.0650 | 0.0050 | [0.0552, 0.0748] | 0.0033 | 9.974e-3 | 3.9 | ok |
| ff+alif | SHD_ALIF_EPROP_CEILING | 0.0050 | 0.0650 | 0.0050 | [0.0552, 0.0748] | 0.0029 | 9.946e-3 | 2.7 | ok |

### Degeneracy

No cell was degenerate.

A collapsed, silent or saturated arm scores near chance. Read as a bare number that is indistinguishable from "this credit rule does not work", which would invert the conclusion — hence the explicit health column.

## Ceiling health

| Architecture | DFA | e-prop ceiling | Modulator RMS ratio | Status |
|---|---:|---:|---:|---|
| ff+fixed | 0.0650 | 0.0600 | 1.00 | INVERTED — ceiling below treatment |
| rec+alif | 0.0650 | 0.0600 | 1.00 | INVERTED — ceiling below treatment |
| rec+fixed | 0.0600 | 0.0650 | 1.00 | ok |
| ff+alif | 0.0700 | 0.0650 | 1.00 | INVERTED — ceiling below treatment |

Parity tolerance 3.50; worst observed 1.00.

## Negative control (shuffled labels)

0.0850 (95% CI [0.0481, 0.1623]); chance 0.0500; threshold 0.1000; **ok**

## Preregistered hypotheses

| ID | Statement | Measured | Verdict |
|---|---|---|---|
| H1 | `rec+alif` DFA beats `ff+fixed` DFA by ≥ 0.10 with disjoint 95% CIs | gain +0.0000 (0.0650 → 0.0650), CIs disjoint = false | UNDERPOWERED |
| H2 | Best-architecture DFA reaches ≥ 0.50 | 0.0650 | UNDERPOWERED |

Validity gates: control ok, modulator parity ok, H1 cells present ok.

## Published reference points (same dataset, not run here)

| Method | Locality | SHD accuracy |
|---|---|---:|
| BPTT + learned delays (Hammouamri et al. 2023) | none | 0.951 |
| e-prop | local in time, non-local in space | 0.808 |
| ETLP (Quintana et al. 2024) | fully local three-factor | 0.746 |
| this run, best DFA architecture | fully local three-factor | 0.065 |

## Interpretation

This is a partial, quick, or learning-rate-pilot schedule. It may verify execution and validity guards, but no H1/H2 scientific verdict is meaningful.

## Non-claims

- **Not SOTA** and not a like-for-like ETLP comparison: different eligibility formulation, different schedule, capped splits unless `--full`.
- **Not Gate G2.**
- The surrogate ALIF eligibility includes both `ε_v` and the adaptation cross-term `ε_a`; fixed-threshold cells use its `β_a = 0` limit.
- An `--lr-sweep` run is a **pilot**. It may inform which learning rate a confirmatory run uses; it may not itself be reported as the confirmatory result.
- `INVALID_HARNESS` blocks every H1/H2 claim; it is not a soft warning.
