"""Pre-run readiness checks and a single-cell convergence probe.

Two questions this answers before a long campaign starts:

    readiness    Will this run fail three hours in for a reason I could have
                 seen now? Artifacts, authorization, disk, runtime estimate,
                 and a dry-run of the temporal manipulation.

    convergence  Is the epoch budget actually enough? The completed matrix
                 reports accuracy still rising with width at h512, which leaves
                 open whether 0.7151 is a ceiling or just where 100 epochs ran
                 out. `--temporal` cells now emit per-epoch loss traces, so this
                 is answerable from one cell instead of by re-running the matrix.

Usage
    python scripts/probe.py readiness --campaign temporal
    python scripts/probe.py convergence --cell <path to cell json>
    python scripts/probe.py convergence --run --contract published-2ms \\
        --geometry adjacent-sum-5 --hidden 512 --epochs 100 --seed 5170001
"""

from __future__ import annotations

import argparse
import itertools
import json
import shutil
import subprocess
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))

ROOT = Path(__file__).resolve().parent.parent
RESULT_ROOT = ROOT / "results" / "shd_instrument_v4"
EVENT_ROOT = ROOT / "data" / "shd" / "events"
RUST_BINARY = ROOT / "target" / "release" / "shd-instrument"

SEEDS = (5170001, 5170002, 5170003)
ARMS = ("ff+fixed", "rec+alif")
CONDITIONS = ("intact", "bin-shuffled", "channel-shuffled", "reversed")
ANCHOR = {
    "contract": "published-2ms",
    "geometry": "adjacent-sum-5",
    "hidden": 512,
    "epochs": 100,
}
# Measured anchor cost (rust, ff+fixed, 3-seed mean) and the derived recurrent
# multiplier from PREREG_2026-08-02_SHD_BPTT_CEILING_RECALIF section 9.
ANCHOR_SECONDS_FF = 1405.0
RECURRENT_MULTIPLIER = 6.5
# PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION gate 5.2.
TRAINED_REGIME_FLOOR = 0.65
MEASURED_CEILING = 0.7151

OK, WARN, FAIL = "ok", "warn", "FAIL"


def check(name: str, status: str, detail: str) -> dict:
    return {"name": name, "status": status, "detail": detail}


def readiness(campaign: str) -> list[dict]:
    results: list[dict] = []

    # --- binary -----------------------------------------------------------
    if not RUST_BINARY.is_file():
        results.append(check("rust binary", FAIL,
                             f"missing {RUST_BINARY.relative_to(ROOT)} - run "
                             "`cargo build --release -p binn-lab --bin shd-instrument`"))
    else:
        results.append(check("rust binary", OK, str(RUST_BINARY.relative_to(ROOT))))
        try:
            probe = subprocess.run([str(RUST_BINARY), "help"], capture_output=True, text=True)
            text = probe.stdout + probe.stderr
            for flag in ("--arm", "--temporal", "--temporal-seed"):
                results.append(check(
                    f"binary supports {flag}",
                    OK if flag in text else FAIL,
                    "present" if flag in text
                    else "absent - the binary predates the wiring, rebuild",
                ))
        except OSError as error:
            # e.g. probing a macOS binary from a linux sandbox.
            results.append(check("binary flag probe", WARN,
                                 f"could not exec ({error.strerror}); flag support unverified"))

    # --- authorization ----------------------------------------------------
    gates_path = RESULT_ROOT / "gates.json"
    if not gates_path.is_file():
        results.append(check("gates.json", FAIL, "missing"))
    else:
        gates = json.loads(gates_path.read_text())
        blocking = [k for k in ("data_parity", "forward_parity", "gradient_parity",
                                "update_parity", "matrix_authorized") if not gates.get(k)]
        results.append(check(
            "prerequisite gates", OK if not blocking else FAIL,
            "all pass" if not blocking else f"failing: {', '.join(blocking)}"))
        # SHD_INSTRUMENT_STATUS blocks new architecture campaigns while UNCALIBRATED.
        scientific = gates.get("scientific_status") or _ledger_field("scientific_status")
        results.append(check(
            "campaign authorization",
            WARN if scientific != "CALIBRATED" else OK,
            f"scientific_status={scientific!r}; SHD_INSTRUMENT_STATUS blocks new "
            "architecture campaigns until calibration closes - an amendment is required"))

    # --- data -------------------------------------------------------------
    for name in ("train.events", "test.events"):
        path = EVENT_ROOT / name
        results.append(check(f"event cache {name}", OK if path.is_file() else FAIL,
                             f"{path.stat().st_size / 1e6:.1f} MB" if path.is_file() else "missing"))

    # --- per-cell initialization artifacts --------------------------------
    n_inputs = 700 if ANCHOR["geometry"] == "channels-700" else 140
    missing = []
    for seed in SEEDS:
        for suffix, stem in ((".weights", f"n{n_inputs}-h{ANCHOR['hidden']}-s{seed}"),
                             (".orders", f"n8156-e100-s{seed}")):
            path = RESULT_ROOT / "initialization" / f"{stem}{suffix}"
            if not path.is_file():
                missing.append(path.name)
    results.append(check(
        "initialization artifacts", OK if not missing else FAIL,
        "all present" if not missing else f"missing {len(missing)}: {', '.join(missing[:4])}"))

    # --- planned cells + runtime + disk -----------------------------------
    planned = list(itertools.product(ARMS, CONDITIONS, SEEDS))
    seconds = sum(
        ANCHOR_SECONDS_FF * (RECURRENT_MULTIPLIER if arm.startswith("rec") else 1.0)
        for arm, _, _ in planned
    )
    results.append(check("planned cells", OK, f"{len(planned)} ({len(ARMS)} arms x "
                                              f"{len(CONDITIONS)} conditions x {len(SEEDS)} seeds)"))
    results.append(check("estimated runtime", WARN if seconds > 48 * 3600 else OK,
                         f"{seconds / 3600:.1f} h serial (derived, not measured - "
                         "run `probe.py convergence --run` for a real timing point)"))
    free_gb = shutil.disk_usage(ROOT).free / 1e9
    # Epoch traces add two float arrays per cell; still tiny next to the ledger.
    results.append(check("disk free", OK if free_gb > 5 else WARN, f"{free_gb:.1f} GB"))

    # --- temporal manipulation dry run ------------------------------------
    if campaign == "temporal":
        results.extend(_temporal_dry_run())

    return results


def _ledger_field(field: str):
    path = RESULT_ROOT / "ledger.json"
    if not path.is_file():
        return None
    return json.loads(path.read_text()).get(field)


def _temporal_dry_run() -> list[dict]:
    """Apply each condition to real samples and check the prereg gates hold."""
    from shd_calibration.data import Contract, frame_events, read_event_cache

    try:
        samples = read_event_cache(EVENT_ROOT / "train.events", 12)
    except Exception as error:  # pragma: no cover - environment dependent
        return [check("temporal dry run", FAIL, f"could not read events: {error}")]

    contract = Contract("published", 2)
    framed = [frame_events(s, contract, ANCHOR["geometry"]) for s in samples]
    results = []
    for condition in CONDITIONS:
        relocated_total = 0
        entries_total = 0
        counts_ok = True
        for index, sample in enumerate(framed):
            before = np.zeros(sample.n_inputs, dtype=np.float64)
            for frame in sample.frames:
                for channel, count in frame:
                    before[channel] += float(count)
            moved, entries, after = _apply(sample, condition, seed=index)
            relocated_total += moved
            entries_total += entries
            counts_ok &= bool(np.array_equal(before, after))
        fraction = relocated_total / max(entries_total, 1)
        if not counts_ok:
            results.append(check(f"temporal {condition}", FAIL,
                                 "per-channel counts changed - prereg gate 5.1 would void the run"))
        elif condition == "intact":
            results.append(check(f"temporal {condition}", OK if fraction == 0 else FAIL,
                                 f"relocated {fraction:.3f} (must be 0)"))
        else:
            results.append(check(
                f"temporal {condition}", OK if fraction > 0.5 else FAIL,
                f"counts preserved, relocated {fraction:.3f} "
                f"({'ok' if fraction > 0.5 else 'too low - manipulation is not doing its job'})"))
    return results


def _apply(sample, condition, seed):
    """Python mirror of shd_temporal::apply_temporal, for the dry run only."""
    frames = [list(f) for f in sample.frames]
    steps = len(frames)
    generator = np.random.default_rng(seed)
    moved = entries = 0
    rebuilt: list[list] = [[] for _ in range(steps)]
    if condition == "intact":
        rebuilt = frames
        entries = sum(len(f) for f in frames)
    elif condition == "reversed":
        for old, frame in enumerate(frames):
            new = steps - 1 - old
            rebuilt[new] = list(frame)
            entries += len(frame)
            moved += len(frame) if new != old else 0
    elif condition == "bin-shuffled":
        permutation = generator.permutation(steps)
        for old, frame in enumerate(frames):
            new = int(permutation[old])
            rebuilt[new].extend(frame)
            entries += len(frame)
            moved += len(frame) if new != old else 0
    elif condition == "channel-shuffled":
        permutations = {}
        for old, frame in enumerate(frames):
            for channel, count in frame:
                if channel not in permutations:
                    permutations[channel] = np.random.default_rng(
                        (seed + 1) * 0x9E3779B9 + channel).permutation(steps)
                new = int(permutations[channel][old])
                rebuilt[new].append((channel, count))
                entries += 1
                moved += 1 if new != old else 0
    after = np.zeros(sample.n_inputs, dtype=np.float64)
    for frame in rebuilt:
        for channel, count in frame:
            after[channel] += float(count)
    return moved, entries, after


def sparkline(values: list[float]) -> str:
    if not values:
        return ""
    blocks = ".:-=+*#%@"
    low, high = min(values), max(values)
    span = high - low or 1.0
    return "".join(blocks[min(len(blocks) - 1, int((v - low) / span * (len(blocks) - 1)))]
                   for v in values)


def convergence(cell_path: Path) -> int:
    payload = json.loads(cell_path.read_text())
    trace = payload.get("epoch_mean_loss")
    if not trace:
        print("This cell carries no epoch trace. It predates the telemetry added on "
              "2026-08-02; re-run it with the current binary.")
        return 1
    trace = [float(v) for v in trace]
    accuracy = float(payload["accuracy"])
    tail = max(len(trace) // 10, 1)
    earlier, later = trace[-tail - 1], trace[-1]
    improvement = (later - earlier) / earlier if earlier else 0.0
    total = (trace[-1] - trace[0]) / trace[0] if trace[0] else 0.0

    print(f"cell        {cell_path.name}")
    print(f"arm         {payload.get('arm', 'ff+fixed')}   "
          f"temporal {payload.get('temporal_condition', 'intact')}")
    print(f"accuracy    {accuracy:.4f}   (measured ceiling {MEASURED_CEILING:.4f})")
    print(f"epochs      {len(trace)}")
    print(f"loss        {trace[0]:.4f} -> {trace[-1]:.4f}   ({total:+.1%} total)")
    print(f"trace       {sparkline(trace)}")
    print(f"tail        {improvement:+.4%} over the final {tail} epoch(s)")

    verdict = 0
    if improvement < -0.01:
        print("\nTRAINING LOSS STILL FALLING at the end of the budget.")
        print("  The 'ceiling' claim is NOT supported by this cell: the run was truncated")
        print("  while the optimiser was still making progress.")
        print("\n  IMPORTANT - this trace is TRAINING loss, so it cannot by itself")
        print("  distinguish two very different situations:")
        print("    (a) undertrained  - test accuracy would still rise with more epochs")
        print("    (b) overfitting   - training loss falls while test accuracy is flat")
        print("  Both look identical here. Only test accuracy across budgets separates")
        print("  them: run `probe.py budget` (~4 h, 3 cells).")
        verdict = 1
    elif improvement < -0.001:
        print("\nMARGINAL. Training loss still drifting down. Treat any ceiling claim as")
        print("  budget-dependent, and confirm with `probe.py budget`.")
    else:
        print("\nTRAINING LOSS FLAT at the end of the budget. Consistent with convergence,")
        print("  and consistent with reading the accuracy as an architecture ceiling -")
        print("  though a budget sweep on test accuracy is still the direct evidence.")

    if accuracy < TRAINED_REGIME_FLOOR:
        print(f"\nGATE 5.2 FAIL: accuracy {accuracy:.4f} < {TRAINED_REGIME_FLOOR} floor.")
        print("  The temporal experiment cannot run at this configuration: an undertrained")
        print("  model has not learned timing, so shuffling cannot take timing away and a")
        print("  null result would be uninformative. This is exactly the confound that")
        print("  disqualified the 2026-08-02 pilot.")
        verdict = 1
    else:
        print(f"\nGate 5.2 pass: accuracy {accuracy:.4f} >= {TRAINED_REGIME_FLOOR} floor.")
    return verdict


REGISTERED_EPOCHS = 100


def probe_orders(seed: int, epochs: int, hidden: int, n_inputs: int) -> Path:
    """Order file covering `epochs`, generated outside `initialization/`.

    The registered `n8156-e100-*.orders` only covers 100 epochs. Longer budgets
    need a longer file, but regenerating in place would rewrite artifacts the
    216 completed rust cells are pinned to. So probe order files live under
    `probe/` and the registered ones are never touched.

    `init` writes weights alongside orders, so its weights go to a throwaway
    path which is then asserted byte-identical to the registered file. That is
    a free determinism check: if it ever differs, the initialisation lineage has
    drifted and every downstream comparison is invalid.

    Epoch orders come from a single `PortableRng` stream, so the first 100
    epochs of a longer file are identical to the registered one - the budgets
    are nested, not independent draws.
    """
    registered = RESULT_ROOT / "initialization" / f"n8156-e{REGISTERED_EPOCHS}-s{seed}.orders"
    if epochs <= REGISTERED_EPOCHS:
        return registered

    directory = RESULT_ROOT / "probe" / "orders"
    directory.mkdir(parents=True, exist_ok=True)
    orders = directory / f"n8156-e{epochs}-s{seed}.orders"
    scratch_weights = directory / f"scratch-n{n_inputs}-h{hidden}-s{seed}.weights"
    if not orders.is_file():
        print(f"generating {epochs}-epoch order file for seed {seed}", flush=True)
        completed = subprocess.run(
            [str(RUST_BINARY), "init",
             "--n-inputs", str(n_inputs), "--hidden", str(hidden), "--classes", "20",
             "--seed", str(seed), "--epochs", str(epochs), "--n-train", "8156",
             "--weights", str(scratch_weights), "--orders", str(orders)],
            capture_output=True, text=True)
        if completed.returncode != 0:
            raise RuntimeError(f"order generation failed:\n{completed.stdout}\n{completed.stderr}")

    registered_weights = RESULT_ROOT / "initialization" / f"n{n_inputs}-h{hidden}-s{seed}.weights"
    if scratch_weights.is_file() and registered_weights.is_file():
        if scratch_weights.read_bytes() != registered_weights.read_bytes():
            raise RuntimeError(
                f"regenerated weights differ from the registered {registered_weights.name}. "
                "The initialisation lineage has drifted; probe results would not be "
                "comparable to the completed matrix.")
    return orders


def run_probe_cell(contract, geometry, hidden, epochs, seed, out) -> Path:
    n_inputs = 700 if geometry == "channels-700" else 140
    weights = RESULT_ROOT / "initialization" / f"n{n_inputs}-h{hidden}-s{seed}.weights"
    orders = probe_orders(seed, epochs, hidden, n_inputs)
    out.parent.mkdir(parents=True, exist_ok=True)
    command = [
        str(RUST_BINARY), "train-cell",
        "--train-events", str(EVENT_ROOT / "train.events"),
        "--test-events", str(EVENT_ROOT / "test.events"),
        "--contract", contract, "--geometry", geometry,
        "--weights", str(weights), "--orders", str(orders),
        "--epochs", str(epochs), "--out", str(out),
    ]
    print("running:", " ".join(command), flush=True)
    completed = subprocess.run(command, capture_output=True, text=True)
    if completed.returncode != 0:
        raise RuntimeError(f"probe cell failed:\n{completed.stdout}\n{completed.stderr}")
    return out


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    subparsers = parser.add_subparsers(dest="command", required=True)
    r = subparsers.add_parser("readiness")
    r.add_argument("--campaign", choices=("temporal", "ceiling"), default="temporal")
    b = subparsers.add_parser("budget", help="test accuracy vs epoch budget at the anchor")
    b.add_argument("--epochs", type=int, nargs="+", default=[100, 200, 400])
    b.add_argument("--seed", type=int, default=SEEDS[0])
    b.add_argument("--contract", default=ANCHOR["contract"])
    b.add_argument("--geometry", default=ANCHOR["geometry"])
    b.add_argument("--hidden", type=int, default=ANCHOR["hidden"])
    c = subparsers.add_parser("convergence")
    c.add_argument("--cell", type=Path)
    c.add_argument("--run", action="store_true")
    c.add_argument("--contract", default=ANCHOR["contract"])
    c.add_argument("--geometry", default=ANCHOR["geometry"])
    c.add_argument("--hidden", type=int, default=ANCHOR["hidden"])
    c.add_argument("--epochs", type=int, default=ANCHOR["epochs"])
    c.add_argument("--seed", type=int, default=SEEDS[0])
    args = parser.parse_args(argv)

    if args.command == "readiness":
        rows = readiness(args.campaign)
        width = max(len(row["name"]) for row in rows)
        for row in rows:
            marker = {OK: "  ok ", WARN: " warn", FAIL: " FAIL"}[row["status"]]
            print(f"[{marker}] {row['name']:<{width}}  {row['detail']}")
        failures = sum(row["status"] == FAIL for row in rows)
        warnings = sum(row["status"] == WARN for row in rows)
        print(f"\n{len(rows) - failures - warnings} ok, {warnings} warning(s), {failures} failure(s)")
        if failures:
            print("NOT READY - fix the failures above before starting the run.")
        return 1 if failures else 0

    if args.command == "budget":
        # Separates undertraining from overfitting, which the training-loss trace
        # cannot. One-cycle LR is defined over total_steps, so each budget is a
        # properly annealed run rather than a truncation of the longest one.
        rows = []
        for epochs in args.epochs:
            out = RESULT_ROOT / "probe" / (
                f"budget__{args.contract}__{args.geometry}__h{args.hidden}"
                f"__e{epochs}__s{args.seed}.json")
            if not out.is_file():
                run_probe_cell(args.contract, args.geometry, args.hidden,
                               epochs, args.seed, out)
            payload = json.loads(out.read_text())
            trace = [float(v) for v in payload.get("epoch_mean_loss", [])]
            rows.append((epochs, float(payload["accuracy"]), trace[-1] if trace else float("nan"),
                         float(payload.get("wall_secs", 0.0))))
        print(f"\n{'epochs':>7} {'test acc':>9} {'train loss':>11} {'wall':>8}")
        for epochs, accuracy, loss, wall in rows:
            print(f"{epochs:>7} {accuracy:>9.4f} {loss:>11.4f} {wall / 60:>7.1f}m")
        if len(rows) >= 2:
            first, last = rows[0], rows[-1]
            gain = last[1] - first[1]
            print(f"\ntest accuracy {first[0]} -> {last[0]} epochs: {gain:+.4f}")
            if gain > 0.01:
                print("UNDERTRAINED. More budget buys real generalisation; the 0.7151 figure")
                print("  is a budget artefact and the ceiling claim must be withdrawn or")
                print("  re-measured at the longer budget.")
            elif last[2] < first[2] - 1e-6:
                print("OVERFITTING. Training loss keeps falling while test accuracy does not.")
                print("  The budget is sufficient; 0.7151 stands as a ceiling, and the")
                print("  still-falling training loss is not evidence against it.")
            else:
                print("FLAT. Neither more budget nor overfitting; consistent with a ceiling.")
        return 0

    if args.run:
        # A probe run against a stale binary silently produces a cell with no
        # epoch trace, and the ~23 minutes are wasted. Gate on readiness rather
        # than trusting the operator to read it first.
        blocking = [row for row in readiness("temporal")
                    if row["status"] == FAIL and row["name"].startswith(
                        ("rust binary", "binary supports", "event cache",
                         "initialization", "prerequisite"))]
        if blocking:
            print("readiness check failed - refusing to start a probe run:")
            for row in blocking:
                print(f"  FAIL {row['name']}: {row['detail']}")
            print("\nfix the above, then re-run. "
                  "`probe.py readiness` shows the full report.")
            return 1
        out = RESULT_ROOT / "probe" / (
            f"probe__{args.contract}__{args.geometry}__h{args.hidden}"
            f"__e{args.epochs}__s{args.seed}.json")
        cell = run_probe_cell(args.contract, args.geometry, args.hidden,
                              args.epochs, args.seed, out)
    elif args.cell:
        cell = args.cell
    else:
        parser.error("convergence needs --cell or --run")
    return convergence(cell)


if __name__ == "__main__":
    raise SystemExit(main())
