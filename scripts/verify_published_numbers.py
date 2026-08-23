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


def acc_present(root: Path, stem: str) -> dict[int, float]:
    """Accuracies keyed by seed, for arms where a cell may legitimately be absent.

    `acc` above refuses a missing cell, which is right for the waves whose arms
    are complete by construction. Waves 13 and 14 measure arms that diverge, so
    absence is data there and this returns what exists.

    A cell that failed the validity gate is **not** an accuracy: wave 13's
    `rec+fixed` cells were emitted while saturating, and counting them would
    inflate both the completion counts and any mean taken over them. The gate
    comes from `cell_validity`, its single owner - the independence this file
    needs is from the ANALYSER'S ARITHMETIC, which is what gets transcribed into
    the paper, not from the validity rule.
    """
    sys.path.insert(0, str(ROOT / "scripts"))
    from cell_validity import validity_problems

    out = {}
    for seed in SEEDS:
        path = root / f"{stem}__s{seed}.json"
        if not path.is_file():
            continue
        cell = json.loads(path.read_text())
        if validity_problems(cell):
            continue
        out[seed] = cell["accuracy"]
    return out


def paired_gain(treatment: dict[int, float], control: dict[int, float]) -> tuple[float, int]:
    """Mean of `treatment - control` over seeds present in BOTH, and how many.

    Written here rather than imported: pooling instead of intersecting is the
    failure mode this cross-check exists to catch in the analyser, so it must
    not share the analyser's implementation of it.
    """
    shared = sorted(set(treatment) & set(control))
    if not shared:
        raise SystemExit("no seed completed in both arms")
    return sum(treatment[s] - control[s] for s in shared) / len(shared), len(shared)


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

    # ---- wave 12: adaptation x attention, scale 1.0 ------------------------
    W12 = "RESULT_2026-08-23_W12_ATTENTION_DOES_NOT_SUBSTITUTE_FOR_ADAPTATION.md"
    alif = acc(V2, f"w12ada__ff-alif__h128__e400__{ANCHOR}")
    alif_attn = acc(V2, f"w12ada__ff-alif-attn__h128__e400__{ANCHOR}__d32l4")
    results.append(check("A-2 ff+alif gain", avg(alif_attn) - avg(alif),
                         published(W12, r"gain \*\*([+-]?\d\.\d+)\*\*\s*\n?\(bar")))
    results.append(check("A-1 difference of gains",
                         (avg(alif_attn) - avg(alif)) - gain,
                         published(W12, r"gains is \*\*([+-]?\d\.\d+)\*\*")))
    results.append(check("A-3 ff+alif mean", avg(alif),
                         published(W12, r"`ff\+alif` reaches\s*\n?\*\*(0\.\d+)\*\*")))

    # ---- wave 13: completion, not accuracy --------------------------------
    W13 = "RESULT_2026-08-23_W13_RECURRENT_STABILITY.md"
    rec04 = acc_present(V2, f"w13rec__rec-alif__h128__e400__{ANCHOR}__ss0.4")
    results.append(check("R-1 rec+alif ss0.4 completions", float(len(rec04)),
                         published(W13, r"\| `rec\+alif` \| \*\*0\.4\*\* \| \*\*(\d+)/12\*\*")))

    # ---- wave 14: paired over the intersection, scale 0.4 -----------------
    W14 = "RESULT_2026-08-23_W14_ATTENTION_AND_RECURRENCE_ARE_COMPLEMENTARY.md"
    rec_attn = acc_present(V2, f"w14sub__rec-alif-attn__h128__e400__{ANCHOR}__d32l4__ss0.4")
    ff04 = acc_present(V2, f"w14sub__ff-fixed__h128__e400__{ANCHOR}__ss0.4")
    ff04_attn = acc_present(V2, f"w14sub__ff-fixed-attn__h128__e400__{ANCHOR}__d32l4__ss0.4")

    rec_gain, rec_pairs = paired_gain(rec_attn, rec04)
    ff_gain, ff_pairs = paired_gain(ff04_attn, ff04)
    results.append(check(f"M-1 rec+alif paired gain (n={rec_pairs})", rec_gain,
                         published(W14, r"\| `rec\+alif` \| 10 \| 0\.\d+ \| 0\.\d+ \| \*\*([+-]?\d\.\d+)\*\*")))
    results.append(check(f"ff+fixed paired gain at 0.4 (n={ff_pairs})", ff_gain,
                         published(W14, r"\| `ff\+fixed` \| 12 \| 0\.\d+ \| 0\.\d+ \| \*\*([+-]?\d\.\d+)\*\*")))
    results.append(check("M-2 difference of gains", rec_gain - ff_gain,
                         published(W14, r"difference \*\*([+-]?\d\.\d+)\*\* against a two-sided bar")))
    results.append(check("M-4 ff+fixed mean at scale 0.4",
                         sum(ff04[k] for k in sorted(set(ff04) & set(ff04_attn)))
                         / len(set(ff04) & set(ff04_attn)),
                         published(W14, r"at scale 0\.4 is \*\*(0\.\d+)\*\*")))

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
        # The four load-bearing caveats on section 3.7. Each is the kind of
        # sentence an editing pass shortens away, and each is what stops the
        # recurrent result reading larger than it is.
        "that the read-out is not causal": "not causal",
        "the headroom normalisation of the recurrent gain": "1.34",
        "that the recurrent substrate does not win": "does not win",
        "that ten pairs is the registered minimum": "the registered minimum",
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
