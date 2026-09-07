wave 26 — the saturation, the structural null, and the timescale
  cells indexed : 264
  cells voided  : 0
  probed cells  : 24

=== H26-1  the h1024 collapse and the read-out's entropy ===
  d32l4 e1    entropy 0.7125  (n=12)
  d32l4 e25   entropy 0.2950  (n=12)
  d32l4 e50   entropy 0.2751  (n=12)
  d32l4 e100  entropy 0.2138  (n=12)
  d32l4 e200  entropy 0.0139  (n=12)
  d32l4 e400  entropy 0.0021  (n=12)
  d32l2 e1    entropy 0.6875  (n=12)
  d32l2 e25   entropy 0.2255  (n=12)
  d32l2 e50   entropy 0.2077  (n=12)
  d32l2 e100  entropy 0.1703  (n=12)
  d32l2 e200  entropy 0.0793  (n=12)
  d32l2 e400  entropy 0.0801  (n=12)
  d32l4 |q||k| e1 46.374 -> e400 8587447.038  (185179.20x, n=12)
  d32l2 |q||k| e1 39.793 -> e400 913.911  (22.97x, n=12)

  entropy drop e100->e400: d32l4 0.2118, d32l2 0.0902

H26-1a  the read-out saturates where the fit is lost, and not in the control: NOT MET
  the entropy drop is absent or is not specific to the collapsing depth. The saturation account is not supported by its own primary clause.

H26-1b  the named term is what grows: NOT MET
  |q||k| grew 185179.20281785828x at d32l4 and 22.966636259379847x at d32l2, against bars of >=10.0x and <3.0x. If H26-1a is MET and this is NOT, the read-out saturates for a reason this wave did not identify, and it is reported as a NEW open problem rather than as confirmation.

  H26-1c (secondary, reported not required): entropy at e400/d32l4 = 0.0020597071458333333, threshold 0.3 -> below

=== H26-2  removing the read-out's access to order ===
  gain default intact          0.1258  positive 12/12
  gain no-position intact      0.0506  positive 12/12
  gain default bin-shuffled    0.005  positive 8/12
  gain(default) - gain(no-position) = 0.0752, above 0.03 in 12/12

H26-2a  position is what the read-out's advantage runs through: MET
  removing the positional code costs more than the campaign's bar. The arm keeps every parameter and loses the advantage.

  H26-2b (registered as a QUESTION, not a prediction)
    removing the read-out's access to order : gain 0.0506
    removing order from the data           : gain 0.0050
    |difference| = 0.0456  (inside 0.03: no)
  DiD(attn vs no-position, bin-shuffled) = 0.1098, positive 12/12 (reported, not barred)

=== H26-3  at what timescale does the read-out use order? ===
  w2    DiD 0.0066  positive 7/12
  w4    DiD -0.0028  positive 7/12
  w8    DiD 0.007  positive 8/12
  w16   DiD 0.0206  positive 10/12
  w32   DiD 0.0161  positive 9/12
  w64   DiD 0.0245  positive 12/12
  w128  DiD 0.0596  positive 12/12
  full  DiD 0.1208  positive 12/12

  tau-half WITHHELD: the half-maximum is not reached on the registered ladder

======================================================================
H26-1a NOT MET | H26-1b NOT MET | H26-2a MET
H26-2b and H26-3 are registered as questions and have no MET/NOT MET.
