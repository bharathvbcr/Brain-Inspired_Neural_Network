#!/usr/bin/env python3
"""Wave 27 — the five registered instruments wave 26 built and did not run.

Frozen with `PREREG_2026-09-04_W27_THE_INSTRUMENTS_THAT_WERE_NEVER_RUN.md` and
committed before the first cell exists. Its output is the authority on every
verdict in the result document.

Wave 27 runs on the SAME binary as wave 26 (no Rust source changed since
`97a5d8a`), so section 9.1's "every comparison self-contained inside one binary"
is satisfied by pairing against wave 26's arms rather than by re-running them.
The reused keys are printed, every time, so a reader never has to infer them.

The three-state verdict is built in here rather than amended in later: a clause
whose inputs are absent reads NOT EVALUABLE and carries its reason. NOT MET is a
refutation and is reachable only when every input was present.
"""

from __future__ import annotations

import argparse
import collections
import json
import re
import statistics
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
from cell_validity import validity_problems  # noqa: E402

THIS_WAVES = {"w27hid", "w27drp", "w27lad", "w27qk", "w27tau", "w27spk"}
#: Wave 26's arms, on this same binary, reused as baselines. Printed, not implied.
REUSE_WAVES = {"w26pos", "w26win"}

SEED = re.compile(r"__s(\d+)\.json$")
DEPTH = re.compile(r"__(d\d+l\d+)__")
TAU = re.compile(r"__tau([0-9.]+)__")

#: The campaign's bar, unchanged since wave 21.
BAR = 0.03
#: Seed-paired floor. Fewer quadruples reads NOT EVALUABLE, never NOT MET.
MIN_PAIRS = 9
#: Section 2: the rate arm must actually lose accuracy, or the manipulation is
#: not a manipulation and no null measured under it means anything.
DROPOUT_MIN_COST = 0.03
#: Section 5: qk-norm is "the same instrument with the score term bounded" only
#: if BOTH halves land inside this band.
QK_BAND = 0.03
#: Section 7: the standing gate, `cell_validity.SATURATED_MAX`, with no exemption.
SATURATED_MAX = 0.05
#: Section 7: a rung reported from its surviving seeds is a survivorship
#: statistic, so a rung with more than this many voided seeds is void as a whole.
RUNG_VOID_MAX = 3
#: Unchanged from wave 26's registration.
MONOTONE_TOLERANCE = 0.02
#: The full ladder: wave 26's rungs plus the two that bracket its crossing.
WINDOWS = (2, 4, 8, 16, 32, 64, 128, 192, 256)
W27_WINDOWS = (192, 256)
TAU_MS = (2.5, 5.0, 10.05, 20.0, 40.0)
#: Every group in this wave, and both reused waves, sit at h128. The index key
#: does not carry the width, so a cell at another width would be averaged into
#: the same key rather than colliding visibly. It is dropped and counted here
#: instead: a silently mixed width is the failure that would not announce itself.
ANCHOR_HIDDEN = 128
#: Provenance, not science. Two cells differing only here are the same cell.
PROVENANCE = frozenset({
    "temporal_condition", "temporal_seed", "temporal_audit", "wall_secs",
    "cell_id", "host", "kernel", "machine", "started_at", "finished_at",
    "emitted_unix_s", "emitted_utc", "instance_id", "wave",
})

NOT_EVALUABLE = "NOT EVALUABLE"


def status_of(met, blocked):
    """Three-state outcome; `blocked` is a reason string and always wins."""
    if blocked:
        return blocked
    return "MET" if met else "NOT MET"


def verdict(status, name, met_reading, not_met_reading):
    print(f"\n{name}: {status}")
    if status == "MET":
        print(f"  {met_reading}")
    elif status == "NOT MET":
        print(f"  {not_met_reading}")
    else:
        print("  The clause did not run. This is not a refutation, and no "
              "reading is taken from it.")
    return status


def index(results):
    """(wave, arm, depth, readout, temporal, tau) -> {seed: accuracy}.

    Also returns the whole cell for each key/seed, because H27-1's gate is an
    equality over every scientific field and H27-5's void rule reads
    `saturated_fraction` — neither is answerable from an accuracy alone.
    """
    out = collections.defaultdict(dict)
    full = collections.defaultdict(dict)
    voided = collections.Counter()
    offwidth = collections.Counter()
    for path in sorted(Path(results).glob("*.json")):
        wave = path.name.split("__", 1)[0]
        if wave not in THIS_WAVES and wave not in REUSE_WAVES:
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
        if cell.get("hidden") != ANCHOR_HIDDEN:
            offwidth[wave] += 1
            continue
        arm = cell.get("arm")
        found = DEPTH.search(path.name)
        depth = found.group(1) if found else None
        if arm == "ff+fixed+attn" and depth is None:
            continue
        if arm == "ff+fixed" and depth is not None:
            continue
        tau_found = TAU.search(path.name)
        tau = float(tau_found.group(1)) if tau_found else None
        key = (wave, arm, depth, cell.get("attn_readout") or "default",
               cell.get("temporal_condition") or "intact", tau)
        if validity_problems(cell):
            voided[key] += 1
            continue
        s = int(seed.group(1))
        out[key].setdefault(s, cell["accuracy"])
        full[key].setdefault(s, cell)
    if offwidth:
        print(f"  dropped, not at h{ANCHOR_HIDDEN}: {dict(offwidth)}")
    return out, full, voided


def paired(*arms):
    if not arms or any(not a for a in arms):
        return []
    shared = set(arms[0])
    for a in arms[1:]:
        shared &= set(a)
    return sorted(shared)


def arm_key(wave, attn, temporal="intact", readout="default", tau=None):
    if attn:
        return (wave, "ff+fixed+attn", "d32l4", readout, temporal, tau)
    return (wave, "ff+fixed", None, "default", temporal, tau)


def did(cells, wave, condition, readout="default", intact_wave="w26pos",
        intact_readout=None, tau=None, intact_tau=None):
    """(mean, positive, n) for (attn_intact - attn_X) - (rate_intact - rate_X).

    `intact_wave` defaults to `w26pos` because wave 27 reuses wave 26's intact
    arms throughout; every group here is at that one anchor and on that one
    binary, and `paired` still requires a common seed on all four arms.
    """
    ir = intact_readout or readout
    ai = cells.get(arm_key(intact_wave, True, "intact", ir, intact_tau), {})
    ax = cells.get(arm_key(wave, True, condition, readout, tau), {})
    bi = cells.get(arm_key(intact_wave, False, "intact", tau=intact_tau), {})
    bx = cells.get(arm_key(wave, False, condition, tau=tau), {})
    shared = paired(ai, ax, bi, bx)
    if not shared:
        return None, 0, 0
    deltas = [(ai[s] - ax[s]) - (bi[s] - bx[s]) for s in shared]
    return statistics.fmean(deltas), sum(d > 0 for d in deltas), len(deltas)


def paired_drop(cells, wave, condition, attn=False, intact_wave="w26pos"):
    """(mean drop from intact, count above BAR, n) on one arm."""
    a = cells.get(arm_key(intact_wave, attn, "intact"), {})
    b = cells.get(arm_key(wave, attn, condition), {})
    shared = paired(a, b)
    if not shared:
        return None, 0, 0
    drops = [a[s] - b[s] for s in shared]
    return statistics.fmean(drops), sum(d > BAR for d in drops), len(drops)


def scientific_diff(one, two):
    """Fields on which two cells differ, ignoring provenance."""
    return sorted(k for k in set(one) | set(two)
                  if k not in PROVENANCE and one.get(k) != two.get(k))


def tau_half(ladder, full):
    """Smallest window whose DiD reaches half the full shuffle's.

    Unchanged from wave 26's registration, including every reason it withholds.
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
    if any(b - a < -MONOTONE_TOLERANCE for a, b in zip(values, values[1:])):
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


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--results",
                        default=str(ROOT / "results/shd_attention_campaign_v3"))
    args = parser.parse_args()

    cells, full, voided = index(args.results)
    mine = sum(len(v) for k, v in cells.items() if k[0] in THIS_WAVES)
    reused = sum(len(v) for k, v in cells.items() if k[0] in REUSE_WAVES)
    print("wave 27 — the instruments that were built and never run")
    print(f"  wave-27 cells   : {mine}")
    print(f"  reused wave-26  : {reused}  (same binary, same anchor)")
    print(f"  cells voided    : {sum(voided.values())}")
    for key, count in sorted(voided.items(), key=str):
        print(f"    {count:3d}  {key}")

    if mine == 0:
        print("\nNOTHING TO ANALYSE: no wave-27 cell was found under "
              f"{args.results}. This is not a verdict.")
        return 2

    states = {}

    # ---- H27-1: the only exact zero in the campaign ----------------------
    print("\n=== H27-1  hidden shuffle: a rate arm must pay exactly nothing ===")
    ri = full.get(arm_key("w26pos", False, "intact"), {})
    rh = full.get(arm_key("w27hid", False, "hidden-shuffled"), {})
    shared = paired(ri, rh)
    mismatches = {s: scientific_diff(ri[s], rh[s]) for s in shared}
    broken = {s: d for s, d in mismatches.items() if d}
    print(f"  seed pairs compared : {len(shared)}")
    print(f"  byte-identical      : {len(shared) - len(broken)}")
    if broken:
        for s, d in sorted(broken.items())[:5]:
            print(f"    seed {s} differs on: {d}")
    if not shared:
        h1 = f"{NOT_EVALUABLE}: no rate-arm pair to compare"
    elif broken:
        h1 = "NOT MET"
    else:
        h1 = "MET"
    states["H27-1 exact zero"] = verdict(
        h1, "H27-1  the rate read-out cannot see the hidden time axis",
        "every scientific field is identical across every seed. The operator "
        "separates read-out access from input structure, and the wave is valid.",
        "a rate arm changed under a manipulation it cannot see. Per section 3 of "
        "the instrument registration, EVERY CELL OF THIS WAVE IS VOID; nothing "
        "below is a result.")
    value, positive, n = did(cells, "w27hid", "hidden-shuffled")
    print(f"  DiD(hidden-shuffled) = "
          f"{NOT_EVALUABLE if value is None else round(value, 4)}, "
          f"positive {positive}/{n}  (registered as a QUESTION, not barred)")

    # ---- H27-2: dropout, and whether the manipulation manipulates --------
    print("\n=== H27-2  spike dropout p30 ===")
    cost, above, n = paired_drop(cells, "w27drp", "spike-dropout-p30")
    print(f"  rate-arm accuracy drop from intact = "
          f"{NOT_EVALUABLE if cost is None else round(cost, 4)}, "
          f"above {DROPOUT_MIN_COST} in {above}/{n}")
    blocked = None if n >= MIN_PAIRS else f"{NOT_EVALUABLE}: {n} seed pairs, floor {MIN_PAIRS}"
    h2 = status_of(cost is not None and cost > DROPOUT_MIN_COST
                   and above >= MIN_PAIRS, blocked)
    states["H27-2 dropout bites"] = verdict(
        h2, "H27-2  the manipulation actually costs the substrate something",
        "dropout removes accuracy from the rate arm above the bar. A null under "
        "it would therefore mean something.",
        "dropout does not cost the rate arm accuracy above the bar. This is a "
        "finding about the INSTRUMENT: no null measured under it is "
        "interpretable, and section 2's ambiguity is not resolved.")
    value, positive, n = did(cells, "w27drp", "spike-dropout-p30")
    print(f"  DiD(spike-dropout-p30) = "
          f"{NOT_EVALUABLE if value is None else round(value, 4)}, "
          f"positive {positive}/{n}  (reported, not barred)")

    # ---- H27-3: the ladder wave 26 could not finish ----------------------
    print("\n=== H27-3  the timescale ladder, extended past its crossing ===")
    ladder = {}
    for window in WINDOWS:
        wave = "w27lad" if window in W27_WINDOWS else "w26win"
        value, positive, n = did(cells, wave, f"window-shuffled-w{window}")
        ladder[window] = value
        tag = "  (new)" if window in W27_WINDOWS else ""
        print(f"  w{window:<4} DiD {NOT_EVALUABLE if value is None else round(value, 4)}"
              f"  positive {positive}/{n}{tag}")
    shuffle, sp, sn = did(cells, "w26pos", "bin-shuffled")
    print(f"  full  DiD {NOT_EVALUABLE if shuffle is None else round(shuffle, 4)}"
          f"  positive {sp}/{sn}")
    value, why = tau_half(ladder, shuffle)
    if value is None:
        print(f"\n  tau-half WITHHELD: {why}")
    else:
        print(f"\n  tau-half = {value:.2f} bins ({why})")
        print(f"           = {value * 2.0:.1f} ms at the anchor's 2 ms bins")

    # ---- H27-4: QK-norm at h128, its motivation withdrawn ----------------
    print("\n=== H27-4  QK-norm: the same instrument, or a different read-out? ===")
    qi = cells.get(arm_key("w27qk", True, "intact", "qk-norm"), {})
    di = cells.get(arm_key("w26pos", True, "intact"), {})
    shared = paired(qi, di)
    acc_gap = statistics.fmean([qi[s] - di[s] for s in shared]) if shared else None
    qk_did, _, qk_n = did(cells, "w27qk", "bin-shuffled", readout="qk-norm",
                          intact_wave="w27qk", intact_readout="qk-norm")
    default_did = shuffle
    did_gap = (None if qk_did is None or default_did is None
               else qk_did - default_did)
    print(f"  accuracy(qk-norm) - accuracy(default), intact = "
          f"{NOT_EVALUABLE if acc_gap is None else round(acc_gap, 4)}  "
          f"(n={len(shared)}, band +-{QK_BAND})")
    print(f"  DiD(bin-shuffled) under qk-norm = "
          f"{NOT_EVALUABLE if qk_did is None else round(qk_did, 4)} (n={qk_n}), "
          f"under default = "
          f"{NOT_EVALUABLE if default_did is None else round(default_did, 4)}")
    print(f"  difference = {NOT_EVALUABLE if did_gap is None else round(did_gap, 4)}")
    blocked = (None if (acc_gap is not None and did_gap is not None
                        and len(shared) >= MIN_PAIRS and qk_n >= MIN_PAIRS)
               else f"{NOT_EVALUABLE}: one half of the bar has no pairs")
    h4 = status_of(acc_gap is not None and did_gap is not None
                   and abs(acc_gap) <= QK_BAND and abs(did_gap) <= QK_BAND, blocked)
    states["H27-4 qk-norm is the same instrument"] = verdict(
        h4, "H27-4  qk-norm is the same instrument with the score term bounded",
        "both halves are inside the band. An h1024 result measured with it may "
        "be read against the default arm's.",
        "at least one half is outside the band, so qk-norm is a DIFFERENT "
        "read-out. Nothing measured with it transfers to the paper's headline. "
        "Note this is a read-out check only: wave 26 withdrew its collapse-"
        "specific motivation, and a MET here would not resurrect it.")

    # ---- H27-5: the tau_m ladder under the standing gate ------------------
    print("\n=== H27-5  the tau_m ladder, under cell_validity's standing gate ===")
    print(f"  void rule: saturated_fraction > {SATURATED_MAX} voids a cell; "
          f"more than {RUNG_VOID_MAX} of 12 voids the rung")
    live = {}
    for tau in TAU_MS:
        rate = full.get(arm_key("w27tau", False, tau=tau), {})
        sat = [c.get("saturated_fraction") for c in rate.values()]
        sat = [x for x in sat if isinstance(x, (int, float))]
        over = sum(1 for x in sat if x > SATURATED_MAX)
        planned = len(sat) + voided.get(arm_key("w27tau", False, tau=tau), 0)
        rung_void = over > RUNG_VOID_MAX or planned - len(sat) > RUNG_VOID_MAX
        acc = statistics.fmean(rate_accs) if (rate_accs := [c["accuracy"] for c in rate.values()]) else None
        value, positive, n = did(cells, "w27tau", "intact", tau=tau)
        live[tau] = None if rung_void else value
        flag = "  VOID RUNG" if rung_void else ""
        print(f"  tau {tau:>5} ms  n={len(sat):2d}  mean saturated="
              f"{'--' if not sat else round(statistics.fmean(sat), 4)}  "
              f"over-gate {over}  acc="
              f"{'--' if acc is None else round(acc, 4)}  DiD="
              f"{NOT_EVALUABLE if value is None else round(value, 4)}"
              f" ({positive}/{n}){flag}")
    print("\n  Registered as a threshold, never as a prediction about which rung "
          "trips it. If the long rungs void wholesale the ladder is reported as "
          "bounded by the standing gate and is NOT re-run under a loosened one.")

    # The 10.05 rung is the default path, so it is also a cross-wave check.
    print("\n  cross-wave bit-identity, tau 10.05 vs wave 26's intact arms:")
    for attn in (False, True):
        a = full.get(arm_key("w26pos", attn, "intact"), {})
        b = full.get(arm_key("w27tau", attn, tau=10.05), {})
        sh = paired(a, b)
        bad = [s for s in sh if scientific_diff(a[s], b[s])]
        name = "attn " if attn else "rate "
        if not sh:
            print(f"    {name}: {NOT_EVALUABLE} (no pair)")
        else:
            print(f"    {name}: {len(sh) - len(bad)}/{len(sh)} byte-identical"
                  + (f"  DIFFERING seeds {bad[:3]}" if bad else ""))

    # ---- H27-6: the speaker split, reported and not barred ---------------
    print("\n=== H27-6  speaker-held-out validation (model selection only) ===")
    print("  Registered non-use: no headline number, no DiD, no bar. These cells "
          "train on 6,987 samples and are not comparable to the corpus.")
    for attn in (False, True):
        arm = full.get(arm_key("w27spk", attn), {})
        name = "attn" if attn else "rate"
        if not arm:
            print(f"  {name}: {NOT_EVALUABLE} (no cells)")
            continue
        test = statistics.fmean([c["accuracy"] for c in arm.values()])
        vals = [c.get("val_accuracy") for c in arm.values()]
        vals = [v for v in vals if isinstance(v, (int, float))]
        nval = {c.get("n_val") for c in arm.values()}
        print(f"  {name}: n={len(arm)}  test acc {test:.4f}  "
              f"val acc {'--' if not vals else round(statistics.fmean(vals), 4)}  "
              f"n_val {sorted(x for x in nval if x is not None)}")

    print("\n" + "=" * 70)
    for name, state in states.items():
        label = state if state in ("MET", "NOT MET") else NOT_EVALUABLE
        print(f"  {name}: {label}")
    print("  H27-3, H27-5 and H27-6 are registered as questions or as "
          "thresholds and carry no MET/NOT MET.")
    if states.get("H27-1 exact zero") == "NOT MET":
        print("\nWAVE VOID: the hidden-shuffle equality failed. Section 3 of the "
              "instrument registration voids every cell of a wave in which it "
              "does. Nothing above is a result.")
        return 4
    unrun = [n for n, s in states.items() if s.startswith(NOT_EVALUABLE)]
    if unrun:
        print(f"\nINCOMPLETE: {', '.join(unrun)} did not run. The result document "
              "records them as unevaluated, never as refuted.")
        return 3
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
