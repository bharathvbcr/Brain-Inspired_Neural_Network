# ANE fit audit — closed decision

**Date:** 2026-07-27  
**Verdict:** **Skip** — do not integrate [maderix/ANE](https://github.com/maderix/ANE) into BINN.

Full audit: Cursor plan `ane_vs_binn_fit_e8e4dbb3` (`~/.cursor/plans/ane_vs_binn_fit_e8e4dbb3.plan.md`).

## Why

1. **Production path mismatch.** BINN’s live substrate is event-driven sparse SpMV / lazy LIF / k-WTA / three-factor CSR updates. ANE’s stack is dense MIL (`conv` / `matmul` / fused SDPA) on private `_ANE*` APIs. There is no shared op surface.
2. **G2 is not FLOPs-bound.** Gate G2 FAIL (`c1-118207fbc3eaba53`) is algorithmic (local credit near chance). Accelerating dense kernels does not reopen G2.
3. **SHD SuperSpike is the only dense island** (~4.45 h CPU ceiling, GC1-exempt baseline). If that wall ever matters, prefer public **MLX / CoreML** (or host Metal dense), not private ANE.
4. **In-tree Apple path is Metal.** `binn-core`’s `MetalGpu` scaffolds CSR SpMV / LIF MSL; ANE cannot substitute that sparse plan. Finish Metal only if systems benches need real GPU numbers — orthogonal to ANE.

## Explicit non-actions

- No vendoring or cloning of maderix/ANE
- No bridge FFI (`ane_bridge_*`) from Rust
- No ANE kernels or third backend in `binn-engine` / `binn-learn` / `binn-core`
- No densify-to-ANE rewrite of the production Engine (would invalidate the sparse/local cost claim)

## Pointers

- Candidate repo: <https://github.com/maderix/ANE>
- Plan / evidence matrix: `ane_vs_binn_fit_e8e4dbb3.plan.md`
- Metal scaffold: `binn-core/src/metal_backend.rs` (`METAL_GPU_DISPATCH_IMPLEMENTED = false`)
