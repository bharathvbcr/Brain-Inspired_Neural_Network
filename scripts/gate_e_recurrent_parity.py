"""Gate E / G7 — cross-backend parity for all four arms, including recurrent.

`GATE_EF_WORK.md` lists G7 as open and `HANDOFF_2026-08-02.md` records that
rust<->python recurrent agreement is **argued, not measured**: both sides are
believed to compute `sum_j w_rec[h,j] * s_j(t-1)`, but nothing compared them.
This measures it.

`gates_ef.py::gate_e` raises "no arm fixtures yet" and then "the python arm
implementations are not wired". The first is fixed by generating the fixtures
(see below); the second is **stale** — `shd_calibration/arms.py` implements
`loss_and_gradient(weights, sample, arm)` for all four arms.

Fixtures are produced by the rust binary:

    target/release/shd-instrument parity --arm <arm> \\
      --events results/shd_instrument_v4/fixtures/events.events --index 3 \\
      --contract published-10ms --geometry channels-700 \\
      --weights <arm weights> --out results/shd_instrument_v4/fixtures/rust-parity-<arm>.json

    .venv-shd/bin/python scripts/gate_e_recurrent_parity.py

# Why this is tolerance-based and Gate F is not

Gate F re-runs the *same* binary and demands bit-identity. This compares two
independent implementations in different languages, so exact equality is not the
right bar and never was: numpy reduces with pairwise summation while the rust
kernel folds sequentially, and the two libm `exp` implementations are not
required to agree in the last ulp.

The fixture is also written with `{:.9}` **fixed-point**, which is lossy for
values well below 1e-9 — a second reason bit-comparison is unavailable here.

The tolerance below is therefore on *relative* deviation with an absolute floor,
and the floor exists to stop near-zero entries dominating the report.
"""

from __future__ import annotations

import json
import struct
import sys
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))

FIXTURES = ROOT / "results" / "shd_instrument_v4" / "fixtures"
ARMS = ("ff+fixed", "ff+alif", "rec+fixed", "rec+alif")

# arms.py cross-checks its own recurrent/adaptive backward against an
# independent scalar loop at ~6e-8 relative deviation, so anything at that
# magnitude is implementation noise rather than disagreement. 1e-5 leaves
# headroom for the two libm `exp` implementations and the fixture's decimal
# rendering without being loose enough to hide a real algebra difference —
# the recurrent aliasing defect fixed on 2026-08-03 produced ~4e-1.
RELATIVE_TOLERANCE = 1e-5

# The fixture is rendered with `{:.9}`, so every stored value sits on a 1e-9
# grid — verified: max deviation from that grid is 4.5e-13, which is float64
# noise, and the smallest nonzero entry is exactly 1.000e-09. A discrepancy of
# one quantum is therefore **below what the fixture can resolve** and is not
# evidence of disagreement.
#
# This is a resolution argument, not a loosened tolerance. It does not hide a
# real defect at any scale that matters: the recurrent aliasing defect fixed on
# 2026-08-03 produced deviations of ~4e-1, eight orders of magnitude above this.
#
# An earlier version used a 1e-6 absolute floor inside the relative scale, which
# reported DIFFERS on entries near 1e-9 where a half-quantum rounding is a 50%
# relative error. That was measuring the fixture's decimal rendering, not the
# backends.
FIXTURE_QUANTUM = 1e-9

COMPARED = ("membrane", "spikes", "rates", "logits", "grad_w_in", "grad_w_out", "grad_b_out")
RECURRENT_COMPARED = ("grad_w_rec",)


def deviation(rust: np.ndarray, python: np.ndarray) -> tuple[float, float, bool]:
    """Return (max absolute, max relative, agrees).

    An entry agrees if it is within one fixture quantum **or** within relative
    tolerance. Both are per-entry: taking array maxima independently would let a
    large-magnitude entry's relative error and a small entry's absolute error be
    judged against each other's criterion.
    """
    rust = np.asarray(rust, dtype=np.float64).ravel()
    python = np.asarray(python, dtype=np.float64).ravel()
    if rust.shape != python.shape:
        raise ValueError(f"shape mismatch: rust {rust.shape} vs python {python.shape}")
    absolute = np.abs(rust - python)
    scale = np.maximum(np.abs(rust), np.abs(python))
    with np.errstate(divide="ignore", invalid="ignore"):
        relative = np.where(scale > 0, absolute / np.maximum(scale, 1e-300), 0.0)
    agrees = (absolute <= FIXTURE_QUANTUM) | (relative <= RELATIVE_TOLERANCE)
    return float(absolute.max()), float(relative.max()), bool(agrees.all())


def main() -> int:
    from shd_calibration import arms as pyarms
    from shd_calibration.data import Contract, frame_events, read_event_cache
    from shd_calibration.model import load_weights

    def load_arm_weights(path: Path, arm: "pyarms.Arm") -> "pyarms.ArmWeights":
        """Read SHDWGT1 or SHDWGT2.

        `model.load_weights` only knows SHDWGT1, which is what `ff+fixed`
        writes. The recurrent and adaptive arms write SHDWGT2 (arm tag, `w_rec`,
        adaptation block) and python has no reader for it, so one is provided
        here rather than widening the shipped loader — that file is inside the
        instrument's provenance fingerprint.
        """
        blob = path.read_bytes()
        if blob[:8] == b"SHDWGT1\0":
            return pyarms.ArmWeights(base=load_weights(path))
        if blob[:8] != b"SHDWGT2\0":
            raise ValueError(f"unknown weight magic in {path.name}: {blob[:8]!r}")
        n_inputs, hidden, n_classes, _code = struct.unpack("<IIII", blob[8:24])
        tau_a, beta_a = struct.unpack("<ff", blob[24:32])
        flat = np.frombuffer(blob[32:], dtype="<f4")
        at = 0

        def take(count: int) -> np.ndarray:
            nonlocal at
            chunk = flat[at:at + count]
            at += count
            return np.array(chunk, dtype=np.float32)

        w_in = take(n_inputs * hidden).reshape(hidden, n_inputs)
        w_out = take(hidden * n_classes).reshape(n_classes, hidden)
        b_out = take(n_classes)
        w_rec = take(hidden * hidden).reshape(hidden, hidden) if arm.recurrent else None
        base = pyarms.Weights(w_in=w_in, w_out=w_out, b_out=b_out)
        return pyarms.ArmWeights(base=base, w_rec=w_rec,
                                 tau_a=np.float32(tau_a), beta_a=np.float32(beta_a))

    rows: list[tuple[str, str, float, float, bool]] = []
    fatal: list[str] = []

    for arm_label in ARMS:
        tag = arm_label.replace("+", "-")
        fixture_path = FIXTURES / f"rust-parity-{tag}.json"
        weights_path = FIXTURES / "arm-init" / f"{tag}.weights"
        if not fixture_path.is_file() or not weights_path.is_file():
            fatal.append(f"{arm_label}: missing {fixture_path.name} or {weights_path.name}")
            continue

        fixture = json.loads(fixture_path.read_text())
        arm = pyarms.Arm.from_label(arm_label)
        weights = load_arm_weights(weights_path, arm)

        samples = read_event_cache(FIXTURES / "events.events", 4)
        framed = frame_events(samples[3], Contract("published", 10), "channels-700")
        forward, gradient = pyarms.loss_and_gradient(weights, framed, arm)

        available = {
            "membrane": forward.membrane,
            "spikes": forward.spikes,
            "rates": forward.rates,
            "logits": forward.logits,
            "grad_w_in": gradient.base.w_in,
            "grad_w_out": gradient.base.w_out,
            "grad_b_out": gradient.base.b_out,
            "grad_w_rec": gradient.w_rec,
        }
        fields = COMPARED + (RECURRENT_COMPARED if arm.recurrent else ())
        for field in fields:
            try:
                absolute, relative, agrees = deviation(fixture[field], available[field])
            except Exception as error:  # noqa: BLE001
                fatal.append(f"{arm_label}/{field}: {error}")
                continue
            rows.append((arm_label, field, absolute, relative, agrees))

    if not rows:
        print("no comparisons ran")
        for line in fatal:
            print(f"  {line}")
        return 1

    print(f"{'arm':<11}{'field':<13}{'max abs':>12}{'max rel':>12}   verdict")
    for arm_label, field, absolute, relative, ok in rows:
        print(f"{arm_label:<11}{field:<13}{absolute:>12.3e}{relative:>12.3e}   "
              f"{'ok' if ok else 'DIFFERS'}")

    failures = [r for r in rows if not r[4]]
    print()
    if fatal:
        print("errors:")
        for line in fatal:
            print(f"  {line}")
    print(f"{len(rows) - len(failures)}/{len(rows)} fields agree "
          f"(within {FIXTURE_QUANTUM:g} absolute or {RELATIVE_TOLERANCE:g} relative, per entry)")
    recurrent = [r for r in rows if r[0].startswith("rec")]
    print(f"recurrent arms: {sum(1 for r in recurrent if r[4])}/{len(recurrent)} fields agree "
          "-- this is the G7 claim that was previously argued rather than measured")
    if failures or fatal:
        print("\nGATE E: FAIL")
        return 1
    print("\nGATE E: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
