#!/usr/bin/env python3
"""Wave 10 — the temporal-resolution ladder. Registered verdicts, computed once.

Written 2026-08-22 **before the first cell landed**, so the thresholds cannot be
tuned to the data. Reads `PREREG_2026-08-22_SHD_ATTENTION_RESOLUTION_LADDER.md`
§3 and does exactly what it says.

Reuses `analyse_wave8`'s loaders so the pinned-binary check, the reused-cell hash
verification and the validity gates have one owner rather than two.
"""

from __future__ import annotations

import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from analyse_wave8 import (  # noqa: E402
    SEEDS,
    accs,
    assert_one_pinned_binary,
    load,
    mean,
    verdict,
)

V2 = "results/shd_attention_campaign_v2"
GEO = "adjacent-sum-5"
RUNGS = ("fixed-t100", "fixed-t250", "fixed-t500")
#: Bin width in ms for each rung: one fixed 1400 ms window / N frames.
DT_MS = {"fixed-t100": 14.0, "fixed-t250": 5.6, "fixed-t500": 2.8}

# Preregistered bars (§3). Named here so a reader can diff them against the doc.
C1_GAIN = 0.05
C1_SEEDS = 10
C2_SPREAD = 0.03
C3_CONFOUND = 0.05
C4_GATE = 0.80
C4_SEEDS = 9


def stem(arm: str, contract: str) -> str:
    tag = "ff-fixed-attn" if arm == "attn" else "ff-fixed"
    suffix = "__d32l4" if arm == "attn" else ""
    return f"w10con__{tag}__h128__e400__{contract}__{GEO}{suffix}"


def main() -> int:
    pinned = assert_one_pinned_binary()
    voided: list[str] = []
    out: list[str] = []
    a = out.append

    a("# Wave 10 — registered verdicts\n")
    a(f"Prereg: `PREREG_2026-08-22_SHD_ATTENTION_RESOLUTION_LADDER.md` §3. "
      f"All 72 cells from one pinned binary `{pinned[:12]}`.\n")

    gains: dict[str, float] = {}
    base: dict[str, float] = {}
    treat: dict[str, float] = {}
    pos: dict[str, int] = {}
    over_gate: dict[str, int] = {}

    a("## Measurements\n")
    a("| contract | bin ms | `ff+fixed` | d32/L4 | gain | gain > 0 | ≥ 0.80 |")
    a("|---|---:|---:|---:|---:|---:|---:|")
    for c in RUNGS:
        t = accs(load(V2, stem("attn", c)), f"attn {c}", voided)
        k = accs(load(V2, stem("rate", c)), f"rate {c}", voided)
        d = [x - y for x, y in zip(t, k)]
        gains[c], base[c], treat[c] = mean(d), mean(k), mean(t)
        pos[c] = sum(1 for v in d if v > 0)
        over_gate[c] = sum(1 for v in t if v >= C4_GATE)
        a(f"| `{c}` | {DT_MS[c]} | {mean(k):.4f} | {mean(t):.4f} | "
          f"**{mean(d):+.4f}** | {pos[c]}/12 | {over_gate[c]}/12 |")

    a("\n**Validity gates: " + ("all 72 cells pass.**" if not voided
                               else f"{len(voided)} PROBLEM(S).**"))
    for v in voided:
        a(f"- {v}")
    if voided:
        a("\n**VOIDED cells present — verdicts below are not reportable "
          "until they are explained.**")

    a("\n## Registered verdicts\n")
    c1 = all(gains[c] >= C1_GAIN and pos[c] >= C1_SEEDS for c in RUNGS)
    detail = "; ".join(f"{c} {gains[c]:+.4f}/{pos[c]}of12" for c in RUNGS)
    a(f"**C-1** the read-out helps at every resolution: {detail}; "
      f"bar ≥ +{C1_GAIN} and ≥ {C1_SEEDS}/12 each -> **{verdict(c1)}**")
    if not c1:
        held = [c for c in RUNGS if gains[c] >= C1_GAIN and pos[c] >= C1_SEEDS]
        a(f"  - holds at: {', '.join(held) if held else '**none**'}; "
          f"the effect is scoped to those resolutions as a measurement.")

    spread = gains["fixed-t500"] - gains["fixed-t100"]
    c2 = abs(spread) >= C2_SPREAD
    direction = ("rises with t (finer resolution)" if spread > 0
                 else "falls with t (finer resolution)" if spread < 0 else "flat")
    a(f"\n**C-2** *(two-sided)* gain depends on resolution: "
      f"gain(t500) − gain(t100) = **{spread:+.4f}**, |·| bar {C2_SPREAD} -> "
      f"**{verdict(c2)}**")
    a(f"  - direction: **{direction}**")
    if not c2:
        a("  - **Flat across a 5× change in temporal resolution.** Resolution is "
          "not the axis; M-1's shuffle result stands alone as the mechanism and "
          "no resolution story is told.")

    drift = base["fixed-t500"] - base["fixed-t100"]
    c3 = abs(drift) > C3_CONFOUND
    a(f"\n**C-3** baseline drift across the ladder: "
      f"`ff+fixed` t500 − t100 = **{drift:+.4f}** (confound bar {C3_CONFOUND}) -> "
      + ("**C-2 IS CONFOUNDED** — the substrate moves with resolution, so C-2 is "
         "reported as uninterpretable." if c3 else
         "**not confounded** — the baseline is stable, so C-2 is about the read-out."))

    a("\n**C-4** rungs clearing the registered gate:")
    for c in RUNGS:
        ok = treat[c] >= C4_GATE and over_gate[c] >= C4_SEEDS
        a(f"  - `{c}`: mean {treat[c]:.4f}, {over_gate[c]}/12 seeds ≥ {C4_GATE} "
          f"-> **{verdict(ok)}**")

    nonfin = diverged = 0
    for c in RUNGS:
        for arm in ("attn", "rate"):
            for cell in load(V2, stem(arm, c)):
                nonfin += cell["non_finite_events"]
                diverged += cell.get("mechanical_status") != "COMPLETE"
    c5 = nonfin == 0 and diverged == 0
    a(f"\n**C-5** stability: {nonfin} non-finite events, {diverged} incomplete "
      f"cells across 72 -> **{verdict(c5)}**")

    # ---- prereg section 5: the registered falsification check ---------------
    a("\n## Cross-cloud check (prereg §5)\n")
    az = f"/tmp/azres/results/az8con__ff-fixed-attn__h128__e400__fixed-t250__{GEO}__d32l4"
    if os.path.exists(f"{az}__s{SEEDS[0]}.json"):
        aws_cells = load(V2, stem("attn", "fixed-t250"))
        az_cells = [json.load(open(f"{az}__s{s}.json")) for s in SEEDS]
        vals = diff = 0
        for x, y in zip(aws_cells, az_cells):
            for key in set(x) & set(y):
                if key == "wall_secs":
                    continue
                vx, vy = x[key], y[key]
                if isinstance(vx, float):
                    vals += 1
                    diff += vx != vy
                elif isinstance(vx, list) and vx and isinstance(vx[0], float):
                    vals += len(vx)
                    diff += sum(1 for p, q in zip(vx, vy) if p != q)
        a(f"aarch64 (this wave) vs x86-64 (Azure `az8con` fixed-t250): "
          f"**{vals} float values, {diff} differing**.")
        if diff:
            a("\n> **FINDING_2026-08-22_REPRODUCIBLE_ACROSS_ISA_UNDER_GLIBC.md IS "
              "WRONG OR INCOMPLETE.** Per prereg §5 it must be amended before any "
              "verdict above is reported.")
        else:
            a("\nThe registered expectation held; the reproducibility finding "
              "survives a test it could have failed.")
    else:
        a("Azure `fixed-t250` cells not present locally — check not run. "
          "**Recorded as not run, not as passed.**")

    print("\n".join(out))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
