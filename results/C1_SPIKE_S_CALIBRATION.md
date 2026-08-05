# C1 calibrated natural-spiking (`c1-spike-s-*`) — one honest PC attempt

Authority: Rust sources + on-disk notes. Does **not** reopen `c1-118207fbc3eaba53` or reinterpret v6 `c1-09442acdbdc0c752`. G2 accuracy / gap / PC **thresholds unchanged**.

## Diagnosis (why v6 PC collapsed)

Under protocol v6 (`c1-spike-*`), hidden θ stays finite during integrate. Membrane-score k-WTA reads residual soma `v` at decision time. Cells that naturally spike reset to `V_RESET=0`, so the best-driven units score worst or drop out of the `v > 0` filter. θ=∞ mute (canonical / iso / project) prevented that reset. Positive control on the same local-assembly path therefore sat ~0.77–0.78 (< 0.90).

## Calibration knobs (protocol v9)

| Knob | Value | Role |
|---|---|---|
| Selection | **spike-count k-WTA** (tie-break residual `v`; subthreshold fallback to membrane) | Production-path fix; uses natural spikes as the selection signal |
| `init_w` | **0.22** (was 0.15) | Stronger feedforward for class-selective integrate spikes |
| `eta` | **0.45** (was 0.35) | Faster three-factor on PC / main |
| `tau_e` | **48** (was 40) | Slightly longer eligibility |
| Readout boost | **1.35 / init_w** clamped to [1, 14] (was 1.15 / init_w → [1, 12]) | Disclosed; forced winners still drive readout |
| PC task | **multi-frame** feature pulse (3 mid frames @ 1.0); PC train/test floors 96/32 | Disclosed easier schedule; **main coincidence task unchanged** |
| Learner mute | **none** (finite θ on scientific path) | Does not secretly restore θ=∞ |

Experiment prefix `c1-spike-s`; protocol version **9**. Frozen v6 hashes remain INVALID historical objects.

## Hashes

| Schedule | Hash |
|---|---|
| Scientific n=20 | `c1-c3e47b1e5f564df6` |
| Quick / PILOT | `c1-078cdbd91088c2f6` |

## Results (thresholds unchanged: PC ≥ 0.90, sparsity ∈ [0.005, 0.03], …)

### Quick (`c1-078cdbd91088c2f6`)

- Verdict: **PILOT**
- PC **1.0000**, sparsity **0.0125** (valid on short schedule)
- local **0.4625**, dense **0.5000**, gradient-ref **0.8000**, elig-ref **0.9375**
- Note: [`c1_spike_s_quick.md`](c1_spike_s_quick.md)

### Scientific n=20 (`c1-c3e47b1e5f564df6`)

- Verdict: **INVALID_HARNESS**
- PC **0.8413** (< 0.90); sparsity **0.0074** (in-band)
- local **0.4700**, dense **0.4250**, gradient-ref **0.9387**, elig-ref **1.0000**
- gap-closed mean **0.0927**, LCB **0.0511** (informational only)
- Note: [`c1_spike_s.md`](c1_spike_s.md)

## Stop rule

One honest calibration attempt completed. Scientific PC improved vs v6 (0.77 → 0.84) but did not clear `g2_min_positive_control`. **No further threshold massage.** Do not claim PASS/FAIL from either spike family.
