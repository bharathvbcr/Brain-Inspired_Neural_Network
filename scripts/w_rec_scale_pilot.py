"""Pilot the recurrent initialisation scale.

`PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION` §5.6 calls for a `W_rec` pilot, and
the initialiser's own comment said the registered scale "is set by the G8 pilot,
not here" — while hard-coding it, so the pilot could not be run. `--w-rec-scale`
now exposes it and this is that pilot.

Motivation is in `MEASUREMENT_2026-08-03_RECURRENT_ARM_STABILITY.md`: at the
default scale, `rec+fixed` reaches an epoch-1 mean gradient norm of 9.8e12 with a
flat loss, and no existing gate reports it, because the value is finite and Adam
normalises the update size.

This is short-horizon and diagnostic. It is looking for the scale at which BPTT
through the recurrent block stays numerically sane — NOT for accuracy. Three
epochs cannot speak to a ceiling and nothing here should be quoted as one.

    .venv-shd/bin/python scripts/w_rec_scale_pilot.py --out-dir DIR

Everything is written under `--out-dir`; the registered `initialization/` and
`cells/` trees are never touched.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_BINARY = ROOT / "target" / "release" / "shd-instrument"
EVENTS = ROOT / "data" / "shd" / "events"

SCALES = (1.0, 0.5, 0.25, 0.1, 0.05)
ARMS = ("rec+fixed", "rec+alif")


def finite(series: list[float | None]) -> list[float]:
    """Map JSON `null` to infinity.

    The instrument writes `null` where a summary overflowed to a non-finite
    value — see the `json_f64` guard, added because `inf` is not valid JSON and
    a diverging cell was writing a file nothing could read. `null` therefore
    means "too large to represent", and infinity is the honest reading of it for
    a diagnostic that is looking for explosions. Silently dropping the entry
    would understate the very thing this pilot measures.
    """
    return [float("inf") if value is None else float(value) for value in series]


def run(command: list[str]) -> None:
    completed = subprocess.run(command, capture_output=True, text=True)
    if completed.returncode != 0:
        raise RuntimeError(f"{command[1]} failed:\n{completed.stdout}\n{completed.stderr}")


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--binary", type=Path, default=DEFAULT_BINARY)
    parser.add_argument("--out-dir", type=Path, required=True)
    parser.add_argument("--hidden", type=int, default=128)
    parser.add_argument("--epochs", type=int, default=3)
    parser.add_argument("--seed", type=int, default=5170001)
    parser.add_argument("--contract", default="published-2ms")
    parser.add_argument("--geometry", default="adjacent-sum-5")
    args = parser.parse_args(argv)

    weights_dir = args.out_dir / "init"
    cells_dir = args.out_dir / "cells"
    weights_dir.mkdir(parents=True, exist_ok=True)
    cells_dir.mkdir(parents=True, exist_ok=True)
    n_inputs = 700 if args.geometry == "channels-700" else 140

    orders = weights_dir / f"orders-s{args.seed}.orders"
    rows = []
    failures: list[tuple[str, float, str]] = []

    for arm in ARMS:
        for scale in SCALES:
            tag = f"{arm.replace('+', '-')}__scale{scale}"
            weights = weights_dir / f"{tag}.weights"
            run([
                str(args.binary), "init",
                "--n-inputs", str(n_inputs), "--hidden", str(args.hidden),
                "--classes", "20", "--seed", str(args.seed),
                "--epochs", str(args.epochs), "--n-train", "8156",
                "--arm", arm, "--w-rec-scale", str(scale),
                "--weights", str(weights), "--orders", str(orders),
            ])
            cell = cells_dir / f"{tag}.json"
            # A cell that dies is data, not an accident: on seed 5170002 the
            # instrument wrote unparseable JSON, and on 5170003 it aborted at
            # optimizer step 52. Both are findings about the recurrent arm. An
            # early version of this script raised on the first of them and threw
            # away the other 19 cells of the sweep, which is the wrong trade for
            # a diagnostic — record the failure and keep going.
            try:
                run([
                    str(args.binary), "train-cell",
                    "--train-events", str(EVENTS / "train.events"),
                    "--test-events", str(EVENTS / "test.events"),
                    "--contract", args.contract, "--geometry", args.geometry,
                    "--weights", str(weights), "--orders", str(orders),
                    "--epochs", str(args.epochs), "--arm", arm,
                    "--out", str(cell),
                ])
                payload = json.loads(cell.read_text())
            except RuntimeError as error:
                reason = str(error).strip().splitlines()[-1] if str(error).strip() else "aborted"
                failures.append((arm, scale, f"ABORTED: {reason}"))
                print(f"[FAIL] {tag}  {reason}")
                continue
            except json.JSONDecodeError:
                failures.append((arm, scale, "UNPARSEABLE CELL (non-finite value written)"))
                print(f"[FAIL] {tag}  wrote a cell that is not valid JSON")
                continue
            norms = finite(payload["epoch_mean_gradient_norm"])
            # Present only on cells written after the max-gradient telemetry
            # landed; older cells fall back to the mean so this stays readable
            # rather than crashing on an artifact from yesterday.
            maxima = finite(payload.get("epoch_max_gradient_norm", norms))
            losses = payload["epoch_mean_loss"]
            rows.append((arm, scale, payload["accuracy"], losses, norms, maxima,
                         payload["mean_firing_rate"], payload["silent_fraction"]))
            print(f"[ ok ] {tag}  meanGrad={max(norms):.3g}  maxGrad={max(maxima):.3g}"
                  f"  acc={payload['accuracy']:.4f}")

    print(f"\n{args.epochs} epochs, h{args.hidden}, {args.contract}/{args.geometry}, "
          f"seed {args.seed}. Diagnostic only — not an accuracy measurement.\n")
    print(f"{'arm':<11}{'scale':>7}{'peak mean':>12}{'peak max':>12}{'max/mean':>10}"
          f"{'loss drop':>11}{'fire':>7}{'acc':>8}")
    for arm, scale, accuracy, losses, norms, maxima, fire, silent in rows:
        # A falling loss with a bounded gradient is the only combination that
        # means the cell measured anything at all.
        #
        # `max/mean` is the discriminator this pilot was re-run for. There are
        # 32 optimizer steps per epoch, so a ratio near 1 means every step is
        # equally bad — clip. A ratio near 32 means one step carries the whole
        # epoch mean, and the question becomes what is in that batch.
        peak_mean, peak_max = max(norms), max(maxima)
        ratio = peak_max / peak_mean if peak_mean > 0 else float("nan")
        print(f"{arm:<11}{scale:>7}{peak_mean:>12.3g}{peak_max:>12.3g}{ratio:>10.1f}"
              f"{losses[0] - losses[-1]:>11.4f}{fire:>7.3f}{accuracy:>8.4f}")

    for arm, scale, reason in failures:
        print(f"{arm:<11}{scale:>7}   {reason}")

    print("\nA usable scale is one where peak |grad| is bounded AND loss drop > 0.")
    print("max/mean near 1 -> uniformly large gradients; near 32 -> one bad batch per epoch.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
