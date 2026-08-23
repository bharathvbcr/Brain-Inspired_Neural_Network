# Result — delays are worth 0.035, not 0.236, and my prediction was wrong

**Prereg:** `PREREG_2026-08-23_DELAY_FREE_ABLATION.md`, committed `027d985` before
the run.
**Artifact:** `results/diagnostics/snn_delay_free_ablation_s5170001.json`.

**Registered outcome 2 fires: the record's attribution is wrong.**

---

## 1. The measurement

Same pinned commit, same seed 5170001, same clean protocol, same 150 epochs, same
corpus, same CPU. One variable: `model_type`.

| | accuracy |
|---|---:|
| `snn_delays` (the calibration reference) | **0.9389628343621399** |
| `snn` (delay-free) | **0.9041602366255144** |
| **difference attributable to delays** | **0.0348** |

The three-seed spread of the clean reference is 0.9368–0.9390, a range of 0.0022,
so 0.0348 is roughly sixteen times the seed spread and is resolvable. Wall time
2.48 h against the delays run's 5.53 h — DCLS is where the compute went, which is
a small independent confirmation the mode actually changed.

## 2. My prediction was wrong

I registered **0.75–0.85**, reasoning that delays should matter substantially. The
answer is 0.9042 — above the top of my range and above the 0.88 threshold I had
set for "the record's attribution is wrong".

I expected delays to carry most of the residual. They carry about a seventh of it.

## 3. The record's attribution is refuted

`HANDOFF_2026-08-02.md` §3:

> 0.7151 → delays 0.951 = **architecture cost 0.236**

That labelled the entire residual "delays" without testing delays. Measured, the
residual decomposes the other way round:

| step | accuracy | gap | attributable to |
|---|---:|---:|---|
| instrument, attention arm converged (d32/L4/e400) | 0.8320 | | |
| **delay-free reference** | **0.9042** | **0.0722** | two hidden layers, dropout 0.4, augmentation, stateful synapses — **four untested causes** |
| full reference | 0.9390 | 0.0348 | **delays** |

**The non-delay differences are worth roughly twice what delays are worth.** And
they are four things the instrument could adopt without implementing DCLS at all
— which makes them the cheaper and more informative next targets, the opposite of
what the record's framing implied.

## 4. What this changes

- **"The instrument is far below the reference" is no longer the right summary.**
  Against a delay-free reference of the same lineage the converged attention arm
  is 0.072 behind, not 0.22.
- **Delays are not the explanation.** Any plan that treated implementing delays as
  the route to closing the gap was aimed at the smaller half of it.
- **The four remaining candidates are now the question**, and they are separable:
  layer count, dropout, augmentation and stateful synapses can each be ablated the
  same way, one variable at a time, from the same pinned commit. Each is one run.

## 5. What this does not establish

- **One seed.** Differences below ~0.003 are not resolvable here and none is
  claimed. 0.0348 is well clear of that; a follow-up would want three seeds before
  the number is quoted as settled.
- **It varies the reference, not the instrument.** Every statement about BINN's
  arms in §3 is inference from a difference of differences, not a measurement of
  BINN. The instrument was not touched.
- **The four causes are named, not ranked.** Nothing here says which of layers,
  dropout, augmentation or stateful synapses dominates the 0.0722 — only that
  together they exceed delays.
- **No gate moved.** This was a diagnostic outside `results/shd_instrument_v4/`;
  `references/`, `reference-manifests/` and `reference-states/` were verified
  untouched during the run, and `gates.json` was never opened.
