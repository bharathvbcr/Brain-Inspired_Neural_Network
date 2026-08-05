# SHD Multi-Seed Scientific Sweep Report

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
