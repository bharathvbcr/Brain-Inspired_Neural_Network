# SHD Architecture Ablation (C1-SHD-ARCH)

**Protocol version:** 142  
**Experiment:** shd-arch-ablation  
**Preregistration:** `results/PREREG_2026-07-25_SHD_ARCH_ABLATION_V142.md`  
**Mode:** CAPPED scientific (2000/500)  
**Question:** is DFA ≈ 0.234 on SHD a limit of local credit assignment, or of a feed-forward fixed-threshold forward model?  
**Schedule:** hidden=128, seeds=3, epochs=15, lrs=[0.005]  
**Data:** n_train=2000, n_test=500, n_in=700, T=100, classes=20, chance=0.0500, fixture=false  

## Ablation grid

Cells run in H1-critical order (`ff+fixed`, then `rec+alif`), so a truncated run still answers the preregistered contrast. Where several learning rates were run, H1/H2 use the **best non-degenerate cell per architecture**.

| Architecture | Rule | lr | Mean acc | SE | 95% CI (seeds) | Hidden activity | Modulator RMS | Wall (s) | Health |
|---|---|---:|---:|---:|---|---:|---:|---:|---|
| ff+fixed | SHD_ALIF_DFA | 0.0050 | 0.2313 | 0.0055 | [0.2206, 0.2420] | 0.0034 | 9.510e-3 | 186.2 | ok |
| rec+alif | SHD_ALIF_DFA | 0.0050 | 0.2213 | 0.0135 | [0.1949, 0.2478] | 0.0028 | 9.512e-3 | 270.3 | ok |
| rec+fixed | SHD_ALIF_DFA | 0.0050 | 0.2393 | 0.0055 | [0.2286, 0.2500] | 0.0035 | 9.502e-3 | 267.6 | ok |
| ff+alif | SHD_ALIF_DFA | 0.0050 | 0.2313 | 0.0219 | [0.1885, 0.2742] | 0.0027 | 9.510e-3 | 189.3 | ok |
| ff+fixed | SHD_ALIF_EPROP_CEILING | 0.0050 | 0.2193 | 0.0134 | [0.1931, 0.2456] | 0.0025 | 9.524e-3 | 185.0 | ok |
| rec+alif | SHD_ALIF_EPROP_CEILING | 0.0050 | 0.1493 | 0.0282 | [0.0942, 0.2045] | 0.0033 | 9.704e-3 | 274.4 | ok |
| rec+fixed | SHD_ALIF_EPROP_CEILING | 0.0050 | 0.2500 | 0.0061 | [0.2380, 0.2620] | 0.0026 | 9.522e-3 | 277.9 | ok |
| ff+alif | SHD_ALIF_EPROP_CEILING | 0.0050 | 0.1453 | 0.0232 | [0.0998, 0.1909] | 0.0033 | 9.700e-3 | 199.8 | ok |

### Degeneracy

No cell was degenerate.

A collapsed, silent or saturated arm scores near chance. Read as a bare number that is indistinguishable from "this credit rule does not work", which would invert the conclusion — hence the explicit health column.

## Ceiling health

| Architecture | DFA | e-prop ceiling | Modulator RMS ratio | Status |
|---|---:|---:|---:|---|
| ff+fixed | 0.2313 | 0.2193 | 1.00 | INVERTED — ceiling below treatment |
| rec+alif | 0.2213 | 0.1493 | 1.02 | INVERTED — ceiling below treatment |
| rec+fixed | 0.2393 | 0.2500 | 1.00 | ok |
| ff+alif | 0.2313 | 0.1453 | 1.02 | INVERTED — ceiling below treatment |

Parity tolerance 3.50; worst observed 1.02.

## Negative control (shuffled labels)

0.0513 (95% CI [0.0357, 0.0751]); chance 0.0500; threshold 0.1000; **ok**

## Preregistered hypotheses

| ID | Statement | Measured | Verdict |
|---|---|---|---|
| H1 | `rec+alif` DFA beats `ff+fixed` DFA by ≥ 0.10 with disjoint 95% CIs | gain -0.0100 (0.2313 → 0.2213), CIs disjoint = false | FAIL |
| H2 | Best-architecture DFA reaches ≥ 0.50 | 0.2213 | FAIL |

Validity gates: control ok, modulator parity ok, H1 cells present ok.

## Published reference points (same dataset, not run here)

| Method | Locality | SHD accuracy |
|---|---|---:|
| BPTT + learned delays (Hammouamri et al. 2023) | none | 0.951 |
| e-prop | local in time, non-local in space | 0.808 |
| ETLP (Quintana et al. 2024) | fully local three-factor | 0.746 |
| this run, best DFA architecture | fully local three-factor | 0.221 |

## Interpretation

**Architecture is NOT the binding constraint under protocol v142.** Recurrence and adaptation moved DFA by only -0.0100 (0.2313 → 0.2213). The feed-forward confound is ruled out and the limit lies in the credit pathway or the eligibility formulation. The ALIF adaptation cross-term is included. This is a legitimate negative result only at the preregistered learning rate; use the separately declared pilot to decide whether a fresh held-out confirmation is warranted.

## Non-claims

- **Not SOTA** and not a like-for-like ETLP comparison: different eligibility formulation, different schedule, capped splits unless `--full`.
- **Not Gate G2.**
- The surrogate ALIF eligibility includes both `ε_v` and the adaptation cross-term `ε_a`; fixed-threshold cells use its `β_a = 0` limit.
- An `--lr-sweep` run is a **pilot**. It may inform which learning rate a confirmatory run uses; it may not itself be reported as the confirmatory result.
- `INVALID_HARNESS` blocks every H1/H2 claim; it is not a soft warning.
