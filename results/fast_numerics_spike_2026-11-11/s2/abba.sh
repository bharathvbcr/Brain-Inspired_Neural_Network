#!/bin/sh
# Interleaved ABBA timing of a 2-epoch ff+fixed+attn d32/L4 train-cell, old
# (aa073b0) vs new (kernel/train-cell-simd) binary. Every run must reproduce the
# oracle weights sha; wall_secs is read from the cell JSON.
set -eu
P=/private/tmp/claude-501/-Users-bharath-Code-devtools-GitPulse/84f5d03c-7572-437b-a627-46a617edb850/scratchpad/prof
EV=/Users/bharath/Code/research/BINN/data/shd/events
ORACLE=9c364bd9441d1406cd97aa15078a83c1042b93180963f6bf159652528df512d3
W=/private/tmp/claude-501/-Users-bharath-Code-devtools-GitPulse/84f5d03c-7572-437b-a627-46a617edb850/scratchpad/spike/s2/${OUTDIR:-abba}
mkdir -p "$W"
"$P/bins/r4" init --n-inputs 140 --hidden 128 --classes 20 --seed 5170001 --epochs 2 \
  --n-train 6987 --arm ff+fixed+attn --attn-dim 32 --attn-layers 4 \
  --weights "$W/init.weights" --orders "$W/orders.bin" > /dev/null 2>&1
i=0
for tag in ${SEQ:-r4 blas blas r4 r4 blas blas r4}; do
  i=$((i + 1)); case $tag in r4) BINOF=r4; EXTRA="";; blas) BINOF=spike-blas; EXTRA="--numerics blas";; esac
  uptime | sed 's/.*load averages*: */load /' > "$W/$i.$tag.load"
  /usr/bin/time -l -o "$W/$i.$tag.time" "$P/bins/$BINOF" train-cell $EXTRA --train-events "$EV/train.events" --test-events "$EV/test.events" \
    --contract published-2ms --geometry adjacent-sum-5 \
    --weights "$W/init.weights" --orders "$W/orders.bin" --epochs 2 --arm ff+fixed+attn \
    --out "$W/$i.$tag.json" --seed 5170001 \
    --train-speakers "$EV/train.speakers" --val-speakers 3,6 \
    --save-final-weights "$W/$i.$tag.weights" > "$W/$i.$tag.log" 2>&1
  sha=$(shasum -a 256 "$W/$i.$tag.weights" | cut -d' ' -f1)
  wall=$(python3 -I -c "import json,sys; print(json.load(open(sys.argv[1]))['wall_secs'])" "$W/$i.$tag.json")
  ok=MISMATCH; [ "$sha" = "$ORACLE" ] && ok=oracle
  ins=$(awk '/instructions retired/ {print $1}' "$W/$i.$tag.time")
  cyc=$(awk '/cycles elapsed/ {print $1}' "$W/$i.$tag.time")
  usr=$(awk '/ user / {for (k = 1; k <= NF; k++) if ($k == "user") print $(k - 1)}' "$W/$i.$tag.time")
  echo "$i $tag wall=$wall user=$usr instructions=$ins cycles=$cyc weights=$ok $(cat "$W/$i.$tag.load")"
done
