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
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
