# Azure d32/L4 scope verdicts

Coverage: **89 valid / 252 planned**, 6 voided, 0 failures, 157 missing.
Binary/gate provenance: **READY** (4/4 node attestations, 1 binary hash(es), 5 quorum record(s) in agreement).

| Hypothesis | Measurement | Verdict |
|---|---|---|
| AZ8-1 x86 replication | incomplete; >=+0.05 and >=9/12 | **INCOMPLETE** |
| AZ8-2 width scope | gain -0.1618; positive 1/12; >=+0.05 and >=9/12 | **NOT SUPPORTED** |
| AZ8-3 geometry scope | incomplete; >=+0.05 and >=9/12 | **INCOMPLETE** |
| AZ8-4 budget stability | incomplete; <0.02 | **INCOMPLETE** |
| AZ8-5 timing scope | published-2ms=?; published-10ms=?; fixed-t100=?; fixed-t250=?; fixed-t500=? | **INCOMPLETE** |
| AZ8-6 d32 bottleneck | incomplete | **INCOMPLETE** |

Gate F licenses absolute comparison with prior machines only when it passes; all registered verdicts above are same-binary, same-machine paired contrasts.

## Coverage, arm by arm

Reporting only — no verdict below depends on this table. `planned` counts the frozen matrix; `ran` counts cells on disk; `valid` applies the preregistered validity gate.

| wave | arm | ran / planned | valid | mean accuracy |
|---|---|---:|---:|---:|
| `az8con` | `ff+fixed` h128 e400 `fixed-t100` `adjacent-sum-5` rate | 0 / 12 | 0 | — |
| `az8con` | `ff+fixed` h128 e400 `fixed-t250` `adjacent-sum-5` rate | 0 / 12 | 0 | — |
| `az8con` | `ff+fixed` h128 e400 `fixed-t500` `adjacent-sum-5` rate | 0 / 12 | 0 | — |
| `az8con` | `ff+fixed` h128 e400 `published-10ms` `adjacent-sum-5` rate | 0 / 12 | 0 | — |
| `az8con` | `ff+fixed+attn` h128 e400 `fixed-t100` `adjacent-sum-5` d32l4 | 0 / 12 | 0 | — |
| `az8con` | `ff+fixed+attn` h128 e400 `fixed-t250` `adjacent-sum-5` d32l4 | 12 / 12 | 12 | 0.859430 |
| `az8con` | `ff+fixed+attn` h128 e400 `fixed-t500` `adjacent-sum-5` d32l4 | 9 / 12 | 9 | 0.852866 |
| `az8con` | `ff+fixed+attn` h128 e400 `published-10ms` `adjacent-sum-5` d32l4 | 0 / 12 | 0 | — |
| `az8dim` | `ff+fixed+attn` h1024 e400 `published-2ms` `adjacent-sum-5` d64l4 | 12 / 12 | 6 | 0.194199 |
| `az8geo` | `ff+fixed` h128 e400 `published-2ms` `channels-700` rate | 0 / 12 | 0 | — |
| `az8geo` | `ff+fixed+attn` h128 e400 `published-2ms` `channels-700` d32l4 | 0 / 12 | 0 | — |
| `az8wid` | `ff+fixed` h128 e200 `published-2ms` `adjacent-sum-5` rate | 0 / 12 | 0 | — |
| `az8wid` | `ff+fixed` h128 e400 `published-2ms` `adjacent-sum-5` rate | 0 / 12 | 0 | — |
| `az8wid` | `ff+fixed` h256 e400 `published-2ms` `adjacent-sum-5` rate | 4 / 12 | 4 | 0.722725 |
| `az8wid` | `ff+fixed` h512 e400 `published-2ms` `adjacent-sum-5` rate | 10 / 12 | 10 | 0.735203 |
| `az8wid` | `ff+fixed` h1024 e400 `published-2ms` `adjacent-sum-5` rate | 12 / 12 | 12 | 0.738590 |
| `az8wid` | `ff+fixed+attn` h128 e200 `published-2ms` `adjacent-sum-5` d32l4 | 8 / 12 | 8 | 0.831493 |
| `az8wid` | `ff+fixed+attn` h128 e400 `published-2ms` `adjacent-sum-5` d32l4 | 0 / 12 | 0 | — |
| `az8wid` | `ff+fixed+attn` h256 e400 `published-2ms` `adjacent-sum-5` d32l4 | 4 / 12 | 4 | 0.818573 |
| `az8wid` | `ff+fixed+attn` h512 e400 `published-2ms` `adjacent-sum-5` d32l4 | 12 / 12 | 12 | 0.823322 |
| `az8wid` | `ff+fixed+attn` h1024 e400 `published-2ms` `adjacent-sum-5` d32l4 | 12 / 12 | 12 | 0.576782 |

**5 arm(s) partially run**, holding 35 cells. A partially run arm carries data and no registered verdict; it must be reported as neither absent nor complete.
