# SHD Architecture Ablation (C1-SHD-ARCH)

**Protocol version:** 142  
**Experiment:** shd-arch-ablation  
**Preregistration:** `results/PREREG_2026-07-25_SHD_ARCH_ABLATION_V142.md`  
**Mode:** LR-SWEEP PILOT (DFA only, reduced schedule — a pilot, not a confirmatory run)  
**Question:** is DFA ≈ 0.234 on SHD a limit of local credit assignment, or of a feed-forward fixed-threshold forward model?  
**Schedule:** hidden=128, seeds=2, epochs=8, lrs=[0.00125, 0.005, 0.02]  
**Data:** n_train=2000, n_test=500, n_in=700, T=100, classes=20, chance=0.0500, fixture=false  

## Ablation grid

Cells run in H1-critical order (`ff+fixed`, then `rec+alif`), so a truncated run still answers the preregistered contrast. Where several learning rates were run, H1/H2 use the **best non-degenerate cell per architecture**.

| Architecture | Rule | lr | Mean acc | SE | 95% CI (seeds) | Hidden activity | Modulator RMS | Wall (s) | Health |
|---|---|---:|---:|---:|---|---:|---:|---:|---|
| ff+fixed | SHD_ALIF_DFA | 0.0012 | 0.1890 | 0.0070 | [0.1753, 0.2027] | 0.0032 | 9.774e-3 | 64.8 | ok |
| ff+fixed | SHD_ALIF_DFA | 0.0050 | 0.2310 | 0.0010 | [0.2290, 0.2330] | 0.0033 | 9.633e-3 | 63.8 | ok |
| ff+fixed | SHD_ALIF_DFA | 0.0200 | 0.2160 | 0.0040 | [0.2082, 0.2238] | 0.0047 | 1.019e-2 | 63.5 | ok |
| rec+alif | SHD_ALIF_DFA | 0.0012 | 0.1620 | 0.0100 | [0.1424, 0.1816] | 0.0028 | 9.806e-3 | 94.7 | ok |
| rec+alif | SHD_ALIF_DFA | 0.0050 | 0.2210 | 0.0150 | [0.1916, 0.2504] | 0.0029 | 9.636e-3 | 94.5 | ok |
| rec+alif | SHD_ALIF_DFA | 0.0200 | 0.2350 | 0.0070 | [0.2213, 0.2487] | 0.0036 | 9.926e-3 | 94.5 | ok |
| rec+fixed | SHD_ALIF_DFA | 0.0012 | 0.2050 | 0.0010 | [0.2030, 0.2070] | 0.0033 | 9.773e-3 | 91.0 | ok |
| rec+fixed | SHD_ALIF_DFA | 0.0050 | 0.2420 | 0.0100 | [0.2224, 0.2616] | 0.0034 | 9.632e-3 | 91.6 | ok |
| rec+fixed | SHD_ALIF_DFA | 0.0200 | 0.2170 | 0.0190 | [0.1798, 0.2542] | 0.0049 | 1.024e-2 | 92.4 | ok |
| ff+alif | SHD_ALIF_DFA | 0.0012 | 0.1620 | 0.0080 | [0.1463, 0.1777] | 0.0027 | 9.807e-3 | 66.7 | ok |
| ff+alif | SHD_ALIF_DFA | 0.0050 | 0.2320 | 0.0060 | [0.2202, 0.2438] | 0.0028 | 9.637e-3 | 66.1 | ok |
| ff+alif | SHD_ALIF_DFA | 0.0200 | 0.2120 | 0.0020 | [0.2081, 0.2159] | 0.0036 | 9.912e-3 | 66.9 | ok |

### Degeneracy

No cell was degenerate.

A collapsed, silent or saturated arm scores near chance. Read as a bare number that is indistinguishable from "this credit rule does not work", which would invert the conclusion — hence the explicit health column.

## Ceiling health

| Architecture | DFA | e-prop ceiling | Modulator RMS ratio | Status |
|---|---:|---:|---:|---|
| — | — | — | — | no comparable pairs yet |

Parity tolerance 3.50; worst observed 0.00.

## Negative control (shuffled labels)

0.0400 (95% CI [0.0260, 0.0610]); chance 0.0500; threshold 0.1000; **ok**

## Preregistered hypotheses

| ID | Statement | Measured | Verdict |
|---|---|---|---|
| H1 | `rec+alif` DFA beats `ff+fixed` DFA by ≥ 0.10 with disjoint 95% CIs | gain +0.0040 (0.2310 → 0.2350), CIs disjoint = false | UNDERPOWERED |
| H2 | Best-architecture DFA reaches ≥ 0.50 | 0.2350 | UNDERPOWERED |

Validity gates: control ok, modulator parity ok, H1 cells present ok.

## Published reference points (same dataset, not run here)

| Method | Locality | SHD accuracy |
|---|---|---:|
| BPTT + learned delays (Hammouamri et al. 2023) | none | 0.951 |
| e-prop | local in time, non-local in space | 0.808 |
| ETLP (Quintana et al. 2024) | fully local three-factor | 0.746 |
| this run, best DFA architecture | fully local three-factor | 0.235 |

## Interpretation

This is a partial, quick, or learning-rate-pilot schedule. It may verify execution and validity guards, but no H1/H2 scientific verdict is meaningful.

## Non-claims

- **Not SOTA** and not a like-for-like ETLP comparison: different eligibility formulation, different schedule, capped splits unless `--full`.
- **Not Gate G2.**
- The surrogate ALIF eligibility includes both `ε_v` and the adaptation cross-term `ε_a`; fixed-threshold cells use its `β_a = 0` limit.
- An `--lr-sweep` run is a **pilot**. It may inform which learning rate a confirmatory run uses; it may not itself be reported as the confirmatory result.
- `INVALID_HARNESS` blocks every H1/H2 claim; it is not a soft warning.
