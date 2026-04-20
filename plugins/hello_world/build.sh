#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
PLUGIN_NAME="opencode_plugin_hello_world"
OUTPUT_DIR="$PROJECT_DIR/plugins/bin"
WASM_SOURCE="$SCRIPT_DIR/target/wasm32-wasip1/release/${PLUGIN_NAME}.wasm"
WASM_DEST="$OUTPUT_DIR/${PLUGIN_NAME}.wasm"

mkdir -p "$OUTPUT_DIR"

cd "$SCRIPT_DIR"
cargo build --target wasm32-wasip1 --release -p opencode-plugin-hello-world

cp "$WASM_SOURCE" "$WASM_DEST"

echo "Built $WASM_DEST"