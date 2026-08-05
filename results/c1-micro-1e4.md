# Micro isolate N=1e4 stress note

Companion to [`c1_mac_probe.md`](../c1_mac_probe.md).

| Field | Value |
|---|---|
| Protocol | `c1-micro-*` isolate `local-assembly` |
| N | 10 000 |
| fan | 256 |
| k | 100 (`k/N=0.01`) |
| Quick MacProbe / runner | `c1-micro-6369e6a32702efba` / `c1-micro-713322a01c532f01` |
| Scientific MacProbe / runner | `c1-micro-bf6e78039b04665d` / `c1-micro-2e51a51ef7702ce6` |
| Scientific measured nnz | 2 570 837 |
| Scientific peak RSS | 186 892 288 bytes (~178 MiB) |
| Scientific wall | 1.425 s |
| Activity sparsity | 0.010 (in band) |

**Verdict framing:** capacity stress after G2 FAIL only. 1e4 RSS is comfortable on this host; optional 1e5 quick stress reached ~1.86 GB (see main note).
