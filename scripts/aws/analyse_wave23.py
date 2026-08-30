#!/usr/bin/env python3
"""Wave 23 — is the h1024 collapse late, and does stopping early avoid it?

Frozen with `results/PREREG_2026-08-29_THE_COLLAPSE_IS_LATE.md` in the same
commit, before any cell of this wave existed.

Motivated by `results/FINDING_2026-08-29_THE_H1024_COLLAPSE_IS_A_LOST_FIT.md`,
which is POST-HOC on existing cells and is why this wave is needed rather than
being the answer itself.

# The comparison discipline

Every gain is **within-budget**: attention minus rate at the same epochs, seed
paired. The archived e400 gain is used at exactly one place, H23-2, and it is
compared gain-to-gain — never accuracy-to-accuracy across budgets, which would
be comparing two different amounts of training.

H23-3 is the control and it can make H23-1 and H23-2 uninformative without
refuting them. That is registered in advance: if `d32l2` improves by more than
its bar when truncated, then e400 is past the optimum for deep read-outs at this
width generally and nothing specific to the collapse has been shown. The
analyser prints that conclusion rather than leaving it to a reader.

Run: python3 scripts/aws/analyse_wave23.py
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

ARCHIVE_V2 = ROOT / "results/shd_attention_campaign_v2"

#: The preregistration's bars.
MIN_PAIRS = 10                 # of 12
H23_1_MIN_GAIN = 0.03
H23_1_MIN_POSITIVE = 9
H23_2_MIN_IMPROVEMENT = 0.10
H23_3_MAX_L2_SHIFT = 0.03
H23_4_MAX_LOST = 3             # of 12
RETENTION_FACTOR = 3.0         # fixed here; `fit_retention.py` shares it
TAIL_EPOCHS = 10

#: The archived e400 figure this wave is measured against. It is a CONSTANT of
#: the preregistration, not something recomputed here: recomputing it would let
#: the reference move under the comparison.
E400_L4_GAIN = -0.1318
E400_L2_GAIN = 0.0405

BUDGETS = (100, 200)
SCIENTIFIC_FIELDS = (
    "accuracy", "mean_loss", "mean_gradient_norm", "mean_update_rms",
    "mean_firing_rate", "majority_prediction", "classes_predicted",
    "silent_fraction", "saturated_fraction", "non_finite_events",
    "non_finite_forward", "epoch_mean_loss", "epoch_mean_gradient_norm",
    "epoch_max_gradient_norm", "tail_loss_improvement",
)

SEED = re.compile(r"__s(\d+)\.json$")
DEPTH = re.compile(r"__(d\d+l\d+)")


def index(root):
    """(epochs, arm, depth) -> {seed: (accuracy, trace)}, h1024 anchor only."""
    out = collections.defaultdict(dict)
    voided = collections.Counter()
    root = Path(root)
    if not root.is_dir():
        return out, voided
    for path in sorted(root.glob("*.json")):
        seed = SEED.search(path.name)
        if not seed:
            continue
        try:
            cell = json.loads(path.read_text())
        except ValueError:
            continue
        if not (isinstance(cell, dict) and "accuracy" in cell):
            continue
        if cell.get("hidden") != 1024 or cell.get("epochs") not in BUDGETS:
            continue
        if cell.get("contract") != "published-2ms":
            continue
        if cell.get("geometry") != "adjacent-sum-5":
            continue
        if cell.get("clip_grad_norm") is not None:
            continue
        if cell.get("surrogate_scale") not in (1.0, None):
            continue
        # Untouched by this wave and not comparable to it.
        if (cell.get("temporal_condition") or "intact") != "intact":
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
        key = (cell["epochs"], arm, depth)
        if validity_problems(cell):
            voided[key] += 1
            continue
        out[key].setdefault(int(seed.group(1)),
                            (cell["accuracy"], cell.get("epoch_mean_loss")))
    return out, voided


def gain(cells, epochs, depth):
    """(mean gain, positive, pairs) — attention minus rate at ONE budget."""
    attn = cells.get((epochs, "ff+fixed+attn", depth), {})
    rate = cells.get((epochs, "ff+fixed", None), {})
    shared = sorted(set(attn) & set(rate))
    if not shared:
        return None, 0, 0
    deltas = [attn[s][0] - rate[s][0] for s in shared]
    return statistics.fmean(deltas), sum(d > 0 for d in deltas), len(deltas)


def lost_fits(cells, epochs, depth):
    """(cells that lost their fit, cells measured) — H23-4."""
    arm = cells.get((epochs, "ff+fixed+attn", depth), {})
    lost = measured = 0
    for _, trace in arm.values():
        if not isinstance(trace, list) or len(trace) < TAIL_EPOCHS:
            continue
        if any(v != v or v in (float("inf"), float("-inf")) for v in trace):
            continue
        measured += 1
        best = min(trace)
        if statistics.fmean(trace[-TAIL_EPOCHS:]) > max(best, 1e-12) * RETENTION_FACTOR:
            lost += 1
    return lost, measured


def main() -> int:
    cells, voided = index(ARCHIVE_V2)

    print("# Wave 23 — is the h1024 collapse late?\n")
    print("Registered: `PREREG_2026-08-29_THE_COLLAPSE_IS_LATE.md`. "
          "This analyser is the authority on every verdict below.\n")

    print("## Gains, within budget\n")
    print("| budget | read-out | gain | positive | pairs |")
    print("|---:|---|---:|---:|---:|")
    table = {}
    for epochs in BUDGETS:
        for depth in ("d32l4", "d32l2"):
            value, positive, pairs = gain(cells, epochs, depth)
            table[(epochs, depth)] = (value, positive, pairs)
            shown = "—" if value is None else f"{value:+.4f}"
            print(f"| e{epochs} | {depth} | {shown} | {positive}/{pairs} | {pairs} |")

    print("\n## Verdicts\n")
    e100_l4, e100_l4_pos, e100_l4_pairs = table[(100, "d32l4")]
    if e100_l4_pairs < MIN_PAIRS:
        print(f"**H23-1: NOT EVALUABLE** — {e100_l4_pairs} seed pairs against a "
              f"floor of {MIN_PAIRS}.")
        h23_1 = None
    else:
        h23_1 = e100_l4 > H23_1_MIN_GAIN and e100_l4_pos >= H23_1_MIN_POSITIVE
        print(f"**H23-1: {'MET' if h23_1 else 'NOT MET'}** — gain at e100/d32l4 "
              f"{e100_l4:+.4f} against {H23_1_MIN_GAIN:+.2f}, positive in "
              f"{e100_l4_pos}/{e100_l4_pairs}.")
        if not h23_1:
            print("  → The late-collapse account is **REFUTED**. "
                  "`FINDING_2026-08-29_THE_H1024_COLLAPSE_IS_A_LOST_FIT.md` is "
                  "withdrawn, not reworded.")

    if e100_l4_pairs >= MIN_PAIRS:
        improvement = e100_l4 - E400_L4_GAIN
        h23_2 = improvement > H23_2_MIN_IMPROVEMENT
        print(f"\n**H23-2: {'MET' if h23_2 else 'NOT MET'}** — e100 gain "
              f"{e100_l4:+.4f} against the archived e400 gain {E400_L4_GAIN:+.4f}, "
              f"an improvement of {improvement:+.4f} against "
              f"{H23_2_MIN_IMPROVEMENT:+.2f}.")

    e100_l2, _, e100_l2_pairs = table[(100, "d32l2")]
    if e100_l2_pairs < MIN_PAIRS:
        print(f"\n**H23-3: NOT EVALUABLE** — {e100_l2_pairs} seed pairs on the "
              f"d32l2 control. **H23-1 and H23-2 are therefore uninterpretable "
              f"about the collapse**, whatever their own verdicts say.")
    else:
        shift = e100_l2 - E400_L2_GAIN
        h23_3 = abs(shift) <= H23_3_MAX_L2_SHIFT
        print(f"\n**H23-3: {'MET' if h23_3 else 'NOT MET'}** — d32l2 gain moves "
              f"{shift:+.4f} between e400 and e100, against ±{H23_3_MAX_L2_SHIFT:.2f}.")
        if not h23_3:
            print("  → e400 is past the optimum for deep read-outs at h1024 "
                  "**generally**. Nothing specific to the collapse is "
                  "established, and H23-1/H23-2 are reported as a budget "
                  "finding rather than an explanation.")

    lost, measured = lost_fits(cells, 100, "d32l4")
    if measured < MIN_PAIRS:
        print(f"\n**H23-4: NOT EVALUABLE** — {measured} usable traces.")
    else:
        h23_4 = lost <= H23_4_MAX_LOST
        print(f"\n**H23-4: {'MET' if h23_4 else 'NOT MET'}** — {lost}/{measured} "
              f"e100/d32l4 cells end above {RETENTION_FACTOR:g}x their own best "
              f"training loss, against a bar of {H23_4_MAX_LOST}. At e400 it is "
              f"63/68.")
        if h23_4 and h23_1 is False:
            print("  → The fit is retained and the gain still does not appear. "
                  "Retention is **not sufficient**, and the accuracy loss is not "
                  "the fit loss. This is the outcome that needs its own wave.")

    if voided:
        print("\n## Voided cells\n")
        for key, count in sorted(voided.items(), key=lambda kv: str(kv[0])):
            print(f"- {count} × {key}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
