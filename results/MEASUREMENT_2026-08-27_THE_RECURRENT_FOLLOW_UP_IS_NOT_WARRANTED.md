# The recurrent-stability follow-up wave is not warranted, and the record already says so

**2026-08-27.** Wave 20 answered whether the recurrent arm is usable — it is,
at 24 seed-paired comparisons against a floor of 24. The follow-up wave named
repeatedly as "the clear next step" was to be registered once that verdict
landed. It should not be, and this document is why.

## The levers are already tested, each with a registered verdict

| lever | verdict | source |
|---|---|---|
| **gradient clipping** | **harmful.** Clipping *caused* the divergence: 0 of 24 became **15 of 24** with the flag removed, and a paired local control on byte-identical initial weights overflowed at optimiser step 244 clipped and completed 100 epochs unclipped | [`RESULT_2026-08-22_W11_CLIPPING_WAS_NOT_THE_WHOLE_CAUSE.md`](RESULT_2026-08-22_W11_CLIPPING_WAS_NOT_THE_WHOLE_CAUSE.md) |
| **surrogate scale** | **0.4 is the stabilising setting.** `rec+alif` completes 11/12 at 0.4 against 8/12 at 1.0 | [`RESULT_2026-08-23_W13_RECURRENT_STABILITY.md`](RESULT_2026-08-23_W13_RECURRENT_STABILITY.md) R-1 |
| **adaptation** | **stabilises.** `rec+alif` 19/24 against `rec+fixed` 12/24, +7 on a two-sided bar of 6 — *"with the sign opposite to the hypothesis's own name"* | same, R-2 |

Wave 20 then ran thirty-two seeds at exactly that operating point — scale 0.4,
adaptation on, unclipped — and got 27/32 and 29/32 valid. There is no untested
lever left to register, and a wave that re-asked any of the three would be
re-asking a question the record answers.

## A wrong reading that this document exists to correct

An ad-hoc sweep of the corpus counted **result files** rather than **valid
cells** and produced this:

> `rec+fixed` completes 100% at ss0.4 where `rec+alif` completes 84% — so
> divergence is associated with adaptation.

That is backwards, and it pointed straight at a wave whose hypothesis wave 13
had already refuted. `rec+fixed` writes a result file and then **fails the
validity gate**: five of its twelve cells at scale 0.4 are voided by saturation.
Applying `scripts/cell_validity.py`, which is what every frozen analyser uses:

| arm | scale | valid | voided | diverged | usable |
|---|---|---:|---:|---:|---:|
| `rec+alif` | 0.4 | 27 | 0 | 5 | **84%** |
| `rec+alif` | 1.0 | 8 | 0 | 4 | 67% |
| `rec+alif+attn` | 0.4 | 29 | 0 | 3 | **91%** |
| `rec+fixed` | 0.4 | 7 | **5** | 0 | **58%** |
| `rec+fixed` | 1.0 | 5 | **5** | 2 | 42% |

`rec+fixed` never diverges at scale 0.4 and is still the least usable arm on the
table. **A cell that finished is not a cell that counts**, and the two failure
modes are different: `rec+alif` diverges loudly and `rec+fixed` saturates
quietly. Counting files sees only the loud one.

The committed tooling does not make this mistake — every `analyse_wave*.py`
gates on `cell_validity.py`, and wave 20's analyser reported "27 valid of 32"
rather than a file count. The error was in a one-off sweep, which is exactly
where it is easiest to make and hardest to notice.

## What the wave-20 stability warnings do and do not license

Wave 20's analyser flagged **28 cells with peak gradient norms above anything in
the recorded campaign** (max 1.13e8), ranging to 1e35 — one within five orders
of f32 overflow. That is a real measurement and it is why the arm's completion
rate is what it is.

It is **not** a lever question. The prereg is explicit that gradient magnitude
never voids a cell here, because it is the covariate H20-3 is measured on and
voiding on it would decide the survivorship question by construction. H20-3 came
back **ρ = −0.274** against a bar of −0.30: among the cells that complete, the
ones closest to diverging do **not** show systematically smaller gains, so the
extreme norms are not silently shaping the result they sit beside.

## What remains open on this arm

Nothing that a lever can reach. The honest statement is that this operating
point completes 84–91% of the time, that the campaign has found the setting that
maximises that among the three tested, and that
[`PAPER_DRAFT.md`](PAPER_DRAFT.md) §3.7's limits already say so. A fourth lever
would need a mechanism nobody has proposed, and inventing one to justify a wave
is the failure mode the amendment of 2026-08-26 was written about — a lever
chosen for a rationale that does not hold for the arm it is applied to.
