"""Replicate the temporal-sensitivity positive control across seeds and widths.

`MEASUREMENT_2026-08-03_TEMPORAL_SENSITIVITY_POSITIVE_CONTROL.md` caveat 3 says
the control ran at a single seed, a single width and 256 test samples — enough
to answer "does the channel exist", not enough to say anything quantitative
about how large the effect is. This sweep discharges that caveat by replicating
across the registered initialisations.

It is a harness-validation run: nothing is trained, no accuracy is produced, and
the registered `initialization/` artifacts are read and never written.

    .venv-shd/bin/python scripts/temporal_sensitivity_sweep.py --out-dir DIR

Gate 5.0 of `PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION` requires
`mean_membrane_rel_l2 >= 0.1` and `mean_spike_hamming > 0` for every
non-identity condition. This reports the *minimum* across the whole sweep, so a
single weak cell cannot hide behind an average.
"""

from __future__ import annotations

import argparse
import json
import statistics
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RESULT_ROOT = ROOT / "results" / "shd_instrument_v4"
INITIALIZATION = RESULT_ROOT / "initialization"
DEFAULT_BINARY = ROOT / "target" / "release" / "shd-instrument"
TEST_EVENTS = ROOT / "data" / "shd" / "events" / "test.events"

GATE_5_0_MEMBRANE_FLOOR = 0.1
SEEDS = (5170001, 5170002, 5170003)
WIDTHS = (128, 256, 512)
GEOMETRIES = ("adjacent-sum-5", "channels-700")


def n_inputs_for(geometry: str) -> int:
    return 700 if geometry == "channels-700" else 140


def run_one(
    binary: Path, contract: str, geometry: str, width: int, seed: int, samples: int, out: Path
) -> dict[str, object]:
    weights = INITIALIZATION / f"n{n_inputs_for(geometry)}-h{width}-s{seed}.weights"
    if not weights.is_file():
        raise FileNotFoundError(f"missing registered initialization: {weights}")
    completed = subprocess.run(
        [
            str(binary), "temporal-sensitivity",
            "--test-events", str(TEST_EVENTS),
            "--contract", contract,
            "--geometry", geometry,
            "--weights", str(weights),
            "--samples", str(samples),
            "--out", str(out),
        ],
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        raise RuntimeError(f"probe failed:\n{completed.stdout}\n{completed.stderr}")
    return json.loads(out.read_text())


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--contract", default="published-2ms")
    parser.add_argument("--samples", type=int, default=256)
    parser.add_argument("--binary", type=Path, default=DEFAULT_BINARY)
    parser.add_argument(
        "--out-dir", type=Path, default=RESULT_ROOT / "temporal-sensitivity" / "sweep"
    )
    args = parser.parse_args(argv)
    args.out_dir.mkdir(parents=True, exist_ok=True)

    by_condition: dict[str, list[tuple[str, float, float]]] = {}
    identity_violations = []
    cells = 0

    for geometry in GEOMETRIES:
        for width in WIDTHS:
            for seed in SEEDS:
                tag = f"{args.contract}__{geometry}__h{width}__s{seed}"
                payload = run_one(
                    args.binary, args.contract, geometry, width, seed,
                    args.samples, args.out_dir / f"{tag}.json",
                )
                cells += 1
                for row in payload["conditions"]:
                    name = row["condition"]
                    hamming = float(row["mean_spike_hamming"])
                    membrane = float(row["mean_membrane_rel_l2"])
                    if name == "intact":
                        # Control on the control: the identity condition must be
                        # an exact identity, or the comparison harness itself is
                        # manufacturing divergence and nothing below means anything.
                        if hamming != 0.0 or membrane != 0.0:
                            identity_violations.append((tag, hamming, membrane))
                        continue
                    by_condition.setdefault(name, []).append((tag, hamming, membrane))
                print(f"[ ok ] {tag}")

    print(f"\n{cells} configurations, {args.samples} test samples each\n")
    print(f"{'condition':<18}{'spikeHam min':>14}{'median':>10}{'max':>10}"
          f"{'membL2 min':>13}{'median':>10}{'max':>10}")
    worst_membrane = float("inf")
    for name, rows in sorted(by_condition.items()):
        hammings = sorted(r[1] for r in rows)
        membranes = sorted(r[2] for r in rows)
        worst_membrane = min(worst_membrane, membranes[0])
        print(f"{name:<18}{hammings[0]:>14.4f}{statistics.median(hammings):>10.4f}"
              f"{hammings[-1]:>10.4f}{membranes[0]:>13.4f}"
              f"{statistics.median(membranes):>10.4f}{membranes[-1]:>10.4f}")

    zero_hamming = [
        (name, tag) for name, rows in by_condition.items() for tag, ham, _ in rows if ham <= 0.0
    ]
    failed = bool(identity_violations or zero_hamming) or worst_membrane < GATE_5_0_MEMBRANE_FLOOR

    print()
    for tag, hamming, membrane in identity_violations:
        print(f"IDENTITY VIOLATION {tag}: hamming={hamming} membrane={membrane}")
    for name, tag in zero_hamming:
        print(f"ZERO HAMMING {name} at {tag}")
    print(
        f"gate 5.0 floor {GATE_5_0_MEMBRANE_FLOOR}: worst membrane rel L2 over the whole sweep "
        f"is {worst_membrane:.4f} -> {'FAIL' if failed else 'PASS'}"
    )
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
