#!/usr/bin/env python3
"""Frozen analyser for PREREG_2026-11-11_FAST_NUMERICS_BRIDGE.

It is the authority on HF-0, HF-1 and HF-2 of that registration: is the
`--numerics blas` build (Accelerate sgemm in the attention read-out) equivalent
to the exact build at the SHD anchor, on one platform? It reads only cell
records. It never reads a test split, a checkpoint or a log.

    analyse_fast_numerics_bridge.py <exact_dir> <blas_dir> --determinism <det_dir>
                                    [--json out.json] [--markdown out.md]

# Corpora (each directory holds exactly its registered cells, and nothing else)

  exact_dir  48 cells, the SAME-PLATFORM exact control: the Apple-silicon Mac that
             produced blas_dir, built from the same source, `numerics` absent or
             "exact". NOT the fleet record: that ran on aarch64-unknown-linux-gnu,
             whose libm differs from Apple's (FINDING_2026-08-19_LIBM_PORTABILITY_
             OF_REPLAY), and the rate arm's byte-identity (HF-0b) holds on one
             platform only. A fleet directory passed here fails closed: it holds
             thousands of unregistered cells and, if reduced to the anchor, fails
             HF-0b.
  blas_dir   48 cells, `numerics` == "blas" and one `numerics_platform`.
  det_dir    8 cells, blas attention intact, seeds 5170001-2 x threads {1, 16} x
             reruns {1, 2}. Neither the thread count nor the rerun index is in a
             cell, so both come from the file name: `<anything>__t<threads>__r<rerun>.json`.

Each corpus is the 12 seeds 5170001-5170012 x {rate, attention d32/L4} x {intact,
bin-shuffled} at h128, published-2ms, adjacent-sum-5, e100: the W25 e100 anchor's
arms. Cells are keyed by their own fields, never by file name (except the two
tokens above). A duplicate key, an unregistered cell, a hidden-shuffled cell, a
numerics value other than the corpus's, or a cell failing
`cell_validity.validity_problems` makes the corpus INVALID: nothing is computed.

# Hypotheses, read in order, stopping at the first failure

  HF-0a  every witness of seed 5170001 (and of 5170002) is bit-identical: the four
         determinism cells plus the main-wave blas attention-intact cell. "Bit-
         identical" is every cell field except the clock fields.
  HF-0b  all 24 blas rate cells equal the exact rate cells field for field, except
         `numerics` and `numerics_platform` (the contract: base arms never touch BLAS).
  HF-1   |mean(blas) - mean(exact)| <= 0.03 on the intact attention and rate arms.
  HF-2   TOST at alpha = 0.05: the 90% paired percentile bootstrap interval of the
         per-seed blas-minus-exact delta of G lies strictly inside +-0.015, and
         that of the DiD strictly inside +-0.02. G = attention intact - rate intact;
         DiD = G - (attention shuffled - rate shuffled).

# Exit status

  0 every hypothesis passed    1 INVALID corpus    2 INCOMPLETE (a registered cell
  is missing: no verdict from fewer cells than registered; HF-0 alone is printed
  if only it is evaluable)    3 HF-0 failed (mode withdrawn)    4 HF-1 failed
  5 HF-2 failed.
"""

from __future__ import annotations

import argparse
import json
import math
import random
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from cell_validity import validity_problems  # noqa: E402

SEEDS = tuple(range(5170001, 5170013))
DET_SEEDS = (5170001, 5170002)
DET_THREADS = (1, 16)
DET_RERUNS = (1, 2)
CONDITIONS = ("intact", "bin-shuffled")
KINDS = ("rate", "attn")

FIXED = {
    "schema": "shd-cal-cell-v1", "backend": "rust", "contract": "published-2ms",
    "geometry": "adjacent-sum-5", "hidden": 128, "epochs": 100, "n_train": 8156,
    "n_test": 2264, "surrogate_scale": 1.0, "clip_grad_norm": None,
}
ARM = {"rate": "ff+fixed", "attn": "ff+fixed+attn"}
ATTN_SHAPE = {"attn_dim": 32, "attn_layers": 4}

#: Clock fields, the only ones two runs of one binary may differ on
#: (`train_cell_fixtures.NONDETERMINISTIC_CELL_FIELDS`; a test pins the parity).
CLOCK_FIELDS = ("wall_secs", "emitted_unix_s", "emitted_utc")
#: Fields HF-0b ignores besides the clock: they name the mode, which differs by design.
MODE_FIELDS = ("numerics", "numerics_platform")
PLATFORM = re.compile(r"^macOS \d+(\.\d+)* \([0-9A-Za-z]+\)$")
DET_NAME = re.compile(r"__t(\d+)__r(\d+)\.json$")

# Bars, as registered.
LEVEL_BAR = 0.03          # the campaign's level bar (HF-1)
DELTA_G = 0.015           # HF-2 equivalence margin on G
DELTA_DID = 0.02          # HF-2 equivalence margin on the DiD
ALPHA = 0.05              # TOST: each one-sided test at 0.05 = a 90% two-sided interval
RESAMPLES = 10_000
RNG_SEED = 20261111

# The fleet record this bridge sits beside (W25 e100, h128 published-2ms
# adjacent-sum-5 d32l4: w25mec attention cells, w24ord rate cells, 12 seeds;
# DID_INTERVALS_2026-09-29 reports G +0.1499, DiD +0.1260). Pinned for a
# DESCRIPTIVE platform-move line only: it cannot gate, void or change a verdict.
FLEET = {"attn_intact": 0.820929034, "attn_shuffled": 0.683119847,
         "rate_intact": 0.671046820, "rate_shuffled": 0.659231449,
         "G": 0.149882214, "DiD": 0.125993816}
PLATFORM_MOVE_BAR = 0.03  # AMENDMENT_2026-09-07_WAVE_29 section 7, reported only

# Bars compare differences of means of float accuracies; a value exactly on a bar
# can miss it by an ulp. Accuracies are multiples of 1/2264, so 1e-12 never moves
# a real difference across a bar.
EPS = 1e-12


def at_most(x: float, bar: float) -> bool:
    return x <= bar + EPS


def above(x: float, bar: float) -> bool:
    return x > bar + EPS


def below(x: float, bar: float) -> bool:
    return x < bar - EPS


class Invalid(Exception):
    pass


def registered_keys() -> list[tuple[str, str, int]]:
    return [(k, c, s) for k in KINDS for c in CONDITIONS for s in SEEDS]


def validate(rec: dict, name: str, role: str) -> tuple[str, str, int]:
    """Check one record against the registration; return (kind, condition, seed)."""
    if not isinstance(rec, dict):
        raise Invalid(f"{name}: not a JSON object")
    for key, want in FIXED.items():
        if rec.get(key) != want:
            raise Invalid(f"{name}: {key} = {rec.get(key)!r}, registered {want!r}")
    arm = rec.get("arm")
    kind = next((k for k, v in ARM.items() if v == arm), None)
    if kind is None:
        raise Invalid(f"{name}: arm {arm!r} is not one of {sorted(ARM.values())}")
    for key, want in ATTN_SHAPE.items():
        have = rec.get(key)
        if kind == "attn" and have != want:
            raise Invalid(f"{name}: {key} = {have!r}, registered {want!r}")
        if kind == "rate" and have is not None:
            raise Invalid(f"{name}: a rate cell carries {key} = {have!r}")
    condition = rec.get("temporal_condition")
    if condition not in CONDITIONS:
        raise Invalid(f"{name}: temporal_condition {condition!r} is not registered "
                      f"(registered: {CONDITIONS})")
    seed = rec.get("seed")
    if isinstance(seed, bool) or seed not in SEEDS:
        raise Invalid(f"{name}: seed {seed!r} is not a registered seed")
    acc = rec.get("accuracy")
    if not isinstance(acc, float) or not math.isfinite(acc) or not 0.0 <= acc <= 1.0:
        raise Invalid(f"{name}: accuracy {acc!r} is not a finite float in [0, 1]")
    problems = validity_problems(rec)
    if problems:
        raise Invalid(f"{name}: cell_validity: {'; '.join(problems)}")
    numerics = rec.get("numerics", "exact")
    if role == "exact":
        if numerics != "exact":
            raise Invalid(f"{name}: numerics = {numerics!r} in the exact corpus (mixed numerics)")
        if "numerics_platform" in rec:
            raise Invalid(f"{name}: an exact cell carries numerics_platform")
    else:
        if numerics != "blas":
            raise Invalid(f"{name}: numerics = {numerics!r} in the blas corpus (mixed numerics)")
        platform = rec.get("numerics_platform")
        if not isinstance(platform, str) or not PLATFORM.match(platform):
            raise Invalid(f"{name}: numerics_platform {platform!r} is not 'macOS <version> (<build>)'")
    if role == "det" and (kind, condition) != ("attn", "intact"):
        raise Invalid(f"{name}: determinism cells are attention intact, not {kind} {condition}")
    return kind, condition, seed


def load_dir(root: Path, role: str) -> dict:
    """key -> record. Keys are (kind, condition, seed), or (seed, threads, rerun) for `det`."""
    if not root.is_dir():
        raise Invalid(f"{root} is not a directory")
    cells: dict = {}
    where: dict = {}
    for path in sorted(root.glob("*.json")):
        try:
            rec = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as exc:
            raise Invalid(f"{path.name}: unreadable: {exc}") from exc
        kind, condition, seed = validate(rec, path.name, role)
        if role == "det":
            m = DET_NAME.search(path.name)
            if not m:
                raise Invalid(f"{path.name}: a determinism cell is named ...__t<threads>__r<rerun>.json")
            key = (seed, int(m.group(1)), int(m.group(2)))
            if key[0] not in DET_SEEDS or key[1] not in DET_THREADS or key[2] not in DET_RERUNS:
                raise Invalid(f"{path.name}: (seed, threads, rerun) = {key} is not registered "
                              f"(seeds {DET_SEEDS}, threads {DET_THREADS}, reruns {DET_RERUNS})")
        else:
            key = (kind, condition, seed)
        if key in cells:
            raise Invalid(f"{path.name}: duplicate cell {key}, also in {where[key]}")
        cells[key] = rec
        where[key] = path.name
    return cells


def check_platforms(blas: dict, det: dict) -> str | None:
    platforms = {r["numerics_platform"] for r in [*blas.values(), *det.values()]}
    if len(platforms) > 1:
        raise Invalid(f"mixed numerics_platform values: {sorted(platforms)}")
    return next(iter(platforms), None)


def comparable(rec: dict, drop: tuple[str, ...]) -> dict:
    return {k: v for k, v in rec.items() if k not in drop}


def differing_fields(a: dict, b: dict, drop: tuple[str, ...]) -> list[str]:
    x, y = comparable(a, drop), comparable(b, drop)
    return sorted(k for k in set(x) | set(y) if x.get(k) != y.get(k))


def mean(values) -> float:
    values = list(values)
    return sum(values) / len(values)


# --- HF-0 ---------------------------------------------------------------------

def hf0(exact: dict, blas: dict, det: dict) -> dict:
    need_det = [(s, t, r) for s in DET_SEEDS for t in DET_THREADS for r in DET_RERUNS]
    need_main = [("attn", "intact", s) for s in DET_SEEDS]
    need_rate = [("rate", c, s) for c in CONDITIONS for s in SEEDS]
    gap = ([f"determinism {k}" for k in need_det if k not in det]
           + [f"blas {k}" for k in need_main + need_rate if k not in blas]
           + [f"exact {k}" for k in need_rate if k not in exact])
    if gap:
        return {"status": "NOT EVALUABLE", "missing": gap}
    a_rows = []
    for seed in DET_SEEDS:
        witnesses = {f"t{t}r{r}": det[(seed, t, r)] for t in DET_THREADS for r in DET_RERUNS}
        witnesses["main"] = blas[("attn", "intact", seed)]
        ref = witnesses["t1r1"]
        diffs = {n: differing_fields(ref, w, CLOCK_FIELDS) for n, w in witnesses.items()}
        a_rows.append({"seed": seed, "witnesses": len(witnesses),
                       "identical": not any(diffs.values()),
                       "differing": {n: d for n, d in diffs.items() if d}})
    b_rows = []
    for key in need_rate:
        diff = differing_fields(exact[key], blas[key], CLOCK_FIELDS + MODE_FIELDS)
        b_rows.append({"cell": key, "identical": not diff, "differing": diff})
    a_ok = all(r["identical"] for r in a_rows)
    b_ok = all(r["identical"] for r in b_rows)
    return {"status": "PASS" if a_ok and b_ok else "FAIL",
            "a": {"pass": a_ok, "seeds": a_rows},
            "b": {"pass": b_ok, "cells": len(b_rows),
                  "failing": [r for r in b_rows if not r["identical"]]}}


# --- HF-1, HF-2 -----------------------------------------------------------------

def acc(cells: dict, kind: str, condition: str, seed: int) -> float:
    return cells[(kind, condition, seed)]["accuracy"]


def gain_and_did(cells: dict, seed: int) -> tuple[float, float]:
    g = acc(cells, "attn", "intact", seed) - acc(cells, "rate", "intact", seed)
    gx = acc(cells, "attn", "bin-shuffled", seed) - acc(cells, "rate", "bin-shuffled", seed)
    return g, g - gx


def percentile_interval(values, alpha: float) -> tuple[float, float]:
    """Elements floor(alpha/2 B) and B - 1 - floor(alpha/2 B) of the sorted draws,
    exactly `did_confidence_intervals.percentile_interval` (a test pins the parity)."""
    ordered = sorted(values)
    b = len(ordered)
    k = math.floor(alpha / 2 * b)
    if k == 0:
        raise ValueError(f"{b} resamples cannot resolve a {1 - alpha:.0%} interval")
    return ordered[k], ordered[b - 1 - k]


def paired_intervals(g_deltas, did_deltas, rng_seed=RNG_SEED, resamples=RESAMPLES):
    """90% percentile intervals of the mean G delta and mean DiD delta, one index
    vector per resample applied to both (the paired bootstrap of the repo's
    `did_confidence_intervals.bootstrap`, which draws with `rng.choices`)."""
    rng = random.Random(rng_seed)
    n = len(g_deltas)
    draws_g, draws_d = [], []
    population = range(n)
    for _ in range(resamples):
        idx = rng.choices(population, k=n)
        draws_g.append(sum(g_deltas[i] for i in idx) / n)
        draws_d.append(sum(did_deltas[i] for i in idx) / n)
    return (percentile_interval(draws_g, 2 * ALPHA),
            percentile_interval(draws_d, 2 * ALPHA))


def equivalent(interval: tuple[float, float], margin: float) -> bool:
    """Strict: both one-sided nulls are rejected only if the interval is wholly inside."""
    lo, hi = interval
    return above(lo, -margin) and below(hi, margin)


def hf1_hf2(exact: dict, blas: dict) -> tuple[dict, dict]:
    levels = {}
    for kind in KINDS:
        for cond in CONDITIONS:
            e = mean(acc(exact, kind, cond, s) for s in SEEDS)
            b = mean(acc(blas, kind, cond, s) for s in SEEDS)
            levels[f"{kind} {cond}"] = {"exact": e, "blas": b, "diff": b - e}
    gated = ("attn intact", "rate intact")
    h1 = {"levels": levels,
          "pass": all(at_most(abs(levels[a]["diff"]), LEVEL_BAR) for a in gated),
          "gated_arms": list(gated)}
    per_seed = {s: (gain_and_did(exact, s), gain_and_did(blas, s)) for s in SEEDS}
    dg = [per_seed[s][1][0] - per_seed[s][0][0] for s in SEEDS]
    dd = [per_seed[s][1][1] - per_seed[s][0][1] for s in SEEDS]
    ci_g, ci_d = paired_intervals(dg, dd)
    ok_g, ok_d = equivalent(ci_g, DELTA_G), equivalent(ci_d, DELTA_DID)
    fleet_move = {
        "G": mean(per_seed[s][0][0] for s in SEEDS) - FLEET["G"],
        "DiD": mean(per_seed[s][0][1] for s in SEEDS) - FLEET["DiD"],
        **{f"{k} {c}": levels[f"{k} {c}"]["exact"] - FLEET[f"{k}_{'intact' if c == 'intact' else 'shuffled'}"]
           for k in KINDS for c in CONDITIONS},
    }
    h2 = {
        "G": {"mean_exact": mean(per_seed[s][0][0] for s in SEEDS),
              "mean_blas": mean(per_seed[s][1][0] for s in SEEDS),
              "delta": mean(dg), "ci90": ci_g, "margin": DELTA_G, "pass": ok_g,
              "deltas": dict(zip(SEEDS, dg))},
        "DiD": {"mean_exact": mean(per_seed[s][0][1] for s in SEEDS),
                "mean_blas": mean(per_seed[s][1][1] for s in SEEDS),
                "delta": mean(dd), "ci90": ci_d, "margin": DELTA_DID, "pass": ok_d,
                "deltas": dict(zip(SEEDS, dd))},
        "pass": ok_g and ok_d,
        "resamples": RESAMPLES, "rng_seed": RNG_SEED,
        "positive_G": sum(d > 0 for d in dg), "positive_DiD": sum(d > 0 for d in dd),
        "platform_move_vs_fleet": {
            "descriptive_only": True, "bar_for_reference": PLATFORM_MOVE_BAR,
            "exact_minus_fleet": fleet_move,
        },
    }
    return h1, h2


def analyse(exact: dict, blas: dict, det: dict, platform: str | None) -> tuple[dict, int]:
    out: dict = {"numerics_platform": platform, "HF-0": hf0(exact, blas, det)}
    s0 = out["HF-0"]
    if s0["status"] == "NOT EVALUABLE":
        out["complete"] = False
        return out, 2
    if s0["status"] == "FAIL":
        out["complete"] = True
        out["stopped_at"] = "HF-0"
        return out, 3
    gap = ([f"exact {k}" for k in registered_keys() if k not in exact]
           + [f"blas {k}" for k in registered_keys() if k not in blas])
    if gap:
        out["HF-1"] = {"status": "NOT EVALUABLE", "missing": gap}
        out["complete"] = False
        return out, 2
    out["complete"] = True
    h1, h2 = hf1_hf2(exact, blas)
    out["HF-1"] = {"status": "PASS" if h1["pass"] else "FAIL", **h1}
    if not h1["pass"]:
        out["stopped_at"] = "HF-1"
        return out, 4
    out["HF-2"] = {"status": "PASS" if h2["pass"] else "FAIL", **h2}
    if not h2["pass"]:
        out["stopped_at"] = "HF-2"
        return out, 5
    return out, 0


# --- output ---------------------------------------------------------------------

def _f(x: float) -> str:
    return f"{x:+.4f}".replace("-", "−")


def report(out: dict) -> str:
    lines = []
    s0 = out["HF-0"]
    if s0["status"] == "NOT EVALUABLE":
        return "INCOMPLETE: HF-0 NOT EVALUABLE, missing " + ", ".join(s0["missing"])
    lines.append(f"HF-0: {s0['status']}   (numerics_platform: {out['numerics_platform']})")
    for row in s0["a"]["seeds"]:
        lines.append(f"  (a) seed {row['seed']}: {row['witnesses']} witnesses "
                     + ("bit-identical" if row["identical"] else f"DIFFER: {row['differing']}"))
    lines.append(f"  (b) rate arms, {s0['b']['cells']} cells blas vs exact: "
                 + ("identical" if s0["b"]["pass"] else f"{len(s0['b']['failing'])} DIFFER: {s0['b']['failing']}"))
    if s0["status"] == "FAIL":
        lines.append("Mode withdrawn. Nothing below is read.")
        return "\n".join(lines)
    h1 = out["HF-1"]
    if h1.get("status") == "NOT EVALUABLE":
        lines.append("INCOMPLETE: HF-1 NOT EVALUABLE, missing " + ", ".join(h1["missing"]))
        return "\n".join(lines)
    lines.append(f"HF-1: {h1['status']} (bar {LEVEL_BAR}, on {', '.join(h1['gated_arms'])})")
    for name, v in h1["levels"].items():
        lines.append(f"  {name}: exact {v['exact']:.4f}, blas {v['blas']:.4f}, diff {_f(v['diff'])}")
    if h1["status"] == "FAIL":
        lines.append("Fast mode usable only for self-contained waves with their own controls.")
        return "\n".join(lines)
    h2 = out["HF-2"]
    lines.append(f"HF-2: {h2['status']} (TOST alpha {ALPHA}, {h2['resamples']:,} resamples, RNG {h2['rng_seed']})")
    for name in ("G", "DiD"):
        v = h2[name]
        lines.append(f"  {name}: exact {_f(v['mean_exact'])}, blas {_f(v['mean_blas'])}, delta {_f(v['delta'])}, "
                     f"90% [{_f(v['ci90'][0])}, {_f(v['ci90'][1])}] vs +-{v['margin']}: "
                     + ("equivalent" if v["pass"] else "NOT shown equivalent"))
    move = h2["platform_move_vs_fleet"]["exact_minus_fleet"]
    lines.append("  platform move, Mac exact minus fleet record (descriptive, gates nothing): "
                 + ", ".join(f"{k} {_f(v)}" for k, v in move.items()))
    if h2["status"] == "PASS":
        lines.append("Licensed sentence: equivalent within +-0.015 (G) / +-0.02 (DiD) at the SHD anchor.")
    else:
        lines.append("Fast mode usable only for self-contained waves with their own controls.")
    return "\n".join(lines)


def markdown(out: dict) -> str:
    rows = ["| clause | verdict | detail |", "|---|---|---|"]
    s0 = out["HF-0"]
    rows.append(f"| HF-0 | {s0['status']} | numerics_platform `{out.get('numerics_platform')}` |")
    h1, h2 = out.get("HF-1", {}), out.get("HF-2", {})
    if "levels" in h1:
        for name, v in h1["levels"].items():
            rows.append(f"| HF-1 level, {name} | {h1['status']} | exact {v['exact']:.4f}, blas "
                        f"{v['blas']:.4f}, diff {_f(v['diff'])} |")
    if "G" in h2:
        for name in ("G", "DiD"):
            v = h2[name]
            rows.append(f"| HF-2 {name} | {'PASS' if v['pass'] else 'FAIL'} | delta {_f(v['delta'])}, 90% "
                        f"[{_f(v['ci90'][0])}, {_f(v['ci90'][1])}], margin ±{v['margin']} |")
    return "\n".join(rows) + "\n"


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(description="Frozen analyser, fast-numerics bridge")
    p.add_argument("exact_dir")
    p.add_argument("blas_dir")
    p.add_argument("--determinism", required=True)
    p.add_argument("--json")
    p.add_argument("--markdown")
    args = p.parse_args(argv)
    try:
        exact = load_dir(Path(args.exact_dir), "exact")
        blas = load_dir(Path(args.blas_dir), "blas")
        det = load_dir(Path(args.determinism), "det")
        platform = check_platforms(blas, det)
        out, code = analyse(exact, blas, det, platform)
    except Invalid as exc:
        print(f"INVALID: {exc}")
        return 1
    print(report(out))
    if args.json:
        Path(args.json).write_text(json.dumps(out, indent=2, default=str) + "\n", encoding="utf-8")
    if args.markdown:
        Path(args.markdown).write_text(markdown(out), encoding="utf-8")
    return code


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
