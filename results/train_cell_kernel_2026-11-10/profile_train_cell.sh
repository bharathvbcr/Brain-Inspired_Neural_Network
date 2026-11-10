#!/bin/sh
# Profile a 2-epoch ff+fixed+attn d32/L4 train-cell (published-2ms, h128) with
# macOS /usr/bin/sample. Same cell as binn-wsp-study/wsp/train_parity.sh.
# usage: profile_train_cell.sh <shd-instrument binary> <out dir> [sample seconds]
set -eu
BIN=$1
OUT=$2
SECS=${3:-40}
EV=/Users/bharath/Code/research/BINN/data/shd/events
mkdir -p "$OUT"
"$BIN" init --n-inputs 140 --hidden 128 --classes 20 --seed 5170001 --epochs 2 \
  --n-train 6987 --arm ff+fixed+attn --attn-dim 32 --attn-layers 4 \
  --weights "$OUT/init.weights" --orders "$OUT/orders.bin" > "$OUT/init.log" 2>&1
START=$(date +%s)
"$BIN" train-cell --train-events "$EV/train.events" --test-events "$EV/test.events" \
  --contract published-2ms --geometry adjacent-sum-5 \
  --weights "$OUT/init.weights" --orders "$OUT/orders.bin" --epochs 2 --arm ff+fixed+attn \
  --out "$OUT/cell.json" --seed 5170001 \
  --train-speakers "$EV/train.speakers" --val-speakers 3,6 \
  --save-final-weights "$OUT/final.weights" > "$OUT/train.log" 2>&1 &
PID=$!
echo "train-cell pid $PID"
sleep 3
/usr/bin/sample "$PID" "$SECS" 1 -file "$OUT/sample.txt" > /dev/null 2>&1 || true
wait "$PID"
END=$(date +%s)
echo "wall $((END - START)) s"
shasum -a 256 "$OUT/final.weights"
