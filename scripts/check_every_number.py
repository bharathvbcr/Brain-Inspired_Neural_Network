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
#: The manuscript. It was deliberately NOT swept until 2026-08-27, on the ground
#: that it draws on corpora this script does not load -- the hybrid lab, the
#: instrument calibration, and published literature -- so a cell-only sweep would
#: report 47 of its 119 numbers as unexplained when they are simply sourced
#: elsewhere. That was true and it was also the largest hole in the record: the
#: one artefact anyone outside this repository will read was the one artefact no
#: mechanical check touched. The note that replaced the check said the 47 had
#: been "traced on 2026-08-27" and each found in another document — by hand,
#: once, with the result written in prose and never re-run.
#:
#: The sweep below closes it at three NAMED tiers, and the tiers are printed
#: separately because they are not the same evidence:
#:
#:   A  derived from the cells, exactly as a wave result is
#:   B  named in ELSEWHERE, with the derivation stated there
#:   C  traced to one named primary record that still contains it
#:
#: Tier C is weaker than tier A and must never be reported as though it were.
#: What it establishes is that the value exists in a specific machine-written run
#: record, that the record still exists, and that it still carries the value —
#: so a paper number cannot drift from its source, and a source cannot be
#: deleted, without this failing. What it does NOT establish is that every
#: occurrence of that value in the paper refers to that record: the table is
#: keyed by value, and `0.5000` occurs eight times in the manuscript meaning
#: chance, an EventProp FAIL, and a broadcast arm.
PAPER = ROOT / "results/PAPER_DRAFT.md"

#: Documents that may not be cited as a source for a paper number, because they
#: are the paper's own downstream artefacts. Tracing `PAPER_DRAFT.md` to
#: `PAPER_RESULTS_TABLE.md` is not provenance; it is the same claim written
#: twice. Every entry in `PAPER_SOURCES` is checked against this list, so the
#: table cannot quietly acquire a circular citation.
PAPER_SIDE = frozenset({
    "PAPER_DRAFT.md", "PAPER_RESULTS_TABLE.md", "PAPER_SKELETON.md",
    "PAPER_METRICS_FULL.md", "PAPER_FIGURE_SPEC.md", "PAPER_VERIFY.md",
    "PAPER_STATUS_2026-08-20.md", "PUBLISHABLE_CLAIMS.md",
    "VENUE_FORMATTING.md", "INDEX.md",
})

#: A count that must not silently shrink. Both floors below are enforced in
#: `main`; `MIN_DOCUMENTS` was declared here on 2026-08-24 and never read, so a
#: narrowed glob would have swept fewer documents and still reported success —
#: the exact failure the constant was written to prevent.
MIN_DOCUMENTS = 14
MIN_PAPER_NUMBERS = 110

#: Values with a named source that a generator ALSO reaches, each with the
#: judgement a human made. A new overlap fails; a declared one is reported and
#: passes; a declaration whose overlap has gone fails as stale.
#:
#: Every entry here is a coincidence rather than a derivation, which is what a
#: 22%-dense `paired` generator predicts. An overlap that is NOT a coincidence
#: means the named entry has become unnecessary and should be deleted instead —
#: that happened to `0.9995` on 2026-08-28, a ratio across budgets the enlarged
#: corpus genuinely produces.
KNOWN_COINCIDENCE = {
    "0.9390": "a published 25-tap temporal-convolutional SHD result from "
              "another paper. No cell of this campaign can produce it, so the "
              "match is arithmetic and nothing else.",
    "0.6775": "a July C1 / Gate G2 local mean from the matched-architecture "
              "program, which does not run on the SHD instrument at all.",
    "0.2370": "a July C1 gap lower bound, same program, same reasoning.",
    "0.8267": "xor_thresh under DFA — a two-class XOR fixture in the "
              "matched-architecture program, which shares no task, no corpus "
              "and no arm with the SHD instrument. It collided with a per-seed "
              "accuracy the moment wave 20 landed, in `arm`, the sparsest "
              "generator at 6.8%.",
    "0.1411": "M-2, a difference OF gains. This sweep deliberately generates no "
              "second-order quantities, so a `pooled` match cannot be the same "
              "computation.",
}

#: Tier C. `(value, primary record, what the number is there)`.
#:
#: Chosen by reading the paper's own sentence for each value and finding the
#: record that produced it, not by taking whichever document happened to contain
#: a matching string: `0.0737` also appears in
#: MEASUREMENT_2026-08-03_RECURRENT_ARM_STABILITY.md as the accuracy
#: `0.073763251`, which has nothing to do with the gap lower bound the paper
#: quotes. An entry whose record no longer contains the value fails, an entry
#: naming a record that does not exist fails, and an entry whose value has left
#: the manuscript fails as stale — so this table cannot rot into a silencer.
PAPER_SOURCES = [
    # --- the July 2026 C1 / Gate G2 track (Section 3.3) ---------------------
    ("0.4912", "results/CAMPAIGN_2026-07-23_CLAIM_FREEZE.md",
     "canonical C1 local-assembly mean, hash c1-118207fbc3eaba53"),
    ("0.6775", "results/c1_sens_capacity.md",
     "capacity-sensitivity local mean, which clears the 0.65 floor"),
    ("0.4900", "results/CAMPAIGN_2026-07-23_CLAIM_FREEZE.md",
     "v13 live ReinforceFeedback local mean"),
    ("0.0737", "results/CAMPAIGN_2026-07-23_CLAIM_FREEZE.md",
     "v13 live ReinforceFeedback gap lower bound"),
    ("0.4838", "results/c1_rfb_em.md",
     "v14 epoch-matched live RFB local mean"),
    ("0.7262", "results/c1_sfb.md", "v15 structured-B local mean"),
    ("0.2567", "results/c1_sfb.md", "v15 structured-B gap lower bound"),
    ("0.6825", "results/c1_sfb_cap.md",
     "v17 structured-by-capacity local mean"),
    ("0.5025", "results/c1_sfb_soft.md", "v21 soft-WTA local mean"),
    ("0.7325", "results/c1_dfa_live.md", "v20 live-DFA local mean"),
    ("0.2601", "results/c1_dfa_live.md", "v20 live-DFA gap lower bound"),
    ("0.3321", "results/c1_dfa_live.md",
     "v20 descriptive chance-normalised gap lower bound, explicitly not a gate"),
    ("0.6638", "results/c1_sfb_finth.md", "v23 finite-theta local mean"),
    ("0.2370", "results/c1_sfb_finth.md", "v23 finite-theta gap lower bound"),
    ("0.6437", "results/c1_sfb_cont.md", "v24 continuous-B local mean"),
    ("0.1380", "results/c1_sfb_cont.md", "v24 continuous-B gap lower bound"),
    ("0.6513", "results/MATCHED_ARCH_DFA_SPIKE_CONTROL.md",
     "P4 spiking-path true-DFA primary arm"),
    # --- the XOR locality flip (Section 3.4) --------------------------------
    ("0.5008", "results/CAMPAIGN_2026-07-23_CLAIM_FREEZE.md",
     "xor_thresh under broadcast error: at chance"),
    ("0.8267", "results/CAMPAIGN_2026-07-23_CLAIM_FREEZE.md",
     "xor_thresh under DFA"),
    ("0.7733", "results/CAMPAIGN_2026-07-23_CLAIM_FREEZE.md",
     "xor_thresh under gradient, the arm DFA passes"),
    # --- the 2026-08-25 matched re-run, which the paper names as the source
    #     of its matched table -----------------------------------------------
    ("0.5000", "results/matched_rerun_2026-08-25/c1_match_feedforward.md",
     "broadcast +-1 three-factor on the feed-forward graph: the lead negative"),
    ("0.5100", "results/matched_rerun_2026-08-25/c1_match_recurrent.md",
     "broadcast +-1 three-factor on the recurrent graph"),
    ("0.0192", "results/matched_rerun_2026-08-25/c1_match_recurrent.md",
     "the lead negative's gap lower bound on the recurrent graph, -0.0192"),
    ("0.7775", "results/matched_rerun_2026-08-25/c1_matched-rl_feedforward.md",
     "RL +-1 broadcast contrast, feed-forward"),
    ("0.8787", "results/matched_rerun_2026-08-25/c1_matched-rl_feedforward.md",
     "RL graded-reward broadcast contrast, feed-forward"),
    ("0.9950", "results/matched_rerun_2026-08-25/c1_matched-rl_feedforward.md",
     "REINFORCE x frozen B_i, the primary gated arm, feed-forward"),
    ("0.9100", "results/matched_rerun_2026-08-25/c1_matched-rl_recurrent.md",
     "RL graded-reward broadcast contrast, recurrent"),
    ("0.9925", "results/matched_rerun_2026-08-25/c1_matched-dfa_feedforward.md",
     "graded error x DFA, feed-forward"),
    ("0.9975", "results/matched_rerun_2026-08-25/c1_matched-dfa_feedforward.md",
     "graded error broadcast: the contrast the honesty note is about"),
    ("0.9875", "results/matched_rerun_2026-08-25/c1_matched-dfa_recurrent.md",
     "graded error x DFA, recurrent"),
    ("0.9150", "results/c1_eventprop.md",
     "SuperSpike BPTT ceiling on the EventProp fixture"),
    # --- the archived A6 ceiling-health sweep (Section 3.6 and the caveat) ---
    ("0.9387", "results/a6_ceiling_health_2026-08-19/a6_report.md",
     "the archived MatchedDfa arm"),
    ("0.8963", "results/a6_ceiling_health_2026-08-19/a6_report.md",
     "the published canonical reference the gap-closed ratio is taken against"),
    ("0.9013", "results/a6_ceiling_health_2026-08-19/a6_report.md",
     "the reference re-run at the canonical e80/lr0.05"),
    ("0.9700", "results/a6_ceiling_health_2026-08-19/a6_report.md",
     "the reference at e320/lr0.05, the best budget swept"),
    ("0.9863", "results/a6_ceiling_health_2026-08-19/matched_80.md",
     "MatchedBroadcastErr, the control that sits above the arm under test"),
    ("0.5113", "results/a6_ceiling_health_2026-08-19/matched_80.md",
     "the archived MatchedRlFlat +-1 baseline"),
    ("0.5250", "results/a6_ceiling_health_2026-08-19/matched_80.md",
     "the archived MatchedRlGraded contrast"),
    # --- elsewhere ----------------------------------------------------------
    ("0.9988", "results/RESULT_2026-08-19_TRACK_B_V130_PASS_WITHDRAWN.md",
     "the withdrawn track-b-rescue v130 gap lower bound"),
    ("0.9390", "results/FINDING_2026-08-24_THE_FORWARD_PASSES_DIFFER_IN_KIND.md",
     "the published 25-tap temporal-convolutional SHD reference: literature, "
     "not a cell of this campaign"),
]

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
    # `0.9995` — attn(e20)/attn(e400) — was here until 2026-08-28 and was
    # deleted because the sweep declared it stale, which is the entry doing its
    # job in the direction nobody plans for. Collecting wave 18's hundred cells
    # and wave 21's first sixty enlarged the comparable-pair set enough that the
    # ratio now falls out of the `pooled` generator directly. The number did not
    # change and W6 still quotes it; it stopped needing an exception, so the
    # exception had to go. An ELSEWHERE list that only ever grows is a list that
    # has stopped meaning "the cells cannot produce this".
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


#: The generators, strongest-first, as they are reported. A number is credited to
#: the FIRST one that can produce it.
#:
#: They are kept apart because they are not equally good evidence, and the single
#: number this script used to print hid that. `arm` is one configuration's own
#: mean, extremes, headroom or a per-seed value: 679 quantities, so a random 4dp
#: value matches one 6.8% of the time. `paired` is anything computed over two
#: comparable arms' shared seeds: 2,720 quantities and 27.2%, which is where
#: essentially all of the global density lives. `pooled` is a cross-arm
#: difference or ratio over each arm's own seeds: 287 quantities and 2.9%.
#:
#: A document whose numbers rest on `arm` has been checked against a sparse set.
#: One whose numbers rest on `paired` has been checked against a set that would
#: accept one number in four by accident. Same word — "derivable" — and very
#: different evidence, so the run now says which.
TIERS = ("arm", "paired", "pooled")


def derivable(groups) -> dict[str, set[float]]:
    """What the cells can produce, split by generator. Absolute, 4dp.

    Was a single set, and `main` reported one coincidence rate over it. That
    rate reached 31% as the corpora grew to 97 configurations, at which point
    "every number follows from the cells" was about three times better than
    chance and read like a proof. Splitting the set does not make the check
    stronger; it makes the run say where its strength actually is.
    """
    out: dict[str, set[float]] = {tier: set() for tier in TIERS}
    tier = "arm"

    def add(value: float) -> None:
        if isinstance(value, float) and value == value:
            out[tier].add(round(abs(value), 4))

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
    tier = "paired"
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
        tier = "pooled"
        add(left_mean - right_mean)                                       # pooled
        if right_mean:
            add(left_mean / right_mean)                                   # ratio
        tier = "paired"

    # Second-order quantities — differences OF gains, which is what every
    # two-sided hypothesis in this campaign is — are deliberately NOT generated
    # here. Adding all pairs of gains took the derived set from 921 to 7,220 and
    # the coincidence rate from 8.6% to 67.7%: at that density the sweep
    # "explains" two numbers in three by accident and stops being evidence. They
    # are verified individually in `verify_published_numbers.py`, which names
    # each derivation instead of guessing it, and are listed in ELSEWHERE with a
    # pointer to that check.
    #
    # Credit each value once, to the strongest generator that reaches it. Without
    # this the tiers overlap and the per-tier counts below would sum to more than
    # the numbers actually checked.
    seen: set[float] = set()
    for name in TIERS:
        out[name] -= seen
        seen |= out[name]
    return out


def explain(value: float, tiers: dict[str, set[float]]) -> str | None:
    """The strongest generator that reaches `value`, or None."""
    for name in TIERS:
        if any(abs(value - k) <= TOL for k in tiers[name]):
            return name
    return None


def sweep_paper(tiers: dict[str, set[float]], allowed: set[str]) -> tuple[dict[str, int], int, int, list[str], list[str]]:
    """Sweep `PAPER_DRAFT.md` at three named tiers.

    Returns `(cells, elsewhere, traced, unexplained, complaints)`, where `cells`
    is keyed by the generator that reached each number. The tiers are returned
    separately rather than summed because a paper number backed by a named run
    record is not the same evidence as one recomputed from cells, and a number
    reached only by a paired statistic is not the same evidence as one that is a
    configuration's own mean. A single "checked" count would erase both.
    """
    complaints: list[str] = []
    empty = {tier: 0 for tier in TIERS}
    if not PAPER.is_file():
        return empty, 0, 0, [], [f"{PAPER} is missing; the manuscript sweep did not run"]

    text = PAPER.read_text()
    sources: dict[str, tuple[str, str]] = {}
    for value, relpath, what in PAPER_SOURCES:
        if value in sources:
            complaints.append(f"PAPER_SOURCES names {value} twice")
        sources[value] = (relpath, what)
        doc = ROOT / relpath
        if Path(relpath).name in PAPER_SIDE:
            complaints.append(
                f"{value} is traced to {relpath}, which is one of the paper's "
                f"own artefacts; that is the claim written twice, not provenance")
            continue
        if not doc.is_file():
            complaints.append(f"{value} is traced to {relpath}, which does not exist")
            continue
        if value not in doc.read_text():
            complaints.append(
                f"{value} is traced to {relpath}, which no longer contains it "
                f"({what}) — the paper and its source have drifted apart")

    cells = dict(empty)
    elsewhere = traced = 0
    coincident: list[tuple[str, str, str]] = []
    unexplained: list[str] = []
    quoted: set[str] = set()
    for raw in sorted({m.group(1) for m in NUMBER.finditer(text)}):
        value = round(abs(float(raw.replace("−", "-").replace("+", ""))), 4)
        plain = f"{value:.4f}"
        quoted.add(plain)
        # A NAMED source wins over a derivation, and the order matters.
        #
        # Crediting the cells first silently relabels provenance. On 2026-08-28,
        # after 242 cells were collected, three paper numbers with explicit
        # sources became cell-derivable and were reported as "derived from the
        # cells": `0.6775` and `0.2370` from the July C1 track, and — decisively
        # — `0.9390`, a **published 25-tap temporal-convolutional SHD result
        # from another paper**, which cannot be derived from this campaign at
        # all. At 22% density in the `paired` generator, coincidences of this
        # kind are expected rather than surprising.
        #
        # So an entry a human wrote and checked outranks a numerical match this
        # script found. The overlaps are still reported below, because an entry
        # that has become genuinely derivable should be retired and only a human
        # can tell that from a coincidence.
        generator = explain(value, tiers)
        if plain in allowed:
            elsewhere += 1
            if generator:
                coincident.append((plain, "ELSEWHERE", generator))
        elif plain in sources:
            traced += 1
            if generator:
                coincident.append((plain, sources[plain][0], generator))
        elif generator:
            cells[generator] += 1
        else:
            unexplained.append(raw)

    if len(quoted) < MIN_PAPER_NUMBERS:
        complaints.append(
            f"only {len(quoted)} distinct numbers found in {PAPER.name}, below "
            f"the floor of {MIN_PAPER_NUMBERS}; the pattern or the file has "
            f"changed and this sweep is now covering less than it claims")
    overlapping = {plain for plain, _, _ in coincident}
    for plain, source, generator in sorted(coincident):
        if plain in KNOWN_COINCIDENCE:
            continue
        complaints.append(
            f"{plain} has a named source ({source}) AND is reachable by the "
            f"`{generator}` generator, and no judgement is recorded. It is "
            f"credited to the named source. If the derivation is genuine, "
            f"retire the entry; if it is a coincidence, declare it in "
            f"KNOWN_COINCIDENCE with the reason. A human decides which.")
    for plain in sorted(set(KNOWN_COINCIDENCE) - overlapping):
        complaints.append(
            f"KNOWN_COINCIDENCE still declares {plain}, which no generator "
            f"reaches any more. The judgement it records is about a collision "
            f"that has gone; delete it.")
    for value in sorted(set(sources) - quoted):
        complaints.append(
            f"PAPER_SOURCES still names {value} ({sources[value][0]}), which is "
            f"no longer quoted in the manuscript; delete the entry")
    return cells, elsewhere, traced, unexplained, complaints


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
    if len(DOCUMENTS) < MIN_DOCUMENTS:
        print(f"{len(DOCUMENTS)} wave-result documents matched, below the floor "
              f"of {MIN_DOCUMENTS}; the glob has narrowed or the directory has "
              f"moved", file=sys.stderr)
        return 1

    tiers = derivable(groups)
    known = set().union(*tiers.values())
    allowed = {value for value, _ in ELSEWHERE}
    unexplained: list[tuple[str, str]] = []
    seen_allowed: set[str] = set()
    checked = 0
    silent: list[str] = []

    print(f"{len(known)} distinct quantities derivable from {len(groups)} "
          f"configurations, by generator:")
    for name in TIERS:
        print(f"  {name:<8} {len(tiers[name]):>5} quantities   a random 4dp value "
              f"in [0,1] matches one {100 * report_power(tiers[name]):.1f}% of "
              f"the time")
    print(f"  tolerance {TOL}. A number reached only by `paired` has been checked "
          f"against a set\n  that would accept one value in four by accident; say "
          f"so rather than calling both\n  outcomes derivable and leaving it "
          f"there.\n")
    for doc in DOCUMENTS:
        if doc.name in UNCHECKED:
            print(f"  [----] {doc.name[:64]:<64} UNCHECKED: {UNCHECKED[doc.name]}")
            continue
        numbers = sorted({m.group(1) for m in NUMBER.finditer(doc.read_text())})
        bad = []
        by_tier = {name: 0 for name in TIERS}
        for text in numbers:
            checked += 1
            value = round(abs(float(text.replace("−", "-").replace("+", ""))), 4)
            generator = explain(value, tiers)
            if generator:
                by_tier[generator] += 1
                continue
            plain = f"{value:.4f}"
            if plain in allowed:
                seen_allowed.add(plain)
                continue
            bad.append(text)
        if bad:
            mark, status = "FAIL", f"UNEXPLAINED: {', '.join(bad)}"
        elif not numbers:
            # `RESULT_2026-08-20_W4_RECURRENT_ARM_IS_UNUSABLE.md` is 93 lines
            # long and quotes no four-decimal number at all. It printed `ok` for
            # as long as this sweep has existed -- the same word as a document
            # whose forty numbers were each recomputed. A sweep that finds
            # nothing to check has not checked anything.
            mark, status = "none", "NOTHING TO CHECK: no four-decimal number here"
            silent.append(doc.name)
        else:
            mark = "ok  "
            status = "  ".join(f"{name} {by_tier[name]}" for name in TIERS
                               if by_tier[name])
        print(f"  [{mark}] {doc.name[:64]:<64} {status}")
        unexplained += [(doc.name, b) for b in bad]

    paper_cells, paper_elsewhere, paper_traced, paper_bad, paper_complaints = \
        sweep_paper(tiers, allowed)
    # ELSEWHERE entries cited only by the manuscript are not stale. Before the
    # paper was swept they could not be seen at all, so staleness was measured
    # against a corpus that excluded one of the two readers.
    for raw in {m.group(1) for m in NUMBER.finditer(PAPER.read_text())} if PAPER.is_file() else set():
        plain = f"{round(abs(float(raw.replace(chr(8722), '-').replace('+', ''))), 4):.4f}"
        if plain in allowed:
            seen_allowed.add(plain)

    stale = [v for v, _ in ELSEWHERE if v not in seen_allowed]
    # A named exclusion that no longer exists is an exclusion hiding nothing,
    # and would let a document be dropped by a rename rather than by a decision.
    missing = [n for n in UNCHECKED if not (ROOT / "results" / n).is_file()]
    if missing:
        print(f"STALE entries in UNCHECKED — these documents are gone: {missing}")
    print()
    paper_from_cells = sum(paper_cells.values())
    total = paper_from_cells + paper_elsewhere + paper_traced + len(paper_bad)
    print(f"  [{'ok  ' if not (paper_bad or paper_complaints) else 'FAIL'}] "
          f"{PAPER.name[:64]:<64} {total} numbers")
    print(f"         tier A, derived from the cells      {paper_from_cells}"
          f"   ({', '.join(f'{n} {paper_cells[n]}' for n in TIERS if paper_cells[n])})")
    print(f"         tier B, named in ELSEWHERE          {paper_elsewhere}")
    print(f"         tier C, traced to a named record    {paper_traced}")
    print(f"  tier C is NOT derivation. It establishes that the value is still "
          f"present in one named,\n  machine-written run record and that the "
          f"record still exists — not that every occurrence\n  in the manuscript "
          f"refers to it. Read it as weaker than tier A, because it is.")
    for complaint in paper_complaints:
        print(f"  PROVENANCE: {complaint}")
    if KNOWN_COINCIDENCE:
        print(f"  {len(KNOWN_COINCIDENCE)} value(s) have a named source that a "
              f"generator also reaches by coincidence; each is declared with "
              f"its reason and credited to the source, not the generator.")
    if paper_bad:
        print(f"  {len(paper_bad)} number(s) in {PAPER.name} with no tier at all: "
              f"{', '.join(paper_bad)}")
    swept = len(DOCUMENTS) - len(UNCHECKED) - len(silent)
    if silent:
        print(f"\n{len(silent)} document(s) carry no four-decimal number and so "
              f"were not checked by this sweep,\nand are excluded from the claim "
              f"below: {', '.join(silent)}")
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
    if unexplained or stale or missing or paper_bad or paper_complaints:
        return 1
    # The claim names what was actually swept. It used to say "every wave
    # result" while a date window silently held one back.
    print(f"every number in the {swept} swept wave results follows from the cells, "
          f"and every\nnumber in {PAPER.name} is derived from them ({paper_from_cells}), "
          f"named in ELSEWHERE ({paper_elsewhere}),\nor traced to a named primary "
          f"record ({paper_traced})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
