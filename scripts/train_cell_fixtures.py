"""Characterisation fixtures for the `shd-instrument train-cell` path.

A change to a training kernel (attention, substrate, readout, optimiser) is only
as safe as the cells it reproduces. Gate F (`gate_f_rust.py`) regresses
*recorded* 20-epoch cells, which takes hours and covers one attention arm.
This script is the quick companion: a matrix of 2-epoch cells chosen to reach
different code paths and vector tails, whose final-weights hashes were recorded
with a known-good binary. A later binary must reproduce every one byte-for-byte.

    python3 scripts/train_cell_fixtures.py --binary target/release/shd-instrument
    python3 scripts/train_cell_fixtures.py --binary ... --record     # (re)write the json
    python3 scripts/train_cell_fixtures.py --binary ... --only oracle --jobs 3
    python3 scripts/train_cell_fixtures.py --binary ... --numerics blas --record
    python3 scripts/train_cell_fixtures.py --binary ... --numerics blas

`--numerics exact` (the default) is the matrix above, unchanged. `--numerics
blas` runs the same 23 fixtures with `--numerics blas` passed to `train-cell`
and reads/writes `train_cell_fixtures_blas.json`. Its expected hashes ship as
null until a reference build records them: a null hash is `ERROR`, never `PASS`
and never a comparison. The pinned oracle weights hash belongs to exact mode
(blas legitimately differs in the last bits), so `--record` in blas mode checks
instead that every fixture's cell says `numerics: blas` with a platform, and
that every cell agrees on that platform, which is stored beside the hashes.

`--events` defaults to `data/shd/events` under this checkout; a git worktree has
no `data/` (it is not tracked), so pass the main checkout's directory there. A
missing event file makes every fixture `ERROR`, never a pass.

What is hashed, per fixture:

* `weights_sha256` - the file written by `--save-final-weights`. This is the
  bit-exactness claim: every spike, every Adam step, every float.
* `cell_sha256` - the cell json with the clock fields removed
  (`NONDETERMINISTIC_CELL_FIELDS`). It covers the measurements the weights do
  not (accuracy, losses, gradient-norm traces, audit counters).

Outcomes are distinct on purpose. A fixture that could not run (non-zero exit,
timeout, missing output, unreadable json) is `ERROR`/`TIMEOUT`, never `PASS`,
and a fixture with no recorded hash is `UNRECORDED`, never `PASS`. Exit status
is 0 only when every selected fixture is `PASS`.

The oracle fixture's weights hash is also pinned in this file
(`ORACLE_WEIGHTS_SHA256`, from the 2026-11-09 kernel round): `--record` refuses
to write anything if the binary does not reproduce it, so a mis-built binary
cannot become the new truth.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import signal
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FIXTURE_JSON = Path(__file__).resolve().with_suffix(".json")
FIXTURE_JSON_BLAS = FIXTURE_JSON.with_name(FIXTURE_JSON.stem + "_blas.json")
NUMERICS_MODES = ("exact", "blas")
DEFAULT_NUMERICS = "exact"
DEFAULT_EVENTS = ROOT / "data" / "shd" / "events"
DEFAULT_OUT = ROOT / "target" / "train_cell_fixtures"
SCHEMA = "train-cell-fixtures-v1"
SEED = 5170001
EPOCHS = 2
CLASSES = 20
#: Whole-corpus training set (Gate F cells) and the set left after holding out
#: speakers 3 and 6 (the attention cells).
N_TRAIN_FULL = 8156
N_TRAIN_SPLIT = 6987
VAL_SPEAKERS = "3,6"

ORACLE_NAME = "oracle-ff-fixed-attn-d32l4-p2ms"
ORACLE_WEIGHTS_SHA256 = "9c364bd9441d1406cd97aa15078a83c1042b93180963f6bf159652528df512d3"

#: Wall-clock fields of a cell json. Everything else in it is a measurement.
NONDETERMINISTIC_CELL_FIELDS = ("wall_secs", "emitted_unix_s", "emitted_utc")

DEFAULT_TIMEOUT_S = 3600.0

PASS = "PASS"
MISMATCH = "MISMATCH"
ERROR = "ERROR"
TIMEOUT = "TIMEOUT"
UNRECORDED = "UNRECORDED"
#: blas only: the recorded hashes come from another OS build than this run's, so
#: the comparison itself is meaningless. Not a MISMATCH (nothing was compared)
#: and never a PASS.
NOT_COMPARABLE = "NOT_COMPARABLE"


@dataclass(frozen=True)
class Fixture:
    name: str
    arm: str = "ff+fixed"
    hidden: int = 128
    geometry: str = "adjacent-sum-5"
    contract: str = "published-2ms"
    attn_dim: int | None = None
    attn_layers: int | None = None
    readout: str | None = None
    temporal: str | None = None
    #: Held-out speakers (the attention cells) or the whole corpus (Gate F cells).
    speaker_split: bool = False

    @property
    def n_inputs(self) -> int:
        return 700 if self.geometry == "channels-700" else 140

    @property
    def n_train(self) -> int:
        return N_TRAIN_SPLIT if self.speaker_split else N_TRAIN_FULL

    @property
    def attention(self) -> bool:
        return self.arm.endswith("+attn")


def _attn(name: str, **kw: object) -> Fixture:
    kw.setdefault("arm", "ff+fixed+attn")
    kw.setdefault("attn_dim", 32)
    kw.setdefault("attn_layers", 4)
    return Fixture(name=name, speaker_split=True, **kw)  # type: ignore[arg-type]


FIXTURES: tuple[Fixture, ...] = (
    # The oracle: the cell every kernel round has been reproduced against.
    _attn(ORACLE_NAME),
    # Every ReadoutVariant the CLI exposes (--attn-readout).
    _attn("attn-readout-no-position", readout="no-position"),
    _attn("attn-readout-qk-norm", readout="qk-norm"),
    _attn("attn-readout-no-position-qk-norm", readout="no-position+qk-norm"),
    # Training-data manipulations (--temporal) that change the attention input.
    _attn("attn-hidden-shuffled", temporal="hidden-shuffled"),
    _attn("attn-bin-shuffled", temporal="bin-shuffled"),
    # Attention on the other substrates.
    _attn("rec-fixed-attn-d32l4", arm="rec+fixed+attn"),
    _attn("ff-alif-attn-d32l4", arm="ff+alif+attn"),
    _attn("rec-alif-attn-d32l4", arm="rec+alif+attn"),
    # Model widths: not a multiple of 16 (24), of 4 (30), and layer depth 1.
    _attn("attn-d24l1", attn_dim=24, attn_layers=1),
    _attn("attn-d30l4", attn_dim=30),
    _attn("attn-d32l1", attn_layers=1),
    # Contracts that give different T (published-2ms is the oracle's).
    _attn("attn-fixed-t100", contract="fixed-t100"),
    _attn("attn-published-4ms", contract="published-4ms"),
    _attn("attn-published-10ms", contract="published-10ms"),
    # A different input width.
    _attn("attn-channels-700", geometry="channels-700"),
    # Base (non-attention) arms the Gate F cells use.
    Fixture("base-h512-channels-700-fixed-t500", hidden=512, geometry="channels-700",
            contract="fixed-t500"),
    Fixture("base-h512-channels-700-published-2ms", hidden=512, geometry="channels-700"),
    Fixture("base-h128-adjacent-sum-5-fixed-t100", contract="fixed-t100"),
    Fixture("base-h128-adjacent-sum-5-fixed-t250", contract="fixed-t250"),
    Fixture("base-ff-alif-fixed-t100", arm="ff+alif", contract="fixed-t100"),
    Fixture("base-rec-fixed-p2ms", arm="rec+fixed"),
    Fixture("base-rec-alif-p2ms", arm="rec+alif"),
)


# --------------------------------------------------------------------------- pure logic


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def canonical_cell_sha256(cell_json_text: str) -> str:
    """Hash of a cell json with its clock fields removed.

    Raises ValueError for text that is not a json object, and when none of the
    clock fields were present - a cell with no `wall_secs` is not a cell this
    instrument wrote, and hashing it would hide that.
    """
    try:
        cell = json.loads(cell_json_text)
    except json.JSONDecodeError as exc:
        raise ValueError(f"cell is not readable json: {exc}") from exc
    if not isinstance(cell, dict):
        raise ValueError("cell json is not an object")
    if not any(field in cell for field in NONDETERMINISTIC_CELL_FIELDS):
        raise ValueError("cell carries none of the clock fields; not an instrument cell")
    kept = {k: v for k, v in cell.items() if k not in NONDETERMINISTIC_CELL_FIELDS}
    return sha256_bytes(json.dumps(kept, sort_keys=True, separators=(",", ":")).encode())


def init_command(fx: Fixture, binary: Path, work: Path) -> list[str]:
    cmd = [
        str(binary), "init",
        "--n-inputs", str(fx.n_inputs),
        "--hidden", str(fx.hidden),
        "--classes", str(CLASSES),
        "--seed", str(SEED),
        "--epochs", str(EPOCHS),
        "--n-train", str(fx.n_train),
        "--arm", fx.arm,
        "--weights", str(work / "init.weights"),
        "--orders", str(work / "orders.bin"),
    ]
    if fx.attention:
        cmd += ["--attn-dim", str(fx.attn_dim), "--attn-layers", str(fx.attn_layers)]
        if fx.readout:
            cmd += ["--attn-readout", fx.readout]
    return cmd


def fixtures_path(numerics: str) -> Path:
    """The recorded-hash file for a mode: exact and blas never share one."""
    return FIXTURE_JSON_BLAS if numerics == "blas" else FIXTURE_JSON


def train_command(fx: Fixture, binary: Path, work: Path, events: Path,
                  numerics: str = DEFAULT_NUMERICS) -> list[str]:
    cmd = [
        str(binary), "train-cell",
        "--train-events", str(events / "train.events"),
        "--test-events", str(events / "test.events"),
        "--contract", fx.contract,
        "--geometry", fx.geometry,
        "--weights", str(work / "init.weights"),
        "--orders", str(work / "orders.bin"),
        "--epochs", str(EPOCHS),
        "--arm", fx.arm,
        "--out", str(work / "cell.json"),
        "--seed", str(SEED),
        "--save-final-weights", str(work / "final.weights"),
    ]
    if fx.speaker_split:
        cmd += ["--train-speakers", str(events / "train.speakers"),
                "--val-speakers", VAL_SPEAKERS]
    if fx.temporal:
        # The instrument refuses a manipulated run without an explicit seed.
        cmd += ["--temporal", fx.temporal, "--temporal-seed", str(SEED)]
    # Only for blas: the exact command line is what it always was.
    if numerics == "blas":
        cmd += ["--numerics", "blas"]
    return cmd


def judge(name: str, observed: dict[str, str], recorded: dict[str, dict[str, str]],
          platforms: tuple[object, object] | None = None) -> dict:
    """Compare one observation with the recorded hashes.

    `platforms` is (recorded, observed) `numerics_platform`, given only in blas
    mode (exact hashes are platform-independent and carry no platform). A
    differing platform is `NOT_COMPARABLE`, decided before any hash is looked
    at; a recorded platform that is missing cannot be compared and is `ERROR`.
    """
    if name not in recorded:
        return {"status": UNRECORDED, "detail": "no recorded hash for this fixture"}
    want = recorded[name]
    # A fixture that is listed but carries null hashes (the blas file before a
    # reference build records it) has nothing to compare against. That is not a
    # mismatch and above all not a pass: it could not be judged.
    keys = ("weights_sha256", "cell_sha256")
    if not isinstance(want, dict) or any(not isinstance(want.get(k), str) or not want[k]
                                         for k in keys):
        return {"status": ERROR,
                "detail": "the recorded hashes for this fixture are null or missing; "
                          "record them with a reference build (--record) before comparing"}
    if platforms is not None:
        rec_platform, obs_platform = platforms
        if not (isinstance(rec_platform, str) and rec_platform.strip()):
            return {"status": ERROR,
                    "detail": "the recorded blas file names no numerics_platform, so the "
                              "hashes cannot be tied to this platform; re-record it"}
        if rec_platform != obs_platform:
            return {"status": NOT_COMPARABLE,
                    "detail": f"platform differs: recorded {rec_platform!r}, observed "
                              f"{obs_platform!r}; blas hashes are only meaningful on the "
                              "OS build that recorded them (hashes not compared)"}
    bad = {
        key: {"recorded": want.get(key), "observed": observed.get(key)}
        for key in ("weights_sha256", "cell_sha256")
        if want.get(key) != observed.get(key)
    }
    if bad:
        return {"status": MISMATCH, "detail": bad}
    return {"status": PASS, "detail": {}}


def verdict_exit_code(statuses: list[str]) -> int:
    """0 only when there is something and all of it passed."""
    return 0 if statuses and all(s == PASS for s in statuses) else 1


def stale_recorded(recorded: dict, fixtures: tuple[Fixture, ...]) -> list[str]:
    """Recorded names that no fixture defines any more."""
    return sorted(set(recorded) - {fx.name for fx in fixtures})


def select(fixtures: tuple[Fixture, ...], only: list[str] | None) -> tuple[Fixture, ...]:
    if not only:
        return fixtures
    known = {fx.name for fx in fixtures}
    unknown = [name for name in only if name not in known]
    if unknown:
        raise SystemExit(f"unknown fixture(s): {', '.join(unknown)}")
    return tuple(fx for fx in fixtures if fx.name in set(only))


# --------------------------------------------------------------------------- running


def run_bounded(command: list[str], timeout_s: float, log: Path) -> int:
    """Run `command` in its own process group; SIGKILL the group on expiry.

    Returns the exit status. Raises subprocess.TimeoutExpired on expiry. Output
    goes to `log`, never a pipe, so a grandchild cannot hold us open.
    """
    with log.open("wb") as sink:
        child = subprocess.Popen(
            command, stdin=subprocess.DEVNULL, stdout=sink, stderr=subprocess.STDOUT,
            start_new_session=True,
        )
        try:
            return child.wait(timeout=timeout_s)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(os.getpgid(child.pid), signal.SIGKILL)
            except (ProcessLookupError, PermissionError):
                child.kill()
            try:
                child.wait(timeout=30)
            except subprocess.TimeoutExpired:
                pass
            raise


def numerics_of_cell(cell_json_text: str) -> tuple[object, object]:
    """(`numerics`, `numerics_platform`) a cell declares; absent numerics is exact."""
    cell = json.loads(cell_json_text)
    return cell.get("numerics", "exact"), cell.get("numerics_platform")


def run_fixture(fx: Fixture, binary: Path, out: Path, events: Path,
                timeout_s: float, numerics: str = DEFAULT_NUMERICS) -> dict:
    started = time.monotonic()
    work = out / fx.name
    work.mkdir(parents=True, exist_ok=True)
    # Remove last run's outputs: an instrument that exits 0 without writing must
    # not be judged on the previous run's files.
    for leftover in ("init.weights", "orders.bin", "cell.json", "final.weights"):
        (work / leftover).unlink(missing_ok=True)

    def result(status: str, detail: object, observed: dict | None = None,
               platform: object = None) -> dict:
        return {"name": fx.name, "status": status, "detail": detail,
                "observed": observed or {}, "platform": platform,
                "secs": round(time.monotonic() - started, 1)}

    deadline = started + timeout_s
    for label, command in (("init", init_command(fx, binary, work)),
                           ("train-cell", train_command(fx, binary, work, events, numerics))):
        remaining = max(1.0, deadline - time.monotonic())
        try:
            code = run_bounded(command, remaining, work / f"{label}.log")
        except subprocess.TimeoutExpired:
            return result(TIMEOUT, f"{label} exceeded the {timeout_s:.0f}s budget")
        except OSError as exc:
            return result(ERROR, f"could not start {label}: {exc}")
        if code != 0:
            tail = (work / f"{label}.log").read_text(errors="replace")[-600:]
            return result(ERROR, f"{label} exited {code}: {tail}")
    for needed in ("final.weights", "cell.json"):
        path = work / needed
        if not path.is_file() or path.stat().st_size == 0:
            return result(ERROR, f"train-cell exited 0 without writing {needed}")
    try:
        cell_text = (work / "cell.json").read_text()
        observed = {
            "weights_sha256": sha256_file(work / "final.weights"),
            "cell_sha256": canonical_cell_sha256(cell_text),
        }
        got, platform = numerics_of_cell(cell_text)
    except (OSError, ValueError) as exc:
        return result(ERROR, f"could not hash the outputs: {exc}")
    # The binary must have run the arithmetic that was asked for. One that
    # ignores or lacks the flag would otherwise be hashed (and recorded, or
    # compared) as the wrong mode. Exact accepts an absent key: a binary from
    # before the flag is exact by definition.
    if got != numerics:
        return result(ERROR, f"asked for numerics={numerics!r} but the cell says {got!r}")
    if numerics == "blas" and not (isinstance(platform, str) and platform.strip()):
        return result(ERROR, "a blas cell carries no numerics_platform")
    return result("RAN", "", observed, platform)


def load_recorded(path: Path, numerics: str = DEFAULT_NUMERICS) -> dict[str, dict[str, str]]:
    payload = json.loads(path.read_text())
    if payload.get("schema") != SCHEMA:
        raise SystemExit(f"{path}: schema is not {SCHEMA}")
    # The exact file has no `numerics` key (it predates the flag): absent is
    # exact. Hashes from one mode must never be compared against the other.
    recorded_mode = payload.get("numerics", "exact")
    if recorded_mode != numerics:
        raise ValueError(f"{path} records numerics={recorded_mode!r}, not {numerics!r}")
    return payload["fixtures"]


def load_recorded_platform(path: Path) -> object:
    """The `numerics_platform` a blas fixture file was recorded on (None if absent)."""
    return json.loads(path.read_text()).get("numerics_platform")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--binary", required=True, type=Path, help="shd-instrument binary")
    parser.add_argument("--record", action="store_true", help="write the expected hashes")
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT, help="scratch directory")
    parser.add_argument("--events", type=Path, default=DEFAULT_EVENTS)
    parser.add_argument("--numerics", choices=NUMERICS_MODES, default=DEFAULT_NUMERICS,
                        help="arithmetic mode: exact (default) or blas")
    parser.add_argument("--fixtures-json", type=Path, default=None,
                        help="recorded hashes (default: train_cell_fixtures.json for exact, "
                             "train_cell_fixtures_blas.json for blas)")
    parser.add_argument("--only", action="append", help="run just this fixture (repeatable)")
    parser.add_argument("--jobs", type=int, default=1, help="fixtures run concurrently")
    parser.add_argument("--timeout", type=float, default=DEFAULT_TIMEOUT_S,
                        help="seconds one fixture (init + train-cell) may take")
    parser.add_argument("--list", action="store_true", help="list fixtures and exit")
    args = parser.parse_args(argv)

    if args.list:
        for fx in FIXTURES:
            print(fx.name)
        return 0
    if args.fixtures_json is None:
        args.fixtures_json = fixtures_path(args.numerics)
    if args.jobs < 1:
        parser.error("--jobs must be >= 1")
    if args.record and args.only:
        parser.error("--record writes the whole matrix; it cannot be combined with --only")
    binary = args.binary.resolve()
    if not binary.is_file() or not os.access(binary, os.X_OK):
        print(f"ERROR: binary {binary} is not an executable file", file=sys.stderr)
        return 2
    fixtures = select(FIXTURES, args.only)
    recorded: dict[str, dict[str, str]] = {}
    recorded_platform: object = None
    if not args.record:
        try:
            recorded = load_recorded(args.fixtures_json, args.numerics)
            if args.numerics == "blas":
                recorded_platform = load_recorded_platform(args.fixtures_json)
        except (OSError, ValueError, KeyError) as exc:
            print(f"ERROR: cannot read {args.fixtures_json}: {exc}", file=sys.stderr)
            return 2
    args.out.mkdir(parents=True, exist_ok=True)

    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        runs = list(pool.map(
            lambda fx: run_fixture(fx, binary, args.out, args.events, args.timeout,
                                 args.numerics), fixtures))

    rows = []
    for run in runs:
        if run["status"] == "RAN":
            verdict = {"status": "RAN", "detail": {}} if args.record else judge(
                run["name"], run["observed"], recorded,
                (recorded_platform, run["platform"]) if args.numerics == "blas" else None)
            run = {**run, **verdict}
        rows.append(run)
        print(f"[{run['status']:<10}] {run['name']:<42} {run['secs']:>7.1f}s", flush=True)
        if run["status"] not in (PASS, "RAN"):
            print(f"             {run['detail']}", flush=True)

    if args.record:
        if any(r["status"] != "RAN" for r in rows):
            print("FAIL: not recording; some fixtures did not run", file=sys.stderr)
            return 1
        extra: dict[str, object] = {}
        if args.numerics == "exact":
            oracle = next((r for r in rows if r["name"] == ORACLE_NAME), None)
            if oracle is None or oracle["observed"]["weights_sha256"] != ORACLE_WEIGHTS_SHA256:
                got = oracle["observed"]["weights_sha256"] if oracle else None
                print(f"FAIL: the oracle weights hash is {got}, not {ORACLE_WEIGHTS_SHA256}; "
                      "this binary is not the reference, nothing recorded", file=sys.stderr)
                return 1
        else:
            # No pinned reference exists for blas, so the guard is consistency:
            # one platform for the whole matrix, because the hashes are only
            # meaningful on the OS build that produced them.
            platforms = sorted({str(r["platform"]) for r in rows})
            if len(platforms) != 1:
                print(f"FAIL: fixtures ran on {len(platforms)} platforms "
                      f"({', '.join(platforms)}); nothing recorded", file=sys.stderr)
                return 1
            extra = {"numerics": "blas", "numerics_platform": platforms[0]}
        payload = {
            "schema": SCHEMA,
            **extra,
            "seed": SEED,
            "epochs": EPOCHS,
            "fixtures": {r["name"]: r["observed"] for r in rows},
        }
        args.fixtures_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
        print(f"recorded {len(rows)} fixtures to {args.fixtures_json}")
        return 0

    stale = [] if args.only else stale_recorded(recorded, FIXTURES)
    if stale:
        print(f"FAIL: recorded fixtures with no definition: {', '.join(stale)}", file=sys.stderr)
    statuses = [r["status"] for r in rows]
    code = verdict_exit_code(statuses)
    if stale:
        code = 1
    ran = sum(s in (PASS, MISMATCH) for s in statuses)
    print(f"{statuses.count(PASS)}/{len(rows)} pass; {ran} compared, "
          f"{len(rows) - ran} could not be judged"
          + (" (subset: --only)" if args.only else ""))
    return code


if __name__ == "__main__":
    sys.exit(main())
