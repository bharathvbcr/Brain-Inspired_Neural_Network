#!/usr/bin/env python3
"""Enumerate every cell in the AWS attention campaign, as a deterministic list.

One JSON object per cell. The list is the campaign: `launch.py` shards it,
`bootstrap.sh` runs a shard, and `collect.py` reconciles what came back against
it. Nothing downstream invents a cell that is not in here.

The `id` is the file name in S3 and on disk. It is built from the fields that
change the result and nothing else, so a cell that is already in the results
bucket is skipped rather than re-run — which is what makes a spot interruption
cost one cell instead of a wave.
"""

from __future__ import annotations

import argparse
import json
import sys

# 12 seeds everywhere a hypothesis is tested. The instrument's own H1 flipped
# from SUPPORTED to NOT SUPPORTED between n=3 and n=6 with the effect size
# unchanged - only the resolution moved. Three seeds is not enough to publish on.
SEEDS = [5170001 + i for i in range(12)]
ANCHOR = ("published-2ms", "adjacent-sum-5")


def cell(wave, arm, hidden, epochs, seed, *, contract=ANCHOR[0], geometry=ANCHOR[1],
         attn_dim=None, attn_layers=None, temporal="intact", surrogate_scale=None,
         clip_grad_norm=None, n_train=8156):
    parts = [wave, arm.replace("+", "-"), f"h{hidden}", f"e{epochs}", contract, geometry]
    if attn_dim is not None:
        parts.append(f"d{attn_dim}l{attn_layers}")
    if temporal != "intact":
        parts.append(temporal)
    if surrogate_scale is not None:
        parts.append(f"ss{surrogate_scale}")
    if clip_grad_norm is not None:
        parts.append(f"clip{clip_grad_norm}")
    parts.append(f"s{seed}")
    return {
        "id": "__".join(parts),
        "wave": wave,
        "arm": arm,
        "hidden": hidden,
        "epochs": epochs,
        "seed": seed,
        "contract": contract,
        "geometry": geometry,
        "attn_dim": attn_dim,
        "attn_layers": attn_layers,
        "temporal": temporal,
        "temporal_seed": 5170001 if temporal != "intact" else None,
        "surrogate_scale": surrogate_scale,
        "clip_grad_norm": clip_grad_norm,
        "n_train": n_train,
        "n_inputs": 700 if geometry == "channels-700" else 140,
    }


def wave1_converged():
    """T1 - the registered next step: does the pilot's +0.1702 survive convergence?

    e400 is the budget the recorded width axis is converged at, so arm A is
    directly comparable to the recorded 0.7032 *on the same machine*; against a
    different machine it is not, which is why A is re-run here rather than cited.
    """
    cells = []
    for seed in SEEDS:
        cells.append(cell("w1", "ff+fixed", 128, 400, seed))
        cells.append(cell("w1", "ff+fixed+attn", 128, 400, seed, attn_dim=32, attn_layers=1))
        cells.append(cell("w1", "ff+fixed", 192, 400, seed))  # capacity control
        # Mechanism, at convergence this time.
        cells.append(cell("w1", "ff+fixed", 128, 400, seed, temporal="bin-shuffled"))
        cells.append(cell("w1", "ff+fixed+attn", 128, 400, seed, attn_dim=32, attn_layers=1,
                          temporal="bin-shuffled"))
    return cells


def wave2_design_space():
    """T2 - the four axes the pilot named as untested. e100, held fixed across the sweep."""
    cells = []
    for seed in SEEDS:
        for dim in (16, 32, 64, 128):
            cells.append(cell("w2dim", "ff+fixed+attn", 128, 100, seed, attn_dim=dim, attn_layers=1))
        for layers in (1, 2, 4):
            cells.append(cell("w2lyr", "ff+fixed+attn", 128, 100, seed, attn_dim=32, attn_layers=layers))
        cells.append(cell("w2dim", "ff+fixed", 128, 100, seed))  # shared control
    return cells


def wave3_scope():
    """T3 - the axes the 0.7378 ceiling is actually scoped on."""
    cells = []
    for seed in SEEDS:
        for hidden in (128, 256, 512, 1024):
            cells.append(cell("w3wid", "ff+fixed+attn", 128 if False else hidden, 400, seed,
                              attn_dim=32, attn_layers=1))
            cells.append(cell("w3wid", "ff+fixed", hidden, 400, seed))
        # channels-700 is the binding scope limit on the ceiling and is unrun at
        # convergence for any arm.
        cells.append(cell("w3geo", "ff+fixed+attn", 128, 400, seed, geometry="channels-700",
                          attn_dim=32, attn_layers=1))
        cells.append(cell("w3geo", "ff+fixed", 128, 400, seed, geometry="channels-700"))
        # Resolution invariance was the observation that started the whole
        # "rate coder" reading. Does attention break it?
        for contract in ("published-10ms", "fixed-t100", "fixed-t250", "fixed-t500"):
            cells.append(cell("w3con", "ff+fixed+attn", 128, 100, seed, contract=contract,
                              attn_dim=32, attn_layers=1))
            cells.append(cell("w3con", "ff+fixed", 128, 100, seed, contract=contract))
    return cells


def wave4_recurrent():
    """T4 - rec+alif is unmeasured, not refuted. Does the attention read-out change that?

    The surrogate-scale ladder and clipping are the two levers the record says
    were needed to get any recurrent cell to complete at all; both are carried
    here so a failure is attributable rather than mysterious.
    """
    cells = []
    for seed in SEEDS[:6]:
        for scale in (1.0, 0.4):
            cells.append(cell("w4rec", "rec+alif", 256, 100, seed, surrogate_scale=scale,
                              clip_grad_norm=1.0))
            cells.append(cell("w4rec", "rec+alif+attn", 256, 100, seed, attn_dim=32, attn_layers=1,
                              surrogate_scale=scale, clip_grad_norm=1.0))
    return cells


def wave5_budget_ladder():
    """W5 - the budget-invariant convergence test W1-4 could not provide.

    `PREREG_2026-08-19_SHD_ATTENTION_BUDGET_LADDER.md`, registered before any
    e800 cell existed. The e400 rung is wave 1; this adds e800 for both arms so
    the instrument's own final-doubling rule can be applied.
    """
    cells = []
    for seed in SEEDS:
        cells.append(cell("w5bud", "ff+fixed+attn", 128, 800, seed, attn_dim=32, attn_layers=1))
        cells.append(cell("w5bud", "ff+fixed", 128, 800, seed))
    return cells


def wave6_learning_curve():
    """W6 - the same-machine budget ladder.

    `PREREG_2026-08-19_SHD_ATTENTION_LEARNING_CURVE.md`, registered before any
    e20 or e50 cell existed. e100/e400/e800 already exist as waves 2/1/5; these
    are the two rungs that turn a cross-machine inference into a measurement.
    """
    cells = []
    for seed in SEEDS:
        for epochs in (20, 50):
            cells.append(cell("w6crv", "ff+fixed+attn", 128, epochs, seed,
                              attn_dim=32, attn_layers=1))
            cells.append(cell("w6crv", "ff+fixed", 128, epochs, seed))
    return cells


def wave7_ladder_floor():
    """W7 - lower the ladder's floor below e20.

    `PREREG_2026-08-20_SHD_ATTENTION_LADDER_FLOOR.md`, registered before any e5
    or e10 cell existed. Wave 6's "20 epochs" is its own floor, not a measured
    convergence point; these rungs are what turn an upper bound into a number -
    or, if the arm is converged at e5 too, into a lower upper bound honestly
    labelled as one.
    """
    cells = []
    for seed in SEEDS:
        for epochs in (5, 10):
            cells.append(cell("w7flr", "ff+fixed+attn", 128, epochs, seed,
                              attn_dim=32, attn_layers=1))
            cells.append(cell("w7flr", "ff+fixed", 128, epochs, seed))
    return cells


def wave8_headline_scope():
    """W8 - does the d32/L4 headline survive off the anchor?

    Everything the campaign knows about scope was measured at d32/**L1**: wave 3
    found the gain inverting by h1024 and failing seed-consistency on
    `channels-700`, both with a single attention block. The result the paper
    leads with is d32/**L4**. Carrying wave 3's scope limits over to it is an
    extrapolation across exactly the axis wave 2 showed matters most, so this
    wave measures them instead of assuming them.

    Controls are reused, not re-run: the matched `ff+fixed` cells at h512/h1024
    (`w3wid`) and at `channels-700` (`w3geo`) already exist at e400 from the same
    binary on the same fleet architecture, so a fresh copy would add cost and no
    information. `published-10ms` at e400 has no control on disk, so one is
    generated here.
    """
    cells = []
    for seed in SEEDS:
        # Geometry: the standard 700-channel SHD input, at the headline config.
        cells.append(cell("w8geo", "ff+fixed+attn", 128, 400, seed,
                          geometry="channels-700", attn_dim=32, attn_layers=4))
        # Width: does depth rescue the inversion wave 3 found at L1?
        for hidden in (512, 1024):
            cells.append(cell("w8wid", "ff+fixed+attn", hidden, 400, seed,
                              attn_dim=32, attn_layers=4))
        # Contract: the other literature-comparable contract, at convergence.
        cells.append(cell("w8con", "ff+fixed+attn", 128, 400, seed,
                          contract="published-10ms", attn_dim=32, attn_layers=4))
        cells.append(cell("w8con", "ff+fixed", 128, 400, seed, contract="published-10ms"))
        # Depth ladder at convergence: L1 (wave 1) and L4 (the registered run)
        # exist at e400, L2 does not - a hole exactly where the e100 ladder
        # showed its largest step.
        cells.append(cell("w8lyr", "ff+fixed+attn", 128, 400, seed,
                          attn_dim=32, attn_layers=2))
    return cells


def wave9_headline_mechanism():
    """W9 - the mechanism control for the configuration the paper leads with.

    The temporal-order claim rests on W1-3: the bin-shuffled arm was worse in
    12/12 seeds at e400. That control was measured at d32/**L1**. The headline is
    d32/**L4**. Wave 8 exists because carrying an L1 scope limit onto an L4 result
    is an extrapolation; carrying an L1 *mechanism control* onto it is the same
    error, applied to the claim rather than to its scope.

    `w9dim` asks the obvious follow-up wave 2 left open: dimension was monotone in
    the gain at e100 and only d32 was ever run at convergence, so d32 is the
    tested configuration, not the chosen one.

    Controls reused: `ff+fixed` bin-shuffled at h128/e400 on the anchor already
    exists from wave 1, same binary, same seeds.
    """
    cells = []
    for seed in SEEDS:
        cells.append(cell("w9shf", "ff+fixed+attn", 128, 400, seed,
                          attn_dim=32, attn_layers=4, temporal="bin-shuffled"))
        cells.append(cell("w9dim", "ff+fixed+attn", 128, 400, seed,
                          attn_dim=64, attn_layers=4))
    return cells


def wave10_resolution_ladder():
    """W10 - the temporal-resolution ladder, on the family that isolates it.

    Wave 8's S-5 compared `published-10ms` to `published-2ms`, which varies the
    timestep count, the bin width, AND the per-sample variability of `t` all at
    once. `fixed-tN` divides one fixed 1400 ms window into exactly N frames, so
    every sample has the same `t` and only the resolution moves: 14.0 / 5.6 / 2.8
    ms bins across a 5x span.

    Both arms are generated. Nothing is reused, because no `fixed-t*` cell exists
    at e400 for either arm anywhere in the record - the Azure campaign that
    registered them ran out of credit with all four of its control arms unrun.
    """
    cells = []
    for seed in SEEDS:
        for contract in ("fixed-t100", "fixed-t250", "fixed-t500"):
            cells.append(cell("w10con", "ff+fixed+attn", 128, 400, seed,
                              contract=contract, attn_dim=32, attn_layers=4))
            cells.append(cell("w10con", "ff+fixed", 128, 400, seed, contract=contract))
    return cells


WAVES = {
    "w1": wave1_converged,
    "w2": wave2_design_space,
    "w3": wave3_scope,
    "w4": wave4_recurrent,
    "w5": wave5_budget_ladder,
    "w6": wave6_learning_curve,
    "w7": wave7_ladder_floor,
    "w8": wave8_headline_scope,
    "w9": wave9_headline_mechanism,
    "w10": wave10_resolution_ladder,
}


def estimated_seconds(cell: dict) -> float:
    """Rough single-core cost, for scheduling only. Never used in analysis.

    Mirrors `estimate_cost.py`'s calibration: a 9.6 s/epoch base at h128 scaled
    by width, plus an attention term whose core scales with timesteps squared.
    Precision does not matter - only the ordering does.
    """
    timesteps = {"published-2ms": 358, "published-10ms": 72,
                 "fixed-t100": 100, "fixed-t250": 250, "fixed-t500": 500}
    t = timesteps[cell["contract"]]
    base = 9.6 * (cell["hidden"] / 128)
    if cell["geometry"] == "channels-700":
        base *= 1.15
    if cell["arm"].startswith("rec"):
        base *= 3.0 * (cell["hidden"] / 256)
    attention = 0.0
    if cell["attn_dim"]:
        core = 58.1 * 0.74 * (t / 358) ** 2 * (cell["attn_dim"] / 32) * cell["attn_layers"]
        io = 58.1 * 0.26 * (t / 358) * (cell["hidden"] / 128) * (cell["attn_dim"] / 32)
        attention = core + io
    return (base + attention) * cell["epochs"]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--waves", default="w1,w2,w3,w4,w5,w6,w7",
                        help="comma-separated subset of " + ",".join(WAVES))
    parser.add_argument("--out", default="-")
    parser.add_argument("--priority", default="w1,w6,w7",
                        help="comma-separated wave-label prefixes to schedule first. "
                             "Must match at least one cell or the plan is refused - "
                             "a priority set that silently matches nothing is how the "
                             "cheapest wave ended up queued last.")
    args = parser.parse_args()

    cells = []
    for name in args.waves.split(","):
        name = name.strip()
        if name not in WAVES:
            parser.error(f"unknown wave {name!r}; expected some of {sorted(WAVES)}")
        cells.extend(WAVES[name]())

    seen = {}
    for entry in cells:
        if entry["id"] in seen:
            raise SystemExit(f"duplicate cell id {entry['id']} - the plan is not injective")
        seen[entry["id"]] = entry

    # Longest-processing-time-first. `claim_next.py` takes the first unclaimed
    # cell in plan order, so plan order IS the schedule.
    #
    # Registration order puts the 8-14 hour h1024 cells near the end, where they
    # start after everything else has drained and each one then extends the
    # campaign by its full length. Sorting longest-first starts them immediately,
    # so they run underneath the short cells instead of after them, and the tail
    # is made of minutes rather than hours. This is the standard makespan result
    # and it is worth more here than any thread-count tuning.
    #
    # Purely a scheduling change: ids, seeds, arms and thresholds are untouched,
    # and a cell computes the same thing whenever it runs.
    # Wave 1 first, then longest-first across everything else.
    #
    # Pure longest-processing-time-first minimises makespan but buries wave 1 -
    # the primary contrast, and the only wave a paper can be written from -
    # behind the 26-hour h1024 cells. Finishing all 420 cells four hours sooner
    # is worth less than having the headline result a day earlier, and wave 1 is
    # only 60 cells, so front-loading it costs little of the makespan gain.
    #
    # Within each group, longest-first still applies, so each group's own tail is
    # short.
    # Prefix match, not exact: wave labels carry a suffix (`w6crv`, `w2dim`), so
    # an exact `in ("w1", "w6")` test silently never matched w6 and left the
    # cheapest, most decision-relevant wave queued at index 336 of 468.
    priority = tuple(p.strip() for p in args.priority.split(",") if p.strip())
    cells.sort(key=lambda c: (not c["wave"].startswith(priority), -estimated_seconds(c)))
    promoted = sum(1 for c in cells if c["wave"].startswith(priority))
    if promoted == 0:
        raise SystemExit(f"priority waves {priority} matched no cell - check the labels")
    print(f"scheduled first: {promoted} cells from waves {priority}", file=sys.stderr)

    payload = json.dumps(cells, indent=2) + "\n"
    if args.out == "-":
        sys.stdout.write(payload)
    else:
        with open(args.out, "w") as handle:
            handle.write(payload)
        print(f"{len(cells)} cells -> {args.out}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
