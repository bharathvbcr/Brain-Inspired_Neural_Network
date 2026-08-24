#!/usr/bin/env python3
"""Wave 8 verdicts, exactly as registered in
`results/PREREG_2026-08-20_SHD_ATTENTION_HEADLINE_SCOPE.md`.

Refuses to report a hypothesis whose cells are not all present and all valid,
because a threshold applied to a partial arm is not the registered threshold.
"""

from __future__ import annotations

import argparse
import glob
import hashlib
import json
import os
import sys

V2 = "results/shd_attention_campaign_v2"
V1 = "results/shd_attention_campaign_v1/cells"

SEEDS = [5170001 + i for i in range(12)]


V1_MANIFEST = "results/shd_attention_campaign_v1/manifest.json"
V2_MANIFEST = "results/shd_attention_campaign_v2/manifest.json"

#: Populated on first use; maps v1 cell filename -> recorded sha256.
_V1_HASHES: dict[str, str] | None = None


def _v1_hashes() -> dict[str, str]:
    global _V1_HASHES
    if _V1_HASHES is None:
        _V1_HASHES = json.load(open(V1_MANIFEST))["cells"]
    return _V1_HASHES


def assert_one_pinned_binary() -> str:
    """Wave 8 reuses wave 1/3 controls instead of re-running them.

    That is only legitimate because both campaigns ran the same pinned binary on
    the same fleet architecture. If the binaries ever differ, the reused controls
    are measurements from a different instrument and every gain below is a
    cross-instrument comparison. Checked, not assumed.
    """
    v1 = json.load(open(V1_MANIFEST))["binary_sha256"]
    v2 = json.load(open(V2_MANIFEST))["pinned_binary_sha256"]
    if v1 != v2:
        raise SystemExit(
            f"PINNED BINARY MISMATCH - reused controls are not comparable.\n"
            f"  wave 1/3: {v1}\n  wave 8:   {v2}"
        )
    return v1


def load(directory: str, stem: str) -> list[dict]:
    """Every seed of one configuration, in registered seed order.

    Cells read from the wave-1 archive are additionally checked against their
    recorded hash. A control that drifted on disk since it was archived is not a
    control, and silently reading it would be indistinguishable from reading the
    real thing.
    """
    verify = os.path.normpath(directory).endswith(
        os.path.join("shd_attention_campaign_v1", "cells")
    )
    out = []
    for seed in SEEDS:
        name = f"{stem}__s{seed}.json"
        path = os.path.join(directory, name)
        if not os.path.exists(path):
            raise SystemExit(f"MISSING CELL {path} - refusing to report a partial arm")
        raw = open(path, "rb").read()
        if verify:
            recorded = _v1_hashes().get(name)
            if recorded is None:
                raise SystemExit(f"REUSED CELL NOT IN MANIFEST: {name}")
            actual = hashlib.sha256(raw).hexdigest()
            if actual != recorded:
                raise SystemExit(
                    f"REUSED CELL DRIFTED: {name}\n"
                    f"  recorded {recorded}\n  actual   {actual}"
                )
        payload = json.loads(raw)
        # Cells emitted after 2026-08-22 carry their own seed. When present it
        # must agree with the filename, because every paired statistic below
        # ("positive in 12 of 12 seeds") assumes treatment and control cells with
        # the same suffix came from the same seed. Older cells have no seed
        # field; their absence is not a failure, only a missing witness.
        recorded_seed = payload.get("seed")
        if recorded_seed is not None and int(recorded_seed) != seed:
            raise SystemExit(
                f"SEED MISMATCH in {name}: filename says {seed}, "
                f"cell says {recorded_seed} - the pairing is not trustworthy"
            )
        out.append(payload)
    return out


# Preregistration section 5, per cell — from the single owner in
# `scripts/cell_validity.py`. This module previously carried its own copy, which
# had no temporal-audit check; wave 9's whole result is a bin-shuffled arm and
# was scored through it, and wave 10 imports this name too. Re-exported rather
# than wrapped so both stay on the same rule.
sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))
from cell_validity import stability_warnings, validity_problems  # noqa: E402


#: Non-voiding notes, printed with the report. A cell can be perfectly valid and
#: still have been straining; a mean cannot say that, so it is said separately.
WARNINGS: list[str] = []


def accs(cells: list[dict], label: str, voided: list[str]) -> list[float]:
    for cell, seed in zip(cells, SEEDS):
        problems = validity_problems(cell)
        if problems:
            voided.append(f"{label} s{seed}: {', '.join(problems)}")
        for warning in stability_warnings(cell):
            WARNINGS.append(f"{label} s{seed}: {warning}")
    return [c["accuracy"] for c in cells]


def mean(xs: list[float]) -> float:
    """Arithmetic mean; **raises** on an empty sequence rather than returning a number.

    Every caller here is guaranteed non-empty by `load`, which refuses a partial
    arm. If that guarantee ever breaks, a bare `sum(xs) / len(xs)` raises
    `ZeroDivisionError` from inside a verdict line, which says nothing about
    which arm was empty. `analyse_wave11.mean` deliberately returns NaN instead,
    because its report tolerates an empty condition; the two are pinned as
    different in `test_campaign_tooling.py`. Do not converge them.
    """
    if not xs:
        raise ValueError("mean of no values: an arm reached a verdict line empty")
    return sum(xs) / len(xs)


def verdict(ok: bool) -> str:
    return "SUPPORTED" if ok else "NOT SUPPORTED"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", default="-")
    args = parser.parse_args()

    pinned = assert_one_pinned_binary()
    voided: list[str] = []
    A = "published-2ms__adjacent-sum-5"

    # --- treatments (wave 8) ---
    geo = accs(load(V2, f"w8geo__ff-fixed-attn__h128__e400__published-2ms__channels-700__d32l4"),
               "w8geo attn", voided)
    w512 = accs(load(V2, f"w8wid__ff-fixed-attn__h512__e400__{A}__d32l4"), "w8wid h512", voided)
    w1024 = accs(load(V2, f"w8wid__ff-fixed-attn__h1024__e400__{A}__d32l4"), "w8wid h1024", voided)
    con_a = accs(load(V2, "w8con__ff-fixed-attn__h128__e400__published-10ms__adjacent-sum-5__d32l4"),
                 "w8con attn", voided)
    con_b = accs(load(V2, "w8con__ff-fixed__h128__e400__published-10ms__adjacent-sum-5"),
                 "w8con ctrl", voided)
    lyr2 = accs(load(V2, f"w8lyr__ff-fixed-attn__h128__e400__{A}__d32l2"), "w8lyr L2", voided)

    # --- reused controls (wave 1 / wave 3 / registered run, same pinned binary) ---
    geo_c = accs(load(V1, "w3geo__ff-fixed__h128__e400__published-2ms__channels-700"),
                 "w3geo ctrl", voided)
    c512 = accs(load(V1, f"w3wid__ff-fixed__h512__e400__{A}"), "w3wid h512 ctrl", voided)
    c1024 = accs(load(V1, f"w3wid__ff-fixed__h1024__e400__{A}"), "w3wid h1024 ctrl", voided)
    c128 = accs(load(V1, f"w1__ff-fixed__h128__e400__{A}"), "w1 h128 ctrl", voided)
    lyr1 = accs(load(V1, f"w1__ff-fixed-attn__h128__e400__{A}__d32l1"), "w1 L1", voided)
    lyr4 = accs(load(V1, f"r1cal__ff-fixed-attn__h128__e400__{A}__d32l4"), "R d32l4", voided)

    lines: list[str] = []
    w = lines.append

    w("# Wave 8 — registered verdicts\n")
    w("Prereg: `PREREG_2026-08-20_SHD_ATTENTION_HEADLINE_SCOPE.md` §3. "
      "All 72 wave-8 cells present, plus 72 reused control cells from waves 1/3 "
      "and the registered run, same pinned binary `22d97c51ab02`.\n")

    if voided:
        w("## VALIDITY GATES FAILED — nothing below is reportable\n")
        for line in voided:
            w(f"- {line}")
        print("\n".join(lines))
        return 1
    w("**Validity gates: all 144 cells pass.**\n")
    # Valid is not the same as untroubled. A cell can clear every gate and still
    # have been straining; printing that is the difference between "this arm
    # scored lower" and "this arm scored lower while straining".
    if WARNINGS:
        w(f"**Stability notes ({len(WARNINGS)}), not voiding:**\n")
        for line in WARNINGS:
            w(f"- {line}")
        w("")
    else:
        w("**Stability notes: none — no cell exceeded the recorded peak "
          "gradient norm, and no cell was clipped.**\n")

    w("## Measurements\n")
    w("| configuration | mean | min | max | seeds >= 0.80 |")
    w("|---|---:|---:|---:|---:|")
    for label, xs in [
        ("d32/L4 anchor h128 *(registered run)*", lyr4),
        ("d32/L2 anchor h128", lyr2),
        ("d32/L1 anchor h128 *(wave 1)*", lyr1),
        ("`ff+fixed` anchor h128 *(wave 1)*", c128),
        ("d32/L4 `channels-700`", geo),
        ("`ff+fixed` `channels-700` *(wave 3)*", geo_c),
        ("d32/L4 h512", w512),
        ("`ff+fixed` h512 *(wave 3)*", c512),
        ("d32/L4 h1024", w1024),
        ("`ff+fixed` h1024 *(wave 3)*", c1024),
        ("d32/L4 `published-10ms`", con_a),
        ("`ff+fixed` `published-10ms`", con_b),
    ]:
        w(f"| {label} | {mean(xs):.4f} | {min(xs):.4f} | {max(xs):.4f} | "
          f"{sum(1 for a in xs if a >= 0.80)}/12 |")

    w("\n## Verdicts\n")

    # S-1
    n80 = sum(1 for a in geo if a >= 0.80)
    s1 = mean(geo) >= 0.80 and n80 >= 9
    w(f"**S-1** d32/L4 clears the gate on `channels-700`: mean **{mean(geo):.4f}** "
      f"(bar 0.80), **{n80}/12** seeds >= 0.80 (bar 9). -> **{verdict(s1)}**\n")

    # S-2
    d2 = [a - b for a, b in zip(geo, geo_c)]
    n2 = sum(1 for d in d2 if d > 0)
    s2 = mean(d2) >= 0.05 and n2 >= 10
    w(f"**S-2** gain on `channels-700`: **{mean(d2):+.4f}** (bar +0.05), positive in "
      f"**{n2}/12** seeds (bar 10). -> **{verdict(s2)}**\n")

    # S-3
    d3 = [a - b for a, b in zip(w1024, c1024)]
    n3 = sum(1 for d in d3 if d > 0)
    s3 = mean(d3) >= 0.05 and n3 >= 10
    w(f"**S-3** gain at h1024 with L4: **{mean(d3):+.4f}** (bar +0.05), positive in "
      f"**{n3}/12** seeds (bar 10). Wave 3 measured **-0.0159** at d32/L1. "
      f"-> **{verdict(s3)}**\n")

    # S-3b (descriptive)
    g128 = mean([a - b for a, b in zip(lyr4, c128)])
    g512 = mean([a - b for a, b in zip(w512, c512)])
    g1024 = mean(d3)
    w(f"**S-3b** *(reported, no threshold)* gain vs width at L4: "
      f"h128 **{g128:+.4f}**, h512 **{g512:+.4f}**, h1024 **{g1024:+.4f}**. "
      f"Monotone decreasing: **{g128 >= g512 >= g1024}**\n")

    # S-4
    d4 = [a - b for a, b in zip(con_a, con_b)]
    n4 = sum(1 for d in d4 if d > 0)
    s4 = mean(d4) >= 0.05 and n4 >= 10
    w(f"**S-4** gain on `published-10ms`: **{mean(d4):+.4f}** (bar +0.05), positive in "
      f"**{n4}/12** seeds (bar 10). -> **{verdict(s4)}**\n")

    # S-5 (mechanistic)
    s5 = mean(d4) <= g128 - 0.02
    w(f"**S-5** *(mechanistic)* the gain shrinks with fewer timesteps: "
      f"`published-10ms` (t=72) **{mean(d4):+.4f}** vs `published-2ms` (t=358) "
      f"**{g128:+.4f}**; bar is <= **{g128 - 0.02:+.4f}**. -> **{verdict(s5)}**\n")

    # S-6
    step12 = mean(lyr2) - mean(lyr1)
    step24 = mean(lyr4) - mean(lyr2)
    s6 = step12 >= 0.0 and step24 >= 0.0
    w(f"**S-6** depth ladder monotone at e400: L1 **{mean(lyr1):.4f}** -> "
      f"L2 **{mean(lyr2):.4f}** ({step12:+.4f}) -> L4 **{mean(lyr4):.4f}** "
      f"({step24:+.4f}). -> **{verdict(s6)}**\n")

    text = "\n".join(lines)
    if args.out == "-":
        print(text)
    else:
        open(args.out, "w").write(text + "\n")
        print(f"wrote {args.out}", file=sys.stderr)
        print(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
