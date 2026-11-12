#!/bin/sh
# usage: oracle.sh <bin> <outdir> [extra train-cell args...]   (2-epoch oracle cell; EPOCHS env overrides)
set -eu
BIN=$1; OUT=$2; shift 2
EV=/Users/bharath/Code/research/BINN/data/shd/events
E=${EPOCHS:-2}
mkdir -p "$OUT"
"$BIN" init --n-inputs 140 --hidden 128 --classes 20 --seed 5170001 --epochs $E --n-train 6987 --arm ff+fixed+attn --attn-dim 32 --attn-layers 4 --weights $OUT/init.weights --orders $OUT/orders.bin > $OUT/init.log 2>&1
$BIN train-cell --train-events $EV/train.events --test-events $EV/test.events --contract published-2ms --geometry adjacent-sum-5 --weights $OUT/init.weights --orders $OUT/orders.bin --epochs $E --arm ff+fixed+attn --out $OUT/cell.json --seed 5170001 --train-speakers $EV/train.speakers --val-speakers 3,6 --save-final-weights $OUT/final.weights "$@" > $OUT/train.log 2>&1
shasum -a 256 $OUT/final.weights | cut -d' ' -f1
