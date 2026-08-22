# Wave 9 — the read-out's advantage *is* temporal order, measured at the headline configuration

**Registered:** `PREREG_2026-08-20_SHD_ATTENTION_HEADLINE_SCOPE.md` §8–12
(addendum), before any wave-9 cell existed.
**Ran:** 2026-08-21, 4 × `c7g.16xlarge` spot, pinned binary `22d97c51ab02`.
**Status:** **complete — 24/24 cells, 0 failures, 0 voided.** M-1 and M-2 were
computed from the settled `w9shf` arm; M-3 followed from `w9dim` and, as
registered, carries **no threshold and no verdict**.

---

## 1. Why this wave mattered more than it looked

Wave 8's **S-5 failed**: the temporal-*resolution* prediction reversed. Per the
prereg's named outcomes the paper lost the resolution story, and what remained of
the mechanism claim was the shuffle inversion — **measured at d32/L1, never at the
d32/L4 configuration the paper leads with.**

So going into this wave, the headline had an accuracy and a mechanism inherited
from a different depth. M-1 was the only thing standing between the paper and
reporting a number it could not explain.

## 2. The measurement

All 72 cells (12 new + 60 reused) pass every validity gate, **and every `w9shf`
cell passes the temporal audit** — counts preserved, relocated fraction ≥ 0.5, so
a "shuffle" that failed to shuffle would have been caught rather than scored.

| arm | intact | bin-shuffled | cost of shuffling |
|---|---:|---:|---:|
| **d32/L4** *(the headline)* | **0.8320** | **0.6983** | **+0.1337** |
| d32/L1 *(wave 1)* | 0.7483 | 0.6442 | +0.1041 |
| `ff+fixed` | 0.7062 | 0.6934 | **+0.0128** |

## 3. Verdicts

**M-1 — SUPPORTED.** Intact minus shuffled at d32/L4 = **+0.1337** (bar +0.05),
intact > shuffled in **12 of 12** seeds (bar 10). Per-seed, every value falls
between **+0.0967 and +0.1568** — there is no seed in which the effect is absent
or marginal.

**M-2 — SUPPORTED.** Shuffling costs the attention arm **+0.1337** and the plain
arm **+0.0128**, a factor of **10**. The order sensitivity lives in the read-out,
not in the spiking layer.

## 4. The number that makes the mechanism claim

Destroy temporal order and **the read-out's entire advantage disappears**:

| | intact | bin-shuffled |
|---|---:|---:|
| gain of d32/L4 over `ff+fixed` | **+0.1258** | **+0.0049** |

Under shuffling the attention arm scores 0.6983 against the plain arm's 0.6934.
**96% of the read-out's benefit is contingent on temporal order being intact.**

That is not "order matters." It is: *order is what the read-out is for.* And it
is now measured at the configuration the paper reports, at n=12, with every seed
agreeing.

## 5. What this does to wave 8's S-5

S-5 predicted the gain would **shrink** with a fifth as many timesteps; it grew,
so the verdict stands as **NOT SUPPORTED** and is not revisited here.

What M-1 shows is that S-5 was a **bad proxy** for the mechanism, not a
refutation of it. Fewer timesteps does not mean less order to exploit — 72
positions still carry the sequence. The registered prediction conflated *temporal
resolution* with *temporal order*, and the direct test of order says the
mechanism is intact and stronger at L4 than at L1.

**The paper may claim order. It may not claim resolution.** Both of those are now
measurements rather than readings, and they point in different directions —
which is exactly why each was registered separately.

## 5b. M-3 — reported, deliberately not claimed

| arm | mean | range | seeds ≥ 0.80 | gain over `ff+fixed` |
|---|---:|---|---:|---:|
| d64/L4 | **0.8441** | 0.8185 – 0.8609 | 12/12 | +0.1379 |
| d32/L4 *(the headline)* | 0.8320 | 0.8083 – 0.8472 | 12/12 | +0.1258 |

**mean(d64/L4) − mean(d32/L4) = +0.0121**, with d64 higher in **10 of 12** seeds.

**No verdict is issued, and none may be inferred.** M-3 was registered as
descriptive precisely because a difference of this shape is what a dimension
search produces when it is allowed to name a winner afterwards. d64/L4 was never
a registered hypothesis; +0.0121 across an untested axis is an estimate, not a
finding, and promoting it now would be the exact move the registration exists to
prevent.

What it does license is a **statement of ignorance in the paper**: d32 is the
*tested* configuration, not the chosen one, and there is a hint — not evidence —
that width in the read-out is not yet saturated at 32. Establishing that needs its
own registration, its own bar, and its own run.

Two seeds go the other way (−0.0199, −0.0093), which is itself the reason a
12-seed +0.0121 is not something to build on.

## 6. Scope, unchanged

- Anchor only: h128, `published-2ms`, `adjacent-sum-5`. M-1 was not measured on
  `channels-700` or at h512/h1024.
- **Not calibration.** Criterion 5 unmet.
- No comparison to macOS-recorded numbers; cross-machine Gate F FAILs by design.
- **M-3 is descriptive**: +0.0121 for d64/L4, reported with its sign and its two
  negative seeds, and converted into no claim. See §5b.

## 7. Discipline note

M-1 and M-2 were computed once, from a complete and settled `w9shf` arm, against
reused controls fixed before the wave ran. M-3's cells were still in flight and
**cannot** influence them: it has no threshold, tests a different arm, and enters
no comparison above.
