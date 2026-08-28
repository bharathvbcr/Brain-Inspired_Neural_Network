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

#: The number of checks this script must run. See the floor test at the end of
#: `main` for why a count that only ever prints itself is not a check.
MIN_CHECKS = 82


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


def acc_present(root: Path, stem: str, seeds=None) -> dict[int, float]:
    """Accuracies keyed by seed, for arms where a cell may legitimately be absent.

    `seeds` defaults to the campaign's twelve. Wave 17 extends two arms to
    thirty-two, and reading those with the default would silently return the
    archived twelve and call it n=32 -- which it did, once.

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
    for seed in (SEEDS if seeds is None else seeds):
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

    results.append(check("d32/L4 under bin-shuffling", avg(shuf),
                         published(W9, r"\| \*\*d32/L4\*\* \*\(the headline\)\* \| \*\*0\.\d+\*\* \| \*\*(0\.\d+)\*\*")))
    results.append(check("ff+fixed under bin-shuffling", avg(ctl_shuf),
                         published(W9, r"\| `ff\+fixed` \| 0\.\d+ \| (0\.\d+) \|")))
    results.append(check("residual gain under shuffling", avg(shuf) - avg(ctl_shuf),
                         published(W9, r"\| gain of d32/L4 over `ff\+fixed` \| \*\*[+-]?\d\.\d+\*\* \| \*\*([+-]?\d\.\d+)\*\*")))

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

    # ---- wave 7: the sample-efficiency number the PAPER cites ---------------
    #
    # 3.5 item 3 quotes "98.1% of e400 accuracy by 10 epochs (0.7337)". Nothing
    # recomputed it until now, which made it the only number in the SHD sections
    # of the draft resting on transcription alone.
    W7 = "RESULT_2026-08-20_W7_CONVERGENCE_IS_BRACKETED.md"
    e10_attn = acc(V1, f"w7flr__ff-fixed-attn__h128__e10__{ANCHOR}__d32l1")
    e10_rate = acc(V1, f"w7flr__ff-fixed__h128__e10__{ANCHOR}")
    e5_attn = acc(V1, f"w7flr__ff-fixed-attn__h128__e5__{ANCHOR}__d32l1")
    results.append(check("W7 e10 attention mean", avg(e10_attn),
                         published(W7, r"\| 10 \| 0\.\d+ \| (0\.\d+) \|")))
    results.append(check("W7 e10 gain over the rate arm", avg(e10_attn) - avg(e10_rate),
                         published(W7, r"\| 10 \| 0\.\d+ \| 0\.\d+ \| ([+-]?\d\.\d+) \|")))
    # The percentage the paper quotes: e10 attention against the d32/L1 arm at
    # convergence, which is wave 1's `w1` cell, not the d32/L4 headline.
    l1_converged = acc(V1, f"w1__ff-fixed-attn__h128__e400__{ANCHOR}__d32l1")
    results.append(check("W7 e10 as a fraction of e400 (%)",
                         100.0 * avg(e10_attn) / avg(l1_converged),
                         published(W7, r"\| 10 \| 0\.\d+ \| 0\.\d+ \| [+-]?\d\.\d+ \| \*\*(\d+\.\d)%\*\*"),
                         tol=0.05))
    results.append(check("W7 e5 attention mean", avg(e5_attn),
                         published(W7, r"\| 5 \| 0\.\d+ \| (0\.\d+) \|")))

    # ---- wave 10: the resolution ladder, on a family that isolates it ------
    W10 = "RESULT_2026-08-22_W10_RESOLUTION_LADDER.md"
    ladder = {}
    for rung in ("fixed-t100", "fixed-t250", "fixed-t500"):
        attn = acc(V2, f"w10con__ff-fixed-attn__h128__e400__{rung}__adjacent-sum-5__d32l4")
        rate = acc(V2, f"w10con__ff-fixed__h128__e400__{rung}__adjacent-sum-5")
        ladder[rung] = (avg(attn), avg(rate))
        results.append(check(f"{rung} rate-readout mean", avg(rate),
                             published(W10, rf"\| `{rung}` \| [\d.]+ \| (0\.\d+) \|")))
        results.append(check(f"{rung} d32/L4 mean", avg(attn),
                             published(W10, rf"\| `{rung}` \| [\d.]+ \| 0\.\d+ \| (0\.\d+) \|")))
        results.append(check(f"C-1 {rung} gain", avg(attn) - avg(rate),
                             published(W10, rf"\| `{rung}` \| [\d.]+ \| 0\.\d+ \| 0\.\d+ \| \*\*([+-]?\d\.\d+)\*\*")))
    results.append(check("C-2 gain(t500) - gain(t100)",
                         (ladder["fixed-t500"][0] - ladder["fixed-t500"][1])
                         - (ladder["fixed-t100"][0] - ladder["fixed-t100"][1]),
                         published(W10, r"gain\(t500\) − gain\(t100\) = \*\*([+-−]?\d\.\d+)\*\*")))
    results.append(check("C-3 baseline drift across the ladder",
                         ladder["fixed-t500"][1] - ladder["fixed-t100"][1],
                         published(W10, r"`ff\+fixed` t500 − t100 = \*\*([+-]?\d\.\d+)\*\*")))

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
    def paired_mean(values: dict[int, float], other: dict[int, float]) -> float:
        shared = sorted(set(values) & set(other))
        return sum(values[k] for k in shared) / len(shared)

    for label, arm, partner, pat in (
        ("rec+alif paired mean", rec04, rec_attn,
         r"\| `rec\+alif` \| 10 \| (0\.\d+) \|"),
        ("rec+alif+attn paired mean", rec_attn, rec04,
         r"\| `rec\+alif` \| 10 \| 0\.\d+ \| (0\.\d+) \|"),
        ("ff+fixed+attn paired mean at 0.4", ff04_attn, ff04,
         r"\| `ff\+fixed` \| 12 \| 0\.\d+ \| (0\.\d+) \|"),
    ):
        results.append(check(label, paired_mean(arm, partner), published(W14, pat)))

    results.append(check("M-2 difference of gains", rec_gain - ff_gain,
                         published(W14, r"difference \*\*([+-]?\d\.\d+)\*\* against a two-sided bar")))
    results.append(check("M-4 ff+fixed mean at scale 0.4",
                         sum(ff04[k] for k in sorted(set(ff04) & set(ff04_attn)))
                         / len(set(ff04) & set(ff04_attn)),
                         published(W14, r"at scale 0\.4 is \*\*(0\.\d+)\*\*")))

    # ---- Azure: the width ladder, including the rung only Azure ran --------
    # The four h256/d32-L4 cells are the only ones of their kind in the project
    # — wave 8 ran h128, h512 and h1024 and skipped h256 — so this rung has no
    # second copy to check it against and needs the named check most.
    AZ = ROOT / "results/azure-d32l4-scope-v1/results"
    AZDOC = "RESULT_2026-08-22_AZURE_TRUNCATED_AT_95_OF_252.md"
    for width, attn_root, attn_stem, rate_root, rate_stem, pattern in (
        (128, V1, f"r1cal__ff-fixed-attn__h128__e400__{ANCHOR}__d32l4",
         V1, f"w1__ff-fixed__h128__e400__{ANCHOR}",
         r"\| h128 \| 12 \| 0\.\d+ \| 0\.\d+ \| \*\*([+-]?\d\.\d+)\*\*"),
        (256, AZ, f"az8wid__ff-fixed-attn__h256__e400__{ANCHOR}__d32l4",
         V1, f"w3wid__ff-fixed__h256__e400__{ANCHOR}",
         r"\| \*\*h256\*\* \| \*\*4\*\* \| \*\*0\.\d+\*\* \| \*\*0\.\d+\*\* \| \*\*([+-]?\d\.\d+)\*\*"),
        (512, V2, f"w8wid__ff-fixed-attn__h512__e400__{ANCHOR}__d32l4",
         V1, f"w3wid__ff-fixed__h512__e400__{ANCHOR}",
         r"\| h512 \| 12 \| 0\.\d+ \| 0\.\d+ \| \*\*([+-]?\d\.\d+)\*\*"),
        (1024, AZ, f"az8wid__ff-fixed-attn__h1024__e400__{ANCHOR}__d32l4",
         V1, f"w3wid__ff-fixed__h1024__e400__{ANCHOR}",
         r"\| h1024 \| 12 \| 0\.\d+ \| 0\.\d+ \| \*\*([+-−]?\d\.\d+)\*\*"),
    ):
        treatment = acc_present(attn_root, attn_stem)
        control = acc_present(rate_root, rate_stem)
        gain, pairs = paired_gain(treatment, control)
        results.append(check(f"width ladder d32/L4 gain at h{width} (n={pairs})",
                             gain, published(AZDOC, pattern)))

    # ---- The six-rung ladder, which says something different -------------
    # The four-rung check above still guards the Azure document's own table and
    # still passes. It is kept rather than replaced because those four numbers
    # did not move. What moved is the SHAPE, and the shape read off four rungs
    # is not the shape read off six, so the superseding claim gets its own
    # assertion instead of inheriting the old one.
    six = []
    for width, attn_root, attn_stem, rate_root, rate_stem in (
        (128, V1, f"r1cal__ff-fixed-attn__h128__e400__{ANCHOR}__d32l4",
         V1, f"w1__ff-fixed__h128__e400__{ANCHOR}"),
        (256, V2, f"w16lad__ff-fixed-attn__h256__e400__{ANCHOR}__d32l4",
         V2, f"w16lad__ff-fixed__h256__e400__{ANCHOR}"),
        (384, V2, f"w16lad__ff-fixed-attn__h384__e400__{ANCHOR}__d32l4",
         V2, f"w16lad__ff-fixed__h384__e400__{ANCHOR}"),
        (512, V2, f"w8wid__ff-fixed-attn__h512__e400__{ANCHOR}__d32l4",
         V1, f"w3wid__ff-fixed__h512__e400__{ANCHOR}"),
        (768, V2, f"w16lad__ff-fixed-attn__h768__e400__{ANCHOR}__d32l4",
         V2, f"w16lad__ff-fixed__h768__e400__{ANCHOR}"),
        (1024, V2, f"w8wid__ff-fixed-attn__h1024__e400__{ANCHOR}__d32l4",
         V1, f"w3wid__ff-fixed__h1024__e400__{ANCHOR}"),
    ):
        six.append(paired_gain(acc_present(attn_root, attn_stem),
                               acc_present(rate_root, rate_stem))[0])
    not_monotone = not all(a > b for a, b in zip(six, six[1:]))
    collapse_is_last_step = six[4] > 0 > six[5]
    only_one_inversion = sum(a <= b for a, b in zip(six, six[1:])) == 1
    six_ok = not_monotone and collapse_is_last_step and only_one_inversion
    print(f"  [{'ok  ' if six_ok else 'FAIL'}] "
          f"{'six-rung ladder: one inversion, collapse at h1024':<46} "
          f"{'; '.join(f'{g:+.4f}' for g in six)}")
    results.append(six_ok)

    # ---- Waves 15-17: the n=32 headline and the two second-order gaps ------
    # These are the quantities `check_every_number.py` deliberately does not
    # generate -- merged cross-wave arms and differences OF gains -- so they are
    # verified by name here and excluded there by name, with each pointing at
    # the other.
    W1517 = "RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md"

    W17_SEEDS = [5170001 + i for i in range(32)]

    def merged_arm(archive_stem, new_stem):
        """The archived twelve seeds plus the twenty new ones, one arm.

        The new stem is read over all thirty-two seeds, not the default twelve:
        reading it with the default returns nothing and leaves the archived
        twelve wearing an n=32 label.
        """
        cells = dict(acc_present(V2, archive_stem) or acc_present(V1, archive_stem))
        cells.update(acc_present(V2, new_stem, W17_SEEDS))
        return cells

    h_rate = merged_arm(f"w1__ff-fixed__h128__e400__{ANCHOR}",
                        f"w17hdl__ff-fixed__h128__e400__{ANCHOR}")
    h_attn = merged_arm(f"r1cal__ff-fixed-attn__h128__e400__{ANCHOR}__d32l4",
                        f"w17hdl__ff-fixed-attn__h128__e400__{ANCHOR}__d32l4")
    shared = sorted(set(h_rate) & set(h_attn))
    results.append(check(f"H17-1 rate mean at n={len(shared)}",
                         sum(h_rate[s] for s in shared) / len(shared),
                         published(W1517, r"\| \*\*32\*\* \| \*\*(0\.\d+)\*\*")))
    results.append(check(f"H17-1 attention mean at n={len(shared)}",
                         sum(h_attn[s] for s in shared) / len(shared),
                         published(W1517, r"\| \*\*32\*\* \| \*\*0\.\d+\*\* \| \*\*(0\.\d+)\*\*")))
    results.append(check(f"H17-1 paired gain at n={len(shared)}",
                         paired_gain(h_attn, h_rate)[0],
                         published(W1517, r"\| \*\*32\*\* \| \*\*0\.\d+\*\* \| \*\*0\.\d+\*\* \| \*\*\+(0\.\d+)\*\*")))

    # ---- Waves 18-19: the depth ladder's MEDIAN accuracy columns -----------
    # `check_every_number.py` generates means, extremes and per-seed values and
    # deliberately no medians, so these four are verified by name here. The
    # analyser prints medians for accuracy and a MEAN for the paired gain, which
    # is why the columns must not be subtracted: at L3 the difference of medians
    # is +0.0446 against a paired mean gain of +0.0371.
    W1819 = "RESULT_2026-08-28_W18_19_THE_DEPTH_OPTIMUM_IS_INTERIOR.md"
    import statistics as _stats

    w18_rate = acc_present(V2, f"w18dep__ff-fixed__h1024__e400__{ANCHOR}",
                           [5170001 + i for i in range(20)])
    for depth, expected_row in ((1, r"\| L1 \| 20 \| 0\.\d+ \| (0\.\d+)"),
                                (2, r"\| \*\*L2\*\* \| 20 \| 0\.\d+ \| (0\.\d+)"),
                                (3, r"\| L3 \| 20 \| 0\.\d+ \| (0\.\d+)"),
                                (4, r"\| L4 \| 20 \| 0\.\d+ \| (0\.\d+)")):
        cells = acc_present(
            V2, f"w18dep__ff-fixed-attn__h1024__e400__{ANCHOR}__d32l{depth}",
            [5170001 + i for i in range(20)])
        shared = sorted(set(cells) & set(w18_rate))
        results.append(check(
            f"H18-1 L{depth} attention median over {len(shared)} shared seeds",
            _stats.median(cells[s] for s in shared),
            published(W1819, expected_row)))

    # ---- Wave 20: the recurrent claim at n=32 ------------------------------
    # Same shape as waves 15-17 and the same reason: merged cross-wave arms and
    # a difference OF gains, neither of which `check_every_number.py` generates.
    # Recomputed here from the cells with an implementation that shares no code
    # with `analyse_wave20.py`, so the analyser and the write-up are checked by
    # something that is not either of them.
    W20 = "RESULT_2026-08-28_W20_THE_RECURRENT_CLAIM_HOLDS_AT_THIRTY_TWO_SEEDS.md"
    W20_SEEDS = [5170001 + i for i in range(32)]
    SS = "ss0.4"

    def w20_arm(archive_stem, new_stem):
        cells = dict(acc_present(V2, archive_stem) or acc_present(V1, archive_stem))
        cells.update(acc_present(V2, new_stem, W20_SEEDS))
        return cells

    # The archived twelve of each arm sit under DIFFERENT wave prefixes: the
    # recurrent rate arm is `w13rec`, the other three are `w14sub`. Using one
    # prefix for all four returns an empty rate arm and a plausible-looking
    # wrong answer.
    rec_rate = w20_arm(f"w13rec__rec-alif__h128__e400__{ANCHOR}__{SS}",
                       f"w20rec__rec-alif__h128__e400__{ANCHOR}__{SS}")
    rec_attn = w20_arm(f"w14sub__rec-alif-attn__h128__e400__{ANCHOR}__d32l4__{SS}",
                       f"w20rec__rec-alif-attn__h128__e400__{ANCHOR}__d32l4__{SS}")
    ff_rate = w20_arm(f"w14sub__ff-fixed__h128__e400__{ANCHOR}__{SS}",
                      f"w20rec__ff-fixed__h128__e400__{ANCHOR}__{SS}")
    ff_attn = w20_arm(f"w14sub__ff-fixed-attn__h128__e400__{ANCHOR}__d32l4__{SS}",
                      f"w20rec__ff-fixed-attn__h128__e400__{ANCHOR}__d32l4__{SS}")

    rec_shared = sorted(set(rec_rate) & set(rec_attn))
    ff_shared = sorted(set(ff_rate) & set(ff_attn))
    results.append(check(f"H20-2 usable recurrent pairs is {len(rec_shared)}",
                         float(len(rec_shared)),
                         published(W20, r"\| \*\*usable pairs\*\* \| \*\*(\d+)\*\* \|"),
                         tol=0.5))

    def mean(cells, seeds):
        return sum(cells[s] for s in seeds) / len(seeds)

    rec_base, rec_top = mean(rec_rate, rec_shared), mean(rec_attn, rec_shared)
    ff_base, ff_top = mean(ff_rate, ff_shared), mean(ff_attn, ff_shared)
    rec_g = paired_gain(rec_attn, rec_rate)[0]
    ff_g = paired_gain(ff_attn, ff_rate)[0]
    both = [s for s in rec_shared
            if s in ff_rate and s in ff_attn]
    did = sum((rec_attn[s] - rec_rate[s]) - (ff_attn[s] - ff_rate[s])
              for s in both) / len(both)

    for label, computed, pattern in (
        ("H20-1 recurrent rate mean", rec_base,
         r"`rec\+alif` \| 24 \| (0\.\d+)"),
        ("H20-1 recurrent attention mean", rec_top,
         r"`rec\+alif` \| 24 \| 0\.\d+ \| (0\.\d+)"),
        ("H20-1 recurrent paired gain", rec_g,
         r"`rec\+alif` \| 24 \| 0\.\d+ \| 0\.\d+ \| \*\*\+(0\.\d+)\*\*"),
        ("H20-1 feed-forward rate mean", ff_base,
         r"`ff\+fixed` \| 32 \| (0\.\d+)"),
        ("H20-1 feed-forward attention mean", ff_top,
         r"`ff\+fixed` \| 32 \| 0\.\d+ \| (0\.\d+)"),
        ("H20-1 feed-forward paired gain", ff_g,
         r"`ff\+fixed` \| 32 \| 0\.\d+ \| 0\.\d+ \| \*\*\+(0\.\d+)\*\*"),
        # SEED-PAIRED across all four arms, not the difference of two
        # independently averaged gains. Those are different numbers: the second
        # gives +0.1568 here because the feed-forward gain is averaged over 32
        # seeds while the recurrent one has only 24, so the two means are not
        # taken over the same seeds. The registration says "seed-paired
        # difference of gains", and a difference-in-differences that is not
        # paired is not one.
        ("H20-1 difference of gains", did,
         r"difference of gains \*\*\+(0\.\d+)\*\*"),
        ("H20-4 recurrent headroom", 1.0 - rec_base,
         r"`rec\+alif` \| 0\.\d+ \| (0\.\d+)"),
        ("H20-4 feed-forward headroom", 1.0 - ff_base,
         r"`ff\+fixed` \| 0\.\d+ \| (0\.\d+)"),
        ("H20-4 recurrent gain over headroom", rec_g / (1.0 - rec_base),
         r"`rec\+alif` \| 0\.\d+ \| 0\.\d+ \| \+0\.\d+ \| (0\.\d+)"),
        ("H20-4 feed-forward gain over headroom", ff_g / (1.0 - ff_base),
         r"`ff\+fixed` \| 0\.\d+ \| 0\.\d+ \| \+0\.\d+ \| (0\.\d+)"),
    ):
        results.append(check(label, computed, published(W20, pattern)))

    # The two adjacent-rung gaps the sweep cannot generate. Stated as a signed
    # difference of gains, which is why h384 - h512 is negative and is the one
    # that breaks H16-1.
    def rung(attn_root, attn_stem, rate_root, rate_stem):
        return paired_gain(acc_present(attn_root, attn_stem),
                           acc_present(rate_root, rate_stem))[0]
    g256 = rung(V2, f"w16lad__ff-fixed-attn__h256__e400__{ANCHOR}__d32l4",
                V2, f"w16lad__ff-fixed__h256__e400__{ANCHOR}")
    g384 = rung(V2, f"w16lad__ff-fixed-attn__h384__e400__{ANCHOR}__d32l4",
                V2, f"w16lad__ff-fixed__h384__e400__{ANCHOR}")
    g512 = rung(V2, f"w8wid__ff-fixed-attn__h512__e400__{ANCHOR}__d32l4",
                V1, f"w3wid__ff-fixed__h512__e400__{ANCHOR}")
    results.append(check("H16-1 gap h256 - h384", g256 - g384,
                         published(W1517, r"gaps: \+0\.\d+, \+(0\.\d+)"), tol=5e-4))
    results.append(check("H16-1 gap h384 - h512 (the negative one)", g384 - g512,
                         -published(W1517, r"gaps: \+0\.\d+, \+0\.\d+, \*\*−(0\.\d+)\*\*"),
                         tol=5e-4))

    # The spread behind "not distinguishable at n=12". A per-seed sd, which this
    # repository's sweep derives no variances for.
    a384 = acc_present(V2, f"w16lad__ff-fixed-attn__h384__e400__{ANCHOR}__d32l4")
    r384 = acc_present(V2, f"w16lad__ff-fixed__h384__e400__{ANCHOR}")
    a512 = acc_present(V2, f"w8wid__ff-fixed-attn__h512__e400__{ANCHOR}__d32l4")
    r512 = acc_present(V1, f"w3wid__ff-fixed__h512__e400__{ANCHOR}")
    common = sorted(set(a384) & set(r384) & set(a512) & set(r512))
    deltas = [(a384[s] - r384[s]) - (a512[s] - r512[s]) for s in common]
    mean = sum(deltas) / len(deltas)
    sd = (sum((d - mean) ** 2 for d in deltas) / len(deltas)) ** 0.5
    results.append(check("H16-1 sd of the seed-paired h384-h512 difference", sd,
                         published(W1517, r"sd\n(0\.\d+)\*\*|sd\s*\*?\*?(0\.\d+)"),
                         tol=5e-4))

    # The ladder is only a ladder if its rungs are ordered as published: the
    # gain decays across h128 -> h256 -> h512 and then collapses. Asserting the
    # shape, not just the four numbers, so that a rung moving without the prose
    # moving fails here rather than being noticed by a reader.
    ladder = []
    for attn_root, attn_stem, rate_root, rate_stem in (
        (V1, f"r1cal__ff-fixed-attn__h128__e400__{ANCHOR}__d32l4",
         V1, f"w1__ff-fixed__h128__e400__{ANCHOR}"),
        (AZ, f"az8wid__ff-fixed-attn__h256__e400__{ANCHOR}__d32l4",
         V1, f"w3wid__ff-fixed__h256__e400__{ANCHOR}"),
        (V2, f"w8wid__ff-fixed-attn__h512__e400__{ANCHOR}__d32l4",
         V1, f"w3wid__ff-fixed__h512__e400__{ANCHOR}"),
        (AZ, f"az8wid__ff-fixed-attn__h1024__e400__{ANCHOR}__d32l4",
         V1, f"w3wid__ff-fixed__h1024__e400__{ANCHOR}"),
    ):
        ladder.append(paired_gain(acc_present(attn_root, attn_stem),
                                  acc_present(rate_root, rate_stem))[0])
    monotone = all(a > b for a, b in zip(ladder, ladder[1:]))
    collapses_last = ladder[2] > 0 > ladder[3]
    shape_ok = monotone and collapses_last
    print(f"  [{'ok  ' if shape_ok else 'FAIL'}] "
          f"{'width ladder decays then collapses at h1024':<46} "
          f"computed {'/'.join(f'{g:+.4f}' for g in ladder)}")
    results.append(shape_ok)

    # ---- the paper draft must not resurrect a withdrawn claim ---------------
    print("\n  paper draft:")
    draft = (ROOT / "results/PAPER_DRAFT.md").read_text()
    forbidden = {
        "v130 PASS as a live claim": r"gap LCB `?0\.9988`?[^—]*PASS",
        "live-transfer 1.0000/0.9983 cited as a result": r"mean 1\.0000 / LCB 0\.9983",
        "the withdrawn RL broadcast contrasts as chance results":
            r"remains at chance \(0\.5120",
        "the pre-repair DFA and REINFORCE means as current":
            r"clear the matched gate \(`c1-dfa-c8c4fe0899908b84`",
    }
    for label, pattern in forbidden.items():
        hit = re.search(pattern, draft)
        ok = hit is None
        print(f"  [{'ok  ' if ok else 'FAIL'}] draft does not cite {label}")
        results.append(ok)

    # A withdrawn number may still be *named* — saying what was withdrawn is how
    # a reader knows. What must never happen is it appearing as a live claim. So
    # this is not a ban on the string: every sentence containing it has to be a
    # sentence that withdraws it. The blunt version of this check failed on the
    # withdrawal notice itself, which would have taught the next person to
    # delete the notice rather than the claim.
    withdrawn_numbers = {
        "the EventProp FAIL against SuperSpike 0.9150":
            r"0\.5000 (?:vs|against) SuperSpike \*?\*?0\.9150",
    }
    for label, pattern in withdrawn_numbers.items():
        unmarked = []
        for match in re.finditer(pattern, draft):
            start = draft.rfind(".", 0, max(match.start() - 400, 0)) + 1
            end = draft.find("\n", match.end())
            sentence = draft[start:end if end != -1 else len(draft)]
            if "withdraw" not in sentence.lower():
                unmarked.append(sentence.strip()[:70])
        ok = not unmarked
        print(f"  [{'ok  ' if ok else 'FAIL'}] draft cites {label} only as withdrawn")
        if unmarked:
            for line in unmarked:
                print(f"           live citation: {line}")
        results.append(ok)
    required = {
        "the A6 undertraining caveat": "The reference is undertrained, and the task saturates",
        "the saturation consequence": "learning speed",
        # The four load-bearing caveats on section 3.7. Each is the kind of
        # sentence an editing pass shortens away, and each is what stops the
        # recurrent result reading larger than it is.
        "that the read-out is not causal": "not causal",
        # S-5 was refuted and wave 10 replaced it. The draft must say both, or a
        # reader takes "resolution is refuted" for "resolution does not matter",
        # which is the opposite of what fixed-tN measured.
        "that S-5 is withdrawn rather than merely refuted": "refuted and is withdrawn",
        "the direction the gain moves with resolution": "shrinks with finer resolution",
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
    # A floor, because `66/66` and `65/65` both read as success. Deleting a
    # failing check is the cheapest way to make this script green, and until
    # 2026-08-28 nothing noticed: `results` is a list of booleans and its
    # LENGTH was never asserted.
    #
    # This is the third instance of the shape found today. `MIN_DOCUMENTS` in
    # check_every_number.py was declared and never read; `PAIRS` in
    # check_verdicts_transcribed.py was a curated list covering four of
    # fourteen wave results. Each printed a total that was true of what it
    # looked at and silent about what it did not.
    #
    # Raise this when checks are added. Lowering it is a decision to verify
    # less and should read like one in the diff.
    if len(results) < MIN_CHECKS:
        print(f"\nonly {len(results)} checks ran, below the floor of "
              f"{MIN_CHECKS}. A check was removed or stopped being reached; "
              f"verifying fewer numbers must not look like verifying them all.",
              file=sys.stderr)
        return 1
    return 1 if bad else 0


if __name__ == "__main__":
    raise SystemExit(main())
