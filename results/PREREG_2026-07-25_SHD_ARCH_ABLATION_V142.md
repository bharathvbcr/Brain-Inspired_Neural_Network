# Preregistration amendment — SHD architecture ablation protocol v142

**Registered:** 2026-07-25, before any protocol-v142 run.  
**Supersedes mechanics only:** protocol v141 remains preserved as historical evidence.  
**Binary:** `cargo run --release -p binn-lab --bin shd-arch-ablation`  
**Claim axis:** architecture confound in the `c1-shd-cal-*` result.

Protocol v141 exposed two mechanical validity gaps during its first run:

1. the adaptive arm omitted the preregistered ALIF adaptation eligibility state
   `ε_a`; and
2. learned readout growth made the e-prop transported modulator exceed the
   frozen DFA modulator RMS by more than the preregistered 3.5× limit.

Protocol v142 fixes only those mechanisms:

- every input and recurrent synapse carries `ε_v` and `ε_a`, with
  `ε_v ← α·ε_v + pre`,
  `ε_a ← σ'·ε_v + (ρ − σ'·β_a)·ε_a`, and
  `e = σ'·(ε_v − β_a·ε_a)`;
- fixed-threshold arms use the same implementation with `β_a = 0`;
- DFA and transported e-prop hidden modulators are RMS-normalized to the same
  frozen initialization scale while retaining their direction and the
  output-error norm.

No outcome threshold is changed. The v141 hypotheses and validity gates carry
forward unchanged:

| ID | Statement | Threshold |
|---|---|---|
| H1 | Architecture is the binding constraint | `rec+alif` DFA − `ff+fixed` DFA ≥ 0.10 and disjoint seed-level 95% CIs |
| H2 | Architecture closes most of the gap | best-architecture DFA ≥ 0.50 |
| Negative control | No shuffled-label leakage | accuracy ≤ chance + 0.05 |
| Modulator parity | Comparable effective hidden step | worst DFA/e-prop RMS ratio ≤ 3.5 |

The default confirmatory schedule remains hidden 128, three seeds, 15 epochs,
learning rate 0.02, and capped 2000/500 real-SHD splits. The existing
learning-rate sweep remains pilot-only and cannot be promoted to confirmatory
evidence. Protocol-v141 and protocol-v142 results must be reported separately.
