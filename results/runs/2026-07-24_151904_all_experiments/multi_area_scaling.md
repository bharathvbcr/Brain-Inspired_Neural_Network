> **RETRACTED 2026-07-25** — see `results/HARD_AUDIT_v12_2026-07-25.md`.
>
> Area-0 input was the literal constant `(0..k)` for every sample and the feedback update used `vec![1.0; n]` dummies, so nothing about the stimulus entered the network. The 0.75 / 0.40 / 0.90 accuracies are the positive-class fractions of three *different* 20-sample datasets (seed `toy(100 + n_areas)`). GPU columns invalid as above.
>
> Fixes landed in the same commit; re-run before citing any number from this file.

# Multi-Area Structural Scaling Report

### Learning Accuracy Across Multi-Area Network Depths
| Area Count (M) | Total Cells | Train Accuracy | Test Accuracy | Status |
|----------------|-------------|----------------|---------------|--------|
| M =  2 |   64 cells | 0.6500 | 0.7500 | PASS |
| M =  4 |  128 cells | 0.7167 | 0.4000 | FAIL |
| M =  8 |  256 cells | 0.7333 | 0.9000 | PASS |

### Multi-Area Scaling Throughput Benchmark
| Area Count (M) | Total Cells | CPU SIMD (ms/step) | Metal GPU (ms/step) | Speedup |
|----------------|-------------|--------------------|---------------------|---------|
| M =  4 |  2000 cells | 0.199 ms | 0.196 ms | 1.02x |
| M =  8 |  4000 cells | 0.285 ms | 0.287 ms | 0.99x |
| M = 16 |  8000 cells | 0.551 ms | 0.529 ms | 1.04x |
