# OpenCode-RS 规格文档 v2.2

**版本：** 2.2 (Iteration-22)  
**日期：** 2026年4月8日  
**作者：** mycode 产品团队  
**状态：** 已发布

---

## 变更日志

| 版本 | 日期 | 变更内容 |
|------|------|----------|
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

### 1.3 目标用户

- 习惯使用终端的开发者和运维工程师
- 需要高效代码协作的团队
- 喜欢使用键盘而非鼠标的效率导向用户

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

| Crate | 职责 | 状态 |
|-------|------|------|
| `opencode-core` | 核心功能（session, tool, error, bus） | ✅ 95% |
| `opencode-cli` | CLI 入口点 | ✅ 95% |
| `opencode-llm` | LLM 提供商抽象（16 providers） | ✅ 90% |
| `opencode-tools` | 工具实现（33/35 tools） | ✅ 95% |
| `opencode-tui` | Ratatui TUI 实现 | ✅ 85% |
| `opencode-agent` | Agent 实现（10 agents） | ✅ 100% |
| `opencode-lsp` | LSP 集成 | ✅ 80% |
| `opencode-storage` | SQLite 持久化 | ✅ 100% |
| `opencode-server` | REST/WebSocket/SSE API | ✅ 90% |
| `opencode-permission` | 权限系统 | ✅ 100% |
| `opencode-auth` | 认证凭证管理 | ✅ 100% |
| `opencode-control-plane` | Control Plane 客户端 | ✅ 100% |
| `opencode-plugin` | WASM 插件运行时 | ⚠️ 60% |
| `opencode-git` | Git 操作 | ⚠️ Partial |
| `opencode-mcp` | MCP 协议实现 | ✅ 80% |

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
| FR-008 | **session_load** | 从存储加载会话 | P0 | ❌ Missing |
| FR-009 | **session_save** | 保存会话到存储 | P0 | ❌ Missing |

#### 3.1.2 Agent 系统

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

#### 3.1.3 工具系统

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

#### 3.1.4 LLM 提供商

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
| FR-055 | **HuggingFace** | ❌ Missing | 额外优先级 |
| FR-056 | **AI21** | ❌ Missing | 额外优先级 |

#### 3.1.5 权限系统

| FR-ID | 功能 | 描述 | 状态 |
|-------|------|------|------|
| FR-060 | Allow | 自动批准工具执行 | ✅ Done |
| FR-061 | Ask | 请求用户确认 | ✅ Done |
| FR-062 | Deny | 拒绝工具执行 | ✅ Done |
| FR-063 | ReadOnly 范围 | 只读工具自动批准 | ✅ Done |
| FR-064 | Restricted 范围 | 安全工具需要确认 | ✅ Done |
| FR-065 | Full 范围 | 所有工具可用 | ✅ Done |
| FR-066 | 权限队列 | 待批准操作队列 | ✅ Done |

---

### 3.2 TUI 功能需求

#### 3.2.1 启动与基础交互

| FR-ID | 功能 | 描述 | 优先级 | 状态 |
|-------|------|------|--------|------|
| FR-070 | TUI 启动 | 运行 `mycode` 启动 TUI | P0 | ✅ Done |
| FR-071 | 目录指定 | `mycode /path` 指定目录 | P0 | ✅ Done |
| FR-072 | 消息输入 | 输入消息进行提示 | P0 | ✅ Done |
| FR-073 | AI 响应渲染 | Ratatui 组件渲染 | P0 | ✅ Done |
| FR-074 | 打字机效果 | 流式输出动画 | P1 | ⚠️ Partial |

#### 3.2.2 文件引用功能

| FR-ID | 功能 | 描述 | 优先级 | 状态 |
|-------|------|------|--------|------|
| FR-080 | `@` 文件引用 | 消息中引用文件 | P0 | ✅ Done |
| FR-081 | 模糊搜索 | 文件名模糊匹配 | P0 | ✅ Done |
| FR-082 | 自动加载 | 引用文件自动加入上下文 | P0 | ✅ Done |
| FR-083 | 列表选择 | 搜索结果 TUI 列表展示 | P0 | ✅ Done |

#### 3.2.3 Bash 命令执行

| FR-ID | 功能 | 描述 | 优先级 | 状态 |
|-------|------|------|--------|------|
| FR-090 | `!` 命令执行 | Shell 命令执行 | P0 | ✅ Done |
| FR-091 | 输出集成 | 命令结果加入对话 | P0 | ✅ Done |
| FR-092 | Ctrl+C 终止 | 终止正在执行的命令 | P1 | ✅ Done |

#### 3.2.4 斜杠命令

| FR-ID | 命令 | 描述 | 快捷键 | 状态 |
|-------|------|------|--------|------|
| FR-100 | `/connect` | 添加提供商 | - | ✅ Done |
| FR-101 | `/compact` | 压缩会话 | `ctrl+x c` | ✅ Done |
| FR-102 | `/details` | 切换详情显示 | `ctrl+x d` | ✅ Done |
| FR-103 | `/editor` | 外部编辑器 | `ctrl+x e` | ✅ Done |
| FR-104 | `/exit` | 退出 | `ctrl+x q` | ✅ Done |
| FR-105 | `/export` | 导出 Markdown | `ctrl+x x` | ✅ Done |
| FR-106 | `/help` | 帮助对话框 | `ctrl+x h` | ✅ Done |
| FR-107 | `/init` | 创建 AGENTS.md | `ctrl+x i` | ✅ Done |
| FR-108 | `/models` | 列出模型 | `ctrl+x m` | ✅ Done |
| FR-109 | `/new` | 新会话 | `ctrl+x n` | ✅ Done |
| FR-110 | `/redo` | 重做 | `ctrl+x r` | ✅ Done |
| FR-111 | `/sessions` | 会话列表/切换 | `ctrl+x l` | ✅ Done |
| FR-112 | `/share` | 分享会话 | `ctrl+x s` | ✅ Done |
| FR-113 | `/themes` | 主题列表 | `ctrl+x t` | ✅ Done |
| FR-114 | `/thinking` | 切换思考块 | - | ✅ Done |
| FR-115 | `/undo` | 撤销 | `ctrl+x u` | ✅ Done |
| FR-116 | `/unshare` | 取消分享 | - | ✅ Done |
| **FR-117** | **/status** | 显示会话状态 | - | ✅ Done |
| **FR-118** | **/cost** | 显示 Token 使用量 | - | ✅ Done |
| **FR-119** | **/permissions** | 切换权限模式 | - | ✅ Done |
| **FR-120** | **/diff** | 显示 Git 差异 | - | ✅ Done |
| **FR-121** | **/version** | 显示版本信息 | - | ✅ Done |
| **FR-122** | **/config** | 查看配置 | - | ✅ Done |
| **FR-123** | **/search** | 搜索对话历史 | - | ⚠️ Pending |

#### 3.2.5 TUI 组件

| FR-ID | 组件 | 描述 | Ratatui 基础 | 状态 |
|-------|------|------|--------------|------|
| FR-130 | MessageBubble | 消息气泡 | Paragraph + Block | ✅ Done |
| FR-131 | CodeBlock | 代码块 | Paragraph + 样式 | ✅ Done |
| FR-132 | FilePicker | 文件选择器 | List + StatefulList | ✅ Done |
| FR-133 | CommandPalette | 命令面板 | List + Input | ✅ Done |
| FR-134 | ThinkingIndicator | 思考指示器 | Spinner | ✅ Done |
| FR-135 | ToolDetail | 工具详情 | 可折叠面板 | ✅ Done |
| FR-136 | SessionList | 会话列表 | Table | ✅ Done |
| FR-137 | StatusBar | 状态栏 | 底部栏 | ✅ Done |
| **FR-138** | **Context Panel** | Token 预算显示 | - | ❌ Missing |
| FR-139 | Todo Panel | 待办面板 | List | ⚠️ Partial |
| FR-140 | Diff Panel | 差异面板 | PatchPreview | ⚠️ Partial |
| FR-141 | Diagnostics Panel | 诊断面板 | LSP 集成 | ⚠️ Partial |
| FR-142 | Files Panel | 文件面板 | FileTree | ⚠️ Partial |
| FR-143 | Permissions Panel | 权限面板 | 显示当前权限 | ⚠️ Partial |

#### 3.2.6 布局系统

| FR-ID | 布局 | 描述 | 状态 |
|-------|------|------|------|
| FR-150 | 单列布局 | 单一对话列 | ✅ Done |
| FR-151 | 双列布局 | 对话 + 侧边栏 | ✅ Done |
| FR-152 | 三列布局 | 对话 + 工具 + 文件 | ✅ Done |
| FR-153 | 响应式 | 终端宽度自适应 | ✅ Done |

#### 3.2.7 状态机

| FR-ID | 状态 | 描述 | 状态 |
|-------|------|------|------|
| FR-160 | AppMode | Idle/Composing/Thinking/Executing | ✅ Done |
| FR-161 | ExecutionState | 执行状态管理 | ✅ Done |
| FR-162 | 有效转换 | 状态转换验证 | ✅ Done |

#### 3.2.8 事件系统

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
| FR-185 | SSE /sse | Server-Sent Events | ✅ Done |
| FR-186 | MCP Protocol | MCP 协议实现 | ✅ Done |
| FR-187 | OpenAPI 3.1 | API 文档 | ⚠️ Partial |

---

### 3.4 MCP/LSP 集成

| FR-ID | 功能 | 描述 | 状态 |
|-------|------|------|------|
| FR-190 | MCP stdio bridge | 标准输入输出桥接 | ✅ Done |
| FR-191 | MCP remote bridge | 远程 MCP 连接 | ✅ Done |
| FR-192 | MCP tool discovery | 工具发现 | ✅ Done |
| FR-193 | LSP diagnostics | 诊断功能 | ✅ Done |
| FR-194 | LSP workspace | 工作区支持 | ✅ Done |
| FR-195 | LSP symbols | 符号搜索 | ⚠️ Partial |
| FR-196 | Incremental diagnostics | 增量诊断更新 | ✅ Done |

---

### 3.5 Skills 系统

| FR-ID | Skill | 描述 | 状态 |
|-------|-------|------|------|
| FR-200 | Skill Registry | 技能注册表 | ✅ Done |
| FR-201 | Command Registry | 命令注册表 | ✅ Done |
| FR-202 | TUI Commands | TUI 命令支持 | ✅ Done |
| FR-203 | Custom commands | 自定义命令 | ✅ Done |
| FR-204 | Skill matching | 语义匹配 | ✅ Done |
| FR-205 | Global/project skills | 覆盖支持 | ✅ Done |
| **FR-206** | **Built-in Skills** | 内置技能 (5/10) | ⚠️ Partial |
| **FR-207** | **OAuth login** | 浏览器认证 | ⚠️ Pending |

---

### 3.6 插件系统

| FR-ID | 功能 | 描述 | 状态 |
|-------|------|------|------|
| FR-210 | WASM runtime | WASM 运行时 | ⚠️ Partial |
| FR-211 | Sidecar plugins | Sidecar 插件 | ⚠️ Partial |
| FR-212 | Event hooks | 事件钩子 | ✅ Done |
| FR-213 | Custom tools | 插件工具注册 | ✅ Done |
| FR-214 | Sandbox isolation | 沙箱隔离 | ⚠️ Partial |
| **FR-215** | **Plugin ABI** | ABI 稳定性 | ⚠️ Pending |

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

### 4.3 性能目标

| 指标 | 目标值 | 当前状态 |
|------|--------|----------|
| TUI 启动时间 | < 300ms | ⚠️ 未测量 |
| 消息渲染延迟 | < 16ms (60fps) | ⚠️ 未测量 |
| 滚动帧率 | >= 60fps | ⚠️ 未测量 |
| 内存占用（空闲） | < 30MB | ⚠️ ~40-50MB |
| 内存占用（大量消息） | < 100MB | ⚠️ 可变 |
| 二进制大小 | < 10MB | ❌ ~15-20MB |
| Build warnings | 0 | ❌ 5 warnings |

---

## 5. 差距分析与待办事项

### 5.1 高优先级 (v1.0)

| ID | 功能 | 当前状态 | 工作量 | 影响 |
|----|------|----------|--------|------|
| FR-008 | session_load 工具 | Missing | Low | 功能完整性 |
| FR-009 | session_save 工具 | Missing | Low | 功能完整性 |
| FR-138 | Context Panel | Missing | Medium | UX 完整性 |
| FR-055 | HuggingFace 提供商 | Missing | Low | 提供商覆盖 |
| FR-056 | AI21 提供商 | Missing | Low | 提供商覆盖 |
| - | Dead code 清理 | 5 warnings | Low | 代码质量 |
| - | Binary size 优化 | ~15-20MB | Medium | 分发效率 |

### 5.2 中优先级 (v1.1)

| ID | 功能 | 当前状态 | 工作量 | 影响 |
|----|------|----------|--------|------|
| FR-139 | Todo Panel 增强 | Partial | Medium | UX 质量 |
| FR-140 | Diff Panel 增强 | Partial | Medium | UX 质量 |
| FR-141 | Diagnostics Panel 增强 | Partial | Medium | UX 质量 |
| FR-142 | Files Panel 增强 | Partial | Medium | UX 质量 |
| FR-143 | Permissions Panel 增强 | Partial | Medium | UX 质量 |
| FR-206 | Built-in Skills | 5/10 | Medium | 功能完整性 |
| FR-207 | OAuth login | Pending | High | 企业就绪 |
| FR-215 | Plugin ABI 稳定 | Pending | High | 插件生态 |

### 5.3 低优先级 (v1.5+)

| ID | 功能 | 状态 |
|----|------|------|
| - | GitHub 集成 | 未开始 |
| - | Desktop shell | 未开始 |
| - | IDE 扩展 | 未开始 |
| - | Public share server | 未开始 |

---

## 6. 验收标准

### 6.1 核心功能验收

- [x] `mycode` 命令可正常启动 TUI
- [x] 指定目录启动功能正常
- [x] `@` 语法可正确引用文件并进行模糊搜索
- [x] `!` 语法可正确执行 Shell 命令并返回输出
- [x] 所有斜杠命令可正常执行
- [x] 快捷键绑定正常工作
- [x] 撤销/重做功能正常（需要 Git 仓库）
- [x] 会话列表和切换功能正常
- [x] 会话分享和取消分享功能正常

### 6.2 UI/组件验收

- [x] 消息气泡正确渲染
- [x] 代码块语法高亮显示
- [x] 文件选择器列表组件正常工作
- [x] 命令面板可正常打开和搜索
- [x] 进度指示器正确显示
- [x] 工具详情面板可展开/收起
- [ ] Context Panel (Token 预算显示)

### 6.3 配置验收

- [x] 配置文件 `mycode.json` 中的 TUI 配置生效
- [x] 滚动加速功能按预期工作
- [x] 滚动速度设置按预期工作
- [x] 主题切换功能正常

### 6.4 性能验收

- [ ] TUI 启动时间满足目标（< 300ms）
- [ ] 滚动流畅，无明显卡顿
- [ ] 长时间使用无内存泄漏
- [ ] 二进制大小满足目标（< 10MB）

---

## 7. 架构原则

### 7.1 核心边界

| 边界 | 原则 |
|------|------|
| Core ↔ Tools | Core 无依赖；Tools 依赖 Core |
| Server ↔ Agent | Server 处理 HTTP；Agent 处理执行 |
| Permission | 独立 crate，清晰 API |
| Storage | 抽象为 `StorageService` trait |

### 7.2 权限模型

| 类别 | 自动批准 | 示例 |
|------|----------|------|
| Read | ReadOnly | read, grep, session_load |
| Safe | Restricted | glob, ls |
| Write | Full | write, bash, session_save |

### 7.3 配置优先级

| 优先级 | 格式 | 状态 |
|--------|------|------|
| 1 | `.opencode/config.jsonc` | 首选 |
| 2 | `.opencode/config.json` | 支持 |
| 3 | `.opencode/config.toml` | **已废弃** |

---

## 8. 测试要求

### 8.1 覆盖率目标

| Crate | 最低覆盖率 |
|-------|-----------|
| opencode-core | 70% |
| opencode-server | 60% |
| opencode-tools | 60% |
| opencode-permission | 70% |
| opencode-storage | 60% |
| opencode-llm | 50% |

### 8.2 测试类别

1. **单元测试**: 位于实现附近的 `#[cfg(test)]` 模块
2. **API 测试**: 位于 `crates/*/src/*_test.rs`
3. **集成测试**: 位于 `tests/src/` 目录

---

## 9. 参考文档

- [mycode TUI 官方文档](https://mycode.ai/docs/zh-cn/tui/)
- [Ratatui GitHub](https://github.com/ratatui/ratatui)
- [Ratatui 官方文档](https://ratatui.rs)
- [Crossterm 文档](https://github.com/crossterm-rs/crossterm)
- [Tokio 文档](https://tokio.rs)

---

## 附录 A: 版本历史

| 版本 | 日期 | 主要变更 |
|------|------|----------|
| 2.2 | 2026-04-08 | 差距分析更新，新增 FR-XXX 编号 |
| 2.1 | 2026-04-07 | Rust Edition PRD |
| 2.0 | 2026-04-01 | Ratatui 架构重构 |
| 1.0 | 2025-xx-xx | 初始版本 |

---

## 附录 B: 已知技术债务

| ID | 描述 | 风险 | 状态 |
|----|------|------|------|
| T1 | 配置格式不一致 | High | C-056 缓解 |
| T2 | README 过时 | Medium | Pending |
| T3 | 权限路由不清晰 | Medium | C-024 缓解 |
| T4 | TUI 权限确认 | Low | Accepted |
| T5 | auth_layered 未集成 | Low | Pending |

---

**状态**: 已发布  
**下一次审查**: Iteration-23  
**变更要求**: RFC 流程 (Article 7)
