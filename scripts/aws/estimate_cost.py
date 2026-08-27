#!/usr/bin/env python3
"""Cost and wall-clock estimate for a cell plan, from measured timings.

Calibration (this repository's dev machine, M5 Pro, one core, published-2ms /
adjacent-sum-5, full 8156-sample training split):

    ff+fixed        h128            9.6 s / epoch
    ff+fixed+attn   h128 d32 L1    67.7 s / epoch

The attention increment (58.1 s) splits into a T^2*D*layers core and a T*H*D
embedding/spike-gradient term in a measured ratio of about 2.8 : 1 at the
anchor. Everything below is that decomposition extrapolated - it is an estimate
and is labelled as one. The real numbers come back in `wall_secs` per cell.
"""

from __future__ import annotations

import argparse
import json
from collections import defaultdict
from pathlib import Path

BASE_H128_S_PER_EPOCH = 9.6
ATTN_INCREMENT_S = 58.1
ATTN_CORE_FRACTION = 0.74  # T^2 * D * layers
ATTN_IO_FRACTION = 0.26  # T * H * D
REF_T, REF_D, REF_H = 358, 32, 128

# Mean SHD utterance is 0.716 s; timesteps follow from the contract.
TIMESTEPS = {
    "published-2ms": 358, "published-4ms": 179, "published-10ms": 72,
    "fixed-t100": 100, "fixed-t250": 250, "fixed-t500": 500,
}
# Graviton3 single-core against this Mac's, for scalar f32. Deliberately
# pessimistic: if it is better than this the campaign finishes early.
GRAVITON_RATIO = 0.55
C7G_16XL_SPOT_USD_PER_HOUR = 0.6259
C7G_16XL_VCPU = 64


def cell_core_seconds(cell: dict) -> float:
    t = TIMESTEPS[cell["contract"]]
    hidden = cell["hidden"]
    epochs = cell["epochs"]

    base = BASE_H128_S_PER_EPOCH * (hidden / REF_H)
    if cell["geometry"] == "channels-700":
        base *= 1.15  # wider input matrix, same event count
    if cell["arm"].startswith("rec"):
        # The recurrent drive is hidden^2 per timestep against an input drive
        # that is events*hidden. Measured at ~21x before the transpose fix and
        # much less after; 3x at h256 is the conservative reading.
        base *= 3.0 * (hidden / 256)

    attn = 0.0
    if cell["attn_dim"]:
        d, layers = cell["attn_dim"], cell["attn_layers"]
        core = ATTN_INCREMENT_S * ATTN_CORE_FRACTION * (t / REF_T) ** 2 * (d / REF_D) * layers
        io = ATTN_INCREMENT_S * ATTN_IO_FRACTION * (t / REF_T) * (hidden / REF_H) * (d / REF_D)
        attn = core + io
    return (base + attn) * epochs



#: Cells on disk to calibrate against. Only same-fleet, same-wave comparisons
#: are meaningful -- `wall_secs` is wall time under four-way co-scheduling, and
#: across waves it is not a function of configuration at all (`d32l1` at h1024
#: records 5.21 h against `d32l4`'s 3.40 h, though layers multiply the cost).
#: So this does not recalibrate anything. It states the model's bias next to the
#: model's answer, because an estimate quoted without its known bias is what
#: produced a "~6 h" ETA against 14 h of remaining work.
CALIBRATION_ROOT = "results/shd_attention_campaign_v2"
#: Waves that ran on the 2026-08-26/27 four-node c7g.16xlarge fleet at 16
#: threads per cell. Cells outside these are on other fleets and are not
#: comparable to each other or to these.
CALIBRATION_WAVES = ("w15col", "w16lad", "w17hdl")


def measured_medians(root: Path) -> dict[tuple, float]:
    """Median `wall_secs` per configuration, over the calibration fleet only."""
    import re
    import statistics
    from collections import defaultdict

    seen = defaultdict(list)
    for path in sorted(root.glob("*.json")):
        parts = path.stem.split("__")
        if len(parts) < 6 or parts[0] not in CALIBRATION_WAVES:
            continue
        try:
            cell = json.loads(path.read_text())
        except (json.JSONDecodeError, OSError):
            continue
        wall = cell.get("wall_secs")
        if not isinstance(wall, (int, float)) or wall <= 0:
            continue
        attn = next((m.groups() for m in map(
            lambda p: re.fullmatch(r"d(\d+)l(\d+)", p), parts[5:]) if m), None)
        key = (parts[1], int(parts[2][1:]), int(parts[3][1:]), parts[4],
               (int(attn[0]), int(attn[1])) if attn else None)
        seen[key].append(wall)
    return {k: statistics.median(v) for k, v in seen.items() if len(v) >= 3}


def calibration_report(effective_threads: float) -> list[str]:
    """Predicted against measured, per configuration. Never silent: if the
    corpus is missing, that is said rather than skipped."""
    root = Path(__file__).resolve().parent.parent.parent / CALIBRATION_ROOT
    if not root.is_dir():
        return [f"CALIBRATION UNAVAILABLE: {CALIBRATION_ROOT} is not on disk. "
                f"The estimate below is UNCHECKED."]
    medians = measured_medians(root)
    if not medians:
        return [f"CALIBRATION UNAVAILABLE: no cell in {CALIBRATION_ROOT} carries "
                f"`wall_secs` for waves {'/'.join(CALIBRATION_WAVES)}. "
                f"The estimate below is UNCHECKED."]
    lines = ["model against the cells on disk "
             f"({len(medians)} configurations, {'/'.join(CALIBRATION_WAVES)}):",
             f"  {'configuration':<34}{'predicted':>11}{'measured':>10}{'ratio':>8}"]
    ratios = []
    for key, wall in sorted(medians.items()):
        arm, hidden, epochs, contract, attn = key
        cell = {"arm": arm.replace("-", "+"), "hidden": hidden, "epochs": epochs,
                "contract": contract, "geometry": "adjacent-sum-5",
                "attn_dim": attn[0] if attn else None,
                "attn_layers": attn[1] if attn else None}
        predicted = cell_core_seconds(cell) / GRAVITON_RATIO / effective_threads
        ratios.append(predicted / wall)
        label = f"{arm} h{hidden}" + (f" d{attn[0]}l{attn[1]}" if attn else "")
        lines.append(f"  {label:<34}{predicted / 3600:>10.2f}h"
                     f"{wall / 3600:>9.2f}h{predicted / wall:>8.2f}x")
    ratios.sort()
    mid = ratios[len(ratios) // 2]
    lines += ["",
              f"  median over-prediction {mid:.2f}x "
              f"(range {ratios[0]:.2f}-{ratios[-1]:.2f}x). Divide the estimate "
              f"below by roughly this.",
              "  A ratio under 1.00x is impossible without an above-100% "
              "parallel efficiency,",
              "  so where one appears the model is under-predicting genuinely."]
    return lines


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("plan")
    parser.add_argument("--vcpus", type=int, default=128, help="total spot vCPUs available")
    parser.add_argument("--threads-per-cell", type=int, default=16)
    parser.add_argument("--parallel-efficiency", type=float, default=0.49,
                        help="measured: 7.9x on 16 threads, memory-bandwidth bound")
    args = parser.parse_args()

    cells = json.load(open(args.plan))
    per_wave = defaultdict(lambda: [0, 0.0, 0.0])
    for cell in cells:
        mac = cell_core_seconds(cell)
        grav = mac / GRAVITON_RATIO
        row = per_wave[cell["wave"]]
        row[0] += 1
        row[1] += mac
        row[2] += grav

    print(f"{'wave':<8}{'cells':>7}{'mac core-h':>13}{'graviton vCPU-h':>18}")
    total_mac = total_grav = 0.0
    for wave in sorted(per_wave):
        count, mac, grav = per_wave[wave]
        total_mac += mac
        total_grav += grav
        print(f"{wave:<8}{count:>7}{mac / 3600:>13.1f}{grav / 3600:>18.1f}")
    print(f"{'TOTAL':<8}{len(cells):>7}{total_mac / 3600:>13.1f}{total_grav / 3600:>18.1f}")

    usd_per_vcpu_hour = C7G_16XL_SPOT_USD_PER_HOUR / C7G_16XL_VCPU
    cost = total_grav / 3600 * usd_per_vcpu_hour
    concurrent_cells = max(1, args.vcpus // args.threads_per_cell)
    effective = args.threads_per_cell * args.parallel_efficiency
    wall = total_grav / (concurrent_cells * effective) / 3600

    slowest = max(cells, key=cell_core_seconds)
    slowest_wall = cell_core_seconds(slowest) / GRAVITON_RATIO / effective / 3600

    print()
    print(f"spot cost              ${cost:,.2f}   (c7g.16xlarge @ ${C7G_16XL_SPOT_USD_PER_HOUR}/h)")
    print(f"concurrency            {concurrent_cells} cells x {args.threads_per_cell} threads "
          f"= {concurrent_cells * args.threads_per_cell} vCPU")
    print(f"estimated wall time    {wall:,.1f} h")
    print(f"longest single cell    {slowest_wall:,.1f} h  ({slowest['id']})")
    print()
    print("The longest single cell is the floor on wall time: no amount of extra")
    print("capacity divides it further once it already has its threads.")
    print()
    for line in calibration_report(effective):
        print(line)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
