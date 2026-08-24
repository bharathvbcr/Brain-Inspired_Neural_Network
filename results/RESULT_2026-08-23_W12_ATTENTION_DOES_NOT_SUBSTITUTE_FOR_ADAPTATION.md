# Wave 12 — the read-out's advantage is not a stand-in for threshold adaptation

**Registered:** `PREREG_2026-08-22_ADAPTATION_BY_ATTENTION.md`, before any wave-12
cell existed (`ed1e9ba`); analyser frozen before the first cell landed
(`2338d07`).
**Ran:** 2026-08-22/23, 3 × `c7g.16xlarge` spot, pinned binary `22d97c51ab02`.
**Status:** **complete — 24/24 cells, 0 failures, 0 voided, 0 stability notes.**

---

## 1. Why this wave existed

Every cell in waves 1–10 at the anchor is on `ff+fixed`. Nothing in 720 cells
varied the substrate, so the headline **+0.1258** had two readings the record
could not separate: attention *adds* temporal structure no substrate of this kind
can represent, or attention *substitutes* for the threshold adaptation `ff+fixed`
happens not to have.

ETLP's conclusion — that threshold adaptation and a recurrent topology are what a
spiking network needs for rich temporal structure — is quoted in this repository
as the motivation for the architecture ablation. Neither is in `ff+fixed`, and
attention was added instead of either.

## 2. The factorial

| substrate | rate read-out | + attention d32/L4 | gain |
|---|---:|---:|---:|
| `ff+fixed` *(reused, hash-checked)* | 0.7062 | 0.8320 | **+0.1258** |
| `ff+alif` | 0.7018 | 0.8303 | **+0.1285** |

| arm | mean | min | max | seeds ≥ 0.80 |
|---|---:|---:|---:|---:|
| `ff+fixed` *(reused)* | 0.7062 | 0.7005 | 0.7164 | 0/12 |
| `ff+fixed+attn` *(reused)* | 0.8320 | 0.8083 | 0.8472 | 12/12 |
| `ff+alif` | 0.7018 | 0.6908 | 0.7085 | 0/12 |
| `ff+alif+attn` | 0.8303 | 0.8167 | 0.8516 | 12/12 |

## 3. Verdicts

**A-1 — NOT SUPPORTED, and that is the informative direction.** The difference of
gains is **+0.0027** against a two-sided bar of 0.03. Per seed it ranges −0.0314
to +0.0287 and is **positive in 6 of 12** — a coin flip, which is what no effect
looks like.

**Substitution is refuted on the adaptation axis.** Whatever the read-out is
doing, it is not standing in for threshold adaptation.

**A-2 — SUPPORTED.** Attention helps the adaptive substrate: gain **+0.1285**
(bar +0.05), positive in **12 of 12** seeds. It is also positive in 12 of 12 on
`ff+fixed`. The effect does not care whether the substrate adapts.

**A-3 — NOT SUPPORTED, and this is the wave's surprise.** `ff+alif` reaches
**0.7018** against the 0.80 gate, with **0 of 12** seeds over it. Adaptation
alone does not approach the gate.

It does not even improve on `ff+fixed`: per seed, `ff+alif` − `ff+fixed` is
**−0.0044** on average and better in only **3 of 12** seeds. At this operating
point threshold adaptation is inert to slightly harmful.

**A-4 — reported, no verdict.** Highest-scoring arm is `ff+fixed+attn` at 0.8320,
0.0017 above `ff+alif+attn`. No verdict is issued and none may be inferred; the
prereg registered this as descriptive for the reason wave 9 registered M-3 that
way.

**A-5 — SUPPORTED.** Zero non-finite events, zero diverged cells, and — as the
prereg anticipated — no stability notes at all: `ff+alif` has no recurrent
recursion, and its peak gradient norms stayed far below the 1e9 warning tier that
wave 11's recurrent cells cleared by twenty-five orders of magnitude.

## 4. What this changes

The mechanism claim survives contact with a second substrate. Wave 9 established
that **96%** of the read-out's advantage is contingent on temporal order; wave 12
establishes that the advantage is **the same size** whether or not the spiking
layer carries an adaptive threshold. Those are complementary: order is what the
read-out uses, and adaptation is not an alternative route to it.

It also puts a measured number on something the record had only cited. ETLP names
adaptation first among the two things a spiking net needs for rich temporal
structure. At this anchor — h128, `published-2ms`, `adjacent-sum-5`, e400 —
adding it changes nothing. That is a finding about this operating point, not a
refutation of ETLP, whose result is at a different width, rule and budget; but it
does mean the paper can no longer cite adaptation as the obvious untried
alternative. It has now been tried.

## 5. Scope, and what is emphatically not settled

* **Anchor only.** h128, `published-2ms`, `adjacent-sum-5`, e400, d32/L4. Wave 8
  already showed the gain inverts by h1024, so nothing here generalises across
  width.
* **Nothing about recurrence.** The recurrent half was deferred on measurement:
  wave 11 completed 15 of 24 unclipped at h256/e100, and under the campaign's own
  rule an arm with any diverged cell reports zero usable cells, so a 12-seed
  recurrent arm cannot carry a verdict at that completion rate. **A substitution
  result on the adaptation axis does not settle the recurrence axis**, and wave
  11's `rec+alif+attn` at 0.68–0.78 against `rec+alif` at 0.45–0.50 is a hint
  that attention helps there too — at a different width, budget, attention depth,
  with unmatched seeds and no registered comparison. It may not be cited as a
  result.
* **Not calibration.** The instrument stays `Uncalibrated`.
* No comparison to macOS-recorded numbers.

## 6. Provenance

24 new cells from the pinned binary `22d97c51ab02`, which
`RESULT_2026-08-22_SOURCE_REPRODUCES_THE_PINNED_BINARY.md` established is
behaviourally reproduced by the current source on aarch64/glibc — so the reused
controls and the new cells are the same instrument, checked rather than assumed.

The 24 reused `ff+fixed` controls were verified against the wave-1 manifest hash
before any mean was taken; the analysis refuses to report otherwise.

Every verdict was computed once, by an analyser frozen before the first cell
landed, from a complete and settled wave. Three spot instances, ~6 hours, torn
down after collection.
