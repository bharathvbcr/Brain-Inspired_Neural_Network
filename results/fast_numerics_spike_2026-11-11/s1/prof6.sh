#!/bin/sh
# 6-epoch attention cell; sample 60s starting ~20s in. usage: prof6.sh <bin> <outdir>
set -eu
BIN=$1; OUT=$2
EV=/Users/bharath/Code/research/BINN/data/shd/events
mkdir -p $OUT
$BIN init --n-inputs 140 --hidden 128 --classes 20 --seed 5170001 --epochs 6 --n-train 6987 --arm ff+fixed+attn --attn-dim 32 --attn-layers 4 --weights $OUT/init.weights --orders $OUT/orders.bin > $OUT/init.log 2>&1
$BIN train-cell --train-events $EV/train.events --test-events $EV/test.events --contract published-2ms --geometry adjacent-sum-5 --weights $OUT/init.weights --orders $OUT/orders.bin --epochs 6 --arm ff+fixed+attn --out $OUT/cell.json --seed 5170001 --train-speakers $EV/train.speakers --val-speakers 3,6 --save-final-weights $OUT/final.weights > $OUT/train.log 2>&1 &
PID=$!
echo pid $PID
sleep 20
/usr/bin/sample $PID 60 1 -file $OUT/sample.txt > /dev/null 2>&1 || true
wait $PID
echo done
