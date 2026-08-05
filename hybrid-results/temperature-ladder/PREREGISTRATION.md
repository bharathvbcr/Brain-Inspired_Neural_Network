# Winner-temperature ladder preregistration

Protocol family: `binn-hybrid-winner-temp-v1-*`

## Locked priors

- Canonical H0 `binn-hybrid-h0-v3-caedeec1a47475a5` remains `HYBRID_NO_GO`.
- Fresh held-out seeds remain unused.
- H1-H3 remain stopped.
- This study is not post-hoc retuning of H0, the smooth diagnostic, or the
  production event diagnostic.

## Question

Where along a soft-to-hard winner-temperature ladder do direct gradients of the
matched soft residual teacher stop transferring into evaluation accuracy?

## Design

1. Train with the soft residual terminal teacher only. Direct-terminal updates
   do not depend on winner temperature.
2. Evaluate the same trained weights under:
   - `soft` — linear residual propagation (smooth-diagnostic endpoint);
   - finite temperatures `2.0`, `1.0`, `0.5`, `0.25`, `0.1` via
     `softmax(scores / T)`;
   - `hard` — one-hot argmax winner chain.
3. Controls: privileged intermediate targets (inadmissible ceiling) and
   shuffled labels (leakage) at maximum budget.
4. Primary metric: direct-terminal D* = max depth with lower-95 accuracy
   ≥ 0.65.
5. Collapse temperature: softest non-soft ladder point whose direct-terminal
   D* is strictly below the soft-endpoint D*.

## Full grid

- 20 development seeds from master `0x4842_5445_4d50_0001`
- depths 1 through 8
- budgets 480, 1,920, 7,680
- learning rates 0.015, 0.035, 0.070
- 1,000 frozen test examples per cell
- mechanism probes: 128 examples per seed/depth/temperature

## Non-claims

- Does not reopen H0.
- Does not authorize production-event scientific pass/fail.
- Does not spend held-out seeds.
