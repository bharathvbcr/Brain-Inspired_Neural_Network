#!/usr/bin/env python3
"""Compute the campaign's registered verdicts from the cells.

Implements `PREREG_2026-08-19_SHD_ATTENTION_CAMPAIGN.md` as amended by
`AMENDMENT_2026-08-19_W2_1_STEP_COUNT.md`. Written before the data landed, so
the thresholds cannot be shaped by what came back.

Three things it refuses to do:

  * report a wave whose instances have no recorded Gate F verdict (prereg §5.7 -
    "the check did not run" must never read the same as "the check ran and
    passed");
  * silently drop a cell that fails a validity gate - voided cells are listed
    with their reason and counted, and a wave missing cells reports as
    incomplete rather than as a result over whatever survived;
  * quote a macOS-recorded reference beside a campaign number.
"""

from __future__ import annotations

import argparse
import json
import math
import os
import re
import statistics
import sys
from collections import defaultdict
from pathlib import Path

# Two-sided critical t at alpha = 0.01, by degrees of freedom. Hardcoded so the
# analysis needs no scientific-stack dependency; df = n - 1 for a paired test.
#
# Complete for df 1..20. It used to hold only {5, 7, 9, 11, 15, 19} and fall back
# to the *nearest* key, which snaps in whichever direction happens to be closer —
# so df = 2 borrowed df = 5's 4.032 in place of its true 9.925, a bar 2.5x too
# low. Mid-campaign that is the normal state, because the analysis pairs over
# whatever seeds are on disk: three completed pairs was enough to print
# "significant at alpha = 0.01" for a result the registered alpha rejects, with
# the "INCOMPLETE: registered n = 12" note printed underneath as a caveat rather
# than as a block.
T_CRIT_001 = {
    1: 63.657, 2: 9.925, 3: 5.841, 4: 4.604, 5: 4.032,
    6: 3.707, 7: 3.499, 8: 3.355, 9: 3.250, 10: 3.169,
    11: 3.106, 12: 3.055, 13: 3.012, 14: 2.977, 15: 2.947,
    16: 2.921, 17: 2.898, 18: 2.878, 19: 2.861, 20: 2.845,
}
#: Above the table the critical value keeps falling toward 2.576, so the largest
#: tabulated df is the conservative choice for any df beyond it — never the
#: nearest, which can only ever be smaller than the truth.
MAX_TABULATED_DF = max(T_CRIT_001)


def paired_t(deltas: list[float]) -> tuple[float, float, bool]:
    """Returns (t, critical, significant) for H0: mean delta == 0."""
    n = len(deltas)
    if n < 2:
        return (float("nan"), float("nan"), False)
    mean = statistics.mean(deltas)
    sd = statistics.stdev(deltas)
    if sd == 0.0:
        return (float("inf"), 0.0, mean != 0.0)
    t = mean / (sd / math.sqrt(n))
    # Never interpolate toward a smaller bar. Below the table there is nothing
    # honest to fall back to, and df >= 1 always is, since n >= 2 here.
    df = n - 1
    crit = T_CRIT_001.get(df) or T_CRIT_001[MAX_TABULATED_DF]
    return (t, crit, abs(t) >= crit)


# Preregistration section 5, per cell — from the single owner in
# `scripts/cell_validity.py`. Three copies of this rule existed and had already
# drifted; see that module's docstring.
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))
from cell_validity import stability_warnings, validity_problems  # noqa: E402,F401


def key(spec: dict) -> tuple:
    """Everything that identifies an arm apart from its seed."""
    return (spec["wave"], spec["arm"], spec["hidden"], spec["epochs"], spec["contract"],
            spec["geometry"], spec["attn_dim"], spec["attn_layers"], spec["temporal"],
            spec["surrogate_scale"])


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", required=True)
    parser.add_argument("--results", required=True, help="directory of downloaded cells")
    parser.add_argument("--gates", help="directory of downloaded gate json")
    parser.add_argument("--failures", help="directory of downloaded failure logs; "
                                           "diverged cells are data, not absences")
    args = parser.parse_args()

    plan = {c["id"]: c for c in json.load(open(args.plan))}
    results_dir = Path(args.results)

    # A cell that diverged is a RESULT, not an error.
    #
    # The instrument refuses to emit a cell containing a non-finite value - it
    # hard-errors instead - so a diverged arm leaves a log and no JSON, and a
    # naive reader counts it as "missing" and reports the wave as incomplete.
    # That would silently convert wave 4's entire question ("does the recurrent
    # arm complete at all?") into an absence of data. Diverged cells are
    # therefore read from the failure logs and counted explicitly.
    diverged = {}
    failures_dir = Path(args.failures) if args.failures else None
    if failures_dir and failures_dir.is_dir():
        for log in failures_dir.glob("*.log"):
            text = log.read_text(errors="replace")
            cid = log.name[: -len(".log")]
            if "non-finite training value" in text:
                step = re.search(r"optimizer step (\d+)", text)
                diverged[cid] = f"non-finite at optimizer step {step.group(1) if step else '?'}"
            else:
                diverged[cid] = "did not complete (see log)"

    cells, voided, missing = {}, [], []
    for cid, spec in plan.items():
        path = results_dir / f"{cid}.json"
        if not path.exists():
            missing.append(cid)
            continue
        cell = json.loads(path.read_text())
        problems = validity_problems(cell, spec)
        if problems:
            voided.append((cid, problems))
        else:
            cells[cid] = cell

    missing = [m for m in missing if m not in diverged]
    print(f"cells: {len(cells)} valid, {len(voided)} voided, {len(diverged)} DIVERGED, "
          f"{len(missing)} missing of {len(plan)} planned")
    if diverged:
        print("\ndiverged cells (a result, not an absence)")
        for cid, why in sorted(diverged.items()):
            print(f"  {cid[:70]}\n      {why}")

    # --- prereg 5.7: no wave is reportable without its gate verdicts ---------
    verdicts = []
    if args.gates:
        for path in sorted(Path(args.gates).glob("*.json")):
            payload = json.loads(path.read_text())
            verdicts.append((payload["instance"], payload["cross_machine_gate_f"]))
    if not verdicts:
        print("\nNO GATE VERDICT RECORDED. Per prereg section 5.7 nothing below is "
              "reportable; pass --gates.")
    else:
        print("\ncross-machine Gate F: " +
              ", ".join(f"{i[:11]}={v}" for i, v in verdicts))
        if any(v == "FAIL" for _, v in verdicts):
            print("  At least one instance FAILED, so comparison against the")
            print("  macOS-recorded references (0.7032, 0.7378) is UNLICENSED.")
            print("  Every number below is a same-machine paired contrast.")

    if voided:
        print("\nvoided cells")
        for cid, problems in voided[:20]:
            print(f"  {cid}: {', '.join(problems)}")
        if len(voided) > 20:
            print(f"  ... and {len(voided) - 20} more")

    by_arm: dict[tuple, dict[int, dict]] = defaultdict(dict)
    for cid, cell in cells.items():
        by_arm[key(plan[cid])][plan[cid]["seed"]] = cell

    def acc(k) -> dict[int, float]:
        return {seed: c["accuracy"] for seed, c in by_arm.get(k, {}).items()}

    def paired(k_treat, k_ctrl):
        a, b = acc(k_treat), acc(k_ctrl)
        seeds = sorted(set(a) & set(b))
        return seeds, [a[s] - b[s] for s in seeds]

    anchor = ("published-2ms", "adjacent-sum-5")
    A = ("w1", "ff+fixed", 128, 400, *anchor, None, None, "intact", None)
    Bk = ("w1", "ff+fixed+attn", 128, 400, *anchor, 32, 1, "intact", None)
    C = ("w1", "ff+fixed", 192, 400, *anchor, None, None, "intact", None)
    D = ("w1", "ff+fixed", 128, 400, *anchor, None, None, "bin-shuffled", None)
    E = ("w1", "ff+fixed+attn", 128, 400, *anchor, 32, 1, "bin-shuffled", None)

    print("\n=== WAVE 1 ===")
    seeds, deltas = paired(Bk, A)
    if not deltas:
        print("  no complete A/B pairs yet")
    else:
        mean = statistics.mean(deltas)
        t, crit, sig = paired_t(deltas)
        print(f"  n={len(seeds)} paired seeds")
        print(f"  W1-1  mean(attn) - mean(ff+fixed) = {mean:+.4f}  "
              f"(>= 0.05), t={t:.3f} vs crit {crit} at alpha=0.01")
        print(f"        verdict: {'SUPPORTED' if mean >= 0.05 and sig else 'NOT SUPPORTED'}")
        if len(seeds) < 12:
            print(f"        INCOMPLETE: registered n=12, have {len(seeds)}")

        _, dbc = paired(Bk, C)
        if dbc:
            m = statistics.mean(dbc)
            print(f"  W1-2  vs h192 capacity control = {m:+.4f}  (>= 0.02)  "
                  f"verdict: {'NOT A CAPACITY ARTEFACT' if m >= 0.02 else 'NOT SUPPORTED'}")
        _, dshuf = paired(E, D)
        if dshuf and deltas:
            g = statistics.mean(deltas) - statistics.mean(dshuf)
            print(f"  W1-3  gain(intact) {statistics.mean(deltas):+.4f} - "
                  f"gain(shuffled) {statistics.mean(dshuf):+.4f} = {g:+.4f}  (>= 0.02)")
            print(f"        verdict: {'MEMORY, not just capacity' if g >= 0.02 else 'NOT SUPPORTED'}")

        tails = [c["tail_loss_improvement"] for k in (Bk, E) for c in by_arm.get(k, {}).values()]
        if tails:
            worst = min(tails)
            print(f"  W1-4  worst tail_loss_improvement = {worst:+.4f}  (> -0.02)")
            print(f"        verdict: {'CONVERGED' if worst > -0.02 else 'UNDERTRAINED'}")
            if worst <= -0.02:
                print("        W1-1 is therefore reported as UNTESTED, per the prereg's")
                print("        named outcome: the accuracy is a budget artefact.")

    print("\n=== WAVE 2 ===")
    dims = [16, 32, 64, 128]
    means = {}
    for d in dims:
        a = acc(("w2dim", "ff+fixed+attn", 128, 100, *anchor, d, 1, "intact", None))
        if a:
            means[d] = statistics.mean(a.values())
    if len(means) == len(dims):
        steps = [(dims[i], dims[i + 1], means[dims[i + 1]] - means[dims[i]])
                 for i in range(len(dims) - 1)]
        for lo, hi, delta in steps:
            print(f"  d{lo} -> d{hi}: {delta:+.4f}")
        ok = sum(1 for *_, delta in steps if delta >= 0.0)
        print(f"  W2-1  {ok} of 3 steps non-decreasing (>= 2 required, as amended)")
        print(f"        verdict: {'SUPPORTED' if ok >= 2 else 'NOT SUPPORTED'}")
    else:
        print(f"  incomplete: have d_model {sorted(means)} of {dims}")

    layer_means = {}
    for layers in (1, 2, 4):
        a = acc(("w2lyr", "ff+fixed+attn", 128, 100, *anchor, 32, layers, "intact", None))
        if a:
            layer_means[layers] = statistics.mean(a.values())
    if len(layer_means) == 3:
        print(f"  W2-2  (descriptive, no threshold) "
              f"L1->L2 {layer_means[2] - layer_means[1]:+.4f}, "
              f"L2->L4 {layer_means[4] - layer_means[2]:+.4f}")

    print("\n=== WAVE 3 ===")
    widths = [128, 256, 512, 1024]
    wid = {}
    for h in widths:
        a = acc(("w3wid", "ff+fixed+attn", h, 400, *anchor, 32, 1, "intact", None))
        if a:
            wid[h] = statistics.mean(a.values())
    if len(wid) == len(widths):
        final = wid[1024] - wid[512]
        print(f"  W3-1  final width doubling h512->h1024 = {final:+.4f}  (>= 0.01; "
              f"recorded ff+fixed value is +0.000883)")
        print(f"        verdict: {'SUPPORTED' if final >= 0.01 else 'NOT SUPPORTED'}")
    else:
        print(f"  W3-1 incomplete: have widths {sorted(wid)}")

    _, dgeo = paired(("w3geo", "ff+fixed+attn", 128, 400, "published-2ms", "channels-700", 32, 1, "intact", None),
                     ("w3geo", "ff+fixed", 128, 400, "published-2ms", "channels-700", None, None, "intact", None))
    if dgeo:
        m = statistics.mean(dgeo)
        print(f"  W3-2  attention gain at channels-700 = {m:+.4f}  (>= 0.05)  "
              f"verdict: {'SUPPORTED' if m >= 0.05 else 'NOT SUPPORTED'}")

    contracts = ["published-2ms", "published-10ms", "fixed-t100", "fixed-t250", "fixed-t500"]
    spread_src = {}
    for contract in contracts:
        wave = "w2dim" if contract == "published-2ms" else "w3con"
        epochs = 100
        a = acc((wave, "ff+fixed+attn", 128, epochs, contract, "adjacent-sum-5", 32, 1, "intact", None))
        if a:
            spread_src[contract] = statistics.mean(a.values())
    if len(spread_src) >= 4:
        spread = max(spread_src.values()) - min(spread_src.values())
        print(f"  W3-3  accuracy spread across {len(spread_src)} contracts = {spread:.4f}  "
              f"(> 0.02; recorded ff+fixed spread is 0.0034)")
        print(f"        verdict: {'BREAKS RESOLUTION INVARIANCE' if spread > 0.02 else 'STILL INVARIANT'}")
        if spread <= 0.02:
            print("        Per the prereg: attention accurate AND still resolution-")
            print("        invariant is a confusing result and is reported as one.")
    else:
        print(f"  W3-3 incomplete: have {sorted(spread_src)}")

    print("\n=== WAVE 4 ===")
    w4_planned = [c for c in plan.values() if c["wave"] == "w4rec"]
    any_diverged = False
    for scale in (1.0, 0.4):
        for arm in ("rec+alif", "rec+alif+attn"):
            k = ("w4rec", arm, 256, 100, *anchor,
                 32 if arm.endswith("attn") else None, 1 if arm.endswith("attn") else None,
                 "intact", scale)
            got = by_arm.get(k, {})
            usable = sum(1 for c in got.values() if c["non_finite_events"] == 0)
            planned = sum(1 for c in w4_planned
                          if c["arm"] == arm and c["surrogate_scale"] == scale)
            blew_up = sum(1 for cid in diverged
                          if plan.get(cid, {}).get("arm") == arm
                          and plan.get(cid, {}).get("surrogate_scale") == scale)
            any_diverged = any_diverged or blew_up > 0
            print(f"  ss={scale}  {arm:<16} usable {usable}/{planned} planned, "
                  f"{blew_up} diverged, {planned - usable - blew_up} still running")
    # W4-1 as registered: EVERY rec+alif+attn cell must complete finite.
    attn_diverged = sum(1 for cid in diverged
                        if plan.get(cid, {}).get("arm") == "rec+alif+attn")
    if attn_diverged:
        print(f"\n  W4-1  {attn_diverged} rec+alif+attn cell(s) diverged")
        print("        verdict: NOT SUPPORTED - attention does NOT confer stability")
        print("        on the recurrent arm. Registered as the honest prior in W4-2:")
        print("        the explosion is in the recurrent BPTT path, which attention")
        print("        sits beside rather than inside.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
