#!/usr/bin/env python3
"""Paired bootstrap intervals on the gain and the difference-in-differences.

Every operating point `mechanism_coverage.py` lists carries a mean DiD and a
count of positive seeds, and nothing else. A mean over twelve seeds with no
interval cannot say whether "the intact gain is +0.004" means "there is no gain
to lose" or "the gain is noisy", and the paper's reading of the wide rungs turns
on exactly that distinction. This reports, per point and per training budget:

    n                seed quadruples (all four arms present, all cells valid)
    gain intact      attention intact  - rate intact
    gain shuffled    attention shuffled - rate shuffled
    DiD              gain intact - gain shuffled
                   = (attn intact - attn shuffled) - (rate intact - rate shuffled)

each with a paired percentile bootstrap 95% interval. All three statistics are
computed over the SAME quadruples, so DiD = gain intact - gain shuffled holds
exactly per row.

# Nothing here re-derives an arm

Each row is read through the frozen analyser that owns it, imported rather than
copied, so this file cannot disagree with a published verdict about which cell
belongs to which arm:

    e400  d32l4 on the W21 points     analyse_wave21.index   (corpus; the anchor
                                                              is the n=32 row of
                                                              waves 15-17)
    e400  the twelve W22 points       analyse_wave22.index   (w22cov only)
    e400  h128 / published-2ms / d32l1 analyse_wave8.load    (w1, hash-verified)
    e100  the nineteen H25-4 points   analyse_wave25.index + H25-4's wave choice
    e100  h128 d32l2, h1024 d32l1     analyse_wave24.index + its SOURCE table
    e400  anchor, hidden-shuffled     analyse_wave27.index   (w27hid vs w26pos)

Deliberately NOT rows: the `w25mec` e400 d32l4 twins (H25-3) and the `w26pos`
bin-shuffled anchor. Their accuracies are bit-identical to the corpus cells at
the same seeds, so they are the same measurement re-run, not a replicate, and a
row for them would present one sample as two.

The row order and the point set come from `mechanism_coverage.coverage`. A point
it lists that no source can pair is printed as an explicit unpaired row with the
reason; it is never dropped.

# The bootstrap

Resample the n quadruples with replacement, `RESAMPLES` times, from
`random.Random(RNG_SEED)`; each resample draws ONE index vector and applies it
to all three statistics (that is what makes the interval paired). The interval
is the percentile interval of the resampled means: with B resamples sorted
ascending, the lower bound is element floor(0.025 B) and the upper bound is
element ceil(0.975 B) - 1 (0-based), i.e. elements 250 and 9749 at B = 10,000.
Rows are processed in a fixed order from one generator, so output is
deterministic.

    python3 scripts/did_confidence_intervals.py
    python3 scripts/did_confidence_intervals.py --markdown results/DID_INTERVALS_2026-09-29.md
"""

from __future__ import annotations

import argparse
import contextlib
import dataclasses
import importlib.util
import math
import random
import statistics
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
V1 = ROOT / "results/shd_attention_campaign_v1/cells"
V2 = ROOT / "results/shd_attention_campaign_v2"
V3 = ROOT / "results/shd_attention_campaign_v3"

RESAMPLES = 10_000
RNG_SEED = 20260929
#: Below this lower bound on the intact gain, attention has no reliable gain for
#: the shuffle to remove, and a positive DiD there is a statement about the
#: shuffled arm, not about lost temporal use. The campaign's bar since wave 21.
NO_GAIN_BAR = 0.03

ATTN, RATE = "ff+fixed+attn", "ff+fixed"
SHUF = "bin-shuffled"


def _module(name, relpath):
    """Import a frozen analyser by path under a distinct name."""
    spec = importlib.util.spec_from_file_location(name, ROOT / relpath)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


MC = _module("mechanism_coverage", "scripts/mechanism_coverage.py")
W8 = _module("analyse_wave8", "scripts/aws/analyse_wave8.py")
W21 = _module("analyse_wave21", "scripts/aws/analyse_wave21.py")
W22 = _module("analyse_wave22", "scripts/aws/analyse_wave22.py")
W24 = _module("analyse_wave24", "scripts/aws/analyse_wave24.py")
W25 = _module("analyse_wave25", "scripts/aws/analyse_wave25.py")
W27 = _module("analyse_wave27", "scripts/aws/analyse_wave27.py")

from cell_validity import validity_problems  # noqa: E402  (path set by W8)

ANCHOR = W21.ANCHOR
W21_POINTS = ({(h, *ANCHOR) for h in W21.LADDER} | set(W21.H21_4_POINTS))
W1_STEMS = {  # the only e400 source of h128 / published-2ms / d32l1 with all four arms
    (ATTN, "intact"): "w1__ff-fixed-attn__h128__e400__published-2ms__adjacent-sum-5__d32l1",
    (ATTN, SHUF): "w1__ff-fixed-attn__h128__e400__published-2ms__adjacent-sum-5__d32l1__bin-shuffled",
    (RATE, "intact"): "w1__ff-fixed__h128__e400__published-2ms__adjacent-sum-5",
    (RATE, SHUF): "w1__ff-fixed__h128__e400__published-2ms__adjacent-sum-5__bin-shuffled",
}


@dataclasses.dataclass
class Row:
    hidden: int
    contract: str
    geometry: str
    depth: str
    budget: int
    source: str
    condition: str = SHUF
    #: seed -> (gain intact, gain shuffled, DiD); empty when unpaired.
    quads: dict = dataclasses.field(default_factory=dict)
    reason: str = ""
    stats: dict = dataclasses.field(default_factory=dict)
    note: str = ""


def quadruples(ai, ax, ri, rx):
    """seed -> (gain intact, gain treated, DiD) over the seeds all four arms share."""
    out = {}
    for s in sorted(set(ai) & set(ax) & set(ri) & set(rx)):
        g_int = ai[s] - ri[s]
        g_x = ax[s] - rx[s]
        out[s] = (g_int, g_x, g_int - g_x)
    return out


def percentile_interval(values):
    """The (lower, upper) 95% percentile interval, per the module docstring."""
    ordered = sorted(values)
    b = len(ordered)
    return ordered[math.floor(0.025 * b)], ordered[math.ceil(0.975 * b) - 1]


def bootstrap(quads, rng, resamples=RESAMPLES):
    """Point estimates, positive count and paired percentile CIs for one row."""
    rows = [quads[s] for s in sorted(quads)]
    n = len(rows)
    cols = list(zip(*rows))
    means = [statistics.fmean(c) for c in cols]
    draws = [[], [], []]
    population = range(n)
    for _ in range(resamples):
        idx = rng.choices(population, k=n)
        for k in range(3):
            col = cols[k]
            draws[k].append(sum(col[i] for i in idx) / n)
    names = ("gain_intact", "gain_shuffled", "did")
    out = {"n": n, "positive": sum(1 for r in rows if r[2] > 0)}
    for k, name in enumerate(names):
        out[name] = means[k]
        out[name + "_ci"] = percentile_interval(draws[k])
    return out


def share_of_gain(quads, rng, resamples=RESAMPLES):
    """(point, (lower, upper)): the share of the intact gain that destroying
    order removes, DiD / gain(intact), as a ratio of seed means.

    Resampled by seed quadruple exactly as `bootstrap` resamples, so the ratio
    and its interval come from the same pairing as every other number here.
    It is a property of ONE point: the registered prediction that the DiD
    tracks the gain across widths failed, so a share at the anchor does not
    transfer. Undefined without a positive gain, and refused rather than
    reported as a large or negative ratio."""
    rows = [quads[s] for s in sorted(quads)]
    n = len(rows)
    gain = [r[0] for r in rows]
    did = [r[2] for r in rows]
    if n == 0 or statistics.fmean(gain) <= 0:
        raise ValueError("a share of the gain needs a positive mean intact gain")
    point = statistics.fmean(did) / statistics.fmean(gain)
    draws = []
    population = range(n)
    for _ in range(resamples):
        idx = rng.choices(population, k=n)
        g = sum(gain[i] for i in idx)
        if g <= 0:
            raise ValueError("a resample has no positive gain; the ratio's "
                             "interval is undefined at this point")
        draws.append(sum(did[i] for i in idx) / g)
    return point, percentile_interval(draws)


def anchor_row(rows):
    """The anchor at 400 epochs under bin-shuffling: the n = 32 point."""
    hits = [r for r in rows if (r.hidden, r.contract, r.geometry, r.depth,
                                r.budget, r.condition)
            == (128, *ANCHOR, "d32l4", 400, SHUF)]
    if len(hits) != 1:
        raise ValueError(f"{len(hits)} anchor rows; expected exactly one")
    return hits[0]


@contextlib.contextmanager
def _quiet():
    """Analysers print diagnostics while indexing; keep stdout a clean table."""
    with contextlib.redirect_stdout(sys.stderr):
        yield


def _w1_arms():
    arms = {}
    # analyse_wave8 resolves its manifest relative to the repository root.
    with _quiet(), contextlib.chdir(ROOT):
        for key, stem in W1_STEMS.items():
            cells = W8.load(str(V1), stem)
            arms[key] = {int(c.get("seed") or W8.SEEDS[i]): c["accuracy"]
                         for i, c in enumerate(cells) if not validity_problems(c)}
    return arms


def build_rows():
    """Every row, in mechanism_coverage order: e400 then e100 per point."""
    with _quiet():
        coverage = MC.coverage(MC.read(MC.DEFAULT_ROOTS))
        w21, _ = W21.index((W21.ARCHIVE_V2, W21.ARCHIVE_V1))
        w22, _ = W22.index([W22.ARCHIVE_V1, W22.ARCHIVE_V2])
        w24, _ = W24.index([W24.ARCHIVE_V2])
        w25, _ = W25.index([W25.ARCHIVE_V2])
    w1 = None
    rows = []
    for (hidden, contract, geometry), depth, *_counts, mc_n in coverage:
        point = (hidden, contract, geometry)
        full = (hidden, contract, geometry, depth)

        # ---- e400 -------------------------------------------------------
        row = Row(hidden, contract, geometry, depth, 400, "")
        if depth == "d32l4" and point in W21_POINTS:
            row.source = "W21 corpus"
            if point == (128, *ANCHOR):
                row.source = "W21 corpus (n=32, waves 15-17)"
            arm = lambda a, t: w21.get((*point, a, t), {})  # noqa: E731
            row.quads = quadruples(arm(ATTN, "intact"), arm(ATTN, SHUF),
                                   arm(RATE, "intact"), arm(RATE, SHUF))
            if hidden == 1024:
                row.note = ("gain over the quadruple seeds; the W21 table's "
                            "gain column (-0.1318) is over 20 intact pairs")
        elif full in W22.POINTS:
            row.source = "W22 w22cov"
            arm = lambda a, d, t: w22.get((*point, a, d, t), {})  # noqa: E731
            row.quads = quadruples(arm(ATTN, depth, "intact"), arm(ATTN, depth, SHUF),
                                   arm(RATE, None, "intact"), arm(RATE, None, SHUF))
        elif full == (128, *ANCHOR, "d32l1"):
            row.source = "wave 1 (v1 archive)"
            w1 = w1 or _w1_arms()
            row.quads = quadruples(w1[(ATTN, "intact")], w1[(ATTN, SHUF)],
                                   w1[(RATE, "intact")], w1[(RATE, SHUF)])
        else:
            row.reason = "no frozen analyser owns an e400 four-arm set here"
        if row.quads and len(row.quads) != mc_n:
            row.note = "; ".join(filter(None, (
                row.note, f"mechanism_coverage counts {mc_n} (it applies no "
                          f"validity gate and pools waves)")))
        if not row.quads and not row.reason:
            row.reason = "the owning analyser's four arms share no seed"
        rows.append(row)

        # ---- e100 -------------------------------------------------------
        row = Row(hidden, contract, geometry, depth, 100, "")
        if full in W25.POINTS_E100:
            # H25-4's own wave choice, verbatim.
            rate_wave = W25.W24 if point in W25.RATE_FROM_W24 else W25.THIS
            row.source = (f"W25 w25mec (rate arms {rate_wave})")

            def arm(wave, kind, d, t):
                return W25.arm(w25, wave, 100, *point, kind, d, t, W25.DEFAULT_SCALE)
            row.quads = quadruples(arm(W25.THIS, ATTN, depth, "intact"),
                                   arm(W25.THIS, ATTN, depth, SHUF),
                                   arm(rate_wave, RATE, None, "intact"),
                                   arm(rate_wave, RATE, None, SHUF))
        elif full in W24.POINTS_E100:
            row.source = "W24 w24ord"

            def arm(kind, d, t):
                return W24.arm(w24, 100, *point, kind, d, t)
            row.quads = quadruples(arm(ATTN, depth, "intact"), arm(ATTN, depth, SHUF),
                                   arm(RATE, None, "intact"), arm(RATE, None, SHUF))
        else:
            row.reason = "no e100 cells in any wave"
        if not row.quads and not row.reason:
            row.reason = "the owning analyser's four arms share no seed"
        rows.append(row)

        if full == (128, *ANCHOR, "d32l4"):
            rows.append(hidden_shuffled_row())
    return rows


def hidden_shuffled_row():
    """W27's hidden-shuffled DiD at the anchor, against wave 26's intact arms.

    H27-1 registers that the rate read-out cannot see the hidden time axis, so
    rate intact and rate hidden-shuffled must be IDENTICAL per seed. If they are
    not, this lookup paired the wrong arms and the row is not a measurement.
    """
    with _quiet():
        cells, _full, _voided = W27.index(V3)
    ai = cells.get(W27.arm_key("w26pos", True, "intact"), {})
    ax = cells.get(W27.arm_key("w27hid", True, "hidden-shuffled"), {})
    ri = cells.get(W27.arm_key("w26pos", False, "intact"), {})
    rx = cells.get(W27.arm_key("w27hid", False, "hidden-shuffled"), {})
    row = Row(128, *ANCHOR, "d32l4", 400, "W27 w27hid vs w26pos",
              condition="hidden-shuffled")
    row.quads = quadruples(ai, ax, ri, rx)
    if not row.quads:
        row.reason = "w27hid and w26pos share no seed on all four arms"
        return row
    moved = [s for s in row.quads if ri[s] != rx[s]]
    if moved:
        raise SystemExit(f"hidden-shuffled rate arm differs from intact at seeds "
                         f"{moved}: H27-1 says it cannot, so the arms are mispaired")
    row.note = "rate arm identical under this operator (H27-1), so DiD = attention's own drop"
    return row


def compute(rows, rng_seed=RNG_SEED, resamples=RESAMPLES):
    rng = random.Random(rng_seed)
    for row in rows:
        if row.quads:
            row.stats = bootstrap(row.quads, rng, resamples)
    return rows


def _f(x):
    return f"{x:+.4f}".replace("-", "−")


def _ci(stats, name):
    lo, hi = stats[name + "_ci"]
    return f"{_f(stats[name])} [{_f(lo)}, {_f(hi)}]"


HEADER = ("| width | contract | geometry | read-out | budget | destruction | source | n "
          "| gain intact [95% CI] | gain destroyed [95% CI] | DiD [95% CI] "
          "| DiD > 0 | flag |")


def render(rows):
    out = [HEADER, "|---:|---|---|---|---|---|---|---:|---:|---:|---:|---:|---|"]
    notes = []
    for row in rows:
        lead = (f"| {row.hidden} | `{row.contract}` | `{row.geometry}` | {row.depth} "
                f"| e{row.budget} | {row.condition} | {row.source or '—'} ")
        if not row.stats:
            out.append(lead + f"| — | not paired: {row.reason} | — | — | — | |")
            continue
        s = row.stats
        flag = ("**no gain to lose**" if s["gain_intact_ci"][0] < NO_GAIN_BAR else "")
        if row.note:
            notes.append(len(notes) + 1)
            flag = (flag + f" [{notes[-1]}]").strip()
        out.append(lead + f"| {s['n']} | {_ci(s, 'gain_intact')} "
                   f"| {_ci(s, 'gain_shuffled')} | {_ci(s, 'did')} "
                   f"| {s['positive']}/{s['n']} | {flag} |")
    footnotes = [f"[{i}] h{r.hidden} {r.depth} e{r.budget} {r.condition}: {r.note}"
                 for i, r in enumerate((r for r in rows if r.stats and r.note), 1)]
    return out, footnotes


def document(table, footnotes):
    head = [
        "# Bootstrap intervals on the gain and the difference-in-differences",
        "",
        "Generated by `python3 scripts/did_confidence_intervals.py --markdown "
        "results/DID_INTERVALS_2026-09-29.md`; do not edit by hand. For every "
        "operating point `scripts/mechanism_coverage.py` lists, at each budget "
        "where all four arms exist, it reports the seed-paired intact gain "
        "(attention − rate), the same gain under the destruction operator, and "
        "their difference (the DiD), each with a paired percentile bootstrap 95% "
        f"interval ({RESAMPLES:,} resamples of seed quadruples, RNG seed "
        f"{RNG_SEED}). Arms are read through each wave's frozen analyser. "
        f"**no gain to lose** marks rows whose intact-gain lower bound is below "
        f"+{NO_GAIN_BAR:.2f}: there, a positive DiD does not show attention "
        "losing a temporal advantage it had.",
        "",
    ]
    tail = [""] + footnotes if footnotes else []
    return "\n".join(head + table + tail) + "\n"


def _f3(x):
    return f"{x:+.3f}".replace("-", "−")


def paper_table(rows) -> str:
    """The manuscript's Appendix K table: every bin-shuffled point at both
    budgets, at three decimals so it fits a page. `test_did_confidence_intervals`
    asserts that the draft carries exactly this text, so the appendix cannot
    drift from the cells the way a hand-copied table would.

    † marks a point whose intact gain is below the +0.03 bar; ‡ one whose gain
    is above it but whose interval is not."""
    # Short labels, keyed in the appendix text: the full names in code font
    # overran their columns and printed on top of each other. Pandoc sizes a
    # table wider than its line from the separator row's dash counts, so each
    # column's dashes are the longest thing it prints (header as read, not as
    # typed). Hand-set counts until 2026-09-29 gave the intervals too little
    # and wrapped every one of them mid-bracket.
    header = ["width", "bins", "chan.", "$d$/$L$", "ep.", "$n$",
              "gain, intact", "gain, shuffled", "DiD", "$>0$"]
    shown = ["width", "bins", "chan.", "d/L", "ep.", "n",
             "gain, intact", "gain, shuffled", "DiD", ">0"]
    right = {0, 4, 5, 9}
    cells = []
    for row in rows:
        if row.condition != SHUF:
            continue
        if not row.stats:
            raise ValueError(f"h{row.hidden} {row.depth} e{row.budget} is not "
                             f"paired ({row.reason}); the paper table has no "
                             "row for it and must not silently omit one.")
        s = row.stats
        mark = ("†" if s["gain_intact"] < NO_GAIN_BAR
                else "‡" if s["gain_intact_ci"][0] < NO_GAIN_BAR else "")

        def cell(name):
            lo, hi = s[name + "_ci"]
            return f"{_f3(s[name])} [{_f3(lo)}, {_f3(hi)}]"
        bins = (row.contract.replace("published-", "")
                .replace("fixed-", ""))
        chan = {"adjacent-sum-5": "a5", "channels-700": "700"}[row.geometry]
        d, L = row.depth[1:].split("l")
        cells.append([str(row.hidden), bins, chan, f"{d}/{L}", str(row.budget),
                      str(s["n"]), f"{cell('gain_intact')}{mark}",
                      cell("gain_shuffled"), cell("did"),
                      f"{s['positive']}/{s['n']}"])
    widths = [max([len(shown[k])] + [len(c[k]) for c in cells])
              for k in range(len(header))]
    separator = "|" + "|".join(("-" * (w - 1) + ":") if k in right else "-" * w
                               for k, w in enumerate(widths)) + "|"
    out = ["| " + " | ".join(header) + " |", separator]
    out += ["| " + " | ".join(c) + " |" for c in cells]
    return "\n".join(out)


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--markdown", help="also write the table document to this path")
    parser.add_argument("--paper", action="store_true",
                        help="print only the manuscript's Appendix K table")
    parser.add_argument("--share", action="store_true",
                        help="print the anchor's share of the gain that "
                             "destroying order removes, with its interval")
    args = parser.parse_args(argv)
    rows = compute(build_rows())
    if args.paper:
        print(paper_table(rows))
        return 0
    if args.share:
        row = anchor_row(rows)
        point, (lo, hi) = share_of_gain(row.quads, random.Random(RNG_SEED))
        print(f"anchor share, n = {len(row.quads)}: {point:.3f} "
              f"(95% CI [{lo:.3f}, {hi:.3f}])")
        return 0
    table, footnotes = render(rows)
    print("\n".join(table))
    if footnotes:
        print("\n" + "\n".join(footnotes))
    if args.markdown:
        Path(args.markdown).write_text(document(table, footnotes))
        print(f"\nwrote {args.markdown}", file=sys.stderr)
    unpaired = [r for r in rows if not r.stats]
    if unpaired:
        print(f"\n{len(unpaired)} row(s) could not be paired; see the table.",
              file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
