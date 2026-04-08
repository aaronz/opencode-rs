# OpenCode-RS Implementation Plan v19

**版本**: 19  
**日期**: 2026年4月8日  
**基于**: Spec v19 + Gap Analysis v19  
**总体进度**: 85%  
**状态**: 执行中

---

## 1. 执行摘要

### 1.1 当前状态

| 指标 | 状态 |
|------|------|
| 总体进度 | █████████████████████░░░░░░░░░ 85% |
| P0 阻断问题 | ✅ 无 |
| P1 高优先级 | 🔲 5项进行中 |
| P2 中优先级 | 🔲 11项待处理 |
| 技术债务 | 🔲 7项待清理 |

### 1.2 核心架构状态

✅ **稳定模块**: Core领域模型(95%) | Config(95%) | LLM Provider(95%) | Agent系统(95%) | Permission(90%) | CLI(95%)

🟡 **进行中模块**: TUI(80%) | Server(80%) | Storage(85%) | Tool Runtime(85%) | LSP(60%) | MCP(70%) | Plugin(70%) | Skills(85%) | Git(70%) | Session(85%)

❌ **缺失/待实现**: LSP 1.1能力 | Web UI | revert/unrevert

---

## 2. P0 阻断性问题 (无)

核心架构稳定，主要功能已实现。无需P0阻断性修复。

---

## 3. P1 高优先级修复计划 (本周)

| 任务ID | 任务 | FR-ID | 依赖 | 预计工时 | 状态 |
|--------|------|-------|------|---------|------|
| **P1-T1** | 打字机效果 | FR-006, FR-117 | 无 | M | 🔲 进行中 |
| **P1-T2** | Token实时显示验证 | FR-007 | P1-T1 | S | 🔲 待实现 |
| **P1-T3** | /thinking模式传递 | FR-059, FR-211 | 无 | M | 🔲 进行中 |
| **P1-T4** | 上下文预算触发 | FR-171, FR-172, FR-173, FR-212 | P1-T3 | M | 🔲 待实现 |
| **P1-T5** | /share远程分享 | FR-153, FR-101 | 无 | M | 🔲 部分实现 |

### 3.1 P1-T1: 打字机效果

**问题**: `input_widget.start_typewriter()` 被调用但增量渲染未实现

**修复方案**:
1. 在 `check_llm_events()` 中实现真正的增量文本渲染
2. 使用 `typewriter_speed` 配置控制速度
3. 确保流式输出时逐字符/逐词显示

**关键文件**:
- `crates/tui/src/components/input_widget.rs`
- `crates/tui/src/app.rs` (check_llm_events)

**验收标准**:
- [ ] 流式输出可见打字机效果
- [ ] 速度可配置 (typewriter_speed)
- [ ] 可中途打断

### 3.2 P1-T2: Token实时显示验证

**问题**: `status_bar.update_usage()` 调用存在但UI实际显示需验证

**修复方案**:
1. 确认 `StatusBar.update_usage()` 实际渲染逻辑
2. 验证 Token 计数器在状态栏正确显示
3. 修复渲染链路问题

**关键文件**:
- `crates/tui/src/components/status_bar.rs`

**验收标准**:
- [ ] 状态栏显示实时token使用量
- [ ] 输入/输出token分开统计

### 3.3 P1-T3: /thinking模式传递

**问题**: `thinking_mode` 标志存在但未传递给 LLM provider

**修复方案**:
1. 在消息构建时注入 thinking 标志到 LLM provider
2. 确保 `init_llm_provider()` 或 `send_message()` 时传递
3. 验证 `ThinkingBlock` 组件正确显示/隐藏

**关键文件**:
- `crates/tui/src/app.rs`
- `crates/llm/src/` (provider接口)

**验收标准**:
- [ ] /thinking 命令可切换思考模式
- [ ] 思考块可见性正确切换
- [ ] thinking标志正确传递给provider

### 3.4 P1-T4: 上下文预算触发

**问题**: `ContextBudget` 存在但未启用，85%/92%/95%阈值不生效

**修复方案**:
1. 实现 `BudgetLimit` 数据模型 (FR-212)
2. 在消息发送前检查token使用量
3. 达到阈值时自动触发 compact

**关键文件**:
- `crates/core/src/context/` (ContextBudget)
- `crates/tui/src/app.rs`

**验收标准**:
- [ ] 85%阈值触发compact提示
- [ ] 92%阈值强制compact
- [ ] 95%阈值阻止继续输入

### 3.5 P1-T5: /share远程分享

**问题**: `/share` 只导出到临时文件，未实现远程分享

**修复方案**:
1. 实现 `ShareStatus` 数据模型 (FR-210)
2. 实现远程分享服务或连接到外部服务
3. 完成 `SessionShare` 对话框 (FR-101)

**关键文件**:
- `crates/tui/src/dialogs/` (SessionShare)
- `crates/storage/src/` (ShareStatus)

**验收标准**:
- [ ] 生成分享链接
- [ ] 链接可访问
- [ ] 支持取消分享

---

## 4. P2 中优先级修复计划 (下周~下两周)

| 任务ID | 任务 | FR-ID | 依赖 | 预计工时 | 状态 |
|--------|------|-------|------|---------|------|
| **P2-T1** | /unshare实现 | FR-062, FR-102, FR-154, FR-210 | P1-T5 | S | 🔲 待实现 |
| **P2-T2** | /compact摘要 | FR-044, FR-155 | P1-T4 | M | 🔲 部分实现 |
| **P2-T3** | LSP 1.1能力 | FR-183-186 | 无 | L | 🔲 未实现 |
| **P2-T4** | MCP工具集成 | FR-193, FR-194 | 无 | M | 🔲 进行中 |
| **P2-T5** | /search实现 | FR-054 | 无 | M | 🔲 未实现 |
| **P2-T6** | /diff集成 | FR-055 | 无 | M | ⚠️ 部分实现 |
| **P2-T7** | Web UI | FR-220-222 | P2-T3 | L | 🔲 进行中 |
| **P2-T8** | 技术债务清理 | TECH-001~007 | 无 | M | 🔲 待清理 |

### 4.1 P2-T1: /unshare实现

**问题**: 只有占位消息，取消分享功能缺失

**修复方案**:
1. 实现 `ShareStatus` 数据模型
2. 实现 `SessionUnshare` 对话框 (FR-102)
3. 清理分享状态

### 4.2 P2-T2: /compact摘要

**问题**: `/compact` 只显示消息，未实际调用 `SummaryAgent`

**修复方案**:
1. 实现 `CompactionAgent` 调用
2. 确保摘要内容正确替换历史消息
3. 验证compact后上下文正确

### 4.3 P2-T3: LSP 1.1能力

**问题**: definition/references/hover/code actions 未实现

**修复方案**:
1. 实现 `goto_definition`
2. 实现 `find_references`
3. 实现 `hover_information`
4. 实现 `code_actions`

**关键文件**:
- `crates/lsp/src/`

### 4.4 P2-T4: MCP工具集成

**问题**: `McpToolAdapter` 存在但集成待验证

**修复方案**:
1. 验证 `register_mcp_tools()` 调用链路
2. 实现Token成本控制 (FR-194)
3. 端到端测试MCP工具执行

### 4.5 P2-T5: /search实现

**问题**: 只有Custom占位符，无法搜索对话历史

**修复方案**:
1. 实现对话历史搜索功能
2. 支持全文搜索
3. 高亮匹配结果

### 4.6 P2-T6: /diff集成

**问题**: diff视图存在但未集成到命令

**修复方案**:
1. 将 `DiffView` 组件与 `/diff` 命令集成
2. 支持选择分支/提交
3. 支持语法高亮

### 4.7 P2-T7: Web UI

**问题**: `web_ui.rs` 存在但功能不完整

**修复方案**:
1. 完成Web UI基础框架 (FR-220)
2. 实现会话管理UI (FR-221)
3. 实现聊天界面UI (FR-222)

### 4.8 P2-T8: 技术债务清理

**问题**: 多个技术债务影响代码质量

**清理项**:
| 债务ID | 问题 | 修复方案 |
|--------|------|----------|
| TECH-001 | Custom(String)命令占位符10+个 | 逐一实现或移除 |
| TECH-002 | 硬编码值(MAX_OUTPUT_SIZE=100KB等) | 移至配置 |
| TECH-003 | 重复命令定义(undo定义两次) | 清理重复 |
| TECH-004 | Error处理不一致 | 统一Result类型 |
| TECH-005 | 魔法数字(100, 5000等) | 命名常量 |
| TECH-006 | `working_dir` 字段未使用 | 删除或实现 |
| TECH-007 | 注释代码散布 | 清理 |

---

## 5. 缺失数据模型

| 模型 | 严重程度 | 说明 | 修复FR |
|------|----------|------|---------|
| `ShareStatus` | P2 | 会话分享状态管理 | FR-210 |
| `ThinkingMode` | P1 | 思考模式配置传递 | FR-211 |
| `BudgetLimit` | P1 | 预算限制配置 | FR-212 |
| `UsageStats` | P2 | 使用统计聚合 | FR-213 |

---

## 6. 测试覆盖目标

### 6.1 当前覆盖

| 模块 | 当前覆盖 | 目标 |
|------|----------|------|
| opencode-core | ? | 70% |
| opencode-server | ? | 60% |
| opencode-tools | ? | 60% |
| opencode-permission | ? | 70% |
| opencode-storage | ? | 60% |
| opencode-llm | ? | 50% |

### 6.2 补充测试计划

- [ ] TUI 渲染测试 (ratatui-testing)
- [ ] 集成测试 (session + LLM + tools)
- [ ] E2E 测试框架
- [ ] 权限系统完整测试
- [ ] LSP 桥接测试
- [ ] MCP 集成测试

---

## 7. 里程碑

| 里程碑 | 目标日期 | 交付内容 |
|--------|----------|----------|
| M1: P1 完成 | 2026-04-14 | 打字机效果、Token显示、thinking模式、预算触发、share远程 |
| M2: P2 核心完成 | 2026-04-21 | unshare、compact、LSP 1.1、MCP集成 |
| M3: 技术债务清理 | 2026-04-28 | TECH-001~007 全部清理 |
| M4: Web UI基础 | 2026-05-05 | FR-220-222 基础功能 |
| M5: v19 发布 | 2026-05-12 | 90%+ 完成度 |

---

## 8. 风险与依赖

### 8.1 关键风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| LSP 1.1 实现复杂度高 | 时间延迟 | 分解为独立任务 |
| Web UI 需要前端资源 | 人力不足 | 考虑使用现有框架 |
| 技术债务清理耗时 | 影响进度 | 碎片化处理 |

### 8.2 依赖关系

```
P1-T4 (预算触发) → P1-T3 (thinking模式)
P2-T1 (unshare) → P1-T5 (share远程)
P2-T2 (compact) → P1-T4 (预算触发)
P2-T7 (Web UI) → P2-T3 (LSP 1.1)
```

---

## 9. 资源分配建议

| 模块 | 优先级 | 建议资源 |
|------|--------|----------|
| TUI 打字机效果 | P1 | 1人 |
| Context 预算 | P1 | 1人 |
| LSP 1.1 | P2 | 1人 |
| Web UI | P2 | 前端资源 |
| 技术债务 | P2 | 碎片化 |

---

**版本**: 19  
**最后更新**: 2026-04-08  
**维护者**: OpenCode Rust Team