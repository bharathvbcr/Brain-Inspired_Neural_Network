# Multi-Area Structural Scaling Report

> **RETRACTED COLUMN — the "Metal GPU" numbers below are CPU numbers.**
>
> This report predates the backend-honesty fix in `binn-core/src/metal_backend.rs`.
> At the time it was generated, the backend selector was a `use_gpu: bool` that
> `SpmvBackend::spmv` never read, so both columns executed byte-identical rayon
> code. The 0.97x / 0.62x / 0.99x "speedups" are measurement noise between two
> runs of the same CPU kernel, not a CPU-vs-GPU comparison.
>
> No GPU code has ever executed in this repository:
> `METAL_GPU_DISPATCH_IMPLEMENTED == false`, and `Backend::MetalGpu` is now
> unconstructible. See `results/HARD_AUDIT_v12_2026-07-25.md` §2.3 and
> `results/MOVING_FORWARD_2026-07-25.md` ("GPU throughput — withdrawn").
>
> The CPU SIMD column is retained; it is a valid single-backend scaling series.
> Do not cite the right-hand two columns anywhere.

### Multi-Area Scaling Throughput Benchmark
| Area Count (M) | Total Cells | CPU SIMD (ms/step) | ~~Metal GPU (ms/step)~~ | ~~Speedup~~ |
|----------------|-------------|--------------------|---------------------|---------|
| M =  4 |  2000 cells | 0.186 ms | ~~0.192 ms~~ (CPU) | ~~0.97x~~ |
| M =  8 |  4000 cells | 0.430 ms | ~~0.695 ms~~ (CPU) | ~~0.62x~~ |
| M = 16 |  8000 cells | 0.551 ms | ~~0.557 ms~~ (CPU) | ~~0.99x~~ |
