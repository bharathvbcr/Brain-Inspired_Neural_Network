# Same-specification Rust/NumPy transfer falsifier

**Protocol:** v146  
**Schedule:** QUICK / non-citable  
**Verdict:** **PILOT**  
**Micro-conformance:** yes  
**Byte-identical replay:** yes  

| Metric | Rust | NumPy |
|---|---:|---:|
| Matched accuracy | 0.2500 | 0.2500 |
| Live accuracy | 0.2500 | 0.2500 |
| Matched − live | 0.0000 (LCB 0.0000) | 0.0000 |

Controls: raw-rate 0.2500; time-shuffled 0.2500; shuffled-label 0.2000; BPTT 0.2500.  
Matched validity: **no**. Transfer phenomenon: **no**. Independent reproduction: **yes**.

Historical `0.51` is intentionally not used; it combined incompatible harnesses.
