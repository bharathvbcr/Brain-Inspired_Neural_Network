# Amendment — H17-2 merged a four-layer intact arm with a one-layer shuffled control

**Raised:** 2026-08-27, while waves 15–17 were still running, on the first
H17-2 verdict the frozen analyser produced.
**Affects:** `scripts/aws/analyse_wave15.py` only.
**Does not affect:** [`PAPER_DRAFT.md`](PAPER_DRAFT.md) §3.5 or
[`RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md`](RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md),
both of which carry the correct number and are verified below.

---

## 1. What happened

H17-2 extends the paper's mechanism control from n=12 to n=32 by merging the
archived twelve seeds with twenty new ones. The intact arm merges
`r1cal__…__d32l4` with `w17hdl__…__d32l4`. The shuffled arm merged

```
w1__ff-fixed-attn__h128__e400__…__d32l1__bin-shuffled     <- archived twelve
w17hdl__ff-fixed-attn__h128__e400__…__d32l4__bin-shuffled <- twenty new
```

**`d32l1` against `d32l4`.** For the archived twelve seeds the "shuffle cost"
was therefore *(four-layer intact − one-layer shuffled)*, which confounds the
temporal-order effect with a read-out-depth effect. The correct archived control
— `w9shf__…__d32l4__bin-shuffled`, twelve cells, the arm the published result
used — exists in the same corpus and was not the one reached for.

Given that waves 15–17 were in the middle of establishing that read-out depth
matters enormously at h1024, a silent depth mismatch inside the mechanism
control is not a hypothetical contamination.

## 2. Size of the error

Measured at the 28 pairs available when the bug was found, with the wave's
terminal n=32 in the last column:

| archived control | archived 12 pairs | merged 28 pairs | at n=32 |
|---|---:|---:|---:|
| `w1 … d32l1` (as coded) | **+0.1878** | **+0.1577** | — |
| `w9shf … d32l4` (correct) | **+0.1337** | **+0.1345** | **+0.1347** |

The archived twelve were inflated by **+0.0541**, the merged figure by
**+0.0232**. The corrected value is stable as seeds are added: +0.1337 at 12,
+0.1345 at 28, +0.1347 at 32. The mean accuracies are unambiguous: intact `r1cal` d32l4 is
0.8320, shuffled `w9shf` d32l4 is 0.6983, shuffled `w1` d32l1 is **0.6442** —
and that 0.054 is read-out depth, not temporal structure.

**The verdict does not change.** H17-2 was MET as coded and is MET corrected:
+0.1347 against a +0.05 bar, 32/32 positive against a 24/32 bar, 9.5× against
a 5.0× bar at the wave's terminal count. What changes is the reported effect size and the fact that the
evidence behind it was contaminated.

## 3. The published claim is unaffected, and is now confirmed at n=28

`PAPER_DRAFT.md` §3.5 states the attention arm drops **+0.1337** (0.8320 →
0.6983) across 12 of 12 seeds against the plain arm's **+0.0128** (0.7062 →
0.6934), a **10× factor**. Every one of those five numbers reproduces exactly
from the archived cells, from the `d32l4` control. The paper never used the
`d32l1` arm for this contrast.

The corrected merge is the stronger statement, not a retraction: extending from
12 seeds to 32 moves the shuffle cost from **+0.1337 to +0.1347**, every added
seed positive. Two disjoint seed sets agree to 0.001 on the paper's central
mechanism claim.

## 4. Fix, and the class of bug rather than the instance

`merged()` now **requires the archived stem and the new stem to agree on
everything after their wave prefix**, and raises if they do not. Any future
merge of two different configurations fails loudly instead of averaging them.
This is what makes the fix a class fix: the wrong stem was one instance, and
nothing in the analyser had ever checked that a merged arm is one arm.

Both directions are tested — a matched merge must succeed, and the exact
`d32l1`/`d32l4` mismatch must raise. A guard that cannot fire is not a guard.

## 5. Why amending a frozen analyser is legitimate here

The preregistration registers H17-2 as *"(intact − bin-shuffled) for the
attention arm"*. Comparing a four-layer intact arm against a one-layer shuffled
arm is **not that comparison**. This amendment restores the registered
comparison; it does not move a bar, change a threshold, or alter which cells are
admitted. Every threshold in `analyse_wave15.py` is unchanged, and
`scripts/test_wave15_analyser.py` still asserts each of them against this
document's predecessor.

Recorded here rather than fixed silently because a frozen analyser that changes
without a record is no longer frozen.
