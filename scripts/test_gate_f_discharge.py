#!/usr/bin/env python3
"""Provenance may only be discharged by a Gate F run that actually judged cells.

`gate_f_discharge` decides whether a binary hash is backed by evidence that it
reproduces recorded cells. Gate F can now report cells it could not run at all —
a timeout, a crashed instrument, a missing artifact — and such a cell says
nothing about the kernel. It must never be counted as evidence.

`scripts/shd_calibration/runner.py` imports h5py at module scope for the data
path, which is not installed here and which this function never touches, so the
import is stubbed. Without that this logic is simply untestable in this
environment, which is how it came to be hardened untested.
"""

from __future__ import annotations

import json
import sys
import tempfile
import types
from pathlib import Path

sys.modules.setdefault("h5py", types.ModuleType("h5py"))
sys.path.insert(0, str(Path(__file__).resolve().parent))
from shd_calibration import runner  # noqa: E402

SHA = "a" * 64

# Eight cells over two geometries and two widths — the breadth the amendment
# requires, so that a refusal in a test is never just "too narrow".
def cells(n: int = 8) -> list[dict[str, str]]:
    out = []
    for i in range(n):
        geometry = "adjacent-sum-5" if i % 2 else "channels-700"
        hidden = "h128" if i % 4 < 2 else "h256"
        out.append({"cell": f"rust__fixed-t100__{geometry}__{hidden}__e20__s517000{i}",
                    "status": "BIT_IDENTICAL"})
    return out


def record(**over) -> dict[str, object]:
    base = {"binary_sha256": SHA, "status": "PASS", "failures": 0,
            "unrunnable": 0, "results": cells()}
    base.update(over)
    return base


class History:
    def __init__(self, *records):
        self.dir = Path(tempfile.mkdtemp(prefix="discharge-"))
        (self.dir / "gate-f-rust").mkdir(parents=True)
        (self.dir / "gate-f-rust" / "runs.jsonl").write_text(
            "".join(json.dumps(r) + "\n" for r in records))
        self.saved = runner.RESULT_ROOT
        runner.RESULT_ROOT = self.dir

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        runner.RESULT_ROOT = self.saved
        import shutil
        shutil.rmtree(self.dir, ignore_errors=True)


def a_clean_pass_discharges():
    with History(record()):
        assert runner.gate_f_discharge(SHA) is not None


def a_legacy_record_without_the_key_still_discharges():
    """Every record written before the key existed must keep its meaning."""
    legacy = record()
    del legacy["unrunnable"]
    with History(legacy):
        assert runner.gate_f_discharge(SHA) is not None, (
            "hardening must not retroactively invalidate the existing record"
        )


def a_pass_carrying_unrunnable_cells_does_not_discharge():
    """The new line. A cell that could not run is not evidence."""
    with History(record(unrunnable=2)):
        assert runner.gate_f_discharge(SHA) is None, (
            "a run with cells the gate never judged was accepted as proof"
        )


def an_incomplete_run_does_not_discharge():
    with History(record(status="INCOMPLETE", unrunnable=1)):
        assert runner.gate_f_discharge(SHA) is None


def a_failing_run_does_not_discharge():
    with History(record(status="FAIL", failures=1)):
        assert runner.gate_f_discharge(SHA) is None


def a_run_of_another_binary_does_not_discharge():
    with History(record(binary_sha256="b" * 64)):
        assert runner.gate_f_discharge(SHA) is None


def too_few_cells_does_not_discharge():
    with History(record(results=cells(runner.PROVENANCE_MIN_GATE_F_CELLS - 1))):
        assert runner.gate_f_discharge(SHA) is None


def one_geometry_does_not_discharge():
    narrow = [{"cell": f"rust__fixed-t100__adjacent-sum-5__h{128 if i % 2 else 256}__e20__s{i}",
               "status": "BIT_IDENTICAL"} for i in range(8)]
    with History(record(results=narrow)):
        assert runner.gate_f_discharge(SHA) is None


def one_width_does_not_discharge():
    narrow = [{"cell": f"rust__fixed-t100__{'adjacent-sum-5' if i % 2 else 'channels-700'}"
                       f"__h128__e20__s{i}", "status": "BIT_IDENTICAL"} for i in range(8)]
    with History(record(results=narrow)):
        assert runner.gate_f_discharge(SHA) is None


def the_broadest_passing_run_is_chosen():
    with History(record(), record(results=cells(12))):
        best = runner.gate_f_discharge(SHA)
        assert best is not None and len(best["results"]) == 12, best


def a_corrupt_history_line_is_skipped_not_fatal():
    with History(record()) as h:
        path = h.dir / "gate-f-rust" / "runs.jsonl"
        path.write_text("{not json\n" + path.read_text())
        assert runner.gate_f_discharge(SHA) is not None


CHECKS = [
    a_clean_pass_discharges,
    a_legacy_record_without_the_key_still_discharges,
    a_pass_carrying_unrunnable_cells_does_not_discharge,
    an_incomplete_run_does_not_discharge,
    a_failing_run_does_not_discharge,
    a_run_of_another_binary_does_not_discharge,
    too_few_cells_does_not_discharge,
    one_geometry_does_not_discharge,
    one_width_does_not_discharge,
    the_broadest_passing_run_is_chosen,
    a_corrupt_history_line_is_skipped_not_fatal,
]


def main() -> int:
    failed = 0
    for check in CHECKS:
        try:
            check()
        except AssertionError as exc:
            print(f"FAIL {check.__name__}: {exc}"); failed += 1
        except Exception as exc:  # noqa: BLE001
            print(f"ERROR {check.__name__}: {type(exc).__name__}: {exc}"); failed += 1
        else:
            print(f"ok   {check.__name__}")
    print(f"{len(CHECKS) - failed}/{len(CHECKS)} passed")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
