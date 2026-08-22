# SHD Multi-Seed Scientific Sweep Report

> **WITHDRAWN — 2026-08-20. This report is not about SHD.** The binary that
> produced it never loads the Spiking Heidelberg Digits corpus. It fabricates its
> own data: 5 classes, 24 channels, 16 timesteps, 100 train / 50 test per seed,
> with each label firing only in its own three reserved channels — **linearly
> separable from per-channel spike counts, with no temporal structure**.
>
> No number in this report is evidence about SHD, temporal credit assignment, or
> locality. The DFA 1.0000 below is unremarkable on a task this separable; the
> e-prop ceiling at 0.2140 against chance 0.2000 is a reference that does not
> learn, the same defect class as
> [`RESULT_2026-08-20_DEEP_SNN_V134_CEILING_IS_AT_CHANCE.md`](RESULT_2026-08-20_DEEP_SNN_V134_CEILING_IS_AT_CHANCE.md).
>
> See [`DEFECT_2026-08-20_SHD_SWEEP_IS_SYNTHETIC.md`](DEFECT_2026-08-20_SHD_SWEEP_IS_SYNTHETIC.md).
> The 2026-08-07 ceiling-inversion banner is retained below; its prescribed fix
> (port the guard, re-run) is superseded.
>
> ---

> **CEILING INVERTED — DO NOT CITE THE DFA ROW.** Graded DFA reports **1.0000 ±
> 0.0000** while the True E-prop Ceiling reports **0.2140 ± 0.0294** against a
> chance level of **0.2000**. The reference arm is at chance while the treatment
> is perfect.
>
> Unlike the other stale reports in this directory, this one **matches its
> source** (both v135), which makes it the genuinely unfixed instance. The
> binary's entire verdict logic is a per-arm `chance + 0.05` check
> (`shd_scientific_sweep.rs:202-214`): no ceiling-inversion check, no gap-closed
> computation, no modulator-scale recording.
>
> The mechanism is already hypothesised in this codebase.
> `deep_snn_scaling.rs:18-20` names **this suite** as the worked example: *"A
> ceiling whose credit signal is orders of magnitude weaker than the treatment's
> is not a ceiling — that is exactly how the SHD suite produced a 'ceiling' below
> its own treatment."* The instrumentation that would test it
> (`dfa_modulator_rms`, `eprop_modulator_rms`, `modulator_rms_ratio`,
> `ceiling_inverted`) was added to `ShdCalReport`, a different binary.
>
> See `AUDIT_2026-08-07_JULY_CAMPAIGN_SCORING_PATH.md` §4 and
> `TODO_2026-08-07_OPEN_WORK.md` §1.

**Protocol Version:** 135 (`shd-scientific-sweep` — **5-class**; chance = 0.20)  
**Experiment:** shd-scientific-sweep  
**Schedule:** FULL SCIENTIFIC (n=10, classes=5)  

```
claim_axis: exploratory appendix (not MUST; not Gate G2)
protocol_label: proto-135 / 5-class SHD sweep
do_not_mix_with: overnight C1-SHD-CAL p27 (20-way, chance 0.05; hashes c1-shd-cal-eb3cb5d93417a638 / c1-shd-cal-bafa6835d8de7eb8)
must_not_claim: neuromorphic SOTA; full-corpus SHD; Gate G2 reinterpretation; drop-in SuperSpike match
```

## SHD Accuracy Summary (Mean ± SE vs Chance=0.2000)

| Arm | Mean Accuracy | SE | Beats Chance (0.20)? |
|---|---:|---:|---|
| Broadcast ±1 Three-Factor | 0.2840 | 0.0398 | ✓ |
| Graded DFA | 1.0000 | 0.0000 | ✓ |
| Frozen REINFORCE×B_i | 0.6780 | 0.0568 | ✓ |
| **Online Learned FB Alignment** | **0.6680** | **0.0527** | **✓** |
| True E-prop Ceiling | 0.2140 | 0.0294 | ✓ |

## Verdict

- Online Learned FB Alignment: **exploratory above-chance** on this 5-class proto-135 schedule — **not** a MUST Gate G2 / 20-way p27 claim.
