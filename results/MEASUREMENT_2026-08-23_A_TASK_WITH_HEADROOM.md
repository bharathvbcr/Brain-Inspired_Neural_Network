# Measurement — finding a task with headroom for the depth suite

**Why:** `RESULT_2026-08-22_DEEP_PATH_AND_TRANSPORT_SCALE.md` §6 records that
v136's depth result is weak evidence because the ceiling saturates at exactly
1.0000 on `CoincidenceTask`. `RESULT_2026-08-23_TRACK_B_REREAD.md` §2 records the
same disease on the matched-arch schedule. A depth suite needs a task where the
reference has somewhere to fall.

**This is a feasibility measurement, not a result.** No hypothesis is under test.

---

## 1. SHD is the obvious answer and it is blocked

`shd-depth-scaling` is built, unit-tested and **refused at the authorization
gate** — verified, exit 2. It requests `CampaignKind::LocalLearning`, refused
while `SHD_INSTRUMENT_STATE` is `Uncalibrated`.

It would run if it requested `HarnessValidation` instead. **That is gaming the
gate, and it is not done.** `binn-lab/tests/campaign_gate_refuse.rs` pins all nine
gated binaries against exactly that reclassification. SHD stays blocked until
calibration, which is the reference re-run currently in flight
(`PREREG_2026-08-22_REFERENCE_RERUN.md`).

So the question became: is there an input-rich task that is *not* SHD?

## 2. `CreditDepthTask`, built for this and never wired

`binn-data/src/credit_depth.rs` — *"a sequence of hidden-state transforms of
tunable length (`depth`) maps a start state to a terminal target. Local learners
see only the terminal reward."* Written for U15 / C3. **Zero callers**, the third
fully-built unwired module found this session.

**It is order-sensitive**, which had to be checked before anything else: a task
where only the *counts* of operations matter gives depth nothing to do. With
`n_states = 4`, op 0 is `s+1` and op 1 is `3s+1`, so `op0∘op1 = 3s+2` while
`op1∘op0 = 3s`. Non-commutative. Compositional depth creates real sequential
credit assignment.

## 3. The headroom sweep

`shared_bptt` at a fixed network depth of 2 × 64, 40 epochs, 400 train / 200 test.
Start state presented at every timestep, one operation per timestep.

| n_states | task depth | chance | **ceiling** | treatment | ceiling − chance |
|---:|---:|---:|---:|---:|---:|
| 4 | 2 | 0.2500 | **1.0000** | 1.0000 | saturated |
| 4 | 4 | 0.2500 | **0.6200** | 0.4950 | 0.3700 |
| 4 | 8 | 0.2500 | **0.4850** | 0.4900 | 0.2350 |
| 8 | 2 | 0.1250 | **1.0000** | 1.0000 | saturated |
| 8 | 4 | 0.1250 | **0.4600** | 0.4250 | 0.3350 |
| 8 | 8 | 0.1250 | **0.2750** | 0.2750 | 0.1500 |
| 16 | 2 | 0.0625 | **1.0000** | 1.0000 | saturated |
| 16 | 4 | 0.0625 | **0.3750** | 0.3600 | 0.3125 |
| 16 | 8 | 0.0625 | **0.1900** | 0.1800 | 0.1275 |

**There is headroom, and it is controllable.** Three readings:

1. **Task depth 2 saturates at every state count.** The ceiling is exactly 1.0000
   in all three — the same disease as `CoincidenceTask`. Any depth study must run
   at task depth ≥ 4.
2. **The ceiling degrades with task depth**, 0.6200 → 0.4850 at `n_states = 4`
   and 0.4600 → 0.2750 at 8. Exact reverse-mode gradients do not solve deep
   composition here, which is precisely the difficulty the task was built to
   create.
3. **Two cells need watching.** At `n_states = 4, depth = 8` the treatment
   (0.4900) *exceeds* the ceiling (0.4850) — an inversion, of the kind
   `CeilingHealth` exists to refuse. And at `n_states = 8, depth = 8` ceiling and
   treatment are identical to four decimals (0.2750), which is the shape of two
   arms agreeing because both are degenerate. Neither is diagnosed here; both are
   reasons the suite must wire `CeilingHealth` and a readout audit rather than
   report a table.

## 4. What this does not establish

- **It varied task depth, not network depth.** The network was fixed at 2 × 64
  throughout. A depth *suite* varies the network and holds the task fixed. This
  measurement only establishes that a fixed operating point with headroom exists.
- **Single seed, single width, 40 epochs.** Nothing here is a result and no
  verdict follows from it. It is sized to answer one question: is there a task
  where the reference has room to fall? Yes.
- **The two suspicious cells are unexplained.** They are flagged, not diagnosed.
