#!/usr/bin/env python3
"""Frozen analyser for the 2026-09-02 membrane ablation.

Registered by `results/PREREG_2026-09-02_THE_MEMBRANE_ABLATION.md`, committed in
the same commit and before any run existed. This file is the authority on
HM-1..HM-4.

It reports NOT EVALUABLE rather than computing a verdict from fewer runs than
were registered, and HM-1 gates everything: a membrane effect measured against a
baseline that moved is not a measurement.
"""
from __future__ import annotations

import json
import pathlib
import statistics
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
CORPUS = ROOT / "results/membrane_ablation_2026-09-02"

SEEDS = 3                      # registered; not a floor to be relaxed
ARM_B_KERNEL_ABLATION = 0.6276  # mean(B) from RESULT_2026-09-02_THE_KERNEL_ABLATION
INSTRUMENT = 0.8320            # the instrument's converged attention arm
MISSING = 0.2044               # how far Arm B sat below it
HM1_MAX_DRIFT = 0.020          # 2.8x Arm B's own three-seed spread of 0.0072
HM2_MIN_M = 0.050              # material
HM3_MAX_DIST = 0.030           # "closes the gap"
HM4_REFUTES_BELOW = 0.020


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
    b = load("armB2-nokernel")
    e = load("armE-nokernel-slowmembrane")
    print("# Membrane ablation — is the fast membrane the term that offsets the kernel?\n")
    print("Registered: `PREREG_2026-09-02_THE_MEMBRANE_ABLATION.md`. This "
          "analyser is the authority on every verdict below.\n")
    print("| arm | `init_tau` | retention | n | mean | values |")
    print("|---|---:|---:|---:|---:|---|")
    for name, tau, ret, arm in (("B′ — membrane as pinned", "10.05 ms", "0.0050", b),
                                ("E — membrane matched", "55.5556 ms", "0.8200", e)):
        mean = f"{statistics.fmean(arm):.4f}" if arm else "—"
        vals = ", ".join(f"{v:.4f}" for v in arm) if arm else "none"
        print(f"| {name} | {tau} | {ret} | {len(arm)} | {mean} | {vals} |")
    print()

    if len(b) < SEEDS or len(e) < SEEDS:
        print(f"**NOT EVALUABLE** — the series registers {SEEDS} seeds per arm and "
              f"this corpus holds {len(b)} and {len(e)}. No verdict is computed "
              f"from a short series.")
        return 0

    mean_b, mean_e = statistics.fmean(b), statistics.fmean(e)
    m = mean_e - mean_b

    drift = abs(mean_b - ARM_B_KERNEL_ABLATION)
    hm1 = drift <= HM1_MAX_DRIFT
    print(f"**HM-1 — {'MET' if hm1 else 'NOT MET'}.** Arm B′ is {mean_b:.4f} "
          f"against the kernel ablation's {ARM_B_KERNEL_ABLATION}, a drift of "
          f"{drift:.4f} against {HM1_MAX_DRIFT}.")
    if not hm1:
        print("\n**Nothing below is read.** The registration makes this the gate: "
              "a membrane effect measured against a baseline that moved is not a "
              "measurement, and the two arms would differ by the platform as well "
              "as by the manipulation.")
        return 0

    print(f"\n**M = {m:+.4f}** — what the membrane is worth once the kernel is "
          f"gone ({mean_e:.4f} − {mean_b:.4f}), against the {MISSING} that "
          f"separated the kernel-free reference from the instrument.\n")

    print(f"**HM-2 — {'MET' if m >= HM2_MIN_M else 'NOT MET'}.** M = {m:+.4f} "
          f"against a bar of {HM2_MIN_M}. "
          + ("The membrane is a material part of the missing term."
             if m >= HM2_MIN_M else
             "The membrane is not a material part of the missing term."))

    dist = abs(mean_e - INSTRUMENT)
    hm3 = dist <= HM3_MAX_DIST
    print(f"\n**HM-3 — {'MET' if hm3 else 'NOT MET'}.** Arm E is {mean_e:.4f} "
          f"against the instrument's {INSTRUMENT}, a distance of {dist:.4f} "
          f"against {HM3_MAX_DIST}.")
    if m >= HM2_MIN_M and hm3:
        print("  Two terms account for the two architectures: a kernel worth "
              "+0.3111 to the reference and a membrane worth this much to it. "
              "§3.8 gains a decomposition it has never had.")
    elif m >= HM2_MIN_M:
        print(f"  The membrane carries part of it and {dist:.4f} is still "
              "unexplained. The register item is narrowed with a number, not "
              "closed.")

    print(f"\n**HM-4 — {'REFUTED' if m < HM4_REFUTES_BELOW else 'not refuted'}.** "
          + (f"M = {m:+.4f} is below {HM4_REFUTES_BELOW}: the membrane is not "
             f"the compensating term, one candidate is eliminated, and the "
             f"§2.4 register item stays open."
             if m < HM4_REFUTES_BELOW else
             f"M = {m:+.4f} is at or above {HM4_REFUTES_BELOW}, so the membrane "
             f"hypothesis stands as measured rather than inferred."))
    if m < -HM2_MIN_M:
        print("\n  **M is strongly negative.** Slowing the membrane makes the "
              "kernel-free reference worse, which says its other machinery is "
              "tuned around a pointwise neuron. Reported as that, not as a null.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
