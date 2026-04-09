#!/bin/bash
# SpecKit 迭代脚本 v2.0 - Constitution驱动的迭代
# 支持恢复模式：指定 --resume <iteration> 从断点继续

set -e

parse_args() {
    RESUME_ITERATION=""
    MODEL=""
    MAX_IMPLEMENTATION_ROUNDS=10
    PRD_INPUT=""

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --resume)
                RESUME_ITERATION="$2"
                shift 2
                ;;
            --model|-m)
                MODEL="$2"
                shift 2
                ;;
            --rounds|-r)
                MAX_IMPLEMENTATION_ROUNDS="$2"
                shift 2
                ;;
            --prd|-p)
                PRD_INPUT="$2"
                shift 2
                ;;
            *)
                shift
                ;;
        esac
    done

    MODEL=${MODEL:-"minimax-cn/MiniMax-M2.7"}
}

parse_args "$@"

WORKSPACE_DIR="$(cd "$(dirname "$0")" && pwd)"
CONSTITUTION_PATH="$WORKSPACE_DIR/iterations/.specify/memory/constitution.md"

if [ -n "$PRD_INPUT" ]; then
    if [ -d "$PRD_INPUT" ]; then
        mapfile -t prd_files < <(find "$PRD_INPUT" -maxdepth 1 -name "*.md" | sort)
        if [ ${#prd_files[@]} -eq 0 ]; then
            echo "❌ 文件夹中未找到.md文件: $PRD_INPUT"
            exit 1
        fi
        PRD_PATH="$OUTPUTS_DIR/_prd_combined.md"
        cat "${prd_files[@]}" > "$PRD_PATH"
        echo "📂 使用PRD文件夹: $PRD_INPUT (合并${#prd_files[@]}个文件)"
    elif [ -f "$PRD_INPUT" ]; then
        PRD_PATH="$PRD_INPUT"
        echo "📄 使用PRD文件: $PRD_PATH"
    else
        echo "❌ PRD路径不存在: $PRD_INPUT"
        exit 1
    fi
else
    PRD_PATH="$WORKSPACE_DIR/PRD.md"
fi

# 确定迭代编号
if [ -n "$RESUME_ITERATION" ]; then
    NEXT_ITERATION="$RESUME_ITERATION"
    OUTPUTS_DIR="$WORKSPACE_DIR/iterations/iteration-${NEXT_ITERATION}"
    if [ ! -d "$OUTPUTS_DIR" ]; then
        echo "❌ 指定迭代不存在: $OUTPUTS_DIR"
        exit 1
    fi
    echo "📦 恢复迭代 #${NEXT_ITERATION}"
else
    LAST_ITERATION=$(ls -d "$WORKSPACE_DIR/iterations/iteration-"* 2>/dev/null | sed 's/.*iteration-//' | sort -n | tail -1)
    NEXT_ITERATION=${LAST_ITERATION:-0}
    NEXT_ITERATION=$((NEXT_ITERATION + 1))
    OUTPUTS_DIR="$WORKSPACE_DIR/iterations/iteration-${NEXT_ITERATION}"
    mkdir -p "$OUTPUTS_DIR"
fi

# 检查文件是否存在
check_file() {
    if [ ! -f "$1" ]; then
        echo "  ❌ 文件缺失: $1"
        return 1
    fi
    if [ ! -s "$1" ] || [ $(wc -c < "$1") -lt 10 ]; then
        echo "  ❌ 文件无效（内容过少）: $1"
        return 1
    fi
    echo "  ✅ 文件存在: $1 ($(wc -c < "$1") bytes)"
    return 0
}

# 检查文件是否存在（不打印日志，静默模式）
check_file_quiet() {
    if [ ! -f "$1" ]; then
        return 1
    fi
    if [ ! -s "$1" ] || [ $(wc -c < "$1") -lt 10 ]; then
        return 1
    fi
    return 0
}

# 重新运行生成命令并检查文件
# 用法: rerun_if_missing <file> <prompt> [max_retries]
rerun_if_missing() {
    local file="$1"
    local prompt="$2"
    local max_retries=${3:-2}
    local attempt=0

    while [ $attempt -lt $max_retries ]; do
        if check_file "$file"; then
            return 0
        fi
        attempt=$((attempt + 1))
        if [ $attempt -lt $max_retries ]; then
            echo "  🔄 重新生成 ($attempt/$max_retries)..."
            opencode run -m "$MODEL" "$prompt"
        fi
    done

    if ! check_file "$file"; then
        echo "  ⚠️  文件生成失败: $file"
        return 1
    fi
    return 0
}

# 生成文件（如果不存在）
# 用法: generate_if_missing <file> <prompt> [max_retries]
generate_if_missing() {
    local file="$1"
    local prompt="$2"
    local max_retries=${3:-5}

    if check_file_quiet "$file"; then
        echo "  ⏭️  跳过（已存在）: $file"
        return 0
    fi
    echo "  📝 生成文件: $file"
    rerun_if_missing "$file" "$prompt" "$max_retries"
}

# 检查未完成的P0/P1任务数量（基于JSON文件）
check_remaining_p0_p1() {
    local json_file="$1"
    if [ ! -f "$json_file" ]; then
        echo "0"
        return
    fi

    local remaining=$(cat "$json_file" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    if isinstance(data, list):
        count = sum(1 for t in data if t.get('priority') in ['P0', 'P1'] and t.get('status') != 'done')
        print(count)
    elif isinstance(data, dict) and 'tasks' in data:
        count = sum(1 for t in data['tasks'] if t.get('priority') in ['P0', 'P1'] and t.get('status') != 'done')
        print(count)
    else:
        print('0')
except:
    print('0')
" 2>/dev/null || echo "0")

    echo "${remaining:-0}"
}

# 检查是否存在TODO任务（基于JSON文件）
has_todo_tasks_json() {
    local json_file="$1"
    if [ ! -f "$json_file" ]; then
        echo "0"
        return
    fi

    local has_todo=$(cat "$json_file" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    tasks = data if isinstance(data, list) else data.get('tasks', [])
    count = sum(1 for t in tasks if t.get('status') == 'todo')
    print(count)
except:
    print('0')
" 2>/dev/null || echo "0")

    echo "${has_todo:-0}"
}

# 生成结构化任务JSON文件
generate_tasks_json() {
    local task_file="$1"
    local json_file="$2"

    if [ -f "$json_file" ] && [ $(wc -c < "$json_file") -gt 10 ]; then
        echo "  ⏭️  跳过JSON生成（已存在）: $json_file"
        return 0
    fi

    if [ ! -f "$task_file" ]; then
        echo "  ⚠️  任务文件不存在，无法生成JSON: $task_file"
        return 1
    fi

    echo "  📝 生成结构化任务JSON: $json_file"

    local first_task=1
    local current_priority=""
    local task_id=""
    local task_desc=""
    local task_lines=""

    process_task() {
        if [[ -z "$task_id" || -z "$current_priority" ]]; then
            return
        fi

        local has_todo=0
        local has_done=0

        while IFS= read -r line; do
            if [[ "$line" =~ ^[[:space:]]*-[[:space:]]*\[ ]]; then
                if [[ "$line" =~ ^[[:space:]]*-[[:space:]]*\[[[:space:]]*\] ]]; then
                    has_todo=1
                else
                    has_done=1
                fi
            fi
        done <<< "$task_lines"

        local status="pending"
        if [[ $has_done -eq 1 && $has_todo -eq 0 ]]; then
            status="done"
        elif [[ $has_todo -eq 1 ]]; then
            status="todo"
        fi

        local desc_json=$(echo "$task_desc" | jq -Rs '.')

        if [[ $first_task -eq 0 ]]; then
            echo ","
        fi
        first_task=0

        echo "    {"
        echo "      \"id\": \"$task_id\","
        echo "      \"priority\": \"$current_priority\","
        echo "      \"description\": $desc_json,"
        echo "      \"status\": \"$status\""
        echo -n "    }"

        task_id=""
        task_desc=""
        task_lines=""
    }

    {
        echo "{"
        echo '  "tasks": ['
        first_task=1

        while IFS= read -r line; do
            if [[ "$line" =~ ^##[[:space:]]*P0 ]]; then
                process_task
                current_priority="P0"
            elif [[ "$line" =~ ^##[[:space:]]*P1 ]]; then
                process_task
                current_priority="P1"
            elif [[ "$line" =~ ^##[[:space:]]*P2 ]]; then
                process_task
                current_priority="P2"
            elif [[ "$line" =~ ^###[[:space:]]*([A-Z][A-Z0-9]*-[0-9][0-9]*):[[:space:]]*(.+) ]]; then
                local _mid="${BASH_REMATCH[1]}" _mdesc="${BASH_REMATCH[2]}"
                process_task
                task_id="$_mid"
                task_desc="$_mdesc"
                task_lines=""
            elif [[ -n "$current_priority" && -n "$task_id" ]]; then
                task_lines="${task_lines}${line}"$'\n'
            fi
        done < "$task_file"

        process_task

        echo ""
        echo "  ]"
        echo "}"
    } > "$json_file"

    if [ -f "$json_file" ] && [ $(wc -c < "$json_file") -gt 10 ]; then
        echo "  ✅ JSON生成成功"
        return 0
    else
        echo "  ⚠️  JSON生成失败"
        return 1
    fi
}

# 检查是否存在TODO任务
has_todo_tasks() {
    local task_file="$1"
    if [ ! -f "$task_file" ]; then
        echo "0"
        return
    fi

    if grep -qE "\|\s*[^|]*\|\s*TODO\s*\|" "$task_file"; then
        echo "1"
    else
        echo "0"
    fi
}

echo "=============================================="
echo "SpecKit 迭代开发 v2.0"
echo "=============================================="
echo "迭代目录: $OUTPUTS_DIR"
echo "模型: $MODEL"
echo "实现轮次: $MAX_IMPLEMENTATION_ROUNDS"
echo ""

echo "[1/6] 执行PRD差距分析..."

PROMPT_GAP_ANALYSIS="分析当前实现与PRD的差距，并将完整的差距分析报告写入文件：$OUTPUTS_DIR/gap-analysis.md

## 重要约束
- 禁止使用 subagent 或 task 工具 spawning 其他 agent
- 禁止将工作委托给其他 agent
- 必须直接在当前 session 中完成所有分析工作
- 只使用 Read、Write、Edit、Grep、LSP 等直接工具

## 任务
1. 读取当前实现目录结构（src/目录、iterations/src/目录等）
2. 读取PRD.md识别核心功能需求
3. 对比实现与PRD的差距

## 分析维度
1. 功能完整性：PRD中描述的功能是否都已实现？
2. 接口完整性：API是否完整？CRUD是否齐全？
3. 前端完整性：PRD中描述的页面/组件是否都已实现？
4. 数据模型：PRD中的数据实体是否都已建模？
5. 配置管理：PRD中要求的配置项是否都已实现？
6. 测试覆盖：是否有必要的测试？

## 通用差距识别
- 缺失的功能模块
- 不完整的实现
- 未连接的模块
- 硬编码/魔法数字
- 错误处理缺失
- 类型定义缺失

## 输出要求
将完整的差距分析报告写入到：$OUTPUTS_DIR/gap-analysis.md

报告必须包含：
1. 差距列表（表格格式：差距项 | 严重程度 | 模块 | 修复建议）
2. P0/P1/P2问题分类（必须包含P0阻断性问题）
3. 技术债务清单
4. 实现进度总结"

if ! check_file_quiet "$OUTPUTS_DIR/gap-analysis.md"; then
    opencode run -m "$MODEL" "$PROMPT_GAP_ANALYSIS"
fi
generate_if_missing "$OUTPUTS_DIR/gap-analysis.md" "$PROMPT_GAP_ANALYSIS" 5

echo ""
echo "[2/6] Constitution 检查..."

PROMPT_CONSTITUTION="检查Constitution是否需要更新，并将更新建议写入文件：$OUTPUTS_DIR/constitution_updates.md

## 重要约束
- 禁止使用 subagent 或 task 工具 spawning 其他 agent
- 禁止将工作委托给其他 agent
- 必须直接在当前 session 中完成所有分析工作
- 只使用 Read、Write、Edit、Grep、LSP 等直接工具

## Constitution
$(cat $CONSTITUTION_PATH 2>/dev/null || echo "Constitution不存在")

## 差距分析
$(cat $OUTPUTS_DIR/gap-analysis.md)

## 任务
1. 检查现有Constitution是否覆盖新的P0问题
2. 如需更新，提出Constitution修订建议
3. 确保新的设计决策符合Constitution

## 输出要求
将Constitution更新建议写入到：$OUTPUTS_DIR/constitution_updates.md"

if ! check_file_quiet "$OUTPUTS_DIR/constitution_updates.md"; then
    opencode run -m "$MODEL" "$PROMPT_CONSTITUTION"
fi
generate_if_missing "$OUTPUTS_DIR/constitution_updates.md" "$PROMPT_CONSTITUTION" 5

echo ""
echo "[3/6] 更新Spec..."

PROMPT_SPEC="基于PRD和差距分析，更新规格文档，并写入文件：$OUTPUTS_DIR/spec_v${NEXT_ITERATION}.md

## 重要约束
- 禁止使用 subagent 或 task 工具 spawning 其他 agent
- 禁止将工作委托给其他 agent
- 必须直接在当前 session 中完成所有分析工作
- 只使用 Read、Write、Edit、Grep、LSP 等直接工具

## PRD
$(cat $PRD_PATH)

## 差距分析
$(cat $OUTPUTS_DIR/gap-analysis.md)

## Constitution
$(cat $CONSTITUTION_PATH 2>/dev/null || echo "使用默认Constitution")

## 任务
1. 基于差距分析，更新spec.md
2. 确保新功能有对应的规格定义
3. 添加功能需求编号(FR-XXX)

## 输出要求
将更新后的规格文档写入到：$OUTPUTS_DIR/spec_v${NEXT_ITERATION}.md"

if ! check_file_quiet "$OUTPUTS_DIR/spec_v${NEXT_ITERATION}.md"; then
    opencode run -m "$MODEL" "$PROMPT_SPEC"
fi
generate_if_missing "$OUTPUTS_DIR/spec_v${NEXT_ITERATION}.md" "$PROMPT_SPEC" 5

echo ""
echo "[4/6] 更新Plan和Tasks..."

PROMPT_PLAN="基于Spec更新实现计划和任务清单，并将它们写入文件。

## 重要约束
- 禁止使用 subagent 或 task 工具 spawning 其他 agent
- 禁止将工作委托给其他 agent
- 必须直接在当前 session 中完成所有分析工作
- 只使用 Read、Write、Edit、Grep、LSP 等直接工具

## Spec
$(cat $OUTPUTS_DIR/spec_v${NEXT_ITERATION}.md)

## Constitution
$(cat $CONSTITUTION_PATH 2>/dev/null || echo "")

## 差距分析
$(cat $OUTPUTS_DIR/gap-analysis.md)

## 任务
1. 更新实现计划
2. 更新任务清单
3. 确保P0任务优先

## 输出要求
将更新后的计划写入到：$OUTPUTS_DIR/plan_v${NEXT_ITERATION}.md
将更新后的任务清单写入到：$OUTPUTS_DIR/tasks_v${NEXT_ITERATION}.md"

PROMPT_PLAN_FILE="基于Spec更新实现计划和任务清单，并将它们写入文件：$OUTPUTS_DIR/plan_v${NEXT_ITERATION}.md

## 重要约束
- 禁止使用 subagent 或 task 工具 spawning 其他 agent
- 禁止将工作委托给其他 agent
- 必须直接在当前 session 中完成所有分析工作"

PROMPT_TASKS_FILE="基于Spec更新实现计划和任务清单，并将它们写入文件：$OUTPUTS_DIR/tasks_v${NEXT_ITERATION}.md

## 重要约束
- 禁止使用 subagent 或 task 工具 spawning 其他 agent
- 禁止将工作委托给其他 agent
- 必须直接在当前 session 中完成所有分析工作"

if ! check_file_quiet "$OUTPUTS_DIR/plan_v${NEXT_ITERATION}.md" || ! check_file_quiet "$OUTPUTS_DIR/tasks_v${NEXT_ITERATION}.md"; then
    opencode run -m "$MODEL" "$PROMPT_PLAN"
fi
generate_if_missing "$OUTPUTS_DIR/plan_v${NEXT_ITERATION}.md" "$PROMPT_PLAN_FILE" 5
generate_if_missing "$OUTPUTS_DIR/tasks_v${NEXT_ITERATION}.md" "$PROMPT_TASKS_FILE" 5

TASKS_JSON="$OUTPUTS_DIR/tasks_v${NEXT_ITERATION}.json"
if ! check_file_quiet "$TASKS_JSON"; then
    generate_tasks_json "$OUTPUTS_DIR/tasks_v${NEXT_ITERATION}.md" "$TASKS_JSON"
fi

echo ""
echo "[5/6] 执行实现..."

TASK_FILE="$OUTPUTS_DIR/tasks_v${NEXT_ITERATION}.md"
TASKS_JSON="$OUTPUTS_DIR/tasks_v${NEXT_ITERATION}.json"

for round in $(seq 1 $MAX_IMPLEMENTATION_ROUNDS); do
    echo ""
    echo "=============================================="
    echo "实现轮次 $round/$MAX_IMPLEMENTATION_ROUNDS"
    echo "=============================================="

    if [ ! -f "$TASKS_JSON" ]; then
        if [ -f "$TASK_FILE" ]; then
            generate_tasks_json "$TASK_FILE" "$TASKS_JSON"
        else
            echo "  ⚠️  任务文件不存在: $TASK_FILE"
            break
        fi
    fi

    remaining_p0_p1=$(check_remaining_p0_p1 "$TASKS_JSON")
    echo "  📋 剩余未完成的P0/P1任务: $remaining_p0_p1"

    todo_tasks=$(has_todo_tasks_json "$TASKS_JSON")
    echo "  📋 存在TODO任务: $todo_tasks"

    if [ "$remaining_p0_p1" -eq 0 ] && [ "$todo_tasks" -eq 0 ]; then
        echo "  ✅ 所有P0/P1任务已完成，无TODO任务!"
        break
    fi

    if [ "$todo_tasks" -gt 0 ]; then
        echo "  ⚠️  存在TODO任务，继续实现..."
    fi

    echo ""
    echo "  🔄 执行实现..."

    opencode run -m "$MODEL" "使用 /speckit.implement 执行实现。

## 重要约束
- 禁止使用 subagent 或 task 工具 spawning 其他 agent
- 禁止将工作委托给其他 agent
- 必须直接在当前 session 中完成所有实现工作
- 只使用 Read、Write、Edit、Grep、LSP、Bash 等直接工具

## 任务清单 (Markdown)
$(cat $TASK_FILE)

## 任务清单 (JSON)
$(cat $TASKS_JSON 2>/dev/null || echo '{"tasks":[]}')

## Spec
$(cat $OUTPUTS_DIR/spec_v${NEXT_ITERATION}.md)

## 实现目录
./iterations/src/

## 当前轮次信息
- 这是第 $round 轮实现（共最多 $MAX_IMPLEMENTATION_ROUNDS 轮）
- 剩余未完成的P0/P1任务: $remaining_p0_p1

## 任务
1. 按任务清单执行实现
2. 优先完成P0任务（阻断性问题）
3. 确保代码符合Constitution
4. Build必须通过

## 验证
- npm run build 必须通过

## 重要提醒
- 如果任务状态已标记为 done，完成，跳过该任务
- 专注于未完成的任务
- 每次完成一个任务，必须更新两个文件：
  1. 任务markdown文件中的Status改为 ✅ Done
  2. 任务JSON文件中的status改为 \"done\""

    if [ $round -eq $MAX_IMPLEMENTATION_ROUNDS ] && [ "$remaining_p0_p1" -gt 0 ]; then
        echo ""
        echo "  ⚠️  达到最大实现轮次，仍有 $remaining_p0_p1 个P0/P1任务未完成"
        echo "  ⚠️  继续到验证阶段..."
    fi
done

echo ""
echo "[6/6] 验证报告..."

PROMPT_VERIFICATION="生成迭代验证报告，并将报告写入文件：$OUTPUTS_DIR/verification-report.md

## 重要约束
- 禁止使用 subagent 或 task 工具 spawning 其他 agent
- 禁止将工作委托给其他 agent
- 必须直接在当前 session 中完成所有验证工作
- 只使用 Read、Write、Edit、Grep、LSP、Bash 等直接工具

## 差距分析
$(cat $OUTPUTS_DIR/gap-analysis.md)

## 任务清单
$(cat $OUTPUTS_DIR/tasks_v${NEXT_ITERATION}.md)

## 实现状态
检查./iterations/src/目录下的代码

## 输出要求
将完整的迭代验证报告写入到：$OUTPUTS_DIR/verification-report.md

报告必须包含：
1. P0问题状态（表格：问题 | 状态 | 备注）
2. Constitution合规性检查
3. PRD完整度评估
4. 遗留问题清单
5. 下一步建议"

if ! check_file_quiet "$OUTPUTS_DIR/verification-report.md"; then
    opencode run -m "$MODEL" "$PROMPT_VERIFICATION"
fi
generate_if_missing "$OUTPUTS_DIR/verification-report.md" "$PROMPT_VERIFICATION" 5

echo ""
echo "=============================================="
echo "SpecKit 迭代完成!"
echo "=============================================="
