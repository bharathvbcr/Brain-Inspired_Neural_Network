# Matched-Architecture Control — preregistration (protocol v4)

**Unit:** C1-MATCH (matched-architecture confound closure)
**Status:** scientific schedule run — **FAIL** (valid ceiling); does not reopen v2
**Config hash:** `c1-match-5dc6822e71229e9e`
**Date drafted:** 2026-07-23
**Author intent:** close residual confound #2 in [`U-NEG_protocol_v2.md`](U-NEG_protocol_v2.md) — *"Shared LIF constants ≠ matched computational graph."*
**Does NOT reopen** hash `c1-118207fbc3eaba53`. This mints a **new** protocol-v4 hash. The v2 kill-gate stands.

---

## 1. The confound this closes

The Gate-G2 gap metric is `gap_closed = (local − dense) / (gradient_reference − dense)`. It is meant to answer one question:

> On the same architecture, how much of what gradients achieve does the local rule recover?

But under protocol v2 the two sides do **not** share a forward graph:

| | `local-assembly` / `dense-local` | `gradient-reference` (`SurrogateLifReference`) |
|---|---|---|
| Input | LatencyEncoder → `force_spike` (2 channels) | continuous frames `x1[t], x2[t]` |
| Forward | event engine, k-WTA (θ=∞ force-select), spiking readout cells | dense recurrent LIF (`win`/`wrec`/`wout`), rate readout |
| Exposure | **one** online pass | **80 epochs** |
| Decision | readout-cell charge comparison | sigmoid on rate logit |
| Update | broadcast three-factor | SuperSpike BPTT |

Because five things differ at once, a low `gap_closed` conflates **"the local rule is too weak"** with **"the local *path* (encoder / k-WTA / single pass) is handicapped."** The v2 FAIL cannot, on its own, distinguish these. This control isolates the rule.

## 2. What the control holds fixed and what it varies

Build a matched pair on **one** forward graph — the dense-LIF forward already implemented in `SurrogateLifReference::forward` (continuous frames, `win`/`wrec`/`wout`, rate readout). Both arms are GC1-exempt `*_baseline.rs` experimental references; neither is a production learner.

Held **identical** across the two arms: forward dynamics, hidden width `n_hidden`, input encoding (continuous frames), rate readout, epoch count, data splits, seed lineage, LIF constants.

**Varies — the only difference:**

- **matched-gradient arm** = existing `SurrogateLifReference` (SuperSpike BPTT).
- **matched-local arm** = new `MatchedLocalReference`: the **production broadcast three-factor rule** ported onto the identical forward. Per-synapse eligibility (surrogate-derivative coincidence trace, decayed by `α`) × a **single broadcast scalar modulator** (reward `±1` from the rate-readout decision) − `λ·w`. No backward graph; O(1) in sequence length; the modulator is one scalar for all synapses, exactly as in production `three_factor::ThreeFactor`.

This makes the pair a true one-variable contrast: **forward graph constant, learning rule swapped.**

## 3. Gate (unchanged thresholds, new hash)

Preregistered PASS for the matched-local arm, identical bar to G2:

| Gate | Requirement |
|---|---|
| Gap LCB | lower 95% bound (z=1.96) on `gap_closed_matched = (matched_local − 0.5) / (matched_gradient − 0.5)` **> 0.5** |
| Accuracy floor | mean matched-local accuracy **≥ 0.65** |
| Harness validity | matched-gradient (the ceiling) mean **≥ 0.65**, else `INVALID_HARNESS` |

`gap_closed` clamped to `[0,1]`; seeds with `(matched_gradient − 0.5) < g2_min_reference_gap` contribute `closed = 0` (same false-PASS guard as v2). n = 20 seeds, protocol version **4**, new experiment name `c1-match` ⇒ fresh config hash (asserted distinct from `c1-118207fbc3eaba53` and the v3 sensitivity hashes).

## 4. Preregistered predictions and their meaning

Prior (consistent with the v2 FAIL and the Hard Audit base rate): **matched-local FAILS.**

- **Outcome A — matched-local FAILS, matched-gradient passes.** The confound is closed *in favor of the negative*: on an identical forward with identical exposure, the broadcast three-factor rule still cannot recover the gradient's accuracy. The v2 FAIL is then attributable to the **rule**, not the spiking-path handicap. This **hardens** U-NEG and sharpens its claim to "the broadcast-scalar credit signal is insufficient on this architecture." Expected outcome.

- **Outcome B — matched-local PASSES (gap LCB > 0.5 and ≥ 0.65).** Then a large part of the v2 FAIL was an **architecture artifact** (LatencyEncoder / k-WTA / single-pass), not the rule. This would be a genuine, important positive: it says the local rule works when not handicapped, and it would **motivate fixing the spiking path** (multi-pass, richer encoder) rather than abandoning the rule. It does **not** retroactively pass `c1-118207fbc3eaba53`; it licenses a new, honestly-scoped effort on the spiking front-end.

Either way the result is decisive about *which* half of the confound carries the v2 negative — which is the whole point.

## 5. Non-claims

- This is **not** a reopen or reinterpretation of `c1-118207fbc3eaba53` (new protocol version, new hash).
- A matched-local PASS is **not** a claim that the *spiking-substrate* local path passes — only that the rule, on the reference's dense forward, recovers the gradient. The substrate path would need its own run.
- Nothing here licenses P3+.
- The matched-gradient arm remains a reference ceiling, never a production learner (v8 rule preserved).

## 6. Python preview (not the scientific verdict)

Before the Rust n=20 run, a faithful NumPy port of the shared forward and both
update rules (`scripts/matched_arch_preview.py`) was run over 8 seeds at the v2
schedule (h=128, 80 epochs, n_train=80). It previews **Outcome A**:

| Arm | Mean acc (8 seeds) | Gate |
|---|---:|---|
| matched-gradient (ceiling) | **0.7875** | — |
| matched-local (three-factor) | **0.4813** | floor needs ≥ 0.65 → **FAIL** |
| gap_closed | mean 0.056, **LCB −0.026** | needs LCB > 0.5 → **FAIL** |

On an **identical** forward graph, with encoder / k-WTA / single-pass handicaps
removed and exposure matched to the gradient arm, the broadcast three-factor rule
still sits at chance while the gradient arm learns. This is the preview signal
that the v2 FAIL is attributable to the **rule**, not the spiking-path
architecture — i.e. the control *hardens* U-NEG. The binding n=20 Rust verdict
under the protocol-v4 hash supersedes this preview.

## 7. Reproduce (Rust scientific verdict)

**Minted hash:** `c1-match-5dc6822e71229e9e` (protocol v4). Full note: [`c1_match.md`](c1_match.md).

| Arm | Mean acc (n=20) | Gate |
|---|---:|---|
| matched-gradient (ceiling) | **0.8963** | harness ≥ 0.65 → **valid** |
| matched-local (three-factor) | **0.5000** | floor needs ≥ 0.65 → **FAIL** |
| gap_closed_matched | mean 0.000, **LCB 0.000** | needs LCB > 0.5 → **FAIL** |

**Verdict: FAIL** (Outcome A). On an identical dense-LIF forward with matched exposure, broadcast three-factor stays at chance while the gradient ceiling learns. Hardens U-NEG toward rule insufficiency on this architecture. Does **not** reopen `c1-118207fbc3eaba53`.

> **The registered hash above is retired; the replay command below is not.**
> This preregistration was registered under `c1-match-5dc6822e71229e9e` and that
> is left as written, because a preregistration records the hash a protocol was
> registered under and rewriting one would falsify the record. But
> `MATCHED_INPUT_SCALE` was not mixed into that hash, so it named two different
> experiments either side of the 2026-08-25 silent-initialisation repair, and
> `from_hash` now returns `None` for it. The replay line was updated on
> 2026-08-29 to `c1-match-6f6000f148f7d30c`, which resolves. **The verdict is
> unchanged on the re-run** — 0.5000 ff / 0.5100 rec, FAIL on both forward
> graphs ([`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md)).

```bash
# quick pilot (not a scientific verdict)
cargo run --locked --release -p binn-lab --bin c1 -- --matched-arch --quick
# full n=20 scientific schedule → writes results note
cargo run --locked --release -p binn-lab --bin c1 -- --matched-arch --out results/c1_match.md
# hash replay
cargo run --locked --release -p binn-lab --bin c1 -- --matched-arch --config-hash c1-match-6f6000f148f7d30c --out results/c1_match.md
```

Run from the `binn/` crate workspace (or monorepo root with `-p binn-lab`).