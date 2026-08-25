# Result — every configurable difference is now measured, and together they do not explain the gap

**Preregs:** `PREREG_2026-08-23_DECOMPOSING_THE_RESIDUAL.md`,
`PREREG_2026-08-23_READOUT_ABLATION.md`,
`PREREG_2026-08-24_BATCHNORM_ABLATION.md`.
**Artifacts:** `results/diagnostics/snn_a{1,2,3,4}_*_s5170001.json`.

---

## 1. The four ablations

All from the delay-free baseline (`model_type = 'snn'`, **0.9042**), seed 5170001,
150 epochs, clean protocol, one variable each.

| id | change | accuracy | Δ | reading against the registered thresholds |
|---|---|---:|---:|---|
| **A-1** | 2 hidden layers → 1 | **0.9187** | **+0.0145** | **real** — the second layer *costs* accuracy |
| **A-2** | dropout 0.4 → 0.0 | **0.8914** | **−0.0128** | **real** — dropout helps |
| **A-3** | summed readout → `spike_count` | **0.9029** | −0.0012 | **not resolvable at n=1** |
| **A-4** | batchnorm on → off | **0.9100** | **+0.0058** | **suggestive** — needs three seeds |

Thresholds as registered, from the 0.0022 three-seed spread: ≥ 0.010 real,
< 0.005 not resolvable, between suggestive.

Separately, from `RESULT_2026-08-23_DELAY_FREE_ABLATION.md`: **learning the delay
positions is worth +0.0348** — with the 25-tap kernel present in every condition
above, including the baseline.

## 2. No total is computed, because the design does not support one

`PREREG_2026-08-23_DECOMPOSING_THE_RESIDUAL.md` §3 registered that these are
**marginal effects in the presence of the others**, and that **no arithmetic
combining them into a total is permitted**. That rule binds here, and it binds in
the direction that would have flattered the analysis: adding the four up would
produce a tidy number, and it would not mean anything.

What can be said without summing: the largest single configurable effect is
delays at +0.0348. The next largest is a hidden layer that **costs** 0.0145. No
single knob approaches the 0.107 gap to the converged attention arm, let alone the
0.198 to the plain arm.

## 3. The reference is carrying choices that hurt it

Two of the four go the wrong way for it, and a third is nil:

- its **second hidden layer costs 0.0145**;
- its **batchnorm looks like it costs 0.0058** (suggestive, one seed);
- its **summed non-spiking readout buys 0.0012**, which is nothing.

Only dropout clearly earns its place. A one-layer, no-batchnorm, delay-free
variant would land at or above 0.9187 — above the reference configuration this
whole comparison is calibrated against.

**That reframes the calibration criterion.** The 0.80 `CELL_PASS` floor was set
from a reference that is not the best version of itself, and three of its choices
are either neutral or harmful. The floor was never derived from what the
architecture *can* do; it was derived from what one configuration happened to
score.

## 4. What this leaves

Every knob on the reference config that could plausibly explain the gap has now
been turned. **Together they account for a fraction of it, and two of them
subtract.** The residual is not in the configuration surface.

What remains is what
`FINDING_2026-08-24_THE_FORWARD_PASSES_DIFFER_IN_KIND.md` identified by reading
the code: a **25-tap `Dcls1d` temporal kernel on every synapse of every layer**,
which the instrument does not have in any form, and which was present in all
five runs above — including the "delay-free" one, where only the tap *positions*
stopped being learned.

**This remains supported by elimination and by code-reading, not by measurement.**
Removing 25 taps per synapse is not a config change; it is a different model. No
run on this config can test it, and after four ablations that is still true. The
distinction must not quietly become "we showed it was the kernel".

**The two claims underneath it are now executable**, in
`scripts/test_shd_calibration.py::ReferenceKernelInvariantTests`:

1. every documented `model_type` — `snn_delays`, `snn_delays_lr0`, `snn` — yields
   the same `max_delay`, so **no configuration removes the kernel**; and all three
   `Dcls1d(` constructions in `snn_delays.py` sit outside any `model_type` branch,
   so the model builds it unconditionally too;
2. `max_delay` is distinct for each of `time_step ∈ {2, 5, 10}`, so **binning
   cannot be varied without moving the kernel width**, with the shipped point
   pinned at 25.

Each is mutation-verified against the change that would make it false: adding a
no-kernel mode, guarding a construction behind `model_type`, and decoupling
`max_delay` from `time_step` each turn the corresponding test red. If upstream
ever does any of those, the conclusion above stops being supported and the tests
say so instead of the record quietly going stale.

## 5. My prediction record in this series

| ablation | predicted | actual | |
|---|---|---|---|
| delays | 0.75–0.85 | 0.9042 | wrong, magnitude |
| A-1 | +0.02–0.05 | +0.0145 | wrong, magnitude |
| A-2 | +0.01–0.03 | −0.0128 | wrong, **sign** |
| A-3 | −0.02–0.05 | −0.0012 | wrong, magnitude |
| A-4 | −0.01–0.04 | +0.0058 | wrong, **sign** |

**Five for five.** Every prediction I registered in this series was wrong, twice
on sign. Recording them was worth more than making them: each was falsified by a
number I would otherwise have been tempted to explain after the fact, and the
outcome sets — which after A-2 always named both signs — are the only reason the
positive results were readable at all.

## 6. What is not established

- **One seed per ablation.** A-4 in particular sits in the suggestive band and
  needs three seeds before it is quoted.
- **These vary the reference, not the instrument.** Every statement about BINN is
  inference from a difference of differences.
- **The temporal kernel is not measured.** See §4.
- **`published-10ms` binning is still untested**, and cannot be tested here:
  `time_step` also drives `max_delay = 250 // time_step`, so changing it moves the
  kernel from 25 taps to 125 at the same time.
