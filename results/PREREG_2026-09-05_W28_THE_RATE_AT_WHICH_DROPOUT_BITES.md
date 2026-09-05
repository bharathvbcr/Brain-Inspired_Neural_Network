# Preregistration — the dropout rate at which the instrument can feel its own hand

**Registered:** 2026-09-05, **before any cell of wave 28 exists and before any
instance for it is launched.** Attested by git history: this file,
`scripts/aws/analyse_wave28.py`, `scripts/test_wave28_analyser.py` and the `w28`
plan are committed together, and the first cell is produced afterwards.

**Analyser:** `scripts/aws/analyse_wave28.py`, frozen in the same commit.

**Pinned binary:** `434d38c2904e9ac5f75429fc1b2ef0a32ae17b26a9319bbf17c2aab14f0b160a`
— the wave-26/27 binary, unchanged. No `.rs`, `Cargo.toml` or `Cargo.lock` file
has changed since `97a5d8a`. Launched with `--require-binary-sha256`, and the
bucket already holds that binary, so the fleet downloads it rather than
rebuilding.

---

## 0. Why this wave exists

[`RESULT_2026-09-05_W27_THE_TIMESCALE_IS_261_MS.md`](RESULT_2026-09-05_W27_THE_TIMESCALE_IS_261_MS.md)
reports **H27-2 NOT MET**: at `p30` the rate arm lost **0.0124**, 41% of the
0.03 it had to lose, in **0 of 12** seeds.

That bar was registered as a check on the *manipulation*, not on the read-out,
and the manipulation failed it. §2 of the instrument registration says why that
matters: every other operator in this campaign preserves per-channel counts
exactly, so a null under all of them is ambiguous between **order does not
matter** and **the measurement cannot see it**. Dropout is the only operator
that breaks count-preservation, and at p30 it could not feel its own hand — so
the ambiguity stayed open.

This wave asks the only question that closes it: **at what rate does dropout
become a manipulation the substrate can feel?**

## 1. Disclosed: the ladder was chosen from a local pilot

Four cells were run locally at this exact anchor, **one seed**, on the **macOS
build** — not the fleet binary, from which it differs by the documented 0.0049:

| | intact | p50 | p70 | p90 |
|---|---:|---:|---:|---:|
| accuracy | 0.7032 | 0.6807 | 0.6648 | 0.5897 |
| drop | — | 0.0225 | **0.0384** | **0.1135** |
| retention | — | 0.5000 | 0.3000 | 0.1000 |
| valid | ok | ok | ok | ok |

**This is not campaign data and no verdict is taken from it.** It is disclosed
because it chose the rungs, and because it settled the design's one real risk:
whether a rate that bites also destroys the substrate. It does not. At p90 the
pilot cell has `silent_fraction` 0.000, `majority_prediction` 0.084, 20 classes
predicted and 0.59 accuracy against a chance level of 0.05. There is a usable
band, and the ladder is placed inside it.

**No bar is moved by this pilot.** H28-1's bar is §2's, unchanged.

## 2. Design

Anchor: `h128 / e400 / published-2ms / adjacent-sum-5 / d32L4`, 12 seeds.
`w26pos` supplies the `intact` arms; `w27drp` supplies the p30 rung. Neither is
re-run — same binary, so §9.1 is satisfied by pairing, as in wave 27.

| group | cells | what runs |
|---|---:|---|
| rate ladder | 60 | `spike-dropout-p{50,60,70,80,90}` × 12 |
| attention | 24 | `spike-dropout-p{70,90}` × 12 |
| **total** | **84** | |

The ladder is swept **wide on the rate arm and narrow on the attention arm**,
because the bar is on the rate arm and rate cells cost 0.15 h against
attention's 10.8 h. p70 is the marginal rung the pilot puts just over the bar;
p90 is the one it puts far above while leaving the substrate healthy.

## 3. H28-1 — the rate at which the substrate loses accuracy

**Registered bar, unchanged from §2:** the paired **rate-arm** accuracy drop
from `intact` is **> 0.03**, in **≥ 9 of 12** seeds.

The analyser reports the drop at every rung and **the smallest rate that clears
the bar**. A rung whose cells are voided by `cell_validity` is reported as void
and cannot count as sensitive: an instrument that works by destroying the
substrate is not an instrument.

## 4. H28-2 — and does the read-out care?

**Dropout deletes spikes; it does not move them. Order survives it.** That is
what makes the pair decisive, and it is why this clause is worth the 24
attention cells.

**Registered prediction:** at every sensitive rate carrying an attention arm,
**|DiD(`spike-dropout-pN`)| < 0.03** — destroying counts while leaving order
intact costs the read-out *less than the campaign's bar*, against **0.1208** for
destroying order.

**Disclosed:** at p30 the DiD was already 0.0015 (7/12), so this prediction is
informed by a measurement that exists. It is registered as a prediction anyway,
because at p30 the instrument was insensitive and that number meant nothing.

**H28-2 is conditional on H28-1, in that order, and the analyser enforces it.**
If no rung is sensitive, H28-2 reads **NOT EVALUABLE** — never NOT MET, and
never quietly reported as a null. Publishing a null measured with an instrument
that cannot see is precisely the ambiguity §2 exists to remove, and an analyser
that did it would be manufacturing the error it was built to prevent.

## 5. Named outcomes, in every direction

| outcome | reading |
|---|---|
| **H28-1 MET, H28-2 MET** | §2's ambiguity is **CLOSED**. Destroying counts costs the read-out nothing while destroying order costs it 0.1208, so its advantage is specific to **order** rather than to spike counts — measured with an instrument now demonstrated to be sensitive. This is the outcome that retires a caveat the paper has carried since §3.5. |
| **H28-1 MET, H28-2 NOT MET** | The read-out **does** depend on spike counts. What it consumes is not order alone, the paper's mechanism claim is broader than it states, and that is the more consequential result of the two. |
| **H28-1 NOT MET at every rung** | Dropout is unusable at this anchor at any rate up to p90. §2's ambiguity is **permanently open** here, no null under a count-preserving operator can be disambiguated by this route, and the paper says so. A finding about the instrument, not about order. |
| **A rung sensitive only where cells void** | The band is empty: the rate that bites is the rate that destroys. Reported as such, and **not** re-run under a loosened validity gate. |

## 6. Cost

Priced at this campaign's measured medians — attention h128 `d32l4` e400 at
10.8 h, rate arms 0.15 h.

| | |
|---|---:|
| slot-hours | **268** |
| fleet | **2** × `c7g.16xlarge` (32 slots) |
| makespan | ~11 h, set by the 24 attention cells running in parallel |
| **spot cost** | **~$15 – 18** |

Two instances, not six. Only 24 cells here are long, and they fit in one round
on 32 slots; a six-box fleet would idle 64 slots for eleven hours and cost three
times as much for the same makespan. Wave 27's lesson was to cost at the right
medians; this one's is to size the fleet to the wave.

## 7. What this wave does not answer

- **Only one manipulation axis.** Dropout is uniform and random over spikes. It
  says nothing about structured deletion, per-channel deletion, or jitter.
- **One anchor, feed-forward, h128.** No claim transfers to h1024 or to the
  recurrent substrate.
- **The attention arm is measured at two rates**, not across the ladder.
- **Nothing here revisits τ½, the h1024 collapse, or the gain/DiD
  dissociation**, which remains the campaign's leading open problem.
- **Nothing here is bit-reproducible off this fleet** — Gate F fails at a
  divergence of 0.0049, characterised and unchanged.
