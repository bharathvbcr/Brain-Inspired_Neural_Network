wave 28 — the rate at which dropout bites
  wave-28 cells : 84
  reused        : 96  (w27drp p30, w26pos intact; same binary)
  cells voided  : 0

=== H28-1  the rate at which the substrate actually loses accuracy ===
  bar: paired rate-arm drop from intact > 0.03 in >= 9 of 12
  p30  drop 0.0124  above bar 0/12  [w27drp]
  p50  drop 0.0219  above bar 1/12  [w28drp]
  p60  drop 0.0231  above bar 2/12  [w28drp]
  p70  drop 0.0388  above bar 11/12  [w28drp]  SENSITIVE
  p80  drop 0.0572  above bar 12/12  [w28drp]  SENSITIVE
  p90  drop 0.1080  above bar 12/12  [w28drp]  SENSITIVE

H28-1  dropout becomes a manipulation the substrate can feel: MET
  the instrument is sensitive from p70 upward. A null measured at or above that rate means the read-out does not care, not that the measurement could not see.
  smallest sensitive rate: p70

=== H28-2  does destroying counts cost the read-out its advantage? ===
  Dropout deletes spikes; it does not move them. Order survives it.
  p30  DiD 0.0015  |DiD|>0.03 in 1/12  (NOT sensitive - no reading taken)
  p70  DiD -0.0132  |DiD|>0.03 in 2/12  (sensitive)
  p90  DiD -0.0382  |DiD|>0.03 in 9/12  (sensitive)

H28-2  the read-out's advantage is specific to order: NOT MET
  destroying counts DOES cost the read-out its advantage. What the read-out consumes is not order alone, and the paper's mechanism claim is broader than it currently states.

======================================================================
  H28-1: MET
  H28-2: NOT MET
