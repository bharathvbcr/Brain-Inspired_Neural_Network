#!/usr/bin/env python3
"""Recompute every published campaign number from the cells, independently.

`analyse_wave8.py` produced the verdicts and a human pasted them into the result
documents. Both steps can be wrong, and neither checks the other: a bug in the
analyser would be faithfully transcribed, and a transcription slip would never be
caught by re-running the analyser.

So this reads the numbers **out of the published markdown** and recomputes them
**from the cell JSON with a separate implementation** that shares no code with the
analyser. It is a cross-check of the paper's numbers, not of the analyser's code.

Exit 1 on any mismatch. Run: python3 scripts/verify_published_numbers.py
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
V1 = ROOT / "results/shd_attention_campaign_v1/cells"
V2 = ROOT / "results/shd_attention_campaign_v2"
SEEDS = [5170001 + i for i in range(12)]
ANCHOR = "published-2ms__adjacent-sum-5"


def acc(root: Path, stem: str) -> list[float]:
    out = []
    for seed in SEEDS:
        path = root / f"{stem}__s{seed}.json"
        if not path.is_file():
            raise SystemExit(f"missing cell {path}")
        out.append(json.loads(path.read_text())["accuracy"])
    return out


def avg(xs):
    return sum(xs) / len(xs)


def published(doc: str, pattern: str) -> float:
    """Pull a number out of a result document by regex."""
    text = (ROOT / "results" / doc).read_text()
    found = re.search(pattern, text)
    if not found:
        raise SystemExit(f"pattern not found in {doc}: {pattern}")
    return float(found.group(1).replace("−", "-").replace("+", ""))


def check(label: str, computed: float, claimed: float, tol: float = 5e-5) -> bool:
    ok = abs(computed - claimed) <= tol
    mark = "ok  " if ok else "FAIL"
    print(f"  [{mark}] {label:<46} computed {computed:+.4f}  published {claimed:+.4f}")
    return ok


def main() -> int:
    print("Recomputing published numbers from cells (independent implementation)\n")
    results = []

    # ---- wave 8 -----------------------------------------------------------
    W8 = "RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md"
    geo_attn = acc(V2, f"w8geo__ff-fixed-attn__h128__e400__published-2ms__channels-700__d32l4")
    geo_rate = acc(V1, f"w3geo__ff-fixed__h128__e400__published-2ms__channels-700")
    results.append(check("S-1 channels-700 d32/L4 mean",
                         avg(geo_attn), published(W8, r"\*\*S-1\*\* \| [^|]+\| (0\.\d+),")))
    results.append(check("S-2 channels-700 gain",
                         avg(geo_attn) - avg(geo_rate),
                         published(W8, r"\*\*S-2\*\* \| [^|]+\| \*\*([+-]?\d\.\d+)\*\*")))

    w_attn = acc(V2, f"w8wid__ff-fixed-attn__h1024__e400__{ANCHOR}__d32l4")
    w_rate = acc(V1, f"w3wid__ff-fixed__h1024__e400__{ANCHOR}")
    results.append(check("S-3 h1024 gain", avg(w_attn) - avg(w_rate),
                         published(W8, r"\*\*S-3\*\* \| [^|]+\| \*\*([+-−]?\d\.\d+)\*\*")))

    con_attn = acc(V2, f"w8con__ff-fixed-attn__h128__e400__published-10ms__adjacent-sum-5__d32l4")
    con_rate = acc(V2, f"w8con__ff-fixed__h128__e400__published-10ms__adjacent-sum-5")
    results.append(check("S-4 published-10ms gain", avg(con_attn) - avg(con_rate),
                         published(W8, r"\*\*S-4\*\* \| [^|]+\| \*\*([+-]?\d\.\d+)\*\*")))

    l2 = acc(V2, f"w8lyr__ff-fixed-attn__h128__e400__{ANCHOR}__d32l2")
    results.append(check("S-6 L2 mean", avg(l2),
                         published(W8, r"\| d32/L2 anchor h128 \| (0\.\d+) \|")))

    # ---- wave 9 -----------------------------------------------------------
    W9 = "RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md"
    intact = acc(V1, f"r1cal__ff-fixed-attn__h128__e400__{ANCHOR}__d32l4")
    shuf = acc(V2, f"w9shf__ff-fixed-attn__h128__e400__{ANCHOR}__d32l4__bin-shuffled")
    ctl = acc(V1, f"w1__ff-fixed__h128__e400__{ANCHOR}")
    ctl_shuf = acc(V1, f"w1__ff-fixed__h128__e400__{ANCHOR}__bin-shuffled")
    d64 = acc(V2, f"w9dim__ff-fixed-attn__h128__e400__{ANCHOR}__d64l4")

    results.append(check("M-1 intact - shuffled at d32/L4", avg(intact) - avg(shuf),
                         published(W9, r"\| \*\*d32/L4\*\* \*\(the headline\)\* \| \*\*0\.\d+\*\* \| \*\*0\.\d+\*\* \| \*\*([+-]?\d\.\d+)\*\*")))
    results.append(check("M-2 shuffle cost, plain arm", avg(ctl) - avg(ctl_shuf),
                         published(W9, r"the plain\s*\n?arm \*\*([+-]?\d\.\d+)\*\*")))
    results.append(check("headline d32/L4 mean", avg(intact),
                         published(W9, r"\| \*\*d32/L4\*\* \*\(the headline\)\* \| \*\*(0\.\d+)\*\*")))
    results.append(check("M-3 d64 - d32 at h128", avg(d64) - avg(intact),
                         published(W9, r"\*\*mean\(d64/L4\) − mean\(d32/L4\) = ([+-]?\d\.\d+)\*\*")))

    # The headline gain, quoted in four separate documents.
    gain = avg(intact) - avg(ctl)
    results.append(check("headline gain over ff+fixed", gain,
                         published("RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md",
                                   r"\*\*R-3\*\* \| gain \*\*([+-]?\d\.\d+)\*\*")))

    # Seed counts, not just means.
    n_pos = sum(1 for a, b in zip(intact, shuf) if a > b)
    print(f"  [{'ok  ' if n_pos == 12 else 'FAIL'}] "
          f"{'M-1 seeds intact > shuffled':<46} computed {n_pos}/12       published 12/12")
    results.append(n_pos == 12)
    n_gate = sum(1 for a in intact if a >= 0.80)
    print(f"  [{'ok  ' if n_gate == 12 else 'FAIL'}] "
          f"{'headline seeds >= 0.80':<46} computed {n_gate}/12       published 12/12")
    results.append(n_gate == 12)

    # ---- the paper draft must not resurrect a withdrawn claim ---------------
    print("\n  paper draft:")
    draft = (ROOT / "results/PAPER_DRAFT.md").read_text()
    forbidden = {
        "v130 PASS as a live claim": r"gap LCB `?0\.9988`?[^—]*PASS",
        "live-transfer 1.0000/0.9983 cited as a result": r"mean 1\.0000 / LCB 0\.9983",
    }
    for label, pattern in forbidden.items():
        hit = re.search(pattern, draft)
        ok = hit is None
        print(f"  [{'ok  ' if ok else 'FAIL'}] draft does not cite {label}")
        results.append(ok)
    required = {
        "the A6 undertraining caveat": "The reference is undertrained, and the task saturates",
        "the saturation consequence": "learning speed",
    }
    for label, needle in required.items():
        ok = needle in draft
        print(f"  [{'ok  ' if ok else 'FAIL'}] draft states {label}")
        results.append(ok)

    bad = results.count(False)
    print(f"\n{len(results) - bad}/{len(results)} published numbers reproduce from the cells.")
    if bad:
        print("MISMATCH — a published number does not follow from the archived cells.")
    return 1 if bad else 0


if __name__ == "__main__":
    raise SystemExit(main())
