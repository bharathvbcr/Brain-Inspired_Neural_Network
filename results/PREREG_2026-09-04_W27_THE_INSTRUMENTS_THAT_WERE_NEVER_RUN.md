# Preregistration — the five instruments wave 26 built and did not run

**Registered:** 2026-09-04, **before any cell of wave 27 exists and before any
instance for it is launched.** Attested by git history: this file,
`scripts/aws/analyse_wave27.py`, `scripts/test_wave27_analyser.py` and the
`w27` plan are committed together, and the first cell is produced afterwards.

**Analyser:** `scripts/aws/analyse_wave27.py`, frozen in the same commit and the
authority on every verdict below.

**Pinned binary:** `434d38c2904e9ac5f75429fc1b2ef0a32ae17b26a9319bbf17c2aab14f0b160a`
— the wave-26 binary, unchanged. Verified, not assumed: no `.rs`, `Cargo.toml`
or `Cargo.lock` file has changed since `97a5d8a`, the commit it was built from.
The wave launches into `binn-campaign-v3-511192439661-us-east-1` under
`launch.py --require-binary-sha256`, which refuses to provision on a mismatch.

---

## 0. Why this wave may reuse where wave 26 could not

Wave 26 re-ran hundreds of arms it already had, because its binary was new and
§0 of [`PREREG_2026-09-03_THE_INSTRUMENT_BEFORE_THE_WAVE.md`](PREREG_2026-09-03_THE_INSTRUMENT_BEFORE_THE_WAVE.md)
forbids pairing a fresh cell against an archived half.

That condition does not hold here. §9.1 of the same document requires **one
binary per wave and every comparison self-contained inside it** — inside the
*binary*, not inside the wave. Wave 27 runs on the binary wave 26 ran on, so
pairing a wave-27 cell against a wave-26 cell satisfies it. The precedent is
wave 25, which paired against `w22cov` arms on its pinned binary.

**Reused, and the analyser prints the keys every run:** `w26pos` rate
`intact`/`bin-shuffled` and attention `intact`/`bin-shuffled` at this anchor,
and the `w26win` ladder rungs below 192. If the binary is ever rebuilt, this
reuse is void and the wave must be re-planned; `--require-binary-sha256` is what
enforces that mechanically.

## 1. The anchor, and the design

Anchor throughout: `h128 / e400 / published-2ms / adjacent-sum-5 / d32L4`,
12 seeds `5170001..5170012`. **DiD(X)** is the campaign's estimator, unchanged:
`(attn_intact − attn_X) − (rate_intact − rate_X)` on seed-paired quadruples.

| group | cells | what runs |
|---|---:|---|
| H27-1 `w27hid` | 24 | `hidden-shuffled`, rate + attn |
| H27-2 `w27drp` | 24 | `spike-dropout-p30`, rate + attn |
| H27-3 `w27lad` | 48 | `window-shuffled` w192 and w256, rate + attn |
| H27-4 `w27qk` | 24 | `qk-norm` read-out, `intact` + `bin-shuffled` |
| H27-5 `w27tau` | 120 | τ ∈ {2.5, 5.0, 10.05, 20.0, 40.0} ms, rate + attn |
| H27-6 `w27spk` | 24 | speakers 3,6 held out of training, rate + attn |
| **total** | **264** | |

## 2. H27-1 — hidden shuffle, and the only exact zero in the campaign

A rate read-out cannot see the hidden train's time axis, so permuting it must
cost that arm **exactly nothing** — byte-identical on every scientific field,
not within a tolerance.

**Registered gate.** Every `w27hid` rate cell must be identical to its `w26pos`
`intact` twin on every field outside a fixed provenance list (timestamps, host,
wall-clock, cell id, and the temporal fields that necessarily differ). **If the
equality fails, every cell of this wave is void** and the analyser exits 4.
That is §3 of the instrument registration, unchanged.

The gate was verified locally on this binary before the wave was planned: 34 of
34 scientific fields identical, accuracy matching to full precision. This runs
it in production, where it is a validity gate on the wave rather than a test.

**DiD(`hidden-shuffled`) is registered as a QUESTION, not a prediction.** The
campaign has no basis for a directional one. It separates what every
input-space operator confounds: removing temporal structure from the data
versus removing the read-out's access to it.

## 3. H27-2 — spike dropout, and what a null under it would mean

**Registered bar, and it is a check on the manipulation, not on the read-out.**
At the anchor, `p30`: the paired **rate-arm** accuracy drop from `intact` is
**> 0.03**, in **≥ 9 of 12** seeds.

Every other registered manipulation preserves per-channel counts exactly, so a
null under all of them is ambiguous between *order does not matter* and *the
measurement is insensitive*. Dropout deletes 30% of spikes from a mask frozen
before the first epoch. If the rate arm does not lose accuracy, the instrument
is insensitive and **no null measured under it is interpretable** — which is a
finding about the instrument and is reported as one.

DiD(`spike-dropout-p30`) is reported and **not barred**.

## 4. H27-3 — the ladder wave 26 could not finish

Wave 26's ladder reached **0.0596** at w128 against a half-maximum of
**0.0604**: τ½ fell off the top by **0.0008**.

**Disclosed:** the choice of *where* to extend — w192 and w256 — was informed by
wave 26's ladder. That is design, and it is stated here rather than discovered
later. **The τ½ estimator and every condition under which it is withheld are
unchanged** from the governing registration: withheld if `DiD(bin-shuffled) ≤
+0.03`, if the ladder is non-monotone beyond ±0.02, or if the half-maximum is
still not reached. No threshold moves.

256 is the last power of two below T — `mean_steps` is 357.9 at this anchor — so
w256 leaves a 102-bin trailing window, permuted at its own length rather than
padded.

## 5. H27-4 — QK-norm, with its motivation withdrawn and said so

§5 of the instrument registration set this up to explain a **collapse-specific**
saturation. [`RESULT_2026-09-04_W26_SATURATION_IS_REAL_AND_NOT_SPECIFIC.md`](RESULT_2026-09-04_W26_SATURATION_IS_REAL_AND_NOT_SPECIFIC.md)
refuted the specificity: `d32l2` saturates too (entropy 0.0801, ‖q‖‖k‖ ×22.97).

**This is therefore a read-out check and not a test of the collapse
hypothesis, and a MET verdict does not resurrect one.** It is still worth
running: it asks whether qk-norm is the same instrument with the score term
bounded, which is a fact about the arm that outlives the motivation.

**Registered h128 bar, unchanged from §5 and both halves in this wave:** the
`qk-norm` arm's accuracy is within **±0.03** of the default arm's, **and**
`DiD(bin-shuffled)` under `qk-norm` is within **±0.03** of the default's.

- **Both inside:** the same instrument with the score bounded.
- **Either outside:** a **different read-out**; nothing measured with it
  transfers to the paper's headline.

## 6. H27-5 — the τ_m ladder, under the standing gate and no exemption

**Registered ladder:** τ ∈ {2.5, 5.0, 10.05, 20.0, 40.0} ms. 10.05 is a rung,
not a baseline appended to the ladder.

**Registered void rule:** a cell whose `saturated_fraction` exceeds **0.05** is
void — `cell_validity.SATURATED_MAX` as it already stands. **A rung with more
than 3 of 12 seeds void is void as a whole**, because a rung reported from its
survivors is a survivorship statistic and the survivors are exactly the seeds
whose initialisation happened to fire less.

**If the long rungs void wholesale, that is the result.** The ladder is reported
as bounded by the standing gate — "τ above X cannot be measured on this arm
without saturating it" — and is **not** re-run under a loosened one. Registered
as a threshold, never as a prediction about which rung trips it: the direction
"longer τ fires more" is false under this campaign's mixed-sign Glorot weights.

**The 10.05 rung is also a cross-wave reproduction check.** `--tau-m 10.05` is
the default path, so those cells must be byte-identical to `w26pos`'s `intact`
arms. The analyser prints the count. A failure there is a reproduction finding
and is reported as one, not quietly absorbed.

## 7. H27-6 — the speaker-held-out split, and what it must not be used for

Speakers 4 and 5 appear **only** in SHD's test split and account for **1,840 of
its 2,264 trials (81.3%)**, so a random hold-out from train contains **zero**
unseen speakers. The instrument refuses `--val-speakers 4,5` outright — a
speaker absent from train cannot be held out of it — which is how this was found.

Held out instead: speakers **3 and 6**, 1,169 of 8,156 trials, leaving **6,987**
for training.

**Registered use: model selection only.** **Registered non-use: no headline
number, no DiD, and no bar.** These cells train on a smaller set and are **not
comparable to the corpus**. Reported, never scored.

## 8. Named outcomes, in every direction

| outcome | reading |
|---|---|
| **H27-1 MET** | The operator separates read-out access from input structure, the wave is valid, and the campaign's only exact zero holds in production. |
| **H27-1 NOT MET** | A rate arm moved under a manipulation it cannot see. **The whole wave is void**, nothing else in it is a result, and the finding is about the instrument or the substrate's determinism. |
| **H27-2 MET** | Dropout bites, so a null measured under it means *order does not matter* rather than *the measurement is insensitive*. §2's ambiguity is resolved. |
| **H27-2 NOT MET** | The instrument is insensitive. No null under it is interpretable, and that is reported as a property of the instrument. |
| **τ½ estimated** | The order the read-out uses has a scale, in milliseconds, and the paper can finally say which. |
| **τ½ still withheld** | The reason is printed. If the half-maximum is still unreached at w256, the order-dependence lives above 512 ms and the ladder is reported as still too short. |
| **H27-4 both inside** | qk-norm is the same instrument with the score term bounded — a fact about the arm, not a rescue of the collapse account. |
| **H27-4 either outside** | A different read-out. Nothing measured with it transfers. |
| **τ rungs void wholesale** | The ladder is bounded by the standing gate and reported as such. No exemption is taken from this document. |
| **H27-6 any result** | Reported as a diagnostic. It cannot become a headline number without a new registration. |

## 9. Cost, from this campaign's own measured medians

Priced at **wave 26's** measured medians, not the archive's — wave 26 overran its
registration 3.3× because it was costed at mixed-load medians while running an
all-attention fleet. Attention h128/`d32l4`/e400 ran **10.78 h**; rate arms
**0.15 h**.

| | |
|---|---:|
| slot-hours | **1,573** |
| ideal packing on 96 slots | 16.4 h |
| expected makespan | 20–24 h |
| **spot cost** | **$86 – $110** at the measured $0.685–0.715/instance-hour |

The τ ladder is 657 of those slot-hours (42%), because it is 60 attention cells.
That is the price of five rungs on both arms and it is stated here rather than
discovered in the bill.

## 10. What this wave does not answer

- **Nothing at h1024.** Every group is at the h128 anchor.
- **The τ ladder is feed-forward only** and takes no exemption from wave 25's
  substrate-scoped one.
- **H27-6 cannot become a headline** under this registration.
- **The gain/DiD dissociation** remains the campaign's leading open problem and
  still has no design.
- **Why `d32l2` saturates**, which wave 26 opened, has no design here either.
- **Nothing here is bit-reproducible off this fleet** — cross-machine Gate F
  fails at a divergence of 0.0049, characterised and unchanged.
