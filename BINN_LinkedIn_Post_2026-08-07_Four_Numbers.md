# LinkedIn Feed Post Draft (companion to the 2026-08-07 article)

Two weeks ago I wrote that I didn't know whether my architecture was better than an ANN, but I knew what evidence I would accept against it.

Here is the first instalment. It cost me four retracted claims, one prediction I registered in advance and lost, and my single best result.

That last one is the part worth your time. Three of my arms score exactly 1.0000 with zero variance, each beating the exact-gradient ceiling it is measured against. A local rule beating backpropagation at zero variance is not a discovery. It is a harness telling you it is measuring something else.

So I audited it, and found I had already caught it. Two weeks ago I found the bug, wrote the fix, made the new code refuse to emit a PASS, and bumped the version. Then I never re-ran the experiment. The old report is still on disk, still says PASS, and is cited in six documents including my paper's abstract.

Finding a defect and repairing a record are two different jobs. Only the first one feels like work.

If you have seen a learning rule beat exact gradients at zero variance, what turned out to be wrong with the harness?

#SpikingNeuralNetworks #NeuromorphicComputing #ComputationalNeuroscience #MachineLearningResearch #ReproducibleResearch
