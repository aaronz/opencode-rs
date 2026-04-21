#!/bin/bash

ts_echo() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*"
}

DEFAULT_MODEL="minimax-cn/MiniMax-M2.7"

SESSION_EXPORT_DIR="${SESSION_EXPORT_DIR:-sessions}"
EXPORT_SESSIONS="${EXPORT_SESSIONS:-true}"

mkdir -p "$SESSION_EXPORT_DIR"

# Content caching to avoid re-reading files in prompts
# Usage: cache_file "VARIABLE_NAME" "/path/to/file"
cache_file() {
    local var_name="$1"
    local file_path="$2"
    if [ -f "$file_path" ] && [ -s "$file_path" ]; then
        eval "${var_name}=\$(cat \"$file_path\")"
    else
        eval "${var_name}="
    fi
}

# Batch cache multiple files at once
# Usage: cache_files GAP_ANALYSIS="$gap_file" SPEC="$spec_file" ...
cache_files() {
    while [ $# -gt 0 ]; do
        local var_name="${1%%=*}"
        local file_path="${1#*=}"
        cache_file "$var_name" "$file_path"
        shift
    done
}

check_file() {
    if [ ! -f "$1" ]; then
        echo "❌ 文件缺失: $1"
        return 1
    fi
    if [ ! -s "$1" ] || [ $(wc -c < "$1") -lt 10 ]; then
        echo "❌ 文件无效（内容过少）: $1"
        return 1
    fi
    ts_echo "✅ 文件存在: $1 ($(wc -c < "$1") bytes)"
    return 0
}

check_file_quiet() {
    if [ ! -f "$1" ]; then
        return 1
    fi
    if [ ! -s "$1" ] || [ $(wc -c < "$1") -lt 10 ]; then
        return 1
    fi
    return 0
}

rerun_if_missing() {
    local file="$1"
    local prompt="$2"
    local max_retries=${3:-2}
    local attempt=0
    local phase_name="${4:-rerun}"

    while [ $attempt -lt $max_retries ]; do
        if check_file "$file"; then
            return 0
        fi
        attempt=$((attempt + 1))
        if [ $attempt -lt $max_retries ]; then
            echo "🔄 重新生成 ($attempt/$max_retries)..."
            run_opencode_with_session_export "$prompt" "$SESSION_EXPORT_DIR/${phase_name}_retry${attempt}.json" "$MODEL"
        fi
    done

    if ! check_file "$file"; then
        echo "⚠️  文件生成失败: $file"
        return 1
    fi
    return 0
}

generate_if_missing() {
    local file="$1"
    local prompt="$2"
    local max_retries=${3:-5}

    if check_file_quiet "$file"; then
        echo "⏭️  跳过（已存在）: $file"
        return 0
    fi
    ts_echo "📝 生成文件: $file"
    rerun_if_missing "$file" "$prompt" "$max_retries"
}

run_phase() {
    local phase_name="$1"
    local output_file="$2"
    local prompt="$3"
    local max_retries=${4:-5}

    ts_echo ""
    ts_echo "[$phase_name]"

    if check_file_quiet "$output_file"; then
        echo "⏭️  跳过（已存在）: $output_file"
        return 0
    fi

    run_opencode_with_session_export "$prompt" "$SESSION_EXPORT_DIR/${phase_name}.json" "$MODEL"

    if check_file_quiet "$output_file"; then
        echo "✅ $phase_name 完成: $output_file"
        return 0
    else
        echo "🔄 $phase_name 失败，尝试重新生成..."
        rerun_if_missing "$output_file" "$prompt" "$max_retries"
    fi
}

save_checkpoint() {
    local iteration="$1"
    local phase="$2"
    local checkpoint_file="${3:-$OUTPUTS_DIR/.checkpoint}"

    ts_echo "iteration=$iteration" > "$checkpoint_file"
    ts_echo "phase=$phase" >> "$checkpoint_file"
    ts_echo "timestamp=$(date +%s)" >> "$checkpoint_file"
}

load_checkpoint() {
    local checkpoint_file="${1:-$OUTPUTS_DIR/.checkpoint}"

    if [ ! -f "$checkpoint_file" ]; then
        return 1
    fi

    while IFS= read -r line; do
        case "$line" in
            iteration=*) CURRENT_ITERATION="${line#*=}" ;;
            phase=*) CURRENT_PHASE="${line#*=}" ;;
        esac
    done < "$checkpoint_file"

    ts_echo "从检查点恢复: iteration=$CURRENT_ITERATION, phase=$CURRENT_PHASE"
    return 0
}

should_skip_phase() {
    local checkpoint_file="${1:-$OUTPUTS_DIR/.checkpoint}"
    local phase_to_check="$2"

    if [ ! -f "$checkpoint_file" ]; then
        return 1
    fi

    load_checkpoint "$checkpoint_file"
    local phase_num=$(echo "$CURRENT_PHASE" | grep -oE '[0-9]+' | head -1)
    local current_phase_num=$(echo "$phase_to_check" | grep -oE '[0-9]+' | head -1)

    if [ "$phase_num" -ge "$current_phase_num" ]; then
        return 0
    fi
    return 1
}

run_opencode_with_session_export() {
    local prompt="$1"
    local export_file="${2:-}"
    local model="${3:-$MODEL}"

    local temp_output
    temp_output=$(mktemp)

    local start_time
    start_time=$(date +%s)
    ts_echo "[DEBUG] run_opencode_with_session_export 开始 | model=$model"

    local opencode_exit_code=0
    local opencode_output

    opencode_output=$(timeout 600 opencode run -m "$model" --dangerously-skip-permissions "$prompt" --format json 2>&1) || opencode_exit_code=$?

    local elapsed=$(( $(date +%s) - start_time ))
    ts_echo "[DEBUG] opencode 完成 | elapsed=${elapsed}s | exit_code=$opencode_exit_code"

    if [ -n "$export_file" ]; then
        echo "$opencode_output" > "$export_file"
        ts_echo "[DEBUG] 已导出到: $export_file"
    fi

    if [ $opencode_exit_code -eq 124 ]; then
        ts_echo "⚠️  opencode 超时 (10分钟)，继续执行"
    elif [ $opencode_exit_code -ne 0 ]; then
        ts_echo "⚠️  opencode 异常退出: exit_code=$opencode_exit_code"
    fi

    rm -f "$temp_output"
}

export_session_by_id() {
    local session_id="$1"
    local export_file="$2"

    if [ -z "$session_id" ] || [ -z "$export_file" ]; then
        echo "⚠️  缺少session_id或export_file参数"
        return 1
    fi

    ts_echo "📦 导出Session: $session_id -> $export_file"
    opencode export "$session_id" > "$export_file" 2>/dev/null && echo "✅ 导出成功: $export_file" || echo "⚠️  导出失败"
}