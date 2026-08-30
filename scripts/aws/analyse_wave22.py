#!/usr/bin/env python3
"""Wave 22 — the mechanism control at the twelve operating points that lacked it.

Frozen with `results/PREREG_2026-08-29_THE_MECHANISM_AT_EVERY_OPERATING_POINT.md`
in the same commit, before any cell of this wave existed. Every bar below is
that document's; changing one changes what was registered.

# The one way this differs from `analyse_wave21.py`, and it is load-bearing

Wave 21 varied width, contract and geometry at a **single read-out depth**, and
its `index()` filters to `d32l4` — because merging two read-out depths into one
comparison is precisely the H17-2 defect
(`AMENDMENT_2026-08-27_H17_2_MERGED_TWO_READOUT_DEPTHS.md`), which inflated a
published shuffle cost by 17%.

**Wave 22 varies the read-out depth on purpose**: six of its twelve points are
`d32/L1`, `d32/L2`, `d32/L3` or `d64/L4`. So the depth cannot be filtered out —
it has to be part of the key. A depth carried in the key can never be merged;
a depth dropped from it merges silently and looks like a cleaner result.

The rate arm carries no read-out depth, and that asymmetry is real rather than
an oversight: one `ff+fixed` bin-shuffled cell is the correct control for every
depth at a given (width, contract, geometry). It is keyed with depth `None`.

Run: python3 scripts/aws/analyse_wave22.py
"""

from __future__ import annotations

import collections
import json
import re
import statistics
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from cell_validity import validity_problems  # noqa: E402

ARCHIVE_V1 = ROOT / "results/shd_attention_campaign_v1/cells"
ARCHIVE_V2 = ROOT / "results/shd_attention_campaign_v2"

#: The preregistration's bars.
MIN_PAIRS = 9                  # of 12, per operating point
H22_1_MIN_DID = 0.03
H22_1_MIN_POSITIVE = 9
H22_3_MAX_DEPTH_RANGE = 0.10   # registered as a question, not a prediction
COVERAGE_TARGET = 21

#: Every field this analyser reads as a measurement. Includes
#: `non_finite_forward`, which `gate_f_rust.py::COMPARED_FIELDS` deliberately
#: does not carry — see the note there. The invariant is one-directional: an
#: analyser may check more than Gate F, never less.
SCIENTIFIC_FIELDS = (
    "accuracy", "mean_loss", "mean_gradient_norm", "mean_update_rms",
    "mean_firing_rate", "majority_prediction", "classes_predicted",
    "silent_fraction", "saturated_fraction", "non_finite_events",
    "non_finite_forward", "epoch_mean_loss", "epoch_mean_gradient_norm",
    "epoch_max_gradient_norm", "tail_loss_improvement",
)

ANCHOR = ("published-2ms", "adjacent-sum-5")

#: The twelve points, as (hidden, contract, geometry, depth). `depth` is the
#: attention arm's read-out; the rate control at each is depth-independent.
POINTS = (
    (128, "fixed-t100", ANCHOR[1], "d32l4"),
    (128, "fixed-t250", ANCHOR[1], "d32l4"),
    (128, "fixed-t500", ANCHOR[1], "d32l4"),
    (128, ANCHOR[0], ANCHOR[1], "d32l2"),
    (128, ANCHOR[0], ANCHOR[1], "d64l4"),
    (128, ANCHOR[0], "channels-700", "d32l1"),
    (256, ANCHOR[0], ANCHOR[1], "d32l1"),
    (512, ANCHOR[0], ANCHOR[1], "d32l1"),
    (768, ANCHOR[0], ANCHOR[1], "d32l2"),
    (1024, ANCHOR[0], ANCHOR[1], "d32l1"),
    (1024, ANCHOR[0], ANCHOR[1], "d32l2"),
    (1024, ANCHOR[0], ANCHOR[1], "d32l3"),
)

#: H22-3's comparison: each of these against its `d32l4` twin at the same width,
#: on the anchor. Registered as a question — the campaign has never varied depth
#: with the shuffle control present.
DEPTH_POINTS = ((128, "d32l2"), (128, "d64l4"), (256, "d32l1"),
                (512, "d32l1"), (768, "d32l2"))

#: This wave is SELF-CONTAINED and only its own cells may enter.
#:
#: The corpus was produced by pinned binary `22d97c51ab02...`, which predates
#: the 2026-08-29 forward-finiteness guard. Wave 22 runs on a new pinned binary,
#: so an archived intact cell and a wave-22 intact cell at the same operating
#: point and seed are DIFFERENT experiments that share every key field. Loading
#: both, `setdefault` would keep whichever the filesystem sorted first — a
#: silent mix of two binaries inside one difference-of-differences, decided by
#: filename order. Requiring the wave label makes that impossible rather than
#: unlikely.
WAVE = "w22cov"

SEED = re.compile(r"__s(\d+)\.json$")
DEPTH = re.compile(r"__(d\d+l\d+)")


def index(roots):
    """(hidden, contract, geometry, arm, depth, temporal) -> {seed: accuracy}.

    `depth` is `None` for the rate arm and the `dNlM` token otherwise. Keying on
    it is what stops two read-out depths being averaged into one contrast.

    Only `WAVE` cells enter. See the note at `WAVE` for why an archived cell at
    the same operating point is not an acceptable substitute here, even though
    wave 21's analyser accepted exactly that.
    """
    out = collections.defaultdict(dict)
    voided = collections.Counter()
    for root in roots:
        root = Path(root)
        if not root.is_dir():
            continue
        for path in sorted(root.glob("*.json")):
            if not path.name.startswith(f"{WAVE}__"):
                continue
            seed = SEED.search(path.name)
            if not seed:
                continue
            try:
                cell = json.loads(path.read_text())
            except ValueError:
                continue
            if not (isinstance(cell, dict) and "accuracy" in cell):
                continue
            # The wave is e400 at the default surrogate scale with no clipping.
            # A rescue-lever cell shares an operating point with a wave cell and
            # is a different experiment.
            if cell.get("epochs") != 400 or cell.get("clip_grad_norm") is not None:
                continue
            if cell.get("surrogate_scale") not in (1.0, None):
                continue
            arm = cell.get("arm")
            found = DEPTH.search(path.name)
            if arm == "ff+fixed+attn":
                if not found:
                    continue
                depth = found.group(1)
            elif arm == "ff+fixed":
                if found:
                    continue
                depth = None
            else:
                continue
            key = (cell.get("hidden"), cell.get("contract"), cell.get("geometry"),
                   arm, depth, cell.get("temporal_condition") or "intact")
            if validity_problems(cell):
                voided[key] += 1
                continue
            out[key].setdefault(int(seed.group(1)), cell["accuracy"])
    return out, voided


def did(cells, hidden, contract, geometry, depth):
    """(mean DiD, positive count, pairs) at one point, seed-paired quadruples.

    The rate arm is looked up at depth `None` — it has no read-out — while the
    attention arm is looked up at `depth`. That pairing is the whole contrast.
    """
    ai = cells.get((hidden, contract, geometry, "ff+fixed+attn", depth, "intact"), {})
    as_ = cells.get((hidden, contract, geometry, "ff+fixed+attn", depth, "bin-shuffled"), {})
    ri = cells.get((hidden, contract, geometry, "ff+fixed", None, "intact"), {})
    rs = cells.get((hidden, contract, geometry, "ff+fixed", None, "bin-shuffled"), {})
    shared = sorted(set(ai) & set(as_) & set(ri) & set(rs))
    if not shared:
        return None, 0, 0
    deltas = [(ai[s] - as_[s]) - (ri[s] - rs[s]) for s in shared]
    return statistics.fmean(deltas), sum(d > 0 for d in deltas), len(deltas)


def main() -> int:
    cells, voided = index([ARCHIVE_V1, ARCHIVE_V2])

    print("# Wave 22 — the mechanism control at every operating point\n")
    print("Registered: `PREREG_2026-08-29_THE_MECHANISM_AT_EVERY_OPERATING_POINT.md`.")
    print("This analyser is the authority on every verdict below.\n")

    print("## H22-1 — the contrast clears its bar at each new point\n")
    print("| point | read-out | DiD | positive | pairs | verdict |")
    print("|---|---|---:|---:|---:|---|")
    results = {}
    evaluable = 0
    for hidden, contract, geometry, depth in POINTS:
        value, positive, pairs = did(cells, hidden, contract, geometry, depth)
        results[(hidden, contract, geometry, depth)] = (value, positive, pairs)
        if pairs < MIN_PAIRS:
            print(f"| h{hidden} / {contract} / {geometry} | {depth} | — | — | "
                  f"{pairs} | **NOT EVALUABLE** |")
            continue
        evaluable += 1
        met = value >= H22_1_MIN_DID and positive >= H22_1_MIN_POSITIVE
        print(f"| h{hidden} / {contract} / {geometry} | {depth} | {value:+.4f} | "
              f"{positive}/{pairs} | {pairs} | **{'MET' if met else 'NOT MET'}** |")

    print(f"\n{evaluable} of {len(POINTS)} points evaluable "
          f"(floor {MIN_PAIRS} seed-paired quadruples each).\n")

    print("## H22-3 — does the contrast depend on read-out depth?\n")
    print("Registered as a QUESTION, not a prediction. Each point against its "
          f"`d32l4` twin at the same width, on the anchor; range bar "
          f"{H22_3_MAX_DEPTH_RANGE:.2f}.\n")
    print("| width | depth | DiD | d32l4 DiD | difference |")
    print("|---:|---|---:|---:|---:|")
    differences = []
    for hidden, depth in DEPTH_POINTS:
        here = results.get((hidden, ANCHOR[0], ANCHOR[1], depth))
        twin, _, twin_pairs = did(cells, hidden, ANCHOR[0], ANCHOR[1], "d32l4")
        if not here or here[2] < MIN_PAIRS or twin is None or twin_pairs < MIN_PAIRS:
            print(f"| h{hidden} | {depth} | — | — | not evaluable |")
            continue
        difference = here[0] - twin
        differences.append(difference)
        print(f"| h{hidden} | {depth} | {here[0]:+.4f} | {twin:+.4f} | {difference:+.4f} |")
    if differences:
        spread = max(differences) - min(differences)
        met = spread <= H22_3_MAX_DEPTH_RANGE
        print(f"\n**H22-3: {'MET' if met else 'NOT MET'}** — range {spread:.4f} "
              f"against {H22_3_MAX_DEPTH_RANGE:.2f}. NOT MET means the lead claim "
              f"is scoped to a read-out depth as well as to a width.")
    else:
        print("\n**H22-3: NOT EVALUABLE** — no depth point cleared the pair floor.")

    if voided:
        print("\n## Voided cells\n")
        for key, count in sorted(voided.items(), key=lambda kv: str(kv[0])):
            print(f"- {count} × {key}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
