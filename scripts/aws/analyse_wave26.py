#!/usr/bin/env python3
"""Wave 26 — the saturation, the structural null, and the timescale.

FROZEN. Registered in `PREREG_2026-09-03_W26_THE_SATURATION_AND_THE_TIMESCALE.md`
and committed with it, before the first cell of `w26sat`, `w26pos` or `w26win`
exists. Its output is the authority on every verdict in the result document.

Nothing here reads a cell from any other wave. Every instrument this wave uses
is new code, so its binary is new, and section 0 of
`PREREG_2026-09-03_THE_INSTRUMENT_BEFORE_THE_WAVE.md` forbids pairing a cell
from it against an archived half. The rate arms are re-run inside the wave for
exactly that reason, and `index` refuses any cell whose wave label is not one of
this wave's three.

    python3 scripts/aws/analyse_wave26.py --results results/shd_attention_campaign_v2 \
        --probes results/shd_attention_campaign_v2/probes
"""

from __future__ import annotations

import argparse
import collections
import json
import re
import statistics
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
from cell_validity import validity_problems  # noqa: E402

THIS_WAVES = ("w26sat", "w26pos", "w26win")
SEED = re.compile(r"__s(\d+)\.json$")
DEPTH = re.compile(r"__(d\d+l\d+)")

#: The campaign's bar, unchanged since wave 21.
BAR = 0.03
#: Seed-paired floor. A contrast with fewer quadruples reads NOT EVALUABLE
#: rather than being computed from whatever survived.
MIN_PAIRS = 9
#: H26-1 thresholds, fixed in the preregistration before any cell.
ENTROPY_DROP_MIN = 0.20
ENTROPY_DROP_MARGIN = 0.15
QK_GROWTH_MIN = 10.0
QK_GROWTH_CONTROL_MAX = 3.0
ENTROPY_ABSOLUTE = 0.30
#: H26-3: tau-half is the smallest window at which DiD reaches half the full
#: shuffle's, by linear interpolation between bracketing rungs.
WINDOWS = (2, 4, 8, 16, 32, 64, 128)
#: The ladder is withheld rather than summarised if it is not monotone to this.
MONOTONE_TOLERANCE = 0.02


def index(results):
    """(wave, arm, depth, readout, temporal) -> {seed: accuracy}."""
    out = collections.defaultdict(dict)
    voided = collections.Counter()
    for path in sorted(Path(results).glob("*.json")):
        wave = path.name.split("__", 1)[0]
        if wave not in THIS_WAVES:
            continue
        seed = SEED.search(path.name)
        if not seed:
            continue
        try:
            cell = json.loads(path.read_text())
        except ValueError:
            continue
        if not (isinstance(cell, dict) and "accuracy" in cell):
            continue
        arm = cell.get("arm")
        found = DEPTH.search(path.name)
        depth = found.group(1) if found else None
        if arm == "ff+fixed+attn" and depth is None:
            continue
        if arm == "ff+fixed" and depth is not None:
            continue
        key = (wave, arm, depth, cell.get("attn_readout") or "default",
               cell.get("temporal_condition") or "intact")
        if validity_problems(cell):
            voided[key] += 1
            continue
        out[key].setdefault(int(seed.group(1)), cell["accuracy"])
    return out, voided


def paired(*arms):
    """Seeds present in every arm, so every statistic is over one seed set."""
    if not arms or any(not a for a in arms):
        return []
    shared = set(arms[0])
    for a in arms[1:]:
        shared &= set(a)
    return sorted(shared)


def did(cells, wave, condition, readout="default", baseline_readout=None):
    """(mean, positive, n) for (attn_intact - attn_X) - (base_intact - base_X).

    `baseline_readout` of None means the rate arm, which is the campaign's
    estimator. Passing a read-out name puts that arm in the rate arm's place,
    which is what H26-2 reports and does not bar.
    """
    ai = cells.get((wave, "ff+fixed+attn", "d32l4", readout, "intact"), {})
    ax = cells.get((wave, "ff+fixed+attn", "d32l4", readout, condition), {})
    if baseline_readout is None:
        bi = cells.get((wave, "ff+fixed", None, "default", "intact"), {})
        bx = cells.get((wave, "ff+fixed", None, "default", condition), {})
    else:
        bi = cells.get((wave, "ff+fixed+attn", "d32l4", baseline_readout, "intact"), {})
        bx = cells.get((wave, "ff+fixed+attn", "d32l4", baseline_readout, condition), {})
    shared = paired(ai, ax, bi, bx)
    if not shared:
        return None, 0, 0
    deltas = [(ai[s] - ax[s]) - (bi[s] - bx[s]) for s in shared]
    return statistics.fmean(deltas), sum(d > 0 for d in deltas), len(deltas)


def gain(cells, wave, readout, temporal):
    """Seed-paired attention-minus-rate at one condition."""
    a = cells.get((wave, "ff+fixed+attn", "d32l4", readout, temporal), {})
    r = cells.get((wave, "ff+fixed", None, "default", temporal), {})
    shared = paired(a, r)
    if not shared:
        return None, 0, 0, {}
    deltas = {s: a[s] - r[s] for s in shared}
    values = list(deltas.values())
    return statistics.fmean(values), sum(v > 0 for v in values), len(values), deltas


def read_probes(probes):
    """cell id -> {epoch: probe record}. Absent is empty, never an error."""
    out = collections.defaultdict(dict)
    directory = Path(probes)
    if not directory.is_dir():
        return out
    for path in sorted(directory.glob("*.jsonl")):
        cell_id = path.name[: -len(".jsonl")]
        if cell_id.split("__", 1)[0] not in THIS_WAVES:
            continue
        for line in path.read_text().splitlines():
            line = line.strip()
            if not line:
                continue
            try:
                record = json.loads(line)
            except ValueError:
                # A probe line that will not parse is reported, not skipped
                # silently: the whole point of the JSON guard is that this
                # cannot happen, so if it does the reader must see it.
                out[cell_id].setdefault("unparseable", []).append(line[:120])
                continue
            out[cell_id][int(record["epoch"])] = record
    return out


def probe_stat(probes, depth, epoch, field, reduce=statistics.fmean):
    """One statistic over every probed cell of one depth at one epoch."""
    values = []
    for cell_id, records in probes.items():
        if f"__{depth}__" not in cell_id and not cell_id.endswith(f"__{depth}"):
            if f"__{depth}__" not in cell_id:
                continue
        record = records.get(epoch)
        if not isinstance(record, dict):
            continue
        series = record.get(field)
        if isinstance(series, list):
            usable = [v for v in series if isinstance(v, (int, float))]
            if usable:
                values.append(reduce(usable))
        elif isinstance(series, (int, float)):
            values.append(float(series))
    return (statistics.fmean(values) if values else None), len(values)


def qk_product(probes, depth, epoch):
    """Median over cells of mean(q_norm) * mean(k_norm) -- the named term."""
    values = []
    for cell_id, records in probes.items():
        if f"__{depth}__" not in cell_id:
            continue
        record = records.get(epoch)
        if not isinstance(record, dict):
            continue
        q = [v for v in record.get("q_norm", []) if isinstance(v, (int, float))]
        k = [v for v in record.get("k_norm", []) if isinstance(v, (int, float))]
        if q and k:
            values.append(statistics.fmean(q) * statistics.fmean(k))
    return (statistics.median(values) if values else None), len(values)


def tau_half(ladder, full):
    """Smallest window whose DiD reaches half the full shuffle's.

    Linear interpolation between the bracketing rungs. Returns None when the
    ladder never reaches the half-maximum, which is reported as "not reached on
    this ladder" and never as a large tau.
    """
    if full is None:
        return None, ("DiD(bin-shuffled) could not be computed, so there is no "
                      "half-maximum to locate. This is not a statement that the "
                      "order effect is absent")
    if full <= BAR:
        return None, ("DiD(bin-shuffled) is not above the bar; there is no order "
                      "effect to locate")
    target = full / 2.0
    rungs = [(w, ladder[w]) for w in WINDOWS if ladder.get(w) is not None]
    if len(rungs) < 2:
        return None, "fewer than two evaluable rungs"
    values = [v for _, v in rungs]
    drops = [b - a for a, b in zip(values, values[1:])]
    if any(d < -MONOTONE_TOLERANCE for d in drops):
        return None, (f"the ladder is not monotone to within {MONOTONE_TOLERANCE}; "
                      "a half-maximum on a non-monotone curve is an artefact of "
                      "which crossing you take, so tau-half is WITHHELD")
    for (w0, v0), (w1, v1) in zip(rungs, rungs[1:]):
        if v0 >= target:
            return float(w0), "first rung already at or above the half-maximum"
        if v1 >= target:
            if v1 == v0:
                return float(w1), "flat bracket"
            frac = (target - v0) / (v1 - v0)
            return w0 + frac * (w1 - w0), "interpolated"
    return None, "the half-maximum is not reached on the registered ladder"


#: A clause whose inputs were not all present. Distinct from NOT MET, which is
#: a refutation: the bar was tested and the data fell short. Conflating the two
#: is this campaign's own failure shape with the sign flipped -- a check that
#: could not run reporting what a check that ran would report.
NOT_EVALUABLE = "NOT EVALUABLE"


def status_of(met, blocked):
    """Three-state outcome. `blocked` is a reason string when the clause could
    not be evaluated at all, and it always wins over `met`."""
    if blocked:
        return blocked
    return "MET" if met else "NOT MET"


def verdict(status, name, met_reading, not_met_reading):
    """Print one clause's outcome.

    `status` is "MET", "NOT MET", or a string opening with NOT_EVALUABLE that
    carries its own reason. Only the first two are scientific verdicts; the
    third says the clause did not run, and no reading is attached to it.
    """
    print(f"\n{name}: {status}")
    if status == "MET":
        print(f"  {met_reading}")
    elif status == "NOT MET":
        print(f"  {not_met_reading}")
    else:
        print("  The clause did not run. This is not a refutation, and no "
              "reading is taken from it.")
    return status


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--results", default=str(ROOT / "results/shd_attention_campaign_v2"))
    parser.add_argument("--probes",
                        default=str(ROOT / "results/shd_attention_campaign_v2/probes"))
    args = parser.parse_args()

    cells, voided = index(args.results)
    probes = read_probes(args.probes)
    indexed = sum(len(v) for v in cells.values())
    print("wave 26 — the saturation, the structural null, and the timescale")
    print(f"  cells indexed : {indexed}")
    print(f"  cells voided  : {sum(voided.values())}")
    for key, count in sorted(voided.items()):
        print(f"    {count:3d}  {key}")
    print(f"  probed cells  : {len(probes)}")
    unparseable = [c for c, r in probes.items() if "unparseable" in r]
    if unparseable:
        print(f"  UNPARSEABLE probe lines in {len(unparseable)} cells: {unparseable[:3]}")

    # A hypothesis that could not be evaluated must never print as NOT MET.
    #
    # Run against an empty results directory -- a wave that has not landed, a
    # mistyped `--results`, a bucket that was never collected -- every clause
    # below reads None, every comparison against None is false, and the summary
    # said NOT MET three times. That is the campaign's own failure shape with
    # the sign flipped: a check that could not run reporting the same thing as a
    # check that ran and refuted. Refuse instead.
    if indexed == 0:
        print("\nNOTHING TO ANALYSE: no wave-26 cell was found under "
              f"{args.results}. This is not a verdict. Every hypothesis below "
              "would read NOT MET because nothing ran, which is exactly what a "
              "refutation must not be confusable with.")
        return 2

    # ---------------- H26-1 : does the collapse saturate the read-out? ------
    print("\n=== H26-1  the h1024 collapse and the read-out's entropy ===")
    rows = []
    for depth in ("d32l4", "d32l2"):
        row = {}
        for epoch in (1, 25, 50, 100, 200, 400):
            value, n = probe_stat(probes, depth, epoch, "normalised_entropy")
            row[epoch] = value
            if value is not None:
                print(f"  {depth} e{epoch:<4} entropy {value:.4f}  (n={n})")
        rows.append((depth, row))
    entropy = dict(rows)

    qk = {}
    for depth in ("d32l4", "d32l2"):
        first, n1 = qk_product(probes, depth, 1)
        last, n2 = qk_product(probes, depth, 400)
        qk[depth] = (first, last)
        if first and last:
            print(f"  {depth} |q||k| e1 {first:.3f} -> e400 {last:.3f}  "
                  f"({last / first:.2f}x, n={min(n1, n2)})")

    void_h26_1 = (entropy["d32l4"].get(1) is not None
                  and entropy["d32l4"][1] < ENTROPY_ABSOLUTE)
    if void_h26_1:
        print(f"\n  VOID: entropy at epoch 1 is already below {ENTROPY_ABSOLUTE}. "
              "The probe is measuring initialisation, not training, and the "
              "epoch grid is wrong. No verdict is taken from it.")

    # The void rule and an absent probe are both "no verdict", never NOT MET.
    # The prereg's words for the void are "No verdict is taken from it";
    # printing NOT MET would be taking one.
    if not probes:
        blocked_h1 = f"{NOT_EVALUABLE}: no probe file was read from {args.probes}"
    elif void_h26_1:
        blocked_h1 = f"{NOT_EVALUABLE} (VOID): the registered void rule fired"
    else:
        blocked_h1 = None

    def drop(depth):
        early, late = entropy[depth].get(100), entropy[depth].get(400)
        return None if early is None or late is None else early - late

    d4, d2 = drop("d32l4"), drop("d32l2")
    h1a_blocked = blocked_h1 or (
        None if (d4 is not None and d2 is not None)
        else f"{NOT_EVALUABLE}: the e100 or e400 entropy row is missing")
    h1a = (d4 is not None and d2 is not None
           and d4 >= ENTROPY_DROP_MIN and (d4 - d2) >= ENTROPY_DROP_MARGIN)
    print(f"\n  entropy drop e100->e400: d32l4 {d4 if d4 is None else round(d4, 4)}, "
          f"d32l2 {d2 if d2 is None else round(d2, 4)}")
    h1a = verdict(status_of(h1a, h1a_blocked), "H26-1a  the read-out saturates where the fit is lost, and not in the control",
            f"d32l4 drops {d4} (bar {ENTROPY_DROP_MIN}) and exceeds d32l2's {d2} "
            f"by at least {ENTROPY_DROP_MARGIN}. The collapse has a mechanism.",
            "the entropy drop is absent or is not specific to the collapsing depth. "
            "The saturation account is not supported by its own primary clause.")

    g4 = qk["d32l4"]
    g2 = qk["d32l2"]
    growth4 = None if not (g4[0] and g4[1]) else g4[1] / g4[0]
    growth2 = None if not (g2[0] and g2[1]) else g2[1] / g2[0]
    h1b_blocked = blocked_h1 or (
        None if (growth4 is not None and growth2 is not None)
        else f"{NOT_EVALUABLE}: the e1 or e400 |q||k| row is missing")
    h1b = (growth4 is not None and growth2 is not None
           and growth4 >= QK_GROWTH_MIN and growth2 < QK_GROWTH_CONTROL_MAX)
    h1b = verdict(status_of(h1b, h1b_blocked), "H26-1b  the named term is what grows",
            f"|q||k| grows {growth4}x at d32l4 against {growth2}x at d32l2. "
            "The proposed cause is the one that moved.",
            f"|q||k| grew {growth4}x at d32l4 and {growth2}x at d32l2, against bars "
            f"of >={QK_GROWTH_MIN}x and <{QK_GROWTH_CONTROL_MAX}x. If H26-1a is MET "
            "and this is NOT, the read-out saturates for a reason this wave did not "
            "identify, and it is reported as a NEW open problem rather than as "
            "confirmation.")

    late4 = entropy["d32l4"].get(400)
    if late4 is None:
        h1c_reading = f"{NOT_EVALUABLE}: no e400 entropy row"
    else:
        h1c_reading = "below" if late4 < ENTROPY_ABSOLUTE else "not below"
    print(f"\n  H26-1c (secondary, reported not required): entropy at e400/d32l4 "
          f"= {late4}, threshold {ENTROPY_ABSOLUTE} -> {h1c_reading}")

    # ---------------- H26-2 : the structural null ---------------------------
    print("\n=== H26-2  removing the read-out's access to order ===")
    gd, gdp, gdn, gd_by_seed = gain(cells, "w26pos", "default", "intact")
    gn, gnp, gnn, gn_by_seed = gain(cells, "w26pos", "no-position", "intact")
    gs, gsp, gsn, _ = gain(cells, "w26pos", "default", "bin-shuffled")
    for name, value, positive, n in (("gain default intact", gd, gdp, gdn),
                                     ("gain no-position intact", gn, gnp, gnn),
                                     ("gain default bin-shuffled", gs, gsp, gsn)):
        print(f"  {name:28s} {value if value is None else round(value, 4)}  "
              f"positive {positive}/{n}")

    shared = sorted(set(gd_by_seed) & set(gn_by_seed))
    h2a_blocked = None
    h2a = False
    if len(shared) < MIN_PAIRS:
        h2a_blocked = (f"{NOT_EVALUABLE}: {len(shared)} seed pairs, "
                       f"floor {MIN_PAIRS}")
        print(f"  {h2a_blocked}")
    else:
        deltas = [gd_by_seed[s] - gn_by_seed[s] for s in shared]
        mean_delta = statistics.fmean(deltas)
        above = sum(d > BAR for d in deltas)
        print(f"  gain(default) - gain(no-position) = {mean_delta:.4f}, "
              f"above {BAR} in {above}/{len(deltas)}")
        h2a = mean_delta > BAR and above >= MIN_PAIRS
    h2a = verdict(status_of(h2a, h2a_blocked), "H26-2a  position is what the read-out's advantage runs through",
            "removing the positional code costs more than the campaign's bar. The "
            "arm keeps every parameter and loses the advantage.",
            "removing the positional code does not cost the read-out its advantage. "
            "The paper's account of WHAT the read-out consumes is wrong, and that is "
            "a finding about the mechanism claim rather than about this wave.")

    if gn is not None and gs is not None:
        agreement = abs(gn - gs)
        print(f"\n  H26-2b (registered as a QUESTION, not a prediction)")
        print(f"    removing the read-out's access to order : gain {gn:.4f}")
        print(f"    removing order from the data           : gain {gs:.4f}")
        print(f"    |difference| = {agreement:.4f}  (inside {BAR}: "
              f"{'yes' if agreement <= BAR else 'no'})")
    for condition, readout in (("bin-shuffled", "no-position"),):
        value, positive, n = did(cells, "w26pos", condition, baseline_readout=readout)
        print(f"  DiD(attn vs no-position, {condition}) = "
              f"{value if value is None else round(value, 4)}, positive {positive}/{n} "
              "(reported, not barred)")

    # ---------------- H26-3 : the timescale ladder --------------------------
    print("\n=== H26-3  at what timescale does the read-out use order? ===")
    ladder = {}
    for window in WINDOWS:
        value, positive, n = did(cells, "w26win", f"window-shuffled-w{window}")
        ladder[window] = value
        print(f"  w{window:<4} DiD "
              f"{NOT_EVALUABLE if value is None else round(value, 4)}  "
              f"positive {positive}/{n}")
    full, full_positive, full_n = did(cells, "w26pos", "bin-shuffled")
    print(f"  full  DiD {NOT_EVALUABLE if full is None else round(full, 4)}  "
          f"positive {full_positive}/{full_n}")
    value, why = tau_half(ladder, full)
    if value is None:
        print(f"\n  tau-half WITHHELD: {why}")
    else:
        print(f"\n  tau-half = {value:.2f} bins ({why})")
        print(f"           = {value * 2.0:.1f} ms at the anchor's 2 ms bins")

    print("\n" + "=" * 70)
    def label(state):
        return state if state in ("MET", "NOT MET") else NOT_EVALUABLE

    print(f"H26-1a {label(h1a)} | H26-1b {label(h1b)} | H26-2a {label(h2a)}")
    print("H26-2b and H26-3 are registered as questions and have no MET/NOT MET.")
    unrun = [name for name, state in (("H26-1a", h1a), ("H26-1b", h1b),
                                      ("H26-2a", h2a))
             if state.startswith(NOT_EVALUABLE)]
    if unrun:
        print(f"INCOMPLETE: {', '.join(unrun)} did not run. The result document "
              "records them as unevaluated, never as refuted.")
        return 3
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
