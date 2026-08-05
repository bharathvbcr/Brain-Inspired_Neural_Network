# v148 decision

**PASS — shortcut accessibility determines whether the frozen multiclass local
path learns this task family.**

Protocol: `shortcut-access-v148-953d6f24133cafb6`

| Variant | Raw rate | Local | BPTT | Local hidden mean | End local modulator RMS |
|---|---:|---:|---:|---:|---:|
| A. rate-accessible | 1.0000 | 1.0000 | 1.0000 | 0.0155 | 1.238e-2 |
| B. rate-immune | 0.2500 | 0.2500 | 1.0000 | 0.0064 | 1.260e-2 |

All three paired seeds followed the same pattern. The intervention contract,
parameter-preserving evaluation, local/BPTT internal replays, raw-rate and BPTT
reference gates, and report replay passed. The report SHA-256 was byte-identical
across two complete executions:

`4202515b9a437c8cc4006de1dcf48d8112b49a0a7bbdb739f54d0160889723a5`

## Interpretation

- The multiclass local code path is not globally broken: it learned the
  rate-accessible positive control perfectly.
- The same path stayed at chance when per-channel counts were label-immune,
  while true shared-forward BPTT remained perfect.
- Similar end-of-training modulator RMS rules out an absent or collapsed
  feedback signal as the immediate explanation for the A/B accuracy contrast.
- Hidden activity differed materially (`0.0155` versus `0.0064` mean final
  rate); the experiment discloses this mediator but does not by itself separate
  representation accessibility from activity magnitude.

This result does not reopen v145/v146, establish matched-to-live transfer,
attribute a substrate gap, or demonstrate a rescue. Per the preregistration,
do not add an optimizer, marker-strength, width, epoch, or difficulty sweep.
