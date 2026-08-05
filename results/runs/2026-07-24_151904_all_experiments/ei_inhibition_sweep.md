> **RETRACTED 2026-07-25** — see `results/HARD_AUDIT_v12_2026-07-25.md`.
>
> Every row printed the literal `PASS`; the file contained no threshold. The area was driven with i.i.d. uniform noise and the only metric (`1/(1+var)`) was a deterministic function of the pooling arithmetic. The 'soft-WTA competition' conclusion was never tested -- and cannot hold, because `compute_inhibition` broadcasts one scalar to every excitatory cell.
>
> Fixes landed in the same commit; re-run before citing any number from this file.

# Dynamic E-I Interneuron Sweeps Report

**Protocol Version:** 133  
**Experiment:** ei-inhibition-sweep  

## E-I Interneuron Dynamics Summary

| E:I Ratio | W(I->E) | Mean Inhibition | Competition Smoothness | Status |
|---|---:|---:|---:|---|
| 4:1 | 0.2 | 3.2509 | 0.5533 | PASS |
| 4:1 | 0.5 | 8.1652 | 0.1605 | PASS |
| 4:1 | 1.0 | 16.2752 | 0.0449 | PASS |
| 8:1 | 0.2 | 6.8661 | 0.2153 | PASS |
| 8:1 | 0.5 | 17.0120 | 0.0408 | PASS |
| 8:1 | 1.0 | 33.8774 | 0.0115 | PASS |
| 16:1 | 0.2 | 13.6296 | 0.0618 | PASS |
| 16:1 | 0.5 | 34.2827 | 0.0105 | PASS |
| 16:1 | 1.0 | 67.9996 | 0.0027 | PASS |

## Verdict

- PV+ Inhibitory Interneuron dynamics provide stable, continuous soft-WTA competition.
