# Amendment — wave 29 is produced on `aarch64-apple-darwin`, not on the fleet

**Date:** 2026-09-07
**Status:** registered **before any wave-29 cell exists**, which is what makes it
an amendment rather than an excuse. The git history is the attestation.
**Amends:** [`PREREG_2026-09-07_W29_THE_ASYMMETRY_HAS_ITS_OWN_BAR.md`](PREREG_2026-09-07_W29_THE_ASYMMETRY_HAS_ITS_OWN_BAR.md) §3
**Frozen with:** `scripts/aws/check_wave29_platform.py`, committed in the same change
**Does not touch:** `scripts/aws/analyse_wave29.py`, which stays frozen as registered

---

## 1. What is amended, in one line

§3 of the registration reads "Same pinned binary." That is amended to: **same
pinned source at `a6acd8d`, built and run on `aarch64-apple-darwin`** rather than
on the campaign's `aarch64-unknown-linux-gnu` fleet.

Nothing else in the registration changes. The anchor, the seed block, the 48
cells, the two one-sided clauses, the bars, the stopping rule and the named
outcomes all stand exactly as registered.

## 2. Why the platform moved

The wave is 263 cell-hours. Running it on the fleet means launching a spot
campaign; running it here means 18 cores that are already paid for. The wave was
registered before it was known which machine would run it, and this is the
machine.

That is a scheduling reason, not a scientific one, which is why the rest of this
document is about what it costs rather than about why it is fine.

The registration's §3 priced the wave from fleet medians: 24 attention cells at
10.82 h and 24 rate cells at 0.14 h, 263 cell-hours in total. Measured on this
host at the same 4 threads per cell, the `ff+fixed` intact cell took **300 s
against the fleet's 506 s median** — this box is about **1.7× faster per cell**,
so the wave is nearer 155 cell-hours here. The registered cost figure is not
amended, because it is a statement about the wave on the fleet and it was
correct; this is a note about the host that ran it.

## 3. The platform difference is real, measured, and documented

[`FINDING_2026-08-19_LIBM_PORTABILITY_OF_REPLAY.md`](FINDING_2026-08-19_LIBM_PORTABILITY_OF_REPLAY.md)
established by direct measurement that `f32` transcendentals are **not**
bit-identical between Apple libm and glibc — `exp`, `ln`, `tanh`, `powf` and the
surrogate sigmoid as written all disagree, across the whole domain rather than at
the extremes. Both hosts are aarch64; this is a libm difference, not an
architecture one.

So a wave-29 cell produced here is **not** the cell the fleet would have
produced. Pretending otherwise would be the largest single defect this document
could contain.

### Production platform, recorded

| | |
|---|---|
| host | Apple M5 Pro, `arm64` Darwin 27.0, 18 cores (6 P / 12 E) |
| target | `aarch64-apple-darwin` |
| rustc | 1.98.0 (`88d9e12ae`, 2026-08-18), LLVM 22.1.8 |
| source | `a6acd8d` |
| binary `sha256` | `fec6c40420953a811cfec284cce31929c565cd7f65c7f18acc7854bf937f86cd` |

This is the same host class as the 2026-08-19 finding, so the divergence
characterised there is the divergence in force here.

## 4. Why every clause survives it anyway

The finding itself states the bound, and it is the bound this wave needs:

> **Bounds A6.** Reference-vs-arm *ordering* measured on one host is a valid
> within-platform comparison, because reference and arms are computed on the same
> box.

Every wave-29 clause is exactly that shape:

| clause | what it compares | crosses platforms? |
|---|---|---|
| sensitivity gate | rate `intact` − rate `p90`, same block | **no** |
| H29-1 | `(attn_i − attn_x) − (rate_i − rate_x)`, same block | **no** |
| H29-2 | the same DiD, opposite sign | **no** |
| H29-3 | DiD(p90) vs DiD(p70) — no p70 cell on this block | not evaluated |

`did()` in the frozen analyser takes only seeds present in **all four** arms, and
all four arms of this wave are produced by one binary on one box. There is no
clause in which a macOS number is subtracted from a fleet number.

This is also precisely why §3 of the registration re-runs the intact arms instead
of reusing `w26pos`: a design that had reused them would now be broken by this
amendment, because the reused controls would be fleet cells. It is not, and the
reason it is not was registered before the platform question arose.

## 5. What it costs, stated rather than waved past

**Absolute accuracies from wave 29 may not be compared to any fleet number**, and
no result document may place a wave-29 accuracy in a table beside a wave-26,
-27 or -28 accuracy without saying which platform produced each. The comparable
object is the DiD, not the accuracy.

**The bars are inherited from a fleet-calibrated wave.** `BAR = 0.03` and
`DROPOUT_MIN_COST = 0.03` were set on fleet cells. They are thresholds on
*differences*, and the evidence that differences transfer is measured rather than
assumed — see §6. If that evidence had gone the other way, the honest move would
have been to re-register the bars, not to run anyway.

**A wave-29 NOT MET is a NOT MET on this platform.** Wave 28's sign was measured
on the fleet; wave 29 measures on macOS. If the two disagree, "the platforms
disagree" is a live explanation alongside "the seed block was the artefact", and
the result document must carry both rather than choosing the more interesting
one.

## 6. What is already known about how far the platform moves *this* measurement

Not a general claim about libm — the specific numbers, at this exact anchor.

The registration's §3 discloses a local single-seed pilot run on this host for
wave 28: `intact 0.7032`, and dropout costs of `p50 0.0225 / p70 0.0384 /
p90 0.1135`. The fleet's own cells at the same anchor, recomputed from the v3
corpus for this amendment:

| quantity | fleet (`aarch64-unknown-linux-gnu`, n=12) | local pilot (n=1) | difference |
|---|---|---|---|
| rate `intact` | 0.7062 (sd 0.0045) | 0.7032 | **0.0030** |
| rate `p90` cost | 0.1080 | 0.1135 | **0.0055** |

**The platform gap on the level is 0.0030, smaller than the fleet's own
seed-to-seed spread of 0.0045.** The gap on the *manipulation cost* — the
quantity every clause is built from — is 0.0055, less than a fifth of the 0.03
bar it will be tested against.

### A second measurement, and the disclosure it requires

While timing this host before the wave was scheduled, one cell of the wave's own
`ff+fixed` intact arm was run at seed `5290001` — a **registered-block seed**,
produced before this amendment was committed. It is disclosed here rather than
quietly kept:

| | fleet (n=12) | local `s5290001` | difference |
|---|---|---|---|
| rate `intact` | 0.7062 (sd 0.0045) | **0.7089** | **+0.0027** |

**+0.0027, against a fleet seed-to-seed sd of 0.0045 and a bar of 0.03.** This
is the same conclusion the wave-28 pilot reached, now on a cell at the wave's own
anchor, on the wave's own seed, from the wave's own binary.

**What that cell is not allowed to do.** No bar in this wave was chosen from it:
`BAR` has been 0.03 since wave 21, `DROPOUT_MIN_COST` is wave 28's H28-1 bar
restated, and §7's platform threshold is that same 0.03 rather than a number
picked to clear this measurement. Nothing was tuned, and nothing could have been
— the analyser was frozen on 2026-09-07 with the registration.

**It is deleted rather than kept.** The cell is discarded and seed `5290001` is
re-run as part of the wave, after this amendment is committed, so that every cell
the analyser reads was produced under a registration that already existed. The
computation is deterministic, so the re-run reproduces `0.7089` exactly; the
point is the order of the commits, not the number.

The two pilots are one seed each and neither is campaign data. Together they are
enough to say the bars are not being carried onto a platform that moves the
measurement by more than the bars are worth. They are **not** enough to say the
platforms agree, and this document does not say that.

For the record, the fleet's full wave-28 anchor, recomputed here and matching the
registration's stated `−0.0382` exactly:

| arm | intact | p90 | cost |
|---|---|---|---|
| `ff+fixed` | 0.7062 | 0.5982 | +0.1080 |
| `ff+fixed+attn` d32/L4 | 0.8320 | 0.7622 | +0.0698 |
| | | **DiD** | **−0.0382** |

## 7. Registered platform gate

A diagnostic, **not** a clause. It bounds how far the platform moved and is
reported whatever it says. It may not void a cell, adjust a number, or change a
verdict — a gate that could do those things would be re-reading the wave against
the fleet, which is the thing §4 exists to prevent.

**Registered check:** the wave's own rate `intact` mean lands within **0.03** —
the campaign's own bar — of the fleet's **0.7062**.

- **Inside 0.03:** the platform moved this measurement by less than the bar every
  clause is tested against. Reported as a number, in the result document.
- **Outside 0.03:** the wave is reported **NOT EVALUABLE on platform grounds**.
  Its cells are kept and its DiD is printed, because a DiD computed within one
  platform is still a valid within-platform comparison — but no verdict is
  entered against H29-1 or H29-2, and the paper does not cite the wave. Then the
  wave is re-registered for the fleet.

Enforced by `scripts/aws/check_wave29_platform.py`, frozen with this document.
The frozen analyser is **not** modified: it was committed with the registration
and adding a gate to it now would mean the analyser that judges the wave is not
the analyser that was registered.

## 8. Corpus isolation

Wave 29's cells land in **`results/shd_attention_wave29_local/`**, not in
`results/shd_attention_campaign_v3/`.

The frozen analyser filters by wave tag and would have been correct either way.
The isolation is structural rather than behavioural: seven scripts read the v3
directory, `check_every_number.py` and `test_paper_number_sweep.py` among them,
and a future analyser that pools across waves would silently mix two platforms in
one mean. Keeping the platforms in separate directories means that mistake
requires someone to type a second `--results` path, instead of requiring every
future reader to remember a filter.

It also leaves the v3 freeze at `(612, 612, d0b83a8c6df0cf71)` untouched: nothing
that was valid becomes invalid, and no archived verdict moves.

## 9. What this amendment does not do

- **It does not re-read wave 28.** Rule §9.3 stands. Wave 28's verdicts stand,
  H28-2 stays **NOT MET**, and no clause here is evaluated against
  `5170001`–`5170012` — the frozen analyser rejects them by value.
- **It does not change a bar, a clause, a seed or the cell count.** 48 cells, two
  one-sided clauses, the stopping rule in §6 of the registration.
- **It does not claim the platforms agree.** It claims, with two measured
  numbers, that the gap at this anchor is small relative to the bar — and
  registers the check that says so before the cells exist.
