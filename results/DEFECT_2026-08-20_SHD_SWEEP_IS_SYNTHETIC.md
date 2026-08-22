# `shd-scientific-sweep` never touched SHD

**Found:** 2026-08-20, while working `TODO_2026-08-07_OPEN_WORK.md` §1 item 5
("Port the ceiling-inversion guard into `shd_scientific_sweep`").
**Status:** the TODO item is **superseded**. Porting the guard would have fixed
the wrong thing.

---

## What the report claims

`results/shd_scientific_results.md` is titled *SHD Multi-Seed Scientific Sweep
Report*, the binary's module doc said it *"Evaluates local feedback alignment vs
broadcast vs e-prop ceiling on multi-class temporal audio digits (Spiking
Heidelberg Digits)"*, and the verdict line the binary printed was:

```
PASS — Learns multi-class SHD audio digits!
```

## What it actually ran

`shd_scientific_sweep.rs` does not import `binn-data`, which is the only crate
that can load the corpus. It calls a local generator instead
(`generate_shd_toy_data`, since renamed to `generate_synthetic_frames`):

| | this binary | real SHD |
|---|---:|---:|
| classes | **5** | 20 |
| input channels | **24** | 700 (140 under `adjacent-sum-5`) |
| timesteps | **16** | 358 at `published-2ms` |
| train / test examples | **100 / 50** | 8156 / 2264 |
| source | `Rng::new(seed)` | recorded utterances |

## The task is also trivially separable, and order-free

The generator picks a label, then places every spike in channel
`(label * 3 + rand(3)) % n_in`. With `n_classes = 5` and `n_in = 24` the reserved
channel sets are `{0,1,2}, {3,4,5}, … {12,13,14}` — **disjoint, no wraparound
collision**. So:

- **per-channel spike counts alone determine the label.** Any linear readout on
  rates scores perfectly.
- **the timestep at which each spike lands is drawn uniformly and carries no
  information.** There is nothing temporal to assign credit across.

That is the opposite of what SHD is for. SHD is in this repository *because* it
has temporal structure a rate coder cannot exploit — the entire attention
read-out result (`RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md`) rests on that
property, established by a 12/12 shuffle inversion.

## What this explains

The banner on `shd_scientific_results.md` records the anomaly that prompted the
TODO item: graded DFA at **1.0000** against a True E-prop Ceiling at **0.2140**,
barely over a chance of 0.2000. That was read as a ceiling-inversion defect
needing the `ceiling_inverted` instrumentation ported in.

On this task, a local rule reaching 1.0000 is unremarkable — the task is
linearly separable from rates. The ceiling sitting at chance is still a defect,
and it is the **same defect class** found the same day in
`RESULT_2026-08-20_DEEP_SNN_V134_CEILING_IS_AT_CHANCE.md`: a gradient reference
that does not learn a task the treatment solves in the same process. Two
independent suites, two references at chance.

**Porting the guard would have made the binary print `INVERTED` next to numbers
that were never about SHD.** A correct label on a mislabelled experiment.

## What was changed

Contained, and correct regardless of what is decided next — the binary no longer
describes itself as SHD:

- module doc states plainly that no SHD sample is loaded, with the shape of what
  is;
- `generate_shd_toy_data` → `generate_synthetic_frames`, documented as not a
  stand-in for SHD;
- `EXPERIMENT_NAME` → `"shd-scientific-sweep (SYNTHETIC DATA)"`;
- the emitted report leads with a "**This report is not about SHD**" block
  carrying the actual data shape;
- the verdict string `"PASS — Learns multi-class SHD audio digits!"` →
  `"above chance on the synthetic task — not an SHD result"`.

**Nothing scientific was changed.** No threshold moved, no arm changed, no data
changed. Only the claims the binary makes about itself.

## What was not changed, and why

**The binary was not renamed or retired.** The bin target name reaches
`binn-lab/Cargo.toml`, `scripts/overnight.sh`, `scripts/run_all_experiments.sh`
and several documents; retiring versus renaming versus keeping it as an arm smoke
test is a call for the maintainer, not a side effect of a labelling fix.

**The ceiling guard was not ported.** It would instrument a comparison that
should not be made.

**The report was not regenerated.** It cannot be: the binary calls
`authorize_campaign(CampaignKind::LocalLearning)` and
`instrument_status.rs` refuses it while `SHD_INSTRUMENT_STATE` is `Uncalibrated`.
Verified by running it — exit 3, no output.

## Scope

- **Verified:** every field in the shape table, from `shd_scientific_sweep.rs:95-99`
  and `generate_synthetic_frames`; the disjoint-channel arithmetic; the absence of
  any `binn-data` import; the authorization refusal, by running the built binary.
- **Not verified:** whether `ShdEpropCeiling` is defective in the same way
  `MatchedDeepGradient` is. Both are references at chance, but that is a shared
  symptom, not a shared diagnosis, and neither has been isolated.
- **Not claimed:** anything about the five arms' relative merits. On a task this
  separable there is nothing to rank.
