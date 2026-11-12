#!/bin/sh
# usage: cell.sh BIN OUTDIR [extra args]   (init shared from $S/spike/s2/init)
set -eu
S=/private/tmp/claude-501/-Users-bharath-Code-devtools-GitPulse/84f5d03c-7572-437b-a627-46a617edb850/scratchpad/spike/s2
EV=/Users/bharath/Code/research/BINN/data/shd/events
BIN=$1; OUT=$2; shift 2
mkdir -p $OUT $S/init
[ -f $S/init/init.weights ] || $BIN init --n-inputs 140 --hidden 128 --classes 20 --seed 5170001 --epochs 2 --n-train 6987 --arm ff+fixed+attn --attn-dim 32 --attn-layers 4 --weights $S/init/init.weights --orders $S/init/orders.bin >/dev/null 2>&1
$BIN train-cell --train-events $EV/train.events --test-events $EV/test.events --contract published-2ms --geometry adjacent-sum-5 --weights $S/init/init.weights --orders $S/init/orders.bin --epochs 2 --arm ff+fixed+attn --out $OUT/cell.json --seed 5170001 --train-speakers $EV/train.speakers --val-speakers 3,6 --save-final-weights $OUT/final.weights "$@" > $OUT/log 2>&1
shasum -a 256 $OUT/final.weights | cut -d' ' -f1
