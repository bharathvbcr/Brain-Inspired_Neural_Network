"""The instrument's attention read-out (R1), ported to PyTorch for the SNN-delays reference.

Preregistration: `results/PREREG_2026-11-08_ATTENTION_ON_THE_DELAY_SUBSTRATE.md` §3.
Definition: `binn-learn/src/shd_attention.rs` (module docs, `AttentionParams::deterministic`,
`attention_forward_presented`), unchanged:

    z_0(t)   = W_e s(t) + pos(t)                        d_model = 32
    4 blocks: z(t) <- z(t) + W_o softmax_row(q k^T / sqrt(d)) v,   q,k,v = W_q z, W_k z, W_v z
    pooled   = mean_t z_L(t)
    a        = W_a pooled                                [n_classes]

- **Input.** `s` is the reference's tensor layout `[T_h, batch, hidden]`: the post-dropout spike
  train of the last hidden layer. In training its values are 0 and 1/(1-p) (asserted by
  `test_readout_patch`); in evaluation dropout is the identity. The read-out is linear in the
  spike value, as the Rust embedding is.
- **Position** is the fixed sinusoidal code over normalised time u = t/(T_h-1), with d_model/2
  geometrically spaced frequencies from ATTENTION_MIN_CYCLES to ATTENTION_MAX_CYCLES cycles. The
  phase is formed in float32 from the Rust forward's own frequency values (TAU_CYCLES_BITS_D32),
  because it reaches about 400 rad, where one float32 ulp of phase is 3e-5: a float64 phase, or a
  frequency recomputed with another `pow`, would differ from the Rust code by more than the parity
  bar.
- **Initialisation** is `AttentionParams::deterministic` reproduced bit for bit: the xorshift64*
  `PortableRng`, one stream per matrix, Glorot for W_e, W_q, W_k, W_v and W_a, and W_o = 0. It
  draws nothing from torch's global generator, so attaching the read-out cannot move the
  substrate's initialisation, dropout masks or data order.
- **Presentation.** `forward(s, presentation)` reads the hidden train through a per-sample order:
  stream position t is built from hidden step `presentation[b, t]`, and position t keeps its own
  positional code. That is the `hidden-shuffled` operator's attachment point (prereg §3); the
  permutations themselves come from the shuffle hook, not from here.

Matrices are stored as the Rust code stores them: W_e is `[hidden, d_model]` (input-major), every
other matrix is `[out, in]`.
"""

from __future__ import annotations

import numpy as np
import torch
from torch import nn

# binn-learn/src/shd_attention.rs
ATTENTION_MIN_CYCLES = np.float32(1.0)
ATTENTION_MAX_CYCLES = np.float32(64.0)
# The registered shape (prereg §3): d32/L4.
D_MODEL = 32
LAYERS = 4

_MASK64 = (1 << 64) - 1
# AttentionParams::deterministic's per-matrix stream keys.
_KEY_EMBED = 0x4154_544E_0000_0001
_KEY_Q = 0x4154_544E_0000_0002
_KEY_K = 0x4154_544E_0000_0003
_KEY_V = 0x4154_544E_0000_0004
_KEY_READOUT = 0x4154_544E_0000_0005


class PortableRng:
    """`binn_learn::shd_matched::PortableRng`: xorshift64* with float32 `uniform`."""

    def __init__(self, seed: int) -> None:
        if seed < 0:
            raise ValueError("PortableRng seed must be a u64")
        self.state = max(seed & _MASK64, 1)

    def next_u64(self) -> int:
        value = self.state
        value ^= value >> 12
        value ^= (value << 25) & _MASK64
        value ^= value >> 27
        self.state = value
        return (value * 0x2545_F491_4F6C_DD1D) & _MASK64

    def uniform(self, low: np.float32, high: np.float32) -> np.float32:
        # (x >> 40) < 2^24, so the quotient is exact in float32; each following
        # operation rounds to float32, as the Rust expression does.
        unit = np.float32(self.next_u64() >> 40) / np.float32(1 << 24)
        return np.float32(low + np.float32(np.float32(high - low) * unit))

    def shuffle(self, values: list[int]) -> None:
        for index in range(len(values) - 1, 0, -1):
            other = self.next_u64() % (index + 1)
            values[index], values[other] = values[other], values[index]


def glorot_limit(fan_in: int, fan_out: int) -> np.float32:
    return np.float32(np.sqrt(np.float32(6.0) / np.float32(fan_in + fan_out), dtype=np.float32))


def glorot(rng: PortableRng, count: int, fan_in: int, fan_out: int) -> np.ndarray:
    limit = glorot_limit(fan_in, fan_out)
    return np.array([rng.uniform(-limit, limit) for _ in range(count)], dtype=np.float32)


def deterministic_params(hidden: int, n_classes: int, seed: int,
                         d_model: int = D_MODEL, layers: int = LAYERS) -> dict[str, np.ndarray]:
    """`AttentionParams::deterministic(hidden, n_classes, AttentionConfig::new(d_model, layers), seed)`.

    Returns `w_e [hidden, d]`, `w_q/w_k/w_v/w_o [layers, d, d]` and `w_a [n_classes, d]`.
    """
    if d_model < 2 or d_model % 2:
        raise ValueError(f"attention d_model must be even and >= 2, got {d_model}")
    if layers < 1 or hidden < 1 or n_classes < 1:
        raise ValueError("attention params need layers, hidden and n_classes >= 1")
    d = d_model
    w_e = glorot(PortableRng(seed ^ _KEY_EMBED), hidden * d, hidden, d).reshape(hidden, d)
    blocks = {name: [] for name in ("w_q", "w_k", "w_v", "w_o")}
    for layer in range(layers):
        layer_key = (layer + 1) << 32
        for name, key in (("w_q", _KEY_Q), ("w_k", _KEY_K), ("w_v", _KEY_V)):
            blocks[name].append(glorot(PortableRng(seed ^ key ^ layer_key), d * d, d, d).reshape(d, d))
        blocks["w_o"].append(np.zeros((d, d), dtype=np.float32))
    w_a = glorot(PortableRng(seed ^ _KEY_READOUT), n_classes * d, d, n_classes).reshape(n_classes, d)
    params = {"w_e": w_e, "w_a": w_a}
    params.update({name: np.stack(mats) for name, mats in blocks.items()})
    return params


# `TAU * cycles` for the 16 sin/cos pairs at d_model = 32, as float32 bit patterns, exactly as
# `attention_forward_presented` computes them (`TAU * (MIN * (MAX/MIN).powf(pair/15))`).
#
# Why constants rather than the formula: float32 `pow` differs in the last bit between
# implementations. These are the values BINN e3017d0's `binn-learn` produced on the macOS aarch64
# build, 2026-11-09; a Linux build of the same Rust may differ in a last bit, which the parity bar
# tolerates (pooled moves by at most 4e-7 across ladders). They agree with neither numpy's float32
# `power` nor macOS libm `powf` at 7 of the 16 pairs. One float32 ulp at 64 cycles
# moves the phase at u = 1 by about 3e-5 rad, so recomputing the ladder would put a position
# difference of up to 4.6e-5 into every stream, beyond the 1e-5 parity bar, and would also make
# the code depend on the host's math library. Fixing the bits makes the read-out identical on the
# Mac and on the Linux run instances. `test_tau_cycles_are_the_registered_ladder` checks that every
# constant is within 2.5 float32 ulps of the exact 2*pi*64^(pair/15) (Rust's are at most 1.93 off),
# and the fixture checks them against Rust bit for bit.
TAU_CYCLES_BITS_D32 = np.array(
    [0x40C90FDB, 0x4104A6C3, 0x412F08DB, 0x4166F5A4, 0x41986060, 0x41C90FDB, 0x4204A6C3, 0x422F08DA,
     0x4266F5A6, 0x42986060, 0x42C90FDB, 0x4304A6C3, 0x432F08DC, 0x4366F5A1, 0x4398605E, 0x43C90FDB],
    dtype="<u4")


def tau_cycles(d_model: int = D_MODEL) -> np.ndarray:
    """`TAU * cycles` per sin/cos pair: the Rust forward's float32 values."""
    if d_model != 32:
        raise ValueError("the frequency ladder is fixed for the registered d_model = 32")
    return TAU_CYCLES_BITS_D32.view("<f4").copy()


def tau_cycles_formula(d_model: int = D_MODEL) -> np.ndarray:
    """The ladder recomputed with numpy's float32 `power`. Not bit-equal to `tau_cycles`."""
    pairs = d_model // 2
    ratio = np.float32(ATTENTION_MAX_CYCLES / ATTENTION_MIN_CYCLES)
    tau = np.float32(2.0 * np.pi)
    out = np.empty(pairs, dtype=np.float32)
    for pair in range(pairs):
        exponent = np.float32(pair) / np.float32(pairs - 1) if pairs > 1 else np.float32(0.0)
        cycles = np.float32(ATTENTION_MIN_CYCLES * np.power(ratio, exponent, dtype=np.float32))
        out[pair] = np.float32(tau * cycles)
    return out


def positional_code(t_steps: int, d_model: int = D_MODEL) -> np.ndarray:
    """`positional_code(t, t_steps, ..)` for every t: `[t_steps, d_model]` float32."""
    if t_steps < 1:
        raise ValueError("positional code needs at least one timestep")
    taus = tau_cycles(d_model)
    out = np.empty((t_steps, d_model), dtype=np.float32)
    for t in range(t_steps):
        u = np.float32(t) / np.float32(t_steps - 1) if t_steps > 1 else np.float32(0.0)
        phase = (taus * u).astype(np.float32)
        # sin/cos of the float32 phase evaluated in float64 and rounded once: this equals Rust's
        # f32 sin/cos bit for bit on the fixture. numpy's vectorised float32 kernels and macOS
        # libm sinf each differ from it by one ulp at some entries (measured 2026-11-09).
        out[t, 0::2] = [np.float32(np.sin(np.float64(p))) for p in phase]
        out[t, 1::2] = [np.float32(np.cos(np.float64(p))) for p in phase]
    return out


class AttentionReadout(nn.Module):
    """R1: `a = W_a mean_t z_L(t)`, batched over samples, looping only over the blocks."""

    def __init__(self, hidden: int, n_classes: int, seed: int,
                 d_model: int = D_MODEL, layers: int = LAYERS) -> None:
        super().__init__()
        params = deterministic_params(hidden, n_classes, seed, d_model, layers)
        self.hidden, self.n_classes, self.d_model, self.layers = hidden, n_classes, d_model, layers
        for name, value in params.items():
            setattr(self, name, nn.Parameter(torch.from_numpy(value.copy())))
        self.score_scale = float(np.float32(1.0) / np.sqrt(np.float32(d_model), dtype=np.float32))
        self._pos: dict[tuple[int, torch.device], torch.Tensor] = {}

    def load_numpy(self, params: dict[str, np.ndarray]) -> None:
        """Overwrite the weights in place (parity tests); shapes must match."""
        with torch.no_grad():
            for name, value in params.items():
                target = getattr(self, name)
                if tuple(target.shape) != value.shape:
                    raise ValueError(f"{name}: shape {value.shape}, expected {tuple(target.shape)}")
                target.copy_(torch.from_numpy(np.ascontiguousarray(value, dtype=np.float32)))

    def positions(self, t_steps: int, device: torch.device) -> torch.Tensor:
        key = (t_steps, device)
        if key not in self._pos:
            self._pos[key] = torch.from_numpy(positional_code(t_steps, self.d_model)).to(device)
        return self._pos[key]

    def pooled(self, s: torch.Tensor, presentation: torch.Tensor | None = None) -> torch.Tensor:
        """`mean_t z_L(t)`, `[batch, d_model]`, from `s` of shape `[T_h, batch, hidden]`."""
        if s.dim() != 3 or s.shape[2] != self.hidden:
            raise ValueError(f"read-out expects [T, batch, {self.hidden}], got {tuple(s.shape)}")
        t_steps = s.shape[0]
        stream = s.permute(1, 0, 2)  # [batch, T, hidden]
        if presentation is not None:
            if presentation.shape != stream.shape[:2]:
                raise ValueError(f"presentation must be [batch, T] = {tuple(stream.shape[:2])}")
            check = torch.sort(presentation, dim=1).values
            if not torch.equal(check, torch.arange(t_steps, device=check.device).expand_as(check)):
                raise ValueError("presentation is not a permutation of 0..T per sample")
            stream = torch.gather(stream, 1, presentation.unsqueeze(2).expand(-1, -1, self.hidden))
        z = stream @ self.w_e + self.positions(t_steps, s.device)
        for layer in range(self.layers):
            q = z @ self.w_q[layer].T
            k = z @ self.w_k[layer].T
            v = z @ self.w_v[layer].T
            weights = torch.softmax((q @ k.transpose(1, 2)) * self.score_scale, dim=2)
            z = z + (weights @ v) @ self.w_o[layer].T
        inverse_t = float(np.float32(1.0) / np.float32(t_steps))
        return (z * inverse_t).sum(dim=1)

    def forward(self, s: torch.Tensor, presentation: torch.Tensor | None = None) -> torch.Tensor:
        """`a = W_a pooled`, `[batch, n_classes]`."""
        return self.pooled(s, presentation) @ self.w_a.T
