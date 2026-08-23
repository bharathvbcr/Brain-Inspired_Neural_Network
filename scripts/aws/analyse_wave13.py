#!/usr/bin/env python3
"""Wave 13 verdicts, exactly as registered in
`results/PREREG_2026-08-23_RECURRENT_STABILITY.md`.

Frozen before the first cell landed.

This analyser counts **completions**, so unlike waves 8–12 a missing cell is the
measurement rather than a reason to refuse. `analyse_wave8.load` raises on a
missing cell — correct there, wrong here — so this carries its own loader and
uses `cell_validity` for the gate, keeping the single owner.

Two traps from wave 11's frozen analyser are avoided deliberately, and
`test_wave13_analyser.py` reproduces both against a synthetic grid rather than a
tidied version of them:

  * `surrogate_scale` is an f32 field that records `1.0` as `1.0` but `0.4` as
    `0.400000006`. Grouping on `== 0.4` silently matches nothing. Conditions are
    keyed off the **plan**, never off a float comparison.
  * The seed is taken from the cell **id**, which the plan defines, not from a
    `cell["seed"]` field.

    python3 scripts/aws/analyse_wave13.py --plan results/shd_attention_campaign_v2/plan_w13.json \\
        --results results/shd_attention_campaign_v2 [--failures DIR] [--out FILE]
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from pathlib import Path

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
from cell_validity import stability_warnings, validity_problems  # noqa: E402

#: Registered thresholds. Named so a reader can check them against the prereg
#: without reading the code that applies them.
R1_COMPLETIONS = 11
R1_OF = 12
R2_R3_DELTA = 6

#: `optimizer step N` in a failure log is where the per-sample guard fired.
ABORT_STEP = re.compile(r"non-finite training value at optimizer step (\d+)")


def condition(spec: dict) -> tuple[str, float]:
    """(arm, surrogate scale) straight from the plan entry.

    The plan is the authority for which condition a cell belongs to. The cell's
    own `surrogate_scale` is an f32 and cannot be compared to `0.4`.
    """
    return spec["arm"], spec["surrogate_scale"]


def load_outcomes(plan: list[dict], results: Path, failures: Path | None) -> dict:
    """Per cell: completed / voided / diverged, and why."""
    out = {}
    for spec in plan:
        path = results / f"{spec['id']}.json"
        if not path.is_file():
            reason = "no cell emitted"
            step = None
            if failures is not None:
                log = failures / f"{spec['id']}.log"
                if log.is_file():
                    match = ABORT_STEP.search(log.read_text(errors="replace"))
                    if match:
                        step = int(match.group(1))
                        reason = f"diverged at optimizer step {step}"
            out[spec["id"]] = {"state": "diverged", "why": reason, "step": step}
            continue
        cell = json.loads(path.read_text())
        problems = validity_problems(cell, spec)
        if problems:
            out[spec["id"]] = {
                "state": "voided",
                "why": "; ".join(problems),
                "cell": cell,
                "step": None,
            }
            continue
        out[spec["id"]] = {"state": "completed", "why": "", "cell": cell, "step": None}
    return out


def peak_norm(cell: dict) -> float | None:
    trace = cell.get("epoch_max_gradient_norm")
    if not isinstance(trace, list):
        return None
    finite = [v for v in trace if isinstance(v, (int, float)) and not isinstance(v, bool)]
    finite = [v for v in finite if v == v and abs(v) != float("inf")]
    return max(finite) if finite else None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", required=True)
    parser.add_argument("--results", required=True)
    parser.add_argument("--failures")
    parser.add_argument("--out", default="-")
    args = parser.parse_args()

    plan = json.load(open(args.plan))
    outcomes = load_outcomes(
        plan, Path(args.results), Path(args.failures) if args.failures else None
    )

    conditions: dict[tuple[str, float], list[dict]] = {}
    for spec in plan:
        conditions.setdefault(condition(spec), []).append(outcomes[spec["id"]])

    lines: list[str] = []

    def w(text=""):
        lines.append(text)

    w("# Wave 13 — recurrent stability at the anchor budget\n")
    w("Prereg: `PREREG_2026-08-23_RECURRENT_STABILITY.md` §3. 48 cells, "
      "h128 / `published-2ms` / `adjacent-sum-5` / e400, same pinned binary.\n")
    w("A cell **completes** iff it was emitted *and* passes the validity gate, "
      "which includes `non_finite_events == 0`.\n")

    w("## Completion\n")
    w("| arm | surrogate scale | completed | voided | diverged |")
    w("|---|---:|---:|---:|---:|")
    completions: dict[tuple[str, float], int] = {}
    for key in sorted(conditions):
        cells = conditions[key]
        done = sum(1 for c in cells if c["state"] == "completed")
        void = sum(1 for c in cells if c["state"] == "voided")
        gone = sum(1 for c in cells if c["state"] == "diverged")
        completions[key] = done
        w(f"| `{key[0]}` | {key[1]} | **{done}/{len(cells)}** | {void} | {gone} |")
    w()

    # --- R-1 -----------------------------------------------------------------
    best = max(completions.items(), key=lambda kv: kv[1])
    r1 = best[1] >= R1_COMPLETIONS
    w("## Registered verdicts\n")
    w(f"**R-1** *(primary)* some condition completes well enough to be "
      f"measurable: best is `{best[0][0]}` at scale {best[0][1]} with "
      f"**{best[1]}/{R1_OF}**; bar {R1_COMPLETIONS}/{R1_OF} -> "
      f"**{'SUPPORTED' if r1 else 'NOT SUPPORTED'}**")
    if r1:
        w(f"  - That condition is the operating point. The recurrent half of the "
          f"factorial is registered separately and run there.")
    else:
        w("  - No operating point at the anchor budget on these levers. The "
          "recurrent axis stays out of the paper as **unmeasured**, not as a "
          "negative result, and the prereg §5 lever order applies.")
    w()

    # --- R-2, R-3 ------------------------------------------------------------
    by_arm = {arm: sum(v for k, v in completions.items() if k[0] == arm)
              for arm in sorted({k[0] for k in completions})}
    by_scale = {scale: sum(v for k, v in completions.items() if k[1] == scale)
                for scale in sorted({k[1] for k in completions})}

    arms = sorted(by_arm)
    delta_arm = by_arm[arms[0]] - by_arm[arms[1]] if len(arms) == 2 else 0
    r2 = abs(delta_arm) >= R2_R3_DELTA
    w(f"**R-2** adaptation is what destabilises: `{arms[0]}` "
      f"{by_arm[arms[0]]}/24 vs `{arms[1]}` {by_arm[arms[1]]}/24, difference "
      f"**{delta_arm:+d}**; bar |Δ| ≥ {R2_R3_DELTA} -> "
      f"**{'SUPPORTED' if r2 else 'NOT SUPPORTED'}**")
    w()

    scales = sorted(by_scale)
    delta_scale = by_scale[scales[0]] - by_scale[scales[1]] if len(scales) == 2 else 0
    r3 = abs(delta_scale) >= R2_R3_DELTA
    w(f"**R-3** the surrogate scale is a stability lever at this width: "
      f"{scales[0]} {by_scale[scales[0]]}/24 vs {scales[1]} "
      f"{by_scale[scales[1]]}/24, difference **{delta_scale:+d}**; bar "
      f"|Δ| ≥ {R2_R3_DELTA} -> **{'SUPPORTED' if r3 else 'NOT SUPPORTED'}**")
    w()

    # --- R-4, diagnostic -----------------------------------------------------
    w("**R-4** *(diagnostic, no verdict)* how far from usable each condition is.\n")
    w("| arm | scale | peak ‖g‖ of completing cells | abort steps of diverged cells |")
    w("|---|---:|---|---|")
    for key in sorted(conditions):
        peaks = [peak_norm(c["cell"]) for c in conditions[key] if c["state"] == "completed"]
        peaks = [p for p in peaks if p is not None]
        steps = sorted(c["step"] for c in conditions[key] if c.get("step") is not None)
        peak_text = (f"{min(peaks):.2e} – {max(peaks):.2e}" if peaks else "—")
        step_text = ", ".join(str(s) for s in steps) if steps else "—"
        w(f"| `{key[0]}` | {key[1]} | {peak_text} | {step_text} |")
    w()

    # --- accuracies, explicitly not a measurement ----------------------------
    w("## Accuracies of completing cells — **not a measurement**\n")
    w("Reported with each condition's completion count beside it. An arm that "
      "diverges more often can look better, because only its luckier "
      "trajectories survive to be scored; that is wave 11's recorded lesson and "
      "the reason no comparison between conditions with different completion "
      "rates is a result here.\n")
    w("| arm | scale | n completed | mean | min | max |")
    w("|---|---:|---:|---:|---:|---:|")
    for key in sorted(conditions):
        accs = [c["cell"]["accuracy"] for c in conditions[key] if c["state"] == "completed"]
        if not accs:
            w(f"| `{key[0]}` | {key[1]} | 0 | — | — | — |")
            continue
        w(f"| `{key[0]}` | {key[1]} | {len(accs)} | {sum(accs)/len(accs):.4f} | "
          f"{min(accs):.4f} | {max(accs):.4f} |")
    w()

    warnings = []
    for key in sorted(conditions):
        for cell in (c["cell"] for c in conditions[key] if c["state"] == "completed"):
            warnings.extend(stability_warnings(cell))
    w(f"**Stability notes: {len(warnings)}.** These are registered as expected "
      "and non-voiding — a recurrent arm above the 1e9 tier is the phenomenon "
      "under study, not a defect.\n")

    voided = [(cid, o["why"]) for cid, o in sorted(outcomes.items()) if o["state"] == "voided"]
    if voided:
        w("## Voided cells\n")
        for cid, why in voided:
            w(f"- `{cid[:60]}`: {why}")

    text = "\n".join(lines)
    if args.out == "-":
        print(text)
    else:
        open(args.out, "w").write(text + "\n")
        print(f"wrote {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
