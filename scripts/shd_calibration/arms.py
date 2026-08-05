"""Four-arm extension of the matched SHD BPTT instrument (G5/G6).

Arms: ff+fixed, ff+alif, rec+fixed, rec+alif.

`model.loss_and_gradient` stays the untouched ff+fixed reference. This module
implements the general form and **must** reproduce it bit-identically at
`ff+fixed` — that is Gate F at the algebra level, checked by `selftest()`.

Forward
    u_h(t)  = alpha * u_h(t-1) * (1 - s_h(t-1))          # detached reset
              + sum_c w_in[h,c] * count_c(t)
              + sum_j w_rec[h,j] * s_j(t-1)              # recurrent arms only
    a_h(t)  = rho * a_h(t-1) + s_h(t-1)                  # alif arms only
    th_h(t) = THRESHOLD + beta_a * a_h(t)                # alif arms only
    s_h(t)  = 1[u_h(t) >= th_h(t)]

Backward (reset stays detached, matching the shipped reference)
    ds_j(t) = direct_spike_j
              + sum_h du_next_h * w_rec[h,j]             # recurrent arms only
              + da_next_j                                # alif arms only
    g_j(t)  = ds_j(t) * surrogate'(u_j(t) - th_j(t))
    du_h(t) = g_h(t) + alpha * (1 - s_h(t)) * du_next_h
    da_h(t) = -beta_a * g_h(t) + rho * da_next_h         # alif arms only

    grad_w_in [h,c] += du_h(t) * count_c(t)
    grad_w_rec[h,j] += du_h(t) * s_j(t-1)                # zero diagonal enforced

With `recurrent=False, adaptive=False` every guarded term drops out and the
expressions collapse term-for-term onto `model.loss_and_gradient`.
"""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np

from .data import FramedSample
from .model import (
    Forward,
    Gradient,
    THRESHOLD,
    PHYSICAL_TAU_MS,
    Weights,
    _frame_matrix,
    surrogate_derivative,
)

# Matches binn-learn/src/shd_alif.rs DEFAULT_TAU_A / DEFAULT_BETA_A, and
# rho = exp(-1/tau_a) as computed at shd_alif.rs:271.
DEFAULT_TAU_A = np.float32(20.0)
DEFAULT_BETA_A = np.float32(0.18)

ARMS = ("ff+fixed", "ff+alif", "rec+fixed", "rec+alif")


@dataclass(frozen=True)
class Arm:
    recurrent: bool
    adaptive: bool

    @property
    def label(self) -> str:
        return f"{'rec' if self.recurrent else 'ff'}+{'alif' if self.adaptive else 'fixed'}"

    @staticmethod
    def from_label(label: str) -> "Arm":
        mapping = {
            "ff+fixed": Arm(False, False),
            "ff+alif": Arm(False, True),
            "rec+fixed": Arm(True, False),
            "rec+alif": Arm(True, True),
        }
        if label not in mapping:
            raise ValueError(f"unknown arm {label!r}; expected one of {ARMS}")
        return mapping[label]


@dataclass
class ArmWeights:
    """`Weights` plus the recurrent block. `w_rec` is None for feed-forward arms."""

    base: Weights
    w_rec: np.ndarray | None = None
    tau_a: np.float32 = DEFAULT_TAU_A
    beta_a: np.float32 = DEFAULT_BETA_A

    @property
    def hidden(self) -> int:
        return self.base.hidden

    @property
    def n_inputs(self) -> int:
        return self.base.n_inputs

    def enforce_zero_diagonal(self) -> None:
        if self.w_rec is not None:
            np.fill_diagonal(self.w_rec, np.float32(0.0))


@dataclass
class ArmGradient:
    base: Gradient
    w_rec: np.ndarray | None = None


def loss_and_gradient(
    weights: ArmWeights,
    sample: FramedSample,
    arm: Arm,
    _capture: dict | None = None,
) -> tuple[Forward, ArmGradient]:
    if sample.n_inputs != weights.n_inputs:
        raise ValueError("sample/model input mismatch")
    if arm.recurrent and weights.w_rec is None:
        raise ValueError(f"arm {arm.label} requires w_rec")
    base = weights.base
    hidden = base.hidden
    steps = sample.valid_steps
    alpha = np.exp(np.float32(-sample.dt_ms / PHYSICAL_TAU_MS), dtype=np.float32)
    rho = np.exp(np.float32(-1.0) / weights.tau_a, dtype=np.float32)
    beta_a = weights.beta_a

    frame_matrix = _frame_matrix(sample)
    drive = frame_matrix @ base.w_in.T

    membrane = np.zeros((steps, hidden), dtype=np.float32)
    spikes = np.zeros_like(membrane)
    thresholds = np.zeros_like(membrane)
    previous_spikes = np.zeros((steps, hidden), dtype=np.float32)

    previous_u = np.zeros(hidden, dtype=np.float32)
    previous_s = np.zeros(hidden, dtype=np.float32)
    adaptation = np.zeros(hidden, dtype=np.float32)

    for time_index in range(steps):
        current = alpha * previous_u * (np.float32(1.0) - previous_s) + drive[time_index]
        if arm.recurrent:
            current = current + weights.w_rec @ previous_s
        if arm.adaptive:
            adaptation = rho * adaptation + previous_s
            threshold = THRESHOLD + beta_a * adaptation
        else:
            threshold = THRESHOLD
        spike = (current >= threshold).astype(np.float32)
        membrane[time_index] = current
        spikes[time_index] = spike
        thresholds[time_index] = threshold
        previous_spikes[time_index] = previous_s
        previous_u = current
        previous_s = spike

    rates = np.mean(spikes, axis=0, dtype=np.float32)
    logits = base.b_out + base.w_out @ rates
    maximum = np.max(logits)
    probabilities = np.exp(logits - maximum, dtype=np.float32)
    probabilities /= np.sum(probabilities, dtype=np.float32)
    loss = float(-np.log(max(float(probabilities[sample.label]), 1e-30)))
    prediction = int(np.argmax(logits))
    probabilities[sample.label] -= np.float32(1.0)

    grad_w_out = probabilities[:, None] * rates[None, :]
    grad_b_out = probabilities.copy()
    direct_spike = (base.w_out.T @ probabilities) / np.float32(steps)

    du_all = np.zeros((steps, hidden), dtype=np.float32)
    du_next = np.zeros(hidden, dtype=np.float32)
    da_next = np.zeros(hidden, dtype=np.float32)
    grad_w_rec = (
        np.zeros((hidden, hidden), dtype=np.float32) if arm.recurrent else None
    )

    for time_index in range(steps - 1, -1, -1):
        ds = direct_spike
        if arm.recurrent:
            ds = ds + weights.w_rec.T @ du_next
        if arm.adaptive:
            ds = ds + da_next
        surrogate = surrogate_derivative(membrane[time_index] - thresholds[time_index])
        gated = ds * surrogate
        du = gated + alpha * (np.float32(1.0) - spikes[time_index]) * du_next
        if arm.adaptive:
            da_next = -beta_a * gated + rho * da_next
        if arm.recurrent:
            grad_w_rec += np.outer(du, previous_spikes[time_index])
        du_all[time_index] = du
        du_next = du

    grad_w_in = du_all.T @ frame_matrix
    if grad_w_rec is not None:
        np.fill_diagonal(grad_w_rec, np.float32(0.0))
    if _capture is not None:
        _capture["thresholds"] = thresholds
        _capture["previous_spikes"] = previous_spikes
        _capture["direct_spike"] = direct_spike

    return (
        Forward(membrane, spikes, rates, logits, loss, prediction),
        ArmGradient(Gradient(grad_w_in, grad_w_out, grad_b_out), grad_w_rec),
    )


# --------------------------------------------------------------------------
# selftest: Gate F algebra check + finite-difference gradcheck
# --------------------------------------------------------------------------


def _numeric_gradient(weights, sample, arm, which, index, eps=1e-3):
    """Central difference on one scalar entry.

    Only meaningful for `w_out` / `b_out`: those reach the loss through the
    softmax with the spike train held fixed, so the loss is genuinely smooth in
    them. `w_in` / `w_rec` reach the loss only through a hard threshold, whose
    finite difference is zero almost everywhere — surrogate gradients are not
    the true gradient there, and finite differences cannot validate them. Those
    paths are covered by `_naive_backward` instead.
    """

    def perturbed(delta):
        base = Weights(
            weights.base.w_in.copy(), weights.base.w_out.copy(), weights.base.b_out.copy()
        )
        rec = None if weights.w_rec is None else weights.w_rec.copy()
        target = {"w_in": base.w_in, "w_out": base.w_out, "b_out": base.b_out, "w_rec": rec}[which]
        target[index] = np.float32(target[index] + delta)
        forward, _ = loss_and_gradient(
            ArmWeights(base, rec, weights.tau_a, weights.beta_a), sample, arm
        )
        return forward.loss

    return (perturbed(eps) - perturbed(-eps)) / (2 * eps)


def _naive_backward(weights, sample, arm, forward, thresholds, previous_spikes, direct_spike):
    """Deliberately slow scalar-loop backward, independent of the vectorised one.

    Written to be obviously correct rather than fast. Its job is to catch the
    realistic bug class in the vectorised path: a transposed `w_rec`, a
    mis-indexed `s(t-1)` in the recurrent outer product, or an adaptation trace
    that carries with the wrong sign.
    """
    hidden = weights.hidden
    steps = sample.valid_steps
    alpha = float(np.exp(np.float32(-sample.dt_ms / PHYSICAL_TAU_MS), dtype=np.float32))
    rho = float(np.exp(np.float32(-1.0) / weights.tau_a, dtype=np.float32))
    beta_a = float(weights.beta_a)

    grad_w_in = np.zeros_like(weights.base.w_in, dtype=np.float64)
    grad_w_rec = np.zeros((hidden, hidden), dtype=np.float64) if arm.recurrent else None
    du_next = np.zeros(hidden, dtype=np.float64)
    da_next = np.zeros(hidden, dtype=np.float64)

    for t in range(steps - 1, -1, -1):
        du = np.zeros(hidden, dtype=np.float64)
        da = np.zeros(hidden, dtype=np.float64)
        for h in range(hidden):
            ds = float(direct_spike[h])
            if arm.recurrent:
                for j in range(hidden):
                    ds += float(du_next[j]) * float(weights.w_rec[j, h])
            if arm.adaptive:
                ds += float(da_next[h])
            surrogate = float(
                surrogate_derivative(
                    np.float32(forward.membrane[t, h] - thresholds[t, h])
                )
            )
            gated = ds * surrogate
            du[h] = gated + alpha * (1.0 - float(forward.spikes[t, h])) * float(du_next[h])
            if arm.adaptive:
                da[h] = -beta_a * gated + rho * float(da_next[h])
            for channel, count in sample.frames[t]:
                grad_w_in[h, channel] += du[h] * float(count)
            if arm.recurrent:
                for j in range(hidden):
                    grad_w_rec[h, j] += du[h] * float(previous_spikes[t, j])
        du_next = du
        da_next = da

    if grad_w_rec is not None:
        np.fill_diagonal(grad_w_rec, 0.0)
    return grad_w_in, grad_w_rec


def selftest(verbose: bool = True) -> bool:
    """Gate F algebra + gradcheck. Returns True when every check passes."""
    from .model import loss_and_gradient as reference_loss_and_gradient

    rng = np.random.default_rng(11)
    hidden, n_inputs, steps, n_classes = 24, 40, 30, 20
    frames = [
        [(int(c), np.float32(1.0)) for c in rng.choice(n_inputs, 6, replace=False)]
        for _ in range(steps)
    ]
    sample = FramedSample(
        label=7,
        frames=frames,
        n_inputs=n_inputs,
        dt_ms=np.float32(10.0),
        original_events=0,
        retained_events=0,
        clipped_events=0,
        first_time_s=np.float32(0.0),
        last_time_s=np.float32(1.0),
    )
    base = Weights(
        rng.standard_normal((hidden, n_inputs)).astype(np.float32) * np.float32(0.30),
        rng.standard_normal((n_classes, hidden)).astype(np.float32) * np.float32(0.30),
        np.zeros(n_classes, dtype=np.float32),
    )
    w_rec = rng.standard_normal((hidden, hidden)).astype(np.float32) * np.float32(0.05)
    np.fill_diagonal(w_rec, np.float32(0.0))

    ok = True

    # --- Gate F: ff+fixed must be bit-identical to the shipped reference ---
    reference_forward, reference_gradient = reference_loss_and_gradient(base, sample)
    arm_forward, arm_gradient = loss_and_gradient(
        ArmWeights(base, None), sample, Arm.from_label("ff+fixed")
    )
    checks = {
        "membrane": (reference_forward.membrane, arm_forward.membrane),
        "spikes": (reference_forward.spikes, arm_forward.spikes),
        "rates": (reference_forward.rates, arm_forward.rates),
        "logits": (reference_forward.logits, arm_forward.logits),
        "grad_w_in": (reference_gradient.w_in, arm_gradient.base.w_in),
        "grad_w_out": (reference_gradient.w_out, arm_gradient.base.w_out),
        "grad_b_out": (reference_gradient.b_out, arm_gradient.base.b_out),
    }
    for name, (expected, observed) in checks.items():
        identical = np.array_equal(expected, observed)
        ok &= identical
        if verbose:
            print(f"  gate-F ff+fixed {name:11s} {'BIT-IDENTICAL' if identical else 'DIFFERS'}")
    if verbose:
        print(f"  gate-F loss identical: {reference_forward.loss == arm_forward.loss}")
    ok &= reference_forward.loss == arm_forward.loss

    # --- readout gradients: exact, validated by finite differences ---
    for label in ARMS:
        arm = Arm.from_label(label)
        weights = ArmWeights(base, w_rec.copy() if arm.recurrent else None)
        _, gradient = loss_and_gradient(weights, sample, arm)
        worst = 0.0
        for which, index in (("w_out", (3, 5)), ("w_out", (11, 2)), ("b_out", (4,))):
            analytic = {"w_out": gradient.base.w_out, "b_out": gradient.base.b_out}[which][index]
            numeric = _numeric_gradient(weights, sample, arm, which, index)
            denominator = max(abs(float(analytic)), abs(float(numeric)), 1e-6)
            worst = max(worst, abs(float(analytic) - numeric) / denominator)
        # float32 central differences at eps=1e-3 carry ~1e-1 relative error in
        # the worst case: the loss is stored to ~1e-7 relative, so the numerator
        # is only good to ~1e-4 and the quotient to ~1e-1. 5e-2 is therefore a
        # sanity bound, not a precision claim. The rigorous hidden-path check is
        # the naive-backward cross-check below, which runs at 1e-5.
        passed = worst < 5e-2
        ok &= passed
        if verbose:
            print(f"  finite-diff readout {label:9s} worst {worst:.2e}  "
                  f"{'PASS' if passed else 'FAIL'}")

    # --- hidden-path gradients: vectorised vs independent scalar-loop backward ---
    for label in ARMS:
        arm = Arm.from_label(label)
        weights = ArmWeights(base, w_rec.copy() if arm.recurrent else None)
        capture: dict = {}
        forward, gradient = loss_and_gradient(weights, sample, arm, capture)
        naive_w_in, naive_w_rec = _naive_backward(
            weights,
            sample,
            arm,
            forward,
            capture["thresholds"],
            capture["previous_spikes"],
            capture["direct_spike"],
        )

        def deviation(expected, observed):
            expected = np.asarray(expected, dtype=np.float64)
            observed = np.asarray(observed, dtype=np.float64)
            denominator = max(
                float(np.linalg.norm(expected)), float(np.linalg.norm(observed)), 1e-12
            )
            return float(np.linalg.norm(expected - observed) / denominator)

        worst = deviation(naive_w_in, gradient.base.w_in)
        if arm.recurrent:
            worst = max(worst, deviation(naive_w_rec, gradient.w_rec))
        passed = worst < 1e-5
        ok &= passed
        if verbose:
            print(f"  naive-backward xcheck {label:9s} rel dev {worst:.2e}  "
                  f"{'PASS' if passed else 'FAIL'}")

    # --- arms must actually differ from the baseline ---
    baseline, _ = loss_and_gradient(ArmWeights(base, None), sample, Arm.from_label("ff+fixed"))
    for label in ("ff+alif", "rec+fixed", "rec+alif"):
        arm = Arm.from_label(label)
        forward, _ = loss_and_gradient(
            ArmWeights(base, w_rec.copy() if arm.recurrent else None), sample, arm
        )
        differs = not np.array_equal(baseline.spikes, forward.spikes)
        ok &= differs
        if verbose:
            print(f"  arm {label:9s} changes spiking: {differs}")

    return bool(ok)


if __name__ == "__main__":
    import sys

    print("=== arms.py selftest ===")
    sys.exit(0 if selftest() else 1)
