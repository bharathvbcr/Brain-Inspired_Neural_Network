# Result — depth *hurts* the reference, and my outcome set had no room for that

**Prereg:** `PREREG_2026-08-23_DECOMPOSING_THE_RESIDUAL.md`, committed `7df3029`
before both runs.
**Artifacts:** `results/diagnostics/snn_a1_layers_s5170001.json`,
`snn_a2_dropout_s5170001.json`.

---

## 1. The measurements

Seed 5170001, clean protocol, 150 epochs, one variable each from the delay-free
baseline.

| config | accuracy | effect vs baseline |
|---|---:|---:|
| `snn_delays` (calibration reference) | 0.9390 | delays **+0.0348** |
| **delay-free baseline** (`snn`) | **0.9042** | — |
| **A-1** `n_hidden_layers: 2 → 1` | **0.9187** | **+0.0145** |
| **A-2** `dropout_p: 0.4 → 0.0` | **0.8914** | **−0.0128** |
| instrument, converged attention arm | 0.8320 | |

Both effects clear the registered 0.010 resolvability bar — 6.6× and 5.8× the
0.0022 three-seed spread respectively.

## 2. My prediction was wrong on sign, and my outcome set could not express it

I predicted **A-1 ≈ 0.02–0.05**, meaning removing the second layer would *cost*
that much. Removing it **gained 0.0145**. The second hidden layer makes the
delay-free reference **worse**.

Worse than a wrong magnitude: **all four of my named outcomes assumed positive
effects.** "A-1 large, A-2 small → depth is dominant"; "both large → they account
for most of the residual". None of them has a branch for *depth being a negative
contributor*, so none fits cleanly.

That is a defect in the preregistration, not in the data. I registered magnitude
thresholds and silently assumed sign — the same class of error as registering a
two-sided hypothesis and then anchoring it on a structurally degenerate depth,
which I did yesterday on the credit-depth suite. Registering "two-sided" in words
does not help if every named outcome reads one way.

## 3. What the decomposition actually says

**Neither tested cause explains why the reference beats the instrument.**

- **Depth: negative.** The reference is 2 layers; 1 layer is *better*. Depth is not
  a source of its advantage, it is a cost it pays.
- **Dropout: positive but small.** Worth 0.0128, and the instrument has none — so
  this is one real, cheap thing the instrument could adopt.
- **Delays: 0.0348**, from the previous run.

And the gap widens rather than closing:

```
best delay-free config seen (1 layer, dropout 0.4)   0.9187
instrument, converged attention arm                  0.8320
                                                     ------
                                                     0.0867
```

Against the *best* delay-free reference the instrument is **0.0867 behind**, not
the 0.0722 I started from. Removing the reference's worst feature made the thing
I am trying to explain larger.

## 4. What is left, and it is most of it

Of the seven enumerated differences, two are now measured (layers, dropout) and
neither accounts for the gap. The remaining five are untested:

| # | difference | status |
|---|---|---|
| 3 | 256 hidden units vs 128 | untested, confounded with instrument tuning |
| 4 | one-cycle + weight decay 1e-5 | untested |
| 5 | 150 epochs vs 400 | untested, confounded |
| 6 | `published-10ms` vs `published-2ms` binning | not a knob on this config |
| 7 | summed non-spiking readout vs rate readout | not a knob on this config |

**And the possibility I named in §5 of the prereg is now the live one:** that the
enumeration is still incomplete. I have been wrong twice today about what this
list contains — once claiming two features that were switched off, once assuming
a sign. A third omission is more likely than not.

Differences 6 and 7 are the ones I would look at first, precisely because they are
*not* configuration: they are what the reference **is**. A summed non-spiking
readout over 10 ms bins is a different computation from a rate readout over 2 ms
bins, and no amount of config-level ablation will reach it.

## 5. What this does not establish

- **One seed each.** Both effects clear the resolvability bar but neither is
  three-seed confirmed. The signs are what matter here and both are well clear of
  the spread; the magnitudes should not be quoted to three decimals.
- **Marginal, not additive.** Registered in advance and it matters more now that
  the effects have opposite signs: `+0.0145` and `−0.0128` must not be summed,
  netted, or subtracted from 0.0722.
- **It varies the reference, not the instrument.** "The instrument could adopt
  dropout" is inference from a reference-side ablation, not a measurement of BINN.
- **No gate moved.** Diagnostics outside `results/shd_instrument_v4/`;
  `references/`, `reference-manifests/` and `reference-states/` untouched;
  `gates.json` never opened; `SHD_INSTRUMENT_STATE` still `Uncalibrated`.
