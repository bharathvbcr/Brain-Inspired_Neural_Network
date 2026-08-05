# Matched-Architecture — depth locality vs inflated `readout_only`

**Date:** 2026-07-23 · **Status:** NumPy preview closed (careful interpretation)
**Script:** `scripts/matched_arch_deep.py --exp depth_locality`
**Data:** `deep_depth_locality.json` (strong), `_mid.json`, `_weak.json`
**Does NOT** reopen Rust `c1-*` hashes, remassage spiking DFA, or retune `rl_graded`.

---

## Question

P1 made 2-layer XOR trainable with a **strong** init (`win`×1.5, `w12`×1.8/√h),
but that same init inflates `readout_only` (~0.78). Can we claim C3-style **depth
locality** for DFA / `rl_reinforce_fb` (v12 family), or are those arms just riding
frozen random features?

Acceptance (interpret carefully):

1. Report paired **excess over `readout_only`** (`exRO` / `exROLCB`), not gap-to-chance alone.
2. Sweep init presets (`strong` / `mid` / `weak`) for a regime where BPTT stays
   valid while `readout_only` is near chance.
3. Include `freeze_l1` (train `w12`+readout with DFA; freeze `win`) and broadcast
   contrasts so “depth help” is not confused with “locality required.”

---

## Results (n=12, epochs=90; weak n=8)

| Init | grad | readout | DFA | DFA exLCB | rl_fb | rl_fb exLCB | broadcast | freeze_l1 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| **strong** (P1) | 0.828 | **0.780** | 0.837 | **+0.023** | 0.814 | **−0.016** | 0.758 | 0.823 |
| **mid** | 0.831 | **0.508** | 0.825 | **+0.261** | 0.803 | **+0.236** | 0.816 | 0.801 |
| **weak** | 0.505 | 0.505 | 0.543 | −0.036 | 0.505 | 0.000 | 0.505 | 0.505 |

`rl_flat` stays ~chance under strong/mid (production impoverishment unchanged).

---

## Interpretation (do not overclaim)

1. **Strong init — soft / inconclusive for C3-style depth locality.**
   `readout_only` is inflated. DFA’s excess LCB is only +0.023; `rl_reinforce_fb`
   does **not** clear paired excess LCB. Numbers that look like “DFA ≈ ceiling”
   vs chance are mostly frozen-feature solvability. **Do not cite P1 `deep_depth`
   as a depth-locality PASS.**

2. **Mid init — valid harness; depth credit helps; locality is *not* required.**
   Gradient stays ~0.83 while `readout_only` ≈ chance. DFA and `rl_reinforce_fb`
   both clear large excess — but **`err_broadcast` does too** (~0.82). Unlike
   1-layer XOR (broadcast fails, DFA solves), 2-layer mid-init is a **depth /
   feature-learning** win, not a locality flip. `freeze_l1` ≈ full DFA → teaching
   L2+readout on frozen L1 features is enough.

3. **Weak init — invalid harness** (gradient ~chance; silent deep path). Same
   failure mode P1 fixed; not usable for claims.

4. **Where locality still stands:** 1-layer XOR / `xor_thresh` (P3) remain the
   clean locality-flip evidence. Depth under mid shows hidden learning matters
   without requiring per-neuron feedback.

---

## Reproduce

```bash
python3 -m scripts.matched_arch_deep --exp depth_locality --seeds 12 --epochs 90 \
  --init-preset strong --out results/deep_depth_locality.json
python3 -m scripts.matched_arch_deep --exp depth_locality --seeds 12 --epochs 90 \
  --init-preset mid --out results/deep_depth_locality_mid.json
python3 -m scripts.matched_arch_deep --exp depth_locality --seeds 8 --epochs 90 \
  --init-preset weak --out results/deep_depth_locality_weak.json
```

---

## Verdict

**Closed as careful negative-on-overclaim:** depth is trainable; under the P1
strong init, local rules do **not** justify a C3-style depth-locality claim vs
inflated `readout_only`. Under mid init, DFA / `rl_reinforce_fb` beat readout, but
so does broadcast — locality is not the depth story here.
