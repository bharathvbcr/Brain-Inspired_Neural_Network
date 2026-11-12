#!/bin/sh
B=/private/tmp/claude-501/-Users-bharath-Code-devtools-GitPulse/84f5d03c-7572-437b-a627-46a617edb850/scratchpad/spike/s2/../../prof/bins/spike-blas
for cfg in "1 st" "3 st" "16 st" "16 st2" "16 mt" "16 mt2"; do
  set -- $cfg
  if [ "$2" = "mt" ] || [ "$2" = "mt2" ]; then export BINN_BLAS_MULTITHREADED=1; else unset BINN_BLAS_MULTITHREADED; fi
  sha=$(RAYON_NUM_THREADS=$1 nice -n 5 /private/tmp/claude-501/-Users-bharath-Code-devtools-GitPulse/84f5d03c-7572-437b-a627-46a617edb850/scratchpad/spike/s2/cell.sh $B /private/tmp/claude-501/-Users-bharath-Code-devtools-GitPulse/84f5d03c-7572-437b-a627-46a617edb850/scratchpad/spike/s2/aa_$1_$2 --numerics blas)
  w=$(python3 -I -c "import json,sys;d=json.load(open(sys.argv[1]));print(d['wall_secs'],d['accuracy'],d['mean_loss'])" /private/tmp/claude-501/-Users-bharath-Code-devtools-GitPulse/84f5d03c-7572-437b-a627-46a617edb850/scratchpad/spike/s2/aa_$1_$2/cell.json)
  echo "threads=$1 blas=$2 sha=$sha wall/acc/loss=$w"
done
