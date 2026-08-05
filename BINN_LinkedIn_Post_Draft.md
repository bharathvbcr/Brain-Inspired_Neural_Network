# LinkedIn Feed Post Draft

I started with a belief that sounded obvious:

If the brain learns through sparse activity, local plasticity, and continual adaptation, a brain-inspired network should have important advantages over today's ANNs.

Then I asked the question that changed my project:

If that is true, why haven't brain-inspired neural networks already taken over?

My first framing focused on spiking cells, dendrites, event-driven computation, and local plasticity. But those ingredients are established research directions; biological plausibility alone does not make their combination efficient or effective.

The real bet is narrower: can sparse assemblies make local credit assignment useful on difficult tasks?

A broader gap analysis exposed throughput, encoding, data, and ecosystem barriers. An adversarial audit then produced three important corrections: spike resets keep forward time sequential; learned encoders conflict with a no-autodiff crux test; and Assembly Calculus does not prove the deep-learning claim I need.

My confidence went down. The experiment became better.

ANNs dominate because their algorithm, hardware, data, pretrained models, and tooling reinforce one another.

I am not claiming to have built a better ANN. No BINN implementation or benchmark evidence exists yet. Here, BINN is project shorthand, not a label for every spiking approach.

The falsifiable hypothesis combines:

1. Compartmental, stateful cells.
2. Sparse timed events and assemblies.
3. Local three-factor plasticity.

A purpose-built runtime will test whether avoiding dense work for silence outweighs queueing and irregular-memory overhead. Every biological mechanism must survive an ablation.

The kill gate is simple: if sparse assemblies do not move local learning toward a well-tuned BPTT reference, I stop the central program and report the negative.

Results will use matched-task accuracy-work curves with sparsity, event counts, wall-clock cost, memory, and uncertainty disclosed.

Given the history of local learning, I think this gate is more likely to fail than pass. The value is a clean answer—not confidence theater.

To researchers in computational neuroscience, SNNs, continual learning, learning theory, and neuromorphic systems:

What is the strongest reason this will fail? What is the cheapest experiment that would expose it? Which missing baseline would make the result credible?

Please share papers, counterexamples, negative results, or benchmark recommendations. If any part of this experiment still protects the idea instead of testing it, tell me where.

[Insert LinkedIn article or repository link]

#BrainInspiredAI #ComputationalNeuroscience #NeuromorphicComputing #ContinualLearning #MachineLearningResearch
