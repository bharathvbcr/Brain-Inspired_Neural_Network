# Scope: the attention gain is a small-network, one-geometry effect — it inverts by h1024

**Protocol:** `PREREG_2026-08-19_SHD_ATTENTION_CAMPAIGN.md` §3 (wave 3),
registered before any campaign cell ran. Verdicts computed once at n=12.
**Cells:** 216 of 216 (width 96, geometry 24, contract 96), **0 voided**.
**Backend:** rust on Linux/aarch64, binary `22d97c51`.

---

## 1. W3-1 — width. NOT SUPPORTED, and the trend reverses

| hidden | `ff+fixed` | `+attn` | gain |
|---:|---:|---:|---:|
| 128 | 0.7062 | 0.7483 | **+0.0421** |
| 256 | 0.7240 | 0.7390 | +0.0150 |
| 512 | 0.7357 | 0.7400 | +0.0043 |
| 1024 | 0.7386 | **0.7227** | **−0.0159** |

Final doubling, h512 → h1024: attention **−0.017263** against a required ≥ 0.01.
**NOT SUPPORTED.**

The control improves monotonically with width (+0.0178, +0.0117, +0.0029). The
attention arm does not: flat, then *worse*. **By h1024 attention is a
disadvantage.** The gain is a small-network effect.

**A candidate explanation this campaign cannot test.** `d_model` is fixed at 32 at
every width, so at h1024 the block is a 32-dimensional bottleneck summarising 1024
units, and the degradation may track the *ratio* rather than width itself. Wave 2
sweeps `d_model` — but only at h128, so it cannot settle this. Naming it as
untested rather than adopting it.

## 2. W3-2 — geometry. NOT SUPPORTED, and not seed-consistent

At `channels-700`, h128, e400: `ff+fixed` 0.6774, `+attn` 0.7017, gain
**+0.0243** against a required ≥ 0.05. **NOT SUPPORTED.**

More telling than the mean: per-seed gains run **−0.0309 to +0.0729** and are
**not unanimous**. Every other contrast in this campaign is — 12/12 positive
intact, 12/12 negative shuffled, 24/24 diverged in wave 4. The attention arm's sd
is 0.0242 against the control's 0.0057, a four-fold spread.

`channels-700` is the geometry the record names as the binding scope limit on the
0.7378 ceiling, and it was unrun at convergence for any arm before this.

## 3. W3-3 — resolution invariance. SUPPORTED as registered; the interpretation does not follow

| contract | `ff+fixed` | `+attn` | gain |
|---|---:|---:|---:|
| published-2ms | 0.6710 | 0.7769 | +0.1058 |
| published-10ms | 0.6587 | 0.7458 | +0.0871 |
| fixed-t100 | 0.6529 | 0.7893 | +0.1364 |
| fixed-t250 | 0.6670 | 0.7811 | +0.1141 |
| fixed-t500 | 0.6802 | 0.7799 | +0.0996 |

Attention spread **0.0435** > 0.02: **SUPPORTED**. But the control's spread is
**0.0273**, also above the bound, against the recorded macOS value of 0.0034 the
threshold was calibrated against. Attention is only **1.6x** its own control.

So the verdict stands and its intended reading does not:
`DEFECT_2026-08-20_THRESHOLDS_ANCHORED_TO_UNLICENSED_REFERENCES.md`.

What this arm *does* show cleanly is the sample-efficiency result again: at e100
the gain is **+0.087 to +0.136 across all five contracts**, unanimous, because the
control has not caught up at that budget.

## 4. The scope, stated plainly

The attention result holds **at h128, at `adjacent-sum-5`**. It weakens at the
other geometry, is not seed-consistent there, and **inverts by h1024**.

Waves 6 and 7 — the sample-efficiency and convergence-bracket findings that are
the campaign's surviving claim — were measured **entirely at h128 /
`adjacent-sum-5`** and inherit this limit exactly. Any paper must carry it in the
claim itself, not in a limitations paragraph.
