"""Count-preserving SHD event cache and canonical framing."""

from __future__ import annotations

from dataclasses import dataclass
import json
import math
from pathlib import Path
import struct
from typing import Iterable, Iterator

import h5py
import numpy as np


EVENT_MAGIC = b"SHDEVT1\0"
FIXED_WINDOW_MS = np.float32(1400.0)
EVENT_HEADER = struct.Struct("<8sI")
SAMPLE_HEADER = struct.Struct("<II")
EVENT_RECORD = struct.Struct("<fHH")


@dataclass(frozen=True)
class EventSample:
    label: int
    times: np.ndarray
    channels: np.ndarray


@dataclass(frozen=True)
class Contract:
    kind: str
    resolution: int

    @property
    def id(self) -> str:
        if self.kind == "published":
            return f"published-{self.resolution}ms"
        if self.kind == "fixed":
            return f"fixed-t{self.resolution}"
        raise ValueError(self.kind)

    @property
    def dt_ms(self) -> np.float32:
        if self.kind == "published":
            if self.resolution not in (2, 4, 10):
                raise ValueError("published resolution must be 2, 4, or 10 ms")
            return np.float32(self.resolution)
        if self.kind == "fixed":
            if self.resolution not in (100, 250, 500):
                raise ValueError("fixed resolution must be T=100, 250, or 500")
            return np.float32(FIXED_WINDOW_MS / np.float32(self.resolution))
        raise ValueError(self.kind)


@dataclass
class FramedSample:
    label: int
    frames: list[list[tuple[int, np.float32]]]
    n_inputs: int
    dt_ms: np.float32
    original_events: int
    retained_events: int
    clipped_events: int
    first_time_s: np.float32
    last_time_s: np.float32

    @property
    def valid_steps(self) -> int:
        return len(self.frames)

    def fingerprint(self) -> str:
        value = 0xCBF29CE484222325

        def mix(raw: bytes) -> None:
            nonlocal value
            for byte in raw:
                value ^= byte
                value = (value * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF

        mix(struct.pack("<I", self.label))
        mix(struct.pack("<Q", len(self.frames)))
        mix(struct.pack("<Q", self.n_inputs))
        mix(struct.pack("<f", self.dt_ms))
        for time_index, frame in enumerate(self.frames):
            mix(struct.pack("<Q", time_index))
            for channel, count in frame:
                mix(struct.pack("<Qf", channel, count))
        return f"{value:016x}"


def write_event_cache(path: Path, samples: Iterable[EventSample], count: int) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(f".tmp-{__import__('os').getpid()}")
    with temporary.open("wb") as handle:
        handle.write(EVENT_HEADER.pack(EVENT_MAGIC, count))
        written = 0
        for sample in samples:
            times = np.asarray(sample.times, dtype="<f4")
            channels = np.asarray(sample.channels, dtype="<u2")
            if len(times) != len(channels):
                raise ValueError("event times/channels length mismatch")
            if sample.label < 0 or sample.label >= 20:
                raise ValueError(f"invalid SHD label {sample.label}")
            order = np.argsort(times, kind="stable")
            times = times[order]
            channels = channels[order]
            handle.write(SAMPLE_HEADER.pack(sample.label, len(times)))
            for time_s, channel in zip(times, channels, strict=True):
                handle.write(EVENT_RECORD.pack(float(time_s), int(channel), 0))
            written += 1
        if written != count:
            raise ValueError(f"declared {count} samples but wrote {written}")
        handle.flush()
        __import__("os").fsync(handle.fileno())
    temporary.replace(path)


def convert_h5_to_event_cache(h5_path: Path, output: Path) -> dict[str, object]:
    with h5py.File(h5_path, "r") as source:
        labels = np.asarray(source["labels"], dtype=np.int64)
        times = source["spikes"]["times"]
        units = source["spikes"]["units"]

        def samples() -> Iterator[EventSample]:
            for index, label in enumerate(labels):
                yield EventSample(
                    int(label),
                    np.asarray(times[index], dtype=np.float32),
                    np.asarray(units[index], dtype=np.uint16),
                )

        write_event_cache(output, samples(), len(labels))
    return {
        "source": str(h5_path),
        "output": str(output),
        "samples": int(len(labels)),
    }


def read_event_cache(path: Path, max_samples: int | None = None) -> list[EventSample]:
    samples: list[EventSample] = []
    with path.open("rb") as handle:
        magic, count = EVENT_HEADER.unpack(handle.read(EVENT_HEADER.size))
        if magic != EVENT_MAGIC:
            raise ValueError(f"bad event cache magic in {path}")
        limit = count if max_samples is None else min(count, max_samples)
        for index in range(count):
            label, n_events = SAMPLE_HEADER.unpack(handle.read(SAMPLE_HEADER.size))
            if index < limit:
                raw = handle.read(n_events * EVENT_RECORD.size)
                records = np.frombuffer(
                    raw,
                    dtype=np.dtype([("time", "<f4"), ("channel", "<u2"), ("reserved", "<u2")]),
                )
                samples.append(
                    EventSample(label, records["time"].copy(), records["channel"].copy())
                )
            else:
                handle.seek(n_events * EVENT_RECORD.size, 1)
    return samples


def frame_events(sample: EventSample, contract: Contract, geometry: str) -> FramedSample:
    if geometry not in ("channels-700", "adjacent-sum-5"):
        raise ValueError(geometry)
    n_inputs = 700 if geometry == "channels-700" else 140
    times = np.asarray(sample.times, dtype=np.float32)
    channels = np.asarray(sample.channels, dtype=np.uint16)
    first = np.float32(times[0]) if len(times) else np.float32(0)
    last = np.float32(times[-1]) if len(times) else np.float32(0)
    if contract.kind == "published":
        dt_s = np.float32(contract.dt_ms / np.float32(1000.0))
        shifted_last = np.maximum(np.float32(0), np.float32(last - first))
        steps = 1 if not len(times) else int(np.floor(np.float32(shifted_last / dt_s))) + 1
        origin = first
    else:
        steps = contract.resolution
        dt_s = np.float32(
            np.float32(FIXED_WINDOW_MS / np.float32(1000.0))
            / np.float32(contract.resolution)
        )
        origin = np.float32(0)

    # Vectorised binning (G3). The scalar loop this replaces cost ~10^8 python
    # iterations per cell across the 8156/2264 split and was redone for every
    # cell. Every arithmetic step below stays in float32 and in the same order
    # as the original scalar expression, so the framing - and therefore
    # `fingerprint()` - is unchanged. `verify_framing_equivalence` asserts that.
    n_frames = max(1, steps)
    total_events = len(times)
    channel_index = channels.astype(np.int64, copy=False)
    keep = channel_index < 700
    if geometry == "adjacent-sum-5":
        channel_index = channel_index // 5
    # np.float32(np.float32(time_s) - origin): times is already float32.
    shifted = (times - origin).astype(np.float32, copy=False)
    keep &= np.isfinite(shifted) & (shifted >= np.float32(0))
    # int(np.floor(np.float32(shifted / dt_s))) - the quotient is float32
    # because both operands are, so the floor lands on the same bin.
    with np.errstate(invalid="ignore", divide="ignore"):
        frame_index = np.floor(shifted / dt_s)
    keep &= np.isfinite(frame_index)
    frame_index = np.where(keep, frame_index, np.float32(-1)).astype(np.int64)
    keep &= (frame_index >= 0) & (frame_index < n_frames)

    retained = int(np.count_nonzero(keep))
    # Each rejected event increments `clipped` exactly once in the scalar form,
    # whichever guard rejects it first.
    clipped = int(total_events - retained)

    sparse: list[list[tuple[int, np.float32]]] = [[] for _ in range(n_frames)]
    if retained:
        # key = frame * n_inputs + channel. np.unique returns keys ascending,
        # which is exactly `sorted(frame.items())` grouped by frame.
        keys = frame_index[keep] * n_inputs + channel_index[keep]
        unique_keys, counts = np.unique(keys, return_counts=True)
        unique_frames = (unique_keys // n_inputs).tolist()
        unique_channels = (unique_keys % n_inputs).tolist()
        # Counts are small integers, so float32 is exact and matches the
        # repeated `+ np.float32(1)` accumulation of the scalar form.
        counts_f32 = counts.astype(np.float32)
        for frame_position, channel_position, count in zip(
            unique_frames, unique_channels, counts_f32
        ):
            sparse[frame_position].append((int(channel_position), count))
    return FramedSample(
        sample.label,
        sparse,
        n_inputs,
        contract.dt_ms,
        len(times),
        retained,
        clipped,
        first,
        last,
    )


def fixture_samples() -> list[EventSample]:
    output = []
    for label in range(20):
        offset = np.float32(label * 0.00001)
        output.append(
            EventSample(
                label,
                np.asarray(
                    [0.100, 0.100, 0.111, 0.121, 1.399, 1.401], dtype=np.float32
                )
                + offset,
                np.asarray([0, 0, 4, 5, 699, 699], dtype=np.uint16),
            )
        )
    return output


def write_fixture_cache(path: Path) -> None:
    samples = fixture_samples()
    write_event_cache(path, samples, len(samples))


def corpus_summary(
    samples: list[EventSample], contract: Contract, geometry: str, split: str
) -> dict[str, object]:
    total = retained = clipped = 0
    collisions = 0
    durations: list[float] = []
    class_total = [0] * 20
    class_retained = [0] * 20
    for sample in samples:
        times = np.asarray(sample.times, dtype=np.float32)
        channels = np.asarray(sample.channels, dtype=np.int64)
        n_events = len(times)
        first = np.float32(times[0]) if n_events else np.float32(0)
        last = np.float32(times[-1]) if n_events else np.float32(0)
        durations.append(float(last - first))
        if contract.kind == "published":
            dt_s = np.float32(contract.dt_ms / np.float32(1000.0))
            shifted = np.asarray(times - first, dtype=np.float32)
            steps = (
                1
                if not n_events
                else int(
                    np.floor(
                        np.float32(np.maximum(np.float32(0), last - first) / dt_s)
                    )
                )
                + 1
            )
        else:
            dt_s = np.float32(
                np.float32(FIXED_WINDOW_MS / np.float32(1000.0))
                / np.float32(contract.resolution)
            )
            shifted = times
            steps = contract.resolution
        frame_indices = np.floor(
            np.asarray(shifted / dt_s, dtype=np.float32)
        ).astype(np.int64)
        valid = (
            np.isfinite(shifted)
            & (shifted >= 0)
            & (frame_indices >= 0)
            & (frame_indices < steps)
            & (channels >= 0)
            & (channels < 700)
        )
        mapped_channels = channels[valid]
        if geometry == "adjacent-sum-5":
            mapped_channels = mapped_channels // 5
            n_inputs = 140
        elif geometry == "channels-700":
            n_inputs = 700
        else:
            raise ValueError(geometry)
        kept_frames = frame_indices[valid]
        n_retained = int(np.count_nonzero(valid))
        occupied = (
            int(np.unique(kept_frames * n_inputs + mapped_channels).size)
            if n_retained
            else 0
        )
        total += n_events
        retained += n_retained
        clipped += n_events - n_retained
        collisions += n_retained - occupied
        class_total[sample.label] += n_events
        class_retained[sample.label] += n_retained
    class_retention = [
        (class_retained[index] / class_total[index]) if class_total[index] else 1.0
        for index in range(20)
    ]
    return {
        "split": split,
        "contract": contract.id,
        "geometry": geometry,
        "samples": len(samples),
        "total_events": total,
        "retained_events": retained,
        "clipped_events": clipped,
        "collision_events": collisions,
        "retained_fraction": retained / max(1, total),
        "collision_fraction": collisions / max(1, retained),
        "duration_min_s": min(durations, default=0.0),
        "duration_max_s": max(durations, default=0.0),
        "duration_mean_s": sum(durations) / max(1, len(durations)),
        "class_retention": class_retention,
    }


def write_json_atomic(path: Path, payload: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(f".tmp-{__import__('os').getpid()}")
    with temporary.open("w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")
        handle.flush()
        __import__("os").fsync(handle.fileno())
    temporary.replace(path)
