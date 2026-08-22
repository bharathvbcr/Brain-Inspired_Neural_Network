# Source versus pinned binary — equivalence verdicts

Instance `i-0ae4798ff32a1c8a7`, Linux 6.18.41-94.142.amzn2023.aarch64 aarch64, 32 vCPU, 4 threads per cell.
- pinned  `22d97c51ab020470…`
- today   `c88a07f840677b07…`
- ldd (GNU libc) 2.34
- rustc 1.98.0 (88d9e12ae 2026-08-18)

## E-1 (primary) — today's build vs the pinned binary, same host

| cell | verdict |
|---|---|
| `w7flr__ff-fixed-attn__h128__e10__published-2ms__adja` | identical (fields on one side only: clip_sample_grad_norm, clipped_samples, seed) |
| `w7flr__ff-fixed-attn__h128__e5__published-2ms__adjac` | identical (fields on one side only: clip_sample_grad_norm, clipped_samples, seed) |
| `w7flr__ff-fixed__h128__e5__published-2ms__adjacent-s` | identical (fields on one side only: clip_sample_grad_norm, clipped_samples, seed) |
| `w8con__ff-fixed-attn__h128__e400__published-10ms__ad` | identical (fields on one side only: clip_sample_grad_norm, clipped_samples, seed) |

## E-2 — the pinned binary vs the archive (tests the environment)

| cell | verdict |
|---|---|
| `w7flr__ff-fixed-attn__h128__e10__published-2ms__adja` | identical |
| `w7flr__ff-fixed-attn__h128__e5__published-2ms__adjac` | identical |
| `w7flr__ff-fixed__h128__e5__published-2ms__adjacent-s` | identical |
| `w8con__ff-fixed-attn__h128__e400__published-10ms__ad` | identical |

## E-3 — today's build vs the archive (follows from E-1 and E-2)

| cell | verdict |
|---|---|
| `w7flr__ff-fixed-attn__h128__e10__published-2ms__adja` | identical |
| `w7flr__ff-fixed-attn__h128__e5__published-2ms__adjac` | identical |
| `w7flr__ff-fixed__h128__e5__published-2ms__adjacent-s` | identical |
| `w8con__ff-fixed-attn__h128__e400__published-10ms__ad` | identical |

## E-4 — thread-count invariance, end to end

- `w7flr__ff-fixed-attn__h128__e5__published-2ms__adjac`: identical at 1 and 4 threads

## Verdict

**Every comparison is identical.** Today's source is behaviourally the campaign binary on the paths tested, on aarch64/glibc.
