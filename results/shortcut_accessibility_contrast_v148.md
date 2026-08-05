# Shortcut-accessibility contrast

**Protocol:** v148  
**Hash:** `shortcut-access-v148-953d6f24133cafb6`  
**Schedule:** one paired experiment; 3 fresh seeds; 200/100; 20 epochs; hidden=64; local lr=0.005  
**Verdict:** **PASS — local learning depends on shortcut accessibility**

## Frozen intervention

Both variants use the same four-class multiclass local arm, true shared-forward BPTT reference, initialization, immutable feedback, labels, nuisance realization, seed, and schedule. Variant A adds 16 fixed-total events on the class-indexed channel; variant B is the exact byte-identical-count v144 task at `(jitter=0, distractors=4)`. No variant can be run separately.

| Variant | Channel counts | Raw-rate test | Local train | Local test | BPTT train | BPTT test | Local hidden mean | Local active frac | Local saturated frac | BPTT hidden mean | End local modulator RMS |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| A. rate-accessible | class-dependent; fixed total | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0.0155 | 0.4426 | 0.0000 | 0.2825 | 1.238e-2 |
| B. rate-immune | byte-identical within quartet | 0.2500 | 0.2450 | 0.2500 | 1.0000 | 1.0000 | 0.0064 | 0.0544 | 0.0000 | 0.1113 | 1.260e-2 |

Hidden activity is evaluated on held-out examples after training. `active frac` is the fraction of final hidden rates >= 0.01; `saturated frac` is the fraction >= 0.99. End modulator RMS is evaluated after training over the frozen training set without applying updates.

## Per-seed audit

| Seed | Variant | Raw rate | Local train | Local test | Local classes | Majority | BPTT test | Local hidden mean | Active frac | Saturated frac | End mod RMS | Replay |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| 9100175366735003649 | A. rate-accessible | 1.0000 | 1.0000 | 1.0000 | 4/4 | 0.250 | 1.0000 | 0.0137 | 0.3891 | 0.000 | 1.221e-2 | yes |
| 9100175366735003649 | B. rate-immune | 0.2500 | 0.2250 | 0.2400 | 2/4 | 0.870 | 1.0000 | 0.0063 | 0.0463 | 0.000 | 1.242e-2 | yes |
| 9100175367003439342 | A. rate-accessible | 1.0000 | 1.0000 | 1.0000 | 4/4 | 0.250 | 1.0000 | 0.0163 | 0.4906 | 0.000 | 1.257e-2 | yes |
| 9100175367003439342 | B. rate-immune | 0.2500 | 0.2500 | 0.2500 | 2/4 | 0.960 | 1.0000 | 0.0064 | 0.0398 | 0.000 | 1.276e-2 | yes |
| 9100175367271875039 | A. rate-accessible | 1.0000 | 1.0000 | 1.0000 | 4/4 | 0.250 | 1.0000 | 0.0164 | 0.4481 | 0.000 | 1.237e-2 | yes |
| 9100175367271875039 | B. rate-immune | 0.2500 | 0.2600 | 0.2600 | 3/4 | 0.810 | 1.0000 | 0.0065 | 0.0772 | 0.000 | 1.261e-2 | yes |

Mechanical health: **yes**. Reference health: **yes**. Every evaluation preserved parameter fingerprints: **yes**. The paired task intervention was exact for every seed: **yes**.

## Frozen interpretation

- A local >= 0.80 and B local <= 0.30: shortcut-accessibility finding; positive control passed.
- Both local <= 0.30: multiclass local positive control failed; prior multiclass-local interpretations are void.
- Both local >= 0.80: v144 was a difficulty artifact rather than a rate-shortcut result.
- Any intermediate pattern: stop; do not promote a claim or add a sweep.
