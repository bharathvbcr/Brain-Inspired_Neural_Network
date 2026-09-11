# Understanding BINN experiments from scratch

Study notes for Bharath · 2026-09-10 · through Wave 29

**Learn by exploring:** open the [visual study lab](study-lab.html) for a neuron you can stimulate, a recording you can rearrange, and the real Wave 29 results you can inspect by training seed. The first two lessons use teaching examples; the third uses all 48 saved runs. An [explorable experiment map](diagrams/experiment-path.html) connects the input, spiking layer, and both readout branches. Both companions work offline.

**The current experiment asks whether a classifier benefits more from the timing of neural activity than from having lots of spikes available.** To understand it, imagine looking at a recording of a population of active cells. You can measure each cell's average activity, or examine how the activity unfolds over time. Those measurements can tell you different things.

The biological comparisons below are learning aids. These experiments run in software; they are not recordings from a living brain, and their results do not establish how brains learn.

Read [the foundations](#1-two-questions-under-one-project-name) first, then [the experiment walkthrough](#5-follow-one-example-through-the-experiment). For the latest result, jump to [Wave 29](#8-wave-29-deleting-most-spikes). Finish with [the self-check](#11-check-your-understanding).

Evidence labels: **verified** means checked against the current implementation or saved experiment records; **inferred** means an interpretation; **unverified** means these experiments have not established it. Checking saved results is different from repeating the training.

## 1. Two questions under one project name

BINN began with a learning question: **can a network learn useful behaviour when each connection changes using mostly local information?** Later work asks an information question: **what can a classifier extract from a spiking network if it can inspect the activity over time?**

A **network** is a connected collection of simulated units. A **model** is the program that uses that network to make predictions. **Training** changes its connection strengths using examples; an **experiment** compares models or conditions to answer a specific question.

| Research line | Biological comparison | What is being tested? |
|---|---|---|
| Original BINN and its local-learning controls | Connections adapt using nearby activity and a feedback signal | Whether a particular learning rule can assign useful credit |
| SHD attention campaign, including Wave 29 | An observer examines a population's activity recording | What extra information an attention readout can use |

**Verified:** the SHD attention models are trained using backpropagation through time. They are a separate experimental instrument from the event-driven, locally learned assembly system. Good attention results therefore do not establish that the original goal of learning without backpropagation succeeded. [Implementation of the learning path](../binn-learn/src/shd_matched_arms.rs#L881).

## 2. What the artificial neuron is doing

Imagine a simplified excitable cell receiving little pushes from incoming signals. Several pushes close together may bring it to a firing threshold. If they arrive too far apart, their effects partly fade before they can add up.

That is the intuition behind a **leaky integrate-and-fire neuron**, or **LIF**:

- **Integrate:** incoming activity contributes to an internal membrane-like state.
- **Leak:** some of that state fades between inputs.
- **Fire:** crossing a threshold produces a spike, followed by a reset.

In this model, a spike is a brief event saying that a unit fired. A stronger input need not produce a bigger spike; it can change whether and when spikes occur. This is a deliberately simplified model of excitability, not a full account of membrane biophysics.

Two kinds of memory are easy to confuse. **Membrane state** carries a short-lived effect of recent input within an example. **Learned weights** retain what training changed across examples. Think of the difference between a cell's current activation state and a longer-lasting change in how strongly it responds to input.

| Term | Biological comparison | Meaning here |
|---|---|---|
| Weight | Strength of a connection's influence | A learned number controlling an input's contribution |
| Hidden neuron | An intermediate processing cell | A simulated unit between input and final decision |
| Feed-forward, `ff` | Signals pass onward without recurrent connections among the hidden units | The anchor configuration; its neurons still have membrane state |
| Recurrent, `rec` | Activity feeds back through connections within a population | An additional route for previous activity to influence later activity |
| Fixed threshold, `fixed` | Firing criterion stays fixed | The threshold does not adapt; **the weights still learn** |
| Adaptive threshold, `alif` | Recent activity changes excitability | A model variant with an activity-dependent threshold |

**Verified:** the anchor's forward loop implements membrane retention, weighted input, thresholding and reset; recurrent and adaptive behaviour are separate switches. [Forward implementation](../binn-learn/src/shd_matched_arms.rs#L762).

## 3. What is the network trying to recognise?

**SHD** means **Spiking Heidelberg Digits**. It contains spoken-digit examples represented as spikes in auditory channels. The repository treats it as a 20-class task: the digit labels distinguish English and German utterances. The model receives spike events, not written words. [Dataset definitions](../binn-data/src/shd.rs#L19).

Picture a population recording with time along the horizontal axis and auditory channels stacked vertically. Each mark says that a channel was active at a particular time. A whole recording belongs to one spoken-digit class.

At the Wave 29 anchor:

- The original **700 channels** are grouped in adjacent sets of five, giving **140 input channels**. Counts are added, not converted into a simple yes/no signal.
- Events are collected into **2 ms time bins**. Each example retains its variable duration under this contract.
- A hidden layer contains **128 simulated neurons**.
- Each run uses **8,156 training examples** and **2,264 test examples**.

Think of binning as selecting the time resolution of a measurement, and channel grouping as combining neighbouring recording channels. Both change the representation presented to the model. [Binning and grouping implementation](../binn-data/src/shd_contract.rs#L61), [Wave 29 plan](../results/shd_attention_wave29_local/plan_w29.json).

## 4. Two ways to read the same kind of activity

A **readout** converts hidden-neuron activity into a class decision. It is the part that answers, “Which spoken digit was this?”

**The rate readout:** average each hidden neuron's spikes over the example, then use those averages to score the classes. The biological comparison is an average activity measurement for each cell over a recording. It retains how active each cell was, but discards the order of the hidden spikes when doing that averaging.

**The attention branch:** keep the hidden activity as a sequence, attach position information, and learn which moments should interact when forming a class score. The comparison is reviewing a time-lapse recording and relating activity near the beginning to activity near the end.

Attention here is a computational operation. It does not mean awareness or conscious attention.

```text
Input spikes → simulated LIF population → hidden spike recording
                                              │
                         ┌────────────────────┴───────────────────┐
                         ↓                                        ↓
                Average activity                        Sequence + positions
                         ↓                                        ↓
                  Rate class scores                     Attention class scores
                         └────────────────────┬───────────────────┘
                                              ↓
                                  Add scores → digit decision
```

The rate-only model uses the left branch. The attention model uses both.

Three details matter:

1. **Attention is added alongside the rate readout.** It does not replace it. Both branches contribute to the final scores.
2. **Attention sees the completed sequence.** Any timestep can interact with any other timestep. There is no restriction to the past, so this is not evidence for a streaming recogniser that must answer before an utterance finishes.
3. **The two models learn separately.** Adding attention changes the feedback reaching the hidden layer during training. The architecture of that layer is matched, but its final learned weights and activity need not be identical.

**Verified:** these details follow from the [readout integration](../binn-learn/src/shd_matched_arms.rs#L815), [all-timestep attention loop](../binn-learn/src/shd_attention.rs#L785), and [gradient passed back into the hidden layer](../binn-learn/src/shd_matched_arms.rs#L946).

Also, the rate model is not completely blind to input timing. Timing can change which hidden neurons fire before their spikes are averaged. It is the final averaging operation that discards their temporal order.

## 5. Follow one example through the experiment

Imagine one recording labelled as the spoken digit “three.”

1. **Prepare the input.** Load its auditory spike events, group channels and bin time.
2. **Apply the assigned condition.** Keep it intact, shuffle its time bins, or delete individual input spikes, depending on the experiment.
3. **Produce hidden activity.** Inputs drive the simulated neurons; their membrane states evolve and some neurons spike.
4. **Make a prediction.** The readout turns the hidden activity into scores for the twenty classes.
5. **During training, compare with the label.** The training system scores how much probability the model gives the correct answer and produces an error signal. The **optimiser**, the procedure choosing weight adjustments, uses that signal so similar future examples may be classified better.
6. **Repeat across the training set.** One pass through that set is an **epoch**. Wave 29 uses 400 epochs.
7. **After training, evaluate.** The final weights classify the test examples without learning from their labels during that evaluation.

Backpropagation through time is easiest to picture as a bookkeeping procedure: the training system traces an error backward through the model's sequence of calculations to estimate which weight changes could help. A **surrogate gradient** gives it a usable training signal around the sharp spike threshold. This procedure is not a claim that biological synapses perform that calculation.

**Verified:** transformations happen before the epoch loop, and test accuracy is evaluated after training. [Preparation](../binn-lab/experiments/shd_instrument.rs#L754), [final evaluation](../binn-lab/experiments/shd_instrument.rs#L1098).

## 6. Why manipulate the recording?

An accuracy improvement tells us that adding attention helped. It does not tell us what information made that possible.

The experiments use an approach similar to controlled perturbations in biology: change one aspect of what the system can use, check that the manipulation worked, and compare the affected groups.

| Condition | Recording comparison | What actually changes? |
|---|---|---|
| `intact` | Original recording | No temporal manipulation |
| `bin-shuffled` | Rearrange whole frames of the recording | Time-bin order is disrupted; per-channel counts and within-bin co-occurrence are preserved |
| `channel-shuffled` | Rearrange time separately for each channel | Order and cross-channel synchrony are disrupted; each channel's total count is preserved |
| `reversed` | Play the whole recording backward | Direction changes, but the transformation can be undone without recovering missing information |
| `hidden-shuffled` | Rearrange the hidden recording only when handing it to attention | Input and the preceding spiking forward pass are untouched at a fixed set of weights |
| `spike-dropout-p90` | Randomly erase individual marks | Each input spike has a 90% deletion probability; surviving spikes stay in their original bins |

**Verified:** these are the [implemented operators](../binn-learn/src/shd_temporal.rs#L665).

The biological analogy has a boundary: spike dropout does not kill 90% of neurons. It removes input events. Nor does it perfectly preserve synchrony: erasing one member of a coincident pair removes that coincidence, even though neither surviving event is moved.

These input manipulations are applied to **both training and test data**. Each model learns under its assigned condition. This asks what can be learned from the remaining information; it does not test sudden damage to an already trained intact model.

## 7. How the earlier waves lead to Wave 29

### Wave 9: does the advantage depend on temporal order?

**Verified from archived cells:** at the h128, d32/L4 anchor on the Linux/glibc fleet, the twelve-seed comparison was:

| Input | Rate-only accuracy | Rate + attention accuracy | Attention advantage |
|---|---:|---:|---:|
| Intact | 70.62% | 83.20% | 12.58 percentage points |
| Time bins shuffled | 69.34% | 69.83% | 0.50 percentage points |

The time-lapse comparison is useful here: when frames lose their order, the extra observer has much less advantage over the average-activity measurement. The models can still classify many examples, so shuffling has not erased every useful cue.

**Inferred:** much of the attention advantage at this configuration depends on temporal structure disrupted by shuffling. This does not identify an individual learned circuit or prove that every benefit of attention has one cause. [Wave 9 result and rounding correction](../results/RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md).

### Wave 27: what scale of timing, and is deletion strong enough?

Instead of shuffling the whole recording, shuffle within progressively larger windows. Small windows disturb local detail while preserving more of the broader sequence. Larger windows disrupt more of that sequence.

**Verified by rerunning the saved-cell analyser:** the measured loss of advantage crosses half its full-shuffle value at an interpolated window of about **261 ms**, at this anchor on the glibc fleet. This is an operational scale of the perturbation experiment. It is not a measured biological memory duration or the simulated neuron's membrane constant.

Deleting 30% of input spikes, however, reduced rate-only accuracy by only about **1.24 percentage points**, below the registered three-point sensitivity threshold. Think of an intervention too weak to shift the control assay enough: a small difference between groups would be hard to interpret.

[Wave 27 result](../results/RESULT_2026-09-05_W27_THE_TIMESCALE_IS_261_MS.md).

### Wave 28: the effect went in an unexpected direction

Wave 28 increased deletion. At 90%, the attention advantage grew. Its registered rule had treated a sufficiently large change in either direction as a failure, although the intended question concerned losing the advantage.

That **NOT MET** result remains in the record. Wave 29 specified separate directional questions and tested them on new training seeds. This is like writing a new protocol after an unexpected assay result, then testing new replicates rather than relabelling the old outcome. [Wave 29 preregistration](../results/PREREG_2026-09-07_W29_THE_ASYMMETRY_HAS_ITS_OWN_BAR.md).

## 8. Wave 29: deleting most spikes

**Question:** when most input spikes are removed, does attention lose its advantage, retain it, or gain a larger advantage?

The design has four conditions, each run at twelve training seeds:

| | Intact input | About 90% of input spikes deleted |
|---|---|---|
| Rate-only model | 12 runs | 12 runs |
| Rate + attention model | 12 runs | 12 runs |

That makes **48 experiment cells**. Here, “cell” means one configured training run, not a biological cell. Every seed has all four conditions, allowing matched comparisons.

**Verified:** all 48 saved runs are mechanically complete and pass the analysis validity checks; zero are voided. They were run on **Apple Silicon/macOS (`aarch64-apple-darwin`)**. The following numbers are all from that platform and seed block.

| Input condition | Rate-only accuracy | Rate + attention accuracy | Attention advantage |
|---|---:|---:|---:|
| Intact | **70.68%** | **82.61%** | **11.93 percentage points** |
| About 90% deleted | **59.56%** | **76.15%** | **16.60 percentage points** |
| Accuracy lost to deletion | **11.12 points** | **6.46 points** | Advantage grows by **4.67 points** |

Values were recomputed from the stored accuracies before rounding. Subtracting already rounded entries can differ in the last displayed digit. [All 48 records and saved verdict](../results/shd_attention_wave29_local/), [experiment report](../results/RESULT_2026-09-09_W29_THE_ASYMMETRY_HAS_ITS_OWN_BAR.md).

**Both models get worse.** Attention retains an advantage because it loses less accuracy. It does not improve from 82.61% to 76.15%.

The biological comparison is two assays applied to an increasingly incomplete recording. Both become less informative, but the assay that can use temporal relationships retains more of its predictive usefulness.

### Read “difference-in-differences” without the algebra

The papers abbreviate this as **DiD**. It asks: **how much more accuracy did the attention model lose than the rate model?**

Here, attention loses 6.46 points and rate loses 11.12 points. Attention loses about 4.67 points less, so the reported DiD is **negative**, about **−0.0467** in accuracy units.

- **Positive DiD:** the attention advantage shrinks.
- **Near-zero DiD:** the advantage changes little.
- **Negative DiD:** the advantage grows.

A value of 0.03 means **three percentage points**, not a 3% relative change. A negative sign here is a direction, not a failed experiment.

### What the registered verdicts mean

| Check | Plain-language meaning | Verified outcome |
|---|---|---|
| Sensitivity | Does deletion hurt the rate model enough to interpret the comparison? | More than three points lost in **12/12** pairs |
| H29-1 | Is the loss of attention advantage below the registered three-point bound? | **MET**, 12/12 comparisons satisfy the bound |
| H29-2 | Does the advantage grow by more than three points in at least nine comparisons? | **MET**, **11/12** satisfy the bound |
| H29-3 | Does the advantage increase further as deletion increases? | **No verdict**; no p70 condition was run in this block |

The advantage grows in all twelve comparisons; one does not clear the three-point threshold. The thresholds are the protocol's decision rules, not biological constants or a probability that the explanation is true. [Frozen analyser](../scripts/aws/analyse_wave29.py).

### What was replicated, and what remains unknown

**Verified:** the deletion pattern for each example is fixed before training. The same temporal seed is used across these training runs. The twelve seeds therefore test variation in training, **not twelve independently sampled datasets or twelve deletion-mask sets**. They are closer to repeated preparations of a model under one assay condition than twelve biological donors. [Plan](../results/shd_attention_wave29_local/plan_w29.json), [preparation code](../binn-lab/experiments/shd_instrument.rs#L770).

**Inferred:** useful information remains accessible to the attention model even after heavy random input deletion. Together with the order manipulations, this supports studying temporal structure as a source of its advantage.

**Unverified:** why the advantage grows; whether it survives structured channel loss or burst deletion; whether it generalises to recurrent networks or much wider networks; whether a model trained on intact data would cope with sudden deletion at test time.

Spike counts still matter: accuracy fell. These data establish resilience of the **relative advantage** under this deletion experiment, not irrelevance of counts, energy efficiency, or equivalence to biological hearing. Nor do the macOS numbers establish exact agreement with the earlier glibc runs.

## 9. Where the original local-learning question fits

Return to a connection between two cells. A useful biological comparison for an **eligibility trace** is a temporary tag: “this connection was recently involved.” A later feedback signal can then influence how a tagged connection changes.

**Three-factor learning** combines information about presynaptic activity, postsynaptic activity, and a feedback or modulatory signal. The difficult part is **credit assignment**: if the whole network got the answer wrong, which of its many connections should change, and in which direction?

The original BINN design also uses sparse assemblies and **k-winners-take-all**, a competition rule limiting which units participate. Think of a population in which only a small selected subset is active. These assembly mechanisms are not the dense SHD attention instrument described above.

**Verified in the repaired matched-control records:** the particular **broadcast ±1 three-factor** rule remained around chance on a two-class task: 50% feed-forward and 51% recurrent, while its gradient reference reached 100%, across twenty seeds. A broadcast graded-error contrast reached 99.75% in its feed-forward suite. Thus the negative is specific to the tested rule and protocol; “all broadcast feedback fails” is too broad. [Feed-forward record](../results/matched_rerun_2026-08-25/c1_match_feedforward.md), [recurrent record](../results/matched_rerun_2026-08-25/c1_match_recurrent.md), [graded contrast](../results/matched_rerun_2026-08-25/c1_matched-dfa_feedforward.md).

That two-class task has a 50% chance baseline; SHD has twenty classes and a 5% uniform-guess baseline. Do not compare their percentages as if they were the same assay.

Some earlier local-learning results were withdrawn after discovering an input scale that left the matched forward pass without spikes. That is analogous to interpreting a perturbation in a preparation that was not responding in the first place. The repaired results, and the limits of the now-easy task, are explained in the [rerun report](../results/RESULT_2026-08-25_MATCHED_ARCH_RERUN.md). Failure of one tested rule does not establish impossibility of biological local learning.

## 10. Reading the filenames and status labels

| Label | Translation |
|---|---|
| `h128` | 128 hidden spiking units |
| `d32/L4` or `d32l4` | Attention represents each timestep with 32 features and uses four attention blocks; it does not mean four spiking layers |
| `e400` | 400 passes through the training set |
| `published-2ms` | The event-binning contract using 2 ms bins |
| `adjacent-sum-5` | Add counts from each group of five neighbouring input channels |
| `ff+fixed+attn` | Feed-forward, fixed-threshold spiking model with the additional attention branch |
| `p90` | 90% deletion probability for each input spike |
| Seed | A setting controlling pseudorandom choices; check whether it is a training seed or a manipulation seed |
| Wave | A planned batch of related experiments |
| Anchor | The reference configuration around which a comparison is organised |
| Preregistration | The questions, thresholds and rules written before the relevant result data |
| `MET` / `NOT MET` | The registered prediction did / did not satisfy its rule |
| `NOT EVALUABLE` | A required condition was missing or invalid; the question could not be scored |
| Voided | Excluded under the instrument's validity rules, with the exclusion disclosed |

One particularly confusing label is **`CELL_FAIL`**. It does not automatically mean a process crashed or a record is invalid. The instrument's per-cell scientific flag includes an 80% accuracy requirement; the wave-level hypotheses use their own comparisons.

**Verified:** Wave 29 has 12 `CELL_PASS` records and 36 `CELL_FAIL` records, all mechanically complete and valid for its analysis. A low-scoring rate control can still be essential evidence for a successful comparison. [Status calculation](../binn-lab/experiments/shd_instrument.rs#L1112), [validity rules](../scripts/cell_validity.py).

## 11. Check your understanding

Try answering before reading the sentence after each prompt.

1. **Why can average activity miss a useful signal?** Two hidden recordings can have identical per-neuron averages but different temporal arrangements.
2. **Does `fixed` mean the model is untrained?** No. It describes the firing threshold; weights still change during training.
3. **Why compare attention and rate under both input conditions?** Otherwise an accuracy loss could simply reflect the task becoming harder for both models.
4. **Did Wave 29 improve accuracy by deleting spikes?** No. Both accuracies fell; the attention model lost less.
5. **Why alter training data as well as test data?** To ask what can be learned from the remaining information, rather than test a surprise distribution change.
6. **Why is a failed sensitivity check different from a negative result?** The intervention may not have changed the control enough to answer the question.
7. **Does a spiking network necessarily learn biologically?** No. This attention campaign uses gradient training.
8. **What remains the central unanswered question after Wave 29?** Why this attention model retains more of its advantage under heavy deletion.

## 12. Continue into the experiment record

Read the [Wave 29 report](../results/RESULT_2026-09-09_W29_THE_ASYMMETRY_HAS_ITS_OWN_BAR.md) with its [preregistration](../results/PREREG_2026-09-07_W29_THE_ASYMMETRY_HAS_ITS_OWN_BAR.md) beside it. Then use [Wave 27](../results/RESULT_2026-09-05_W27_THE_TIMESCALE_IS_261_MS.md) for the timescale and assay-sensitivity story, and [Wave 9](../results/RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md) for the original order comparison. The [record index](../results/INDEX.md) leads to the broader history and identifies retired documents.

**Verification scope for these notes:** selected current code paths and relevant records were read; all 48 Wave 29 cells were checked and their group means recomputed; the Wave 27 and Wave 29 analysers and Wave 29 platform diagnostic were rerun; the existing published-number check passed 125/125 assertions, including the Wave 9 figures. Local-learning figures were checked in saved per-seed reports. Training, biological validation and a whole-repository scientific audit were not rerun for this tutorial.
