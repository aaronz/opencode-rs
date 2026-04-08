# OpenCode-RS Task List v22

**版本**: 22  
**日期**: 2026年4月8日  
**基于**: Spec v22 (承接v19未完成任务)  
**总任务数**: 6  
**已完成**: 6  
**进行中**: 0  
**待完成**: 0

---

## 任务统计

| 优先级 | 总数 | 已完成 | 进行中 | 待完成 |
|--------|------|--------|--------|--------|
| P0 | 0 | 0 | 0 | 0 |
| P1 | 0 | 0 | 0 | 0 |
| P2 | 6 | 6 | 0 | 0 |
| Tech Debt | 0 | 0 | 0 | 0 |
| **合计** | **6** | **6** | **0** | **0** |

---

## P2 中优先级任务 (承接v19)

### P2-T1: LSP语言解析后端集成 ✅
- **FR-ID**: FR-183, FR-184, FR-185, FR-186
- **GAP**: GAP-P2-007 (from v19)
- **模块**: LSP
- **问题**: tower_lsp处理器返回None，因无语言解析后端
- **预计工时**: L
- **状态**: ✅ 已完成
- **验收标准**:
  - [x] goto_definition 返回真实代码位置
  - [x] find_references 返回引用列表
  - [x] hover 显示类型/文档信息
  - [x] codeAction 提供可用代码动作
- **关键文件**: `crates/lsp/src/server.rs`
- **子任务**:
  - [x] 实现文件解析和AST构建（简化文本分析）
  - [x] 实现definition查询（find_definition_line）
  - [x] 实现references查询（find_all_references）
  - [x] 实现hover信息查询（classify_word函数）
  - [x] 实现codeAction生成（基于diagnostics）

### P2-T2: MCP工具注册到TUI ✅
- **FR-ID**: FR-193, FR-194
- **GAP**: GAP-P2-008 (from v19)
- **模块**: MCP
- **问题**: McpManager存在但未连接到AgentExecutor
- **预计工时**: M
- **状态**: ✅ 已完成
- **验收标准**:
  - [x] MCP工具可被发现
  - [x] MCP工具可执行
  - [x] Token成本控制
- **关键文件**: `crates/tui/src/app.rs`, `crates/mcp/src/registry.rs`
- **子任务**:
  - [x] 在App结构中添加ToolRegistry字段
  - [x] 在App::new()中初始化ToolRegistry
  - [x] 调用McpManager::bridge_to_tool_registry()
  - [x] 将ToolRegistry传递给AgentExecutor

### P2-T3: TUI插件架构 - AddTools能力 ✅
- **FR-ID**: FR-201
- **GAP**: GAP-P2-009 (from v19)
- **模块**: Plugin
- **问题**: 无插件架构设计
- **预计工时**: L
- **状态**: ✅ 已完成
- **验收标准**:
  - [x] 插件可添加工具到ToolRegistry
  - [x] 插件可被启用/禁用
  - [x] Sidecar模式支持
- **关键文件**: `crates/core/src/plugin.rs`
- **子任务**:
  - [x] 扩展PluginCapability::AddTools（已存在）
  - [x] 实现插件工具注册流程（collect_plugin_tools）
  - [x] 实现Sidecar进程管理（SidecarConfig, start_sidecar, stop_sidecar）
  - [x] 实现插件生命周期管理（enable, disable, is_enabled）

### P2-T4: CLI NDJSON输出格式 ✅
- **FR-ID**: FR-142
- **GAP**: GAP-v19-001
- **模块**: CLI
- **问题**: CLI无流式输出模式
- **预计工时**: M
- **状态**: ✅ 已完成
- **验收标准**:
  - [x] 支持--format ndjson选项
  - [x] 流式输出逐行JSON
  - [x] 事件类型完整
- **关键文件**: `crates/cli/src/cmd/run.rs`
- **子任务**:
  - [x] 添加OutputFormat枚举（已在TUI中存在）
  - [x] 实现NdjsonSerializer（带timestamp的NdjEvent结构）
  - [x] 集成到CLI run命令

### P2-T5: LSP UI集成 ✅
- **FR-ID**: FR-183-FR-186
- **模块**: TUI
- **问题**: LSP功能需要TUI集成
- **预计工时**: M
- **状态**: ✅ 已完成
- **验收标准**:
  - [x] TUI可触发LSP查询
  - [x] 结果显示在UI中
- **子任务**:
  - [x] 在TUI中添加LSP客户端支持（lsp_client字段）
  - [x] 实现结果渲染（lsp_diagnostics集成到right_panel_data）
  - [x] 键盘快捷键绑定（Ctrl+D, Ctrl+R）

### P2-T6: MCP Token成本控制 ✅
- **FR-ID**: FR-194
- **GAP**: GAP-P2-008 (from v19)
- **模块**: MCP
- **问题**: MCP工具调用需要Token成本追踪
- **预计工时**: S
- **状态**: ✅ 已完成
- **验收标准**:
  - [x] MCP工具调用计入Token统计
  - [x] 成本显示在UI中
- **关键文件**: `crates/mcp/src/tool_bridge.rs`
- **子任务**:
  - [x] 在McpToolAdapter中添加成本追踪（token_counter字段, with_token_counter方法）
  - [x] 集成到TokenCounter（通过gpt-4o估算）
  - [x] 在StatusBar显示（mcp_cost_usd字段，显示"LLM + MCP"）

---

## 里程碑追踪

| 里程碑 | 目标日期 | 状态 | 完成任务 |
|--------|----------|------|----------|
| M1: LSP后端 | 2026-04-21 | ✅ | P2-T1 |
| M2: MCP集成 | 2026-04-28 | ✅ | P2-T2, P2-T6 |
| M3: 插件架构 | 2026-05-05 | ✅ | P2-T3 |
| M4: CLI增强 | 2026-05-12 | ✅ | P2-T4 |
| M5: UI集成 | 2026-05-19 | ✅ | P2-T5 |
| M6: v22发布 | 2026-05-26 | 🔲 | 90%+ 完成度 |

---

**版本**: 22  
**最后更新**: 2026-04-08  
**维护者**: OpenCode Rust Team
