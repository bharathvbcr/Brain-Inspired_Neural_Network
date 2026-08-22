#!/usr/bin/env python3
"""Compute the PREREG_2026-08-19_SHD_ATTENTION_READOUT verdicts from the cells."""
import json, pathlib, statistics, sys

P = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else ".") / "cells"
SEEDS = [5170001, 5170002, 5170003]
ARMS = {
    "A": "A_base_h128_s{}",
    "B": "B_attn_h128_s{}",
    "C": "C_base_h192_s{}",
    "D": "D_base_h128_shuf_s{}",
    "E": "E_attn_h128_shuf_s{}",
}

cells = {}
for arm, pattern in ARMS.items():
    for seed in SEEDS:
        path = P / (pattern.format(seed) + ".json")
        if path.exists():
            cells[(arm, seed)] = json.loads(path.read_text())

def acc(arm):
    return [cells[(arm, s)]["accuracy"] for s in SEEDS if (arm, s) in cells]

print(f"{'cell':<34}{'acc':>10}{'classes':>9}{'major':>9}{'nonfin':>8}{'maxgrad':>12}{'tailimp':>10}")
for arm in ARMS:
    for seed in SEEDS:
        c = cells.get((arm, seed))
        if not c:
            print(f"{arm}/{seed:<28} MISSING")
            continue
        print(f"{arm}/{seed} {c['arm']:<18}{c['accuracy']:>10.4f}{c['classes_predicted']:>9}"
              f"{c['majority_prediction']:>9.3f}{c['non_finite_events']:>8}"
              f"{max(c['epoch_max_gradient_norm']):>12.3e}{c['tail_loss_improvement']:>10.4f}")

if all(len(acc(a)) == 3 for a in ARMS):
    mean = {a: statistics.mean(acc(a)) for a in ARMS}
    sd = {a: statistics.stdev(acc(a)) for a in ARMS}
    print("\nmeans:", {a: round(mean[a], 4) for a in ARMS})
    print("sds:  ", {a: round(sd[a], 4) for a in ARMS})

    per_seed_ba = [cells[("B", s)]["accuracy"] - cells[("A", s)]["accuracy"] for s in SEEDS]
    per_seed_bc = [cells[("B", s)]["accuracy"] - cells[("C", s)]["accuracy"] for s in SEEDS]
    gain_intact = mean["B"] - mean["A"]
    gain_shuf = mean["E"] - mean["D"]

    print(f"\nH-A1  mean(B)-mean(A) = {gain_intact:+.4f}  (>= 0.05 and all seeds positive)")
    print(f"      per-seed: {[round(v, 4) for v in per_seed_ba]}")
    ha1 = gain_intact >= 0.05 and all(v > 0 for v in per_seed_ba)
    print(f"      verdict: {'SUPPORTED' if ha1 else 'NOT SUPPORTED'}")

    print(f"\nH-A2  mean(B)-mean(C) = {mean['B'] - mean['C']:+.4f}  (>= 0.02, positive in >=2/3)")
    print(f"      per-seed: {[round(v, 4) for v in per_seed_bc]}")
    ha2 = (mean["B"] - mean["C"]) >= 0.02 and sum(v > 0 for v in per_seed_bc) >= 2
    print(f"      verdict: {'NOT A CAPACITY ARTEFACT' if ha2 else 'NOT SUPPORTED'}")

    print(f"\nH-A3  gain(intact) {gain_intact:+.4f} - gain(bin-shuffled) {gain_shuf:+.4f} "
          f"= {gain_intact - gain_shuf:+.4f}  (>= 0.02)")
    ha3 = (gain_intact - gain_shuf) >= 0.02
    print(f"      verdict: {'MEMORY, not just capacity' if ha3 else 'NOT SUPPORTED'}")

    attn_cells = [cells[(a, s)] for a in ("B", "E") for s in SEEDS]
    peak = max(max(c["epoch_max_gradient_norm"]) for c in attn_cells)
    nonfinite = sum(c["non_finite_events"] for c in attn_cells)
    ha4 = nonfinite == 0 and peak < 1e3
    print(f"\nH-A4  non_finite_events {nonfinite}, peak gradient norm {peak:.4e}  (0 and < 1e3)")
    print(f"      verdict: {'STABLE' if ha4 else 'NOT SUPPORTED'}")

    print("\nvalidity gates")
    for (a, s), c in sorted(cells.items()):
        problems = []
        if c["non_finite_events"] != 0: problems.append("non-finite")
        if c["classes_predicted"] != 20: problems.append(f"classes={c['classes_predicted']}")
        if c["majority_prediction"] >= 0.30: problems.append(f"majority={c['majority_prediction']:.3f}")
        if c["silent_fraction"] > 0.95: problems.append("silent")
        if c["saturated_fraction"] > 0.05: problems.append(f"saturated={c['saturated_fraction']:.3f}")
        if a in ("D", "E"):
            audit = c["temporal_audit"]
            if not audit["counts_preserved"]: problems.append("counts moved")
            if audit["relocated_fraction"] < 0.5: problems.append("weak manipulation")
        if problems:
            print(f"  {a}/{s}: {', '.join(problems)}")
    print("  (nothing listed above = all gates pass)")
else:
    print("\nincomplete: %d/15 cells" % len(cells))
