Now I'll explore the implementation details to compare with PRD requirements.
Based on my context gathering, I can now produce the gap analysis. Let me synthesize the findings:

# 差距分析报告

## 实现概况

### 当前代码结构 (rust-opencode-port/)

| 模块 | 状态 | 文件数 |
|------|------|--------|
| core | ✅ 已实现 | 核心域模型 |
| llm | ✅ 已实现 | LLM Provider 抽象 |
| tools | ✅ 已实现 | 内置工具 |
| agent | ✅ 已实现 | Build/Plan/General/Review/Refactor/Debug Agent |
| storage | ✅ 已实现 | SQLite 持久化 |
| permission | ✅ 已实现 | 权限引擎 |
| lsp | ✅ 已实现 | LSP 诊断/符号/定义/引用 |
| server | ✅ 已实现 | REST API |
| tui | ✅ 已实现 | Ratatui 终端 |
| mcp | ✅ 已实现 | MCP 桥接 |
| git | ✅ 已实现 | Git 集成 |
| auth | ✅ 已实现 | 认证层 |
| plugin | ✅ 已实现 | 插件系统 |
| control-plane | ✅ 已实现 | 控制平面流 |

---

## 差距列表

### 功能完整性差距

| 差距项 | 严重程度 | 模块 | 修复建议 |
|--------|----------|------|----------|
| Commands 系统 - 宏命令定义与执行 | P1 | TUI/CLI | 实现 `.opencode/commands/*.md` 加载，支持 `${file}` 等变量替换 |
| Skills 系统 - 延迟加载与语义匹配 | P2 | Agent | 实现 `.opencode/skills/<name>/SKILL.md` 按需发现与语义匹配 |
| Context Engine - Token Budget 压缩 | P1 | Core | 实现 85% 预警 / 92% compact / 95% 强制新 session 机制 |
| Share 功能 - 导出与短链 | P2 | Server/Storage | 实现 session JSON/Markdown 导出，self-hosted share 服务层 |
| GitHub Integration - Issue/PR Trigger | P2 | CLI/Server | v2 功能，非 v1 目标 |

### 接口完整性差距

| 差距项 | 严重程度 | 模块 | 修复建议 |
|--------|----------|------|----------|
| REST API - `/providers/{id}/credentials` | P1 | Server | 添加凭证设置/测试/撤销 API |
| REST API - `/sessions/{id}/fork` | P1 | Server | 实现 session 分叉端点 |
| REST API - `/sessions/{id}/summarize` | P1 | Server | 实现自动摘要端点 |
| SSE/WebSocket 双协议支持 | P1 | Server | 当前实现需验证是否完整支持两种流式协议 |
| Provider 管理 API 完善 | P1 | Server | 补充连通性测试、credential 过期状态 |

### 前端完整性差距

| 差距项 | 严重程度 | 模块 | 修复建议 |
|--------|----------|------|----------|
| TUI 三栏/双栏切换 | P2 | TUI | 实现可切换布局 |
| TUI 右栏功能 - diagnostics/todo/权限队列 | P2 | TUI | 完善右栏面板 |
| TUI 快捷输入 - `@file` `/command` `!shell` | P1 | TUI | 完整实现三种快捷输入解析器 |
| TUI Token/Cost 显示 | P2 | TUI | 实现 token 统计与成本显示 |
| TUI Patch 预览展开 | P2 | TUI | 实现 diff 展开/收起交互 |
| Web UI | P2 | Server/TUI | v0.3 目标，非 v1 强制 |

### 数据模型差距

| 差距项 | 严重程度 | 模块 | 修复建议 |
|--------|----------|------|----------|
| Session Fork Lineage | P1 | Storage | 添加 `parent_session_id` 字段与 fork 逻辑 |
| Session Share 状态 | P2 | Storage | 添加 `shared_id` 字段与分享逻辑 |
| PermissionDecision 审计模型 | P2 | Storage | 完善权限决策审计表 |
| Snapshot 元数据 | P1 | Storage | 完善 snapshots 表关联 |
| Auth Store - 凭证加密存储 | P2 | Auth | 实现系统密钥链集成与加密 |
| OAuth Session 存储 | P2 | Auth | v1.5 目标，预留扩展字段 |

### 配置管理差距

| 差距项 | 严重程度 | 模块 | 修复建议 |
|--------|----------|------|----------|
| 多层配置合并 (global/project/env/CLI) | P1 | Core | 实现配置优先级合并逻辑 |
| JSONC 解析器 | P2 | Core | 实现 JSONC 注释支持 |
| Provider Credential 引用机制 | P1 | Config | 实现 `credentialRef` 而非明文密钥 |
| 环境变量约定 - Provider-specific | P1 | Config | 实现 `OPENAI_API_KEY` 等自动绑定 |

### 测试覆盖差距

| 差距项 | 严重程度 | 模块 | 修复建议 |
|--------|----------|------|----------|
| Skill 加载与匹配 E2E 测试 | P2 | Agent | 补充 skills 集成测试 |
| Command 模板展开测试 | P2 | CLI | 补充命令解析测试 |
| MCP 工具发现与调用测试 | P1 | MCP | 补充 MCP 完整流程测试 |
| Session 并发与分叉测试 | P1 | Storage | 补充 session 并发测试 |
| 权限 scope 记忆测试 | P2 | Permission | 补充权限范围记忆测试 |

---

## P0/P1/P2 问题分类

### P0 - 阻塞性问题 (必须修复)

| # | 问题 | 影响 |
|---|------|------|
| P0-1 | Commands 系统未实现 | 用户无法使用自定义宏命令 |
| P0-2 | `@file` / `!shell` / `/command` 快捷输入未完整实现 | TUI 核心交互缺失 |
| P0-3 | Session Fork 未实现 | 无法分叉会话 |
| P0-4 | 多层配置合并未实现 | 配置优先级混乱 |
| P0-5 | Context Token Budget 压缩未实现 | 长会话上下文溢出 |

### P1 - 重要问题 (应该修复)

| # | 问题 | 影响 |
|---|------|------|
| P1-1 | Skills 延迟加载未实现 | 无法按需加载领域知识包 |
| P1-2 | Permission 审计记录不完整 | 权限决策无法追溯 |
| P1-3 | Share 导出功能缺失 | 无法分享会话 |
| P1-4 | Provider API 凭证管理不完整 | 无法测试/撤销凭证 |
| P1-5 | Token/Cost 统计未显示 | 用户无法感知成本 |

### P2 - 改进问题 (可以后续修复)

| # | 问题 | 影响 |
|---|------|------|
| P2-1 | TUI 三栏布局未实现 | UI 布局受限 |
| P2-2 | GitHub Integration 未实现 | v2 目标 |
| P2-3 | Web UI 未实现 | v0.3 目标 |
| P2-4 | OAuth 登录预留 | v1.5 目标 |
| P2-5 | Auth Store 加密未实现 | 安全增强项 |

---

## 技术债务清单

| 债务项 | 描述 | 优先级 |
|--------|------|--------|
| TD-1 | 硬编码魔法数字 (如 token 阈值) | P1 |
| TD-2 | 错误码与人类可读文案未完整映射 | P1 |
| TD-3 | Provider-specific header 注入逻辑散落 | P2 |
| TD-4 | 插件事件总线未完整测试 | P2 |
| TD-5 | LSP 增量刷新性能优化 | P2 |
| TD-6 | MCP 工具 schema 缓存机制 | P2 |

---

## 总结

**实现完成度: ~75%**

- **已实现**: Core/LLM/Tools/Agent/Storage/Permission/LSP/Server/TUI/MCP/Git/Auth/Plugin/ControlPlane
- **核心缺失 (P0)**: Commands 系统、快捷输入解析、Session Fork、多层配置、Token Budget 压缩
- **次要缺失 (P1)**: Skills 系统、Share 功能、Provider API 完善、Token 统计
- **未来目标 (P2)**: GitHub Integration、Web UI、OAuth

**建议优先级**:
1. 修复 P0 问题 (Commands + 快捷输入 + Session Fork + 配置 + Context)
2. 完善 P1 问题 (Skills + Share + Provider API)
3. 跟进 P2 问题 (TUI 增强 + 测试覆盖)
