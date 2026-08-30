#!/usr/bin/env python3
"""Does an arm keep the fit it reached, or lose it?

# Why this exists

`PAPER_DRAFT.md` §3.5 calls the h1024 gain inversion **located but
unexplained**, and the wave-21 preregistration registers it as the paper's
leading open problem. One alternative account — overfitting on 8,156 training
samples — is recorded as *"neither excluded nor supported"*, because the
argument for it was conditional on a collapse that did not occur.

It is answerable from cells already on disk, and this script is that answer.
Every cell carries `epoch_mean_loss`, the **training** loss per epoch. That
separates the two accounts directly:

* **overfitting** — the arm fits the training set at least as well as the arms
  that generalise, and does worse on test. Final training loss LOW.
* **loss of fit** — the arm reaches a fit and does not hold it. Final training
  loss HIGH, and higher than its own best.

These are not the same failure and they do not have the same remedy.

# What this is NOT

**Post-hoc, on cells that already existed.** It is not a registered verdict and
must never be transcribed as one. What it is for is deciding what to
preregister: a prediction derived here is testable by a wave, and
`PREREG_2026-08-29_THE_COLLAPSE_IS_LATE.md` is that wave.

Run: python3 scripts/fit_retention.py [--width 1024]
"""

from __future__ import annotations

import argparse
import json
import statistics as st
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CORPUS = ROOT / "results/shd_attention_campaign_v2"

#: Epochs averaged to define "where the arm ended up". One epoch is noisy and a
#: single final value would make the verdict depend on which epoch training
#: happened to stop at.
TAIL_EPOCHS = 10

#: `final > best * RETENTION_FACTOR` is "the fit was lost". Three is loose on
#: purpose: these losses span four orders of magnitude between arms, so the
#: question is never a near-thing, and a tight factor would invite tuning it.
RETENTION_FACTOR = 3.0

#: A corpus this size cannot be read from too few cells without the reader
#: mistaking a slice for the whole. Below this the script refuses rather than
#: reporting a confident summary of almost nothing.
MIN_CELLS = 20


def configuration(cell: dict) -> str:
    """The arm, without its seed. Two cells share one iff they are the same
    experiment at different seeds."""
    if cell["arm"] == "ff+fixed":
        readout = "rate"
    else:
        readout = f"d{cell['attn_dim']}l{cell['attn_layers']}"
    return (f"{cell['arm']} h{cell['hidden']} e{cell['epochs']} {readout} "
            f"{cell['contract']} {cell['geometry']} {cell.get('temporal_condition', 'intact')}")


def retention(cell: dict) -> dict | None:
    """One cell's fit trajectory, or None if it carries no usable trace."""
    trace = cell.get("epoch_mean_loss")
    if not isinstance(trace, list) or len(trace) < TAIL_EPOCHS:
        return None
    finite = [v for v in trace if isinstance(v, (int, float)) and v == v
              and v not in (float("inf"), float("-inf"))]
    # A trace that went non-finite is a different failure and is not a
    # retention measurement; it is reported as its own count rather than
    # silently averaged into one.
    if len(finite) != len(trace):
        return {"non_finite_trace": True}
    best = min(trace)
    tail = st.fmean(trace[-TAIL_EPOCHS:])
    return {
        "non_finite_trace": False,
        "accuracy": cell.get("accuracy"),
        "best": best,
        "best_epoch": trace.index(best),
        "final": tail,
        # `best` can be 0.0 in principle; the guard keeps the ratio defined.
        "lost": tail > max(best, 1e-12) * RETENTION_FACTOR,
    }


def load(width: int | None) -> dict[str, list[dict]]:
    groups: dict[str, list[dict]] = defaultdict(list)
    for path in sorted(CORPUS.glob("*.json")):
        try:
            cell = json.loads(path.read_text())
        except (json.JSONDecodeError, OSError):
            continue
        # The corpus directory also holds analyser output and manifests, which
        # are lists and dicts of other shapes. Schema-gating on a non-dict would
        # raise rather than skip, so the type check comes first.
        if not isinstance(cell, dict) or cell.get("schema") != "shd-cal-cell-v1":
            continue
        if width is not None and cell.get("hidden") != width:
            continue
        measured = retention(cell)
        if measured is None:
            continue
        groups[configuration(cell)].append(measured)
    return groups


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--width", type=int, default=1024,
                        help="hidden width to report; omit for all")
    parser.add_argument("--all-widths", action="store_true")
    args = parser.parse_args()

    width = None if args.all_widths else args.width
    groups = load(width)
    total = sum(len(v) for v in groups.values())
    if total < MIN_CELLS:
        print(f"{total} cells matched, below the floor of {MIN_CELLS}. Either the "
              f"corpus is not where this expects it ({CORPUS}) or the filter is "
              f"too narrow. Refusing to summarise a slice as if it were the set.")
        return 1

    print(f"Fit retention — {total} cells"
          f"{'' if width is None else f', h{width}'}. "
          f"POST-HOC on existing cells; not a registered verdict.\n")
    print(f"{'configuration':58} {'n':>3} {'acc':>7} {'best':>9} {'final':>9} {'lost':>7}")
    for name in sorted(groups):
        rows = [r for r in groups[name] if not r["non_finite_trace"]]
        if not rows:
            print(f"{name:58} {len(groups[name]):3}   every trace non-finite")
            continue
        lost = sum(r["lost"] for r in rows)
        print(f"{name:58} {len(rows):3} "
              f"{st.fmean(r['accuracy'] for r in rows):7.4f} "
              f"{st.fmean(r['best'] for r in rows):9.4f} "
              f"{st.fmean(r['final'] for r in rows):9.4f} "
              f"{lost:3}/{len(rows):<3}")

    print(f"\n'lost' counts cells whose final training loss (mean of the last "
          f"{TAIL_EPOCHS} epochs) exceeds {RETENTION_FACTOR:g}x their own best.")
    print("An arm that OVERFITS keeps a low final training loss. An arm that "
          "loses its fit does not.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
