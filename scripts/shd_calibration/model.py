"""NumPy implementation of the matched SHD BPTT instrument."""

from __future__ import annotations

from dataclasses import dataclass
import json
import math
from pathlib import Path
import struct
import time

import numpy as np

from .data import FramedSample, Contract, frame_events, read_event_cache, write_json_atomic


WEIGHT_MAGIC = b"SHDWGT1\0"
ORDER_MAGIC = b"SHDORD1\0"
THRESHOLD = np.float32(1.0)
SURROGATE_ALPHA = np.float32(5.0)
PHYSICAL_TAU_MS = np.float32(10.05)
BETA1 = np.float32(0.9)
BETA2 = np.float32(0.999)
ADAM_EPS = np.float32(1e-8)


@dataclass
class Weights:
    w_in: np.ndarray
    w_out: np.ndarray
    b_out: np.ndarray

    @property
    def hidden(self) -> int:
        return self.w_in.shape[0]

    @property
    def n_inputs(self) -> int:
        return self.w_in.shape[1]

    @property
    def n_classes(self) -> int:
        return self.w_out.shape[0]


@dataclass
class Gradient:
    w_in: np.ndarray
    w_out: np.ndarray
    b_out: np.ndarray

    def norm(self) -> float:
        return float(
            np.sqrt(
                np.sum(self.w_in * self.w_in, dtype=np.float64)
                + np.sum(self.w_out * self.w_out, dtype=np.float64)
                + np.sum(self.b_out * self.b_out, dtype=np.float64)
            )
        )

    def finite(self) -> bool:
        return all(
            np.all(np.isfinite(value)) for value in (self.w_in, self.w_out, self.b_out)
        )


@dataclass
class Forward:
    membrane: np.ndarray
    spikes: np.ndarray
    rates: np.ndarray
    logits: np.ndarray
    loss: float
    prediction: int


def load_weights(path: Path) -> Weights:
    with path.open("rb") as handle:
        if handle.read(8) != WEIGHT_MAGIC:
            raise ValueError(f"bad matched-weight magic in {path}")
        n_inputs, hidden, n_classes = struct.unpack("<III", handle.read(12))
        count = hidden * n_inputs + n_classes * hidden + n_classes
        values = np.frombuffer(handle.read(count * 4), dtype="<f4").copy()
    offset = hidden * n_inputs
    w_in = values[:offset].reshape(hidden, n_inputs)
    end = offset + n_classes * hidden
    w_out = values[offset:end].reshape(n_classes, hidden)
    b_out = values[end:]
    return Weights(w_in, w_out, b_out)


def load_orders(path: Path) -> np.ndarray:
    with path.open("rb") as handle:
        if handle.read(8) != ORDER_MAGIC:
            raise ValueError(f"bad epoch-order magic in {path}")
        epochs, n_items = struct.unpack("<II", handle.read(8))
        return np.frombuffer(handle.read(epochs * n_items * 4), dtype="<u4").copy().reshape(
            epochs, n_items
        )


def surrogate_derivative(values: np.ndarray) -> np.ndarray:
    scaled = np.float32(np.pi / 2) * SURROGATE_ALPHA * values
    return (SURROGATE_ALPHA * np.float32(0.5)) / (
        np.float32(1.0) + scaled * scaled
    )


def _frame_matrix(sample: FramedSample) -> np.ndarray:
    """Dense (steps, n_inputs) float32 frame matrix, from coordinates cached on the sample.

    The coordinate arrays are built once per sample and reused across every epoch;
    the dense matrix is rebuilt per call because caching it would cost ~1.8 MB per
    sample (~19 GB over the full 8156/2264 split at T=655).
    """
    coordinates = getattr(sample, "_coordinates", None)
    if coordinates is None:
        rows: list[int] = []
        cols: list[int] = []
        vals: list[np.float32] = []
        for time_index, frame in enumerate(sample.frames):
            for channel, count in frame:
                rows.append(time_index)
                cols.append(channel)
                vals.append(count)
        # uint16 is exact for both axes: step counts stay under 700 for every
        # registered contract and channel ids under 700 by construction. This
        # keeps the per-sample cache at 8 bytes/event instead of 20, which
        # matters because the cache is held for the whole 8156/2264 split.
        if sample.valid_steps > 65535 or sample.n_inputs > 65535:
            raise ValueError("frame coordinates exceed uint16 range")
        coordinates = (
            np.asarray(rows, dtype=np.uint16),
            np.asarray(cols, dtype=np.uint16),
            np.asarray(vals, dtype=np.float32),
        )
        object.__setattr__(sample, "_coordinates", coordinates)
    rows, cols, vals = coordinates
    matrix = np.zeros((sample.valid_steps, sample.n_inputs), dtype=np.float32)
    if len(rows):
        np.add.at(matrix, (rows, cols), vals)
    return matrix


def loss_and_gradient(weights: Weights, sample: FramedSample) -> tuple[Forward, Gradient]:
    if sample.n_inputs != weights.n_inputs:
        raise ValueError("sample/model input mismatch")
    steps = sample.valid_steps
    alpha = np.exp(np.float32(-sample.dt_ms / PHYSICAL_TAU_MS), dtype=np.float32)
    # Input drive carries no recurrence, so the whole sample collapses into one
    # matmul instead of a per-event python loop. The time loop below keeps only
    # the LIF recurrence, which is genuinely sequential.
    frame_matrix = _frame_matrix(sample)
    drive = frame_matrix @ weights.w_in.T
    membrane = np.zeros((steps, weights.hidden), dtype=np.float32)
    spikes = np.zeros_like(membrane)
    previous_u = np.zeros(weights.hidden, dtype=np.float32)
    previous_s = np.zeros(weights.hidden, dtype=np.float32)
    for time_index in range(steps):
        current = alpha * previous_u * (np.float32(1.0) - previous_s) + drive[time_index]
        spike = (current >= THRESHOLD).astype(np.float32)
        membrane[time_index] = current
        spikes[time_index] = spike
        previous_u = current
        previous_s = spike
    rates = np.mean(spikes, axis=0, dtype=np.float32)
    logits = weights.b_out + weights.w_out @ rates
    maximum = np.max(logits)
    probabilities = np.exp(logits - maximum, dtype=np.float32)
    probabilities /= np.sum(probabilities, dtype=np.float32)
    loss = float(-np.log(max(float(probabilities[sample.label]), 1e-30)))
    prediction = int(np.argmax(logits))
    probabilities[sample.label] -= np.float32(1.0)

    grad_w_out = probabilities[:, None] * rates[None, :]
    grad_b_out = probabilities.copy()
    direct_spike = (weights.w_out.T @ probabilities) / np.float32(steps)
    du_all = np.zeros((steps, weights.hidden), dtype=np.float32)
    du_next = np.zeros(weights.hidden, dtype=np.float32)
    for time_index in range(steps - 1, -1, -1):
        derivative = surrogate_derivative(membrane[time_index] - THRESHOLD)
        du = direct_spike * derivative + alpha * (
            np.float32(1.0) - spikes[time_index]
        ) * du_next
        du_all[time_index] = du
        du_next = du
    # Same collapse on the backward side: grad_w_in is one matmul over time.
    grad_w_in = du_all.T @ frame_matrix
    return (
        Forward(membrane, spikes, rates, logits, loss, prediction),
        Gradient(grad_w_in, grad_w_out, grad_b_out),
    )


class Adam:
    def __init__(self, weights: Weights):
        self.first = [
            np.zeros_like(weights.w_in),
            np.zeros_like(weights.w_out),
            np.zeros_like(weights.b_out),
        ]
        self.second = [np.zeros_like(value) for value in self.first]
        self.step = 0

    def update(
        self, weights: Weights, gradient: Gradient, lr: float, weight_decay: float
    ) -> float:
        self.step += 1
        values = [weights.w_in, weights.w_out, weights.b_out]
        gradients = [gradient.w_in, gradient.w_out, gradient.b_out]
        squared = 0.0
        count = 0
        for index, (value, grad) in enumerate(zip(values, gradients, strict=True)):
            decay = np.float32(0 if index == 2 else weight_decay)
            effective = grad + decay * value
            self.first[index] = BETA1 * self.first[index] + (np.float32(1) - BETA1) * effective
            self.second[index] = BETA2 * self.second[index] + (
                np.float32(1) - BETA2
            ) * effective * effective
            correction1 = np.float32(1.0 - float(BETA1) ** self.step)
            correction2 = np.float32(1.0 - float(BETA2) ** self.step)
            update = np.float32(lr) * (self.first[index] / correction1) / (
                np.sqrt(self.second[index] / correction2, dtype=np.float32) + ADAM_EPS
            )
            value -= update
            squared += float(np.sum(update * update, dtype=np.float64))
            count += update.size
        return math.sqrt(squared / max(1, count))


def one_cycle_lr(step: int, total_steps: int, base_lr: float = 1e-3, max_lr: float = 5e-3) -> float:
    if total_steps <= 1:
        return base_lr
    progress = step / (total_steps - 1)
    if progress <= 0.3:
        return base_lr + (max_lr - base_lr) * progress / 0.3
    final_lr = base_lr / 100
    return max_lr - (max_lr - final_lr) * (progress - 0.3) / 0.7


def evaluate(weights: Weights, samples: list[FramedSample]) -> dict[str, float | int]:
    correct = 0
    predictions = np.zeros(weights.n_classes, dtype=np.int64)
    unit_rates = np.zeros(weights.hidden, dtype=np.float64)
    for sample in samples:
        forward, _ = loss_and_gradient(weights, sample)
        correct += int(forward.prediction == sample.label)
        predictions[forward.prediction] += 1
        unit_rates += forward.rates
    unit_rates /= max(1, len(samples))
    return {
        "accuracy": correct / max(1, len(samples)),
        "classes_predicted": int(np.count_nonzero(predictions)),
        "majority_prediction": float(np.max(predictions) / max(1, len(samples))),
        "mean_firing_rate": float(np.mean(unit_rates)),
        "silent_fraction": float(np.mean(unit_rates <= 1e-6)),
        "saturated_fraction": float(np.mean(unit_rates >= 0.95)),
    }


def train_cell(
    train_events: Path,
    test_events: Path,
    contract: Contract,
    geometry: str,
    weights_path: Path,
    orders_path: Path,
    epochs: int,
    output: Path,
    max_train: int | None = None,
    max_test: int | None = None,
) -> dict[str, object]:
    started = time.monotonic()
    train = [
        frame_events(sample, contract, geometry)
        for sample in read_event_cache(train_events, max_train)
    ]
    test = [
        frame_events(sample, contract, geometry)
        for sample in read_event_cache(test_events, max_test)
    ]
    weights = load_weights(weights_path)
    orders = load_orders(orders_path)
    if len(orders) < epochs or orders.shape[1] != len(train):
        raise ValueError("epoch order shape mismatch")
    batch_size = 256
    total_steps = epochs * math.ceil(len(train) / batch_size)
    optimizer = Adam(weights)
    loss_sum = gradient_norm_sum = update_rms_sum = 0.0
    sample_count = optimizer_steps = non_finite = 0
    global_step = 0
    for epoch in range(epochs):
        for start in range(0, len(train), batch_size):
            indices = orders[epoch, start : start + batch_size]
            gradient = Gradient(
                np.zeros_like(weights.w_in),
                np.zeros_like(weights.w_out),
                np.zeros_like(weights.b_out),
            )
            for index in indices:
                forward, sample_gradient = loss_and_gradient(weights, train[int(index)])
                if not math.isfinite(forward.loss) or not sample_gradient.finite():
                    non_finite += 1
                    raise FloatingPointError(
                        f"non-finite training value at optimizer step {global_step}"
                    )
                loss_sum += forward.loss
                sample_count += 1
                gradient.w_in += sample_gradient.w_in
                gradient.w_out += sample_gradient.w_out
                gradient.b_out += sample_gradient.b_out
            scale = np.float32(1 / len(indices))
            gradient.w_in *= scale
            gradient.w_out *= scale
            gradient.b_out *= scale
            gradient_norm_sum += gradient.norm()
            update_rms_sum += optimizer.update(
                weights, gradient, one_cycle_lr(global_step, total_steps), 1e-5
            )
            optimizer_steps += 1
            global_step += 1
    evaluation = evaluate(weights, test)
    cell_pass = (
        evaluation["accuracy"] >= 0.80
        and evaluation["classes_predicted"] == weights.n_classes
        and evaluation["majority_prediction"] < 0.30
        and evaluation["silent_fraction"] <= 0.95
        and evaluation["saturated_fraction"] <= 0.05
        and non_finite == 0
    )
    payload: dict[str, object] = {
        "schema": "shd-cal-cell-v1",
        "backend": "python",
        "contract": contract.id,
        "geometry": geometry,
        "hidden": weights.hidden,
        "epochs": epochs,
        "n_train": len(train),
        "n_test": len(test),
        **evaluation,
        "mean_loss": loss_sum / max(1, sample_count),
        "mean_gradient_norm": gradient_norm_sum / max(1, optimizer_steps),
        "mean_update_rms": update_rms_sum / max(1, optimizer_steps),
        "non_finite_events": non_finite,
        "mechanical_status": "COMPLETE",
        "scientific_status": "CELL_PASS" if cell_pass else "CELL_FAIL",
        "wall_secs": time.monotonic() - started,
    }
    write_json_atomic(output, payload)
    return payload


def parity_payload(weights_path: Path, sample: FramedSample) -> dict[str, object]:
    weights = load_weights(weights_path)
    forward, gradient = loss_and_gradient(weights, sample)
    Adam(weights).update(weights, gradient, 1e-3, 1e-5)
    return {
        "frame_hash": sample.fingerprint(),
        "valid_steps": sample.valid_steps,
        "dt_ms": float(sample.dt_ms),
        "loss": forward.loss,
        "prediction": forward.prediction,
        "membrane": forward.membrane.ravel().tolist(),
        "spikes": forward.spikes.ravel().tolist(),
        "rates": forward.rates.tolist(),
        "logits": forward.logits.tolist(),
        "grad_w_in": gradient.w_in.ravel().tolist(),
        "grad_w_out": gradient.w_out.ravel().tolist(),
        "grad_b_out": gradient.b_out.tolist(),
        "updated_w_in": weights.w_in.ravel().tolist(),
        "updated_w_out": weights.w_out.ravel().tolist(),
        "updated_b_out": weights.b_out.tolist(),
    }
