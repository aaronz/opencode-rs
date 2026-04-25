#!/bin/bash

set -e

OPENCODE_PATH="${OPENCODE_PATH:-$(command -v opencode 2>/dev/null || echo "$HOME/.opencode/bin/opencode")}"
MODEL="${MODEL:-minimax-cn/MiniMax-M2.7}"
SESSION_DIR="${SESSION_DIR:-sessions}"

CONSTRAINTS="## 重要约束
- 禁止使用 subagent 或 task 工具 spawning 其他 agent
- 禁止将工作委托给其他 agent
- 必须直接在当前 session 中完成所有分析工作
- 只使用 Read、Write、Edit、Grep、LSP 等直接工具
- 禁止向用户提问或请求确认，必须独立做出最佳判断并继续执行
- 如遇不确定情况，使用你自己的最佳判断，无需等待用户输入
- 禁止向用户提问或请求确认，必须独立做出最佳判断并继续执行
- 如遇不确定情况，使用你自己的最佳判断，无需等待用户输入"

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --prompt, -p       Prompt to execute (required if no --prompt-file)"
    echo "  --prompt-file, -f  Read prompt from file (required if no --prompt)"
    echo "  --loops, -n        Number of loops (default: 1)"
    echo "  --model, -m        Model to use (default: $MODEL)"
    echo "  --session-dir      Session export directory (default: $SESSION_DIR)"
    echo "  --no-constraints   Skip appending default constraints"
    echo "  --share-session    Share same session across all loops"
    echo "  --session          Start with a specific session ID"
    echo "  --print-logs       Print opencode logs to stderr"
    echo "  --log-level        Opencode log level: DEBUG, INFO, WARN, ERROR (default: INFO)"
    echo "  --verbose, -v      Shortcut for --print-logs --log-level DEBUG"
    echo "  --help, -h         Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 -p 'Implement feature X' -n 3"
    echo "  $0 -p 'Continue work' -n 3 --share-session"
    echo "  $0 -p 'Debug task' -n 2 --verbose"
    echo "  $0 -p 'Task with logs' --print-logs --log-level DEBUG"
    echo "  $0 --prompt-file ./task.txt -n 3"
    exit 1
}

LOOPS=1
PROMPT=""
PROMPT_FILE=""
SHARE_SESSION="false"
INITIAL_SESSION=""
PRINT_LOGS="false"
OPENCODE_LOG_LEVEL="INFO"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --prompt|-p)
            PROMPT="$2"
            shift 2
            ;;
        --prompt-file|-f)
            PROMPT_FILE="$2"
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
        --no-constraints)
            export APPEND_CONSTRAINTS="false"
            shift
            ;;
        --share-session)
            SHARE_SESSION="true"
            shift
            ;;
        --session)
            INITIAL_SESSION="$2"
            shift 2
            ;;
        --print-logs)
            PRINT_LOGS="true"
            shift
            ;;
        --log-level)
            OPENCODE_LOG_LEVEL="$2"
            shift 2
            ;;
        --verbose|-v)
            PRINT_LOGS="true"
            OPENCODE_LOG_LEVEL="DEBUG"
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

APPEND_CONSTRAINTS="${APPEND_CONSTRAINTS:-true}"

if [ "$1" = "help" ]; then
    usage
fi

if [ -z "$PROMPT" ] && [ -z "$PROMPT_FILE" ]; then
    echo "Error: either --prompt or --prompt-file is required"
    usage
fi

if [ -n "$PROMPT_FILE" ] && [ ! -f "$PROMPT_FILE" ]; then
    echo "Error: prompt file not found: $PROMPT_FILE"
    exit 1
fi

if [ -z "$PROMPT" ] && [ -n "$PROMPT_FILE" ]; then
    PROMPT=$(cat "$PROMPT_FILE")
fi

if [ ! -x "$OPENCODE_PATH" ]; then
    echo "Error: opencode not found at $OPENCODE_PATH"
    echo "Set OPENCODE_PATH environment variable to override"
    exit 1
fi

mkdir -p "$SESSION_DIR"

echo ""
echo "[INFO] Starting opencode loops"
echo "[INFO] Prompt: ${PROMPT:0:100}..."
[ ${#PROMPT} -gt 100 ] && echo "[INFO]     ... (truncated)"
echo "[INFO] Loops: $LOOPS"
echo "[INFO] Model: $MODEL"
echo "[INFO] Session dir: $SESSION_DIR"
echo "[INFO] Constraints appended: $APPEND_CONSTRAINTS"
echo "[INFO] Share session: $SHARE_SESSION"
echo "[INFO] Initial session: ${INITIAL_SESSION:-none}"
echo "[INFO] Print logs: $PRINT_LOGS"
echo "[INFO] Log level: $OPENCODE_LOG_LEVEL"
echo ""

build_full_prompt() {
    local user_prompt="$1"

    if [ "$APPEND_CONSTRAINTS" = "true" ]; then
        echo "$user_prompt

$CONSTRAINTS"
    else
        echo "$user_prompt"
    fi
}

extract_session_id() {
    local json_file="$1"
    if [ ! -f "$json_file" ]; then
        return 1
    fi

    local session_id
    session_id=$(grep -o '"sessionId":"[^"]*"' "$json_file" 2>/dev/null | head -1 | sed 's/"sessionId":"//;s/"//')
    if [ -n "$session_id" ]; then
        echo "$session_id"
        return 0
    fi

    session_id=$(grep -o 'ses_[a-f0-9]*' "$json_file" 2>/dev/null | head -1)
    if [ -n "$session_id" ]; then
        echo "$session_id"
        return 0
    fi

    return 1
}

run_opencode_loop() {
    local prompt="$1"
    local loop_num="$2"
    local total_loops="$3"
    local session_id="$4"
    local session_file="$SESSION_DIR/loop_${loop_num}_$(date +%Y%m%d_%H%M%S).json"

    echo ""
    echo "[INFO] =========================================="
    echo "[INFO] Loop $loop_num/$total_loops"
    echo "[INFO] =========================================="
    [ -n "$session_id" ] && echo "[INFO] Continuing session: $session_id"

    local start_time
    start_time=$(date +%s)

    local opencode_args=("-m" "$MODEL" "--dangerously-skip-permissions" "$prompt" "--format" "json")

    if [ "$PRINT_LOGS" = "true" ]; then
        opencode_args+=("--print-logs")
    fi

    if [ -n "$OPENCODE_LOG_LEVEL" ]; then
        opencode_args+=("--log-level" "$OPENCODE_LOG_LEVEL")
    fi

    if [ -n "$session_id" ]; then
        opencode_args+=("--continue" "--session" "$session_id")
    fi

    echo "[DEBUG] Running: opencode run ${opencode_args[*]}"
    echo "[DEBUG] Prompt length: $(echo -n "$prompt" | wc -c) chars"

    local output
    local exit_code=0

    output=$("$OPENCODE_PATH" run "${opencode_args[@]}" 2>&1) || exit_code=$?

    local elapsed=$(( $(date +%s) - start_time ))

    echo "$output" > "$session_file"
    echo "[INFO] Completed in ${elapsed}s (exit code: $exit_code)"
    echo "[DEBUG] Session exported to: $session_file"

    if [ $exit_code -ne 0 ]; then
        echo "[WARN] opencode exited with code $exit_code"
    fi

    if [ "$SHARE_SESSION" = "true" ] && [ -n "$session_id" ]; then
        return $exit_code
    fi

    local new_session_id
    new_session_id=$(extract_session_id "$session_file")
    [ -n "$new_session_id" ] && echo "[DEBUG] Session ID: $new_session_id"

    return $exit_code
}

FULL_PROMPT=$(build_full_prompt "$PROMPT")

CURRENT_SESSION="$INITIAL_SESSION"
EXIT_CODE=0

for ((i=1; i<=LOOPS; i++)); do
    if [ "$SHARE_SESSION" = "true" ] && [ -n "$CURRENT_SESSION" ]; then
        run_opencode_loop "$FULL_PROMPT" "$i" "$LOOPS" "$CURRENT_SESSION"
        loop_exit_code=$?
    else
        run_opencode_loop "$FULL_PROMPT" "$i" "$LOOPS" ""
        loop_exit_code=$?
        if [ "$SHARE_SESSION" = "true" ]; then
            CURRENT_SESSION=$(ls "$SESSION_DIR"/loop_${i}_*.json 2>/dev/null | head -1 | xargs extract_session_id 2>/dev/null || echo "")
        fi
    fi

    if [ $loop_exit_code -ne 0 ]; then
        EXIT_CODE=$loop_exit_code
        echo "[ERROR] Loop $i failed with exit code $loop_exit_code"
        if [ "$LOOPS" -gt 1 ] && [ $i -lt $LOOPS ]; then
            echo "[WARN] Continuing to next loop..."
        fi
    else
        echo "[INFO] Loop $i completed successfully"
    fi

    if [ $i -lt $LOOPS ]; then
        echo "[DEBUG] Waiting before next loop..."
        sleep 2
    fi
done

echo ""
echo "[INFO] =========================================="
echo "[INFO] All loops completed"
[ "$SHARE_SESSION" = "true" ] && [ -n "$CURRENT_SESSION" ] && echo "[INFO] Shared session: $CURRENT_SESSION"
echo "[INFO] Sessions saved in: $SESSION_DIR"
echo "[INFO] =========================================="

exit $EXIT_CODE