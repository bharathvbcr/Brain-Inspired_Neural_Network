# Preregistration — Azure d32/L4 scope, budget stability, and bottleneck diagnostic

**Registered:** 2026-08-20T19:20:33Z, before any Azure VM existed and before any
cell in `scripts/azure/plan_cells.py` ran. At registration the Azure resource
group contained only a non-billable Batch account and an empty LRS storage
account; the Batch account's six-core quota made it unsuitable for the run.

**Feasibility amendment:** 2026-08-20T19:34:30Z, still before any VM, gate, or
cell existed. A measured-cost audit found the initial 480-cell draft required
about 143 wall hours at the frozen four-thread throughput, while the dedicated
credit envelope buys at most 72 hours after reserving about $5 for disks and
Blob Storage. Launching it would guarantee an
incomplete experiment. The matrix below supersedes that unrun draft: it removes
e200 outside the primary replication and d128. No hypotheses are added after
seeing outcomes. The amended measured-cost estimate is 67.4 hours.

**Infrastructure amendment:** 2026-08-20T20:44:21Z, still before any VM, gate,
or cell existed. Azure rejected additional Fsv2 quota as `DeprecatedQuotaType`.
The approved East US regional quota is now 128 vCPUs and the existing Dalsv7
family quota is 350. The runner therefore uses eight `Standard_D16als_v7`
nodes instead of four `Standard_F16s_v2` nodes. The experimental matrix, seeds,
hypotheses, and analysis thresholds do not change. The same measured-throughput
model estimates 33.7 hours; this is an extrapolation until the first Azure cells
report wall time. The 37-hour cap costs at most $190.33 at the registered live
retail price of $0.643/node-hour, reserving at least $9 for disks and storage.

**256-core scheduling amendment:** 2026-08-20T21:13:39Z, still before any VM,
gate, result, failure record, or cell existed. The approved East US regional
quota is now 256 vCPUs, while the Dalsv7-family quota remains 350. The frozen
matrix is therefore sharded over four `Standard_D64als_v7` nodes (64 vCPU and
128 GiB each), with a core-token scheduler allowing at most 16 simultaneous
processes per node. Normal cells reserve four Rayon threads; only the registered
h1024/d64 tail cells reserve eight. Strict longest-processing-time dispatch
replaces anchor-first dispatch because simulation of the latter left an
avoidable paid tail; cell membership and every scientific parameter are
unchanged. A batch-scoped read-only weight layout removes a redundant matrix
transpose per sample. On an exact one-epoch h128/d32/L4 replay it reduced wall
time from 75.731654 s to 72.525924 s while every scientific output field was
bit-identical. Applying that measured 0.9577 runtime factor to the existing
estimator gives 17.83 h and 94.5% scheduled core utilization on every shard.
This remains a non-Azure estimate until cells report wall time. The 19-hour hard
cap costs at most $195.55 at the live $2.573/node-hour retail price, leaving
about $4.45 of the $200 credit for disks and Blob Storage. A failed cross-machine
Gate F now deallocates the whole scale set before any scientific cell starts.

**Outbound networking amendment:** 2026-08-21T06:00:59Z, still before any host
report, gate, failure record, or scientific cell existed. Azure's post-March
2026 private-subnet default prevented the workers from reaching Ubuntu and Rust
package repositories; the first corrected bootstrap stopped at `apt-get` and
the fleet was deallocated immediately. With explicit user approval, the worker
subnet now receives one Standard NAT Gateway and one Standard static IPv4
address for outbound and response traffic only. VM NICs still have no public IP,
the NSG still has no custom inbound rule, and unsolicited inbound connections
cannot traverse the NAT Gateway. The live fixed network ceiling is $0.95 for 19
hours plus $0.045/GB processed. No matrix member, seed, threshold, binary target,
or analysis rule changes.

**Pre-outcome portability amendment:** 2026-08-21T06:30:56Z, after explicit
user approval and before any scientific cell result, process failure record, or
experimental outcome existed. The first NAT-enabled launch completed four host attestations with one
binary (`sha256 666a73420a6378c3870da04612a178d9103164f579b9683c1d6e75660b347446`)
and then deallocated all four nodes when the first two workers reproduced the
known macOS-to-Linux Gate F `FAIL`. Zero of 252 scientific cells ran. This
fail-closed rule contradicted the pre-existing 2026-08-19 portability finding:
Rust `f32` transcendental results are not bit-identical across Apple libm and
glibc, while Linux cross-instance Gate F observations had already matched in
all 18 compared values. The cross-platform `FAIL` remains evidence and forbids
absolute macOS-to-Azure comparisons; it is not reclassified as a pass. Before
any scientific cell starts, all four Azure nodes must now attest the same frozen
source hash, binary hash, three Gate F cell identities, and bit-identical values
for every scientific field. Missing, stale, malformed, or differing reports
deallocate the fleet. The matrix, seeds, hypotheses, thresholds, validity rules,
and same-platform paired analysis are unchanged. This paragraph supersedes only
the final cross-machine-failure sentence of the 256-core scheduling amendment.

**Protocol:** `azure-d32l4-scope-v1`; exactly 252 cells, twelve fixed seeds
(5170001–5170012), one source archive and one binary hash across all nodes.
**Status:** confirmatory for the named contrasts below; exploratory for hardware
performance. Linux/x86 results are not absolute comparisons to macOS or the
earlier Linux/aarch64 campaign unless the cross-machine gate passes.

## Question

The d32/L4 attention arm reached 0.8320 at h128 and was stable from e200 to
e400. The earlier d32/L1 arm inverted by h1024 and was not seed-consistent on
`channels-700`. This campaign asks whether the stronger configuration fixes
those scope failures, and whether a fixed 32-dimensional stream is the h1024
bottleneck.

## Frozen matrix

- Arms: `ff+fixed` and `ff+fixed+attn` d32/L4.
- Width: h128, h256, h512, h1024 at e400.
- Geometry: `adjacent-sum-5` plus `channels-700` at h128.
- Contracts: `published-2ms`, `published-10ms`, `fixed-t100`, `fixed-t250`,
  `fixed-t500` at h128.
- Budgets: e200 and e400 for the primary h128 replication; e400 elsewhere.
- Bottleneck diagnostic: d64/L4 at h1024/e400, sharing the d32/L4 and rate-only
  controls already in the width matrix.
- Validity gates, unchanged: zero non-finite events; all 20 classes predicted;
  majority prediction below 0.30; silent fraction at most 0.95; saturated
  fraction at most 0.05.

Every comparison is paired by seed and measured on the same pinned binary.
Missing, timed-out, or invalid cells are reported as such and never silently
dropped. Twelve is the terminal seed count; no thirteenth seed is available to
rescue a marginal verdict.

## Registered hypotheses

| ID | Statement | Criterion |
|---|---|---|
| AZ8-1 | The prior d32/L4 result replicates on x86 | h128/e400 paired gain at least 0.05 and positive in at least 9/12 seeds |
| AZ8-2 | The architecture result generalizes across width | h1024/e400 paired gain at least 0.05 and positive in at least 9/12 seeds |
| AZ8-3 | It generalizes to the binding geometry | `channels-700`/e400 paired gain at least 0.05 and positive in at least 9/12 seeds |
| AZ8-4 | The primary reported e400 gain is budget-stable | at h128, absolute change in paired gain from e200 to e400 below 0.02 |
| AZ8-5 | It generalizes across timing contracts | every contract's e400 paired gain at least 0.05 and positive in at least 9/12 seeds |
| AZ8-6 | Fixed d32 is the h1024 bottleneck | h1024/e400 d64 gain exceeds d32 gain by at least 0.02 and is at least 0.05 |

AZ8-2, AZ8-3, and AZ8-5 are independent scope claims. One passing does not
rescue another. AZ8-6 is interpreted only as a one-step width-of-attention
diagnostic; it cannot establish optimality because dimensions above 64 and
depths above four are not in the amended plan.

## Compute and stopping rule

Four dedicated `Standard_D64als_v7` nodes, with a core-token cap of 64 vCPUs and
at most 16 concurrent cells per node. Normal cells receive four Rayon threads;
the twelve registered h1024/d64 cells receive eight. The binary is built once with the portable
`x86-64-v4` baseline, hash-pinned, and reused on every node. EPYC 9005 provides
a full-width AVX-512 path. The scheduler packs independent cells until every
core is reserved without oversubscription; eight threads are a tail-latency
exception because within-cell scaling is sublinear. A hard watchdog deallocates
the scale set no later than 19 hours after
boot. Each cell has a 24-hour wall limit and one recorded outcome: result,
validity failure, process failure, or timeout.

The first complete 12-seed matrix is the result. Thresholds, budgets, dimensions,
and seeds do not move after outcomes are visible. Additional Azure credit may be
spent only on a separately registered successor, not by extending this matrix
until it passes.
