# C3 / U15 — credit assignment vs compositional depth

**Kill-gate override:** this run is an **exploratory post-G2 branch**. Gate G2 FAIL under `c1-118207fbc3eaba53` still stands. C3 does **not** reopen the v8 kill-gate; it requires `--enable-c3` / `--override-g2-for c3`.

- config hash: `c3-adf27f8ffc4185ca`
- protocol version: 1
- quick/PILOT: true
- seeds: 3
- depth sweep: 1..= 4
- states / operations: 4 / 2
- train / test per depth×seed: 600 / 200
- baseline: `C3_V1_ORACLE_TEACHER_FORCED_REFERENCE` (lr=0.2)
- D* accuracy floor: 0.650
- measured D* (local): **2**
- measured D* (gradient ref): **4**
- verdict: **PILOT**

> PILOT only: the quick schedule validates the harness and cannot support a scientific depth claim.

## Accuracy versus depth

| depth | local mean | local var | oracle mean | chance |
|---:|---:|---:|---:|---:|
| 1 | 1.0000 | 0.000000 | 1.0000 | 0.2500 |
| 2 | 1.0000 | 0.000000 | 1.0000 | 0.2500 |
| 3 | 0.5317 | 0.040758 | 1.0000 | 0.2500 |
| 4 | 0.3383 | 0.000233 | 1.0000 | 0.2500 |

## Protocol

Local path: each layer chooses a next state from locally stored transition synapses. The only teaching signal is terminal `+1/-1` reward; earlier layers receive exponentially decayed eligibility (three-factor style). No target transport across layers.

Oracle reference (`C3_V1_ORACLE_TEACHER_FORCED_REFERENCE`): disclosed teacher-forced updates with the true next-state at every layer. This is a tabular oracle control, not a gradient run on the production learner or event graph.

## Full scientific schedule

```bash
cargo run -p binn-lab --release --bin c3 -- --enable-c3 \
--out results/c3_credit_depth.md
```
