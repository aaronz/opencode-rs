#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PLUGINS_DIR="$PROJECT_ROOT/plugins"
OUTPUT_DIR="$PLUGINS_DIR/bin"

echo "Building plugins from $PLUGINS_DIR"

if [ ! -d "$PLUGINS_DIR" ]; then
    echo "Error: plugins directory not found at $PLUGINS_DIR"
    exit 1
fi

mkdir -p "$OUTPUT_DIR"

plugin_count=0
success_count=0
failed_plugins=()

for plugin_dir in "$PLUGINS_DIR"/*/; do
    if [ ! -d "$plugin_dir" ]; then
        continue
    fi

    plugin_name=$(basename "$plugin_dir")
    ((plugin_count++))

    echo "Building plugin: $plugin_name"

    if [ ! -f "$plugin_dir/Cargo.toml" ]; then
        echo "  Skip: No Cargo.toml found in $plugin_name"
        continue
    fi

    cd "$plugin_dir"

    if cargo build --release --target wasm32-wasip1 2>&1; then
        wasm_pattern="${plugin_name}.wasm"
        wasm_file_candidate="$plugin_dir/target/wasm32-wasip1/release/opencode_plugin_${plugin_name}.wasm"
        if [ -f "$wasm_file_candidate" ]; then
            wasm_file="$wasm_file_candidate"
        elif [ -f "$plugin_dir/target/wasm32-wasip1/release/${wasm_pattern}" ]; then
            wasm_file="$plugin_dir/target/wasm32-wasip1/release/${wasm_pattern}"
        else
            wasm_file=$(find "$plugin_dir/target/wasm32-wasip1/release" -maxdepth 1 -name "*.wasm" -type f 2>/dev/null | head -n1)
        fi

        if [ -n "$wasm_file" ] && [ -f "$wasm_file" ]; then
            cp "$wasm_file" "$OUTPUT_DIR/"
            echo "  Success: $(basename "$wasm_file") copied to plugins/bin/"
            ((success_count++))
        else
            echo "  Error: WASM file not found for $plugin_name"
            failed_plugins+=("$plugin_name")
        fi
    else
        echo "  Error: Failed to build $plugin_name"
        failed_plugins+=("$plugin_name")
    fi
done

echo ""
echo "=== Build Summary ==="
echo "Plugins found: $plugin_count"
echo "Successfully built: $success_count"

if [ ${#failed_plugins[@]} -gt 0 ]; then
    echo "Failed plugins: ${failed_plugins[*]}"
    exit 1
fi

echo "All plugins built successfully!"