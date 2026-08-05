# Continual Learning Advantage Benchmark

**Experiment:** continual-learning  
**Tasks:** 3 · hidden=256 · epochs_per_task=40  

```
claim_axis: exploratory (not MUST / not Gate G3 reopen)
must_not_claim: Gate G2; Gate G3 remassage; biology; camera-ready continual SOTA
```

========================================================================
Continual Learning Advantage Benchmark (continual-learning)
Tasks=3, hidden=256, epochs_per_task=40
========================================================================

Task 1: Train Acc -> LearnedFB=0.9000 | BPTT=1.0000
   -> Retention on Task 1: LearnedFB=0.9000 | BPTT=1.0000
Task 2: Train Acc -> LearnedFB=1.0000 | BPTT=1.0000
   -> Retention on Task 1: LearnedFB=1.0000 | BPTT=1.0000
   -> Retention on Task 2: LearnedFB=1.0000 | BPTT=1.0000
Task 3: Train Acc -> LearnedFB=1.0000 | BPTT=0.7250
   -> Retention on Task 1: LearnedFB=1.0000 | BPTT=0.7250
   -> Retention on Task 2: LearnedFB=1.0000 | BPTT=0.7250
   -> Retention on Task 3: LearnedFB=1.0000 | BPTT=0.7250

Continual Learning Evaluation Complete.

**Reading:** exploratory retention smoke only. Formal continual gate remains C2 / Gate G3 FAIL in [`APPENDIX_POST_G2.md`](APPENDIX_POST_G2.md) — do not cite this binary as overturning G3.
