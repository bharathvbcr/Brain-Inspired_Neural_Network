# Finding — the h1024 collapse is a lost fit, not overfitting

**Date:** 2026-08-29.
**Evidence:** cells already on disk. **No compute was spent.**
**Script:** [`scripts/fit_retention.py`](../scripts/fit_retention.py), tested by
`scripts/test_fit_retention.py` (13 tests).
**Status: POST-HOC.** This is analysis of cells that already existed. It is
**not** a registered verdict, must not be transcribed as one, and the
preregistration it motivates is
[`PREREG_2026-08-29_THE_COLLAPSE_IS_LATE.md`](PREREG_2026-08-29_THE_COLLAPSE_IS_LATE.md).

---

## 1. The question it answers

[`PAPER_DRAFT.md`](PAPER_DRAFT.md) §3.5 calls the h1024 gain inversion
**"located but unexplained"**, and the wave-21 preregistration registers it as
the paper's leading open problem. One alternative account — overfitting on
8,156 training samples — is recorded as **"neither excluded nor supported"**,
because the argument for it was conditional on a collapse that did not occur.

It was answerable without spending anything. Every cell carries
`epoch_mean_loss`, the **training** loss per epoch, and the two accounts make
opposite predictions about it:

| account | training loss at the end |
|---|---|
| **overfitting** — fits the training set, generalises badly | **low** |
| **lost fit** — reaches a fit and does not hold it | **high, and above its own best** |

## 2. What the cells say

`ff+fixed+attn`, `h1024`, `e400`, `published-2ms`, `adjacent-sum-5`. "Lost"
counts cells whose final training loss — the mean of the last ten epochs —
exceeds **three times their own best**.

| read-out | n | test acc | best train loss | final train loss | lost |
|---|---:|---:|---:|---:|---:|
| rate (intact) | 32 | 0.7388 | 0.0273 | 0.0275 | **0/32** |
| `d32l1` | 20 | 0.7230 | 0.0003 | 0.0005 | **0/20** |
| `d32l2` | 32 | 0.7788 | 0.0001 | 0.0003 | 4/32 |
| `d32l3` | 20 | 0.7759 | 0.0018 | 0.0085 | 3/20 |
| **`d32l4`** | **68** | **0.5730** | **0.0343** | **1.9277** | **63/68** |
| `d32l4` (bin-shuffled) | 12 | 0.4592 | 0.1314 | 7.1121 | **12/12** |

**The overfitting account is excluded.** `d32l4` does not end with a low
training loss — it ends **56× above its own best**, while every arm that
generalises ends within a factor of three of its own. An arm that overfits
holds its fit; this one does not.

Per seed at `w18dep` (20 seeds), the fit is reached at **epoch 39–99** in 15 of
the 17 that lose it, and — the part that makes the reading sharp — **the three
seeds that hold the fit carry the three highest test accuracies in the set**:
0.7412, 0.7518 and 0.7703, against the arm's 0.6071 mean. Where the fit
survives, so does the accuracy.

## 3. What follows, and what does not

**Follows.** The collapse is a **late-training** phenomenon at a configuration
that fits perfectly well early. That is a different object from "h1024/d32l4
cannot learn", which is what the gain column alone suggests, and it has a
different remedy.

**Does not follow — and this is the limit that matters.** Nothing here says
*why* the fit is lost, and the register already refused the obvious candidate:
H15-1 declined the gradient-norm account because `d32l3` sits above the
registered sickness threshold at 1.347 and gains anyway
([`RESULT_2026-08-28_W18_19_THE_DEPTH_OPTIMUM_IS_INTERIOR.md`](RESULT_2026-08-28_W18_19_THE_DEPTH_OPTIMUM_IS_INTERIOR.md)).
**"Located but unexplained" stands.** What changes is that one of the two named
alternatives is now excluded rather than open, and the remaining question is
narrower.

**A scope note the record did not have.** The h1024 difference-in-differences —
**+0.1122** in 10 of 12 seeds, the number that made wave 21's registered
prediction fail — is computed between two arms that are **both** losing their
fit: intact `d32l4` at 63/68 and bin-shuffled `d32l4` at 12/12. That does not
invalidate the DiD, which is a paired difference and does not require either
arm to be healthy. It does mean the h1024 row of Table SHD-2b is a contrast
between two late-collapsed arms, and any reading of it as "the mechanism is
intact at h1024" is claiming more than the cells support.

## 4. Why this was worth doing before spending

The wave authorized for h1024 would otherwise have re-measured the collapse.
It now tests a **named prediction with a falsifier**: if the collapse is late,
stopping early should avoid it. No `e100` or `e200` cell exists at
`h1024/d32l4`, so that does need compute — but for a registered question rather
than a fishing expedition.

## 5. Reproduce

```bash
python3 scripts/fit_retention.py --width 1024
python3 scripts/test_fit_retention.py
```

The script refuses to summarise fewer than 20 cells, so a narrowed corpus fails
rather than reporting a confident table over a slice.
