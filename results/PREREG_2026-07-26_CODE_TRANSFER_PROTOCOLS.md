# Frozen code protocol manifest — SHD 0c-1, temporal depth, and transfer

Registered 2026-07-26. This is a code-integrity artifact, not manuscript text.

## Version isolation

- SHD paired input control: protocol v143, fresh capped and full seed families.
- Temporal task calibration: protocol v144, calibration-only seed family.
- Shared-forward depth run: protocol v145, disjoint scientific seed family.
- Same-specification transfer falsifier: protocol v146, `BINNTRF1` bundles.
- Canonical C1, v13–v24, p27/p29, v142, and historical deep artifacts are
  immutable inputs to status interpretation and are never rewritten.

## Frozen decision gates

SHD equivalence is `mean(hidden - input) < 0.02` and hierarchical-bootstrap
upper 95% bound `< 0.05`. A lower bound `> 0.05` retains SHD. Otherwise the
same schedule extends from 10 to 20 seeds.

Temporal calibration considers only `(0,4)`, `(1,8)`, `(2,12)`, `(3,16)` and
chooses the hardest candidate with matched feedback in `[0.55,0.80]`, BPTT in
`[0.65,0.90]`, and raw-rate/time-shuffled controls `<= 0.28`. No qualifying
candidate means `INVALID_TASK`; no extra grid is allowed.

The BPTT ceiling uses deterministic Adam (`lr=1e-3`, beta1 `0.9`, beta2
`0.999`, epsilon `1e-8`) and global norm clipping at `5.0`. The scientific
depth schedule is widths 128, depths 1–4, 40 epochs, 10 seeds.

Transfer matched validity requires accuracy `[0.40,0.85]`, lower 95% bound
above `0.35`, all three controls at chance, and non-degenerate BPTT not
materially below the local arm. The Rust phenomenon requires mean gap `>=0.10`
and paired lower 95% bound `>0.05`. NumPy reproduction requires endpoint means
within `0.05`, gap within `0.10`, micro-conformance within `1e-6`, and replay.

## Execution stop rules

- Fixture, quick, or smoke reports are always `PILOT`.
- v145 refuses to run without a valid v144 freeze file.
- v146 scientific execution refuses to run without that same freeze.
- Micro-conformance failure is `INVALID_HARNESS` and stops before a scientific
  transfer claim.
- If NumPy conformance passes but the paired gap fails, the transfer campaign
  stops.
