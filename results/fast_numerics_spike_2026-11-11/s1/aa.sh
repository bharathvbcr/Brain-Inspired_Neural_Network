#!/bin/sh
# A/A: variant x threads, each its own dir. usage: aa.sh variant
V=$1; S=/private/tmp/claude-501/-Users-bharath-Code-devtools-GitPulse/84f5d03c-7572-437b-a627-46a617edb850/scratchpad; D=$S/spike/s1/aa
for T in 16 3 1 16; do
  O=$D/$V.t$T.$(date +%s)
  sha=$(RAYON_NUM_THREADS=$T nice -n 5 $S/spike/s1/oracle.sh $S/prof/bins/spike-fma $O --numerics $V)
  echo "$V threads=$T sha=$sha dir=$O" >> $D/$V.results
done
echo finished >> $D/$V.results
