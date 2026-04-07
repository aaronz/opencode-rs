#!/bin/bash
# SpecKit 迭代脚本 v2.0 - Constitution驱动的迭代

set -e

# MODEL=${1:-"opencode/qwen3.6-plus-free"}
# MODEL=${1:-"opencode/minimax-m2.5-free"}
MODEL=${1:-"minimax-cn/MiniMax-M2.7"}
WORKSPACE_DIR="$(cd "$(dirname "$0")" && pwd)"
PRD_PATH="$WORKSPACE_DIR/TUI_PRD_Rust.md"
CONSTITUTION_PATH="$WORKSPACE_DIR/outputs/.specify/memory/constitution.md"

LAST_ITERATION=$(ls -d "$WORKSPACE_DIR/outputs/iteration-"* 2>/dev/null | sed 's/.*iteration-//' | sort -n | tail -1)
NEXT_ITERATION=${LAST_ITERATION:-0}
NEXT_ITERATION=$((NEXT_ITERATION + 1))
OUTPUTS_DIR="$WORKSPACE_DIR/outputs/iteration-${NEXT_ITERATION}"

mkdir -p "$OUTPUTS_DIR"

echo "=============================================="
echo "SpecKit 迭代开发 v2.0"
echo "=============================================="

echo ""
echo "[1/6] 执行PRD差距分析..."

GAP_ANALYSIS=$(cat << 'GAPEOF'
分析当前实现与PRD的差距：

## 任务
1. 读取当前实现目录结构
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

## 输出格式
# 差距分析报告

## 差距列表
| 差距项 | 严重程度 | 模块 | 修复建议 |

## P0/P1/P2问题分类
## 技术债务清单
GAPEOF
)

opencode run -m "$MODEL" "$GAP_ANALYSIS" > "$OUTPUTS_DIR/gap-analysis.md"
echo "差距分析完成: $OUTPUTS_DIR/gap-analysis.md"

echo ""
echo "[2/6] Constitution 检查..."

opencode run -m "$MODEL" "检查Constitution是否需要更新。

## Constitution
$(cat $CONSTITUTION_PATH 2>/dev/null || echo "Constitution不存在")

## 差距分析
$(cat $OUTPUTS_DIR/gap-analysis.md)

## 任务
1. 检查现有Constitution是否覆盖新的P0问题
2. 如需更新，提出Constitution修订建议
3. 确保新的设计决策符合Constitution

## 输出
Constitution更新建议保存到: $OUTPUTS_DIR/constitution_updates.md"

echo ""
echo "[3/6] 更新Spec..."

opencode run -m "$MODEL" "使用 /speckit.specify 命令更新规格文档。

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

## 输出
更新后的规格保存到: $OUTPUTS_DIR/spec_v${NEXT_ITERATION}.md"

echo ""
echo "[4/6] 更新Plan和Tasks..."

opencode run -m "$MODEL" "使用 /speckit.plan 和 /speckit.tasks 命令更新计划。

## Spec
$OUTPUTS_DIR/spec_v${NEXT_ITERATION}.md

## Constitution
$(cat $CONSTITUTION_PATH 2>/dev/null || echo "")

## 差距分析
$(cat $OUTPUTS_DIR/gap-analysis.md)

## 任务
1. 更新实现计划
2. 更新任务清单
3. 确保P0任务优先

## 输出
更新后的计划保存到: $OUTPUTS_DIR/plan_v${NEXT_ITERATION}.md
更新后的任务保存到: $OUTPUTS_DIR/tasks_v${NEXT_ITERATION}.md"

echo ""
echo "[5/6] 执行实现..."

opencode run -m "$MODEL" "使用 /speckit.implement 执行实现。

## 任务清单
$OUTPUTS_DIR/tasks_v${NEXT_ITERATION}.md

## Spec
$OUTPUTS_DIR/spec_v${NEXT_ITERATION}.md

## 实现目录
./outputs/src/

## 任务
1. 按任务清单执行实现
2. 优先完成P0任务
3. 确保代码符合Constitution
4. Build必须通过

## 验证
- npm run build 必须通过"

echo ""
echo "[6/6] 验证报告..."

opencode run -m "$MODEL" "生成迭代验证报告。

## 差距分析
$(cat $OUTPUTS_DIR/gap-analysis.md)

## 任务清单
$OUTPUTS_DIR/tasks_v${NEXT_ITERATION}.md

## 实现状态
检查./outputs/src/目录下的代码

## 输出格式
# 迭代验证报告

## P0问题状态
| 问题 | 状态 | 备注 |

## Constitution合规性
## PRD完整度
## 遗留问题
## 下一步建议

## 输出
验证报告保存到: ./outputs/iteration-2/verification-report.md"

echo ""
echo "=============================================="
echo "SpecKit 迭代完成!"
echo "=============================================="
