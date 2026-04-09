下面给你一份可直接用于 **AI Coding 落地 Rust 版** 的完整设计文档。
我先说明一个前提：这里的 **OpenCode** 按当前公开资料，指的是 **opencode.ai 的开源 AI coding agent**，其现有产品形态包括终端 TUI、桌面端、IDE 扩展，以及本地 server / web / SDK 能力；公开文档还显示它支持 agents、skills、plugins、MCP、LSP、share、GitHub 集成等。与此同时，早期公开仓库 `opencode-ai/opencode` 已归档，并说明项目后续迁移到 **Crush**，所以这份文档的定位是：**以当前公开产品能力为基线，设计一个可工程化实现的 Rust 重构版，而不是对旧仓库逐文件翻译。** ([opencode.ai][1])

---

# OpenCode Rust 版产品设计文档

**代号：OpenCode-RS**
**文档类型：PRD + 系统设计草案 + AI Coding 拆解说明**
**目标：作为 AI Coding 的统一输入，指导开发一个 Rust 语言实现的 OpenCode 风格 AI Coding Agent**

---

## 1. 产品定义

### 1.1 产品愿景

OpenCode-RS 是一个面向开发者的 **本地优先 AI Coding Agent Runtime**。
它不是单一聊天工具，也不是单一 IDE 插件，而是一个：

* 以 **项目目录** 为核心上下文的 AI 开发代理
* 以 **会话、工具调用、代码修改、验证反馈** 为主循环的工程系统
* 以 **TUI / Server / Web / IDE** 多前端复用同一 Runtime 的平台
* 以 **权限控制、可审计、可扩展、可私有部署** 为设计原则的开发底座

### 1.2 产品定位

Rust 版不追求“先做一个能聊天的 CLI”，而是追求：

1. **可替代日常 AI coding CLI**
2. **可作为 IDE/桌面/Web 的统一后端**
3. **可扩展到企业级使用场景**
4. **可被 AI Coding Agent 自举维护**

### 1.3 产品口号

**一个本地优先、可编排、可扩展、可审计的 Rust AI Coding Agent。**

---

## 2. 设计基线

当前公开 OpenCode 的关键基线能力包括：

* 默认入口是 **TUI**，但也支持命令行调用和程序化调用。([opencode.ai][2])
* 运行时采用 **TUI + Server** 架构，server 暴露 **OpenAPI 3.1**，并用于生成 SDK。([opencode.ai][3])
* 支持 **Build / Plan** 两类主 Agent，Plan 默认限制写文件和 bash。([opencode.ai][4])
* 支持 **skills、commands、plugins、MCP、LSP、permissions、session share、GitHub workflow 集成**。([opencode.ai][5])
* 当前公开产品还强调 **多会话并行、可分享 session、支持终端/桌面/IDE**。([opencode.ai][6])

因此，Rust 版的设计原则是：

**兼容“产品能力边界”，不强求兼容历史实现细节。**

---

## 3. 产品目标与边界

### 3.1 核心目标

Rust 版在 v1 必须实现：

1. 项目感知的 AI 会话
2. TUI 交互
3. Tool Calling 与权限系统
4. 文件读取 / 编辑 / 补丁 / Bash 执行
5. Session 持久化与恢复
6. 模型提供方抽象
7. 配置系统
8. Server API
9. LSP 诊断接入
10. MCP 接入
11. 自定义 Commands / Skills
12. 基础插件系统

### 3.2 非目标

v1 不强制实现：

* 完整桌面壳
* 完整 IDE 扩展
* 云端账号体系
* 商业化登录适配（如对接 GitHub Copilot / ChatGPT Plus/Pro 账号能力）
* 公共分享服务托管平台

这些能力可以在 v1.5 / v2 追加。公开站点确实宣传了 GitHub/Copilot 与 ChatGPT 账号能力，但 Rust 版初期不建议直接把商业登录兼容作为首发目标。([opencode.ai][6])

---

## 4. 目标用户

### 4.1 核心用户

* 独立开发者
* 后端工程师
* 全栈工程师
* DevOps / 平台工程师
* 需要在终端中高频完成分析、改码、执行、验证闭环的用户

### 4.2 扩展用户

* 企业内网开发团队
* 私有模型 / 企业 AI Gateway 用户
* 需要审计与权限控制的组织

---

## 5. 核心使用场景

### 5.1 场景 A：代码问答

用户进入项目目录后执行：

```bash
opencode-rs
```

在 TUI 中提问：

> 帮我梳理这个仓库的认证流程

系统自动读取仓库结构、必要文件、LSP 诊断，输出结构化分析。

### 5.2 场景 B：需求实现

用户输入：

> 给订单模块增加取消原因字段，并补齐 API、数据库迁移和测试

系统进入任务循环：
分析 → 计划 → 读文件 → 修改 → 跑测试 → 修复 → 输出结果

### 5.3 场景 C：安全规划模式

用户切换到 Plan Agent：

> 先别改代码，只做影响分析和实施方案

系统只能读和分析，不能真正改写文件或执行危险命令。

### 5.4 场景 D：外部工具扩展

用户配置 MCP：

* Jira
* GitHub
* Docs
* 内部 API

系统可在 AI 会话中调用外部工具完成跨系统协作。公开文档确认 OpenCode 已支持本地/远程 MCP。([opencode.ai][7])

### 5.5 场景 E：程序化调用

其他客户端通过 HTTP / SDK 调用 Runtime：

* 创建会话
* 发起 prompt
* 获取消息流
* 查询 diff
* 恢复/分叉 session

这与当前 OpenCode 公开的 server / SDK 模式一致。([opencode.ai][3])

---

## 6. 产品形态

Rust 版采用 **一核多端**：

### 6.1 内核

* Agent Runtime
* Tool Runtime
* Session Engine
* Context Engine
* Permission Engine
* Model Gateway
* Persistence Layer

### 6.2 客户端

* CLI
* TUI
* HTTP Server
* Web 前端（v1.5）
* IDE 插件（v2）

### 6.3 一核多端的原因

公开资料显示，当前 OpenCode 就是用 server 统一服务 TUI / IDE / web，并允许 attach 到已有 server。这个方向适合 Rust：
**一次实现 runtime，多端复用协议与状态机。** ([opencode.ai][3])

---

## 7. 功能设计

## 7.1 Workspace 与 Project 机制

### 功能要求

系统启动后必须绑定一个工作目录 `cwd`。
所有上下文、索引、权限、session 都围绕 `project root` 工作。

### 识别规则

1. 优先识别 Git 根目录
2. 否则使用当前目录
3. 生成稳定 project_id
4. 建立 project-level storage

### 设计原因

当前 OpenCode 也是以项目目录启动，并将项目数据持久化到本地存储。([opencode.ai][8])

### 关键能力

* 项目摘要缓存
* 文件树索引
* Git 状态感知
* LSP workspace 绑定
* Session 隔离

---

## 7.2 Session 会话系统

### 功能定义

Session 是一次持续的 AI coding 上下文单元，包含：

* 会话元信息
* 消息列表
* 工具调用记录
* 文件改动快照
* 汇总摘要
* fork lineage
* share 状态

### 必须支持

* 新建 session
* 继续上次 session
* 指定 session 恢复
* fork session
* abort
* summarize / compact
* revert / unrevert

这与公开 SDK / CLI 能力保持一致。([opencode.ai][2])

### 数据模型

```text
Session
- id
- project_id
- title
- agent
- model
- status
- created_at
- updated_at
- parent_session_id
- shared_id
- summary
- token_stats
- metadata
```

```text
Message
- id
- session_id
- role (system/user/assistant/tool)
- parts[]
- state
- created_at
```

### Session 状态机

```text
idle
thinking
awaiting_permission
executing_tool
streaming
applying_changes
verifying
summarizing
aborted
error
completed
```

---

## 7.3 Agent 系统

### 目标

将“Prompt + Tool Policy + Model Policy + Behavior Policy”封装成 Agent。

### v1 内置 Agent

#### 1) build

默认 Agent。
可读、可写、可执行、可调用全部允许工具。

#### 2) plan

只做分析和方案。
默认禁止：

* create file
* edit file
* patch
* bash
* 高风险外部调用

这与当前公开的 Build / Plan 模式一致。([opencode.ai][4])

### v1.1 可扩展 Agent

#### 3) review

代码审查模式，只读、可生成 review 建议。

#### 4) refactor

偏重跨文件改造与测试修复。

#### 5) debug

偏重错误归因、日志、复现、诊断。

### Agent 结构

```json
{
  "name": "build",
  "description": "Default coding agent",
  "prompt": "...",
  "tools": {
    "read": "allow",
    "edit": "allow",
    "patch": "allow",
    "bash": "ask"
  },
  "model": "openai/gpt-5.4-coding",
  "temperature": 0.2,
  "capabilities": ["plan", "code", "test", "diff"]
}
```

---

## 7.4 Tool Runtime

Tool 是整个产品的执行核心。
AI 不是直接操作系统，而是通过受控工具层完成动作。

### v1 内置工具

#### 文件工具

* `read`
* `glob`
* `grep`
* `stat`
* `write`
* `edit`
* `patch`
* `move`
* `delete`

#### Shell 工具

* `bash`

#### 项目工具

* `git_status`
* `git_diff`
* `git_log`
* `git_show`

#### 会话工具

* `todo_write`
* `summarize_session`

#### 网络工具

* `webfetch`（可选）

#### 结构化上下文工具

* `lsp_diagnostics`
* `lsp_definition`（v1.1）
* `lsp_references`（v1.1）

### 工具设计原则

1. 每个工具必须有明确 schema
2. 每次调用必须落审计日志
3. 高风险工具必须经过权限网关
4. 输出必须可流式回传
5. 错误必须结构化

### Tool 接口

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn schema(&self) -> serde_json::Value;
    async fn execute(&self, ctx: ToolContext, input: serde_json::Value) -> ToolResult;
}
```

---

## 7.5 权限系统

公开文档显示，当前 OpenCode 已支持 `allow / ask / deny` 的 permission 配置，并且可按工具配置；同时旧版文档和当前 docs 都表明，默认可较宽松，用户可手动收紧。([opencode.ai][9])

### Rust 版建议

为了兼顾兼容性与安全性，做两套 profile：

#### profile 1：compat

* read: allow
* edit/write/patch: allow
* bash: ask
* webfetch: allow
* mcp.remote: ask

#### profile 2：safe（默认）

* read: allow
* edit/write/patch: ask
* bash: ask
* delete: ask
* network: ask
* share: ask

### 交互要求

权限请求必须展示：

* 调用工具名
* 参数摘要
* 风险等级
* 是否记住本次决定
* 作用范围（仅本次 / 本 session / 本 project）

### 审计记录

```text
PermissionDecision
- id
- session_id
- tool_name
- args_hash
- decision
- scope
- user_note
- created_at
```

---

## 7.6 Context Engine

### 目标

把“项目上下文”压缩成模型可消费的、分层的、预算可控的输入。

### 上下文层次

#### L0：显式用户输入

当前 prompt、本轮附件、@file 引用、选中文本

#### L1：会话上下文

最近 N 条消息、todo、上轮工具结果、当前 diff 摘要

#### L2：项目上下文

文件树、关键入口文件、package manifest、git status、README、配置文件

#### L3：结构化辅助上下文

LSP 诊断、符号关系、测试失败摘要、MCP 查询结果

#### L4：压缩记忆

session summary / compaction summary

### 关键能力

* token budget 计算
* relevance ranking
* context compaction
* stale context 清理
* deterministic context order

当前 OpenCode 公开版本已有 skills 加载、session summarize、plugin compaction hook 等能力，说明“可控上下文编排”是产品核心之一。([opencode.ai][5])

### Rust 版策略

* 70% token 用于主要上下文
* 20% 留给工具输出
* 10% 留给响应空间
* 接近阈值时自动 compact

---

## 7.7 文件引用与命令快捷输入

公开 TUI 文档显示：

* `@` 可做文件引用
* `!` 可直接执行 shell 命令
* `/` 可执行内置或自定义命令。([opencode.ai][8])

### Rust 版设计

#### `@`

* 模糊匹配文件
* 支持多选
* 支持最近文件优先
* 引入文件内容时自动裁切与摘要

#### `!`

* 直接 shell 执行
* 输出进入会话流
* 可复用 bash tool 的权限体系

#### `/`

* 内置命令：

  * `/help`
  * `/init`
  * `/undo`
  * `/redo`
  * `/share`
  * `/agent`
  * `/model`
  * `/clear`
* 用户命令：

  * 从 `.opencode/commands/*.md` 加载
  * 从 config 加载

---

## 7.8 Commands 系统

公开文档确认当前 OpenCode 已支持用 Markdown 或 JSON 定义自定义命令，并能绑定 agent/model。([opencode.ai][10])

### Rust 版设计

命令本质是“带前置模板的 prompt 宏”。

### 结构

```yaml
---
description: Run tests with coverage
agent: build
model: default
---
运行完整测试并总结失败原因，只给出最小修复方案。
```

### 执行流程

`/test` → 解析模板 → 注入变量 → 创建用户消息 → 发送到 Session Engine

### 变量支持

* `${file}`
* `${selection}`
* `${cwd}`
* `${git_branch}`
* `${input}`

---

## 7.9 Skills 系统

公开文档显示，OpenCode 从 `.opencode/skills/<name>/SKILL.md` 等路径按需发现并加载 skills。([opencode.ai][5])

### Rust 版目标

将技能视作“可按需装载的领域知识包”。

### 技能用途

* 仓库规范
* 编码规范
* 发布流程
* 框架最佳实践
* 团队约定

### 设计要求

* 延迟加载
* 可列出技能目录
* 可根据 prompt 语义匹配技能
* 支持手动指定
* 支持全局与项目级别覆盖

### 技能结构

```text
.opencode/skills/rust-service/SKILL.md
```

### 技能元信息建议

```md
---
name: rust-service
description: Rust 后端服务实现规范
triggers:
  - axum
  - sqlx
  - migration
priority: 80
---
```

---

## 7.10 插件系统

公开文档显示，OpenCode 插件可以：

* 来自本地文件或 npm 包
* 监听 command/file/lsp/message/session/tool/tui 等事件
* 增加 custom tools。([opencode.ai][11])

### Rust 版目标

插件系统必须解耦于主进程，避免“脚本语言插件拖垮核心”。

### v1 方案

采用 **WASM 插件优先**，并保留本地进程插件接口。

#### 插件类型

1. WASM 插件
2. Sidecar 插件（子进程 RPC）
3. 内建插件

### 插件能力

* 监听事件
* 改写 prompt
* 注入 shell 环境变量
* 添加工具
* 增加 context sources
* 拦截敏感读取
* 发送通知

### 事件总线

```text
session.created
session.updated
session.compacted
message.updated
tool.execute.before
tool.execute.after
permission.asked
permission.replied
file.edited
lsp.updated
shell.env
tui.toast.show
```

### 插件 API 原则

* 不直接暴露内部数据库
* 所有宿主调用都要经过 capability 授权
* 运行崩溃不影响主 Runtime

---

## 7.11 MCP 系统

公开文档确认 OpenCode 已支持本地与远程 MCP server，并提醒 MCP 会显著增加上下文消耗。([opencode.ai][7])

### Rust 版设计

#### 目标

* 本地 MCP
* 远程 MCP
* 工具发现
* 工具 schema 缓存
* token 成本控制

### 核心策略

1. 默认禁用重型 MCP
2. 工具调用前只注入元信息，不预灌满上下文
3. 只有模型明确请求时再执行
4. 执行结果摘要优先，原始结果按需展开

### 配置

```jsonc
{
  "mcp": {
    "github": {
      "type": "remote",
      "url": "https://mcp.example.com/github",
      "enabled": false
    },
    "jira": {
      "type": "stdio",
      "command": "jira-mcp",
      "args": []
    }
  }
}
```

---

## 7.12 LSP 集成

公开文档确认当前 OpenCode 会接入 LSP，并把 diagnostics 提供给模型；旧版 README 也说明虽然底层 LSP 客户端能力更全，但给 AI 暴露出来的核心是 diagnostics。([opencode.ai][12])

### Rust 版 v1 目标

* 启动 / 连接项目对应 LSP
* 获取诊断
* 获取 symbols
* 增量刷新
* 将诊断结果输入给 Agent

### v1 能力边界

优先实现：

* diagnostics
* workspace symbols
* document symbols

v1.1 扩展：

* definition
* references
* hover
* code actions（只读建议）

### 原则

LSP 是“结构化上下文增强器”，不是替代全文检索。

---

## 7.13 模型与 Provider 抽象

公开文档显示，当前 OpenCode 使用 AI SDK / Models.dev 体系，支持大量 provider 与本地模型。([opencode.ai][13])

### Rust 版设计

不要绑定单一大模型供应商，采用 Provider Adapter：

```rust
pub trait ModelProvider {
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;
    async fn chat(&self, req: ChatRequest) -> Result<ChatStream>;
    async fn embed(&self, req: EmbedRequest) -> Result<EmbedResponse>;
}
```

### Provider 支持矩阵（v1 目标）

| Provider 类型 | 典型 endpoint / 协议形态 | 认证方式 | v1 支持策略 | 备注 |
|---|---|---|---|---|
| OpenAI compatible | REST / OpenAI-style JSON schema | `Authorization: Bearer <api_key>` | 必须支持 | 作为通用适配层，覆盖官方 OpenAI 与大量兼容网关 |
| Anthropic | REST / Claude Messages API | `x-api-key` + `anthropic-version` | 必须支持 | Header 语义与 OpenAI-compatible 不同，不能只靠 base URL 切换 |
| Gemini | REST / Gemini API | API Key | 必须支持 | 认证模型更接近“显式 key”而非 Bearer-only 适配 |
| OpenRouter | OpenAI-compatible API facade | `Authorization: Bearer <api_key>` | 必须支持 | 可额外携带站点标识 header，且可作为多模型路由层 |
| Local OpenAI-compatible endpoint | 本地或内网 REST | Bearer / 无认证 / 反向代理认证 | 必须支持 | 必须允许关闭内建鉴权并交给企业网关处理 |
| Enterprise AI Gateway | 公司内部统一网关 | API Key / Bearer / mTLS / 代理头 | 推荐支持 | 作为“OpenAI-compatible+自定义 header”的特例建模 |

### 认证协议抽象

Rust 版不能把“provider”与“auth”写死在一起，而应拆成两个层次：

1. **Provider Protocol**：请求/响应 schema、streaming 事件格式、tool calling 语义、模型枚举方式
2. **Auth Strategy**：请求如何签名、header 如何注入、token 是否过期、是否需要 refresh、是否涉及浏览器登录

建议将认证抽象为独立策略：

```rust
enum AuthStrategy {
    BearerApiKey,
    HeaderApiKey,
    QueryApiKey,
    OAuthSession,
    None,
}
```

其中：

* `BearerApiKey`：适用于 OpenAI-compatible、OpenRouter、部分企业网关
* `HeaderApiKey`：适用于 Anthropic 这类需要专有 header 的协议
* `QueryApiKey`：用于兼容少数显式 key 参数风格的 provider / gateway
* `OAuthSession`：用于后续浏览器登录兼容层，不作为 v1 首发必选能力
* `None`：用于本地无认证模型服务或已由 sidecar / reverse proxy 完成鉴权的场景

### v1 支持的 Provider

#### 1) OpenAI compatible

支持范围应包含：

* OpenAI 官方 API
* OpenAI-compatible SaaS
* 自建 gateway
* 本地 OpenAI-compatible endpoint

最低要求：

* 可配置 `base_url`
* 可配置 `Authorization: Bearer ...`
* 可覆盖默认 header
* 可声明是否支持 chat / responses / embeddings / streaming / tool calls

#### 2) Anthropic

最低要求：

* 使用独立 provider adapter
* 注入 `x-api-key`
* 注入 `anthropic-version`
* 明确其 tool use / streaming / token accounting 与 OpenAI-compatible 的差异

#### 3) Gemini

最低要求：

* 支持 Gemini API 的 API Key 模式
* 支持 endpoint/version 配置
* 不假设其认证方式一定能完全复用 OpenAI-compatible 适配器

#### 4) OpenRouter

最低要求：

* 使用 Bearer token
* 支持 OpenAI-compatible client 方式接入
* 支持可选 header（如来源站点标识）
* 支持把 OpenRouter 视为“路由 provider”，而不是单一模型厂商

#### 5) Local OpenAI-compatible endpoint

最低要求：

* 可将 `auth.strategy` 设为 `none`
* 可配置本地 endpoint、超时、重试、并发限制
* 可允许反向代理在 Runtime 之外完成鉴权

### 认证协议细化

#### API Key / Bearer 类

这是 v1 的默认路线。要求：

* Provider adapter 只接收“已解析好的 credential”，不直接读取环境变量
* header 注入在 Model Gateway 内统一完成
* 日志与 tracing 中默认脱敏，不记录完整 key
* 权限/审计系统只记录 credential 来源与 credential id，不记录明文

#### Header API Key 类

对于 Anthropic 这类协议，Runtime 不能假设所有 provider 都是 `Authorization: Bearer`。因此必须支持：

* provider-specific required headers
* provider-specific API version header
* provider-specific beta / capability header 扩展

#### Browser / OAuth Session 类（非 v1 强制）

为了兼容未来的“浏览器登录 + 本地 runtime”模式，架构上应预留：

* OAuth authorization code + PKCE
* 本地 localhost callback
* access token / refresh token / expires_at 元数据
* token refresh 与 session revoke

但这类能力应放在 **v1.5+ 或兼容层**，不应阻塞 v1 首发。原因是：

* 与商业账号体系耦合更强
* 需要浏览器交互与 callback 安全校验
* provider 条款、客户端 ID、账户权限模型都可能变化

### 模型配置

* default model
* per-agent model
* per-command override
* temperature / max_tokens / reasoning effort
* fallback chain

还应包含：

* provider-specific `base_url`
* auth strategy override
* credential reference（而非明文 credential 本体）
* default headers / extra headers
* API version / feature flag
* request timeout / retry policy / circuit breaker

### Provider 凭证对象建议

```jsonc
{
  "provider": "anthropic",
  "auth": {
    "strategy": "header_api_key",
    "credentialRef": "cred_anthropic_default",
    "headers": {
      "anthropic-version": "2023-06-01"
    }
  }
}
```

### 建议

先把 provider 抽象做好，再接多个供应商，不要把 prompt、streaming、tool schema 和单一厂商协议耦合。

同样重要的是：**不要把 credential 读取、header 拼装、token refresh 逻辑散落在每个 provider 实现里。** 这些应由统一认证层或 gateway middleware 负责。

---

## 7.14 配置系统

公开文档显示，当前 OpenCode 支持 JSON / JSONC，并且多层配置是“合并”而不是单纯覆盖。([opencode.ai][14])

### Rust 版配置来源

1. 默认内建配置
2. 全局配置
3. 项目配置
4. 环境变量
5. CLI 参数

### 合并原则

* 标量：后者覆盖
* map：递归合并
* list：按策略合并（append / replace）
* agent / command / mcp：按 key 合并

### Provider 凭证配置与优先级

对于 provider 相关配置，建议拆成：

1. **provider definition**：这个 provider 是什么协议、默认 endpoint 是什么
2. **credential binding**：这个 provider 当前绑定哪个 credential
3. **runtime override**：本次启动是否临时覆盖 endpoint / model / auth strategy

建议优先级：

1. CLI 显式传参
2. 环境变量
3. 项目配置
4. 全局配置
5. 内建默认值

其中敏感字段建议遵循：

* 配置文件中允许写 `credentialRef`，不鼓励写明文 key
* 环境变量适合注入明文密钥
* CLI 允许传 credential alias，不建议直接传完整 secret
* UI/TUI 输入的 secret 应直接进入本地 auth store，而不是回写 PRD 示例中的配置文件

建议的 provider 配置结构：

```jsonc
{
  "providers": {
    "openai": {
      "protocol": "openai-compatible",
      "baseUrl": "https://api.openai.com/v1",
      "auth": {
        "strategy": "bearer_api_key",
        "credentialRef": "cred_openai_default"
      }
    },
    "anthropic": {
      "protocol": "anthropic",
      "baseUrl": "https://api.anthropic.com",
      "auth": {
        "strategy": "header_api_key",
        "credentialRef": "cred_anthropic_default",
        "headers": {
          "anthropic-version": "2023-06-01"
        }
      }
    },
    "openrouter": {
      "protocol": "openai-compatible",
      "baseUrl": "https://openrouter.ai/api/v1",
      "auth": {
        "strategy": "bearer_api_key",
        "credentialRef": "cred_openrouter_default"
      },
      "headers": {
        "HTTP-Referer": "https://example.dev",
        "X-OpenRouter-Title": "OpenCode-RS"
      }
    }
  }
}
```

### 认证字段的环境变量约定

建议支持两类环境变量：

#### 1) 通用入口

* `OPENCODE_PROVIDER`
* `OPENCODE_MODEL`
* `OPENCODE_BASE_URL`
* `OPENCODE_API_KEY`

用于快速启动单 provider 场景。

#### 2) Provider-specific 入口

* `OPENAI_API_KEY`
* `ANTHROPIC_API_KEY`
* `GEMINI_API_KEY`
* `OPENROUTER_API_KEY`

用于长期持有多 provider 配置时的自动绑定。

### 建议路径

```text
~/.config/opencode-rs/config.jsonc
./.opencode/config.jsonc
```

### 示例

```jsonc
{
  "$schema": "https://opencode-rs.dev/config.schema.json",
  "model": "openai/gpt-5.4-coding",
  "agent": "build",
  "providers": {
    "openai": {
      "protocol": "openai-compatible",
      "baseUrl": "https://api.openai.com/v1",
      "auth": {
        "strategy": "bearer_api_key",
        "credentialRef": "cred_openai_default"
      }
    },
    "anthropic": {
      "protocol": "anthropic",
      "baseUrl": "https://api.anthropic.com",
      "auth": {
        "strategy": "header_api_key",
        "credentialRef": "cred_anthropic_default",
        "headers": {
          "anthropic-version": "2023-06-01"
        }
      }
    }
  },
  "permission": {
    "read": "allow",
    "edit": "ask",
    "patch": "ask",
    "bash": "ask"
  },
  "server": {
    "port": 4096,
    "hostname": "127.0.0.1"
  },
  "mcp": {
    "github": {
      "type": "remote",
      "url": "https://mcp.example.com/github",
      "enabled": false
    }
  }
}
```

---

## 7.15 TUI 设计

公开资料表明 TUI 是当前 OpenCode 的默认使用方式。([opencode.ai][2])

### TUI 布局

建议三栏或双栏可切换：

#### 左栏

* 会话列表
* agent / model 状态
* project 概览

#### 主区

* 消息流
* tool 调用流
* diff / plan / errors 内联展示

#### 右栏

* 文件引用
* todo
* diagnostics
* permission queue

### 输入区

支持：

* 普通 prompt
* `@file`
* `/command`
* `!shell`
* agent 切换
* model 切换

### 关键 UX

1. token / cost 显示
2. 正在思考与正在执行工具分离显示
3. patch 预览可展开
4. 权限确认不打断主流
5. 错误提示人类可读

---

## 7.16 Server / API / SDK

公开文档显示，当前 OpenCode 会启动 server，并公开 OpenAPI 3.1；SDK 也围绕 session.create / prompt / shell / share / summarize 等对象设计。([opencode.ai][3])

### Rust 版目标

Runtime 是 server-first，TUI 只是一个 client。

### API 分层

#### Session API

* `POST /sessions`
* `GET /sessions`
* `GET /sessions/{id}`
* `POST /sessions/{id}/fork`
* `POST /sessions/{id}/summarize`
* `POST /sessions/{id}/abort`

#### Message API

* `POST /sessions/{id}/prompt`
* `GET /sessions/{id}/messages`
* `GET /sessions/{id}/messages/{msg_id}`

#### Tool API

* `POST /sessions/{id}/shell`
* `POST /sessions/{id}/command`
* `POST /sessions/{id}/permissions/{req_id}/reply`

#### Artifact API

* `GET /sessions/{id}/diff`
* `GET /sessions/{id}/snapshots`
* `POST /sessions/{id}/revert`

#### Runtime API

* `GET /doc`
* `GET /health`
* `GET /providers`
* `GET /models`

### Runtime 与 Provider 认证边界

Server-first 架构下必须明确两层认证边界：

#### 1) Client → OpenCode-RS Server

这是 Runtime 自身的访问控制，解决“谁可以操作本地/远程会话”。

v1 可以支持：

* localhost only（默认）
* 本地 token / session cookie（可选）
* 反向代理统一鉴权（企业部署推荐）

这里的认证与上游 LLM provider 凭证必须隔离，不能混用。

#### 2) OpenCode-RS Server → Upstream Provider

这是 Runtime 代表用户调用上游模型时的 provider 认证层。要求：

* provider credential 默认只保存在 server 本地
* browser / IDE / TUI client 不直接持有全部 provider secret
* SDK 不默认把 provider key 回传到前端
* 可按 project / session / user 绑定默认 provider credential

### Provider 管理 API 建议

除了列出 provider / model，建议补充：

* `GET /providers`：列出 provider、连接状态、默认模型、认证策略（脱敏）
* `POST /providers/{id}/credentials`：设置或更新 credential
* `POST /providers/{id}/test`：连通性与权限测试
* `DELETE /providers/{id}/credentials`：撤销当前绑定

返回值中应避免直接返回 secret，只返回：

* provider id
* auth strategy
* 是否已配置 credential
* credential 是否过期
* 最近测试时间
* 最近错误摘要

### Browser / OAuth 登录协议预留

若未来支持“浏览器账号登录”，建议在 server 侧采用：

* authorization code + PKCE
* localhost callback 或受控 redirect URI
* access token / refresh token / expiry metadata
* logout / revoke / refresh 接口

但这类接口应标记为 **实验性或 v1.5+**，避免与 v1 的 API key 路线耦合。

### 流式协议

建议同时支持：

* SSE
* WebSocket

### SDK 输出

* Rust SDK
* TypeScript SDK

---

## 7.17 Share 能力

公开文档显示，当前 OpenCode 的 share 是把 conversation history 同步到服务端并生成公开链接，且任何持链接者都可访问。([opencode.ai][15])

### Rust 版建议

把 Share 拆成两层：

#### 本地层

* 导出 session JSON
* 导出 Markdown transcript
* 导出 patch bundle

#### 服务层（可选）

* self-hosted share server
* public share server
* 短链
* 访问令牌
* 过期时间
* 红线脱敏

### 默认策略

* 默认关闭自动分享
* 手动触发
* 明确提示“将上传对话内容”

---

## 7.18 GitHub 集成

公开文档显示，当前 OpenCode 已支持在 GitHub issue / PR 评论中通过 `/opencode` 或 `/oc` 触发任务，并在 runner 中执行。([opencode.ai][16])

### Rust 版建议

作为 v2 功能：

1. GitHub Action Runner 集成
2. issue/PR comment trigger
3. 自动新分支
4. 自动提交 patch
5. 自动创建 PR
6. 安全沙箱与密钥隔离

### 不建议放到 v1 的原因

* 安全面广
* 需要 GitHub API 适配
* Runner/checkout/patch/PR 流程复杂
* 与核心 Runtime 无强耦合

---

## 8. 非功能设计

## 8.1 性能

### 要求

* TUI 首屏启动 < 500ms（冷启动不含 LSP）
* 首条响应开始流式输出 < 2s（不含模型本身网络延迟）
* 10k 消息 session 可分页加载
* 工具结果可增量流式返回

## 8.2 可靠性

* 插件崩溃不拖垮主进程
* LSP 断开可自动重连
* provider 超时可重试或 fallback
* Session 每次消息提交前后都要持久化 checkpoint

## 8.3 安全

* 默认不读取 `.env` 等敏感文件，除非用户明确允许
* shell 执行保留审计
* 远程 MCP 默认 ask
* 分享前脱敏检查
* 凭证本地加密存储
* provider secret、access token、refresh token 不进入普通日志与 trace payload
* browser/OAuth callback 必须校验 `state`，并优先使用 PKCE
* credential rotation / revoke 后应立即使旧会话失效或进入重验状态
* provider 凭证与 session transcript 必须物理隔离，避免导出 share 时误带 secret

## 8.4 可观测

* 结构化日志
* session traces
* tool spans
* provider latency / token / cost 统计
* 崩溃转储

---

## 9. 系统架构设计

## 9.1 总体架构

```text
CLI / TUI / Web / IDE
        │
        ▼
   OpenCode-RS Server
        │
 ┌──────┼────────┬─────────┬─────────┐
 ▼      ▼        ▼         ▼         ▼
Session Agent   Tools     LSP       MCP
Engine  Runtime Runtime   Bridge    Bridge
        │
        ▼
 Context Engine
        │
        ▼
 Model Gateway
        │
        ▼
 Persistence (SQLite + FS)
```

## 9.2 核心模块职责

### Session Engine

管理消息、状态、分叉、恢复、摘要、快照。

### Agent Runtime

负责 prompt assembling、tool plan、response parsing、执行循环。

### Tool Runtime

统一调度本地工具、LSP 工具、MCP 工具、插件工具。

### Context Engine

基于 token budget 选择上下文。

### Model Gateway

屏蔽不同大模型协议差异。

### Persistence

管理 SQLite 元数据与文件系统对象。

---

## 10. Rust 工程结构建议

```text
opencode-rs/
  Cargo.toml
  crates/
    opencode-core/         # 核心领域模型
    opencode-config/       # 配置加载与合并
    opencode-session/      # session/message/snapshot
    opencode-agent/        # agent runtime
    opencode-tools/        # built-in tools
    opencode-permission/   # permission engine
    opencode-context/      # context ranking / compaction
    opencode-model/        # provider adapters
    opencode-lsp/          # lsp bridge
    opencode-mcp/          # mcp bridge
    opencode-plugin/       # wasm/sidecar plugin host
    opencode-storage/      # sqlite/fs persistence
    opencode-server/       # http/ws/sse api
    opencode-tui/          # ratatui client
    opencode-cli/          # clap entry
    opencode-web/          # optional web shell
```

### 技术选型建议

这是建议，不是强约束：

* async runtime：`tokio`
* HTTP：`axum`
* TUI：`ratatui` + `crossterm`
* 配置：`serde` + `jsonc-parser`
* DB：`sqlx` 或 `rusqlite`
* streaming：`tokio-stream`
* FS watcher：`notify`
* patch/diff：自定义统一 diff 模块
* plugin：`wasmtime`
* tracing：`tracing` + `tracing-subscriber`

---

## 11. 持久化设计

## 11.1 存储策略

### SQLite

存：

* project
* session
* message
* tool call
* permissions
* snapshot metadata
* provider stats

### 文件系统

存：

* transcript export
* patch bundle
* snapshots
* temp artifacts
* logs
* auth store

### auth store 建议结构

`auth store` 不应只是“provider -> api_key”的平面映射，而应保存：

* credential id
* provider id
* auth strategy
* secret ciphertext
* expires_at / refreshed_at
* scopes / account metadata（如有）
* created_at / updated_at
* revoked_at

对于 API key 类 credential，只需保存 secret 与基础元数据。
对于 OAuth/session 类 credential，需要额外保存：

* access token
* refresh token
* expiry
* account id / workspace id（若 provider 返回）

建议：

* 明文 secret 不落 SQLite 主表
* 优先使用系统密钥链或本地加密封装
* 文件权限默认收紧到当前用户
* 导出 session / share / debug bundle 时默认排除 auth store

公开文档显示当前 OpenCode 也将 session 与应用数据存到本地目录，并保留日志与认证数据。([opencode.ai][17])

## 11.2 关键表

### sessions

* id
* project_id
* title
* model
* agent
* summary
* status
* parent_id
* created_at
* updated_at

### messages

* id
* session_id
* role
* content_json
* token_in
* token_out
* tool_calls
* created_at

### tool_invocations

* id
* session_id
* message_id
* tool_name
* args_json
* result_json
* status
* latency_ms

### snapshots

* id
* session_id
* based_on_message_id
* patch_path
* created_at

---

## 12. 关键执行流程

## 12.1 Prompt 主循环

```text
用户输入
→ 解析特殊语法(@ / ! / /)
→ 选择 agent / model
→ 组装上下文
→ 发送给模型
→ 若模型发起 tool call
   → 权限检查
   → 执行工具
   → 记录结果
   → 回注上下文
   → 继续推理
→ 生成最终回复
→ 若有文件变更，生成 diff/snapshot
→ 更新 session 状态
```

## 12.2 文件修改流程

```text
分析任务
→ 读取目标文件
→ 生成 edit/patch
→ 预执行格式校验
→ 应用变更
→ 生成 unified diff
→ 可选执行 tests/lints
→ 输出修改摘要
```

## 12.3 自动摘要流程

当前公开 SDK 已有 summarize 能力，旧 README 还有 auto compact 思路。([opencode.ai][18])

Rust 版建议：

* 85%：预警
* 92%：触发 compact
* 95%：强制转入新 session continuation

---

## 13. 错误处理设计

### 错误分层

1. 用户错误
2. 配置错误
3. Provider 错误
4. Tool 错误
5. Permission 错误
6. LSP/MCP 桥接错误
7. 存储错误
8. 插件错误

### 要求

* 所有错误要有 machine code
* 同时输出人类可读文案
* 支持 retry hint
* 可归档到 session timeline

### Provider / Auth 相关错误细分

建议额外细分：

* `provider_auth_missing`
* `provider_auth_invalid`
* `provider_auth_expired`
* `provider_auth_refresh_failed`
* `provider_header_invalid`
* `provider_account_mismatch`
* `provider_endpoint_unauthorized`

错误处理策略：

* **缺少 credential**：直接阻断并提示配置
* **credential 失效/过期**：优先 refresh；失败后转人工确认
* **provider 403/401**：记录 provider 维度错误，不自动切换到无关 provider
* **header/schema 不兼容**：视为配置错误，不做盲目重试

---

## 14. 版本规划

## v0.1（MVP）

目标：可用的终端 AI coding agent

包含：

* CLI + TUI
* Session 持久化
* Build / Plan agent
* read / edit / patch / bash / git tools
* permission
* provider abstraction
* config
* basic summary
* diff/snapshot

## v0.2

目标：平台化

新增：

* server API
* SSE / WS
* SDK
* command / skills
* LSP diagnostics
* model fallback
* export/import

## v0.3

目标：扩展化

新增：

* MCP
* plugin host
* web UI
* self-hosted share
* richer context compaction

## v1.0

目标：生产可用

新增：

* observability
* crash recovery
* enterprise policy profile
* auth encryption
* remote multi-client attach
* stable plugin ABI

## v1.5+

* desktop shell
* IDE extension
* GitHub Actions integration

---

## 15. AI Coding 任务拆分

下面这个拆分方式最适合直接喂给 AI Coding：

### Epic 1：core domain

* 定义 Session / Message / ToolCall / Snapshot / Permission 数据模型
* 定义统一错误模型
* 定义 event bus

### Epic 2：config

* JSONC loader
* 多层 merge
* env override
* schema validation

### Epic 3：storage

* SQLite schema
* repository pattern
* snapshot fs layout
* log layout

### Epic 4：model gateway

* provider trait
* streaming response
* tool call normalization
* token accounting

### Epic 5：tools

* read/glob/grep
* edit/write/patch
* bash
* git status/diff

### Epic 6：permission

* allow/ask/deny
* scope memory
* approval queue

### Epic 7：session engine

* create/list/get/fork/abort
* append message
* tool loop
* summarize

### Epic 8：tui

* message view
* input composer
* permission modal
* diff panel
* session sidebar

### Epic 9：server

* REST
* SSE
* OpenAPI
* SDK generation hooks

### Epic 10：context engine

* file ranking
* token budget
* summary/compaction

### Epic 11：lsp

* workspace startup
* diagnostics
* symbols

### Epic 12：mcp

* stdio bridge
* remote bridge
* schema cache
* permission integration

### Epic 13：commands/skills

* loader
* parser
* template expansion
* agent integration

### Epic 14：plugins

* event hooks
* wasm runtime
* custom tools
* sandbox

---

## 16. 验收标准

### MVP 验收

满足以下条件即可认为 Rust 版 MVP 成功：

1. 能在任意 Git 项目中启动 TUI
2. 能创建、恢复、分叉 session
3. 能通过模型完成读文件 → 改文件 → 执行测试 → 输出 diff 的闭环
4. Plan 模式确实不能直接改代码
5. 权限确认可用
6. 本地持久化稳定
7. 异常退出后 session 不损坏
8. 可通过 HTTP API 发 prompt 并拿到流式结果

### v1 验收

1. 支持 LSP diagnostics
2. 支持至少一种 MCP
3. 支持 commands / skills / plugins
4. 支持导出 session
5. 提供完整 OpenAPI 文档
6. 在 Linux/macOS/Windows 三平台基础可运行

---

## 17. 风险与决策

### 风险 1：过早做多前端

解决：先 server-first，但只正式交付 CLI/TUI。

### 风险 2：Provider 协议差异过大

解决：先统一抽象到 ChatRequest / ToolCall / StreamEvent。

### 风险 3：插件系统污染内核

解决：v1 只开 WASM/sidecar，不允许任意脚本直接跑进主进程。

### 风险 4：MCP 上下文膨胀

解决：工具元信息延迟注入，结果强制摘要。

### 风险 5：编辑质量不稳定

解决：优先 patch-based edit；大文件修改前先分块。

### 风险 6：安全问题

解决：safe profile 默认 ask；敏感文件和远程能力受限。

---

## 18. 实施建议

最合理的开发顺序是：

**先做 Runtime，再做 UI；先做本地能力，再做生态扩展。**

具体顺序建议：

1. core + config + storage
2. model gateway
3. tools + permission
4. session engine
5. TUI
6. server
7. LSP
8. commands / skills
9. MCP
10. plugin

---

## 19. 最终结论

这份文档定义的不是“Rust 版聊天 CLI”，而是一个完整的 **OpenCode-RS 平台**：

* 以 **Session** 为核心
* 以 **Agent + Tool + Permission** 为执行主轴
* 以 **Context Engine** 控制质量与成本
* 以 **Server-first** 支撑 TUI / Web / IDE
* 以 **Skills / Commands / Plugins / MCP / LSP** 形成扩展生态

如果你要基于 AI Coding 直接开工，最先应冻结的不是 UI，而是这四件事：

**领域模型、Tool 协议、Session 状态机、Provider 抽象。**

这四个一旦稳定，Rust 版就能持续自举开发。

我建议把这个文档直接拆成 14 个 epics，按上面的顺序让 AI Coding 逐个生成 crate。

[1]: https://opencode.ai/docs/ "Intro | AI coding agent built for the terminal"
[2]: https://opencode.ai/docs/cli/ "CLI | OpenCode"
[3]: https://opencode.ai/docs/server/ "Server | OpenCode"
[4]: https://opencode.ai/docs/agents/ "Agents | OpenCode"
[5]: https://opencode.ai/docs/skills/ "Agent Skills | OpenCode"
[6]: https://opencode.ai/ "OpenCode | The open source AI coding agent"
[7]: https://opencode.ai/docs/mcp-servers/ "MCP servers | OpenCode"
[8]: https://opencode.ai/docs/tui/ "TUI | OpenCode"
[9]: https://opencode.ai/docs/permissions/ "Permissions | OpenCode"
[10]: https://opencode.ai/docs/commands/ "Commands | OpenCode"
[11]: https://opencode.ai/docs/plugins/ "Plugins | OpenCode"
[12]: https://opencode.ai/docs/lsp/ "LSP Servers | OpenCode"
[13]: https://opencode.ai/docs/models/ "Models | OpenCode"
[14]: https://opencode.ai/docs/config/ "Config | OpenCode"
[15]: https://opencode.ai/docs/share/ "Share | OpenCode"
[16]: https://opencode.ai/docs/github/ "GitHub | OpenCode"
[17]: https://opencode.ai/docs/troubleshooting/ "Troubleshooting | OpenCode"
[18]: https://opencode.ai/docs/sdk/ "SDK | OpenCode"
