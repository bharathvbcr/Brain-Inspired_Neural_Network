"""Gated, fingerprint-safe orchestration for SHD instrument calibration."""

from __future__ import annotations

import argparse
from dataclasses import asdict, dataclass
import hashlib
import itertools
import json
from pathlib import Path
import shutil
import subprocess
import sys
from typing import Iterable

import numpy as np

from . import CALIBRATION_FLOOR, PUBLISHED_ACCURACY, REFERENCE_COMMIT, SCHEMA_VERSION
from .data import (
    Contract,
    convert_h5_to_event_cache,
    corpus_summary,
    frame_events,
    read_event_cache,
    write_fixture_cache,
    write_json_atomic,
)
from .model import parity_payload, train_cell
from .provenance import (
    Exposure,
    cache_compatible,
    environment_record,
    immutable_manifest,
    sha256_file,
    source_fingerprint,
    write_cell_state,
)
from .reference import (
    SPIKINGJELLY_COMMIT,
    ensure_checkout,
    historical_payload_from_log,
    prepare_seed_worktree,
    run_reference,
    verify_clean_source,
)


ROOT = Path(__file__).resolve().parents[2]
RESULT_ROOT = ROOT / "results" / "shd_instrument_v4"
PRIOR_RESULT_ROOT = ROOT / "results" / "shd_instrument_v3"
EVENT_ROOT = ROOT / "data" / "shd" / "events"
RUST_BINARY = ROOT / "target" / "release" / "shd-instrument"
SEEDS = (5_170_001, 5_170_002, 5_170_003)
SOURCE_PATHS = (
    Path("Cargo.toml"),
    Path("Cargo.lock"),
    Path("binn-data/src/lib.rs"),
    Path("binn-data/src/shd_contract.rs"),
    Path("binn-learn/src/lib.rs"),
    Path("binn-learn/src/shd_matched.rs"),
    Path("binn-lab/src/instrument_status.rs"),
    Path("binn-lab/experiments/shd_instrument.rs"),
    Path("scripts/run_shd_instrument.py"),
    Path("scripts/shd_calibration/data.py"),
    Path("scripts/shd_calibration/model.py"),
    Path("scripts/shd_calibration/provenance.py"),
    Path("scripts/shd_calibration/reference.py"),
    Path("scripts/shd_calibration/reference_clean_main.py"),
    Path("scripts/shd_calibration/runner.py"),
    Path("scripts/test_shd_calibration.py"),
)


@dataclass(frozen=True)
class Cell:
    contract: str
    geometry: str
    hidden: int
    epochs: int
    seed: int
    backend: str

    @property
    def id(self) -> str:
        return (
            f"{self.backend}__{self.contract}__{self.geometry}"
            f"__h{self.hidden}__e{self.epochs}__s{self.seed}"
        )

    @property
    def n_inputs(self) -> int:
        return 700 if self.geometry == "channels-700" else 140


def all_cells() -> list[Cell]:
    contracts = (
        "published-10ms",
        "published-4ms",
        "published-2ms",
        "fixed-t100",
        "fixed-t250",
        "fixed-t500",
    )
    cells = [
        Cell(*values)
        for values in itertools.product(
            contracts,
            ("channels-700", "adjacent-sum-5"),
            (128, 256, 512),
            (20, 100),
            SEEDS,
            ("python", "rust"),
        )
    ]
    if len(cells) != 432:
        raise AssertionError(f"matrix cardinality changed: {len(cells)}")
    return cells


def contract_from_id(value: str) -> Contract:
    if value.startswith("published-") and value.endswith("ms"):
        return Contract("published", int(value.removeprefix("published-").removesuffix("ms")))
    if value.startswith("fixed-t"):
        return Contract("fixed", int(value.removeprefix("fixed-t")))
    raise ValueError(value)


def run_checked(command: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
    print("+", " ".join(command), flush=True)
    return subprocess.run(command, check=True, text=True, **kwargs)


def build_native() -> None:
    run_checked(
        ["cargo", "build", "--locked", "--release", "-p", "binn-lab", "--bin", "shd-instrument"],
        cwd=ROOT,
    )


def prepare_events() -> dict[str, object]:
    EVENT_ROOT.mkdir(parents=True, exist_ok=True)
    outputs: dict[str, object] = {}
    for split in ("train", "test"):
        source = ROOT / "data" / "shd" / f"shd_{split}.h5"
        destination = EVENT_ROOT / f"{split}.events"
        if not source.is_file():
            raise FileNotFoundError(source)
        if not destination.is_file():
            outputs[split] = convert_h5_to_event_cache(source, destination)
        else:
            outputs[split] = {
                "source": str(source),
                "output": str(destination),
                "status": "existing",
            }
    return outputs


def relevant_source_fingerprint() -> str:
    return source_fingerprint(ROOT, SOURCE_PATHS)


# Reference artifacts are produced by a third-party PyTorch baseline
# (`Thvnvtos/SNN-delays`, pinned by commit in `reference.py`) and never execute a
# line of the rust instrument. Fingerprinting them against SOURCE_PATHS — which
# includes `shd_matched.rs`, `shd_instrument.rs` and `Cargo.lock` — meant every
# kernel edit invalidated a PyTorch model's recorded accuracy, permanently
# blocking `historical_reference` / `clean_reference` / `matrix_authorized`,
# because the kernel keeps changing.
#
# All six reference artifacts were verified intact when this narrowed: every
# content hash matched and only the fingerprint failed. Registered in
# `results/AMENDMENT_2026-08-03_REFERENCE_FINGERPRINT_SCOPE.md`.
#
# `relevant_source_fingerprint` is unchanged and still governs instrument cells.
REFERENCE_SOURCE_PATHS = (
    Path("scripts/shd_calibration/reference.py"),
    Path("scripts/shd_calibration/reference_clean_main.py"),
    Path("scripts/shd_calibration/data.py"),
)


def reference_source_fingerprint() -> str:
    """Fingerprint over only what can affect a reference run.

    `reference.py` carries `REFERENCE_COMMIT`, so bumping the pinned upstream
    revision still invalidates the references - correctly.
    """
    return source_fingerprint(ROOT, REFERENCE_SOURCE_PATHS)


def core_manifest() -> dict[str, object]:
    required = {
        "train_h5": ROOT / "data/shd/shd_train.h5",
        "test_h5": ROOT / "data/shd/shd_test.h5",
        "train_events": EVENT_ROOT / "train.events",
        "test_events": EVENT_ROOT / "test.events",
        "rust_binary": RUST_BINARY,
        "cargo_lock": ROOT / "Cargo.lock",
    }
    missing = [str(path) for path in required.values() if not path.is_file()]
    if missing:
        raise RuntimeError("manifest inputs missing: " + ", ".join(missing))
    return {
        "schema": SCHEMA_VERSION,
        "calibration_revision": "v4",
        "supersedes": {
            "revision": "v3",
            "path": str(PRIOR_RESULT_ROOT),
            "reason": (
                "historical finalizer matched current and best accuracy on each "
                "epoch line; v3 reference training artifacts remain immutable"
            ),
        },
        "instrument_state": "UNCALIBRATED",
        "source_fingerprint": relevant_source_fingerprint(),
        "files": {name: {"path": str(path), "sha256": sha256_file(path)} for name, path in required.items()},
        "environment": environment_record(ROOT),
        "contracts": {
            "published": {"frame_ms": [10, 4, 2], "per_sample_t0": True, "count_preserving": True},
            "fixed": {
                "frames": [100, 250, 500],
                "duration_ms": 1400,
                "per_sample_t0": False,
                "count_preserving": True,
            },
            "frequency": ["channels-700", "adjacent-sum-5"],
        },
        "matched_model": {
            "topology": "input-linear-lif-linear",
            "tau_ms": 10.05,
            "detached_reset": True,
            "surrogate": "atan-alpha-5",
            "readout": "masked-mean-hidden-rate",
            "batch_size": 256,
            "optimizer": "adam",
            "base_lr": 1e-3,
            "max_lr": 5e-3,
            "weight_decay": 1e-5,
        },
        "seeds": list(SEEDS),
        "matrix_cells": 432,
        "reference": {
            "repository": "https://github.com/Thvnvtos/SNN-delays",
            "commit": REFERENCE_COMMIT,
            "spikingjelly_commit": SPIKINGJELLY_COMMIT,
            "historical_exposure": asdict(Exposure(150, 0, 150, 0, "official-test-best-accuracy")),
            "clean_exposure": asdict(Exposure(150, 0, 0, 1, "none-final-epoch")),
        },
    }


def manifest_path() -> Path:
    return RESULT_ROOT / "manifest.json"


def advance_provenance(
    payload: dict[str, object],
    fingerprint: str,
    evidence: dict[str, object],
) -> dict[str, object]:
    """Record a freeze transition justified by proven bit-identity.

    Appends to `provenance_chain` rather than overwriting, so the state that
    produced each recorded cell stays recoverable. The manifest is rewritten
    through the normal atomic writer; `immutable_manifest` is deliberately not
    reused, since this is the one sanctioned mutation.
    """
    cells = [str(item.get("cell", "")) for item in evidence.get("results", [])]
    chain = list(payload.get("provenance_chain", []))
    chain.append(
        {
            "from_source_fingerprint": payload.get("source_fingerprint"),
            "to_source_fingerprint": fingerprint,
            "from_rust_binary": payload.get("files", {}).get("rust_binary", {}).get("sha256"),
            "to_rust_binary": evidence.get("binary_sha256"),
            "discharged_by": "gate-f-rust bit-identity",
            "gate_f_cells": sorted(cells),
            "gate_f_cell_count": len(cells),
            "amendment": "AMENDMENT_2026-08-03_PROVENANCE_DISCHARGE_BY_BIT_IDENTITY.md",
        }
    )
    payload["provenance_chain"] = chain
    payload["source_fingerprint"] = fingerprint
    files = dict(payload.get("files", {}))
    for name, path in (("rust_binary", RUST_BINARY), ("cargo_lock", ROOT / "Cargo.lock")):
        record = dict(files.get(name, {}))
        record["sha256"] = sha256_file(path)
        files[name] = record
    payload["files"] = files
    write_json_atomic(manifest_path(), payload)
    return payload


"""Minimum recorded cells a Gate F report must cover to discharge the freeze.

Small enough to be cheap, large enough to span geometries and widths. The
spanning requirement matters more than the count: the original Gate F passed on a
fixture with atypically sparse frames and still missed cell-level divergence
(`HANDOFF_2026-08-02.md` §6 pitfall 2).
"""
PROVENANCE_MIN_GATE_F_CELLS = 8

#: Registered in `AMENDMENT_2026-08-03_PROVENANCE_DISCHARGE_BY_BIT_IDENTITY.md`.
#:
#: **Enabled 2026-08-05 by explicit human authorization.** It shipped default-off
#: because discharging a provenance freeze is a judgement about what counts as
#: evidence, not an engineering call, and that judgement was not the agent's to
#: make. It has now been made.
#:
#: Enabling this is **not** a bypass. `gate_f_discharge` still requires a PASS
#: report, for *this* binary hash, covering at least
#: `PROVENANCE_MIN_GATE_F_CELLS` recorded cells across two geometries and two
#: widths. Data-file changes (`train_h5`, `test_h5`, `train_events`,
#: `test_events`) remain **undischargeable** regardless of this flag — a kernel
#: proven to reproduce recorded cells says nothing about whether its inputs
#: moved. `scripts/test_provenance_discharge.py` pins both properties.
PROVENANCE_DISCHARGE_ENABLED = True


def gate_f_discharge(binary_sha: str) -> dict[str, object] | None:
    """Return a Gate F report that proves `binary_sha` reproduces recorded cells.

    Returns None when no such evidence exists, which leaves the caller to raise.
    Requires a PASS covering at least `PROVENANCE_MIN_GATE_F_CELLS` recorded
    cells across more than one geometry and more than one hidden width; see the
    amendment §4 for why breadth is required and not just count.
    """
    history = RESULT_ROOT / "gate-f-rust" / "runs.jsonl"
    if not history.is_file():
        return None
    best: dict[str, object] | None = None
    for line in history.read_text().splitlines():
        if not line.strip():
            continue
        try:
            record = json.loads(line)
        except json.JSONDecodeError:
            continue
        if record.get("binary_sha256") != binary_sha:
            continue
        if record.get("status") != "PASS" or record.get("failures") != 0:
            continue
        # Defence in depth on the provenance path. `PASS` already implies
        # nothing went unjudged, but a cell the gate could not run is not
        # evidence about the kernel, and this must not depend on how a
        # later version of the gate happens to compute its status. Records
        # written before the key existed default to 0 and are unaffected.
        if record.get("unrunnable", 0) != 0:
            continue
        cells = [str(item.get("cell", "")) for item in record.get("results", [])]
        if len(cells) < PROVENANCE_MIN_GATE_F_CELLS:
            continue
        # Cell ids are `rust__<contract>__<geometry>__h<width>__e<epochs>__s<seed>`.
        geometries = {part.split("__")[2] for part in cells if part.count("__") >= 5}
        widths = {part.split("__")[3] for part in cells if part.count("__") >= 5}
        if len(geometries) < 2 or len(widths) < 2:
            continue
        if best is None or len(cells) > len(best.get("results", [])):
            best = record
    return best


def ensure_manifest() -> tuple[dict[str, object], str]:
    if manifest_path().is_file():
        payload = json.loads(manifest_path().read_text())
        mismatches: list[str] = []
        observed_fingerprint = relevant_source_fingerprint()
        if payload.get("source_fingerprint") != observed_fingerprint:
            mismatches.append(
                "source fingerprint changed after calibration manifest freeze: "
                f"manifest={payload.get('source_fingerprint')} current={observed_fingerprint}"
            )
        expected_files = {
            "train_h5": ROOT / "data/shd/shd_train.h5",
            "test_h5": ROOT / "data/shd/shd_test.h5",
            "train_events": EVENT_ROOT / "train.events",
            "test_events": EVENT_ROOT / "test.events",
            "rust_binary": RUST_BINARY,
            "cargo_lock": ROOT / "Cargo.lock",
        }
        # Data mismatches are never dischargeable. Proving the kernel reproduces
        # recorded cells says nothing about whether the inputs changed under it,
        # so these keep the original unconditional hard error (amendment §5).
        undischargeable = {"train_h5", "test_h5", "train_events", "test_events"}
        for name, path in expected_files.items():
            expected = payload.get("files", {}).get(name, {}).get("sha256")
            observed = sha256_file(path)
            if expected == observed:
                continue
            message = (
                f"manifest input changed after freeze: {name} "
                f"manifest={expected} current={observed}"
            )
            if name in undischargeable:
                raise RuntimeError(message)
            mismatches.append(message)
        if mismatches:
            detail = "\n  ".join(mismatches)
            if not PROVENANCE_DISCHARGE_ENABLED:
                raise RuntimeError(
                    f"{detail}\n"
                    "  Bit-identity discharge exists but is disabled; see "
                    "AMENDMENT_2026-08-03_PROVENANCE_DISCHARGE_BY_BIT_IDENTITY.md "
                    "and set PROVENANCE_DISCHARGE_ENABLED once signed off. "
                    "Do NOT delete manifest.json to get past this."
                )
            evidence = gate_f_discharge(sha256_file(RUST_BINARY))
            if evidence is None:
                raise RuntimeError(
                    f"{detail}\n"
                    "  No Gate F evidence for the current binary. Run "
                    "`scripts/gate_f_rust.py` over at least "
                    f"{PROVENANCE_MIN_GATE_F_CELLS} recorded cells spanning two "
                    "geometries and two widths, then retry."
                )
            payload = advance_provenance(payload, observed_fingerprint, evidence)
        return payload, sha256_file(manifest_path())
    payload = core_manifest()
    immutable_manifest(manifest_path(), payload)
    return payload, sha256_file(manifest_path())


def gate_path() -> Path:
    return RESULT_ROOT / "gates.json"


def load_gates() -> dict[str, object]:
    if not gate_path().is_file():
        return {
            "data_parity": False,
            "forward_parity": False,
            "gradient_parity": False,
            "update_parity": False,
            "historical_reference": False,
            "clean_reference": False,
            "fresh_process_replay": False,
            "matrix_authorized": False,
        }
    return json.loads(gate_path().read_text())


def store_gates(gates: dict[str, object]) -> None:
    gates["matrix_authorized"] = all(
        gates.get(name) is True
        for name in (
            "data_parity",
            "forward_parity",
            "gradient_parity",
            "update_parity",
            "historical_reference",
            "clean_reference",
            "fresh_process_replay",
        )
    )
    write_json_atomic(gate_path(), gates)


def prepare() -> None:
    RESULT_ROOT.mkdir(parents=True, exist_ok=True)
    prepare_events()
    build_native()
    ensure_manifest()
    ledger = {
        "schema": SCHEMA_VERSION,
        "mechanical_status": "MATERIALIZED",
        "harness_status": "PENDING_PREREQUISITES",
        "scientific_status": "UNCALIBRATED",
        "cells_total": 432,
        "cells": [{"id": cell.id, **asdict(cell), "status": "PENDING"} for cell in all_cells()],
        "references": [
            {"mode": mode, "seed": seed, "status": "PENDING"}
            for mode in ("historical", "clean")
            for seed in SEEDS
        ],
    }
    immutable_manifest(RESULT_ROOT / "ledger.initial.json", ledger)
    if not gate_path().exists():
        store_gates(load_gates())
    print(f"prepared {RESULT_ROOT}")


def fixture_parity() -> dict[str, float | bool]:
    fixture_path = RESULT_ROOT / "fixtures" / "events.events"
    write_fixture_cache(fixture_path)
    data_ok = True
    for contract_id in (
        "published-10ms",
        "published-4ms",
        "published-2ms",
        "fixed-t100",
        "fixed-t250",
        "fixed-t500",
    ):
        contract = contract_from_id(contract_id)
        for geometry in ("channels-700", "adjacent-sum-5"):
            expected = [
                frame_events(sample, contract, geometry).fingerprint()
                for sample in read_event_cache(fixture_path)
            ]
            completed = subprocess.run(
                [
                    str(RUST_BINARY),
                    "fixture-hashes",
                    "--events",
                    str(fixture_path),
                    "--contract",
                    contract_id,
                    "--geometry",
                    geometry,
                ],
                check=True,
                text=True,
                capture_output=True,
            )
            observed = [line.split("\t")[1] for line in completed.stdout.splitlines()]
            data_ok &= expected == observed
    return {"data_parity": data_ok}


def relative_error(expected: np.ndarray, observed: np.ndarray) -> float:
    numerator = np.linalg.norm(expected - observed)
    denominator = max(float(np.linalg.norm(expected)), float(np.linalg.norm(observed)), 1e-12)
    return float(numerator / denominator)


def model_parity() -> dict[str, object]:
    fixture_path = RESULT_ROOT / "fixtures" / "events.events"
    weights_path = RESULT_ROOT / "fixtures" / "parity-weights.bin"
    orders_path = RESULT_ROOT / "fixtures" / "parity-orders.bin"
    run_checked(
        [
            str(RUST_BINARY),
            "init",
            "--n-inputs",
            "700",
            "--hidden",
            "4",
            "--classes",
            "20",
            "--seed",
            "91",
            "--epochs",
            "1",
            "--n-train",
            "20",
            "--weights",
            str(weights_path),
            "--orders",
            str(orders_path),
        ]
    )
    sample = frame_events(
        read_event_cache(fixture_path)[3],
        Contract("published", 10),
        "channels-700",
    )
    python = parity_payload(weights_path, sample)
    rust_path = RESULT_ROOT / "fixtures" / "rust-parity.json"
    run_checked(
        [
            str(RUST_BINARY),
            "parity",
            "--events",
            str(fixture_path),
            "--index",
            "3",
            "--contract",
            "published-10ms",
            "--geometry",
            "channels-700",
            "--weights",
            str(weights_path),
            "--out",
            str(rust_path),
        ]
    )
    rust = json.loads(rust_path.read_text())
    forward_keys = ("membrane", "spikes", "rates", "logits")
    gradient_keys = ("grad_w_in", "grad_w_out", "grad_b_out")
    update_keys = ("updated_w_in", "updated_w_out", "updated_b_out")
    forward_error = max(
        relative_error(np.asarray(python[key]), np.asarray(rust[key])) for key in forward_keys
    )
    gradient_error = max(
        relative_error(np.asarray(python[key]), np.asarray(rust[key])) for key in gradient_keys
    )
    update_error = max(
        relative_error(np.asarray(python[key]), np.asarray(rust[key])) for key in update_keys
    )
    return {
        "frame_hash_match": python["frame_hash"] == rust["frame_hash"],
        "forward_error": forward_error,
        "gradient_error": gradient_error,
        "update_error": update_error,
        "forward_parity": forward_error <= 1e-6,
        "gradient_parity": gradient_error <= 1e-4,
        "update_parity": update_error <= 1e-5,
    }


def summaries() -> list[dict[str, object]]:
    train = read_event_cache(EVENT_ROOT / "train.events")
    test = read_event_cache(EVENT_ROOT / "test.events")
    output = []
    for contract_id in (
        "published-10ms",
        "published-4ms",
        "published-2ms",
        "fixed-t100",
        "fixed-t250",
        "fixed-t500",
    ):
        contract = contract_from_id(contract_id)
        for geometry in ("channels-700", "adjacent-sum-5"):
            output.append(corpus_summary(train, contract, geometry, "train"))
            output.append(corpus_summary(test, contract, geometry, "test"))
    write_json_atomic(RESULT_ROOT / "data_summary.json", output)
    return output


def preflight(skip_summary: bool = False) -> None:
    verify_clean_source(ROOT / "scripts/shd_calibration/reference_clean_main.py")
    data_result = fixture_parity()
    model_result = model_parity()
    if not skip_summary:
        summaries()
    report = {**data_result, **model_result}
    write_json_atomic(RESULT_ROOT / "parity_report.json", report)
    gates = load_gates()
    for key in ("data_parity", "forward_parity", "gradient_parity", "update_parity"):
        gates[key] = bool(report[key])
    store_gates(gates)
    if not all(gates[key] for key in ("data_parity", "forward_parity", "gradient_parity", "update_parity")):
        raise RuntimeError(f"parity prerequisite failed: {report}")
    print(json.dumps(report, indent=2, sort_keys=True))


def setup_reference(python: Path) -> None:
    if not python.exists():
        run_checked(
            [
                "uv",
                "venv",
                "--clear",
                "--seed",
                "--python",
                "3.12",
                str(python.parent.parent),
            ],
            cwd=ROOT,
        )
    run_checked(
        [
            str(python),
            "-m",
            "pip",
            "install",
            "torch",
            "torchvision",
            "torchaudio",
            "numpy",
            "h5py",
            "tqdm",
            "wandb",
            "dcls",
            "pillow",
        ],
        cwd=ROOT,
    )
    run_checked(
        [
            str(python),
            "-m",
            "pip",
            "install",
            (
                "git+https://github.com/fangwei123456/spikingjelly.git"
                f"@{SPIKINGJELLY_COMMIT}"
            ),
        ],
        cwd=ROOT,
    )
    prepare_reference_dataset()
    run_checked(
        [
            str(python),
            "-c",
            (
                "import torch, h5py, DCLS, spikingjelly; "
                "from spikingjelly.datasets.shd import SpikingHeidelbergDigits; "
                "print(torch.__version__)"
            ),
        ],
        cwd=ROOT,
    )


def prepare_reference_dataset() -> Path:
    dataset_root = ROOT / "data/shd/reference"
    # The compatible 2023 loader treats an existing `extract/` directory as
    # authoritative and reads these exact files without downloading.
    raw_roots = (dataset_root / "extract", dataset_root / "events_h5")
    for raw_root in raw_roots:
        raw_root.mkdir(parents=True, exist_ok=True)
    for split in ("train", "test"):
        source = (ROOT / "data/shd" / f"shd_{split}.h5").resolve()
        for raw_root in raw_roots:
            target = raw_root / f"shd_{split}.h5"
            if target.is_symlink() and target.resolve() == source:
                continue
            if target.exists() or target.is_symlink():
                raise RuntimeError(
                    f"reference dataset target conflicts with official HDF5: {target}"
                )
            target.symlink_to(source)
    return dataset_root


def reference(mode: str, seed: int, python: Path) -> None:
    gates = load_gates()
    for gate in ("data_parity", "forward_parity", "gradient_parity", "update_parity"):
        if gates.get(gate) is not True:
            raise RuntimeError(f"reference blocked: prerequisite {gate} has not passed")
    checkout = ensure_checkout(RESULT_ROOT / "reference-cache")
    dataset_root = prepare_reference_dataset()
    result = RESULT_ROOT / "references" / f"{mode}-seed-{seed}.json"
    log = RESULT_ROOT / "references" / f"{mode}-seed-{seed}.log"
    worktree = prepare_seed_worktree(
        checkout,
        RESULT_ROOT / "reference-worktrees",
        seed,
        dataset_root,
        mode,
        ROOT / "scripts/shd_calibration/reference_clean_main.py",
    )
    freeze = subprocess.check_output(
        [str(python), "-m", "pip", "freeze"], text=True
    )
    reference_manifest = {
        "schema": "shd-reference-manifest-v1",
        "mode": mode,
        "seed": seed,
        "reference_commit": REFERENCE_COMMIT,
        "spikingjelly_commit": SPIKINGJELLY_COMMIT,
        "reference_tree": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=worktree, text=True
        ).strip(),
        "config_sha256": sha256_file(worktree / "config.py"),
        "datasets_harness_sha256": sha256_file(worktree / "datasets.py"),
        "platform_adaptations": [
            "DataLoader num_workers 4 to 0 on macOS; samples, shuffle RNG, batches, and optimizer unchanged"
        ],
        "clean_template_sha256": (
            sha256_file(ROOT / "scripts/shd_calibration/reference_clean_main.py")
            if mode == "clean"
            else None
        ),
        "dependencies": freeze.splitlines(),
        "dependencies_sha256": hashlib.sha256(freeze.encode()).hexdigest(),
        "python": {"path": str(python), "sha256": sha256_file(python)},
        "datasets": {
            "train_h5": sha256_file(ROOT / "data/shd/shd_train.h5"),
            "test_h5": sha256_file(ROOT / "data/shd/shd_test.h5"),
        },
        "exposure": asdict(
            Exposure(
                150,
                0,
                150 if mode == "historical" else 0,
                0 if mode == "historical" else 1,
                "official-test-best-accuracy" if mode == "historical" else "none-final-epoch",
            )
        ),
        # Narrow scope, registered in
        # `AMENDMENT_2026-08-22_REFERENCE_FINGERPRINT_SCOPE_FORWARD.md`. The
        # reference is a third-party PyTorch baseline that never executes a line
        # of the rust instrument, so fingerprinting it against the kernel meant
        # every unrelated kernel edit invalidated a recorded accuracy. The scope
        # is written into the artifact because a fingerprint is only meaningful
        # alongside the path set it was computed over; `reference_fingerprint_matches`
        # reads it back and will not compare across scopes.
        "fingerprint_scope": "reference",
        "source_fingerprint": reference_source_fingerprint(),
        # Recorded alongside, for provenance only. Nothing validates against it:
        # it is what the broad scope happened to be when this artifact was
        # frozen, and it is expected to drift.
        "instrument_source_fingerprint_at_freeze": relevant_source_fingerprint(),
    }
    reference_manifest_path = (
        RESULT_ROOT / "reference-manifests" / f"{mode}-seed-{seed}.json"
    )
    immutable_manifest(reference_manifest_path, reference_manifest)
    state_path = RESULT_ROOT / "reference-states" / f"{mode}-seed-{seed}.json"
    write_cell_state(
        state_path,
        {
            "mechanical_status": "RUNNING",
            "mode": mode,
            "seed": seed,
            "manifest_sha256": sha256_file(reference_manifest_path),
            "result_path": str(result),
        },
    )
    payload = run_reference(
        checkout,
        RESULT_ROOT / "reference-worktrees",
        python,
        dataset_root,
        seed,
        mode,
        result,
        log,
        ROOT / "scripts/shd_calibration/reference_clean_main.py",
        prepared_worktree=worktree,
    )
    write_cell_state(
        state_path,
        {
            "mechanical_status": "COMPLETE",
            "mode": mode,
            "seed": seed,
            "manifest_sha256": sha256_file(reference_manifest_path),
            "result_path": str(result),
            "result_sha256": sha256_file(result),
            "log_sha256": sha256_file(log),
        },
    )
    print(json.dumps(payload, indent=2, sort_keys=True))
    update_reference_gates()


def reference_fingerprint_matches(reference_manifest: dict[str, object]) -> bool:
    """Check a reference manifest's fingerprint against the scope it declares.

    Two scopes exist, and which one applies is a property **of the artifact**,
    recorded when it was frozen. That is not a convenience: a fingerprint is the
    output of a function over a set of paths, so a value computed over
    `SOURCE_PATHS` and a value computed over `REFERENCE_SOURCE_PATHS` are not a
    stale and a fresh reading of the same quantity -- they are outputs of
    different functions and are never comparable.
    `AMENDMENT_2026-08-03_REFERENCE_FINGERPRINT_SCOPE.md` was withdrawn for
    assuming otherwise, and this must not repeat that mistake.

    So the narrow scope applies **only** to artifacts that declare it. Every
    artifact frozen before 2026-08-22 declares nothing and is checked against the
    broad scope exactly as before: this change cannot validate a single existing
    reference, and is not intended to. It is registered in
    `AMENDMENT_2026-08-22_REFERENCE_FINGERPRINT_SCOPE_FORWARD.md`.

    What it buys is that the *next* set of references does not rot. The reference
    pipeline never executes a line of the rust instrument, so without this a
    freshly re-run reference is invalidated by the next unrelated kernel edit and
    route 2 of that amendment's section 6 becomes a treadmill.
    """
    recorded = reference_manifest.get("source_fingerprint")
    scope = reference_manifest.get("fingerprint_scope", "source")
    if scope == "reference":
        return recorded == reference_source_fingerprint()
    if scope == "source":
        return recorded == relevant_source_fingerprint()
    # An unrecognised scope is not a reason to accept the artifact.
    return False


def valid_reference_payload(mode: str, seed: int) -> dict[str, object] | None:
    result_path = RESULT_ROOT / "references" / f"{mode}-seed-{seed}.json"
    log_path = RESULT_ROOT / "references" / f"{mode}-seed-{seed}.log"
    manifest = RESULT_ROOT / "reference-manifests" / f"{mode}-seed-{seed}.json"
    state_path = RESULT_ROOT / "reference-states" / f"{mode}-seed-{seed}.json"
    if not all(path.is_file() for path in (result_path, log_path, manifest, state_path)):
        return None
    try:
        state = json.loads(state_path.read_text())
        reference_manifest = json.loads(manifest.read_text())
        payload = json.loads(result_path.read_text())
    except (OSError, json.JSONDecodeError):
        return None
    if (
        state.get("mechanical_status") != "COMPLETE"
        or state.get("mode") != mode
        or state.get("seed") != seed
        or state.get("result_path") != str(result_path)
        or state.get("result_sha256") != sha256_file(result_path)
        or state.get("log_sha256") != sha256_file(log_path)
        or state.get("manifest_sha256") != sha256_file(manifest)
        or not reference_fingerprint_matches(reference_manifest)
        or payload.get("mode") != mode
        or payload.get("seed") != seed
    ):
        return None
    return payload


def update_reference_gates() -> None:
    gates = load_gates()
    historical = []
    clean = []
    gates["historical_reference"] = False
    gates["clean_reference"] = False
    gates.pop("historical_mean_accuracy", None)
    gates.pop("clean_accuracies", None)
    for seed in SEEDS:
        for mode, target in (("historical", historical), ("clean", clean)):
            payload = valid_reference_payload(mode, seed)
            if payload is not None:
                target.append(payload)
    if len(historical) == 3:
        mean = sum(float(item["accuracy"]) for item in historical) / 3
        exposures_historical = all(
            item.get("test_reads_during_training") == 150
            and item.get("final_test_reads") == 0
            and item.get("exposure_status") == "EXPOSURE_TAINTED_DESCRIPTIVE"
            for item in historical
        )
        gates["historical_reference"] = (
            abs(mean - PUBLISHED_ACCURACY) <= 0.05 and exposures_historical
        )
        gates["historical_mean_accuracy"] = mean
    if len(clean) == 3:
        accuracies = [float(item["accuracy"]) for item in clean]
        exposures_clean = all(
            item.get("test_reads_during_training") == 0
            and item.get("final_test_reads") == 1
            for item in clean
        )
        gates["clean_reference"] = (
            all(value >= CALIBRATION_FLOOR for value in accuracies) and exposures_clean
        )
        gates["clean_accuracies"] = accuracies
    store_gates(gates)


def recover_references(source_root: Path = PRIOR_RESULT_ROOT) -> None:
    """Import immutable completed v3 training artifacts under a v4 finalizer."""
    ensure_manifest()
    gates = load_gates()
    for gate in (
        "data_parity",
        "forward_parity",
        "gradient_parity",
        "update_parity",
        "fresh_process_replay",
    ):
        if gates.get(gate) is not True:
            raise RuntimeError(f"reference recovery blocked: prerequisite {gate} has not passed")

    parser_path = ROOT / "scripts/shd_calibration/reference.py"
    current_fingerprint = relevant_source_fingerprint()
    for mode in ("historical", "clean"):
        for seed in SEEDS:
            source_log = source_root / "references" / f"{mode}-seed-{seed}.log"
            source_manifest = (
                source_root / "reference-manifests" / f"{mode}-seed-{seed}.json"
            )
            source_state = (
                source_root / "reference-states" / f"{mode}-seed-{seed}.json"
            )
            if not all(
                path.is_file() for path in (source_log, source_manifest, source_state)
            ):
                raise RuntimeError(f"missing v3 reference provenance for {mode} seed {seed}")
            prior_manifest = json.loads(source_manifest.read_text())
            prior_state = json.loads(source_state.read_text())
            if (
                prior_manifest.get("mode") != mode
                or prior_manifest.get("seed") != seed
                or prior_state.get("mode") != mode
                or prior_state.get("seed") != seed
                or prior_state.get("manifest_sha256") != sha256_file(source_manifest)
            ):
                raise RuntimeError(f"v3 provenance mismatch for {mode} seed {seed}")

            if mode == "historical":
                payload = historical_payload_from_log(
                    source_log.read_text(errors="replace"), seed
                )
                prior_mechanical_status = prior_state.get("mechanical_status")
                if prior_mechanical_status != "RUNNING":
                    raise RuntimeError(
                        f"unexpected v3 historical state for seed {seed}: "
                        f"{prior_mechanical_status}"
                    )
                recovery_reason = (
                    "training completed 150 epochs; v3 finalizer failed because it "
                    "counted current and best accuracy on every epoch line"
                )
            else:
                source_result = (
                    source_root / "references" / f"clean-seed-{seed}.json"
                )
                if (
                    prior_state.get("mechanical_status") != "COMPLETE"
                    or not source_result.is_file()
                    or prior_state.get("result_sha256") != sha256_file(source_result)
                    or prior_state.get("log_sha256") != sha256_file(source_log)
                ):
                    raise RuntimeError(f"invalid completed v3 clean state for seed {seed}")
                payload = json.loads(source_result.read_text())
                recovery_reason = "immutable completed clean result imported without change"

            result_path = RESULT_ROOT / "references" / f"{mode}-seed-{seed}.json"
            log_path = RESULT_ROOT / "references" / f"{mode}-seed-{seed}.log"
            result_path.parent.mkdir(parents=True, exist_ok=True)
            if not log_path.exists():
                shutil.copy2(source_log, log_path)
            elif sha256_file(log_path) != sha256_file(source_log):
                raise RuntimeError(f"recovered log mismatch for {mode} seed {seed}")
            write_json_atomic(result_path, payload)

            recovery_manifest = {
                "schema": "shd-reference-recovery-manifest-v1",
                "mode": mode,
                "seed": seed,
                "source_fingerprint": current_fingerprint,
                "training_source_fingerprint": prior_manifest.get("source_fingerprint"),
                "finalizer_source_fingerprint": current_fingerprint,
                "recovery_reason": recovery_reason,
                "training_rerun": False,
                "source_revision": {
                    "path": str(source_root),
                    "manifest_path": str(source_manifest),
                    "manifest_sha256": sha256_file(source_manifest),
                    "state_path": str(source_state),
                    "state_sha256": sha256_file(source_state),
                    "log_path": str(source_log),
                    "log_sha256": sha256_file(source_log),
                },
                "parser": {
                    "path": str(parser_path),
                    "sha256": sha256_file(parser_path),
                    "contract": "one current validation accuracy for each epoch 0..149",
                },
                "exposure": prior_manifest.get("exposure"),
            }
            manifest = (
                RESULT_ROOT / "reference-manifests" / f"{mode}-seed-{seed}.json"
            )
            immutable_manifest(manifest, recovery_manifest)
            state = RESULT_ROOT / "reference-states" / f"{mode}-seed-{seed}.json"
            write_cell_state(
                state,
                {
                    "mechanical_status": "COMPLETE",
                    "mode": mode,
                    "seed": seed,
                    "manifest_sha256": sha256_file(manifest),
                    "result_path": str(result_path),
                    "result_sha256": sha256_file(result_path),
                    "log_sha256": sha256_file(log_path),
                    "training_rerun": False,
                    "recovered_from": str(source_root),
                },
            )
    update_reference_gates()


def initialization(cell: Cell) -> tuple[Path, Path]:
    directory = RESULT_ROOT / "initialization"
    weights = directory / f"n{cell.n_inputs}-h{cell.hidden}-s{cell.seed}.weights"
    orders = directory / f"n8156-e100-s{cell.seed}.orders"
    if not weights.is_file() or not orders.is_file():
        run_checked(
            [
                str(RUST_BINARY),
                "init",
                "--n-inputs",
                str(cell.n_inputs),
                "--hidden",
                str(cell.hidden),
                "--classes",
                "20",
                "--seed",
                str(cell.seed),
                "--epochs",
                "100",
                "--n-train",
                "8156",
                "--weights",
                str(weights),
                "--orders",
                str(orders),
            ]
        )
    return weights, orders


def run_cell(cell: Cell, max_train: int | None = None, max_test: int | None = None) -> None:
    gates = load_gates()
    if gates.get("matrix_authorized") is not True:
        raise RuntimeError("matrix is blocked until every prerequisite in gates.json passes")
    manifest, manifest_sha = ensure_manifest()
    fingerprint = str(manifest["source_fingerprint"])
    state = RESULT_ROOT / "states" / f"{cell.id}.json"
    if cache_compatible(state, fingerprint, manifest_sha):
        print(f"CACHE {cell.id}")
        return
    result = RESULT_ROOT / "cells" / f"{cell.id}.json"
    weights, orders = initialization(cell)
    cell_manifest = {
        "schema": "shd-cal-cell-manifest-v1",
        **asdict(cell),
        "id": cell.id,
        "source_fingerprint": fingerprint,
        "core_manifest_sha256": manifest_sha,
        "weights": {"path": str(weights), "sha256": sha256_file(weights)},
        "orders": {"path": str(orders), "sha256": sha256_file(orders)},
        "datasets": {
            "train": sha256_file(EVENT_ROOT / "train.events"),
            "test": sha256_file(EVENT_ROOT / "test.events"),
        },
        "exposure": asdict(Exposure(cell.epochs, 0, 0, 1, "none-final-epoch")),
        "replay_identity": f"{cell.id}:{fingerprint}:{manifest_sha}",
    }
    cell_manifest_path = RESULT_ROOT / "cell-manifests" / f"{cell.id}.json"
    immutable_manifest(cell_manifest_path, cell_manifest)
    state_payload = {
        "id": cell.id,
        "mechanical_status": "RUNNING",
        "source_fingerprint": fingerprint,
        "manifest_sha256": manifest_sha,
        "cell_manifest_sha256": sha256_file(cell_manifest_path),
        "result_path": str(result),
    }
    write_cell_state(state, state_payload)
    try:
        contract = contract_from_id(cell.contract)
        if cell.backend == "python":
            train_cell(
                EVENT_ROOT / "train.events",
                EVENT_ROOT / "test.events",
                contract,
                cell.geometry,
                weights,
                orders,
                cell.epochs,
                result,
                max_train,
                max_test,
            )
        else:
            command = [
                str(RUST_BINARY),
                "train-cell",
                "--train-events",
                str(EVENT_ROOT / "train.events"),
                "--test-events",
                str(EVENT_ROOT / "test.events"),
                "--contract",
                cell.contract,
                "--geometry",
                cell.geometry,
                "--weights",
                str(weights),
                "--orders",
                str(orders),
                "--epochs",
                str(cell.epochs),
                "--out",
                str(result),
            ]
            if max_train is not None:
                command += ["--max-train", str(max_train)]
            if max_test is not None:
                command += ["--max-test", str(max_test)]
            run_checked(command)
        state_payload["mechanical_status"] = "COMPLETE"
        state_payload["result_sha256"] = sha256_file(result)
    except BaseException as error:
        state_payload["mechanical_status"] = "FAILED"
        state_payload["error"] = repr(error)
        write_cell_state(state, state_payload)
        raise
    write_cell_state(state, state_payload)


def run_matrix(limit: int | None = None, backend: str | None = None) -> None:
    selected = [cell for cell in all_cells() if backend is None or cell.backend == backend]
    if limit is not None:
        selected = selected[:limit]
    for index, cell in enumerate(selected, 1):
        print(f"[{index}/{len(selected)}] {cell.id}", flush=True)
        run_cell(cell)
    write_ledger()


def write_ledger() -> None:
    rows = []
    for cell in all_cells():
        state_path = RESULT_ROOT / "states" / f"{cell.id}.json"
        result_path = RESULT_ROOT / "cells" / f"{cell.id}.json"
        state = json.loads(state_path.read_text()) if state_path.is_file() else {}
        result = json.loads(result_path.read_text()) if result_path.is_file() else {}
        rows.append({"id": cell.id, **asdict(cell), **state, "result": result})
    complete = sum(row.get("mechanical_status") == "COMPLETE" for row in rows)
    failed = sum(row.get("mechanical_status") == "FAILED" for row in rows)
    gates = load_gates()
    payload = {
        "schema": SCHEMA_VERSION,
        "mechanical_status": "COMPLETE" if complete == 432 else "IN_PROGRESS",
        "harness_status": "VALID" if gates.get("matrix_authorized") else "PENDING_PREREQUISITES",
        "scientific_status": "UNCALIBRATED" if complete < 432 else matrix_verdict(rows),
        "cells_total": 432,
        "cells_complete": complete,
        "cells_failed": failed,
        "gates": gates,
        "cells": rows,
    }
    write_json_atomic(RESULT_ROOT / "ledger.json", payload)


def matrix_verdict(rows: list[dict[str, object]]) -> str:
    groups: dict[tuple[object, ...], dict[str, list[dict[str, object]]]] = {}
    for row in rows:
        key = (row["contract"], row["geometry"], row["hidden"], row["epochs"])
        groups.setdefault(key, {"python": [], "rust": []})[str(row["backend"])].append(row)
    for backends in groups.values():
        if any(len(backends[name]) != 3 for name in ("python", "rust")):
            continue
        by_seed = {
            name: {row["seed"]: row.get("result", {}) for row in values}
            for name, values in backends.items()
        }
        if not all(
            result.get("scientific_status") == "CELL_PASS"
            for backend in by_seed.values()
            for result in backend.values()
        ):
            continue
        if all(
            abs(
                float(by_seed["python"][seed]["accuracy"])
                - float(by_seed["rust"][seed]["accuracy"])
            )
            <= 0.05
            for seed in SEEDS
        ):
            return "CALIBRATED"
    return "FAIL"


def replay_smoke() -> None:
    fixture = RESULT_ROOT / "fixtures" / "events.events"
    if not fixture.is_file():
        write_fixture_cache(fixture)
    weights = RESULT_ROOT / "fixtures" / "replay.weights"
    orders = RESULT_ROOT / "fixtures" / "replay.orders"
    run_checked(
        [
            str(RUST_BINARY),
            "init",
            "--n-inputs",
            "140",
            "--hidden",
            "4",
            "--classes",
            "20",
            "--seed",
            "8128",
            "--epochs",
            "1",
            "--n-train",
            "20",
            "--weights",
            str(weights),
            "--orders",
            str(orders),
        ]
    )
    outputs = []
    for replay in (1, 2):
        output = RESULT_ROOT / "fixtures" / f"replay-{replay}.json"
        run_checked(
            [
                str(RUST_BINARY),
                "train-cell",
                "--train-events",
                str(fixture),
                "--test-events",
                str(fixture),
                "--contract",
                "fixed-t100",
                "--geometry",
                "adjacent-sum-5",
                "--weights",
                str(weights),
                "--orders",
                str(orders),
                "--epochs",
                "1",
                "--out",
                str(output),
            ]
        )
        payload = json.loads(output.read_text())
        payload.pop("wall_secs", None)
        outputs.append(payload)
    passed = outputs[0] == outputs[1]
    report = {"fresh_process_replay": passed, "first": outputs[0], "second": outputs[1]}
    write_json_atomic(RESULT_ROOT / "fresh_process_replay.json", report)
    gates = load_gates()
    gates["fresh_process_replay"] = passed
    store_gates(gates)
    if not passed:
        raise RuntimeError("fresh-process deterministic replay mismatch")


def status() -> None:
    update_reference_gates()
    write_ledger()
    ledger = json.loads((RESULT_ROOT / "ledger.json").read_text())
    print(
        json.dumps(
            {
                "mechanical_status": ledger["mechanical_status"],
                "harness_status": ledger["harness_status"],
                "scientific_status": ledger["scientific_status"],
                "cells_total": ledger["cells_total"],
                "cells_complete": ledger["cells_complete"],
                "cells_failed": ledger["cells_failed"],
                "gates": ledger["gates"],
            },
            indent=2,
            sort_keys=True,
        )
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    subparsers.add_parser("prepare")
    preflight_parser = subparsers.add_parser("preflight")
    preflight_parser.add_argument("--skip-summary", action="store_true")
    setup_parser = subparsers.add_parser("setup-reference")
    setup_parser.add_argument("--python", type=Path, default=ROOT / ".venv-shd-cal/bin/python")
    reference_parser = subparsers.add_parser("reference")
    reference_parser.add_argument("--mode", choices=("historical", "clean"), required=True)
    reference_parser.add_argument("--seed", type=int, choices=SEEDS, required=True)
    reference_parser.add_argument("--python", type=Path, default=ROOT / ".venv-shd-cal/bin/python")
    recover_parser = subparsers.add_parser("recover-references")
    recover_parser.add_argument("--from-root", type=Path, default=PRIOR_RESULT_ROOT)
    matrix_parser = subparsers.add_parser("run-matrix")
    matrix_parser.add_argument("--limit", type=int)
    matrix_parser.add_argument("--backend", choices=("python", "rust"))
    subparsers.add_parser("replay-smoke")
    subparsers.add_parser("status")
    args = parser.parse_args(argv)
    if args.command == "prepare":
        prepare()
    elif args.command == "preflight":
        preflight(args.skip_summary)
    elif args.command == "setup-reference":
        setup_reference(args.python)
    elif args.command == "reference":
        reference(args.mode, args.seed, args.python)
    elif args.command == "recover-references":
        recover_references(args.from_root)
    elif args.command == "run-matrix":
        run_matrix(args.limit, args.backend)
    elif args.command == "replay-smoke":
        replay_smoke()
    elif args.command == "status":
        status()
    return 0
