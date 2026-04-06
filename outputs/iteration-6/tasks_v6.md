# OpenCode-RS 任务清单 v6

**版本**: 6.0
**日期**: 2026-04-05
**基于**: spec_v6.md 拆分 — 功能需求任务部分
**状态**: 已完成

---

## 1. 文档概述

本文档从 spec_v6.md 拆分而来，包含所有功能需求 (FR-001 ~ FR-088) 的详细规格定义和验收标准。
实施架构、阶段规划和状态追踪请参见 `plan_v6.md`。

### 1.1 与 v5 的关系

v6 保留 v5 的所有需求 (FR-001 ~ FR-062)，并新增：
- **FR-063 ~ FR-064**: P0 认证架构缺陷 (Provider 认证分层 / OAuth/Device Code)
- **FR-065 ~ FR-073**: P1 重要功能缺失 (云厂商认证 / Remote Config / MCP OAuth / TUI 三栏 / TUI 状态机 / Context Engine 分层 / WASM 运行时 / 凭证加密 / Compaction 阈值)
- **FR-074 ~ FR-085**: P2 增强功能 (Event Bus 完整 / Share 服务 / SDK 输出 / OpenAPI 文档 / LSP 扩展 / session_load/save / HF+AI21 / Formatters 接入 / 虚拟滚动 / Server Basic Auth / 观测性)
- **FR-087 ~ FR-088**: P3 远期规划 (GitHub Integration / Enterprise 配置)

---

## 2. P0 - 阻断性问题

> FR-001 ~ FR-010, FR-033, FR-034 继承自 v5，内容不变。以下为 v6 新增 P0 需求。

### FR-063: Provider 认证协议分层抽象

**模块**: llm/auth
**严重程度**: P0
**来源**: v6 (PRD-providers.md §7.x.5, §7.x.7, Constitution C-026 v1.6 重写)

#### 需求描述

PRD-providers.md 要求认证架构分为 4 层，当前 `provider.rs` 使用平面结构 `ProviderConfig { api_key, model, temperature }`，无法支持 75+ providers 的认证需求。必须重构为分层认证架构。

#### 详细规格

1. **4 层认证架构** (PRD-providers.md §7.x.7)
   - **Layer 1: Credential Source** (凭证来源)
     - auth.json (`~/.local/share/opencode/auth.json`)
     - 环境变量 (`OPENCODE_*` 或 provider 特定变量)
     - .env 文件 (项目根目录)
     - 配置文件内联变量
     - 文件引用 (`{file:path}`)
     - OAuth Store (OAuth 流程结果)
     - System Keychain (系统密钥链，Rust 版可扩展)
   - **Layer 2: Auth Mechanism** (认证机制)
     - API Key, Bearer Token, Basic Auth
     - OAuth Browser Flow, Device Code Flow
     - AWS Credential Chain, Service Account JSON
     - SSO Gateway Token
   - **Layer 3: Provider Transport** (传输层)
     - Header `Authorization: Bearer {token}`
     - Header `Authorization: Basic {base64}`
     - 自定义 Header (通过 `options.headers` 配置)
     - Query / Body 字段注入
     - AWS SigV4 签名
     - OpenAI-compatible transport (`/v1/chat/completions`)
     - Responses API transport (`/v1/responses`)
   - **Layer 4: Runtime Access Control** (运行时访问控制)
     - Server Basic Auth
     - MCP Token Store
     - Enterprise Central Policy
     - Provider allow/deny list

2. **数据结构** (PRD-providers.md §7.x.8)
   ```rust
   pub enum AuthMechanism {
       ApiKey, BearerToken, BasicAuth,
       OAuthBrowser, DeviceCode,
       AwsCredentialChain, ServiceAccountJson, SsoGatewayToken,
   }

   pub enum CredentialSource {
       AuthFile, EnvVar, DotEnv, ConfigInline, FileRef, OAuthStore, SystemKeychain,
   }

   pub struct ProviderAuthSpec {
       pub provider_id: String,
       pub mechanism: AuthMechanism,
       pub source: CredentialSource,
       pub supports_interactive_login: bool,
       pub supports_env_override: bool,
       pub supports_header_injection: bool,
   }
   ```

3. **约束**
   - Provider 认证与 Runtime 访问认证必须分开接口
   - 订阅式登录 (ChatGPT Plus/Pro, GitHub Copilot, GitLab Duo) 不得硬编码为通用协议
   - Claude Pro/Max 订阅直连不得作为稳定能力支持 (Anthropic 明确禁止)
   - 本地/网关/OpenAI-compatible Provider 必须允许自定义 `baseURL + headers + model map`

#### 验收标准

- [ ] 认证分为 4 层 (Credential Source / Auth Mechanism / Provider Transport / Runtime Access Control)
- [ ] Provider 认证与 Runtime 访问认证分开接口
- [ ] 每个 provider 声明其 ProviderAuthSpec
- [ ] 订阅式登录未硬编码为通用协议
- [ ] 认证失败不暴露具体错误原因
- [ ] 日志不记录 API Key 或完整请求体

---

### FR-064: OAuth/Device Code 浏览器登录流程

**模块**: auth
**严重程度**: P0
**来源**: v6 (PRD-providers.md §7.x.5.B, Constitution C-026 §3c-d)

#### 需求描述

PRD-providers.md 明确要求支持 OAuth Browser Flow 和 Device Code Flow，用于 GitHub Copilot、GitLab Duo 等订阅式 provider 的登录。当前 `auth/oauth.rs` 存在但未见完整实现。

#### 详细规格

1. **OAuth Browser Flow**
   - 本地 HTTP 回调服务器 (接收 OAuth callback)
   - 浏览器自动打开 (`xdg-open` / `open` / `start`)
   - Token 持久化到 `auth.json` 或 OAuth Store
   - Token 自动刷新 (refresh_token 机制)
   - 超时与取消处理

2. **Device Code Flow**
   - Device code 获取与展示
   - 轮询授权状态 (polling interval 遵循 provider 规范)
   - 超时与取消处理
   - 用户输入 code 完成授权 (如 GitHub Copilot 的 `github.com/login/device`)

3. **适用 Provider**
   - OpenAI ChatGPT Plus/Pro (OAuth Browser Flow)
   - GitHub Copilot (Device Code Flow)
   - GitLab Duo (OAuth 推荐 / PAT 备选)

4. **安全约束**
   - OAuth 回调服务器仅监听 localhost
   - Token 文件权限必须为 600
   - 认证状态缓存不得超过 24 小时

#### 验收标准

- [ ] OAuth Browser Flow 完整实现 (本地回调 + 浏览器打开 + token 持久化 + 自动刷新)
- [ ] Device Code Flow 完整实现 (code 获取 + 轮询授权 + 超时处理)
- [ ] GitHub Copilot 登录可工作
- [ ] GitLab Duo OAuth 登录可工作
- [ ] Token 文件权限为 600
- [ ] 认证状态缓存不超过 24 小时

---

## 3. P1 - 核心功能缺失

> FR-011 ~ FR-020, FR-032, FR-035 ~ FR-039, FR-044 ~ FR-048, FR-053 ~ FR-056 继承自 v5，内容不变。以下为 v6 新增 P1 需求。

### FR-065: 云厂商原生认证 (AWS SigV4, Vertex AI, SAP AI Core)

**模块**: llm
**严重程度**: P1
**来源**: v6 (PRD-providers.md §7.x.5.C, Constitution C-026 §3e, C-030 §2b-e)

#### 需求描述

云厂商 Provider 复用云平台认证体系，不是简单 API Key。Bedrock provider 存在但 AWS credential chain 优先级未实现；Vertex AI 的 GOOGLE_APPLICATION_CREDENTIALS 支持不完整；SAP AI Core 完全缺失。

#### 详细规格

1. **Amazon Bedrock**
   - 支持 `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY`
   - 支持 `AWS_PROFILE`
   - 支持 `AWS_BEARER_TOKEN_BEDROCK`
   - 支持 `AWS_WEB_IDENTITY_TOKEN_FILE` + `AWS_ROLE_ARN` (OIDC / IRSA)
   - **优先级**: Bearer Token > AWS Credential Chain
   - Credential Chain 顺序: 环境变量 > 配置文件 > IAM Role

2. **Google Vertex AI**
   - 需要 `GOOGLE_CLOUD_PROJECT`
   - `GOOGLE_APPLICATION_CREDENTIALS` 指向 service account JSON
   - 或走 `gcloud auth application-default login`

3. **SAP AI Core**
   - 接受 service key JSON
   - 通过 `AICORE_SERVICE_KEY` 环境变量注入

4. **Cloudflare AI Gateway / Workers AI**
   - 除 API token 外，还要求 Account ID
   - AI Gateway 还需要 Gateway ID

#### 验收标准

- [ ] Bedrock AWS Credential Chain 按优先级工作
- [ ] Vertex AI GOOGLE_APPLICATION_CREDENTIALS 可配置
- [ ] SAP AI Core service key 可注入
- [ ] Cloudflare Account ID + Gateway ID 可配置

---

### FR-066: Remote Config 自动发现 (.well-known/opencode)

**模块**: core/config
**严重程度**: P1
**来源**: v6 (PRD-Configuration §1, Constitution C-037)

#### 需求描述

PRD-Configuration 优先级 1 为 Remote Config，当前代码有 `fetch_remote_config_with_fallback` 但需 `OPENCODE_REMOTE_CONFIG` 环境变量才触发，非自动发现 `.well-known/opencode` 端点。

#### 详细规格

1. **自动发现机制**
   - 启动时检查 `.well-known/opencode` 端点 (如果配置了 remote URL)
   - Remote Config 优先级高于本地配置 (见 C-011)
   - Remote Config 获取失败时降级到本地配置，记录 warning

2. **配置内容**
   - Remote Config 可覆盖: providers, agents, tools, formatters, instructions
   - Remote Config **不得覆盖**: permission (安全约束，见 C-024 §5a)
   - Remote Config 必须支持 JSON 格式

3. **安全约束**
   - Remote Config 必须支持 HTTPS (禁止 HTTP)
   - Remote Config 响应必须进行 JSON Schema 验证
   - Remote Config 不得包含 API Key 明文 (必须使用变量引用)
   - Remote Config 缓存不得超过 1 小时

#### 验收标准

- [ ] 启动时自动检查 `.well-known/opencode` 端点
- [ ] Remote Config 优先级高于本地配置
- [ ] Remote Config 不得覆盖 permission 配置
- [ ] Remote Config 仅支持 HTTPS
- [ ] Remote Config 缓存不超过 1 小时

---

### FR-067: disabled_providers 优先级高于 enabled_providers

**模块**: core/config
**严重程度**: P1
**来源**: v6 (PRD-Configuration §5.14, Constitution C-030 §1c)

#### 需求描述

PRD-Configuration 明确要求 `disabled_providers > enabled_providers` 优先级规则，代码中两字段都存在但未见优先级冲突处理逻辑。

#### 详细规格

1. **优先级规则**
   - `disabled_providers` (黑名单) > `enabled_providers` (白名单) — 强制
   - 同时设置时，disabled 中的 provider 即使出现在 enabled 中也被禁用
   - 被禁用的 provider 不得出现在模型列表中

2. **实现要求**
   - 配置加载时先应用 enabled_providers，再排除 disabled_providers
   - 配置验证阶段检测冲突并记录 warning

#### 验收标准

- [ ] disabled_providers 严格优先于 enabled_providers
- [ ] 被禁用的 provider 不出现在模型列表中
- [ ] 配置冲突时记录 warning

---

### FR-068: MCP OAuth 独立 token store

**模块**: mcp/auth
**严重程度**: P1
**来源**: v6 (PRD-providers.md §7.x.5.D, Constitution C-033)

#### 需求描述

PRD-providers.md 要求 MCP OAuth 存储在独立 `mcp-auth.json`，不与普通 provider auth 混用。

#### 详细规格

1. **存储隔离**
   - MCP OAuth 凭证存储在 `~/.local/share/opencode/mcp-auth.json`
   - MCP OAuth 不得与普通 provider auth (`auth.json`) 混用
   - MCP OAuth 凭证结构独立于 Provider Auth 结构

2. **OAuth 流程**
   - 通过 `opencode mcp auth` 命令触发浏览器 OAuth 流程
   - OAuth 回调由本地 HTTP 服务器接收
   - Token 持久化到 `mcp-auth.json`

3. **非 OAuth MCP Server**
   - 不走 OAuth 的 MCP Server 可在配置中显式设置 `oauth: false`
   - 通过 `headers.Authorization` 手动注入 Bearer Token
   - API Key / Header 方式认证的 MCP Server 不使用 `mcp-auth.json`

4. **安全约束**
   - `mcp-auth.json` 文件权限必须为 600
   - MCP token 过期时必须自动触发刷新或重新认证
   - MCP token 不得传递给非 MCP 的 provider

#### 验收标准

- [ ] MCP OAuth 存储在独立 `mcp-auth.json`
- [ ] MCP OAuth 未与普通 provider auth 混用
- [ ] `mcp-auth.json` 文件权限为 600
- [ ] 非 OAuth MCP Server 可通过 `oauth: false` 配置
- [ ] MCP token 不传递给非 MCP provider

---

### FR-069: TUI 三栏布局与 Inspector 面板

**模块**: tui
**严重程度**: P1
**来源**: v6 (PRD-tui.md §7.15.3, §7.15.4, Constitution C-034 §1-2)

#### 需求描述

PRD-tui.md 要求三栏布局 (Sidebar/Timeline/Inspector)，Inspector 需 6 个 tab (Todo/Diff/Diagnostics/Context/Permissions/Files)。当前仅有 `right_panel.rs` 和 `components/right_panel.rs`，未完成。

#### 详细规格

1. **三栏布局**
   - 左栏: Sidebar (会话列表、Agent 选择、设置入口)
   - 中栏: Timeline (对话主区域，消息流、工具执行结果)
   - 右栏: Inspector (详细信息面板)

2. **Inspector 6 个 tab**
   - Todo: 当前任务的 todo 列表
   - Diff: 文件变更 diff 视图
   - Diagnostics: LSP 诊断信息
   - Context: 上下文文件/变量列表
   - Permissions: 权限请求历史
   - Files: 会话涉及的文件列表

3. **响应式布局**
   - >= 160 列: 三栏布局
   - 100~160 列: 双栏布局 (Sidebar + Timeline)
   - < 100 列: 单栏 + 侧边面板 Tab 弹出

#### 验收标准

- [ ] >= 160 列终端显示三栏布局
- [ ] 100~160 列终端显示双栏布局
- [ ] < 100 列终端显示单栏 + Tab 弹出
- [ ] Inspector 包含 6 个 tab
- [ ] Tab 切换不重新渲染整个面板

---

### FR-070: TUI 状态机完整实现

**模块**: tui
**严重程度**: P1
**来源**: v6 (PRD-tui.md §7.15.7, Constitution C-034 §3-4)

#### 需求描述

PRD-tui.md 定义 10 种顶层状态，需验证是否全部实现并正确流转。

#### 详细规格

1. **10 种状态**
   - `idle`: 等待用户输入
   - `composing`: 正在编辑输入
   - `submitting`: 消息已提交，等待 server 确认
   - `streaming`: 模型正在输出
   - `executing_tool`: 工具执行中
   - `awaiting_permission`: 等待用户审批
   - `showing_diff`: 当前聚焦 patch/diff
   - `showing_error`: 当前有错误需要确认
   - `aborting`: 终止当前轮
   - `reconnecting`: TUI 与 server 重连中

2. **状态流转约束**
   - 每次状态变更必须触发 UI 刷新
   - `aborting` 状态必须可中断 `streaming` / `executing_tool`
   - `reconnecting` 状态必须有超时机制

3. **关键事件**
   - send_prompt, stream_started, tool_started
   - permission_requested, permission_resolved
   - diff_ready, task_finished
   - abort_requested, server_disconnected, reconnect_success

#### 验收标准

- [ ] 10 种状态全部实现
- [ ] aborting 可中断 streaming / executing_tool
- [ ] reconnecting 有超时机制
- [ ] 每次状态变更触发 UI 刷新
- [ ] 状态切换延迟不超过 16ms (60fps)

---

### FR-071: Context Engine 分层上下文 (L0-L4)

**模块**: core/context
**严重程度**: P1
**来源**: v6 (PRD §6, Constitution C-035)

#### 需求描述

PRD 要求 L0-L4 五层上下文，当前 `context.rs` 可能未完全实现 token budget 计算和 relevance ranking。

#### 详细规格

1. **五层上下文结构**
   - L0 - 显式输入: 用户直接提供的输入 (prompt, @file 引用)
   - L1 - 会话上下文: 当前会话的对话历史、工具执行结果
   - L2 - 项目上下文: 项目结构、AGENTS.md、instructions 文件
   - L3 - 结构化上下文: LSP 符号、诊断、git diff
   - L4 - 压缩记忆: 历史会话的压缩摘要

2. **Token Budget 计算**
   - 每层上下文声明其 token 预算上限
   - 总 token 使用不得超过模型上下文窗口的 85%
   - 超过 85% 触发预警
   - 超过 92% 触发自动 compact
   - 超过 95% 强制转入新 session

3. **Relevance Ranking**
   - 上下文条目按相关性排序
   - L0 > L1 > L2 > L3 > L4 (优先级递减)
   - 同层内按时间倒序或用户显式指定顺序

#### 验收标准

- [ ] L0-L4 五层上下文结构实现
- [ ] 85% 预警 / 92% compact / 95% 强制转新 session
- [ ] 上下文按 L0 > L1 > L2 > L3 > L4 排序
- [ ] 上下文不包含 API Key 或敏感环境变量

---

### FR-072: Plugin WASM 运行时

**模块**: plugin
**严重程度**: P1
**来源**: v6 (PRD §8, Constitution C-036)

#### 需求描述

PRD 要求 WASM 插件 (wasmtime) + Sidecar 插件，当前 plugin crate 仅有 discovery/loader/registry，无 WASM 执行引擎。

#### 详细规格

1. **WASM 运行时**
   - 使用 wasmtime 作为 WASM 执行引擎
   - WASM 插件声明所需 capabilities (文件系统/网络/环境变量)
   - WASM 插件通过 Sidecar 模式与主进程通信

2. **插件能力**
   - WASM 插件可注册自定义 tools
   - WASM 插件可注册自定义 commands
   - WASM 插件可注册 event listeners
   - WASM 插件不得直接访问主进程内存

3. **沙箱隔离**
   - WASM 插件默认只能访问自身目录
   - 文件系统访问必须通过 WASI 接口，受 capabilities 限制
   - 网络访问必须显式声明允许的域名
   - 环境变量访问必须显式声明允许的变量名

4. **生命周期**
   - WASM 插件加载失败不阻断启动，记录 warning
   - WASM 插件执行超时可配置 (默认 30s)
   - WASM 插件崩溃不得导致主进程退出

#### 验收标准

- [ ] wasmtime 作为 WASM 执行引擎
- [ ] WASM 插件通过 WASI 接口访问文件系统
- [ ] WASM 插件崩溃不导致主进程退出
- [ ] WASM 插件不执行系统命令 (除非显式声明)
- [ ] WASM 插件不读取 .git/ 目录

---

### FR-073: 凭证加密存储

**模块**: auth
**严重程度**: P1
**来源**: v6 (PRD 安全要求, Constitution C-026 §6, C-028 §4b)

#### 需求描述

PRD 安全要求"凭证本地加密存储"，当前 `auth/credential_store.rs` 需验证是否使用系统密钥链或加密。

#### 详细规格

1. **加密存储**
   - 使用系统密钥链 (macOS Keychain, Windows DPAPI, Linux libsecret)
   - 或使用 AES-256-GCM 加密 + 系统派生密钥
   - `auth.json` 文件权限必须为 600

2. **安全约束**
   - 禁止在配置文件中明文存储 API Key
   - API Key 文件权限必须为 600 (仅所有者可读写)
   - 认证失败不暴露具体错误原因 (防止枚举攻击)

#### 验收标准

- [ ] 凭证使用系统密钥链或 AES-256-GCM 加密存储
- [ ] `auth.json` 文件权限为 600
- [ ] 配置文件中无明文 API Key
- [ ] 认证失败不暴露具体错误原因

---

## 4. P2 - 增强功能

> FR-021 ~ FR-031, FR-040 ~ FR-052, FR-057 ~ FR-062 继承自 v5，内容不变。以下为 v6 新增/修订 P2 需求。

### FR-074: Event Bus 事件类型完整性

**模块**: core/bus
**严重程度**: P2
**来源**: v6 (PRD §7.10, Constitution C-025)

#### 需求描述

PRD 定义 12+ 事件类型，需验证 `bus.rs` 覆盖度。

#### 详细规格

1. **事件类型清单**
   - `session.created` / `session.updated` / `session.compacted`
   - `message.updated`
   - `tool.execute.before` / `tool.execute.after`
   - `permission.asked` / `permission.resolved`
   - `file.edited`
   - `lsp.updated`
   - `shell.env`
   - `tui.toast.show`

2. **事件总线架构**
   - 发布/订阅模式
   - 异步事件分发
   - 事件过滤和路由
   - 事件处理失败不影响主流程

#### 验收标准

- [ ] 所有 12+ 事件类型可发布和订阅
- [ ] 事件异步分发不阻塞主流程
- [ ] 插件可订阅和发布事件
- [ ] 事件处理失败有容错机制

---

### FR-075: Share 服务层

**模块**: core/share
**严重程度**: P2
**来源**: v6 (PRD §7.12, Constitution C-027)

#### 需求描述

PRD 要求 self-hosted share server + public share server + 短链 + 访问令牌 + 过期时间 + 红线脱敏，当前仅有本地导出。

#### 详细规格

1. **Share 服务**
   - Self-hosted share server (本地部署)
   - Public share server (公共托管)
   - 短链生成
   - 访问令牌控制
   - 过期时间设置
   - 红线脱敏 (API Key / 敏感信息)

#### 验收标准

- [ ] Session 可分享到 self-hosted server
- [ ] 短链可生成和访问
- [ ] 访问令牌可设置和验证
- [ ] 过期时间可配置
- [ ] 敏感信息自动脱敏

---

### FR-076: SDK 输出 (Rust + TypeScript)

**模块**: server
**严重程度**: P2
**来源**: v6 (PRD §7.16)

#### 需求描述

PRD 要求提供 Rust SDK 和 TypeScript SDK，当前仅有 REST/WS/SSE API。

#### 详细规格

1. **Rust SDK**
   - 基于 OpenAPI 规范自动生成
   - 支持同步和异步调用
   - 类型安全的 API 客户端

2. **TypeScript SDK**
   - 基于 OpenAPI 规范自动生成
   - 支持 Promise 和 async/await
   - 浏览器和 Node.js 兼容

#### 验收标准

- [ ] Rust SDK 可生成和使用
- [ ] TypeScript SDK 可生成和使用
- [ ] SDK 基于 OpenAPI 规范自动生成
- [ ] SDK 覆盖所有 Server API

---

### FR-077: OpenAPI 文档自动生成

**模块**: server
**严重程度**: P2
**来源**: v6 (PRD §10 验收标准)

#### 需求描述

PRD v1 验收标准要求"提供完整 OpenAPI 文档"。

#### 详细规格

1. **OpenAPI 3.1 规范**
   - 从代码自动生成 OpenAPI 文档
   - 支持 Swagger UI 展示
   - 支持 SDK 代码生成

#### 验收标准

- [ ] OpenAPI 3.1 文档完整
- [ ] 可从代码自动生成
- [ ] Swagger UI 可访问
- [ ] 可用于 SDK 生成

---

### FR-078: LSP definition/references/hover/code actions

**模块**: lsp
**严重程度**: P2
**来源**: v6 (PRD v1.1 扩展)

#### 需求描述

PRD 要求 v1.1 扩展 definition/references/hover/code actions，当前仅实现 diagnostics + symbols。

#### 详细规格

1. **LSP 扩展**
   - `textDocument/definition`: 跳转到定义
   - `textDocument/references`: 查找引用
   - `textDocument/hover`: 悬停信息
   - `textDocument/codeAction`: 代码操作

#### 验收标准

- [ ] definition 可工作
- [ ] references 可工作
- [ ] hover 可工作
- [ ] code actions 可工作

---

### FR-079: session_load/session_save 工具

**模块**: tools
**严重程度**: P2
**来源**: v6 (差距分析 #16)

#### 需求描述

上次分析已识别，仍未实现。

#### 详细规格

1. **session_load 工具**
   - 加载指定 session 的历史记录
   - 恢复上下文到当前会话

2. **session_save 工具**
   - 保存当前 session 到指定位置
   - 导出为可分享格式

#### 验收标准

- [ ] session_load 可加载历史 session
- [ ] session_save 可保存当前 session
- [ ] 加载后上下文正确恢复

---

### FR-080: HuggingFace + AI21 Provider

**模块**: llm
**严重程度**: P2
**来源**: v6 (差距分析 #17)

#### 需求描述

上次分析已识别，文件存在 (`huggingface.rs`, `ai21.rs`) 但可能为 stub。

#### 详细规格

1. **HuggingFace Provider**
   - 支持 Inference API
   - 支持 Inference Endpoints
   - 支持 HF_TOKEN 环境变量

2. **AI21 Provider**
   - 支持 Jurassic 系列模型
   - 支持 AI21_API_KEY 环境变量

#### 验收标准

- [ ] HuggingFace Provider 可工作
- [ ] AI21 Provider 可工作
- [ ] 非 stub，实际可调用 API

---

### FR-081: Formatters 配置接入 agent 执行循环

**模块**: core/formatter
**严重程度**: P2
**来源**: v6 (差距分析 #19)

#### 需求描述

PRD-Configuration 定义 formatter 配置，`formatter.rs` 存在但需验证是否接入 agent 执行循环。

#### 详细规格

1. **Formatter 接入**
   - 文件写入后自动触发匹配的 formatter
   - Formatter 执行失败不阻断写入，记录 warning
   - 多个 formatter 匹配同一扩展名时，按配置顺序执行

#### 验收标准

- [ ] Formatter 接入 agent 执行循环
- [ ] 文件写入后自动触发格式化
- [ ] Formatter 失败不阻断写入

---

### FR-082: Compaction 自动触发阈值

**模块**: core/compaction
**严重程度**: P2
**来源**: v6 (PRD §7.9, Constitution C-035 §2b-e)

#### 需求描述

PRD 要求 85% 预警 / 92% 触发 compact / 95% 强制转入新 session。

#### 详细规格

1. **自动触发阈值**
   - 85%: 预警 (toast 通知)
   - 92%: 自动触发 compact
   - 95%: 强制转入新 session

#### 验收标准

- [ ] 85% 时显示预警
- [ ] 92% 时自动 compact
- [ ] 95% 时强制转入新 session

---

### FR-083: TUI 虚拟滚动

**模块**: tui
**严重程度**: P2
**来源**: v6 (PRD-tui.md §7.15.18, Constitution C-034 §5a)

#### 需求描述

PRD-tui.md 性能要求 10k+ 消息 session 可打开，需虚拟滚动。`components/virtual_list.rs` 存在需验证。

#### 详细规格

1. **虚拟滚动**
   - Timeline 虚拟滚动
   - 长 block 延迟渲染
   - Shell 输出和普通消息分缓存
   - Inspector 按需计算
   - Markdown 渲染缓存
   - Diff 只渲染可视区

#### 验收标准

- [ ] 10k+ 消息 session 可打开
- [ ] 10k+ 行 shell 输出不阻塞输入
- [ ] 大 diff 能渐进展示
- [ ] 空闲刷新 10~20 FPS
- [ ] 流式输出 20~30 FPS

---

### FR-084: Server 认证 (HTTP Basic Auth)

**模块**: server
**严重程度**: P2
**来源**: v6 (PRD-Configuration §5.14, Constitution C-026 §5b)

#### 需求描述

PRD-Configuration 要求 `OPENCODE_SERVER_PASSWORD` + `OPENCODE_SERVER_USERNAME` 支持 HTTP Basic Auth。

#### 详细规格

1. **HTTP Basic Auth**
   - 通过 `OPENCODE_SERVER_PASSWORD` 启用
   - 用户名默认 "opencode"
   - 可通过 `OPENCODE_SERVER_USERNAME` 覆盖

#### 验收标准

- [ ] `OPENCODE_SERVER_PASSWORD` 启用 Basic Auth
- [ ] 用户名可配置
- [ ] 未认证请求返回 401

---

### FR-085: 观测性 (tracing/crash recovery/token cost 统计)

**模块**: core
**严重程度**: P2
**来源**: v6 (PRD §7.17)

#### 需求描述

PRD 要求结构化日志、session traces、tool spans、provider latency/token/cost 统计、崩溃转储。

#### 详细规格

1. **结构化日志**
   - 使用 `tracing` crate
   - Session traces 记录
   - Tool spans 记录

2. **统计**
   - Provider latency 统计
   - Token 使用统计
   - Cost 统计

3. **崩溃恢复**
   - 崩溃转储
   - 自动恢复最近 session

#### 验收标准

- [ ] 结构化日志输出
- [ ] Session traces 可查询
- [ ] Tool spans 可追踪
- [ ] Provider latency/token/cost 统计可展示
- [ ] 崩溃后可恢复最近 session

---

## 5. 功能需求清单汇总

### 按模块分组

| 模块 | FR 编号 | 需求名称 | 优先级 | 来源 |
|------|---------|----------|--------|------|
| core | FR-001 | Context Engine | P0 | v2 |
| core | FR-003 | Skills 系统 | P0 | v2 |
| core | FR-004 | Commands 系统 | P0 | v2 |
| core | FR-012 | Share 功能 | P1 | v2 |
| core | FR-014 | 插件事件总线 | P1 | v2 |
| core | FR-022 | Session Summarize | P2 | v2 |
| core | FR-051 | Compaction 会话压缩 | P2 | v4 |
| core | FR-052 | 文件 Watcher 配置 | P2 | v4 |
| core | FR-057 | Event Bus 事件总线 | P2 | v5 |
| core | FR-058 | Effect System 效果系统 | P2 | v5 |
| core | FR-071 | Context Engine 分层上下文 (L0-L4) | P1 | v6 |
| core | FR-074 | Event Bus 事件类型完整性 | P2 | v6 |
| core | FR-075 | Share 服务层 | P2 | v6 |
| core | FR-082 | Compaction 自动触发阈值 | P2 | v6 |
| core | FR-085 | 观测性 (tracing/crash recovery/token cost) | P2 | v6 |
| core | FR-088 | Enterprise 配置 (Central Config + SSO) | P3 | v6 |
| core/config | FR-008 | 多层配置合并 | P0 | v2 |
| core/config | FR-009 | .opencode 目录加载 | P0 | v2 |
| core/config | FR-010 | Provider 环境变量约定 | P0 | v2 |
| core/config | FR-021 | 配置系统完善 | P2 | v2 |
| core/config | FR-030 | 废弃字段清理 | P2 | v2 |
| core/config | FR-033 | OPENCODE_TUI_CONFIG 环境变量 | P0 | v3 |
| core/config | FR-034 | TUI 配置分离为独立文件 | P0 | v3 |
| core/config | FR-035 | modes/ 目录扫描 | P1 | v3 |
| core/config | FR-036 | 配置路径命名统一 | P1 | v3 |
| core/config | FR-037 | {file:path} ~ 路径展开 | P1 | v3 |
| core/config | FR-038 | {file:path} 相对路径支持 | P1 | v3 |
| core/config | FR-039 | .opencode/ 目录扫描集成 | P1 | v3 |
| core/config | FR-040 | 变量替换覆盖完整性 | P2 | v3 |
| core/config | FR-041 | theme/keybinds 迁移到 TUI | P2 | v3 |
| core/config | FR-042 | AgentMapConfig 动态 HashMap | P2 | v3 |
| core/config | FR-043 | JSON Schema 远程验证 | P2 | v3 |
| core/config | FR-054 | Provider 控制 (disabled/enabled) | P1 | v5 |
| core/config | FR-055 | Formatters 自动格式化 | P1 | v5 |
| core/config | FR-056 | Instructions 指令文件加载 | P1 | v5 |
| core/config | FR-066 | Remote Config 自动发现 | P1 | v6 |
| core/config | FR-067 | disabled_providers 优先级 | P1 | v6 |
| core/context | FR-056 | Instructions 指令文件加载 | P1 | v5 |
| core/formatter | FR-055 | Formatters 自动格式化 | P1 | v5 |
| core/formatter | FR-081 | Formatters 接入 agent 执行循环 | P2 | v6 |
| core/share | FR-075 | Share 服务层 | P2 | v6 |
| core/tools | FR-044 | session_load/session_save | P1 | v4 |
| core/tools | FR-053 | Tools 配置禁用机制 | P1 | v5 |
| core/tools | FR-079 | session_load/session_save 工具 | P2 | v6 |
| core/watcher | FR-052 | 文件 Watcher 配置 | P2 | v4 |
| config/remote | FR-062 | Remote Config 安全验证 | P2 | v5 |
| config/tui | FR-019 | scroll_acceleration 结构修复 | P1 | v2 |
| config/tui | FR-020 | keybinds 自定义绑定 | P1 | v2 |
| config/tui | FR-031 | theme 路径解析增强 | P2 | v2 |
| llm | FR-049 | HuggingFace/AI21 Provider | P2 | v4 |
| llm | FR-063 | Provider 认证协议分层抽象 | P0 | v6 |
| llm | FR-065 | 云厂商原生认证 | P1 | v6 |
| llm | FR-080 | HuggingFace + AI21 Provider 完整实现 | P2 | v6 |
| auth | FR-015 | 凭证加密存储 | P1 | v2 |
| auth | FR-029 | OAuth 登录预留 | P2 | v2 |
| auth | FR-047 | OAuth 登录支持 | P1 | v4 |
| auth | FR-064 | OAuth/Device Code 浏览器登录流程 | P0 | v6 |
| auth | FR-073 | 凭证加密存储 (完善) | P1 | v6 |
| tui | FR-017 | TUI Token/Cost 显示 | P1 | v2 |
| tui | FR-023 | TUI 布局切换 | P2 | v2 |
| tui | FR-024 | TUI 右栏功能完善 | P2 | v2 |
| tui | FR-025 | TUI Patch 预览展开 | P2 | v2 |
| tui | FR-026 | Web UI | P2 | v2 |
| tui | FR-069 | TUI 三栏布局与 Inspector 面板 | P1 | v6 |
| tui | FR-070 | TUI 状态机完整实现 | P1 | v6 |
| tui | FR-083 | TUI 虚拟滚动 | P2 | v6 |
| plugin | FR-002 | Plugin System | P0 | v2 |
| plugin | FR-072 | Plugin WASM 运行时 | P1 | v6 |
| mcp | FR-005 | MCP 工具接入 | P0 | v2 |
| mcp | FR-068 | MCP OAuth 独立 token store | P1 | v6 |
| server | FR-006 | TUI 快捷输入解析器 | P0 | v2 |
| server | FR-007 | Session Fork | P0 | v2 |
| server | FR-011 | Server API 完善 | P1 | v2 |
| server | FR-050 | Server mDNS 服务发现 | P2 | v4 |
| server | FR-059 | Streaming 消息架构 | P2 | v5 |
| server | FR-076 | SDK 输出 (Rust + TypeScript) | P2 | v6 |
| server | FR-077 | OpenAPI 文档自动生成 | P2 | v6 |
| server | FR-084 | Server 认证 (HTTP Basic Auth) | P2 | v6 |
| storage | FR-032 | Snapshot 元数据完善 | P1 | v2 |
| storage/permission | FR-016 | Permission 审计记录 | P1 | v2 |
| lsp | FR-013 | LSP 功能增强 | P1 | v2 |
| lsp | FR-078 | LSP definition/references/hover/code actions | P2 | v6 |
| git | FR-028 | GitHub 集成预留 | P2 | v2 |
| git | FR-048 | GitHub 集成 | P1 | v4 |
| git | FR-087 | GitHub Integration (v2) | P3 | v6 |
| control-plane | FR-060 | Control Plane / ACP 协议 | P2 | v5 |
| cli | FR-061 | CLI 命令架构完善 | P2 | v5 |
| schema | FR-018 | TUI Schema 验证 | P1 | v2 |
| - | FR-027 | IDE 扩展预留 | P2 | v2 |

### 按优先级分组

| 优先级 | FR 编号 |
|--------|---------|
| P0 | FR-001, FR-002, FR-003, FR-004, FR-005, FR-006, FR-007, FR-008, FR-009, FR-010, FR-033, FR-034, **FR-063, FR-064** |
| P1 | FR-011, FR-012, FR-013, FR-014, FR-015, FR-016, FR-017, FR-018, FR-019, FR-020, FR-032, FR-035, FR-036, FR-037, FR-038, FR-039, FR-044, FR-045, FR-046, FR-047, FR-048, FR-053, FR-054, FR-055, FR-056, **FR-065, FR-066, FR-067, FR-068, FR-069, FR-070, FR-071, FR-072, FR-073** |
| P2 | FR-021, FR-022, FR-023, FR-024, FR-025, FR-026, FR-027, FR-028, FR-029, FR-030, FR-031, FR-040, FR-041, FR-042, FR-043, FR-049, FR-050, FR-051, FR-052, FR-057, FR-058, FR-059, FR-060, FR-061, FR-062, **FR-074, FR-075, FR-076, FR-077, FR-078, FR-079, FR-080, FR-081, FR-082, FR-083, FR-084, FR-085** |
| P3 | **FR-087, FR-088** |

---

## 6. v5 → v6 变更摘要

| 变更项 | 说明 |
|--------|------|
| 新增 FR-063 | Provider 认证协议分层抽象 (P0) — 架构级重构 |
| 新增 FR-064 | OAuth/Device Code 浏览器登录流程 (P0) |
| 新增 FR-065 | 云厂商原生认证 (P1) |
| 新增 FR-066 | Remote Config 自动发现 (P1) |
| 新增 FR-067 | disabled_providers 优先级 (P1) |
| 新增 FR-068 | MCP OAuth 独立 token store (P1) |
| 新增 FR-069 | TUI 三栏布局与 Inspector 面板 (P1) |
| 新增 FR-070 | TUI 状态机完整实现 (P1) |
| 新增 FR-071 | Context Engine 分层上下文 L0-L4 (P1) |
| 新增 FR-072 | Plugin WASM 运行时 (P1) |
| 新增 FR-073 | 凭证加密存储完善 (P1) |
| 新增 FR-074 | Event Bus 事件类型完整性 (P2) |
| 新增 FR-075 | Share 服务层 (P2) |
| 新增 FR-076 | SDK 输出 (Rust + TypeScript) (P2) |
| 新增 FR-077 | OpenAPI 文档自动生成 (P2) |
| 新增 FR-078 | LSP definition/references/hover/code actions (P2) |
| 新增 FR-079 | session_load/session_save 工具 (P2) |
| 新增 FR-080 | HuggingFace + AI21 Provider 完整实现 (P2) |
| 新增 FR-081 | Formatters 接入 agent 执行循环 (P2) |
| 新增 FR-082 | Compaction 自动触发阈值 (P2) |
| 新增 FR-083 | TUI 虚拟滚动 (P2) |
| 新增 FR-084 | Server 认证 HTTP Basic Auth (P2) |
| 新增 FR-085 | 观测性 tracing/crash recovery/token cost (P2) |
| 新增 FR-087 | GitHub Integration v2 (P3) |
| 新增 FR-088 | Enterprise 配置 Central Config + SSO (P3) |
| 更新 §1 | 文档概述 (新增 26 项 FR 说明) |
| 更新 §2 | P0 需求 (新增 FR-063, FR-064) |
| 更新 §3 | P1 需求 (新增 FR-065 ~ FR-073) |
| 更新 §4 | P2 需求 (新增 FR-074 ~ FR-085) |
| 更新 §5 | 功能需求清单汇总 (88 项 FR, 按模块/优先级分组) |

---

**文档状态**: 草稿
**下一步**: 基于本任务清单 + plan_v6.md 创建迭代 6 实施计划
