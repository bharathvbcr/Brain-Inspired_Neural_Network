#!/usr/bin/env python3
"""Wave 14 verdicts, exactly as registered in
`results/PREREG_2026-08-23_RECURRENT_MEASUREMENT.md`.

Thresholds registered before any cell existed; this file committed after the
control arm had landed. **Not** "frozen before the first cell" — see the
correction in `RESULT_2026-08-23_W14_ATTENTION_AND_RECURRENCE_ARE_COMPLEMENTARY.md`
§8. The registered bars are in `PREREG_2026-08-23_RECURRENT_MEASUREMENT.md`,
committed at 08:23 UTC against a first cell at 08:30, and
`test_the_registered_bars_are_the_ones_in_the_prereg` pins that this file
carries those and no others.

Two things make this analyser different from waves 8–12, and both are places a
silent bug would change a verdict rather than crash:

  * **Completion gates the measurement.** M-0 requires each arm at ≥ 11/12 and
    each comparison at ≥ 10 surviving seed-pairs. A gate that computes the mean
    anyway and prints it beside a NOT EVALUABLE banner is the shape this
    repository keeps finding, so a blocked comparison here carries **no numbers
    at all**.
  * **Every comparison is paired on seed**, over pairs where *both* arms
    completed. Pooling instead would compare two differently filtered subsets
    and reintroduce exactly the survivorship wave 11 recorded.

    python3 scripts/aws/analyse_wave14.py --plan results/shd_attention_campaign_v2/plan_w14.json \\
        --results results/shd_attention_campaign_v2 [--failures DIR] [--out FILE]
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
from cell_validity import stability_warnings, validity_problems  # noqa: E402

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from analyse_wave13 import ABORT_STEP  # noqa: E402

SEEDS = [5170001 + i for i in range(12)]

#: Registered thresholds. Named so a reader can check them against the prereg
#: without reading the code that applies them.
M0_COMPLETIONS = 11
M0_OF = 12
M0_PAIRS = 10
M1_GAIN = 0.05
M1_PAIRS_POSITIVE = 10
M2_DELTA = 0.03
M3_GATE = 0.80
M3_SEEDS = 9
#: The archived `ff+fixed` anchor mean, at surrogate scale 1.0. M-4 only.
ARCHIVED_FF_FIXED_AT_SCALE_1 = 0.7062

#: `rec+alif` at this operating point is wave 13's, not regenerated. The
#: instrument is deterministic, so re-running the identical spec would produce
#: byte-identical cells.
REUSED_ARM = "rec+alif"
REUSED_STEM = "w13rec__rec-alif__h128__e400__published-2ms__adjacent-sum-5__ss0.4"

#: What a reused cell must say about itself. There is no manifest hash for wave
#: 13's cells, so this is a content check, not a provenance one: it catches
#: reusing the wrong cell, not a cell edited in place.
REUSED_EXPECTED = {
    "arm": "rec+alif",
    "hidden": 128,
    "epochs": 400,
    "contract": "published-2ms",
    "geometry": "adjacent-sum-5",
}


def outcome_for(path: Path, spec: dict | None, failures: Path | None, stem: str) -> dict:
    """completed / voided / diverged for one cell."""
    if not path.is_file():
        step, why = None, "no cell emitted"
        if failures is not None:
            log = failures / f"{stem}.log"
            if log.is_file():
                match = ABORT_STEP.search(log.read_text(errors="replace"))
                if match:
                    step = int(match.group(1))
                    why = f"diverged at optimizer step {step}"
        return {"state": "diverged", "why": why, "step": step}
    cell = json.loads(path.read_text())
    problems = validity_problems(cell, spec)
    if problems:
        return {"state": "voided", "why": "; ".join(problems), "cell": cell, "step": None}
    return {"state": "completed", "why": "", "cell": cell, "step": None}


def collect(plan: list[dict], results: Path, failures: Path | None) -> dict:
    """`{(arm, seed): outcome}` over the generated arms plus the reused one."""
    out: dict[tuple[str, int], dict] = {}
    for spec in plan:
        out[(spec["arm"], spec["seed"])] = outcome_for(
            results / f"{spec['id']}.json", spec, failures, spec["id"]
        )

    for seed in SEEDS:
        stem = f"{REUSED_STEM}__s{seed}"
        # No plan entry: wave 13's spec is the authority for these, and passing
        # a wave-14 spec would make the gate compare the wrong `temporal`.
        outcome = outcome_for(results / f"{stem}.json", None, failures, stem)
        if outcome["state"] == "completed":
            cell = outcome["cell"]
            wrong = {
                key: cell.get(key)
                for key, want in REUSED_EXPECTED.items()
                if cell.get(key) != want
            }
            if wrong:
                outcome = {
                    "state": "voided",
                    "why": f"reused cell does not match its expected spec: {wrong}",
                    "cell": cell,
                    "step": None,
                }
        out[(REUSED_ARM, seed)] = outcome
    return out


def completions(outcomes: dict, arm: str) -> int:
    return sum(1 for (a, _), o in outcomes.items() if a == arm and o["state"] == "completed")


def pairs(outcomes: dict, treatment: str, control: str) -> list[tuple[int, float, float]]:
    """`(seed, treatment accuracy, control accuracy)` where BOTH completed."""
    out = []
    for seed in SEEDS:
        t, c = outcomes.get((treatment, seed)), outcomes.get((control, seed))
        if t and c and t["state"] == "completed" and c["state"] == "completed":
            out.append((seed, t["cell"]["accuracy"], c["cell"]["accuracy"]))
    return out


def evaluable(outcomes: dict, treatment: str, control: str) -> tuple[bool, str]:
    """M-0, applied to one comparison. Returns (ok, why not)."""
    reasons = []
    for arm in (treatment, control):
        done = completions(outcomes, arm)
        if done < M0_COMPLETIONS:
            reasons.append(f"`{arm}` completed {done}/{M0_OF} (bar {M0_COMPLETIONS})")
    surviving = len(pairs(outcomes, treatment, control))
    if surviving < M0_PAIRS:
        reasons.append(f"{surviving} surviving seed-pairs (bar {M0_PAIRS})")
    return (not reasons), "; ".join(reasons)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", required=True)
    parser.add_argument("--results", required=True)
    parser.add_argument("--failures")
    parser.add_argument("--out", default="-")
    args = parser.parse_args()

    plan = json.load(open(args.plan))
    outcomes = collect(
        plan, Path(args.results), Path(args.failures) if args.failures else None
    )
    arms = ["ff+fixed", "ff+fixed+attn", "rec+alif", "rec+alif+attn"]

    lines: list[str] = []

    def w(text=""):
        lines.append(text)

    w("# Wave 14 — attention on a recurrent substrate, at the operating point\n")
    w("Prereg: `PREREG_2026-08-23_RECURRENT_MEASUREMENT.md` §4. h128 / "
      "`published-2ms` / `adjacent-sum-5` / e400, **surrogate scale 0.4 on every "
      "arm**, same pinned binary. 36 new cells; `rec+alif` reused from wave 13.\n")

    w("## Completion\n")
    w("| arm | completed | voided | diverged |")
    w("|---|---:|---:|---:|")
    for arm in arms:
        cells = [o for (a, _), o in outcomes.items() if a == arm]
        done = sum(1 for c in cells if c["state"] == "completed")
        void = sum(1 for c in cells if c["state"] == "voided")
        gone = sum(1 for c in cells if c["state"] == "diverged")
        w(f"| `{arm}` | **{done}/{len(cells)}** | {void} | {gone} |")
    w()

    def gain_block(treatment: str, control: str):
        """Paired gain over surviving pairs, or None if M-0 blocks it."""
        ok, why = evaluable(outcomes, treatment, control)
        if not ok:
            return None, why
        matched = pairs(outcomes, treatment, control)
        deltas = [t - c for _, t, c in matched]
        return {
            "n": len(matched),
            "gain": sum(deltas) / len(deltas),
            "positive": sum(1 for d in deltas if d > 0),
            "treatment_mean": sum(t for _, t, _ in matched) / len(matched),
            "control_mean": sum(c for _, _, c in matched) / len(matched),
            "min": min(deltas),
            "max": max(deltas),
        }, ""

    rec, rec_why = gain_block("rec+alif+attn", "rec+alif")
    ff, ff_why = gain_block("ff+fixed+attn", "ff+fixed")

    w("## Paired gains, over seeds where both arms completed\n")
    w("| substrate | pairs | rate read-out | + attention d32/L4 | gain | per-pair range |")
    w("|---|---:|---:|---:|---:|---|")
    for label, block, why in (("`rec+alif`", rec, rec_why), ("`ff+fixed`", ff, ff_why)):
        if block is None:
            w(f"| {label} | — | — | — | **NOT EVALUABLE** | {why} |")
        else:
            w(f"| {label} | {block['n']} | {block['control_mean']:.4f} | "
              f"{block['treatment_mean']:.4f} | **{block['gain']:+.4f}** | "
              f"{block['min']:+.4f} to {block['max']:+.4f} |")
    w()

    w("## Registered verdicts\n")

    # --- M-1 -----------------------------------------------------------------
    if rec is None:
        w(f"**M-1** *(primary)* **NOT EVALUABLE** — {rec_why}. No mean is "
          "reported for this comparison; M-0 blocks it.")
    else:
        m1 = rec["gain"] >= M1_GAIN and rec["positive"] >= M1_PAIRS_POSITIVE
        w(f"**M-1** *(primary)* attention helps a recurrent, adaptive substrate: "
          f"gain **{rec['gain']:+.4f}** (bar +{M1_GAIN}), positive in "
          f"**{rec['positive']}/{rec['n']}** pairs (bar {M1_PAIRS_POSITIVE}) -> "
          f"**{'SUPPORTED' if m1 else 'NOT SUPPORTED'}**")
    w()

    # --- M-2 -----------------------------------------------------------------
    if rec is None or ff is None:
        blocked = rec_why or ff_why
        w(f"**M-2** *(primary, two-sided)* **NOT EVALUABLE** — {blocked}.")
    else:
        delta = rec["gain"] - ff["gain"]
        m2 = abs(delta) >= M2_DELTA
        w(f"**M-2** *(primary, two-sided)* the gain depends on whether the "
          f"substrate is recurrent: gain(`rec+alif`) **{rec['gain']:+.4f}** vs "
          f"gain(`ff+fixed`) **{ff['gain']:+.4f}**, difference **{delta:+.4f}**; "
          f"bar |Δ| ≥ {M2_DELTA} -> "
          f"**{'SUPPORTED' if m2 else 'NOT SUPPORTED'}**")
        if not m2:
            w("  - Flat. Substitution is refuted on the recurrence axis as wave "
              "12 refuted it on adaptation: the read-out's advantage is "
              "indifferent to what the spiking layer carries.")
        elif delta < 0:
            w("  - Smaller on the recurrent substrate: attention was partly "
              "standing in for recurrence.")
        else:
            w("  - Larger on the recurrent substrate: attention and recurrence "
              "are complementary, and that needs its own explanation rather "
              "than an assumption.")
    w()

    # --- M-3 -----------------------------------------------------------------
    rec_plain = [
        o["cell"]["accuracy"]
        for (a, _), o in outcomes.items()
        if a == "rec+alif" and o["state"] == "completed"
    ]
    if completions(outcomes, "rec+alif") < M0_COMPLETIONS:
        w(f"**M-3** **NOT EVALUABLE** — `rec+alif` completed "
          f"{completions(outcomes, 'rec+alif')}/{M0_OF}.")
    else:
        mean_rec = sum(rec_plain) / len(rec_plain)
        over = sum(1 for a in rec_plain if a >= M3_GATE)
        m3 = mean_rec >= M3_GATE and over >= M3_SEEDS
        control_mean = ff["control_mean"] if ff else float("nan")
        w(f"**M-3** recurrence plus adaptation alone reaches the gate: "
          f"`rec+alif` mean **{mean_rec:.4f}** (bar {M3_GATE}), **{over}/"
          f"{len(rec_plain)}** completing seeds ≥ {M3_GATE} (bar {M3_SEEDS}); "
          f"`ff+fixed` at the same scale is {control_mean:.4f} -> "
          f"**{'SUPPORTED' if m3 else 'NOT SUPPORTED'}**")
    w()

    # --- M-4 -----------------------------------------------------------------
    if ff is None:
        w("**M-4** *(descriptive)* not reportable — the `ff+fixed` comparison is "
          "blocked.")
    else:
        drop = ff["control_mean"] - ARCHIVED_FF_FIXED_AT_SCALE_1
        w(f"**M-4** *(descriptive, no verdict)* the scale is not quietly "
          f"crippling the baseline: `ff+fixed` at 0.4 is "
          f"**{ff['control_mean']:.4f}** against the archived "
          f"{ARCHIVED_FF_FIXED_AT_SCALE_1:.4f} at 1.0, a difference of "
          f"**{drop:+.4f}**.")
        if drop < -0.05:
            w("  - **Materially below.** M-2 above compares a healthy recurrent "
              "arm against a weakened feed-forward one and is reported as "
              "scale-limited, not clean.")
    w()

    warnings = []
    for (arm, seed), o in sorted(outcomes.items()):
        if o["state"] == "completed":
            warnings.extend(f"`{arm}` s{seed}: {t}" for t in stability_warnings(o["cell"]))
    w(f"**Stability notes: {len(warnings)}**, registered as non-voiding.")
    for line in warnings[:8]:
        w(f"- {line}")
    if len(warnings) > 8:
        w(f"- … and {len(warnings) - 8} more")
    w()

    bad = [(a, s, o["why"]) for (a, s), o in sorted(outcomes.items())
           if o["state"] in ("voided", "diverged")]
    if bad:
        w("## Cells that did not complete\n")
        for arm, seed, why in bad:
            w(f"- `{arm}` s{seed}: {why}")

    w("\n## Scope\n")
    w("- One scale (0.4), one width, one contract, one budget. The anchor runs "
      "at scale 1.0, so this is **not** the anchor.")
    w("- Nothing about `rec+fixed`: wave 13 measured it and it does not complete.")
    w("- Not calibration. No comparison to macOS-recorded numbers.")

    text = "\n".join(lines)
    if args.out == "-":
        print(text)
    else:
        open(args.out, "w").write(text + "\n")
        print(f"wrote {args.out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
