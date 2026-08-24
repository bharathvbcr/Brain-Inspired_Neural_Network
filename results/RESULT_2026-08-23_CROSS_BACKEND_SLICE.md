# Result — the two backends agree, and the criterion still cannot be met

**Prereg:** `PREREG_2026-08-23_CROSS_BACKEND_SLICE.md`, registered before any cell
of the slice ran.
**Run:** 21 Python cells at `e20`/`h128`, 170 minutes, 21/21 complete, 0 failed.

**Both halves of the outcome matter, and they point opposite ways.**

---

## 1. The agreement half: it holds, everywhere it can be measured

The 21 new cells completed 33 evaluable groups — more than the 7 the slice
targeted, because 80 Python cells were already on disk and the new ones closed
groups those had left open.

**99 seed pairs. Every one agrees within 0.05.**

| | |
|---|---:|
| largest \|python − rust\| across all 99 | **0.009276** |
| typical | ~0.002 |
| exact ties | 3 pairs at 0.000000 |
| pairs outside the 0.05 bar | **0** |

The bar is 0.05 and the worst observation is 0.0093 — a fifth of it. Two
independently written implementations of the same registered configuration
compute the same thing, and this is the **first time that has been measured**
rather than assumed. My registered prediction was agreement, and I noted it
rested on nothing measured; it now rests on 99 pairs.

## 2. The other half: `matrix_verdict` still returns FAIL, and no further cell can change it

Agreement is only one conjunct. `matrix_verdict` also requires every cell in a
group to carry `scientific_status: CELL_PASS`, and:

```
scientific_status across all 317 recorded matrix cells: {'CELL_FAIL': 317}
cells at or above the 0.80 CELL_PASS floor: 0 of 317
highest accuracy of any cell: 0.718198
```

`CELL_PASS` requires `accuracy >= 0.80` (`shd_instrument.rs:953`). **Not one cell
in the matrix reaches it.** The best is 0.7182 — short by 0.082, on the arm and
configuration most favourable to it.

So every one of the 33 groups fails on `CELL_PASS`, not on disagreement. And
because the floor is a per-cell property of the instrument rather than anything
about backends, **running the remaining 115 Python cells cannot change the
verdict.** They would add agreement evidence to groups that are already excluded.

`TODO_2026-08-07_OPEN_WORK.md` §8 asserted that rerunning the Python arm "cannot
change any conclusion". That was right, and it is now demonstrated rather than
asserted — with the reason named: the blocker is the 0.80 floor, not the backends.

## 3. What this says about the instrument

The two implementations agree with each other to about three decimal places, and
**both sit roughly 0.22 below the third-party reference**, which reaches
0.9390–0.9573 on the same corpus
(`RESULT_2026-08-23_REFERENCE_RERUN.md`).

That is exactly the caveat registered in §6 of the preregistration, and it is
worth restating now that it has content: **agreement between two implementations
is agreement, not correctness.** They are consistent with each other and a long
way from the target they are calibrated against. Nothing here diagnoses why — the
gap could be architecture, budget, encoding, or something in the shared data
pipeline that both arms inherit and neither would reveal to the other.

## 4. Cost, and what it bought

170 minutes for 21 cells — 8.1 minutes each, against the Rust arm's ~11–39
seconds for comparable configurations. The record's ≈4.4-day estimate for all 216
is consistent with that.

What it bought, for a tenth of the remaining spend: the first measurement of a
criterion that had sat unmet since the matrix was designed, and a demonstration
that the other 115 cells would buy nothing further.

## 5. What is not claimed

- **`scientific_status` did not move**, and could not: `write_ledger` only calls
  `matrix_verdict` when all 432 cells are complete. It reads `UNCALIBRATED` and
  this run was registered as unable to change that.
- **The deferral is not lifted.** 115 Python cells remain unrun, now with a
  measured reason rather than only an instruction.
- **No claim about correctness**, per §3.
- **`SHD_INSTRUMENT_STATE` is untouched** and every gated binary still exits 2.

## 6. Recommendation

> **Correction, 2026-08-23.** The framing below is wrong and is superseded by
> `FINDING_2026-08-23_THE_MATRIX_GRID_EXCLUDES_ITS_OWN_GATE.md`. The instrument
> does **not** top out at 0.72 — that is the ceiling of the matrix grid, which
> contains no e400, no h1024 and no attention read-out. The instrument reaches
> **0.8821**, and 140 recorded cells carry `CELL_PASS` at or above the 0.80 floor.
>
> The recommendation itself stands, and for a stronger reason: the matrix grid
> excludes every configuration that clears its own gate, so no additional cell of
> any kind can make `matrix_verdict` return `CALIBRATED`.

**Do not run the remaining 115 cells.** They cost ~2.5 days and cannot change
`matrix_verdict`, because every group they would complete fails the same 0.80
floor that all 317 existing cells fail.

The question worth asking instead is why the instrument tops out at 0.72 when the
pinned reference reaches 0.94 on the same data — a gap of 0.22 that no amount of
cross-backend agreement addresses. That is a scientific question about the arm,
and it needs its own registration.
