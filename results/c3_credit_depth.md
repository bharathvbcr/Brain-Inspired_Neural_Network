# C3 / U15 — credit assignment vs compositional depth

**Kill-gate override:** this run is an **exploratory post-G2 branch**. Gate G2 FAIL under `c1-118207fbc3eaba53` still stands. C3 does **not** reopen the v8 kill-gate; it requires `--enable-c3` / `--override-g2-for c3`.

- config hash: `c3-445aa8de7761d4f4`
- protocol version: 1
- quick/PILOT: false
- seeds: 10
- depth sweep: 1..= 8
- states / operations: 4 / 2
- train / test per depth×seed: 5000 / 1000
- baseline: `C3_V1_ORACLE_TEACHER_FORCED_REFERENCE` (lr=0.2)
- D* accuracy floor: 0.650
- measured D* (local): **3**
- measured D* (gradient ref): **8**
- verdict: **MEASURED**

## Accuracy versus depth

| depth | local mean | local var | oracle mean | chance |
|---:|---:|---:|---:|---:|
| 1 | 1.0000 | 0.000000 | 1.0000 | 0.2500 |
| 2 | 1.0000 | 0.000000 | 1.0000 | 0.2500 |
| 3 | 0.9460 | 0.012964 | 1.0000 | 0.2500 |
| 4 | 0.2877 | 0.003058 | 1.0000 | 0.2500 |
| 5 | 0.2456 | 0.000285 | 1.0000 | 0.2500 |
| 6 | 0.2443 | 0.000239 | 1.0000 | 0.2500 |
| 7 | 0.2474 | 0.000269 | 1.0000 | 0.2500 |
| 8 | 0.2503 | 0.000362 | 1.0000 | 0.2500 |

## Protocol

Local path: each layer chooses a next state from locally stored transition synapses. The only teaching signal is terminal `+1/-1` reward; earlier layers receive exponentially decayed eligibility (three-factor style). No target transport across layers.

Oracle reference (`C3_V1_ORACLE_TEACHER_FORCED_REFERENCE`): disclosed teacher-forced updates with the true next-state at every layer. This is a tabular oracle control, not a gradient run on the production learner or event graph.

## Full scientific schedule

```bash
cargo run -p binn-lab --release --bin c3 -- --enable-c3 \
--out results/c3_credit_depth.md
```
