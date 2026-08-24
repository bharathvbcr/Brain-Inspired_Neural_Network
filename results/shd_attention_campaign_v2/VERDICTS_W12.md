# Wave 12 — adaptation × attention at the anchor

Prereg: `PREREG_2026-08-22_ADAPTATION_BY_ATTENTION.md` §3. 24 new cells, plus 24 reused controls from waves 1 and the registered run, same pinned binary `22d97c51ab02`.

**Validity gates: all 48 cells pass.**

**Stability notes: none — no cell exceeded the recorded peak gradient norm, and no cell was clipped.**

## The factorial

| substrate | rate read-out | + attention d32/L4 | gain |
|---|---:|---:|---:|
| `ff+fixed` *(reused)* | 0.7062 | 0.8320 | **+0.1258** |
| `ff+alif` | 0.7018 | 0.8303 | **+0.1285** |

| arm | mean | min | max | seeds >= 0.80 |
|---|---:|---:|---:|---:|
| `ff+fixed` *(reused)* | 0.7062 | 0.7005 | 0.7164 | 0/12 |
| `ff+fixed+attn` *(reused)* | 0.8320 | 0.8083 | 0.8472 | 12/12 |
| `ff+alif` | 0.7018 | 0.6908 | 0.7085 | 0/12 |
| `ff+alif+attn` | 0.8303 | 0.8167 | 0.8516 | 12/12 |

## Registered verdicts

**A-1** *(primary, two-sided)* the attention gain depends on adaptation: gain(`ff+alif`) **+0.1285** vs gain(`ff+fixed`) **+0.1258**, difference **+0.0027**; bar |Δ| ≥ 0.03 -> **NOT SUPPORTED**
  - Flat: adaptation is not what the read-out's advantage rests on. Substitution is refuted on this axis.

**A-2** attention still helps an adaptive substrate: gain **+0.1285** (bar +0.05), positive in **12/12** seeds (bar 10) -> **SUPPORTED**

**A-3** adaptation alone clears the gate: `ff+alif` mean **0.7018** (bar 0.8), **0/12** seeds >= 0.8 (bar 9); `ff+fixed` was 0.7062 -> **NOT SUPPORTED**

**A-4** *(reported, no verdict)* highest-scoring arm: `ff+fixed+attn` at 0.8320. **No verdict is issued and none may be inferred** — the prereg registers this as descriptive for the reason wave 9's M-3 was: a factorial invites naming a winner after the fact, and that is what registration exists to prevent.

**A-5** stability: every cell passed the validity gate above, which includes `non_finite_events == 0` and completion.

## Scope

- Anchor only: h128, `published-2ms`, `adjacent-sum-5`, e400, d32/L4.
- **Nothing about recurrence.** The recurrent arms were deferred on wave 11's measured completion rate; see the prereg §2.
- Not calibration. No comparison to macOS-recorded numbers.
