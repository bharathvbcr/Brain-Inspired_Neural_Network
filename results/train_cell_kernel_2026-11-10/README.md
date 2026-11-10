# Train-cell kernel profile, 2026-11-10

Evidence for the bit-exact NEON attention kernel (`8c00289`). This directory is
a subdirectory on purpose: `scripts/build_results_index.py` indexes only
`results/*.md`, and this is a profile record, not a result. Every number below
is recomputed from the raw files here by `derive_tables.py`
(`python3 -I results/train_cell_kernel_2026-11-10/derive_tables.py`); none is
copied from a log's own summary.

## What was profiled

* **Cell.** A 2-epoch `ff+fixed+attn` train-cell: attention d32, 4 layers,
  h128, `published-2ms`, `adjacent-sum-5`, seed 5170001, 6 987 training
  samples (speakers 3 and 6 held out). The invocation is
  `profile_train_cell.sh`.
* **Binary.** `shd-instrument` built from `aa073b0` (the scalar attention
  kernel), release profile.
* **Tool.** macOS `/usr/bin/sample`, 1 ms interval, 40 s, started 3 s after the
  child. `head/sample.txt` is its raw output (path anonymised by the tool);
  `head/cell.json` is the cell it wrote. Final-weights SHA-256
  `9c364bd9441d1406cd97aa15078a83c1042b93180963f6bf159652528df512d3`.

## Where the samples land

The "Sort by top of stack" table in `head/sample.txt` has 55 rows summing to
**407 183** samples (all threads; rayon workers included).

| Frame | Samples |
|---|---:|
| `attention_gradient` | 193 155 |
| `attention_forward_presented` | 110 091 |
| `expf` (8 882) + `DYLD-STUB$$expf` (2 511) | 11 393 |
| `__psynch_cvwait` (idle) | 64 374 |
| `swtch_pri` (rayon spin yield) | 11 524 |

Shares, with the arithmetic. The choice of denominator moves the headline, so
all three are shown:

| Frame | / 407 183 (all) | / 342 809 (all - cvwait) | / 331 285 (all - cvwait - swtch_pri) |
|---|---:|---:|---:|
| `attention_gradient` | 47.44 % | 56.34 % | 58.30 % |
| `attention_forward_presented` | 27.04 % | 32.11 % | 33.23 % |
| gradient + forward | 74.47 % | 88.46 % | **91.54 %** |
| `expf` | 2.80 % | 3.32 % | 3.44 % |
| `__psynch_cvwait` | 15.81 % | 18.78 % | (excluded) |

e.g. gradient + forward = 193 155 + 110 091 = 303 246; 303 246 / 331 285 =
0.9154. The "about 91 % of non-idle CPU" in the commit message therefore holds
only when `swtch_pri`, rayon's spin-yield, is also counted as idle; with only
`__psynch_cvwait` removed it is 88.5 %. Both exceed 88 %, and the conclusion
(the two attention functions are the cost) does not depend on the choice.

## ABBA timing, `aa073b0` against `8c00289`

`abba3/`: ten interleaved runs of the same cell, one init, `/usr/bin/time -l`,
machine load average 76 to 110 throughout (`*.load`). Tag `old` is `aa073b0`;
`new3` is the binary carried by `8c00289`. The tag-to-commit mapping for `new3`
is taken from the run notes and the commit message's "1.85x"; the binaries were
not kept, so it cannot be re-derived from the files here. Every run's final
weights hash to the oracle value above (`weights.sha256`; the weights files
themselves are not kept).

| Binary | n | instructions retired (min) | cycles elapsed (min) | cycles (max) | user CPU (min) | user CPU (max) |
|---|---:|---:|---:|---:|---:|---:|
| `aa073b0` (`old`) | 4 | 7.9130e12 | 1.5530e12 | 1.5942e12 | 363.57 s | 371.01 s |
| `8c00289` (`new3`) | 4 | 4.6609e12 | 8.4067e11 | 8.6585e11 | 200.68 s | 208.31 s |
| ratio of minima | | 1.698 | **1.847** | | 1.812 | |

Cycles: 1.5530e12 / 8.4067e11 = 1.847. The ranges do not overlap: the slowest
`8c00289` run (8.6585e11) is below the fastest `aa073b0` run (1.5530e12). `abba3/` also holds two runs of `new2`, an earlier
candidate (cycles min 9.3592e11); it is not the committed kernel.

`abba4/` compares two variants of the committed kernel with no `old` arm, so it
is not a comparison against `aa073b0`: `new3` (cycles min 8.2877e11, user min
200.19 s, n = 4) against `new3c64` (8.3877e11, 199.10 s, n = 4), a variant with
a different training sub-chunk floor. The 1.2 % cycle gap is inside the
within-arm spread (`new3` alone spans 8.2877e11 to 8.5306e11). Its `new3`
minimum, 8.2877e11, is a second observation of the same binary and agrees with
`abba3` to 1.4 %.

Limits. Load averages of 76 to 110 mean wall time and user CPU carry
scheduler noise; cycles and instructions are the figures to read, and only
minima are compared. Four runs per arm is a sample, not a distribution. The
instruction ratio (7.9130e12 / 4.6609e12 = 1.70) is smaller than the cycle
ratio (1.85); the difference is a higher instructions-per-cycle on the new
kernel. Why was not measured here.

## Files

* `head/sample.txt`, `head/cell.json`: the profile of `aa073b0` (4.8 MB).
* `profile_train_cell.sh`: the profiling invocation.
* `abba3/`, `abba4/`: `N.tag.json` (cell), `.time` (`/usr/bin/time -l`),
  `.load` (`uptime` before the run), `weights.sha256`.
* `round3.log`: the verification log of the kernel branch at the time. It ends
  `pins exit 1`: the attention pin tests failed in that run. `round3b.log` is a
  later run of the checks, which ended `pins exit 0`. `round3.log` is retained
  as written and is not evidence that the pins pass.
* `round3b.log`: the later verification log (see above).
* `derive_tables.py`: recomputes the tables above.
