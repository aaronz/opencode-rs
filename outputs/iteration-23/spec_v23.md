# OpenCode-RS 规格文档 v2.3

**版本：** 2.3 (Iteration-23)  
**日期：** 2026年4月8日  
**作者：** mycode 产品团队  
**状态：** 已发布

---

## 变更日志

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| 2.3 | 2026-04-08 | 基于 iteration-23 差距分析全面更新，新增 SDK、安全、观测性需求 |
| 2.2 | 2026-04-08 | 基于差距分析更新，新增 FR-XXX 编号体系，标记缺失功能 |
| 2.1 | 2026-04-07 | Rust Edition PRD，Ratatui 架构详细定义 |
| 2.0 | 2026-04-01 | 重大架构重构，引入 Ratatui |

---

## 1. 产品概述

### 1.1 产品定位

mycode TUI（Terminal User Interface，终端用户界面）是 mycode 提供的一个交互式终端界面，旨在配合大语言模型（LLM）处理项目代码。TUI 为开发者提供了一种在终端环境中与 AI 进行协作的高效方式，无需切换到 Web 界面即可完成代码分析、修改和协作任务。

TUI 基于 **Rust** 生态系统构建，使用 **Ratatui** 作为核心 TUI 库，享受 Rust 带来的安全性、高性能和零成本抽象。

### 1.2 核心价值

- **终端原生体验**：为习惯命令行工作的开发者提供无缝的 AI 协作体验
- **文件智能引用**：通过 `@` 语法快速引用项目文件
- **Shell 命令集成**：支持直接执行 Bash 命令并获取输出
- **会话管理**：提供完整的会话历史、撤销/重做功能
- **协作分享**：支持会话分享，方便团队协作和代码审查
- **高性能渲染**：基于 Rust + Ratatui，享受安全、高性能的终端渲染体验
- **程序化集成**：提供 Rust/TypeScript SDK，支持自动化和脚本场景

### 1.3 目标用户

- 习惯使用终端的开发者和运维工程师
- 需要高效代码协作的团队
- 喜欢使用键盘而非鼠标的效率导向用户
- 需要程序化集成 mycode 能力的开发者

---

## 2. 技术架构

### 2.1 核心架构概览

```
┌─────────────────────────────────────────────────────────┐
│                    mycode TUI 应用层                    │
│  (会话管理、命令处理、LLM集成、协作分享)                   │
├─────────────────────────────────────────────────────────┤
│                    Rust TUI 渲染层                        │
│  (Ratatui + 自定义组件)                                  │
├─────────────────────────────────────────────────────────┤
│                    Crossterm 终端层                       │
│  (跨平台终端控制、ANSI 渲染、输入捕获)                    │
├─────────────────────────────────────────────────────────┤
│                    Tokio 异步运行时                       │
│  (异步任务调度、并发处理)                                 │
└─────────────────────────────────────────────────────────┘
```

### 2.2 Crate 结构

| Crate | 职责 | 状态 | 进度 |
|-------|------|------|------|
| `opencode-core` | 核心功能（session, tool, error, bus） | ✅ | 95% |
| `opencode-cli` | CLI 入口点 | ✅ | 95% |
| `opencode-llm` | LLM 提供商抽象（20+ providers） | ✅ | 95% |
| `opencode-tools` | 工具实现（35/37 tools） | ✅ | 95% |
| `opencode-tui` | Ratatui TUI 实现 | ✅ | 90% |
| `opencode-agent` | Agent 实现（7 agents） | ✅ | 100% |
| `opencode-lsp` | LSP 集成 | ⚠️ | 75% |
| `opencode-storage` | SQLite 持久化 | ✅ | 100% |
| `opencode-server` | REST/WebSocket/SSE API | ⚠️ | 70% |
| `opencode-permission` | 权限系统 | ✅ | 90% |
| `opencode-auth` | 认证凭证管理 | ⚠️ | 70% |
| `opencode-control-plane` | Control Plane 客户端 | ✅ | 100% |
| `opencode-plugin` | WASM 插件运行时 | ⚠️ | 60% |
| `opencode-git` | Git 操作 | ✅ | 90% |
| `opencode-mcp` | MCP 协议实现 | ⚠️ | 65% |
| `opencode-sdk` | Rust/TypeScript SDK | ❌ | 0% |

---

## 3. 功能需求与验收标准

### 3.1 核心功能需求

#### 3.1.1 会话系统

| FR-ID | 功能 | 描述 | 优先级 | 状态 |
|-------|------|------|--------|------|
| FR-001 | 会话创建 | 创建新会话，支持目录初始化 | P0 | ✅ Done |
| FR-002 | 会话列表 | 列出所有会话 | P0 | ✅ Done |
| FR-003 | 会话获取 | 根据 ID 获取会话详情 | P0 | ✅ Done |
| FR-004 | 会话 Fork | Fork 现有会话创建分支 | P0 | ✅ Done |
| FR-005 | 会话中止 | 中止正在执行的会话 | P0 | ✅ Done |
| FR-006 | 会话压缩 | Summarize/compact 压缩会话 | P0 | ✅ Done |
| FR-007 | 会话持久化 | SQLite 存储会话数据 | P0 | ✅ Done |
| FR-008 | **session_load** | 从存储加载会话 | P0 | ⚠️ Pending |
| FR-009 | **session_save** | 保存会话到存储 | P0 | ⚠️ Pending |
| **FR-220** | **Session Fork Lineage** | parent_session_id 追溯，支持完整 fork 历史 | P1 | ❌ Missing |
| **FR-221** | **Session Lineage Path** | session 表增加 lineage_path 字段 | P1 | ❌ Missing |

#### 3.1.2 SDK 需求 (P0 阻断性)

| FR-ID | 功能 | 描述 | 优先级 | 状态 |
|-------|------|------|--------|------|
| **FR-222** | **Rust SDK** | opencode-sdk crate，实现程序化调用 | P0 | ❌ Missing |
| **FR-223** | **TypeScript SDK** | @opencode/sdk，实现 npm 包发布 | P0 | ❌ Missing |

#### 3.1.3 Agent 系统

| FR-ID | Agent | 描述 | 权限级别 | 状态 |
|-------|-------|------|----------|------|
| FR-010 | BuildAgent | 完全访问，执行所有操作 | Full | ✅ Done |
| FR-011 | PlanAgent | 只读，规划和分析 | ReadOnly | ✅ Done |
| FR-012 | GeneralAgent | 通用对话 | Restricted | ✅ Done |
| FR-013 | ExploreAgent | 代码库探索 | ReadOnly | ✅ Done |
| FR-014 | CompactionAgent | 会话压缩 | Restricted | ✅ Done |
| FR-015 | TitleAgent | 生成会话标题 | Restricted | ✅ Done |
| FR-016 | SummaryAgent | 生成会话摘要 | Restricted | ✅ Done |
| FR-017 | ReviewAgent | 代码审查 | ReadOnly | ✅ Done |
| FR-018 | RefactorAgent | 代码重构 | Full | ✅ Done |
| FR-019 | DebugAgent | 调试辅助 | Full | ✅ Done |

#### 3.1.4 工具系统

| FR-ID | 工具 | 描述 | 权限级别 | 状态 |
|-------|------|------|----------|------|
| FR-020 | read | 读取文件内容 | ReadOnly | ✅ Done |
| FR-021 | write | 写入文件 | Full | ✅ Done |
| FR-022 | edit | 编辑文件 | Full | ✅ Done |
| FR-023 | glob | 文件模式匹配 | ReadOnly | ✅ Done |
| FR-024 | grep | 内容搜索 | ReadOnly | ✅ Done |
| FR-025 | apply_patch | 应用补丁 | Full | ✅ Done |
| FR-026 | bash | 执行 Shell 命令 | Full | ✅ Done |
| FR-027 | git_status | Git 状态 | ReadOnly | ✅ Done |
| FR-028 | git_diff | Git 差异 | ReadOnly | ✅ Done |
| FR-029 | git_log | Git 日志 | ReadOnly | ✅ Done |
| FR-030 | git_show | Git 显示 | ReadOnly | ✅ Done |
| FR-031 | webfetch | 获取网页内容 | ReadOnly | ✅ Done |
| FR-032 | lsp_diagnostics | LSP 诊断 | ReadOnly | ✅ Done |
| FR-033 | todowrite | 任务管理 | Restricted | ✅ Done |
| **FR-224** | **move** | 移动文件/目录 | Full | ❌ Missing |
| **FR-225** | **delete** | 删除文件/目录 | Full | ❌ Missing |

#### 3.1.5 LLM 提供商

| FR-ID | 提供商 | 状态 | 备注 |
|-------|--------|------|------|
| FR-040 | OpenAI | ✅ Done | GPT-4o, GPT-4 Turbo |
| FR-041 | Anthropic | ✅ Done | Claude 3.5, Claude 3 |
| FR-042 | Google Gemini | ✅ Done | Vertex 支持 |
| FR-043 | Ollama | ✅ Done | 本地模型 |
| FR-044 | Azure OpenAI | ✅ Done | 企业支持 |
| FR-045 | AWS Bedrock | ✅ Done | 企业支持 |
| FR-046 | OpenRouter | ✅ Done | 多模型路由 |
| FR-047 | Cohere | ✅ Done | Command R+ |
| FR-048 | Mistral | ✅ Done | Mistral Large |
| FR-049 | Groq | ✅ Done | 快速推理 |
| FR-050 | Perplexity | ✅ Done | 在线模型 |
| FR-051 | DeepSeek | ✅ Done | DeepSeek Coder |
| FR-052 | Fireworks | ✅ Done | Mixtral |
| FR-053 | AMD Seginfo | ✅ Done | AMD GPU 支持 |
| FR-054 | Replicate | ✅ Done | 开源模型 |
| FR-055 | HuggingFace | ✅ Done | 已实现 |
| FR-056 | AI21 | ✅ Done | Jurassic-2 |
| FR-057 | xAI | ✅ Done | Grok |
| FR-058 | Vercel | ✅ Done | v0 |
| FR-059 | SAP AI Core | ✅ Done | 企业支持 |
| FR-060 | GitHub Copilot | ✅ Done | 企业支持 |

#### 3.1.6 权限系统

| FR-ID | 功能 | 描述 | 状态 |
|-------|------|------|------|
| FR-070 | Allow | 自动批准工具执行 | ✅ Done |
| FR-071 | Ask | 请求用户确认 | ✅ Done |
| FR-072 | Deny | 拒绝工具执行 | ✅ Done |
| FR-073 | ReadOnly 范围 | 只读工具自动批准 | ✅ Done |
| FR-074 | Restricted 范围 | 安全工具需要确认 | ✅ Done |
| FR-075 | Full 范围 | 所有工具可用 | ✅ Done |
| FR-076 | 权限队列 | 待批准操作队列 | ✅ Done |
| **FR-226** | **敏感文件默认拒绝** | .env 等敏感文件默认 deny | P1 | ❌ Missing |
| **FR-227** | **external_directory 拦截** | permission external_directory 检查 | P1 | ❌ Missing |
| **FR-228** | **远程 MCP ask 严格实施** | 配置存在但执行层面检查缺失 | P1 | ❌ Missing |
| **FR-229** | **凭证明文存储加密** | credential ciphertext 加密存储 | P1 | ❌ Missing |

---

### 3.2 TUI 功能需求

#### 3.2.1 启动与基础交互

| FR-ID | 功能 | 描述 | 优先级 | 状态 |
|-------|------|------|--------|------|
| FR-080 | TUI 启动 | 运行 `mycode` 启动 TUI | P0 | ✅ Done |
| FR-081 | 目录指定 | `mycode /path` 指定目录 | P0 | ✅ Done |
| FR-082 | 消息输入 | 输入消息进行提示 | P0 | ✅ Done |
| FR-083 | AI 响应渲染 | Ratatui 组件渲染 | P0 | ✅ Done |
| FR-084 | 打字机效果 | 流式输出动画 | P1 | ⚠️ Partial |

#### 3.2.2 文件引用功能

| FR-ID | 功能 | 描述 | 优先级 | 状态 |
|-------|------|------|--------|------|
| FR-090 | `@` 文件引用 | 消息中引用文件 | P0 | ✅ Done |
| FR-091 | 模糊搜索 | 文件名模糊匹配 | P0 | ✅ Done |
| FR-092 | 自动加载 | 引用文件自动加入上下文 | P0 | ✅ Done |
| FR-093 | 列表选择 | 搜索结果 TUI 列表展示 | P0 | ✅ Done |
| **FR-230** | **@ 多选功能** | 支持同时选择多个文件 | P2 | ❌ Missing |

#### 3.2.3 Bash 命令执行

| FR-ID | 功能 | 描述 | 优先级 | 状态 |
|-------|------|------|--------|------|
| FR-100 | `!` 命令执行 | Shell 命令执行 | P0 | ✅ Done |
| FR-101 | 输出集成 | 命令结果加入对话 | P0 | ✅ Done |
| FR-102 | Ctrl+C 终止 | 终止正在执行的命令 | P1 | ✅ Done |

#### 3.2.4 斜杠命令

| FR-ID | 命令 | 描述 | 快捷键 | 状态 |
|-------|------|------|--------|------|
| FR-110 | `/connect` | 添加提供商 | - | ✅ Done |
| FR-111 | `/compact` | 压缩会话 | `ctrl+x c` | ✅ Done |
| FR-112 | `/details` | 切换详情显示 | `ctrl+x d` | ✅ Done |
| FR-113 | `/editor` | 外部编辑器 | `ctrl+x e` | ✅ Done |
| FR-114 | `/exit` | 退出 | `ctrl+x q` | ✅ Done |
| FR-115 | `/export` | 导出 Markdown | `ctrl+x x` | ✅ Done |
| FR-116 | `/help` | 帮助对话框 | `ctrl+x h` | ✅ Done |
| FR-117 | `/init` | 创建 AGENTS.md | `ctrl+x i` | ✅ Done |
| FR-118 | `/models` | 列出模型 | `ctrl+x m` | ✅ Done |
| FR-119 | `/new` | 新会话 | `ctrl+x n` | ✅ Done |
| FR-120 | `/redo` | 重做 | `ctrl+x r` | ✅ Done |
| FR-121 | `/sessions` | 会话列表/切换 | `ctrl+x l` | ✅ Done |
| FR-122 | `/share` | 分享会话 | `ctrl+x s` | ✅ Done |
| FR-123 | `/themes` | 主题列表 | `ctrl+x t` | ✅ Done |
| FR-124 | `/thinking` | 切换思考块 | - | ✅ Done |
| FR-125 | `/undo` | 撤销 | `ctrl+x u` | ✅ Done |
| FR-126 | `/unshare` | 取消分享 | - | ✅ Done |
| FR-127 | `/status` | 显示会话状态 | - | ✅ Done |
| FR-128 | `/cost` | 显示 Token 使用量 | - | ✅ Done |
| FR-129 | `/permissions` | 切换权限模式 | - | ✅ Done |
| FR-130 | `/diff` | 显示 Git 差异 | - | ✅ Done |
| FR-131 | `/version` | 显示版本信息 | - | ✅ Done |
| FR-132 | `/config` | 查看配置 | - | ✅ Done |
| FR-133 | `/search` | 搜索对话历史 | - | ⚠️ Pending |

#### 3.2.5 TUI 组件

| FR-ID | 组件 | 描述 | Ratatui 基础 | 状态 |
|-------|------|------|--------------|------|
| FR-140 | MessageBubble | 消息气泡 | Paragraph + Block | ✅ Done |
| FR-141 | CodeBlock | 代码块 | Paragraph + 样式 | ✅ Done |
| FR-142 | FilePicker | 文件选择器 | List + StatefulList | ✅ Done |
| FR-143 | CommandPalette | 命令面板 | List + Input | ✅ Done |
| FR-144 | ThinkingIndicator | 思考指示器 | Spinner | ✅ Done |
| FR-145 | ToolDetail | 工具详情 | 可折叠面板 | ✅ Done |
| FR-146 | SessionList | 会话列表 | Table | ✅ Done |
| FR-147 | StatusBar | 状态栏 | 底部栏 | ✅ Done |
| **FR-231** | **Token Budget Panel** | Context Panel 显示 Token 预算 | P1 | ❌ Missing |
| FR-148 | Todo Panel | 待办面板 | List | ⚠️ Partial |
| **FR-232** | **Diff Panel 可展开** | 差异面板增强可展开功能 | P2 | ❌ Missing |
| **FR-233** | **Token/Cost 显示** | 状态栏准确显示 token/cost | P2 | ⚠️ Partial |

#### 3.2.6 布局系统

| FR-ID | 布局 | 描述 | 状态 |
|-------|------|------|------|
| FR-160 | 单列布局 | 单一对话列 | ✅ Done |
| FR-161 | 双列布局 | 对话 + 侧边栏 | ✅ Done |
| FR-162 | 三列布局 | 对话 + 工具 + 文件 | ✅ Done |
| FR-163 | 响应式 | 终端宽度自适应 | ✅ Done |

#### 3.2.7 事件系统

| FR-ID | 组件 | 描述 | 状态 |
|-------|------|------|------|
| FR-170 | EventBus | 事件总线 | ✅ Done |
| FR-171 | TuiEvent | TUI 事件类型 | ✅ Done |
| FR-172 | Server events | 服务器事件 | ✅ Done |

---

### 3.3 服务器 API 需求

| FR-ID | 端点 | 描述 | 状态 |
|-------|------|------|------|
| FR-180 | REST /sessions | 会话 CRUD | ✅ Done |
| FR-181 | REST /providers | 提供商 API | ✅ Done |
| FR-182 | REST /models | 模型列表 | ✅ Done |
| FR-183 | REST /config | 配置端点 | ✅ Done |
| FR-184 | WebSocket /ws | WebSocket 连接 | ✅ Done |
| FR-185 | SSE /sse | Server-Sent Events | ⚠️ Partial |
| FR-186 | MCP Protocol | MCP 协议实现 | ✅ Done |
| **FR-234** | **SSE Heartbeat** | 心跳机制，客户端重连处理 | P1 | ❌ Missing |
| **FR-235** | **WebSocket Handshake Fix** | 修复连接不稳定问题 | P1 | ❌ Missing |
| FR-187 | OpenAPI 3.1 | API 文档 | ⚠️ Partial |

---

### 3.4 LSP 集成

| FR-ID | 功能 | 描述 | 状态 |
|-------|------|------|------|
| FR-190 | LSP diagnostics | 诊断功能 | ✅ Done |
| FR-191 | LSP workspace | 工作区支持 | ✅ Done |
| FR-192 | LSP symbols | 符号搜索 | ⚠️ Partial |
| FR-193 | Incremental diagnostics | 增量诊断更新 | ✅ Done |
| **FR-236** | **LSP definition** | 跳转定义 | P1 | ❌ Missing |
| **FR-237** | **LSP references** | 查找引用 | P1 | ❌ Missing |
| **FR-238** | **LSP hover** | 悬停信息 | P2 | ❌ Missing |
| **FR-239** | **LSP code_actions** | 代码动作 | P2 | ❌ Missing |

---

### 3.5 MCP 集成

| FR-ID | 功能 | 描述 | 状态 |
|-------|------|------|------|
| FR-200 | MCP stdio bridge | 标准输入输出桥接 | ✅ Done |
| FR-201 | MCP remote bridge | 远程 MCP 连接 | ⚠️ Partial |
| FR-202 | MCP tool discovery | 工具发现 | ✅ Done |
| **FR-240** | **MCP Connection Pooling** | 连接池、timeout、重试机制 | P1 | ❌ Missing |
| **FR-241** | **MCP OAuth** | OAuth 认证完整流程 | P2 | ❌ Missing |

---

### 3.6 Skills 系统

| FR-ID | Skill | 描述 | 状态 |
|-------|-------|------|------|
| FR-210 | Skill Registry | 技能注册表 | ✅ Done |
| FR-211 | Command Registry | 命令注册表 | ✅ Done |
| FR-212 | TUI Commands | TUI 命令支持 | ✅ Done |
| FR-213 | Custom commands | 自定义命令 | ✅ Done |
| FR-214 | Skill matching | 语义匹配 | ✅ Done |
| FR-215 | Global/project skills | 覆盖支持 | ✅ Done |
| FR-216 | Built-in Skills | 内置技能 | ⚠️ Partial |
| **FR-217** | **OAuth login** | 浏览器认证 | P2 | ⚠️ Pending |

---

### 3.7 插件系统

| FR-ID | 功能 | 描述 | 状态 |
|-------|------|------|------|
| FR-220 | WASM runtime | WASM 运行时 | ⚠️ Partial |
| FR-221 | Sidecar plugins | Sidecar 插件 | ⚠️ Partial |
| FR-222 | Event hooks | 事件钩子 | ⚠️ Partial |
| FR-223 | Custom tools | 插件工具注册 | ✅ Done |
| **FR-242** | **Plugin Event Bus** | session.created 等事件总线 | P1 | ❌ Missing |
| **FR-243** | **WASM Sandbox Isolation** | crash 不影响主 Runtime | P1 | ❌ Missing |
| **FR-244** | **Plugin ABI** | ABI 稳定性 | P2 | ⚠️ Pending |

---

### 3.8 Share 功能

| FR-ID | 功能 | 描述 | 状态 |
|-------|------|------|------|
| FR-250 | Share Session | 分享当前会话 | ✅ Done |
| FR-251 | Unshare Session | 取消分享 | ✅ Done |
| **FR-252** | **JSON Export** | 导出会话为 JSON | P1 | ❌ Missing |
| **FR-253** | **Markdown Export** | 导出会话为 Markdown | P1 | ❌ Missing |
| **FR-254** | **Patch Bundle** | 导出 patch bundle | P1 | ❌ Missing |
| **FR-255** | **Self-hosted Share Server** | 短链/访问令牌/过期时间 | P2 | ❌ Missing |

---

### 3.9 Context Engine

| FR-ID | 功能 | 描述 | 状态 |
|-------|------|------|------|
| FR-260 | Token Budget | Token 预算管理 | ⚠️ Partial |
| FR-261 | Context Compaction | 会话压缩 | ⚠️ Partial |
| FR-262 | Context Ranking | 上下文 ranking | ⚠️ Partial |
| **FR-256** | **Token Budget Calibration** | tiktoken-rs 计数校准 | P1 | ❌ Missing |
| **FR-257** | **Compaction Thresholds** | 85%/92%/95% 阈值精确触发 | P1 | ❌ Missing |
| **FR-258** | **Summary Quality** | 压缩记忆质量提升 | P1 | ⚠️ Partial |

---

### 3.10 错误处理

| FR-ID | 功能 | 描述 | 状态 |
|-------|------|------|------|
| FR-270 | Error Enum | thiserror 错误枚举 | ✅ Done |
| FR-271 | Error Codes | 错误代码体系 | ⚠️ Partial |
| **FR-259** | **Error Code 1xxx** | Authentication errors | P1 | ❌ Missing |
| **FR-260** | **Error Code 2xxx** | Authorization errors | P1 | ❌ Missing |
| **FR-261** | **Error Code 3xxx** | Provider errors | P1 | ❌ Missing |
| **FR-262** | **Error Code 4xxx** | Tool errors | P1 | ❌ Missing |
| **FR-263** | **Error Code 5xxx** | Session errors | P1 | ❌ Missing |
| **FR-264** | **Error Code 6xxx** | Config errors | P1 | ❌ Missing |
| **FR-265** | **Error Code 7xxx** | Validation errors (含 provider_header_invalid) | P2 | ❌ Missing |
| **FR-266** | **Error Code 9xxx** | Internal errors | P1 | ❌ Missing |

---

### 3.11 可观测性

| FR-ID | 功能 | 描述 | 状态 |
|-------|------|------|------|
| FR-280 | Tracing Integration | tracing 集成 | ⚠️ Partial |
| **FR-267** | **Session Traces** | 完整 session traces | P2 | ❌ Missing |
| **FR-268** | **Tool Spans** | 工具调用 spans | P2 | ❌ Missing |
| **FR-269** | **Cost Calculator** | Provider latency/token/cost 统计 | P2 | ⚠️ Partial |
| **FR-270** | **Crash Recovery** | 崩溃转储机制 | P1 | ❌ Missing |

---

### 3.12 安全功能

| FR-ID | 功能 | 描述 | 状态 |
|-------|------|------|------|
| FR-290 | Permission System | 权限系统 | ✅ Done |
| FR-291 | Audit Log | 审计日志 | ✅ Done |
| FR-292 | API Key Auth | API Key 认证 | ✅ Done |
| **FR-271** | **Credential Encryption** | 凭据加密存储 | P1 | ❌ Missing |
| **FR-272** | **PKCE Support** | OAuth callback state 校验 | P2 | ❌ Missing |
| **FR-273** | **Token Refresh/Revoke** | 完整的 token refresh 流程 | P2 | ❌ Missing |
| **FR-274** | **Export Auth Isolation** | 导出时 auth store 隔离 | P1 | ❌ Missing |

---

### 3.13 企业功能

| FR-ID | 功能 | 描述 | 状态 |
|-------|------|------|------|
| FR-295 | Control Plane Client | Control Plane 客户端 | ✅ Done |
| **FR-275** | **Enterprise Policy Profile** | 高级策略 | P2 | ⚠️ Partial |

---

### 3.14 平台支持

| FR-ID | 功能 | 描述 | 状态 |
|-------|------|------|------|
| FR-300 | Linux Support | Linux 平台支持 | ✅ Done |
| FR-301 | macOS Support | macOS 平台支持 | ✅ Done |
| **FR-276** | **Windows Support** | Windows 平台支持 | P2 | ❌ Missing |

---

## 4. 技术实现规格

### 4.1 依赖配置

```toml
[dependencies]
ratatui = "0.30"
crossterm = "0.28"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = "0.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### 4.2 模块结构

```
src/
├── main.rs              # 入口、CLI 参数解析、事件循环
├── app.rs               # 应用状态管理、命令分发
├── args.rs              # CLI 参数定义、解析
├── session.rs           # 会话管理、持久化
├── runtime.rs           # LLM 运行时、API 调用
│
├── ui/
│   ├── mod.rs           # UI 模块入口
│   ├── renderer.rs      # 终端渲染器、Markdown 解析
│   ├── theme.rs         # 主题系统、颜色配置
│   ├── spinner.rs       # 加载动画
│   └── widgets/         # 自定义 Widget
│       ├── mod.rs
│       ├── message.rs   # 消息气泡
│       ├── code.rs      # 代码块
│       ├── input.rs     # 输入框
│       └── picker.rs    # 选择器
│
├── commands/
│   ├── mod.rs           # 命令模块入口
│   ├── handler.rs       # 命令处理
│   └── ...
│
└── ...
```

### 4.3 SDK 模块结构

```
opencode-sdk/
├── Cargo.toml
├── src/
│   ├── lib.rs              # SDK 入口，导出所有模块
│   ├── client.rs          # OpenCodeClient 主客户端
│   ├── session.rs         # Session 管理
│   ├── tools.rs           # 工具调用
│   ├── config.rs          # 配置管理
│   ├── auth.rs            # 认证
│   ├── error.rs           # SDK 错误类型
│   └── async_runtime.rs   # Tokio 集成
│
└── examples/
    └── basic.rs            # 基本使用示例
```

### 4.4 性能目标

| 指标 | 目标值 | 当前状态 |
|------|--------|----------|
| TUI 启动时间 | < 300ms | ⚠️ 未测量 |
| 消息渲染延迟 | < 16ms (60fps) | ⚠️ 未测量 |
| 滚动帧率 | >= 60fps | ⚠️ 未测量 |
| 内存占用（空闲） | < 30MB | ⚠️ ~40-50MB |
| 内存占用（大量消息） | < 100MB | ⚠️ 可变 |
| 二进制大小 | < 10MB | ❌ ~15-20MB |
| Build warnings | 0 | ⚠️ 有 warnings |

---

## 5. 差距分析与待办事项

### 5.1 P0 阻断性问题 (必须立即修复)

| ID | 功能 | 当前状态 | 工作量 | 影响 |
|----|------|----------|--------|------|
| FR-222 | Rust SDK | Missing | High | 程序化调用 |
| FR-223 | TypeScript SDK | Missing | High | 程序化调用 |
| FR-226 | 敏感文件默认拒绝 | Missing | Medium | 安全合规 |
| FR-227 | external_directory 拦截 | Missing | Medium | 安全合规 |

### 5.2 P1 高优先级问题 (本迭代应完成)

| ID | 功能 | 当前状态 | 工作量 | 影响 |
|----|------|----------|--------|------|
| FR-220 | Session Fork Lineage | Missing | Medium | 功能完整性 |
| FR-221 | Session Lineage Path | Missing | Low | 功能完整性 |
| FR-236 | LSP definition | Missing | Medium | 开发者体验 |
| FR-237 | LSP references | Missing | Medium | 开发者体验 |
| FR-240 | MCP Connection Pooling | Missing | Medium | 稳定性 |
| FR-242 | Plugin Event Bus | Missing | Medium | 插件生态 |
| FR-243 | WASM Sandbox Isolation | Missing | Medium | 安全性 |
| FR-234 | SSE Heartbeat | Missing | Low | 连接稳定性 |
| FR-235 | WebSocket Handshake Fix | Missing | Medium | 连接稳定性 |
| FR-229 | Credential Encryption | Missing | Medium | 安全性 |
| FR-256 | Token Budget Calibration | Missing | Medium | 上下文管理 |
| FR-257 | Compaction Thresholds | Missing | Low | 上下文管理 |
| FR-259-266 | Error Code System | Missing | Medium | 可调试性 |
| FR-267 | Crash Recovery | Missing | High | 可靠性 |
| FR-270 | 测试覆盖率提升 | Low | Medium | 代码质量 |

### 5.3 P2 中优先级问题 (下个迭代完成)

| ID | 功能 | 当前状态 | 工作量 | 影响 |
|----|------|----------|--------|------|
| FR-238 | LSP hover | Missing | Low | 开发者体验 |
| FR-239 | LSP code_actions | Missing | Medium | 开发者体验 |
| FR-241 | MCP OAuth | Missing | High | 企业支持 |
| FR-272 | PKCE Support | Missing | Medium | 企业安全 |
| FR-273 | Token Refresh/Revoke | Missing | Medium | 凭证管理 |
| FR-230 | TUI @ 多选 | Missing | Medium | 用户体验 |
| FR-232 | TUI Diff 可展开 | Missing | Medium | 代码审查 |
| FR-252 | Share JSON Export | Missing | Low | 导出功能 |
| FR-253 | Share Markdown Export | Missing | Low | 导出功能 |
| FR-254 | Patch Bundle | Missing | Medium | 导出功能 |
| FR-255 | Self-hosted Share Server | Missing | High | 公开发布 |
| FR-258 | Summary Quality | Partial | Medium | 上下文质量 |
| FR-268 | Tool Spans | Missing | Medium | 可观测性 |
| FR-269 | Cost Calculator | Partial | Medium | 可观测性 |
| FR-274 | Export Auth Isolation | Missing | Medium | 安全性 |
| FR-275 | Enterprise Policy Profile | Partial | High | 企业功能 |
| FR-276 | Windows Support | Missing | High | 平台覆盖 |

---

## 6. 技术债务清单

| 债务项 | 模块 | 复杂度 | 说明 |
|--------|------|--------|------|
| T1 | `opencode-core` 单一职责膨胀 | 高 | 62个文件，职责过多，建议拆分 domain 模块 |
| T2 | thiserror vs anyhow 混用 | 低 | 应统一为 thiserror |
| T3 | 异步运行时混用风险 | 中 | tokio 为主，部分模块未明确标注 |
| T4 | 硬编码超时值 | 低 | MCP/Server 应抽取为配置项 |
| T5 | 魔法数字 (85%/92%/95%) | 低 | 应定义为常量 |
| T6 | 日志脱敏不完整 | 中 | 部分 credential 可能泄露到日志 |
| T7 | 配置字段别名处理 | 低 | `defaultAgent` vs `default_agent` 等 alias 处理重复 |
| T8 | 错误处理不一致 | 中 | 部分工具返回 Result，部分直接 panic |
| T9 | 文档注释缺失 | 低 | 公共 API 文档不完整 |
| T10 | 依赖版本未锁定 | 中 | 使用 `version = "1.0"` 而非 `version = "=1.0.0"` |
| T11 | Dead code 清理 | 低 | 减少 warnings |
| T12 | Binary size 优化 | 中 | ~15-20MB 需优化 |

---

## 7. 实现进度总结

### 按 PRD v1.0 目标达成率

| 功能域 | PRD 要求 | 已实现 | 达成率 | 关键差距 |
|--------|----------|--------|--------|----------|
| **核心能力** |
| 项目感知 AI 会话 | ✅ | ✅ | 100% | - |
| TUI 交互 | ✅ | ✅ | 90% | diff 可展开、token/cost 显示待完善 |
| Tool Calling | ✅ | ✅ | 95% | move/delete 缺失 |
| 权限系统 | ✅ | ✅ | 85% | 敏感文件默认 deny 未实现 |
| 文件操作 (read/edit/patch/bash) | ✅ | ✅ | 95% | move/delete 工具缺失 |
| Session 持久化 | ✅ | ✅ | 90% | fork lineage 不完整 |
| 模型抽象 | ✅ | ✅ | 100% | - |
| 配置系统 | ✅ | ✅ | 95% | schema validation 可加强 |
| **平台能力** |
| Server API | ✅ | ⚠️ | 70% | SDK 缺失，SSE 心跳缺失 |
| SDK | ✅ | ❌ | 0% | Rust/TypeScript SDK 未实现 |
| LSP 接入 | ✅ | ⚠️ | 75% | definition/references 缺失 |
| MCP 接入 | ✅ | ⚠️ | 65% | OAuth 缺失，远程连接不稳定 |
| 自定义 Commands | ✅ | ⚠️ | 80% | 命令模板变量扩展不完整 |
| Skills 系统 | ✅ | ✅ | 85% | 语义匹配能力有限 |
| 基础插件系统 | ✅ | ⚠️ | 60% | 事件总线缺失 |
| **扩展能力** |
| Share 能力 | ✅ | ⚠️ | 50% | 服务端未实现，导出不完整 |
| GitHub 集成 | ✅ | ❌ | 0% | 未开始 (PRD 规划 v2) |
| Web 前端 | ✅ | ❌ | 0% | 未开始 (PRD 规划 v1.5) |
| IDE 插件 | ✅ | ❌ | 0% | 未开始 (PRD 规划 v2) |
| **非功能需求** |
| 性能 (<500ms 启动) | ✅ | ⚠️ | 80% | 未做专项 benchmark |
| 可靠性 (崩溃恢复) | ✅ | ❌ | 0% | 崩溃转储未实现 |
| 安全 (凭证加密) | ✅ | ⚠️ | 60% | 明文存储风险 |
| 可观测性 | ✅ | ⚠️ | 40% | tracing 不完整，统计缺失 |
| **测试覆盖** | - | ⚠️ | 30% | 核心模块测试不足 |

### 总体进度

```
[=========================================----------------------------------] 65%
                            已完成                              未完成
```

---

## 8. 验收标准

### 8.1 核心功能验收

- [x] `mycode` 命令可正常启动 TUI
- [x] 指定目录启动功能正常
- [x] `@` 语法可正确引用文件并进行模糊搜索
- [x] `!` 语法可正确执行 Shell 命令并返回输出
- [x] 所有斜杠命令可正常执行
- [x] 快捷键绑定正常工作
- [x] 撤销/重做功能正常（需要 Git 仓库）
- [x] 会话列表和切换功能正常
- [x] 会话分享和取消分享功能正常

### 8.2 SDK 验收 (新增)

- [ ] Rust SDK 可通过 cargo 集成
- [ ] TypeScript SDK 可通过 npm 集成
- [ ] SDK 支持会话创建/恢复/执行

### 8.3 安全验收 (新增)

- [ ] 敏感文件 (.env) 默认被拒绝读取
- [ ] 凭据以加密形式存储
- [ ] 导出 session 时 auth store 被隔离

### 8.4 UI/组件验收

- [x] 消息气泡正确渲染
- [x] 代码块语法高亮显示
- [x] 文件选择器列表组件正常工作
- [x] 命令面板可正常打开和搜索
- [x] 进度指示器正确显示
- [x] 工具详情面板可展开/收起
- [ ] Token Budget Panel (Token 预算显示)
- [ ] Diff Panel 可展开功能

### 8.5 配置验收

- [x] 配置文件 `mycode.json` 中的 TUI 配置生效
- [x] 滚动加速功能按预期工作
- [x] 滚动速度设置按预期工作
- [x] 主题切换功能正常

### 8.6 性能验收

- [ ] TUI 启动时间满足目标（< 300ms）
- [ ] 滚动流畅，无明显卡顿
- [ ] 长时间使用无内存泄漏
- [ ] 二进制大小满足目标（< 10MB）

---

## 9. 架构原则

### 9.1 核心边界

| 边界 | 原则 |
|------|------|
| Core ↔ Tools | Core 无依赖；Tools 依赖 Core |
| Server ↔ Agent | Server 处理 HTTP；Agent 处理执行 |
| Permission | 独立 crate，清晰 API |
| Storage | 抽象为 `StorageService` trait |
| SDK ↔ Core | SDK 依赖 Core，Core 无 SDK 依赖 |

### 9.2 权限模型

| 类别 | 自动批准 | 示例 |
|------|----------|------|
| Read | ReadOnly | read, grep, session_load |
| Safe | Restricted | glob, ls |
| Write | Full | write, bash, session_save |

### 9.3 配置优先级

| 优先级 | 格式 | 状态 |
|--------|------|------|
| 1 | `.opencode/config.jsonc` | 首选 |
| 2 | `.opencode/config.json` | 支持 |
| 3 | `.opencode/config.toml` | **已废弃** |

### 9.4 错误代码体系

| 范围 | 类别 | 说明 |
|------|------|------|
| 1xxx | Authentication | 认证错误 |
| 2xxx | Authorization | 授权错误 |
| 3xxx | Provider | 提供商错误 |
| 4xxx | Tool | 工具错误 |
| 5xxx | Session | 会话错误 |
| 6xxx | Config | 配置错误 |
| 7xxx | Validation | 验证错误 |
| 9xxx | Internal | 内部错误 |

---

## 10. 测试要求

### 10.1 覆盖率目标

| Crate | 最低覆盖率 | 当前覆盖 |
|-------|-----------|----------|
| opencode-core | 70% | ⚠️ ~40% |
| opencode-server | 60% | ⚠️ ~30% |
| opencode-tools | 60% | ⚠️ ~30% |
| opencode-permission | 70% | ⚠️ ~40% |
| opencode-storage | 60% | ⚠️ ~30% |
| opencode-llm | 50% | ⚠️ ~20% |

### 10.2 测试优先级

1. Session tools (session_load, session_save)
2. Permission system (allow/ask/deny)
3. Tool execution (read, write, edit)
4. MCP/LSP integration
5. Server API endpoints

---

## 11. 参考文档

- [mycode TUI 官方文档](https://mycode.ai/docs/zh-cn/tui/)
- [Ratatui GitHub](https://github.com/ratatui/ratatui)
- [Ratatui 官方文档](https://ratatui.rs)
- [Crossterm 文档](https://github.com/crossterm-rs/crossterm)
- [Tokio 文档](https://tokio.rs)

---

## 附录 A: 版本历史

| 版本 | 日期 | 主要变更 |
|------|------|----------|
| 2.3 | 2026-04-08 | 全面更新：新增 SDK、安全、观测性需求，完善 FR 编号体系 |
| 2.2 | 2026-04-08 | 差距分析更新，新增 FR-XXX 编号 |
| 2.1 | 2026-04-07 | Rust Edition PRD |
| 2.0 | 2026-04-01 | Ratatui 架构重构 |
| 1.0 | 2025-xx-xx | 初始版本 |

---

## 附录 B: FR 编号总览

| 范围 | 类别 | 当前编号 |
|------|------|----------|
| FR-001 ~ FR-019 | 会话系统 + Agent | FR-001 ~ FR-019 |
| FR-020 ~ FR-037 | 工具系统 | FR-020 ~ FR-037, FR-224, FR-225 |
| FR-040 ~ FR-069 | LLM 提供商 | FR-040 ~ FR-060 |
| FR-070 ~ FR-079 | 权限系统 | FR-070 ~ FR-079, FR-226 ~ FR-229 |
| FR-080 ~ FR-109 | TUI 基础功能 | FR-080 ~ FR-102, FR-230 |
| FR-110 ~ FR-139 | 斜杠命令 + TUI 组件 | FR-110 ~ FR-148, FR-231 ~ FR-233 |
| FR-160 ~ FR-179 | 布局 + 事件 | FR-160 ~ FR-172 |
| FR-180 ~ FR-189 | Server API | FR-180 ~ FR-187, FR-234, FR-235 |
| FR-190 ~ FR-209 | LSP + MCP | FR-190 ~ FR-202, FR-236 ~ FR-241 |
| FR-210 ~ FR-219 | Skills | FR-210 ~ FR-217 |
| FR-220 ~ FR-249 | 插件 + SDK | FR-220 ~ FR-255 |
| FR-250 ~ FR-269 | Share + Context | FR-250 ~ FR-258 |
| FR-270 ~ FR-289 | 错误 + 可观测性 | FR-270 ~ FR-270, FR-259 ~ FR-270 |
| FR-290 ~ FR-299 | 安全 + 企业 | FR-290 ~ FR-295, FR-271 ~ FR-275 |
| FR-300 ~ FR-310 | 平台支持 | FR-300, FR-301, FR-276 |

---

**状态**: 已发布  
**下一次审查**: Iteration-24  
**变更要求**: RFC 流程 (Article 7)
