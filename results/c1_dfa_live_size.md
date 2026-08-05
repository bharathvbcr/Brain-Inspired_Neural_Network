# H2 dfa-live size protocol — pointer

Full prereg + results: [`runs/2026-07-24-dfa-live-size/`](runs/2026-07-24-dfa-live-size/)

| Arm | Hash | Verdict |
|---|---|---|
| pm1 | `c1-mac-probe-e14b8f80a366d9c0` | reference (mean 0.4969) |
| structured-fb | `c1-mac-probe-a180f42193693700` | **Reject-floor** |
| dfa-live | `c1-mac-probe-e13053acc4e40c18` | **Accept** (mean 0.7781, gap LCB 0.1010) |

N=2000 syn-matched, n_seeds=8, floor 0.60, gap LCB vs pm1 > 0.

**Not G2 reopen. Not frozen v20 remassage. Not overnight quick H2.**
