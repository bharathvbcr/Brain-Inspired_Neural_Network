#!/usr/bin/env python3
"""Wave 21 — the temporal-order mechanism away from h128.

Frozen with `results/PREREG_2026-08-27_THE_MECHANISM_ACROSS_THE_DESIGN_SPACE.md`,
before the first cell of the wave existed.

The statistic is a difference **of differences**, and every part of that matters:

    DiD(x) = (attention intact − attention shuffled)
           − (rate intact      − rate shuffled)

seed-paired at every step. A cell missing from any of the four arms removes that
seed from all four, because a DiD computed over four differently-populated arms
is four unrelated means wearing a subtraction. The intact halves come from the
corpus; only the shuffled halves are new.

* **Nine seed-paired quadruples is the floor**, per the preregistration's
  stopping rule. Below it an operating point carries no numbers at all -- not a
  mean, not a direction -- because a four-arm intersection thins fast and a DiD
  over five seeds is not a measurement.
* **H21-3 needs every width.** A rank correlation over six rungs is already a
  weak instrument; computed over whichever rungs happened to survive it is not
  an instrument at all. If any width is NOT EVALUABLE, H21-3 is NOT EVALUABLE.
"""

from __future__ import annotations

import argparse
import collections
import json
import math
import re
import statistics
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from cell_validity import validity_problems  # noqa: E402

ARCHIVE_V1 = ROOT / "results/shd_attention_campaign_v1/cells"
ARCHIVE_V2 = ROOT / "results/shd_attention_campaign_v2"

#: The preregistration's bars. Changing one changes what was registered.
MIN_PAIRS = 9                  # per operating point
H21_1_MIN_DID = 0.03
H21_1_MIN_POSITIVE = 9         # of 12
H21_2_MAX_DID = 0.02           # at h1024
H21_3_MIN_RHO = 0.829          # Spearman critical value, n=6, one-tailed, 0.05
H21_4_MIN_DID = 0.03
H21_4_MIN_POSITIVE = 9

ANCHOR = ("published-2ms", "adjacent-sum-5")
LADDER = (128, 256, 384, 512, 768, 1024)
H21_1_WIDTHS = (256, 384, 512)
H21_4_POINTS = ((128, "published-2ms", "channels-700"),
                (128, "published-10ms", "adjacent-sum-5"))

SEED = re.compile(r"__s(\d+)\.json$")
DEPTH = re.compile(r"__(d\d+l\d+)")


def spearman(xs, ys):
    """Rank correlation. None below four points, where it is not a number worth
    printing rather than a number worth doubting.

    Kept byte-identical to `analyse_wave20.spearman`; both are frozen analysers
    so neither imports the other, and `test_wave21_analyser.py` pins that they
    agree on the same inputs.
    """
    n = len(xs)
    if n < 4:
        return None

    def rank(values):
        order = sorted(range(n), key=lambda i: values[i])
        ranks = [0.0] * n
        i = 0
        while i < n:                       # average ties, or a plateau in the
            j = i                          # covariate biases rho toward zero
            while j + 1 < n and values[order[j + 1]] == values[order[i]]:
                j += 1
            shared = (i + j) / 2
            for k in range(i, j + 1):
                ranks[order[k]] = shared
            i = j + 1
        return ranks

    rx, ry = rank(xs), rank(ys)
    mx, my = statistics.fmean(rx), statistics.fmean(ry)
    num = sum((a - mx) * (b - my) for a, b in zip(rx, ry))
    den = math.sqrt(sum((a - mx) ** 2 for a in rx) * sum((b - my) ** 2 for b in ry))
    return num / den if den else None


def index(roots):
    """(hidden, contract, geometry, arm, temporal) -> {seed: accuracy}.

    Only d32/L4 attention cells and rate cells enter; other read-out depths are
    outside this wave and merging one in is the H17-2 defect.
    """
    out = collections.defaultdict(dict)
    voided = collections.Counter()
    for root in roots:
        root = Path(root)
        if not root.is_dir():
            continue
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
            if cell.get("epochs") != 400 or cell.get("clip_grad_norm") is not None:
                continue
            if cell.get("surrogate_scale") not in (1.0, None):
                continue
            arm = cell.get("arm")
            depth = DEPTH.search(path.name)
            if arm == "ff+fixed+attn":
                if not depth or depth.group(1) != "d32l4":
                    continue
            elif arm == "ff+fixed":
                if depth:
                    continue
            else:
                continue
            key = (cell.get("hidden"), cell.get("contract"), cell.get("geometry"),
                   arm, cell.get("temporal_condition") or "intact")
            if validity_problems(cell):
                voided[key] += 1
                continue
            out[key].setdefault(int(seed.group(1)), cell["accuracy"])
    return out, voided


def did(cells, hidden, contract, geometry):
    """(mean DiD, positive count, pairs) at one operating point, seed-paired."""
    def arm(a, t):
        return cells.get((hidden, contract, geometry, a, t), {})
    ai = arm("ff+fixed+attn", "intact")
    as_ = arm("ff+fixed+attn", "bin-shuffled")
    ri = arm("ff+fixed", "intact")
    rs = arm("ff+fixed", "bin-shuffled")
    shared = sorted(set(ai) & set(as_) & set(ri) & set(rs))
    if not shared:
        return None, 0, 0
    deltas = [(ai[s] - as_[s]) - (ri[s] - rs[s]) for s in shared]
    return statistics.fmean(deltas), sum(d > 0 for d in deltas), len(deltas)


def gain(cells, hidden, contract, geometry):
    """The intact attention gain, for H21-3's covariate. Corpus-only.

    Carries its own pair count and its own floor. The gain is paired over the
    two INTACT arms, so it survives an operating point whose shuffled arms are
    missing -- but the stopping rule says an operating point below the floor
    carries no numbers, and a gain printed beside a suppressed DiD is exactly
    the number a reader would quote.
    """
    ai = cells.get((hidden, contract, geometry, "ff+fixed+attn", "intact"), {})
    ri = cells.get((hidden, contract, geometry, "ff+fixed", "intact"), {})
    shared = sorted(set(ai) & set(ri))
    if len(shared) < MIN_PAIRS:
        return None
    return statistics.fmean(ai[s] - ri[s] for s in shared)


def verdict(value, pairs, floor, positive, min_positive, name):
    if pairs < MIN_PAIRS:
        return (f"**{name}: NOT EVALUABLE** — {pairs} seed-paired quadruple(s) "
                f"against a floor of {MIN_PAIRS}. No number is reported.", None)
    met = value >= floor and positive >= min_positive
    return (f"**{name}: {'MET' if met else 'NOT MET'}** — DiD {value:+.4f} "
            f"against {floor:+.2f}, positive in {positive}/{pairs}.", met)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--results", required=True)
    parser.add_argument("--archive", action="append", default=[])
    args = parser.parse_args()

    roots = [Path(args.results)] + \
        ([Path(a) for a in args.archive] or [ARCHIVE_V2, ARCHIVE_V1])
    cells, voided = index(roots)

    lines = ["# Wave 21 — the temporal-order mechanism away from h128", "",
             "Bars from `PREREG_2026-08-27_THE_MECHANISM_ACROSS_THE_DESIGN_SPACE.md`.",
             ""]
    if voided:
        lines.append(f"**{sum(voided.values())} cell(s) voided by "
                     f"`cell_validity`** and excluded from every arm they "
                     f"appear in.")
        lines.append("")

    points = [(h, *ANCHOR) for h in LADDER] + list(H21_4_POINTS)
    table = {}
    lines += ["| operating point | quadruples | gain | DiD | positive |",
              "|---|---:|---:|---:|---:|"]
    for hidden, contract, geometry in points:
        value, positive, pairs = did(cells, hidden, contract, geometry)
        table[(hidden, contract, geometry)] = (value, positive, pairs)
        g = gain(cells, hidden, contract, geometry)
        lines.append(
            f"| h{hidden} / `{contract}` / `{geometry}` | {pairs} | "
            f"{'—' if g is None else f'{g:+.4f}'} | "
            f"{'—' if pairs < MIN_PAIRS or value is None else f'{value:+.4f}'} | "
            f"{'—' if pairs < MIN_PAIRS else f'{positive}/{pairs}'} |")

    # --- H21-1 --------------------------------------------------------------
    lines += ["", "## H21-1 — is the mechanism unique to h128?", ""]
    results = []
    for hidden in H21_1_WIDTHS:
        value, positive, pairs = table[(hidden, *ANCHOR)]
        line, met = verdict(value, pairs, H21_1_MIN_DID, positive,
                            H21_1_MIN_POSITIVE, f"h{hidden}")
        lines.append(f"- {line}")
        results.append(met)
    if None in results:
        lines.append("\n**H21-1: NOT EVALUABLE** — at least one width is below "
                     "the pair floor, and the hypothesis is over all three.")
    else:
        lines.append(f"\n**H21-1: {'MET' if all(results) else 'NOT MET'}** — "
                     f"{sum(bool(r) for r in results)}/3 widths clear both bars; "
                     f"the hypothesis requires all three.")

    # --- H21-2 --------------------------------------------------------------
    lines += ["", "## H21-2 — does the shuffle cost collapse where the gain "
              "inverts?", ""]
    value, positive, pairs = table[(1024, *ANCHOR)]
    if pairs < MIN_PAIRS:
        lines.append(f"**H21-2: NOT EVALUABLE** — {pairs} quadruple(s) against "
                     f"a floor of {MIN_PAIRS}.")
    else:
        met = value <= H21_2_MAX_DID
        lines.append(f"**H21-2: {'MET' if met else 'NOT MET'}** — DiD "
                     f"{value:+.4f} against a ceiling of {H21_2_MAX_DID:+.2f}. "
                     f"The bar is one-sided and a negative DiD satisfies it: the "
                     f"prediction is the absence of an order-dependent benefit, "
                     f"not its sign.")

    # --- H21-3 --------------------------------------------------------------
    lines += ["", "## H21-3 — does the shuffle cost track the gain?", ""]
    gains, dids, missing = [], [], []
    for hidden in LADDER:
        value, _, pairs = table[(hidden, *ANCHOR)]
        g = gain(cells, hidden, *ANCHOR)
        if pairs < MIN_PAIRS or g is None:
            missing.append(hidden)
            continue
        gains.append(g)
        dids.append(value)
    if missing:
        lines.append(f"**H21-3: NOT EVALUABLE** — width(s) "
                     f"{', '.join(f'h{h}' for h in missing)} are below the pair "
                     f"floor. A rank correlation over six rungs is already weak; "
                     f"over whichever rungs survived it is not an instrument.")
    else:
        rho = spearman(gains, dids)
        met = rho is not None and rho >= H21_3_MIN_RHO
        lines.append(f"**H21-3: {'MET' if met else 'NOT MET'}** — Spearman ρ "
                     f"{rho:+.3f} over {len(gains)} widths against a bar of "
                     f"{H21_3_MIN_RHO:+.3f}, the n=6 one-tailed critical value "
                     f"at α=0.05. A ρ below it is not a trend and is not "
                     f"reported as one.")

    # --- H21-4 --------------------------------------------------------------
    lines += ["", "## H21-4 — does the mechanism survive a change of binning?", ""]
    results = []
    for hidden, contract, geometry in H21_4_POINTS:
        value, positive, pairs = table[(hidden, contract, geometry)]
        line, met = verdict(value, pairs, H21_4_MIN_DID, positive,
                            H21_4_MIN_POSITIVE, f"`{contract}` / `{geometry}`")
        lines.append(f"- {line}")
        results.append(met)
    if None in results:
        lines.append("\n**H21-4: NOT EVALUABLE** — at least one point is below "
                     "the pair floor.")
    else:
        lines.append(f"\n**H21-4: {'MET' if all(results) else 'NOT MET'}** — "
                     f"the hypothesis requires both points.")

    lines += ["", "---", "",
              "Cross-machine Gate F FAILs macOS-vs-Linux on every node of this "
              "campaign by design. Every contrast above is between arms that "
              "ran on the same fleet from the same pinned binary."]
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
