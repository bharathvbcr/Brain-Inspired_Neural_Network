# Defect — HK-5 compares an arm with itself

**Found:** 2026-09-01, **before any run of the kernel-ablation series existed.**
**Affects:** [`PREREG_2026-09-01_THE_KERNEL_ABLATION.md`](PREREG_2026-09-01_THE_KERNEL_ABLATION.md)
§6 (the amendment at `c292c4a`), Arm C, and HK-5.
**Cost had it not been found:** three of nine runs, ~13 h of compute, and a
verdict that would have read as evidence.

---

## What the amendment claimed

§6 was added because `time_mask_size = max_delay//3` falls from 8 to 0 when
`max_delay` is set to 1, which would switch off a time-masking augmentation in
the same edit that removes the temporal kernel. The amendment held
`time_mask_size` at 8 for Arm B, added **Arm C** at `max_delay = 1` with the
value allowed to fall, and registered **HK-5**: `|mean(B) − mean(C)| ≤ 0.020`,
so the augmentation's contribution would be *measured rather than assumed away*.

The instinct was right. The premise was not checked.

## The augmentation is unreachable at this operating point

`time_mask_size` is read at exactly two places in the pinned checkout:

```
datasets.py:45   mask_size = np.random.randint(0, self.config.time_mask_size)
datasets.py:46   ind = np.random.randint(0, sample.shape[0] - self.config.time_mask_size)
```

Both are inside `TimeNeurons_mask_aug.__call__`. That class is constructed only
by `Augs` (`datasets.py:96`), and `Augs` is constructed only at `model.py:193`
and applied only at `model.py:217`, under `if self.config.augment:`.

**Two independent reasons it never runs here:**

1. **`best_config_SHD.py:125` sets `augment = False`.** The pinned SHD recipe
   does not use these augmentations at all, in any protocol.
2. **The clean protocol does not go through `model.train_model`.**
   `scripts/shd_calibration/reference_clean_main.py` carries its own training
   loop — the whole point of the clean fork — and contains no reference to
   `Augs`, `TimeNeurons_mask_aug`, `CutMix`, or `augment`.

So Arm B and Arm C differ in a value that nothing reads. They are the **same
configuration**, and HK-5 would compare an arm with itself.

## Why this is the dangerous shape and not merely a waste

Three runs and 13 h of compute is the small half. The large half is that HK-5
would have **passed**: `|mean(B) − mean(C)|` between two identical
configurations is small by construction, the analyser would have printed *"The
augmentation is not carrying K, so K is the kernel's"*, and that sentence would
have entered the record as a measured control.

It would have been true — the augmentation is indeed not carrying K — and it
would have been supported by no evidence whatsoever. A check that cannot fail
reporting the same words as a check that ran and passed is the defect class this
campaign keeps finding, most recently in
[`DEFECT_2026-08-31_H22_3_CANNOT_BE_EVALUATED.md`](DEFECT_2026-08-31_H22_3_CANNOT_BE_EVALUATED.md).
There, a registered hypothesis had no cells to evaluate and said so loudly.
Here, it would have had cells and said the wrong thing quietly. **The quiet
failure is the worse one**, and it is the one an empty-corpus test cannot catch.

## How it was found

Not by an empty-corpus run — that check passes on this analyser and always did.
By tracing every consumer of both manipulated values before launching:

```
grep -rn "max_delay"      <checkout>/*.py   # -> Dcls1d dilated_kernel_size x3, + 6 derived config values
grep -rn "time_mask_size" <checkout>/*.py   # -> datasets.py:45,46 only
```

`max_delay` survives that trace: its only non-config consumer is
`dilated_kernel_size` at `snn_delays.py:33`, `:61` and `:89`, which is exactly
the mechanism the series is about. `time_mask_size` does not.

A construction smoke test at all three arms corroborates it. All three build,
forward and backward; Arm A emits 136 timesteps against Arm B and Arm C's 100 —
the 24 + 12 kernel padding, present and then gone — while B and C produce an
identical loss on identical synthetic input. That last fact is corroboration
rather than proof: the synthetic input bypasses the loader either way. **The
reachability trace is the proof.**

## The check this adds

**Before registering a manipulation, trace every consumer of every value it
moves, and record which are reachable in the protocol that will actually run.**

The existing rule — *run the frozen analyser against its own plan, not only
against an empty corpus* — is necessary and was not sufficient. A plan can be
complete, an analyser can be correct, every arm can land, and a hypothesis can
still be about a value the code never reads. Reachability is a separate
question from coverage and needs its own answer, in writing, before launch.

## Resolution

Arm C and HK-5 are **withdrawn before any run**, by amendment §7 of the
preregistration. The series returns to the six runs of the original design, and
`time_mask_size` is left to follow `max_delay` because holding it changes
nothing — which is now a verified statement rather than an assumption in either
direction.
