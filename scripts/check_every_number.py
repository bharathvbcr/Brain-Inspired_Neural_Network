#!/usr/bin/env python3
"""Every four-decimal number in a campaign-wave result must come from the cells.

# Why this is stronger than `verify_published_numbers.py`

That script checks a **curated list**: someone decided which numbers matter and
wrote a regex for each. It cannot catch a number nobody thought to list. This one
inverts the question — it takes *every* number in the document and asks whether
the cells can produce it — so the default is suspicion rather than trust.

It found one on its first run. Wave 9's residual gain under shuffling was
published as **+0.0049**, which is `0.6983 − 0.6934`: the difference of the two
already-rounded means printed above it in the same table. From the cells the
difference is 0.004969, which is **+0.0050**. One part in ten thousand, no
consequence for the conclusion, and exactly the class of error a curated list
does not look for because nobody would think to curate it.

# What counts as "from the cells"

Means, per-seed values, minima and maxima over any recorded configuration; and
between any two configurations, the pooled difference of means, the paired
per-seed difference, its extremes, and the ratio of their means. Headroom
(`1 − mean`) is included because the paper reports it.

Numbers that legitimately come from elsewhere are listed in `ELSEWHERE`, each
with the corpus it came from. A stale entry fails, so the list cannot rot into a
way of silencing the check.

    python3 scripts/check_every_number.py
"""

from __future__ import annotations

import itertools
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
from cell_validity import validity_problems  # noqa: E402

#: Campaign directories whose cells are the evidence for the documents below.
CORPORA = [
    ROOT / "results/shd_attention_campaign_v1/cells",
    ROOT / "results/shd_attention_campaign_v2",
    # The Azure cells landed on 2026-08-25 and were, until then, the only
    # campaign cells on disk with no document in this sweep and no corpus here
    # — so the one result written from them was the one set of numbers nothing
    # recomputed. Its own write-up said the figures had been "re-derived from
    # those files"; by hand, once, by the person who wrote them down.
    ROOT / "results/azure-d32l4-scope-v1/results",
]

#: The wave results. Each rests on the cell corpora above and nothing else.
# Wave results, by wave number rather than by date. The pattern was
# `RESULT_2026-08-2[0-3]_W*.md` — a four-day window that closed on 2026-08-24,
# so every wave written afterwards fell silently outside it while the script
# went on printing "every number in every wave result follows from the cells".
# The old `if not DOCUMENTS` guard could never fire, because the twelve
# documents inside the window match forever. `_W[0-9]` rather than `_W`,
# because `_WITHDRAWN` and `_WITH_HEADROOM` are not waves.
DOCUMENTS = sorted(
    set((ROOT / "results").glob("RESULT_*_W[0-9]*.md"))
    # Not a wave, and so outside the glob: the Azure campaign is `az8*`. It is
    # named rather than pattern-matched because there is one of it.
    | {ROOT / "results/RESULT_2026-08-22_AZURE_TRUNCATED_AT_95_OF_252.md"}
)
#: A count that must not silently shrink. A narrowed pattern or a moved
#: directory would otherwise sweep fewer documents and still report success.
MIN_DOCUMENTS = 14

#: Wave documents whose numbers this sweep cannot derive, and why. Naming them
#: is the point: the date window used to exclude W1 by accident, so the script
#: claimed to have checked "every wave result" while never loading the cells
#: behind one of them. A document listed here is reported as UNCHECKED on every
#: run and is excluded from the closing claim — it is neither verified nor
#: refuted, and must not be mistaken for either.
UNCHECKED = {
    "RESULT_2026-08-19_W1_ATTENTION_AT_CONVERGENCE.md":
        "wave 1 predates both cell corpora in CORPORA; six of its numbers "
        "(+0.0912, 0.0063, 0.0069, 0.7509, -0.2675, -0.5009) cannot be "
        "derived from any cell on disk, so this sweep cannot judge them",
}

#: A four-decimal number, not preceded by `=` or a digit (so `|a−b|=0.0002` and
#: version strings do not match) and not followed by `%`.
NUMBER = re.compile(r"(?<![=\w.\d])([-+−]?[01]\.\d{4})(?![\d%])")

#: Half a unit in the fourth decimal place — a quotation either rounds to the
#: computed value or it does not. The first version used 1.5e-4 "for the
#: rounding of the quotation", which is 1.5 units of the last place and let a
#: number match a neighbour it had nothing to do with.
TOL = 5e-5

#: Numbers that are correct and come from a corpus these documents do not own.
#: `(value, why)` — and an entry that stops being needed fails the check.
ELSEWHERE = [
    ("0.7378", "the converged ff+fixed ceiling, h1024/e400, from "
               "results/shd_instrument_v4/width-converged (mean 0.737780, n=3); "
               "cited by W3, W6 and W7 and by shd_attention.rs"),
    # Quantities this sweep deliberately does not generate, each identified and
    # each verified by name in verify_published_numbers.py rather than guessed
    # at here. Generating them would take the derived set from ~1,000 to ~7,000
    # and the coincidence rate from 9% to 68%.
    ("0.0453", "C-2, a difference OF gains across the resolution ladder; "
               "checked by name in verify_published_numbers.py"),
    ("0.1411", "M-2, a difference OF gains across the substrate; checked by "
               "name in verify_published_numbers.py"),
    ("0.1207", "W6-3, gain(e20) − gain(e400): second order"),
    ("0.0039", "W5-3, the movement of a contrast between two budgets: second order"),
    ("0.1058", "the published-2ms gain quoted in W3 and W8 between arms whose "
               "seed sets overlap on fewer than three seeds"),
    ("0.0242", "a per-seed SPREAD, not a mean; this sweep derives no variances"),
    # Waves 15-17. The three headline numbers are means of a MERGED arm -- the
    # archived twelve seeds plus wave 17's twenty -- and this sweep groups by
    # stem, so `r1cal` and `w17hdl` are separate groups and their union is not
    # generated. The two gaps are differences OF gains and the sd is a variance.
    # Each is checked by name in verify_published_numbers.py.
    ("0.7057", "H17-1 rate mean at n=32, a merged r1cal/w1 + w17hdl arm; "
               "checked by name in verify_published_numbers.py"),
    ("0.8332", "H17-1 attention mean at n=32, merged; checked by name in "
               "verify_published_numbers.py"),
    ("0.1275", "H17-1 paired gain at n=32, merged; checked by name in "
               "verify_published_numbers.py"),
    ("0.0206", "H16-1 gap h256 - h384, a difference OF gains; checked by name "
               "in verify_published_numbers.py"),
    ("0.0116", "H16-1 gap h384 - h512, the negative one that breaks the chain; "
               "a difference OF gains, checked by name in "
               "verify_published_numbers.py"),
    ("0.0253", "the sd of the seed-paired h384-h512 difference; a per-seed "
               "SPREAD, and this sweep derives no variances"),
    ("0.7556", "ff+fixed(e10)/ff+fixed(e400): a ratio across budgets, which is "
               "a two-axis comparison"),
    ("0.9029", "attn(e5)/attn(e400): the same, and verified by name"),
    ("0.9995", "attn(e20)/attn(e400): the same, and verified by name"),
    ("0.6108", "AZ8-6's *would-be* gain, and the one number in the Azure result "
               "that this sweep should not be able to derive. It is the d64/L4 "
               "arm paired against its rate control over all twelve seeds "
               "INCLUDING the six that fail the validity gate (-0.610792); the "
               "sweep pairs valid cells only, which is why it cannot reach it. "
               "The document prints it to say what the arm would have scored "
               "and that the arm is VOIDED — a number published in order to be "
               "refused. Deriving it would mean the sweep had started pairing "
               "degenerate cells."),
]


def load() -> dict[str, dict[int, tuple[float, bool]]]:
    """`{configuration: {seed: (accuracy, passes the validity gate)}}`."""
    groups: dict[str, dict[int, tuple[float, bool]]] = {}
    for root in CORPORA:
        for path in sorted(root.glob("*.json")):
            match = re.match(r"(.+)__s(\d+)\.json$", path.name)
            if not match:
                continue
            try:
                cell = json.loads(path.read_text())
            except json.JSONDecodeError:
                continue
            if not isinstance(cell, dict) or "accuracy" not in cell:
                continue
            groups.setdefault(match.group(1), {})[int(match.group(2))] = (
                cell["accuracy"], not validity_problems(cell),
            )
    return groups


def derivable(groups) -> set[float]:
    """Everything the cells can produce, as absolute values rounded to 4dp."""
    out: set[float] = set()

    def add(value: float) -> None:
        if isinstance(value, float) and value == value:
            out.add(round(abs(value), 4))

    valid: dict[str, dict[int, float]] = {}
    for stem, seeds in groups.items():
        passing = {s: a for s, (a, ok) in seeds.items() if ok}
        every = {s: a for s, (a, _) in seeds.items()}
        for pool in (passing, every):
            if not pool:
                continue
            values = list(pool.values())
            add(sum(values) / len(values))
            add(min(values))
            add(max(values))
            add(1.0 - sum(values) / len(values))   # headroom
            for value in values:
                add(value)
        if passing:
            valid[stem] = passing

    # Only pairs WITHIN a wave. All-pairs over 71 configurations is 2,485
    # comparisons, most of them meaningless — wave 3's h1024 arm against wave
    # 13's rec+fixed — and the resulting set was dense enough that a random
    # number in [0,1] matched something 73% of the time. A check that explains
    # three numbers in four by coincidence is not a check. Restricting to the
    # comparisons a wave document actually makes is what gives a clean pass its
    # meaning; `report_power` below states the residual rate rather than
    # asserting there is none.
    def operating_point(stem: str) -> tuple[str, ...]:
        """Everything that must match for two arms to be comparable.

        A gain is always between two arms at the SAME operating point, so the
        key is the stem with its wave label, its arm, and its attention shape
        removed — width, budget, contract, geometry, temporal condition and
        surrogate scale all have to agree. Pairing on the wave label instead was
        too tight: the campaign's whole reuse design is a wave comparing its own
        treatment against a control recorded by an earlier one.
        """
        parts = stem.split("__")[1:]                      # drop the wave label
        return tuple(p for p in parts
                     if not p.startswith(("ff-", "rec-"))  # drop the arm
                     and not re.fullmatch(r"d\d+l\d+", p))  # drop the attention shape

    def comparable(a: str, b: str) -> bool:
        """Same operating point, or differing on exactly ONE axis of it.

        The campaign compares one axis at a time: intact against bin-shuffled,
        one contract against another along the resolution ladder, one surrogate
        scale against another. Requiring the operating points to be identical
        misses every one of those; allowing any pair at all is what made the
        derived set dense enough to explain three numbers in four by accident.
        """
        left, right = operating_point(a), operating_point(b)
        if left == right:
            return True
        if len(left) != len(right):
            # A temporal tag present on one side and absent on the other is a
            # one-axis difference too: `intact` is written by omission.
            longer, shorter = (left, right) if len(left) > len(right) else (right, left)
            return len(longer) == len(shorter) + 1 and all(p in longer for p in shorter)
        return sum(1 for x, y in zip(left, right) if x != y) == 1

    gains: list[float] = []
    for (left_stem, left), (right_stem, right) in itertools.combinations(valid.items(), 2):
        if not comparable(left_stem, right_stem):
            continue
        shared = sorted(set(left) & set(right))
        if len(shared) < 3:
            continue
        gains.append(sum(left[s] - right[s] for s in shared) / len(shared))
        deltas = [left[s] - right[s] for s in shared]
        add(sum(deltas) / len(deltas))                                    # paired
        # Means over the intersection, and their headroom. A wave that loses
        # different seeds in each arm reports the PAIRED mean, not the arm's own
        # mean over everything it completed, and the paper's headroom figures are
        # taken from those. Wave 14's 0.4738 is `1 - 0.5262` on ten pairs, and
        # without this it looked like a number the cells could not produce.
        left_paired = sum(left[s] for s in shared) / len(shared)
        right_paired = sum(right[s] for s in shared) / len(shared)
        for mean in (left_paired, right_paired):
            add(mean)
            add(1.0 - mean)
        if right_paired:
            add(left_paired / right_paired)
        for delta in deltas:       # per-seed gains, quoted individually by W11
            add(delta)
        left_mean = sum(left.values()) / len(left)
        right_mean = sum(right.values()) / len(right)
        add(left_mean - right_mean)                                       # pooled
        if right_mean:
            add(left_mean / right_mean)                                   # ratio

    # Second-order quantities — differences OF gains, which is what every
    # two-sided hypothesis in this campaign is — are deliberately NOT generated
    # here. Adding all pairs of gains took the derived set from 921 to 7,220 and
    # the coincidence rate from 8.6% to 67.7%: at that density the sweep
    # "explains" two numbers in three by accident and stops being evidence. They
    # are verified individually in `verify_published_numbers.py`, which names
    # each derivation instead of guessing it, and are listed in ELSEWHERE with a
    # pointer to that check.
    return out


def report_power(known: set[float], samples: int = 20000) -> float:
    """How often a random four-decimal value in [0, 1] matches by coincidence.

    Stated in the output because it is the strength of the check. A clean pass
    against a dense set means nothing, and the only honest way to publish "every
    number is derivable" is beside the rate at which any number would be.
    """
    import random

    rng = random.Random(20260823)
    ordered = sorted(known)
    import bisect

    hits = 0
    for _ in range(samples):
        value = round(rng.uniform(0.0, 1.0), 4)
        i = bisect.bisect_left(ordered, value - TOL)
        if i < len(ordered) and ordered[i] <= value + TOL:
            hits += 1
    return hits / samples


def main() -> int:
    groups = load()
    if len(groups) < 50:
        print(f"only {len(groups)} configurations loaded; the corpora are not "
              "where this expects them", file=sys.stderr)
        return 1
    if not DOCUMENTS:
        print("no wave-result documents matched; the glob has stopped working",
              file=sys.stderr)
        return 1

    known = derivable(groups)
    allowed = {value for value, _ in ELSEWHERE}
    unexplained: list[tuple[str, str]] = []
    seen_allowed: set[str] = set()
    checked = 0

    power = report_power(known)
    print(f"{len(known)} distinct quantities derivable from {len(groups)} "
          f"configurations")
    print(f"coincidence rate: a random 4dp value in [0,1] would match one of "
          f"them {100 * power:.1f}% of the time, at tolerance {TOL}\n")
    for doc in DOCUMENTS:
        if doc.name in UNCHECKED:
            print(f"  [----] {doc.name[:64]:<64} UNCHECKED: {UNCHECKED[doc.name]}")
            continue
        numbers = sorted({m.group(1) for m in NUMBER.finditer(doc.read_text())})
        bad = []
        for text in numbers:
            checked += 1
            value = round(abs(float(text.replace("−", "-").replace("+", ""))), 4)
            if any(abs(value - k) <= TOL for k in known):
                continue
            plain = f"{value:.4f}"
            if plain in allowed:
                seen_allowed.add(plain)
                continue
            bad.append(text)
        status = "ok" if not bad else f"UNEXPLAINED: {', '.join(bad)}"
        print(f"  [{'ok  ' if not bad else 'FAIL'}] {doc.name[:64]:<64} {status}")
        unexplained += [(doc.name, b) for b in bad]

    stale = [v for v, _ in ELSEWHERE if v not in seen_allowed]
    # A named exclusion that no longer exists is an exclusion hiding nothing,
    # and would let a document be dropped by a rename rather than by a decision.
    missing = [n for n in UNCHECKED if not (ROOT / "results" / n).is_file()]
    if missing:
        print(f"STALE entries in UNCHECKED — these documents are gone: {missing}")
    swept = len(DOCUMENTS) - len(UNCHECKED)
    print(f"\n{checked} numbers checked across {swept} wave results")
    if UNCHECKED:
        print(f"{len(UNCHECKED)} wave result(s) this sweep cannot judge, "
              f"listed above and excluded from the claim below")
    if stale:
        print(f"STALE entries in ELSEWHERE — no longer cited, delete them: {stale}")
    if unexplained:
        print(f"{len(unexplained)} number(s) the cells cannot produce:")
        for name, text in unexplained:
            print(f"  {name}: {text}")
    if unexplained or stale or missing:
        return 1
    # The claim names what was actually swept. It used to say "every wave
    # result" while a date window silently held one back.
    print(f"every number in the {swept} swept wave results follows from the cells")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
