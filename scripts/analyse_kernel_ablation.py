#!/usr/bin/env python3
"""Frozen analyser for the 2026-09-01 kernel ablation.

Registered by `results/PREREG_2026-09-01_THE_KERNEL_ABLATION.md`, committed in
the same commit and before any run existed. This file is the authority on
HK-1..HK-4.

It reports NOT EVALUABLE rather than computing a verdict from fewer runs than
were registered. A series that half-ran must not read as a series that ran.
"""
from __future__ import annotations

import json
import pathlib
import statistics
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
CORPUS = ROOT / "results/kernel_ablation_2026-09-01"

SEEDS = 3                      # registered; not a floor to be relaxed
RESIDUAL = 0.087               # the gap this series is about
INSTRUMENT = 0.8320            # the instrument's converged attention arm
REFERENCE = 0.9376             # mean of the three recorded reference values
HK1_MIN_K = 0.050              # majority of the residual
HK2_MAX_DIST = 0.030           # "lands near the instrument"
HK3_REFUTES_BELOW = 0.020      # below this the attribution is refuted
HK4_MAX_DRIFT = 0.010          # Arm A must reproduce the pinned reference
HK5_MAX_AUG = 0.020            # amendment: the augmentation must not carry K


def load(arm: str) -> list[float]:
    if not CORPUS.is_dir():
        return []
    out = []
    for path in sorted(CORPUS.glob(f"{arm}__*.json")):
        try:
            cell = json.loads(path.read_text())
        except ValueError:
            continue
        accuracy = cell.get("accuracy")
        if isinstance(accuracy, (int, float)) and 0.0 <= accuracy <= 1.0:
            out.append(float(accuracy))
    return out


def main() -> int:
    a = load("armA-reference")
    b = load("armB-nokernel")
    c = load("armC-nokernel-augfalls")
    print("# Kernel ablation — what the 25-tap temporal kernel is worth\n")
    print(f"Registered: `PREREG_2026-09-01_THE_KERNEL_ABLATION.md`. This "
          f"analyser is the authority on every verdict below.\n")
    print(f"| arm | n | mean | values |")
    print(f"|---|---:|---:|---|")
    for name, arm in (("A — reference as pinned", a),
                      ("B — kernel removed, augmentation held", b),
                      ("C — kernel removed, augmentation falls", c)):
        mean = f"{statistics.fmean(arm):.4f}" if arm else "—"
        vals = ", ".join(f"{v:.4f}" for v in arm) if arm else "none"
        print(f"| {name} | {len(arm)} | {mean} | {vals} |")
    print()

    if len(a) < SEEDS or len(b) < SEEDS or len(c) < SEEDS:
        print(f"**NOT EVALUABLE** — the series registers {SEEDS} seeds per arm "
              f"and this corpus holds {len(a)}, {len(b)} and {len(c)}. No verdict "
              f"is computed from a short series.")
        return 0

    mean_a, mean_b = statistics.fmean(a), statistics.fmean(b)
    k = mean_a - mean_b

    print(f"**K = {k:+.4f}** — the accuracy the kernel is worth "
          f"({mean_a:.4f} − {mean_b:.4f}), against a residual of {RESIDUAL}.\n")

    hk4 = abs(mean_a - REFERENCE) <= HK4_MAX_DRIFT
    print(f"**HK-4 — {'MET' if hk4 else 'NOT MET'}.** Arm A is {mean_a:.4f} "
          f"against the pinned {REFERENCE}, a drift of {abs(mean_a-REFERENCE):.4f} "
          f"against {HK4_MAX_DRIFT}.")
    if not hk4:
        print("\n**Nothing below is read.** The registration says an Arm A that "
              "does not reproduce means the harness moved, and a kernel cost "
              "measured against a broken baseline is worse than no measurement.")
        return 0

    print(f"\n**HK-1 — {'MET' if k >= HK1_MIN_K else 'NOT MET'}.** K = {k:+.4f} "
          f"against a bar of {HK1_MIN_K}. "
          + ("The kernel carries the majority of the residual."
             if k >= HK1_MIN_K else
             "The kernel does not carry the majority of the residual."))

    dist = abs(mean_b - INSTRUMENT)
    print(f"\n**HK-2 — {'MET' if dist <= HK2_MAX_DIST else 'NOT MET'}.** "
          f"Arm B is {mean_b:.4f} against the instrument's {INSTRUMENT}, a "
          f"distance of {dist:.4f} against {HK2_MAX_DIST}.")
    if k >= HK1_MIN_K and dist > HK2_MAX_DIST:
        print("  This is the outcome the registration calls most informative: "
              "the kernel is real AND it is not the whole story, and section 3.8 "
              "has to say so.")

    print(f"\n**HK-3 — {'REFUTED' if k < HK3_REFUTES_BELOW else 'not refuted'}.** "
          + (f"K = {k:+.4f} is below {HK3_REFUTES_BELOW}: the residual does not "
             f"live in the kernel and section 3.8's attribution is WITHDRAWN."
             if k < HK3_REFUTES_BELOW else
             f"K = {k:+.4f} is at or above {HK3_REFUTES_BELOW}, so the "
             f"attribution stands as measured rather than inferred."))

    mean_c = statistics.fmean(c)
    aug = abs(mean_b - mean_c)
    print(f"\n**HK-5 — {'MET' if aug <= HK5_MAX_AUG else 'NOT MET'}.** Holding "
          f"`time_mask_size` versus letting it fall to 0 moves the ablated "
          f"reference by {aug:.4f} ({mean_b:.4f} vs {mean_c:.4f}) against "
          f"{HK5_MAX_AUG}. "
          + ("The augmentation is not carrying K, so K is the kernel's."
             if aug <= HK5_MAX_AUG else
             "The augmentation is a material term here: K is reported WITH this "
             "caveat, and the naive max_delay=1 ablation is confounded."))
    return 0


if __name__ == "__main__":
    sys.exit(main())
