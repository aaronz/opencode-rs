#!/bin/bash
set -e

DEBUG=""
TEST=""
CARGO_BUILD_FLAGS="--release"

while [[ $# -gt 0 ]]; do
    case $1 in
        --debug)
            DEBUG="1"
            CARGO_BUILD_FLAGS="--debug"
            shift
            ;;
        --test)
            TEST="1"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--debug] [--test]"
            exit 1
            ;;
    esac
done

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/rust-opencode-port" && pwd)"
cd "$PROJECT_DIR"

echo "========================================"
echo "Building OpenCode RS"
echo "========================================"
echo "Project: $PROJECT_DIR"
echo "Build type: ${DEBUG:-release}"
echo ""

echo "Building workspace..."
cargo build $CARGO_BUILD_FLAGS

if [[ -n "$TEST" ]]; then
    echo ""
    echo "Running tests..."
    cargo test
fi

if [[ -n "$DEBUG" ]]; then
    BINARY_PATH="$PROJECT_DIR/target/debug/opencode-rs"
else
    BINARY_PATH="$PROJECT_DIR/target/release/opencode-rs"
fi

echo ""
echo "========================================"
echo "Build Complete!"
echo "========================================"
echo "Binary: $BINARY_PATH"

if [[ -f "$BINARY_PATH" ]]; then
    ls -lh "$BINARY_PATH"
fi

echo ""
echo "To run:"
echo "  $BINARY_PATH"
