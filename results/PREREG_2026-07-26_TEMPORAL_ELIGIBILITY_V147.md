# Protocol v147 — temporal eligibility mechanism diagnostic

Registered 2026-07-26 before execution. This is a code-integrity artifact, not
manuscript text.

## Reason

Protocol v144 was `INVALID_TASK`: true shared-forward BPTT reached
`0.9433–1.0000`, while matched random feedback remained at
`0.2467–0.2733`. Audit found that the matched-feedback update used only an
instantaneous surrogate derivative. It omitted temporal eligibility through
membrane leak, soft reset, and cumulative rate.

## Frozen mechanism

For each unit, differentiate its final cumulative rate with respect to local
weights and bias while holding that layer's input trajectory fixed:

```text
q_t = alpha m_(t-1) + w x_t + b
s_t = sigmoid(beta (q_t - threshold))
m_t = q_t - threshold s_t
r_t = r_(t-1) + s_t / T
```

The class error is transported by the immutable random-feedback matrix. No
between-layer derivative, RMS normalization, optimizer substitution, or
difficulty tuning is allowed.

## Mechanical gate

Before the diagnostic:

- local eligibility must match central finite differences with a fixed input
  trajectory;
- the RFB arm must overfit 40 easiest-task training examples above `0.95`
  after 200 epochs at `lr=0.005`, hidden width 64;
- it must predict all four classes with majority below `0.95`;
- hidden/readout gradient and applied-step RMS must be finite and positive;
- a complete replay must be byte-identical.

Failure blocks calibration.

## Frozen diagnostic

- difficulty: `(jitter radius=0, distractor events=4)` only;
- train/test: `200/100`;
- epochs: `20`;
- hidden width: `64`;
- treatment optimizer: deterministic SGD, `lr=0.005`, global clip `5.0`;
- ceiling: frozen deterministic Adam;
- seeds: three fresh v147 seeds derived from `0x7E4A514700000001`;
- report training/test accuracy, predicted-class count, majority fraction,
  hidden/readout gradient RMS, hidden/readout step RMS, and replay.

The treatment learns only when mean held-out accuracy is at least `0.55`, BPTT
is at least `0.90`, every health/replay gate passes, and the overfit gate
passes. RFB at or below `0.30` with BPTT at or above `0.90` stops the current
learned-feedback design. Any intermediate miss also stops for mechanism
reassessment; no optimizer or difficulty sweep follows.

If the treatment learns, register a fresh v148 calibration and new v149/v150
depth/transfer protocols. Protocols v145/v146 are never reused.
