"""Frozen analyser for PREREG_2026-11-08_ATTENTION_ON_THE_DELAY_SUBSTRATE (§7).

It is the authority on every verdict in that registration. It reads only run
records, never a test split, a checkpoint or a log. Its eventual home is
BINN's `scripts/analyse_attention_on_delays.py`, committed with the
registration. Until then it lives in binn-wsp-study/analyser/.

Record contract (`attn-on-delays-run-v1`, one JSON file per run, any file name).
Every field of the reference's `shd-reference-v1` payload, plus:

  arm              one of A00, A00r, B00, A10, A10h, B10 (A00r = A00′, the rerun)
  max_delay        25 (A arms) or 1 (B arms)
  readout          "reference" or "attention"
  order            "intact" or "hidden-shuffled"
  hooks            "absent" (stage 0) or "enabled" (stage 1 and 2)
  memo             true or false, the same in every record
  device           "cpu"; machine "aarch64" (AWS Graviton)
  torch_threads, mkldnn, instance_type
                   the registered run configuration, CONFIG below (prereg §2)
  instance_id      the EC2 instance the run used; A00r must not share A00
                   seed 5170001's (HA-0(c) measures cross-box determinism)
  torch_version    "2.13.0"
  reference_commit "d169b4e"

The arm is read from the record, never from a file name. A record that breaks
the contract makes the corpus INVALID: nothing is computed from it.

Stages and verdicts:
- stage 0: A00 x 5170001-3, A00r x 5170001, B00 x 5170001-3 -> HA-0.
- stage 1: A10, A10h, B10 x 5170001-3 -> HA-1 (or HA-1r), HA-2, HA-3, HA-4.
- stage 2, only if triggered: A00, A10, A10h x 5170004-5 -> HA-1, HA-2, HA-3 at n=5.

A stage is evaluable only when every one of its registered cells is present.
A missing cell gives NOT EVALUABLE, never a verdict from fewer runs.

Exit status: 0 the requested verdicts were read; 1 INVALID corpus; 2 NOT
EVALUABLE; 3 HA-0 failed (the series stops).

usage: analyse_attention_on_delays.py <records_dir> [--json out.json]
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path

SCHEMA = "attn-on-delays-run-v1"
STAGE1_SEEDS = (5170001, 5170002, 5170003)
STAGE2_SEEDS = (5170004, 5170005)
RERUN_SEED = 5170001

ARMS = {
    "A00": {"max_delay": 25, "readout": "reference", "order": "intact", "hooks": "absent"},
    "A00r": {"max_delay": 25, "readout": "reference", "order": "intact", "hooks": "absent"},
    "B00": {"max_delay": 1, "readout": "reference", "order": "intact", "hooks": "absent"},
    "A10": {"max_delay": 25, "readout": "attention", "order": "intact", "hooks": "enabled"},
    "A10h": {"max_delay": 25, "readout": "attention", "order": "hidden-shuffled", "hooks": "enabled"},
    "B10": {"max_delay": 1, "readout": "attention", "order": "intact", "hooks": "enabled"},
}
# A00 at seeds 5170004-5 is a stage-2 control. It is run with the stage-0 code
# path (hooks absent), the same as the stage-0 A00 it extends.

FIXED = {
    "schema": SCHEMA, "mode": "clean", "epochs": 150, "device": "cpu", "machine": "aarch64",
    "exposure_status": "CLEAN_TEST_ONCE", "test_reads_during_training": 0,
    "final_test_reads": 1, "checkpoint_selected_on": "none-final-epoch",
    "torch_version": "2.13.0", "reference_commit": "d169b4e",
}

# The run configuration every record must carry: prereg §2's pick from the
# configuration probe (aws_attn/pick_config.py; aws_attn/fresh_config_probe/
# RESULT.md, run 2026-11-08). If this is ever None, no configuration is
# registered and every corpus is INVALID: the analyser fails closed rather than
# accepting whatever configuration the records happen to share.
CONFIG: dict | None = {"torch_threads": 32, "mkldnn": "off", "instance_type": "c8g.8xlarge"}

STAGE0 = [("A00", s) for s in STAGE1_SEEDS] + [("A00r", RERUN_SEED)] + [("B00", s) for s in STAGE1_SEEDS]
STAGE1 = [(a, s) for a in ("A10", "A10h", "B10") for s in STAGE1_SEEDS]
STAGE2 = [(a, s) for a in ("A00", "A10", "A10h") for s in STAGE2_SEEDS]
REGISTERED = set(STAGE0) | set(STAGE1) | set(STAGE2)

# Bars, as registered.
PINNED_A = 0.9376           # pinned macOS CPU mean of the reference (HK-4's reference)
ARCHIVED_B = 0.6276         # archived Graviton Arm B mean (kernel ablation)
PLATFORM_BAR = 0.010        # HK-4's bar for a platform move
RERUN_NOISE_BAR = 0.005     # same-seed difference that forces stage 2
HA1_STAGE1 = 0.020
HA1_STAGE2 = 0.010
HA1R_BAR = 0.003
HA3_SHARE_BAR = 0.214       # half of the instrument's 0.1258 / (1 - 0.7057)
HA4_MARGIN = 0.030
KERNEL_K = 0.3111           # kernel ablation's K, for the descriptive G_B share

# Bars compare differences of means of float accuracies, so a value that sits
# exactly on a bar in decimal can miss it by one ulp (0.9276 - 0.9376 is
# -0.01000000000000001). Every bar comparison goes through these helpers with
# this tolerance. Accuracies are multiples of 1/2264, so 1e-12 can never move a
# real difference across a bar.
EPS = 1e-12


def at_least(x: float, bar: float) -> bool:
    return x >= bar - EPS


def at_most(x: float, bar: float) -> bool:
    return x <= bar + EPS


def below(x: float, bar: float) -> bool:
    return x < bar - EPS


def above(x: float, bar: float) -> bool:
    return x > bar + EPS


class Invalid(Exception):
    pass


def validate(record: dict, source: str) -> tuple[str, int]:
    if CONFIG is None:
        raise Invalid("no run configuration is registered (CONFIG is None; prereg §2's pick "
                      "is written into the analyser before the commit)")
    for key, want in {**FIXED, **CONFIG}.items():
        if record.get(key) != want:
            raise Invalid(f"{source}: {key} = {record.get(key)!r}, contract requires {want!r}")
    instance = record.get("instance_id")
    if not isinstance(instance, str) or not instance:
        raise Invalid(f"{source}: instance_id must be a non-empty string")
    arm = record.get("arm")
    if arm not in ARMS:
        raise Invalid(f"{source}: arm {arm!r} is not one of {sorted(ARMS)}")
    for key, want in ARMS[arm].items():
        if record.get(key) != want:
            raise Invalid(f"{source}: arm {arm} needs {key} = {want!r}, has {record.get(key)!r}")
    seed = record.get("seed")
    if (arm, seed) not in REGISTERED:
        raise Invalid(f"{source}: ({arm}, seed {seed!r}) is not a registered cell")
    if not isinstance(record.get("memo"), bool):
        raise Invalid(f"{source}: memo must be true or false")
    acc = record.get("accuracy")
    if not isinstance(acc, float) or not math.isfinite(acc) or not 0.0 <= acc <= 1.0:
        raise Invalid(f"{source}: accuracy {acc!r} is not a finite float in [0, 1]")
    return arm, seed


def load_corpus(root: Path) -> dict[tuple[str, int], float]:
    cells: dict[tuple[str, int], float] = {}
    sources: dict[tuple[str, int], str] = {}
    instances: dict[tuple[str, int], str] = {}
    memo_values = set()
    files = sorted(root.glob("*.json"))
    for path in files:
        try:
            record = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as exc:
            raise Invalid(f"{path.name}: unreadable: {exc}") from exc
        if not isinstance(record, dict):
            raise Invalid(f"{path.name}: not a JSON object")
        cell = validate(record, path.name)
        if cell in cells:
            raise Invalid(f"{path.name}: duplicate cell {cell}, also in {sources[cell]}")
        cells[cell] = record["accuracy"]
        sources[cell] = path.name
        instances[cell] = record["instance_id"]
        memo_values.add(record["memo"])
    if len(memo_values) > 1:
        raise Invalid("memo differs across records; the registration fixes one code path")
    rerun, original = ("A00r", RERUN_SEED), ("A00", RERUN_SEED)
    if rerun in instances and instances.get(original) == instances[rerun]:
        raise Invalid(f"A00r ran on the same instance as A00 seed {RERUN_SEED} "
                      f"({instances[rerun]}); HA-0(c) needs a different box")
    return cells


def mean(values: list[float]) -> float:
    return sum(values) / len(values)


def missing(cells: dict, wanted: list[tuple[str, int]]) -> list[tuple[str, int]]:
    return [c for c in wanted if c not in cells]


def stage0(cells: dict) -> dict:
    gap = missing(cells, STAGE0)
    if gap:
        return {"status": "NOT EVALUABLE", "missing": gap}
    a00 = [cells[("A00", s)] for s in STAGE1_SEEDS]
    b00 = [cells[("B00", s)] for s in STAGE1_SEEDS]
    drift_a = mean(a00) - PINNED_A
    drift_b = mean(b00) - ARCHIVED_B
    rerun_delta = cells[("A00r", RERUN_SEED)] - cells[("A00", RERUN_SEED)]
    a_ok = at_most(abs(drift_a), PLATFORM_BAR)
    b_ok = at_most(abs(drift_b), PLATFORM_BAR)
    return {
        "status": "PASS" if a_ok and b_ok else "FAIL",
        "a": {"mean": mean(a00), "drift": drift_a, "pass": a_ok},
        "b": {"mean": mean(b00), "drift": drift_b, "pass": b_ok},
        "rerun": {"delta": rerun_delta, "identical": rerun_delta == 0.0,
                  "forces_stage2": above(abs(rerun_delta), RERUN_NOISE_BAR)},
        "a00_spread": max(a00) - min(a00),
    }


def contrasts(cells: dict, seeds: tuple[int, ...]) -> dict:
    a00 = [cells[("A00", s)] for s in seeds]
    a10 = [cells[("A10", s)] for s in seeds]
    a10h = [cells[("A10h", s)] for s in seeds]
    g_pairs = [x - y for x, y in zip(a10, a00)]
    did_pairs = [x - y for x, y in zip(a10, a10h)]
    g = mean(g_pairs)
    return {
        "n": len(seeds),
        "G": g,
        "G_pairs": g_pairs,
        "G_positive": sum(d > 0 for d in g_pairs),
        "G_h": mean([x - y for x, y in zip(a10h, a00)]),
        "DiD": mean(did_pairs),
        "DiD_positive": sum(d > 0 for d in did_pairs),
        "headroom_share": g / (1.0 - mean(a00)),
    }


def ha1(c: dict) -> str:
    n, g, pos = c["n"], c["G"], c["G_positive"]
    bar = HA1_STAGE1 if n == 3 else HA1_STAGE2
    if at_least(g, bar) and pos == n:
        return "MET"
    if below(g, HA1R_BAR) or pos <= (1 if n == 3 else 3):
        return "REFUTED"
    return "AMBIGUOUS" if n == 3 else "INCONCLUSIVE"


def analyse(cells: dict) -> tuple[dict, int]:
    out: dict = {"HA-0": stage0(cells)}
    s0 = out["HA-0"]
    if s0["status"] == "NOT EVALUABLE":
        return out, 2
    if s0["status"] == "FAIL":
        return out, 3

    later = [c for c in STAGE1 + STAGE2 if c in cells]
    if not later:
        out["stage1"] = {"status": "NOT STARTED"}
        return out, 0
    gap = missing(cells, STAGE1)
    if gap:
        out["stage1"] = {"status": "NOT EVALUABLE", "missing": gap}
        return out, 2

    c1 = contrasts(cells, STAGE1_SEEDS)
    v1 = ha1(c1)
    b00 = [cells[("B00", s)] for s in STAGE1_SEEDS]
    b10 = [cells[("B10", s)] for s in STAGE1_SEEDS]
    gb_pairs = [x - y for x, y in zip(b10, b00)]
    g_b = mean(gb_pairs)
    out["HA-4"] = {
        "G_B": g_b, "G_stage1": c1["G"], "positive": sum(d > 0 for d in gb_pairs),
        "met": at_least(g_b, c1["G"] + HA4_MARGIN) and all(d > 0 for d in gb_pairs),
        "kernel_share_recovered": g_b / KERNEL_K,
    }
    out["stage1"] = {"contrasts": c1, "HA-1": v1}

    stage2_needed = v1 == "AMBIGUOUS" or s0["rerun"]["forces_stage2"]
    stage2_present = [c for c in STAGE2 if c in cells]
    out["stage2_triggered"] = stage2_needed
    if not stage2_needed:
        if stage2_present:
            raise Invalid(f"stage-2 cells exist but stage 2 was not triggered: {stage2_present}")
        read = c1
    else:
        gap2 = missing(cells, STAGE2)
        if gap2:
            out["stage2"] = {"status": "NOT EVALUABLE", "missing": gap2}
            return out, 2
        read = contrasts(cells, STAGE1_SEEDS + STAGE2_SEEDS)
        out["stage2"] = {"contrasts": read, "HA-1": ha1(read)}

    verdict = ha1(read)
    out["HA-1"] = {"n": read["n"], "verdict": verdict}
    out["HA-2"] = (
        {"met": at_least(read["DiD"], 0.5 * read["G"]) and read["DiD_positive"] == read["n"],
         "DiD": read["DiD"], "positive": read["DiD_positive"]}
        if verdict == "MET" else {"status": "NOT READ (HA-1 not met)"}
    )
    out["HA-3"] = {"share": read["headroom_share"], "prediction_held": below(read["headroom_share"], HA3_SHARE_BAR)}
    return out, 0


def report(out: dict) -> str:
    lines = []
    s0 = out["HA-0"]
    if s0["status"] == "NOT EVALUABLE":
        return f"HA-0: NOT EVALUABLE, missing {s0['missing']}"
    lines.append(f"HA-0: {s0['status']}")
    lines.append(f"  (a) mean(A00) {s0['a']['mean']:.4f}, drift {s0['a']['drift']:+.4f} vs {PINNED_A}, bar {PLATFORM_BAR}: "
                 + ("pass" if s0["a"]["pass"] else "FAIL"))
    lines.append(f"  (b) mean(B00) {s0['b']['mean']:.4f}, drift {s0['b']['drift']:+.4f} vs {ARCHIVED_B}, bar {PLATFORM_BAR}: "
                 + ("pass" if s0["b"]["pass"] else "FAIL"))
    r = s0["rerun"]
    lines.append(f"  (c) same-seed rerun delta {r['delta']:+.6f}"
                 + (" (identical)" if r["identical"] else "")
                 + (", over 0.005: stage 2 forced" if r["forces_stage2"] else ""))
    lines.append(f"  A00 three-seed spread {s0['a00_spread']:.4f} (CPU spread was 0.0022)")
    if s0["status"] == "FAIL":
        lines.append("Series stops. Nothing below is read.")
        return "\n".join(lines)
    st1 = out.get("stage1", {})
    if st1.get("status") in ("NOT STARTED", "NOT EVALUABLE"):
        lines.append(f"stage 1: {st1['status']}" + (f", missing {st1['missing']}" if "missing" in st1 else ""))
        return "\n".join(lines)
    c1 = st1["contrasts"]
    lines.append(f"stage 1 (n=3): G {c1['G']:+.4f} ({c1['G_positive']}/3 positive), G_h {c1['G_h']:+.4f}, "
                 f"DiD {c1['DiD']:+.4f}: HA-1 {st1['HA-1']}")
    h4 = out["HA-4"]
    lines.append(f"HA-4: G_B {h4['G_B']:+.4f} vs G {h4['G_stage1']:+.4f} + {HA4_MARGIN} ({h4['positive']}/3 positive): "
                 + ("MET" if h4["met"] else "not met")
                 + f"; kernel share recovered {h4['kernel_share_recovered']:.3f} (descriptive)")
    if "stage2" in out:
        st2 = out["stage2"]
        if st2.get("status") == "NOT EVALUABLE":
            lines.append(f"stage 2: TRIGGERED, NOT EVALUABLE, missing {st2['missing']}")
            return "\n".join(lines)
        c2 = st2["contrasts"]
        lines.append(f"stage 2 (n=5): G {c2['G']:+.4f} ({c2['G_positive']}/5 positive), DiD {c2['DiD']:+.4f}")
    lines.append(f"HA-1 (read at n={out['HA-1']['n']}): {out['HA-1']['verdict']}")
    h2 = out["HA-2"]
    lines.append("HA-2: " + (h2["status"] if "status" in h2 else
                             ("MET" if h2["met"] else "not met") + f", DiD {h2['DiD']:+.4f}, {h2['positive']} pairs positive"))
    h3 = out["HA-3"]
    lines.append(f"HA-3: headroom share {h3['share']:.3f} vs {HA3_SHARE_BAR}: "
                 + ("prediction held" if h3["prediction_held"] else "prediction failed; §3.8's kind sentence is withdrawn"))
    return "\n".join(lines)


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(description="Frozen analyser, attention on the delay substrate")
    p.add_argument("records_dir")
    p.add_argument("--json")
    args = p.parse_args(argv)
    root = Path(args.records_dir)
    if not root.is_dir():
        print(f"INVALID: {root} is not a directory")
        return 1
    try:
        cells = load_corpus(root)
        out, code = analyse(cells)
    except Invalid as exc:
        print(f"INVALID: {exc}")
        return 1
    print(report(out))
    if args.json:
        Path(args.json).write_text(json.dumps(out, indent=2, default=list) + "\n", encoding="utf-8")
    return code


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
