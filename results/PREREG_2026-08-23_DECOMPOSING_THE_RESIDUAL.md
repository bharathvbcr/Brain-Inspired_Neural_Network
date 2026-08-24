# Preregistration — decomposing the delay-free residual, one variable at a time

**Registered:** 2026-08-23, before any of the runs and before any number exists.

**Follows:** `RESULT_2026-08-23_DELAY_FREE_ABLATION.md`, which measured delays at
0.0348 and left 0.0722 unattributed — and which named the causes wrongly, as its
own correction records.

---

## 1. What is actually different, read rather than inferred

The delay-free reference (`model_type = 'snn'`, 0.9042) against the instrument's
converged attention arm (0.8320), from the effective config:

| # | difference | reference | instrument |
|---|---|---|---|
| 1 | hidden layers | **2** | 1 |
| 2 | dropout | **0.4** | none |
| 3 | hidden units | **256** | 128 (headline arm) |
| 4 | schedule | one-cycle, weight decay 1e-5 | one-cycle (`one_cycle_lr`) |
| 5 | budget | 150 epochs | 400 |
| 6 | binning | `published-10ms` | `published-2ms` |
| 7 | readout | summed, non-spiking (`output_v_threshold = 1e9`) | rate |

`stateful_synapse` and `augment` are both **False** and are not differences at
all; the previous document said otherwise and is corrected.

## 2. What this run does, and what it deliberately does not

**Two ablations, each removing one feature from the delay-free baseline:**

| id | change from `snn` at 0.9042 | isolates |
|---|---|---|
| **A-1** | `n_hidden_layers: 2 → 1` | difference 1 |
| **A-2** | `dropout_p: 0.4 → 0.0` | difference 2 |

Everything else stays exactly as the delay-free run had it, including the seed
(5170001), the 150 epochs, and the clean protocol.

**Differences 3–7 are not run.** Three and five are cheap but confounded with the
instrument's own tuning rather than the reference's; six and seven are not knobs
on this config at all — they would require changing what the reference *is*, not
how it is configured, and a run that changed them would no longer be measuring
this reference. They stay named and untested, which is the honest state.

So this decomposes **part** of the 0.0722, not all of it. Saying so now prevents
the sum of two ablations being read later as the whole residual.

## 3. These are marginal effects, not additive shares

Each ablation removes one feature **in the presence of the others**. Two marginal
effects need not sum to the joint effect of removing both, and if they overshoot
or undershoot 0.0722 that is interaction, not error. **No arithmetic combining
A-1 and A-2 into a total is permitted from this design.**

## 4. Registered thresholds

The clean reference's three-seed spread is 0.0022, so:

- an effect **≥ 0.010** is resolvable and reported as real;
- an effect **< 0.005** is within noise for a single seed and reported as "not
  resolvable at n=1", never as zero;
- between the two, reported as suggestive and requiring three seeds.

## 5. Named outcomes

- **A-1 large (≥ 0.03), A-2 small** → depth is the dominant remaining cause, and
  the instrument's single hidden layer is the thing to change. Note this would be
  a *reference-side* depth effect, and the instrument's own depth suite has never
  been runnable on SHD because of the calibration gate.
- **A-2 large, A-1 small** → regularisation dominates, which the instrument could
  adopt in an afternoon. The cheapest actionable outcome available.
- **both large** → the two together plausibly account for most of the 0.0722, and
  differences 3–7 are lower priority.
- **both small** → the residual is in 3–7, or in something not yet enumerated.
  This is the outcome that would say my enumeration is still incomplete, and it is
  the one worth naming because I have already been wrong once about what the list
  contains.

## 6. Prediction

I expect **A-1 ≈ 0.02–0.05** and **A-2 ≈ 0.01–0.03** — depth mattering more than
dropout, both real. Recorded so it can fail, as the last prediction did.

## 7. Constraints

Diagnostics, not references. Scratch copies outside `results/`; nothing written to
`references/`, `reference-manifests/`, `reference-states/`; `runner.reference()`
not called; `gates.json` not opened. `SHD_INSTRUMENT_STATE` untouched.
