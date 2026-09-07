wave 27 — the instruments that were built and never run
  wave-27 cells   : 264
  reused wave-26  : 240  (same binary, same anchor)
  cells voided    : 0

=== H27-1  hidden shuffle: a rate arm must pay exactly nothing ===
  seed pairs compared : 12
  byte-identical      : 12

H27-1  the rate read-out cannot see the hidden time axis: MET
  every scientific field is identical across every seed. The operator separates read-out access from input structure, and the wave is valid.
  DiD(hidden-shuffled) = 0.1159, positive 12/12  (registered as a QUESTION, not barred)

=== H27-2  spike dropout p30 ===
  rate-arm accuracy drop from intact = 0.0124, above 0.03 in 0/12

H27-2  the manipulation actually costs the substrate something: NOT MET
  dropout does not cost the rate arm accuracy above the bar. This is a finding about the INSTRUMENT: no null measured under it is interpretable, and section 2's ambiguity is not resolved.
  DiD(spike-dropout-p30) = 0.0015, positive 7/12  (reported, not barred)

=== H27-3  the timescale ladder, extended past its crossing ===
  w2    DiD 0.0066  positive 7/12
  w4    DiD -0.0028  positive 7/12
  w8    DiD 0.007  positive 8/12
  w16   DiD 0.0206  positive 10/12
  w32   DiD 0.0161  positive 9/12
  w64   DiD 0.0245  positive 12/12
  w128  DiD 0.0596  positive 12/12
  w192  DiD 0.0814  positive 12/12  (new)
  w256  DiD 0.126  positive 12/12  (new)
  full  DiD 0.1208  positive 12/12

  tau-half = 130.33 bins (interpolated)
           = 260.7 ms at the anchor's 2 ms bins

=== H27-4  QK-norm: the same instrument, or a different read-out? ===
  accuracy(qk-norm) - accuracy(default), intact = -0.0692  (n=12, band +-0.03)
  DiD(bin-shuffled) under qk-norm = 0.0632 (n=12), under default = 0.1208
  difference = -0.0577

H27-4  qk-norm is the same instrument with the score term bounded: NOT MET
  at least one half is outside the band, so qk-norm is a DIFFERENT read-out. Nothing measured with it transfers to the paper's headline. Note this is a read-out check only: wave 26 withdrew its collapse-specific motivation, and a MET here would not resurrect it.

=== H27-5  the tau_m ladder, under cell_validity's standing gate ===
  void rule: saturated_fraction > 0.05 voids a cell; more than 3 of 12 voids the rung
  tau   2.5 ms  n=12  mean saturated=0.0  over-gate 0  acc=0.7041  DiD=0.0076 (9/12)
  tau   5.0 ms  n=12  mean saturated=0.0  over-gate 0  acc=0.7026  DiD=0.0073 (7/12)
  tau 10.05 ms  n=12  mean saturated=0.0  over-gate 0  acc=0.7062  DiD=0.0 (0/12)
  tau  20.0 ms  n=12  mean saturated=0.0  over-gate 0  acc=0.7149  DiD=0.0156 (8/12)
  tau  40.0 ms  n=12  mean saturated=0.0  over-gate 0  acc=0.7238  DiD=0.0275 (11/12)

  Registered as a threshold, never as a prediction about which rung trips it. If the long rungs void wholesale the ladder is reported as bounded by the standing gate and is NOT re-run under a loosened one.

  cross-wave bit-identity, tau 10.05 vs wave 26's intact arms:
    rate : 12/12 byte-identical
    attn : 12/12 byte-identical

=== H27-6  speaker-held-out validation (model selection only) ===
  Registered non-use: no headline number, no DiD, no bar. These cells train on 6,987 samples and are not comparable to the corpus.
  rate: n=12  test acc 0.6786  val acc 0.7035  n_val [1169]
  attn: n=12  test acc 0.7926  val acc 0.6661  n_val [1169]

======================================================================
  H27-1 exact zero: MET
  H27-2 dropout bites: NOT MET
  H27-4 qk-norm is the same instrument: NOT MET
  H27-3, H27-5 and H27-6 are registered as questions or as thresholds and carry no MET/NOT MET.
