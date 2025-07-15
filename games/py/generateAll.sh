#!/bin/bash

SRC_DIR="lineGames"
OUT_DIR="hrg"

rm "$OUT_DIR"/*
mkdir -p "$OUT_DIR"

for script in "$SRC_DIR"/*.py; do
    fileName=$(basename "$script" .py)
    if [[ "$fileName" == "lineGames" ]]; then
        continue
    fi
    python3 "$script" > "$OUT_DIR/${fileName}.hrg" 2>&1
    echo "Saved ${fileName}.hrg"
done
