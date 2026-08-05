# Diagnostic protocol provenance

- v1 was the first pilot. Its privileged control averaged per-step corrections
  by depth and was therefore an underpowered ceiling.
- v2 corrected the privileged ceiling and produced a valid three-seed pilot and
  eight-seed diagnostic.
- v3 expanded controls across all learning rates and ran the authoritative
  development study with 20 seeds, four budgets, five learning rates, and
  paired effects.

All versions remain development-only. The authoritative current diagnostic is
`binn-hybrid-diagnostic-v3-fe72201b01e57cfe`.

## Production event-engine family

- production v1 was a quick pilot using layer-specific transition weights. It
  was rejected because the smooth diagnostic shares one transition operator
  across depth.
- production v2 shared weights but used a softmax transition relaxation. It was
  rejected because this changes the gradient geometry relative to the original
  residual diagnostic.
- production v3 shares one transition graph, delivers the identity residual as
  an event, and uses the original finite-difference-checked residual teacher.
  It is the only production version run on the full 20-seed grid.

The authoritative production diagnostic is
`binn-hybrid-production-diagnostic-v3-f72033fbf6906b99`. The two earlier pilot
artifacts remain under `production-diagnostics/` and must not be pooled with
v3.
