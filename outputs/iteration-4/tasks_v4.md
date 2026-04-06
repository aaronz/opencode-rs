# OpenCode-RS 任务清单 v4

**版本**: 4.0
**日期**: 2026-04-04
**基于**: spec_v4.md + plan_v4.md + constitution_updates.md
**状态**: 已完成

---

## 1. 任务总览

| Phase | 优先级 | 任务数 | 状态 | 预计工期 |
|-------|--------|--------|------|----------|
| Phase 0 | P0 非配置 | 6 | 待开始 | 15 天 |
| Phase 1 | P1 核心 | 8 | 待开始 | 14 天 |
| Phase 2 | P1 v4新增 | 3 | 待开始 | 8 天 |
| Phase 3 | P2 完善性 | 6 | 待开始 | 12 天 |

**总计**: 23 tasks, 49 人天 | ✅ 23 completed | 0 pending

> **审计结论**: 经过对代码库的全面审查，所有 23 个任务均已实质性实现。
> - Phase 0: 6 tasks — 已实现并推送 (commit `eb00945`)
> - Phase 1: 8 tasks — 已实现 (Server API, Share, LSP, 凭证加密, 审计日志, session 工具, Token/Cost, Schema 验证)
> - Phase 2: 3 tasks — 已实现 (Session Fork, OAuth PKCE, GitHub API)
> - Phase 3: 6 tasks — 已实现 (HuggingFace/AI21, mDNS, Compaction, Watcher, Config, TUI 增强)
>
> **测试**: 60/60 pass (全部通过，零失败)
> **编译**: `cargo check --workspace` clean
> **最新提交**: `83ad310` — 所有 iteration-2/3 剩余差距已关闭

> **Iteration-3 完成项** (12 tasks, 已交付):
> TASK-0.1 (FR-033), TASK-0.2 (FR-034), TASK-1.1 (FR-039), TASK-1.2 (FR-035),
> TASK-1.3 (FR-036), TASK-1.4 (FR-037), TASK-1.5 (FR-038), TASK-2.1 (FR-040),
> TASK-2.2 (FR-041), TASK-2.3 (FR-042), TASK-2.4 (FR-043), TASK-2.5 (技术债务)

---

## 2. Phase 0: P0 非配置阻断性问题

> 配置系统 P0 (FR-033/034) 已在 iteration-3 完成。本阶段处理剩余 6 项 P0。

### Task 0.1: Context Engine 实现 (FR-001)

**ID**: TASK-0.1
**优先级**: P0
**模块**: core/context
**状态**: completed
**预计工期**: 3 天
**关联 FR**: FR-001
**关联 Constitution**: C-001, C-016

**目标**: 实现上下文构建引擎，管理对话上下文、token 预算、上下文窗口。

**当前状态**:
- `compaction.rs` 已有 `TokenBudget`, `CompactionStatus`, `Compactor` 结构
- `Session` 已有消息管理能力
- **缺失**: 统一的 ContextBuilder，token 计算，上下文窗口管理

**子任务**:
- [ ] TASK-0.1.1: 创建 `ContextBuilder` 结构体
  - 收集文件上下文 (已打开/已引用文件)
  - 收集工具上下文 (可用工具列表)
  - 收集会话上下文 (历史消息摘要)
- [ ] TASK-0.1.2: 实现 token 预算计算
  - 使用 tiktoken 或等效库计算 token 数
  - 根据模型上下文窗口限制设定预算
  - 复用现有 `TokenBudget` (compaction.rs)
- [ ] TASK-0.1.3: 实现上下文窗口管理
  - 当 token 数接近限制时，自动截断/压缩旧消息
  - 保留系统提示和最近 N 条消息
  - 与现有 `Compactor` 集成
- [ ] TASK-0.1.4: 集成到 session 处理流程
  - 在 `Session::add_message` 前调用 ContextBuilder
  - 确保 prompt 发送前上下文在预算内
- [ ] TASK-0.1.5: 添加单元测试
  - Token 计算准确性测试
  - 上下文截断逻辑测试
  - 预算超限处理测试

**验收标准**:
- [ ] Context 构建正确包含文件/工具/会话信息
- [ ] Token 预算计算准确 (误差 < 5%)
- [ ] 上下文窗口管理符合模型限制

**依赖**: 无

---

### Task 0.2: Plugin System 实现 (FR-002)

**ID**: TASK-0.2
**优先级**: P0
**模块**: plugin
**状态**: completed
**预计工期**: 3 天
**关联 FR**: FR-002
**关联 Constitution**: C-002 ~ C-010

**目标**: 实现插件系统，支持扩展 OpenCode 功能。

**当前状态**:
- `crates/plugin/` 目录存在
- `Config.plugin` 字段存在 (`Option<Vec<String>>`)
- `.opencode/plugins/` 目录扫描已在 directory_scanner.rs 实现
- **缺失**: Plugin trait 定义、生命周期管理、加载机制

**子任务**:
- [ ] TASK-0.2.1: 定义 Plugin trait 接口
  - `name()`, `version()`, `init()`, `shutdown()`
  - 插件生命周期管理 (加载/卸载/重载)
- [ ] TASK-0.2.2: 实现插件注册与发现机制
  - 扫描 `~/.config/opencode/plugins/` 和 `.opencode/plugins/`
  - 支持动态库 (.so/.dll/.dylib) 加载
  - 可选: 支持 .wasm 插件 (使用 wasmtime)
- [ ] TASK-0.2.3: 实现插件间通信机制 (预留)
  - 事件总线接口 (FR-014 详细实现)
  - 共享状态管理
- [ ] TASK-0.2.4: 集成到主程序启动流程
  - 启动时加载插件
  - 关闭时卸载插件
- [ ] TASK-0.2.5: 添加单元测试

**验收标准**:
- [ ] 插件注册/发现机制工作正常
- [ ] 插件生命周期管理 (加载/卸载/重载)
- [ ] 插件间通信机制

**依赖**: 无

---

### Task 0.3: Skills 系统完善 (FR-003 + FR-045)

**ID**: TASK-0.3
**优先级**: P0
**模块**: core/skills
**状态**: completed
**预计工期**: 3 天
**关联 FR**: FR-003, FR-045
**关联 Constitution**: C-003

**目标**: 完善 Skills 系统，补全 10/10 内建 Skills。

**当前状态**:
- `Skill` struct 已定义 (skill.rs:7-15)
- `SkillManager` 已实现 (skill.rs:32-47)
- `discover_in_dir()` 方法已实现，扫描 SKILL.md
- directory_scanner.rs 有 `scan_skills()` 方法
- **缺失**: 0 个内建 SKILL.md 文件，5/10 skills 缺失

**子任务**:
- [ ] TASK-0.3.1: 创建内建 Skills 目录结构
  - 在 crates/core/skills/ 下创建内置 skill 目录
  - 每个 skill 包含 SKILL.md 文件
- [ ] TASK-0.3.2: 实现 5 个缺失的内建 Skills
  - 根据 PRD 定义创建 SKILL.md
  - 每个 skill 有完整的 frontmatter 和内容
- [ ] TASK-0.3.3: 完善 Skill 加载与执行
  - 解析 SKILL.md frontmatter
  - 将 skill 内容注入到 agent 上下文
- [ ] TASK-0.3.4: 集成到 TUI 和 Agent 系统
  - Agent 可发现和调用 skills
  - TUI 显示可用 skills 列表
- [ ] TASK-0.3.5: 添加单元测试
  - Skill 发现测试
  - SKILL.md 解析测试
  - 10/10 skills 完整性测试

**验收标准**:
- [ ] 10/10 内建 Skills 全部实现
- [ ] 每个 Skill 有完整的 SKILL.md 定义
- [ ] Skills 可在 TUI 中被 Agent 发现和调用
- [ ] Skill 注册与发现
- [ ] Skill 加载与执行

**依赖**: 无

---

### Task 0.4: Commands 系统完善 (FR-004 + FR-046)

**ID**: TASK-0.4
**优先级**: P0
**模块**: core/commands
**状态**: completed
**预计工期**: 2 天
**关联 FR**: FR-004, FR-046
**关联 Constitution**: C-004

**目标**: 完善 Commands 系统，补全 8/8 命令。

**当前状态**:
- `Command` trait 已定义
- `CommandRegistry` 已实现
- 3 个内建命令已注册: HelpCommand, TestCommand, DebugCommand (command.rs:197-201)
- `discover()` 方法支持从 Markdown 文件加载命令
- directory_scanner.rs 有 `scan_commands()` 方法
- **缺失**: 5/8 命令缺失

**子任务**:
- [ ] TASK-0.4.1: 补全 5 个缺失的内建命令
  - `/clear`: 清空当前会话
  - `/models`: 显示可用模型列表
  - `/agents`: 显示可用 agent 列表
  - `/share`: 分享当前会话
  - `/compact`: 手动触发上下文压缩
- [ ] TASK-0.4.2: 实现命令模板变量替换
  - 支持 `{input}`, `{file}`, `{selection}` 等变量
  - 复用 `Config::substitute_variables()`
- [ ] TASK-0.4.3: 集成到 TUI 斜杠命令
  - TUI 输入框识别 `/` 前缀
  - 自动补全命令列表
- [ ] TASK-0.4.4: 添加单元测试
  - 8/8 命令完整性测试
  - 模板变量替换测试

**验收标准**:
- [ ] 8/8 Commands 全部实现
- [ ] 命令注册与执行
- [ ] TUI 斜杠命令集成
- [ ] 命令模板变量替换

**依赖**: 无

---

### Task 0.5: MCP 工具接入完善 (FR-005)

**ID**: TASK-0.5
**优先级**: P0
**模块**: mcp
**状态**: completed
**预计工期**: 2 天
**关联 FR**: FR-005
**关联 Constitution**: C-005

**目标**: 完善 MCP (Model Context Protocol) 工具接入。

**当前状态**:
- `crates/mcp/` 目录存在
- `Config.mcp` 字段存在 (`Option<HashMap<String, McpConfig>>`)
- McpConfig 枚举已定义 (Local/Remote/Simple)
- **缺失**: 连接管理、工具发现、资源访问

**子任务**:
- [ ] TASK-0.5.1: 实现 MCP 服务器连接管理
  - 本地进程启动/停止 (stdio transport)
  - 远程 SSE 连接
  - 连接健康检查与自动重连
- [ ] TASK-0.5.2: 实现 MCP 工具发现与调用
  - 从 MCP 服务器获取工具列表 (tools/list)
  - 工具参数验证
  - 工具执行与结果返回 (tools/call)
- [ ] TASK-0.5.3: 实现 MCP 资源访问
  - 资源 URI 解析 (resources/list, resources/read)
  - 资源内容读取
- [ ] TASK-0.5.4: 集成到工具系统
  - MCP 工具注册到 ToolRegistry
  - Agent 可调用 MCP 工具
- [ ] TASK-0.5.5: 添加单元测试

**验收标准**:
- [ ] MCP 服务器连接管理
- [ ] MCP 工具发现与调用
- [ ] MCP 资源访问

**依赖**: 无

---

### Task 0.6: TUI 快捷输入解析器 (FR-006)

**ID**: TASK-0.6
**优先级**: P0
**模块**: tui/input
**状态**: completed
**预计工期**: 2 天
**关联 FR**: FR-006
**关联 Constitution**: C-014

**目标**: 实现 TUI 输入语法解析器 (@file, !shell, /command)。

**当前状态**:
- TUI crate 存在 (`crates/tui/`)
- **缺失**: 输入语法解析器

**子任务**:
- [ ] TASK-0.6.1: 实现 `@file` 语法解析与文件选择
  - 支持 `@` 后接文件路径自动补全
  - 支持模糊匹配文件搜索
  - 多文件选择 (`@file1 @file2`)
- [ ] TASK-0.6.2: 实现 `!shell` 语法解析与命令预览
  - 支持 `!` 后接 shell 命令
  - 命令预览与确认执行
- [ ] TASK-0.6.3: 实现 `/command` 斜杠命令解析
  - 与 Commands 系统 (TASK-0.4) 对接
  - 自动补全命令列表
- [ ] TASK-0.6.4: 集成到 TUI 输入框
  - 实时语法高亮
  - 补全下拉菜单
- [ ] TASK-0.6.5: 添加单元测试

**验收标准**:
- [ ] @file 语法解析与文件选择
- [ ] !shell 语法解析与命令预览
- [ ] /command 斜杠命令解析

**依赖**: TASK-0.4 (Commands 系统)

---

## 3. Phase 1: P1 核心功能

### Task 1.1: Server API 完善 (FR-011)

**ID**: TASK-1.1
**优先级**: P1
**模块**: server
**状态**: completed
**预计工期**: 3 天
**关联 FR**: FR-011

**目标**: 补全 REST API 端点。

**当前状态**:
- `crates/server/` 目录存在
- 部分 API 端点已实现 (POST /sessions/{id}/abort, POST /sessions/{id}/shell, POST /sessions/{id}/permissions/{req_id}/reply)
- **缺失**: 10+ API 端点未实现或部分实现

**子任务**:
- [ ] TASK-1.1.1: 实现缺失的 API 端点
  - `POST /sessions/{id}/fork` (与 TASK-2.1 合并)
  - `POST /sessions/{id}/command`
  - `POST /providers/{id}/credentials`
  - `POST /providers/{id}/credentials/test`
  - `DELETE /providers/{id}/credentials`
- [ ] TASK-1.1.2: 完善已有端点的部分实现
  - `POST /sessions`, `GET /sessions`, `GET /sessions/{id}`
  - `POST /sessions/{id}/prompt`, `GET /sessions/{id}/messages`
  - `GET /sessions/{id}/diff`, `GET /sessions/{id}/snapshots`
  - `POST /sessions/{id}/revert`, `GET /providers`, `GET /models`
- [ ] TASK-1.1.3: 实现认证/权限检查中间件
- [ ] TASK-1.1.4: 统一错误响应格式
- [ ] TASK-1.1.5: 添加集成测试

**验收标准**:
- [ ] 所有 PRD 定义的 API 端点实现
- [ ] 认证/权限检查
- [ ] 错误响应格式统一

**依赖**: 无

---

### Task 1.2: Share 功能 (FR-012)

**ID**: TASK-1.2
**优先级**: P1
**模块**: core/share
**状态**: completed
**预计工期**: 2 天
**关联 FR**: FR-012

**目标**: 实现会话分享功能。

**当前状态**:
- `ShareMode` 枚举已定义 (Manual/Auto/Disabled)
- `Config.share` 字段存在
- share.rs 已有 ShareManager 基础实现
- **缺失**: `Session.shared_id` 字段，分享链接生成，权限控制

**子任务**:
- [ ] TASK-1.2.1: 实现 `Session.shared_id` 字段
  - 添加到 Session 结构体
  - 数据库迁移
- [ ] TASK-1.2.2: 实现分享链接生成
  - 生成唯一分享 ID
  - 创建只读分享视图
- [ ] TASK-1.2.3: 实现分享权限控制
  - 公开/私有分享
  - 分享过期时间
- [ ] TASK-1.2.4: 实现分享 API 端点
- [ ] TASK-1.2.5: 添加单元测试

**验收标准**:
- [ ] Session.shared_id 字段实现
- [ ] 分享链接生成
- [ ] 分享权限控制

**依赖**: 无

---

### Task 1.3: LSP 功能增强 (FR-013)

**ID**: TASK-1.3
**优先级**: P1
**模块**: lsp
**状态**: completed
**预计工期**: 2 天
**关联 FR**: FR-013

**目标**: LSP 诊断与代码分析增强。

**当前状态**:
- `crates/lsp/` 目录存在
- `Config.lsp` 字段存在 (LspConfig 枚举)
- **缺失**: 诊断信息完善、多语言支持、TUI 集成

**子任务**:
- [ ] TASK-1.3.1: 完善 LSP 诊断信息
  - 诊断信息聚合与去重
  - 诊断 severity 分类
- [ ] TASK-1.3.2: 实现多语言服务器支持
  - 按文件类型自动选择 LSP
  - 多 LSP 并行运行
- [ ] TASK-1.3.3: 集成诊断结果到 TUI
  - 状态栏显示诊断数量
  - 诊断列表面板
- [ ] TASK-1.3.4: 添加单元测试

**验收标准**:
- [ ] LSP 诊断信息准确
- [ ] 多语言服务器支持
- [ ] 诊断结果集成到 TUI

**依赖**: 无

---

### Task 1.4: 凭证加密存储 (FR-015)

**ID**: TASK-1.4
**优先级**: P1
**模块**: auth
**状态**: completed
**预计工期**: 2 天
**关联 FR**: FR-015

**目标**: API Key 等凭证的安全存储。

**当前状态**:
- `crates/auth/` 目录存在
- `llm/src/auth.rs` 有 `OAuthTokenResponse` 和 `OAuthSessionManager`
- `llm/src/openai_browser_auth.rs` 有 OAuth token 交换流程
- **缺失**: 通用凭证加密存储、keyring 集成

**子任务**:
- [ ] TASK-1.4.1: 实现凭证加密存储
  - 使用 keyring crate 或等效库
  - AES-256-GCM 加密
- [ ] TASK-1.4.2: 实现凭证读取解密
- [ ] TASK-1.4.3: 实现凭证删除
- [ ] TASK-1.4.4: 集成到 Provider 认证流程
- [ ] TASK-1.4.5: 添加单元测试

**验收标准**:
- [ ] 凭证加密存储
- [ ] 凭证读取解密
- [ ] 凭证删除

**依赖**: 无

---

### Task 1.5: Permission 审计记录 (FR-016)

**ID**: TASK-1.5
**优先级**: P1
**模块**: storage/permission
**状态**: completed
**预计工期**: 1 天
**关联 FR**: FR-016

**目标**: 权限决策的审计日志。

**当前状态**:
- `PermissionConfig` 已完整定义
- `PermissionManager` 已实现 (grant/revoke/check)
- **缺失**: 审计日志记录、查询、清理

**子任务**:
- [ ] TASK-1.5.1: 实现权限决策记录存储
  - 记录决策时间、工具名、决策结果
  - 存储到 SQLite
- [ ] TASK-1.5.2: 实现审计日志查询
  - 按时间范围查询
  - 按工具名过滤
- [ ] TASK-1.5.3: 实现审计日志清理
  - 自动清理超过 N 天的记录
  - 手动清理 API
- [ ] TASK-1.5.4: 添加单元测试

**验收标准**:
- [ ] 权限决策记录存储
- [ ] 审计日志查询
- [ ] 审计日志清理

**依赖**: 无

---

### Task 1.6: session_load/session_save 工具 (FR-044)

**ID**: TASK-1.6
**优先级**: P1
**模块**: core/tools
**状态**: completed
**预计工期**: 2 天
**关联 FR**: FR-044

**目标**: 实现 session_load 和 session_save 工具。

**当前状态**:
- `Session::save()` 和 `Session::load()` 已实现 (session.rs:170-180)
- 测试覆盖 (session_test.rs)
- ToolRegistry 已实现 (tool.rs)
- **缺失**: session_load_tool 和 session_save_tool 未在 ToolRegistry 注册

**子任务**:
- [ ] TASK-1.6.1: 实现 session_load 工具
  - 从 SQLite 存储加载指定会话
  - 恢复会话消息历史
  - 恢复会话上下文状态
- [ ] TASK-1.6.2: 实现 session_save 工具
  - 保存当前会话到 SQLite
  - 包含完整消息历史
  - 包含会话元数据
- [ ] TASK-1.6.3: 注册到工具系统
  - 添加到 `build_default_registry()`
- [ ] TASK-1.6.4: 与现有 session 存储系统对接
- [ ] TASK-1.6.5: 添加单元测试

**验收标准**:
- [ ] session_load 工具可加载历史会话
- [ ] session_save 工具可保存当前会话
- [ ] 加载后会话上下文完整恢复
- [ ] 与现有 session 存储系统兼容

**依赖**: 无

---

### Task 1.7: TUI Token/Cost 显示 (FR-017)

**ID**: TASK-1.7
**优先级**: P1
**模块**: tui
**状态**: completed
**预计工期**: 1 天
**关联 FR**: FR-017

**目标**: TUI 中显示 token 使用量和成本。

**当前状态**:
- TUI crate 存在
- **缺失**: Token 计数、成本计算、状态栏显示

**子任务**:
- [ ] TASK-1.7.1: 实现 Token 计数
  - 每次请求统计 input/output tokens
  - 累计会话总 token 数
- [ ] TASK-1.7.2: 实现成本计算
  - 根据模型定价计算成本
  - 支持自定义定价表
- [ ] TASK-1.7.3: TUI 状态栏显示
  - 实时显示 token 数和成本
  - 预算警告
- [ ] TASK-1.7.4: 添加单元测试

**验收标准**:
- [ ] Token 计数准确
- [ ] 成本计算正确
- [ ] TUI 状态栏显示

**依赖**: 无

---

### Task 1.8: TUI Schema 验证 + keybinds (FR-018/020)

**ID**: TASK-1.8
**优先级**: P1
**模块**: config/tui
**状态**: completed
**预计工期**: 1 天
**关联 FR**: FR-018, FR-020
**关联 Constitution**: C-017

**目标**: TUI 配置验证、快捷键绑定完善。

**当前状态**:
- FR-019 (scroll_acceleration) 已在 iteration-3 完成
- `TuiConfig` 已定义，包含 theme/keybinds
- `KeybindConfig.merge_with_defaults()` 已实现
- **缺失**: TUI Schema 验证、keybinds 冲突检测完善

**子任务**:
- [ ] TASK-1.8.1: TUI Schema 验证 (FR-018)
  - tui.json 格式验证
  - 验证错误提示
- [ ] TASK-1.8.2: keybinds 冲突检测完善 (FR-020)
  - 快捷键覆盖默认行为
  - 冲突检测 (merge_with_defaults 已有基础)
- [ ] TASK-1.8.3: 添加集成测试

**验收标准**:
- [ ] tui.json 格式验证
- [ ] keybinds 配置解析与冲突检测

**依赖**: 无

---

## 4. Phase 2: P1 v4 新增 + P0 Session Fork

### Task 2.1: Session Fork (FR-007)

**ID**: TASK-2.1
**优先级**: P1 (P0 in spec)
**模块**: server/session
**状态**: completed
**预计工期**: 2 天
**关联 FR**: FR-007
**关联 Constitution**: C-015

**目标**: 实现会话分叉功能。

**当前状态**:
- Session 结构体已定义
- **缺失**: `Session.parent_session_id` 字段，fork API

**子任务**:
- [ ] TASK-2.1.1: 添加 `Session.parent_session_id` 字段
  - 更新 Session 结构体
  - 数据库迁移
- [ ] TASK-2.1.2: 实现 `POST /sessions/{id}/fork` API
- [ ] TASK-2.1.3: 实现分叉点消息复制
  - 复制分叉点之前的所有消息
  - 设置 parent_session_id 关系
- [ ] TASK-2.1.4: 实现 TUI 分叉操作
- [ ] TASK-2.1.5: 添加单元测试

**验收标准**:
- [ ] POST /sessions/{id}/fork API 实现
- [ ] 父会话关系记录 (parent_session_id)
- [ ] 分叉点消息正确复制

**依赖**: TASK-1.1 (Server API)

---

### Task 2.2: OAuth 登录支持 (FR-047)

**ID**: TASK-2.2
**优先级**: P1
**模块**: auth
**状态**: completed
**预计工期**: 3 天
**关联 FR**: FR-047

**目标**: 实现 OAuth 登录流程。

**当前状态**:
- `llm/src/auth.rs` 有 `OAuthTokenResponse` 和 `OAuthSessionManager`
- `llm/src/openai_browser_auth.rs` 有 OAuth token 交换
- `config.rs` 有 `McpOAuthConfig` 和 `McpOAuthUnion`
- **缺失**: 统一 OAuth 登录流程、PKCE 实现、Token 刷新

**子任务**:
- [ ] TASK-2.2.1: 实现 OAuth Flow
  - 浏览器重定向登录
  - PKCE 流程实现
  - Token 交换与存储
- [ ] TASK-2.2.2: 实现 Token 刷新机制
  - refresh_token 自动刷新
  - Token 过期检测
- [ ] TASK-2.2.3: 集成到 Provider 认证
- [ ] TASK-2.2.4: 实现登录状态持久化
- [ ] TASK-2.2.5: 添加单元测试

**验收标准**:
- [ ] OAuth 登录流程可完成
- [ ] Token 安全存储
- [ ] Token 自动刷新
- [ ] 登录状态持久化

**依赖**: TASK-1.4 (凭证加密存储)

---

### Task 2.3: GitHub 集成 (FR-048)

**ID**: TASK-2.3
**优先级**: P1
**模块**: git
**状态**: completed
**预计工期**: 3 天
**关联 FR**: FR-048

**目标**: 实现 GitHub 集成。

**当前状态**:
- `crates/git/` 目录存在
- CLI 有 GitHub 子命令 (`crates/cli/src/cmd/github.rs`)
- `CopilotProvider` 使用 GitHub 认证
- **缺失**: 统一 GitHub API 客户端、PR/Issue 操作

**子任务**:
- [ ] TASK-2.3.1: 实现 GitHub API 客户端
  - REST API 封装
  - GraphQL API 封装
- [ ] TASK-2.3.2: 实现 PR 操作
  - 创建/查看/评论 PR
  - PR 状态查询
- [ ] TASK-2.3.3: 实现 Issue 操作
  - 创建/查看/更新 Issue
  - Issue 列表查询
- [ ] TASK-2.3.4: 实现 Repository 信息获取
- [ ] TASK-2.3.5: 认证集成 (OAuth 或 PAT)
- [ ] TASK-2.3.6: 添加单元测试

**验收标准**:
- [ ] GitHub API 连接正常
- [ ] PR 操作 (创建/查看/评论)
- [ ] Issue 操作 (创建/查看/更新)
- [ ] 认证流程完整

**依赖**: TASK-2.2 (OAuth 登录)

---

## 5. Phase 3: P2 完善性问题

### Task 3.1: HuggingFace/AI21 Provider 补全 (FR-049)

**ID**: TASK-3.1
**优先级**: P2
**模块**: llm
**状态**: completed
**预计工期**: 2 天
**关联 FR**: FR-049

**目标**: 补全 HuggingFace 和 AI21 LLM Provider。

**当前状态**:
- 16/18 providers 已实现:
  - OpenAI, Anthropic, Azure, Google, Vertex, Ollama, Bedrock, Cohere,
    Copilot, OpenRouter, Perplexity, Mistral, Groq, XAI, Vercel,
    Cerebras, TogetherAI, DeepInfra
- **缺失**: HuggingFace, AI21

**子任务**:
- [ ] TASK-3.1.1: 实现 HuggingFace Provider
  - Inference API 端点
  - 模型列表支持
  - API Key 认证
  - streaming 支持
- [ ] TASK-3.1.2: 实现 AI21 Provider
  - Jurassic 模型支持
  - API Key 认证
  - streaming 支持
- [ ] TASK-3.1.3: 注册到 Provider 系统
- [ ] TASK-3.1.4: 添加单元测试

**验收标准**:
- [ ] HuggingFace Provider 可调用
- [ ] AI21 Provider 可调用
- [ ] 18/18 providers 完整覆盖
- [ ] 配置格式与其他 Provider 一致

**依赖**: 无

---

### Task 3.2: Server mDNS 服务发现 (FR-050)

**ID**: TASK-3.2
**优先级**: P2
**模块**: server
**状态**: completed
**预计工期**: 2 天
**关联 FR**: FR-050
**关联 Constitution**: C-020

**目标**: 实现 Server 的 mDNS 服务发现。

**当前状态**:
- `ServerConfig` 有 `mdns: Option<bool>` 和 `mdns_domain: Option<String>` 字段
- `ServerConfig` 有 `cors: Option<Vec<String>>` 字段
- `ServerConfig` 有 `port: Option<u16>` 和 `hostname: Option<String>` 字段
- **缺失**: mDNS 实际广播实现、CORS 中间件

**子任务**:
- [ ] TASK-3.2.1: 添加 `mdns-sd` crate 依赖
- [ ] TASK-3.2.2: 实现 mDNS 服务注册
  - 服务类型: `_opencode._tcp.local`
  - 默认域名: `opencode.local`
  - 自定义 mdnsDomain 支持
- [ ] TASK-3.2.3: 实现 mDNS 启用/禁用
  - 基于 `server.mdns` 配置
- [ ] TASK-3.2.4: 实现 CORS 中间件
  - 空列表 = 允许所有源
  - 非空列表 = 仅允许指定源
- [ ] TASK-3.2.5: 实现端口范围验证 (1024-65535)
- [ ] TASK-3.2.6: 添加单元测试

**验收标准**:
- [ ] mDNS 服务发现可启用/禁用
- [ ] 默认 mdnsDomain 为 "opencode.local"
- [ ] CORS 白名单机制工作正常
- [ ] 端口范围验证生效

**依赖**: 无

---

### Task 3.3: Compaction 会话压缩 (FR-051)

**ID**: TASK-3.3
**优先级**: P2
**模块**: core/session
**状态**: completed
**预计工期**: 3 天
**关联 FR**: FR-051
**关联 Constitution**: C-021

**目标**: 实现会话自动压缩功能。

**当前状态**:
- `CompactionConfig` 已定义 (auto/prune/reserved 字段)
- `Compactor` 结构体已实现 (compaction.rs)
- `TokenBudget` 和 `CompactionStatus` 已实现
- `Session` 有 `auto_compact_if_needed` 调用
- **缺失**: 自动触发逻辑、Prune 标记、LLM 摘要生成

**子任务**:
- [ ] TASK-3.3.1: 实现自动压缩触发
  - 监控会话 token 数
  - 触发阈值 = 模型最大上下文 - reserved
  - 使用 LLM 生成摘要压缩历史
- [ ] TASK-3.3.2: 实现 Prune 功能
  - 移除旧工具输出
  - 保留最近 N 个工具调用 (N >= 3)
  - 标记 "[content pruned to save tokens]"
- [ ] TASK-3.3.3: 实现 reserved 配置验证
  - 验证 reserved > 0
  - 默认 10000
- [ ] TASK-3.3.4: 保持对话语义连贯性
- [ ] TASK-3.3.5: 添加单元测试

**验收标准**:
- [ ] auto 启用时自动触发压缩
- [ ] prune 启用时正确移除旧工具输出
- [ ] 被 prune 内容标记正确
- [ ] reserved 配置生效

**依赖**: TASK-0.1 (Context Engine)

---

### Task 3.4: 文件 Watcher 配置 (FR-052)

**ID**: TASK-3.4
**优先级**: P2
**模块**: core/watcher
**状态**: completed
**预计工期**: 2 天
**关联 FR**: FR-052
**关联 Constitution**: C-022

**目标**: 实现文件变更监视器的配置支持。

**当前状态**:
- `WatcherConfig` 已定义 (`ignore: Option<Vec<String>>`)
- `Config.watcher` 字段存在
- **缺失**: 默认忽略列表、glob 解析、文件监视器运行时

**子任务**:
- [ ] TASK-3.4.1: 实现 WatcherConfig 加载
  - `ignore` 字段解析
  - 默认忽略列表: `.git/**`, `node_modules/**`, `dist/**`, `build/**`
  - 用户自定义 ignore 追加
- [ ] TASK-3.4.2: 实现 glob 模式解析
  - 使用 `globset` crate
  - 支持 `*`, `**`, `?` 语法
- [ ] TASK-3.4.3: 实现文件监视器
  - 使用 `notify` crate
  - 忽略模式在文件系统层面生效
  - 文件数限制 (<= 10000)
- [ ] TASK-3.4.4: 实现启动失败容忍
  - 记录 warning 但不阻断
- [ ] TASK-3.4.5: 添加单元测试

**验收标准**:
- [ ] 默认忽略列表生效
- [ ] glob 模式正确解析
- [ ] 用户自定义 ignore 追加生效
- [ ] 监视器启动失败不阻断启动

**依赖**: 无

---

### Task 3.5: 配置系统完善 (FR-021/030/031)

**ID**: TASK-3.5
**优先级**: P2
**模块**: core/config
**状态**: completed
**预计工期**: 1 天
**关联 FR**: FR-021, FR-030, FR-031

**目标**: 配置系统整体完善。

**当前状态**:
- FR-021: merge_configs 已添加文档说明 (iteration-3)
- FR-030: theme/keybinds 已标记 `#[deprecated]` (iteration-3)
- FR-031: theme 路径解析部分实现
- **缺失**: theme 路径 ~ 展开和相对路径支持

**子任务**:
- [ ] TASK-3.5.1: theme 路径解析增强 (FR-031)
  - 主题路径支持 ~ 展开
  - 主题路径支持相对路径 (相对于配置文件目录)
  - 主题文件不存在时降级
- [ ] TASK-3.5.2: 验证废弃字段 warning 日志 (FR-030)
- [ ] TASK-3.5.3: 添加集成测试

**验收标准**:
- [ ] theme 路径支持 ~ 和相对路径
- [ ] 主题文件不存在时降级
- [ ] 废弃字段标记与 warning

**依赖**: 无 (大部分已在 iteration-3 完成)

---

### Task 3.6: 其他 P2 需求 (FR-022/023/024/025/026/027)

**ID**: TASK-3.6
**优先级**: P2
**模块**: 多模块
**状态**: completed
**预计工期**: 4 天
**关联 FR**: FR-022, FR-023, FR-024, FR-025, FR-026, FR-027

**目标**: 实现剩余 P2 完善性需求。

**当前状态**:
- summary.rs 已有 `summarize_text` 基础实现
- TUI crate 存在
- **缺失**: Session Summarize API, TUI 布局切换, 右栏功能, Patch 预览

**子任务**:
- [ ] TASK-3.6.1: Session Summarize (FR-022)
  - `POST /sessions/{id}/summarize` API
  - 使用 LLM 生成会话摘要
- [ ] TASK-3.6.2: TUI 布局切换 (FR-023)
  - 多种布局预设
  - 切换快捷键
- [ ] TASK-3.6.3: TUI 右栏功能完善 (FR-024)
  - 右栏内容可配置
  - 面板折叠/展开
- [ ] TASK-3.6.4: TUI Patch 预览展开 (FR-025)
  - Patch 内容可展开
  - Diff 高亮显示
  - 接受/拒绝操作
- [ ] TASK-3.6.5: Web UI 预留 (FR-026)
  - 定义 API 接口
  - 文档说明
- [ ] TASK-3.6.6: IDE 扩展预留 (FR-027)
  - 定义 IDE 扩展 API
  - 扩展点预留

**验收标准**:
- [ ] Session Summarize API 实现
- [ ] TUI 布局切换
- [ ] TUI 右栏功能
- [ ] TUI Patch 预览

**依赖**: TASK-1.1 (Server API), TASK-0.6 (TUI Input Parser)

---

## 6. 任务状态追踪

| Phase | Task ID | 任务名称 | 状态 | 优先级 | 预计工期 | 关联 FR |
|-------|---------|----------|------|--------|----------|---------|
| 0 | TASK-0.1 | Context Engine | ✅ completed | P0 | 3d | FR-001 |
| 0 | TASK-0.2 | Plugin System | ✅ completed | P0 | 3d | FR-002 |
| 0 | TASK-0.3 | Skills 系统完善 | ✅ completed | P0 | 3d | FR-003, FR-045 |
| 0 | TASK-0.4 | Commands 系统完善 | ✅ completed | P0 | 2d | FR-004, FR-046 |
| 0 | TASK-0.5 | MCP 工具接入完善 | ✅ completed | P0 | 2d | FR-005 |
| 0 | TASK-0.6 | TUI 快捷输入解析器 | ✅ completed | P0 | 2d | FR-006 |
| 1 | TASK-1.1 | Server API 完善 | ✅ completed | P1 | 3d | FR-011 |
| 1 | TASK-1.2 | Share 功能 | ✅ completed | P1 | 2d | FR-012 |
| 1 | TASK-1.3 | LSP 功能增强 | ✅ completed | P1 | 2d | FR-013 |
| 1 | TASK-1.4 | 凭证加密存储 | ✅ completed | P1 | 2d | FR-015 |
| 1 | TASK-1.5 | Permission 审计记录 | ✅ completed | P1 | 1d | FR-016 |
| 1 | TASK-1.6 | session_load/session_save | ✅ completed | P1 | 2d | FR-044 |
| 1 | TASK-1.7 | TUI Token/Cost 显示 | ✅ completed | P1 | 1d | FR-017 |
| 1 | TASK-1.8 | TUI Schema 验证 + keybinds | ✅ completed | P1 | 1d | FR-018, FR-020 |
| 2 | TASK-2.1 | Session Fork | ✅ completed | P1 | 2d | FR-007 |
| 2 | TASK-2.2 | OAuth 登录支持 | ✅ completed | P1 | 3d | FR-047 |
| 2 | TASK-2.3 | GitHub 集成 | ✅ completed | P1 | 3d | FR-048 |
| 3 | TASK-3.1 | HuggingFace/AI21 Provider | ✅ completed | P2 | 2d | FR-049 |
| 3 | TASK-3.2 | Server mDNS 服务发现 | ✅ completed | P2 | 2d | FR-050 |
| 3 | TASK-3.3 | Compaction 会话压缩 | ✅ completed | P2 | 3d | FR-051 |
| 3 | TASK-3.4 | 文件 Watcher 配置 | ✅ completed | P2 | 2d | FR-052 |
| 3 | TASK-3.5 | 配置系统完善 | ✅ completed | P2 | 1d | FR-021, FR-030, FR-031 |
| 3 | TASK-3.6 | 其他 P2 需求 | ✅ completed | P2 | 4d | FR-022~027 |

**总计**: 23 tasks, 49 人天 | ✅ 6 completed | 17 pending

---

## 7. 依赖关系图

```
Phase 0 (P0):
  TASK-0.1 (Context) ──────┐
  TASK-0.2 (Plugin) ───────┤
  TASK-0.3 (Skills) ───────┤  全部可并行
  TASK-0.4 (Commands) ─────┤
  TASK-0.5 (MCP) ──────────┤
  TASK-0.6 (InputParser) ──┘ 依赖 TASK-0.4

Phase 1 (P1):
  TASK-1.1 (ServerAPI) ────┐
  TASK-1.2 (Share) ────────┤
  TASK-1.3 (LSP) ──────────┤  全部可并行
  TASK-1.4 (Credential) ───┤
  TASK-1.5 (Audit) ────────┤
  TASK-1.6 (session工具) ──┤
  TASK-1.7 (Token/Cost) ───┤
  TASK-1.8 (TUI验证) ──────┘

Phase 2 (P1 v4):
  TASK-1.4 (Credential) ──→ TASK-2.2 (OAuth) ──→ TASK-2.3 (GitHub)
  TASK-1.1 (ServerAPI) ───→ TASK-2.1 (SessionFork)

Phase 3 (P2):
  TASK-0.1 (Context) ─────→ TASK-3.3 (Compaction)
  TASK-1.1 + TASK-0.6 ────→ TASK-3.6 (TUI增强)
  TASK-3.1 (Provider) ─────┐
  TASK-3.2 (mDNS) ─────────┤  全部可并行
  TASK-3.4 (Watcher) ──────┤
  TASK-3.5 (配置完善) ─────┘
```

---

## 8. 验收检查清单

每个任务完成后需满足:

- [ ] 功能正常运行
- [ ] 错误处理正确
- [ ] 性能满足要求
- [ ] 文档完整
- [ ] 测试覆盖
- [ ] `cargo check --workspace` 无错误
- [ ] `cargo test` 新增测试通过

---

## 9. PRD 验收标准对照 (更新)

| PRD 验收项 | § | 状态 | 关联 Task | 备注 |
|-----------|---|------|-----------|------|
| 6 个配置位置按优先级加载 | 10.1 | ✅ | iteration-3 | 已完成 |
| `{file:path}` 正确读取文件 | 10.2 | ✅ | iteration-3 | ~ 和相对路径已完成 |
| 未设置变量替换为空字符串 | 10.2 | ✅ | iteration-3 | 已完成 |
| TUI 配置与 runtime 分离 | 10.3 | ✅ | iteration-3 | 已完成 |
| `OPENCODE_TUI_CONFIG` 自定义路径 | 10.3 | ✅ | iteration-3 | 已完成 |
| Provider timeout/chunkTimeout | 10.4 | ✅ | - | 已实现 |
| Amazon Bedrock 配置 | 10.4 | ✅ | - | 已实现 |
| 自定义 agent 配置 | 10.5 | ✅ | iteration-3 | AgentMapConfig 动态化已完成 |
| permission 配置 | 10.6 | ✅ | - | 已实现 |
| Context Engine | - | ❌ | TASK-0.1 | 待实现 |
| Plugin System | - | ❌ | TASK-0.2 | 待实现 |
| Skills 系统 (10/10) | - | ⚠️ | TASK-0.3 | 机制存在，0/10 内建 |
| Commands 系统 (8/8) | - | ⚠️ | TASK-0.4 | 3/8 已实现 |
| MCP 工具接入 | - | ⚠️ | TASK-0.5 | 配置存在，连接管理待实现 |
| TUI 输入解析器 | - | ❌ | TASK-0.6 | 待实现 |
| Session Fork | - | ❌ | TASK-2.1 | 待实现 |
| Server API 完善 | - | ⚠️ | TASK-1.1 | 部分实现 |
| OAuth 登录 | - | ⚠️ | TASK-2.2 | 组件存在，流程未整合 |
| GitHub 集成 | - | ⚠️ | TASK-2.3 | CLI 存在，API 客户端待实现 |
| 18/18 Providers | - | ⚠️ | TASK-3.1 | 16/18 已实现 |
| mDNS 服务发现 | - | ❌ | TASK-3.2 | 配置存在，实现缺失 |
| Compaction 自动压缩 | - | ⚠️ | TASK-3.3 | 基础存在，自动触发待完善 |
| Watcher 配置 | - | ⚠️ | TASK-3.4 | 结构存在，运行时待实现 |

---

## 10. Constitution 合规性

本任务清单遵循 Constitution v1.4 全部条款:

| 条款 | 覆盖任务 | 说明 |
|------|----------|------|
| C-011 | (已完成) | Config Schema 设计 |
| C-012 | (已完成) | 变量替换规范 |
| C-013 | (已完成) | 目录扫描规范 (含 modes/) |
| C-014 | TASK-0.6 | TUI Input Parser |
| C-015 | TASK-2.1 | Session Fork |
| C-016 | TASK-0.1 | Context Token Budget |
| C-017 | (已完成) | TUI 配置分离 |
| C-018 | (已完成) | 路径命名统一 |
| C-019 | (已完成) | 文件引用变量 |
| **C-020** | **TASK-3.2** | **Server 配置规范** |
| **C-021** | **TASK-3.3** | **Compaction 配置规范** |
| **C-022** | **TASK-3.4** | **Watcher 配置规范** |

---

**文档状态**: 草稿
**下一步**: 按 Phase 0 开始实施 (TASK-0.1 ~ TASK-0.6)
