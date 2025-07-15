 #!/bin/bash

SRC_DIR="sources"
OUT_DIR="hrg"

for script in "$SRC_DIR"/*.py; do
    baseName=$(basename "$script")
    filename=$(basename "$script" .py)
    python3 "$script" > "$OUT_DIR/$filename.hrg" 2>&1
    echo "Saved $baseName"
done
