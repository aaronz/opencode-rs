#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OPENCODE_PATH="${OPENCODE_PATH:-$(command -v opencode 2>/dev/null || echo "$HOME/.opencode/bin/opencode")}"
MODEL="${MODEL:-minimax-cn/MiniMax-M2.7}"
SESSION_DIR="${SESSION_DIR:-sessions}"

usage() {
    echo "Usage: $0 [OPTIONS] --prompt 'YOUR_PROMPT_HERE'"
    echo ""
    echo "Options:"
    echo "  --prompt, -p       Prompt to execute (required)"
    echo "  --loops, -n        Number of loops (default: 1)"
    echo "  --model, -m        Model to use (default: $MODEL)"
    echo "  --session-dir      Session export directory (default: $SESSION_DIR)"
    echo "  --verbose, -v      Verbose output"
    echo "  --help, -h         Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 -p 'Implement feature X' -n 3"
    echo "  $0 --prompt 'Fix bug Y' --loops 5 --verbose"
    exit 1
}

LOOPS=1
PROMPT=""
VERBOSE="false"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --prompt|-p)
            PROMPT="$2"
            shift 2
            ;;
        --loops|-n)
            LOOPS="$2"
            shift 2
            ;;
        --model|-m)
            MODEL="$2"
            shift 2
            ;;
        --session-dir)
            SESSION_DIR="$2"
            shift 2
            ;;
        --verbose|-v)
            VERBOSE="true"
            shift
            ;;
        --help|-h)
            usage
            ;;
        *)
            echo "Unknown option: $1"
            usage
            ;;
    esac
done

if [ -z "$PROMPT" ]; then
    echo "Error: --prompt is required"
    usage
fi

if [ ! -x "$OPENCODE_PATH" ]; then
    echo "Error: opencode not found at $OPENCODE_PATH"
    echo "Set OPENCODE_PATH environment variable to override"
    exit 1
fi

mkdir -p "$SESSION_DIR"

log() {
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] $1"
}

run_opencode_loop() {
    local prompt="$1"
    local loop_num="$2"
    local total_loops="$3"
    local session_file="$SESSION_DIR/loop_${loop_num}_$(date +%Y%m%d_%H%M%S).json"

    log "=========================================="
    log "Loop $loop_num/$total_loops"
    log "=========================================="

    local start_time
    start_time=$(date +%s)

    local output
    local exit_code=0

    if [ "$VERBOSE" = "true" ]; then
        echo "Running: opencode run -m '$MODEL' --dangerously-skip-permissions '$prompt'"
    fi

    output=$("$OPENCODE_PATH" run -m "$MODEL" --dangerously-skip-permissions "$prompt" --format json 2>&1) || exit_code=$?

    local elapsed=$(( $(date +%s) - start_time ))

    echo "$output" > "$session_file"
    log "Completed in ${elapsed}s (exit code: $exit_code)"
    log "Session exported to: $session_file"

    if [ $exit_code -ne 0 ]; then
        log "Warning: opencode exited with code $exit_code"
    fi

    return $exit_code
}

echo ""
log "Starting opencode loops"
log "Prompt: ${PROMPT:0:100}..."
[ ${#PROMPT} -gt 100 ] && echo "    ... (truncated)"
log "Loops: $LOOPS"
log "Model: $MODEL"
log "Session dir: $SESSION_DIR"
echo ""

for ((i=1; i<=LOOPS; i++)); do
    run_opencode_loop "$PROMPT" "$i" "$LOOPS"
    exit_code=$?

    if [ $exit_code -ne 0 ]; then
        log "Loop $i failed with exit code $exit_code"
        if [ "$LOOPS" -gt 1 ] && [ $i -lt $LOOPS ]; then
            log "Continuing to next loop..."
        fi
    fi

    if [ $i -lt $LOOPS ]; then
        log "Waiting before next loop..."
        sleep 2
    fi
done

echo ""
log "All loops completed"
log "Sessions saved in: $SESSION_DIR"