#!/usr/bin/env python3
"""Independent NumPy implementation of the BINNTRF1 transfer specification.

This file does not import Rust code or generated dynamics. It reads the frozen
binary bundle and implements event delivery, selection, threshold/reset,
eligibility traces, readout learning, and local feedback updates directly.
"""

from __future__ import annotations

import argparse
import heapq
import json
import struct
from dataclasses import dataclass
from pathlib import Path
from typing import BinaryIO

import numpy as np

MAGIC = b"BINNTRF1"
VERSION = 1
N_IN = 32
TIMESTEPS = 32
N_CLASSES = 4
KWTA_K = 8
TRACE_TAU_E = np.float32(40.0)


@dataclass(frozen=True)
class Pole:
    selection: str
    timing: str
    threshold_reset: str
    trace: str

    @staticmethod
    def matched() -> "Pole":
        return Pole("all", "sync", "fixed_soft", "held")

    @staticmethod
    def live() -> "Pole":
        return Pole("kwta", "event", "adaptive_hard", "decay")


@dataclass
class Example:
    frames: np.ndarray
    label: int


@dataclass
class Bundle:
    seed: int
    hidden: int
    train: list[Example]
    test: list[Example]
    delays: np.ndarray
    input_weights: np.ndarray
    feedback: np.ndarray
    readout: np.ndarray
    readout_bias: np.ndarray


def _read_exact(stream: BinaryIO, count: int) -> bytes:
    data = stream.read(count)
    if len(data) != count:
        raise ValueError("truncated BINNTRF1 bundle")
    return data


def _u32(stream: BinaryIO) -> int:
    return struct.unpack("<I", _read_exact(stream, 4))[0]


def _u64(stream: BinaryIO) -> int:
    return struct.unpack("<Q", _read_exact(stream, 8))[0]


def _f32_array(stream: BinaryIO, count: int) -> np.ndarray:
    return np.frombuffer(_read_exact(stream, count * 4), dtype="<f4").copy()


def read_bundle(path: Path) -> Bundle:
    with path.open("rb") as stream:
        if _read_exact(stream, 8) != MAGIC:
            raise ValueError("invalid BINNTRF1 magic")
        version = _u32(stream)
        if version != VERSION:
            raise ValueError(f"unsupported BINNTRF1 version {version}")
        seed = _u64(stream)
        n_in = _u32(stream)
        timesteps = _u32(stream)
        n_classes = _u32(stream)
        hidden = _u32(stream)
        n_train = _u32(stream)
        n_test = _u32(stream)
        if (n_in, timesteps, n_classes) != (N_IN, TIMESTEPS, N_CLASSES):
            raise ValueError("BINNTRF1 task dimensions do not match protocol")

        def examples(count: int) -> list[Example]:
            result = []
            for _ in range(count):
                label = _u32(stream)
                frames = _f32_array(stream, N_IN * TIMESTEPS).reshape(
                    TIMESTEPS, N_IN
                )
                result.append(Example(frames, label))
            return result

        train = examples(n_train)
        test = examples(n_test)
        delays = np.frombuffer(
            _read_exact(stream, hidden * N_IN * 4), dtype="<u4"
        ).copy()
        input_weights = _f32_array(stream, hidden * N_IN)
        feedback = _f32_array(stream, hidden * N_CLASSES)
        readout = _f32_array(stream, N_CLASSES * hidden)
        readout_bias = _f32_array(stream, N_CLASSES)
        if stream.read(1):
            raise ValueError("BINNTRF1 bundle has trailing bytes")
    return Bundle(
        seed,
        hidden,
        train,
        test,
        delays,
        input_weights,
        feedback,
        readout,
        readout_bias,
    )


def sigmoid(value: np.float32) -> np.float32:
    value = np.float32(value)
    if value >= 0:
        return np.float32(1.0) / (
            np.float32(1.0) + np.exp(np.float32(-value), dtype=np.float32)
        )
    exp_value = np.exp(value, dtype=np.float32)
    return exp_value / (np.float32(1.0) + exp_value)


def softmax(logits: np.ndarray) -> np.ndarray:
    shifted = logits - np.max(logits)
    values = np.exp(shifted, dtype=np.float32)
    return values / np.maximum(np.sum(values, dtype=np.float32), np.float32(1e-12))


def select_spikes(
    membrane: np.ndarray, thresholds: np.ndarray, selection: str
) -> np.ndarray:
    if selection == "all":
        return np.asarray(
            [
                sigmoid(np.float32(5.0) * (value - threshold))
                for value, threshold in zip(membrane, thresholds, strict=True)
            ],
            dtype=np.float32,
        )
    if selection != "kwta":
        raise ValueError(f"unknown selection {selection}")
    # Primary key descending membrane, secondary key ascending unit.
    order = sorted(range(len(membrane)), key=lambda unit: (-float(membrane[unit]), unit))
    spikes = np.zeros_like(membrane)
    for unit in order[:KWTA_K]:
        if membrane[unit] >= thresholds[unit]:
            spikes[unit] = np.float32(1.0)
    return spikes


class TransferModel:
    def __init__(self, bundle: Bundle):
        self.hidden = bundle.hidden
        self.input_weights = bundle.input_weights.astype(np.float32, copy=True)
        self.feedback = bundle.feedback.astype(np.float32, copy=True)
        self.readout = bundle.readout.astype(np.float32, copy=True)
        self.readout_bias = bundle.readout_bias.astype(np.float32, copy=True)
        self.delays = bundle.delays.astype(np.uint32, copy=True)

    def forward(self, example: Example, pole: Pole) -> dict:
        max_delay = int(np.max(self.delays)) if pole.timing == "event" else 0
        total_ticks = TIMESTEPS + max_delay
        queue: list[tuple[int, int, int, np.float32]] = []
        insertion = 0
        if pole.timing == "event":
            for tick in range(TIMESTEPS):
                for input_index in range(N_IN):
                    count = np.float32(example.frames[tick, input_index])
                    if count == 0:
                        continue
                    repetitions = int(round(float(count)))
                    if abs(float(count) - repetitions) >= 1e-6:
                        raise ValueError("transfer events require integer spike counts")
                    for unit in range(self.hidden):
                        edge = unit * N_IN + input_index
                        delivery = tick + int(self.delays[edge])
                        for _ in range(repetitions):
                            amount = np.float32(self.input_weights[edge])
                            heapq.heappush(
                                queue, (delivery, insertion, edge, amount)
                            )
                            insertion += 1

        membrane = np.zeros(self.hidden, dtype=np.float32)
        thresholds = np.ones(self.hidden, dtype=np.float32)
        rates = np.zeros(self.hidden, dtype=np.float32)
        eligibility = np.zeros(self.hidden * N_IN, dtype=np.float32)
        event_ticks: list[int] = []
        recipients: list[int] = []
        winners_by_tick: list[list[int]] = []
        alpha = np.exp(np.float32(-0.1), dtype=np.float32)
        threshold_decay = np.exp(np.float32(-0.05), dtype=np.float32)
        trace_decay = np.exp(
            np.float32(-1.0) / TRACE_TAU_E, dtype=np.float32
        )

        for tick in range(total_ticks):
            if pole.trace == "decay":
                eligibility *= trace_decay
            current = np.zeros(self.hidden, dtype=np.float32)
            pre = np.zeros(self.hidden * N_IN, dtype=np.float32)
            if pole.timing == "sync" and tick < TIMESTEPS:
                for unit in range(self.hidden):
                    for input_index in range(N_IN):
                        count = np.float32(example.frames[tick, input_index])
                        if count != 0:
                            edge = unit * N_IN + input_index
                            current[unit] = np.float32(
                                current[unit] + self.input_weights[edge] * count
                            )
                            pre[edge] = np.float32(pre[edge] + count)
            elif pole.timing == "event":
                while queue and queue[0][0] == tick:
                    delivery, _, edge, amount = heapq.heappop(queue)
                    unit = edge // N_IN
                    current[unit] = np.float32(current[unit] + amount)
                    pre[edge] = np.float32(pre[edge] + 1.0)
                    event_ticks.append(delivery)
                    recipients.append(unit)

            for unit in range(self.hidden):
                if pole.threshold_reset == "adaptive_hard":
                    thresholds[unit] = np.float32(
                        1.0 + (thresholds[unit] - 1.0) * threshold_decay
                    )
                membrane[unit] = np.float32(
                    alpha * membrane[unit] + current[unit]
                )

            spikes = select_spikes(membrane, thresholds, pole.selection)
            winners_by_tick.append(
                [int(unit) for unit in np.flatnonzero(spikes > np.float32(0.5))]
            )
            for unit in range(self.hidden):
                rates[unit] = np.float32(
                    rates[unit] + spikes[unit] / np.float32(total_ticks)
                )
                if pole.threshold_reset == "fixed_soft":
                    membrane[unit] = np.float32(
                        membrane[unit] - thresholds[unit] * spikes[unit]
                    )
                elif spikes[unit] > np.float32(0.5):
                    membrane[unit] = np.float32(0.0)
                    thresholds[unit] = np.float32(thresholds[unit] + 0.2)
                derivative_sigmoid = sigmoid(
                    np.float32(5.0) * (membrane[unit] - thresholds[unit])
                )
                derivative = np.float32(
                    5.0 * derivative_sigmoid * (1.0 - derivative_sigmoid)
                )
                start = unit * N_IN
                eligibility[start : start + N_IN] += (
                    pre[start : start + N_IN] * derivative
                )

        logits = self.readout_bias.copy()
        for class_index in range(N_CLASSES):
            for unit in range(self.hidden):
                logits[class_index] = np.float32(
                    logits[class_index]
                    + self.readout[class_index * self.hidden + unit] * rates[unit]
                )
        return {
            "rates": rates,
            "logits": logits,
            "eligibility": eligibility,
            "event_ticks": event_ticks,
            "recipients": recipients,
            "winners_by_tick": winners_by_tick,
            "final_thresholds": thresholds,
        }

    def apply_update(self, label: int, forward: dict, lr: np.float32) -> np.ndarray:
        delta = softmax(forward["logits"])
        delta[label] = np.float32(delta[label] - 1.0)
        before = self.input_weights.copy()
        for class_index in range(N_CLASSES):
            for unit in range(self.hidden):
                index = class_index * self.hidden + unit
                self.readout[index] = np.float32(
                    self.readout[index]
                    - lr * delta[class_index] * forward["rates"][unit]
                )
            self.readout_bias[class_index] = np.float32(
                self.readout_bias[class_index] - lr * delta[class_index]
            )
        for unit in range(self.hidden):
            modulator = np.float32(0.0)
            for class_index in range(N_CLASSES):
                modulator = np.float32(
                    modulator
                    + self.feedback[unit * N_CLASSES + class_index]
                    * (-delta[class_index])
                )
            start = unit * N_IN
            for input_index in range(N_IN):
                edge = start + input_index
                self.input_weights[edge] = np.float32(
                    self.input_weights[edge]
                    + lr * modulator * forward["eligibility"][edge]
                )
        return self.input_weights - before

    def micro_step(self, example: Example, pole: Pole, lr: float) -> dict:
        forward = self.forward(example, pole)
        prediction = int(np.argmax(forward["logits"]))
        weight_delta = self.apply_update(
            example.label, forward, np.float32(lr)
        )
        return {
            "event_ticks": forward["event_ticks"],
            "recipients": forward["recipients"],
            "winners_by_tick": forward["winners_by_tick"],
            "prediction": prediction,
            "eligibility": forward["eligibility"].astype(float).tolist(),
            "weight_delta": weight_delta.astype(float).tolist(),
            "final_thresholds": forward["final_thresholds"].astype(float).tolist(),
        }

    def train(self, examples: list[Example], pole: Pole, epochs: int, lr: float) -> None:
        for _ in range(epochs):
            for example in examples:
                forward = self.forward(example, pole)
                self.apply_update(example.label, forward, np.float32(lr))

    def evaluate(self, examples: list[Example], pole: Pole) -> dict:
        before = (
            self.input_weights.tobytes(),
            self.readout.tobytes(),
            self.readout_bias.tobytes(),
        )
        predictions = [
            int(np.argmax(self.forward(example, pole)["logits"]))
            for example in examples
        ]
        accuracy = sum(
            prediction == example.label
            for prediction, example in zip(predictions, examples, strict=True)
        ) / len(examples)
        counts = np.bincount(predictions, minlength=N_CLASSES)
        after = (
            self.input_weights.tobytes(),
            self.readout.tobytes(),
            self.readout_bias.tobytes(),
        )
        return {
            "accuracy": accuracy,
            "predictions": predictions,
            "n_distinct_predicted": int(np.count_nonzero(counts)),
            "majority_pred_frac": float(np.max(counts) / len(examples)),
            "no_test_update": before == after,
        }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--bundle", required=True, type=Path)
    parser.add_argument("--pole", required=True, choices=("matched", "live"))
    parser.add_argument("--micro", action="store_true")
    parser.add_argument("--epochs", type=int, default=20)
    parser.add_argument("--lr", type=float, default=0.005)
    parser.add_argument("--out", type=Path)
    parser.add_argument("--compare-rust", type=Path)
    parser.add_argument("--accuracy-only", action="store_true")
    args = parser.parse_args()

    bundle = read_bundle(args.bundle)
    pole = Pole.matched() if args.pole == "matched" else Pole.live()
    model = TransferModel(bundle)
    if args.micro:
        result = model.micro_step(bundle.train[0], pole, args.lr)
    else:
        model.train(bundle.train, pole, args.epochs, args.lr)
        result = model.evaluate(bundle.test, pole)
    if args.compare_rust:
        expected = json.loads(args.compare_rust.read_text(encoding="utf-8"))
        compare_micro(expected, result)
    if args.accuracy_only:
        if args.micro:
            raise ValueError("--accuracy-only is only valid for scientific evaluation")
        text = f"{result['accuracy']:.9f}"
    else:
        text = json.dumps(result, sort_keys=True, separators=(",", ":"))
    if args.out:
        args.out.write_text(text + "\n", encoding="utf-8")
    else:
        print(text)
    return 0


def compare_micro(expected: dict, actual: dict, tolerance: float = 1e-6) -> None:
    """Raise when a Rust/NumPy micro trace diverges."""
    for key in ("event_ticks", "recipients", "winners_by_tick", "prediction"):
        if expected[key] != actual[key]:
            raise ValueError(f"micro-conformance mismatch in {key}")
    for key in ("eligibility", "weight_delta", "final_thresholds"):
        left = np.asarray(expected[key], dtype=np.float64)
        right = np.asarray(actual[key], dtype=np.float64)
        if left.shape != right.shape:
            raise ValueError(f"micro-conformance shape mismatch in {key}")
        error = float(np.max(np.abs(left - right), initial=0.0))
        if error > tolerance:
            raise ValueError(
                f"micro-conformance mismatch in {key}: max error {error} > {tolerance}"
            )


if __name__ == "__main__":
    raise SystemExit(main())
