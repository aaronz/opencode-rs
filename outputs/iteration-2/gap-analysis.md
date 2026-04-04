## 分析意图

**任务类型**: 分析调研 — 对比当前实现与PRD的差距，识别缺失和不完整的部分

**我的方法**: 
1. 并行探索实现结构 + 定位PRD.md
2. 读取PRD核心需求
3. 按功能/接口/前端/数据模型/配置/测试 6个维度系统对比
4. 识别通用差距（缺失模块、不完整实现、硬编码、错误处理等）

让我先并行探索实现结构和PRD文件：
Based on my analysis of the PRD and the current implementation, here's the gap analysis:

---

# 差距分析报告

## 一、实现概览

| 模块 | 状态 | 说明 |
|------|------|------|
| core | ✅ 完整 | 包含 session, message, tool, permission 等核心模型 |
| tools | ✅ 完整 | 30+ 工具实现（read, edit, bash, git, lsp 等） |
| llm | ✅ 完整 | 17+ Provider 支持（OpenAI, Anthropic, Ollama 等） |
| agent | ✅ 完整 | 6 种 Agent（build, plan, general, explore, review, refactor, debug） |
| storage | ✅ 完整 | SQLite + 文件系统持久化 |
| server | ⚠️ 基础 | Actix-web 框架，基础路由 |
| tui | ✅ 完整 | ratatui 实现 |
| lsp | ⚠️ 基础 | diagnostics, symbols |
| mcp | ⚠️ 基础 | stdio/remote bridge |
| permission | ✅ 完整 | allow/ask/deny + approval queue |
| auth | ✅ 完整 | JWT, password, OAuth |
| plugin | ⚠️ 未实现 | 计划中 |
| control-plane | ⚠️ ACP协议 | ACP 流式协议实现 |
| git | ⚠️ 基础 | git2 封装 |

---

## 二、差距列表

### P0 - 阻断性问题

| 差距项 | 严重程度 | 模块 | 修复建议 |
|--------|----------|------|----------|
| Context Engine 未实现 | P0 | core | 缺少 token budget 管理、context ranking、compaction |
| Plugin System 未实现 | P0 | plugin | 需实现 WASM 运行时、事件总线 |
| Skills 系统未实现 | P0 | core | 需实现 skill loader、semantic matching |
| Commands 系统未实现 | P0 | core | 需实现 command parser、template expansion |
| MCP 工具接入不完整 | P0 | mcp | 缺少 schema cache、permission 集成 |

### P1 - 核心功能缺失

| 差距项 | 严重程度 | 模块 | 修复建议 |
|--------|----------|------|----------|
| Server API 不完整 | P1 | server | 缺少 Session/Messages/Tool/Artifact API |
| Share 能力未实现 | P1 | core | 需实现 export JSON/Markdown、share server |
| LSP 功能有限 | P1 | lsp | 缺少 definition、references、hover |
| 插件事件总线不完整 | P1 | core | 事件类型覆盖不全 |
| 凭证加密存储缺失 | P1 | auth | auth store 未实现加密 |

### P2 - 完善性问题

| 差距项 | 严重程度 | 模块 | 修复建议 |
|--------|----------|------|----------|
| 配置系统不完整 | P2 | core | 缺少 JSONC loader、env override、schema validation |
| Session summarize 不完整 | P2 | core | 自动 compact 逻辑未实现 |
| Web UI 未实现 | P2 | tui | PRD 中规划的 web shell v1.5 |
| IDE 扩展未实现 | P2 | - | v2 目标 |
| GitHub 集成未实现 | P2 | git | v2 目标 |

---

## 三、功能完整性对比

| PRD 功能 | 实现状态 |
|----------|----------|
| Workspace/Project 机制 | ✅ 已实现 |
| Session 会话系统（create/continue/fork/abort） | ✅ 已实现 |
| Agent 系统（build/plan + 4 种扩展） | ✅ 已实现 |
| 文件工具（read/glob/grep/write/edit/patch/move/delete） | ✅ 已实现 |
| Shell 工具（bash） | ✅ 已实现 |
| Git 工具（status/diff/log/show） | ✅ 已实现 |
| 权限系统（allow/ask/deny + scope） | ✅ 已实现 |
| @file 引用 | ⚠️ 部分实现 |
| !shell 直接执行 | ⚠️ 部分实现 |
| /command 快捷命令 | ❌ 未实现 |
| Skills 延迟加载 | ❌ 未实现 |
| Commands 自定义 | ❌ 未实现 |
| MCP 本地/远程 | ⚠️ 基础实现 |
| LSP diagnostics | ⚠️ 基础实现 |
| Server API (REST + SSE/WS) | ⚠️ 基础实现 |
| Share 分享 | ❌ 未实现 |
| Plugin WASM 宿主 | ❌ 未实现 |
| 凭证加密存储 | ❌ 未实现 |

---

## 四、接口完整性对比

| PRD API 路径 | 实现状态 |
|--------------|----------|
| POST /sessions | ⚠️ 部分 |
| GET /sessions | ⚠️ 部分 |
| GET /sessions/{id} | ⚠️ 部分 |
| POST /sessions/{id}/fork | ❌ 未实现 |
| POST /sessions/{id}/summarize | ⚠️ 部分 |
| POST /sessions/{id}/abort | ✅ 已实现 |
| POST /sessions/{id}/prompt | ⚠️ 部分 |
| GET /sessions/{id}/messages | ⚠️ 部分 |
| POST /sessions/{id}/shell | ✅ 已实现 |
| POST /sessions/{id}/command | ❌ 未实现 |
| POST /sessions/{id}/permissions/{req_id}/reply | ✅ 已实现 |
| GET /sessions/{id}/diff | ⚠️ 部分 |
| GET /sessions/{id}/snapshots | ⚠️ 部分 |
| POST /sessions/{id}/revert | ⚠️ 部分 |
| GET /providers | ⚠️ 部分 |
| GET /models | ⚠️ 部分 |

---

## 五、数据模型对比

| PRD 数据模型 | 实现状态 |
|--------------|----------|
| Session | ✅ 完整 |
| Message | ✅ 完整 |
| ToolCall | ✅ 完整 |
| Snapshot | ✅ 完整 |
| PermissionDecision | ✅ 完整 |
| Provider/Credential | ✅ 完整 |
| Project | ✅ 完整 |
| Checkpoint | ✅ 完整 |

---

## 六、技术债务清单

| 债务项 | 描述 | 影响 |
|--------|------|------|
| 硬编码 provider 默认值 | 缺少配置灵活性 | 中 |
| 错误码不统一 | 未完全按 PRD 细分错误类型 | 低 |
| 缺少 observability | 无结构化 tracing、session traces | 中 |
| Session checkpoint 粒度 | 未实现每次消息后的持久化 | 中 |
| MCP token 成本控制 | 未实现工具元信息延迟注入 | 中 |
| 多层配置合并 | 未完全实现 config merge 逻辑 | 中 |

---

## 七、总结

**完成度：约 60%**

- **已完成**：核心领域模型、工具系统、Agent 系统、权限系统、LLM Provider 抽象、基础 TUI
- **需完成**：Context Engine、Plugin、Skills、Commands、完整 Server API、Share、MCP 增强、LSP 增强

**建议优先级**：
1. 先完成 Context Engine（token budget + compaction）
2. 再补齐 Server API（REST + SSE）
3. 最后实现 Skills/Commands/Plugin 系统
