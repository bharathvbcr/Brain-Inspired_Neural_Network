> **RETRACTED 2026-07-25** — see `results/HARD_AUDIT_v12_2026-07-25.md`.
>
> The accuracy in this report is a constant predictor. The sample was never injected into the engine and `predicted = !winners.is_empty()` is constant by construction, so 0.8500 is the positive-class fraction of a 20-sample split. The training loop applied no weight updates. The GPU benchmark columns are also invalid: `SpmvBackend::spmv` never read `use_gpu`, so both arms ran identical rayon CPU code.
>
> Fixes landed in the same commit; re-run before citing any number from this file.

# Enhanced Live Spiking Engine Report
- **Enhanced Live Test Accuracy**: 0.8500
- **Execution Time**: 0.005s

### Benchmark Results (CPU SIMD vs Metal GPU Backend)
| Network Size (N) | CPU SIMD Time | Metal GPU Time | Speedup / Efficiency |
|------------------|----------------|----------------|----------------------|
| N =  1000 | 0.126 ms | 0.134 ms | 0.94x |
| N =  5000 | 0.340 ms | 0.339 ms | 1.00x |
| N = 10000 | 0.712 ms | 0.697 ms | 1.02x |
