# PRD：opencode-rs Desktop UI — AI Coding Control Center

## 0. 文档信息

| 项目   | 内容                                           |
| ---- | -------------------------------------------- |
| 产品名称 | opencode-rs Desktop UI                       |
| 产品定位 | 面向 AI Coding Agent 的桌面控制中心                   |
| 首发平台 | macOS                                        |
| 后续平台 | Windows / Linux                              |
| 核心用户 | AI 工程师、独立开发者、研发团队、架构师、测试工程师                  |
| 目标系统 | opencode-rs：Rust 实现的 AI Coding Agent Runtime |
| 文档类型 | Product Requirements Document                |
| 输出用途 | 可直接作为 AI Coding / 工程实现输入                     |
| 设计重点 | Agent-first、上下文透明、生成可控、输出可验证、全过程可追踪          |

---

# 1. Product Vision & Goals

## 1.1 产品愿景

opencode-rs Desktop UI 不是一个传统 IDE，也不是一个简单的 AI Chat 面板，而是一个面向 AI Coding 全生命周期的 **AI Coding Control Center**。

它的目标是让用户能够以工程化、可观察、可调试、可复用的方式驱动 AI Coding Agent 完成真实软件开发任务。

核心理念：

> 让 AI Coding 从“聊天式代码生成”升级为“可治理、可追踪、可验证的工程执行系统”。

用户不仅可以输入需求，还可以管理任务、查看上下文、控制规则、观察多 Agent 协作、追踪每一步执行、验证输出结果，并持续优化 AI Coding 过程。

---

## 1.2 目标用户

### 1.2.1 Solo Developer

典型需求：

| 需求           | 描述              |
| ------------ | --------------- |
| 快速实现功能       | 从需求描述生成任务、代码、测试 |
| 理解 Agent 行为  | 知道 AI 为什么这么改代码  |
| 避免失控修改       | 限制 AI 修改范围      |
| 本地模型 / 云模型切换 | 根据成本、速度、隐私切换模型  |
| 一人维护多个项目     | 快速初始化、复用规则、复用技能 |

典型场景：

* 给一个 Rust 项目增加 CLI 参数。
* 让 Agent 修复失败测试。
* 基于 PRD 自动拆解 task.json。
* 对某次 AI 修改进行回滚、复盘、再执行。

---

### 1.2.2 AI Engineer / Agent Engineer

典型需求：

| 需求               | 描述                                      |
| ---------------- | --------------------------------------- |
| 调试 Agent Runtime | 观察 planner / implementer / reviewer 的状态 |
| 调试 LLM 调用        | 查看模型、provider、proxy、request、response    |
| 调试上下文注入          | 确认 repo、MCP、rules、skills 是否正确注入         |
| 管理工具链            | 管理 MCP、skills、commands、hooks            |
| 运行多 Agent 实验     | 比较不同模型、不同策略、不同上下文组合                     |

典型场景：

* 调试 `model not found`。
* 分析某次输出质量差的原因。
* 比较 GLM / Kimi / Minimax 在同一任务上的效果。
* 设计新的 skill 并验证其效果。

---

### 1.2.3 Team / Engineering Organization

典型需求：

| 需求              | 描述                                          |
| --------------- | ------------------------------------------- |
| 统一 AI Coding 规范 | 团队级 rules / policies                        |
| 任务可追踪           | 每个 AI run 都可审计                              |
| 质量门禁            | lint / test / static analysis / review 必须通过 |
| DevOps 集成       | 与 CI/CD、issue、PR、release 流程衔接               |
| 安全治理            | API key、代码、日志、模型请求受控                        |

典型场景：

* 团队定义 AI Coding 开发规范。
* 架构师定义禁止修改的模块。
* 开发人员从 PRD 生成 task.json。
* Reviewer 追踪 AI 修改与验证结果。
* CI 失败后由 Agent 自动分析并提出修复方案。

---

## 1.3 核心价值主张

### 1.3.1 从 Chat UI 到 Control Center

传统 Chat UI：

```text
用户输入需求 → AI 输出代码建议 → 用户手动复制/修改/验证
```

opencode-rs Desktop UI：

```text
需求输入
  ↓
任务建模
  ↓
上下文供给
  ↓
生成约束
  ↓
多 Agent 执行
  ↓
代码变更
  ↓
自动验证
  ↓
问题诊断
  ↓
迭代修复
  ↓
可审计结果
```

---

### 1.3.2 与传统 IDE 的差异

| 维度             | 传统 IDE   | opencode-rs Desktop UI |
| -------------- | -------- | ---------------------- |
| 核心对象           | 文件 / 编辑器 | 任务 / Agent / 执行过程      |
| 交互方式           | 人手写代码    | 人控制 Agent 完成任务         |
| 上下文            | 用户自己理解   | UI 显示 Agent 实际上下文      |
| 质量控制           | 手动运行测试   | 自动验证 + 质量门禁            |
| 多模型支持          | 通常弱      | 一等公民                   |
| MCP 集成         | 非核心      | 一等公民                   |
| Hooks / Skills | 插件式辅助    | Agent 生命周期核心机制         |
| 可调试性           | 调试代码     | 调试 AI 行为 + 工具链行为       |

---

### 1.3.3 与 Chat-based Coding Tool 的差异

| 维度      | Chat Coding Tool | opencode-rs Desktop UI        |
| ------- | ---------------- | ----------------------------- |
| 交互入口    | Chat             | Task / Plan / Run / Context   |
| 任务状态    | 隐式               | 显式状态机                         |
| 上下文可见性  | 模糊               | 可检查、可裁剪、可锁定                   |
| 约束机制    | Prompt 内隐式描述     | Rules / Policies / Guardrails |
| 验证机制    | 依赖用户             | 内置 validation pipeline        |
| 多 Agent | 通常不可见            | 显式编排视图                        |
| 日志      | 简单历史             | Timeline + Event Log + Trace  |
| 工程治理    | 弱                | 强                             |

---

## 1.4 关键工程场景

### 场景 A：从 PRD 到可执行任务

用户输入 PRD，系统自动拆解为 task.json，并生成执行计划。

```text
PRD.md
  ↓
Planner Agent
  ↓
task.json
  ↓
任务树
  ↓
逐任务执行
```

---

### 场景 B：基于上下文透明的代码生成

用户可以看到 Agent 实际使用了哪些上下文：

* 哪些文件被读取？
* 哪些规则被注入？
* 哪些 MCP 数据源被调用？
* 哪些 skills 被激活？
* 哪些历史 run 被引用？
* 哪些代码区域被禁止修改？

---

### 场景 C：生成质量约束

用户可以在生成前设置：

* 只能修改指定目录。
* 必须符合 Rust module boundary。
* 必须补充测试。
* 禁止引入新依赖。
* 禁止修改 public API。
* 必须保持与 opencode 原项目行为一致。

---

### 场景 D：输出质量校验

系统自动执行：

* `cargo check`
* `cargo test`
* `cargo clippy`
* `cargo fmt --check`
* 静态分析
* 差异分析
* AI reviewer
* Git diff risk scan

---

### 场景 E：调试模型与 Provider 问题

当出现 `model not found`：

用户可以从 UI 查看：

* 当前 provider 配置
* 当前 model id
* proxy route
* request 是否真正发出
* proxy 是否收到请求
* provider 返回内容
* fallback 策略
* agent runtime 内部错误栈

---

# 2. Design Principles

## 2.1 Agent-first UI

产品的核心不是文件编辑器，而是 Agent 执行控制。

### 设计要求

| 原则       | 要求                                       |
| -------- | ---------------------------------------- |
| 任务优先     | 所有工作围绕 Task / Plan / Run 展开              |
| Agent 可见 | Planner / Implementer / Reviewer 状态必须显性化 |
| 执行可控     | 用户可以暂停、恢复、重试、回滚                          |
| 过程可解释    | 每一步 Agent 行为都有原因、输入、输出                   |
| 人机协作     | 用户是控制者，不是旁观者                             |

---

## 2.2 Context Transparency

AI Coding 的核心问题之一是：

> 用户不知道模型到底看到了什么。

因此 UI 必须提供完整上下文透明度。

### 上下文类型

| 类型                | 示例                                |
| ----------------- | --------------------------------- |
| Repo Context      | 文件、目录、符号、依赖、测试                    |
| Task Context      | PRD、task.json、acceptance criteria |
| Knowledge Context | 项目规范、架构文档、历史决策                    |
| Rules Context     | 编码规范、架构约束、安全规则                    |
| MCP Context       | issue、CI、日志、数据库、监控                |
| Git Context       | diff、branch、commit、PR             |
| Runtime Context   | 执行日志、测试结果、错误栈                     |

---

## 2.3 Full Lifecycle Visibility

UI 必须覆盖完整 AI Coding 生命周期：

```text
Plan
  ↓
Context Supply
  ↓
Generation Constraints
  ↓
Generate
  ↓
Apply Changes
  ↓
Validate
  ↓
Review
  ↓
Iterate
  ↓
Commit / PR
```

每个阶段都必须有：

* 当前状态
* 输入内容
* 输出内容
* 触发的工具
* 调用的模型
* 错误信息
* 可操作入口

---

## 2.4 Deterministic + Debuggable AI Behavior

AI 行为本身具有不确定性，因此系统需要尽可能工程化。

### 要求

| 目标  | 机制                                            |
| --- | --------------------------------------------- |
| 可复现 | run snapshot、context snapshot、prompt snapshot |
| 可比较 | 同一任务多模型对比                                     |
| 可追踪 | event log、trace id、tool call log              |
| 可诊断 | model request / response viewer               |
| 可回滚 | Git checkpoint                                |
| 可约束 | rules / guardrails                            |
| 可验证 | validation pipeline                           |

---

## 2.5 Extensibility

系统必须支持持续扩展。

扩展点包括：

* MCP server
* Skill
* Command
* Hook
* Rule
* Provider
* Validation runner
* Agent role
* Prompt template
* Context provider
* UI panel plugin

---

# 3. System Architecture

## 3.1 高层架构图

```text
┌───────────────────────────────────────────────────────────────┐
│                    opencode-rs Desktop UI                     │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │                     UI Layer                            │  │
│  │                                                         │  │
│  │  Workspace  Task  Agent  Context  Validation  Timeline │  │
│  └─────────────────────────────────────────────────────────┘  │
│                              │                                │
│                              ▼                                │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │              Interaction / State Layer                  │  │
│  │                                                         │  │
│  │  App State Store                                        │  │
│  │  Run State Machine                                      │  │
│  │  Event Bus                                              │  │
│  │  Command Dispatcher                                     │  │
│  └─────────────────────────────────────────────────────────┘  │
│                              │                                │
│                              ▼                                │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │              Agent Orchestration Layer                  │  │
│  │                                                         │  │
│  │  Planner Agent                                          │  │
│  │  Implementer Agent                                      │  │
│  │  Reviewer Agent                                         │  │
│  │  Tool Router                                            │  │
│  │  Model Router                                           │  │
│  └─────────────────────────────────────────────────────────┘  │
│                              │                                │
│        ┌─────────────────────┼──────────────────────┐         │
│        ▼                     ▼                      ▼         │
│ ┌──────────────┐     ┌──────────────┐      ┌──────────────┐   │
│ │Context Engine│     │Toolchain     │      │Provider Proxy│   │
│ │              │     │Integration   │      │              │   │
│ │Repo Index    │     │Git           │      │Kimi          │   │
│ │Rules         │     │Cargo         │      │GLM           │   │
│ │MCP           │     │CI/CD         │      │Minimax       │   │
│ │Knowledge     │     │Linters       │      │Local Models  │   │
│ └──────────────┘     └──────────────┘      └──────────────┘   │
│        │                     │                      │         │
└────────┼─────────────────────┼──────────────────────┼─────────┘
         ▼                     ▼                      ▼
┌────────────────┐    ┌────────────────┐     ┌────────────────┐
│ Local Repo     │    │ DevOps / CI     │     │ LLM Providers  │
│ File System    │    │ MCP Servers     │     │ Proxy Gateway  │
└────────────────┘    └────────────────┘     └────────────────┘
```

---

## 3.2 分层职责

### 3.2.1 UI Layer

负责：

* 展示 workspace、task、run、agent、context、validation。
* 提供用户操作入口。
* 实时展示流式输出。
* 处理错误、确认、风险提示。
* 不直接执行底层命令。

不负责：

* 不直接调用 LLM。
* 不直接修改代码。
* 不直接管理复杂 Agent 状态机。
* 不直接持久化执行逻辑。

---

### 3.2.2 Interaction / State Layer

负责：

* 全局状态管理。
* 事件订阅与分发。
* UI command 到 runtime command 的转换。
* Run state machine。
* 本地缓存。
* 用户设置。
* 面板之间的数据同步。

核心对象：

```text
WorkspaceState
TaskState
RunState
AgentState
ContextState
ValidationState
GitState
McpState
ProviderState
TimelineState
```

---

### 3.2.3 Agent Orchestration Layer

负责：

* Planner / Implementer / Reviewer 编排。
* Agent 生命周期管理。
* Tool call routing。
* Model routing。
* Prompt construction。
* Retry / fallback / reflection。
* Agent event emit。

---

### 3.2.4 Context Engine

负责：

* Repo indexing。
* 文件选择。
* 符号索引。
* 上下文裁剪。
* MCP 数据注入。
* Rules 注入。
* Knowledge 注入。
* Context snapshot 生成。
* Context token budget 管理。

---

### 3.2.5 Toolchain Integration

负责：

* Git 操作。
* Cargo 命令。
* Shell command。
* Test runner。
* Linter。
* Static analyzer。
* DevOps pipeline。
* MCP client。
* 文件系统读写。

---

### 3.2.6 Provider Proxy

负责：

* 多模型 provider 统一接入。
* API key 管理。
* OAuth / Browser auth。
* Request / response logging。
* Provider fallback。
* Model alias。
* Rate limit。
* Cost tracking。
* Error normalization。

---

## 3.3 Event-driven / State-driven UI Model

系统必须采用事件驱动模型。

### 事件流示例

```text
User creates task
  ↓
TaskCreated
  ↓
PlannerStarted
  ↓
ContextRequested
  ↓
ContextResolved
  ↓
PlanGenerated
  ↓
UserApprovesPlan
  ↓
ImplementerStarted
  ↓
FileRead
  ↓
ToolCallStarted
  ↓
FilePatchProposed
  ↓
PatchApplied
  ↓
ValidationStarted
  ↓
TestFailed
  ↓
ReviewerStarted
  ↓
FixSuggested
  ↓
IterationStarted
```

---

## 3.4 UI 如何反映 Agent 内部状态

每个 Agent 必须暴露以下状态：

| 状态                    | UI 表现                |
| --------------------- | -------------------- |
| Idle                  | 灰色 Agent card        |
| Preparing Context     | Context panel 高亮     |
| Thinking              | 流式 reasoning summary |
| Calling Tool          | Tool call chip       |
| Waiting User Approval | 黄色确认状态               |
| Applying Patch        | File diff 动画         |
| Validating            | Validation panel 运行态 |
| Failed                | 红色错误状态               |
| Completed             | 绿色完成状态               |

---

# 4. Core Modules

---

# 4.1 Workspace & Project Manager

## 4.1.1 模块定位

Workspace & Project Manager 是用户进入系统后的第一入口，负责管理本地或远程代码仓、项目配置、AI coding 初始化状态。

它不是普通文件打开器，而是 AI Coding workspace 管理器。

---

## 4.1.2 Responsibilities

| 职责                     | 描述                                      |
| ---------------------- | --------------------------------------- |
| Workspace 管理           | 创建、打开、切换 workspace                      |
| Repo 管理                | 绑定本地 Git repo 或远程 repo                  |
| 项目初始化                  | 生成 `.opencode-rs/` 配置                   |
| AI coding readiness 检查 | 检查 rules、skills、MCP、provider、validation |
| 项目画像                   | 展示语言、框架、依赖、测试命令                         |
| 多 workspace 支持         | 支持最近项目、收藏项目                             |
| 安全边界                   | 用户确认允许 Agent 访问哪些目录                     |

---

## 4.1.3 UI Layout

```text
┌────────────────────────────────────────────────────┐
│ Workspace Manager                                  │
├────────────────────────────────────────────────────┤
│ Recent Workspaces                                  │
│  ▸ opencode-rs        ~/dev/opencode-rs            │
│  ▸ agent-proxy        ~/dev/agent-proxy            │
│  ▸ tech-debt-board    ~/dev/tech-debt-board        │
│                                                    │
│ Actions                                            │
│  [Open Local Repo] [Clone Repo] [Create Workspace] │
│                                                    │
│ Selected Workspace                                 │
│  Name: opencode-rs                                 │
│  Path: ~/dev/opencode-rs                           │
│  Git: main                                         │
│  Language: Rust                                    │
│  AI Coding Status: Ready                           │
│                                                    │
│ Readiness                                          │
│  ✓ Git detected                                    │
│  ✓ Cargo detected                                  │
│  ✓ Rules configured                                │
│  ✓ Provider configured                             │
│  ! MCP not configured                              │
│  ✓ Validation pipeline ready                       │
└────────────────────────────────────────────────────┘
```

---

## 4.1.4 Key Components

| Component                  | 描述                    |
| -------------------------- | --------------------- |
| Workspace List             | 最近项目、收藏项目、当前项目        |
| Repo Open Dialog           | 打开本地仓库                |
| Clone Dialog               | 输入 Git URL，clone 后初始化 |
| Readiness Checklist        | AI coding 准备度检查       |
| Project Profile Card       | 技术栈、语言、包管理器、测试命令      |
| Permission Boundary Editor | 允许 Agent 访问的目录        |
| Workspace Settings         | workspace 级配置         |

---

## 4.1.5 State Model

```ts
type WorkspaceState = {
  id: string
  name: string
  rootPath: string
  repo?: GitRepoState
  projectProfile: ProjectProfile
  aiCodingStatus: "not_initialized" | "partial" | "ready" | "error"
  permissions: WorkspacePermission[]
  recentRuns: RunSummary[]
  configPath: string
}
```

```ts
type ProjectProfile = {
  languages: string[]
  primaryLanguage: "rust" | "typescript" | "python" | "go" | "unknown"
  packageManagers: string[]
  testCommands: CommandSpec[]
  buildCommands: CommandSpec[]
  lintCommands: CommandSpec[]
  detectedFrameworks: string[]
}
```

---

## 4.1.6 User Interactions

### 打开项目

```text
User clicks "Open Local Repo"
  ↓
Select folder
  ↓
System scans Git + language + config
  ↓
Show readiness report
  ↓
User confirms workspace permissions
  ↓
Workspace opened
```

### 初始化 AI Coding

```text
User clicks "Initialize for AI Coding"
  ↓
System scans repo structure
  ↓
Suggest rules / commands / validation
  ↓
Generate .opencode-rs/
  ↓
Create baseline context index
  ↓
Ready
```

---

## 4.1.7 Data Flow

```text
File System
  ↓
Repo Scanner
  ↓
Project Profiler
  ↓
Workspace State
  ↓
Readiness UI
  ↓
User Approval
  ↓
.opencode-rs config generated
```

---

## 4.1.8 Integration Points

| 系统                  | 集成点                       |
| ------------------- | ------------------------- |
| Git                 | branch、status、commit、diff |
| File System         | repo scan、permission      |
| Cargo               | Rust 项目检测                 |
| MCP                 | workspace MCP 配置          |
| Provider Proxy      | 模型配置 readiness            |
| Validation Pipeline | test/lint/build 命令        |

---

# 4.2 Agent Interaction Panel

## 4.2.1 模块定位

Agent Interaction Panel 不是普通聊天窗口，而是任务驱动的 Agent 控制面板。

它承载：

* 用户意图输入
* Agent 输出
* Tool call 展示
* Plan approval
* Patch approval
* Human-in-the-loop 决策

---

## 4.2.2 Responsibilities

| 职责           | 描述                                     |
| ------------ | -------------------------------------- |
| 输入用户任务       | 支持自然语言、PRD、issue、文件引用                  |
| 展示 Agent 响应  | 展示 plan、analysis summary、tool calls    |
| 控制 Agent 行为  | pause、resume、retry、stop                |
| 审批关键动作       | 修改文件、执行命令、访问 MCP                       |
| 展示多 Agent 对话 | planner / implementer / reviewer 分角色展示 |
| 任务上下文引用      | 支持 `@file`、`@symbol`、`@rule`、`@mcp`    |

---

## 4.2.3 UI Layout

```text
┌─────────────────────────────────────────────────────┐
│ Agent Interaction                                   │
├─────────────────────────────────────────────────────┤
│ Current Task: Implement CLI config loader            │
│ Status: Implementing                                │
│                                                     │
│ ┌ Planner ────────────────────────────────────────┐ │
│ │ Proposed 5-step plan                            │ │
│ │ [View Plan] [Approve] [Modify]                  │ │
│ └─────────────────────────────────────────────────┘ │
│                                                     │
│ ┌ Implementer ────────────────────────────────────┐ │
│ │ Reading src/config.rs                           │ │
│ │ Calling tool: file.read                         │ │
│ │ Proposed patch: 3 files                         │ │
│ │ [View Diff] [Apply] [Reject]                    │ │
│ └─────────────────────────────────────────────────┘ │
│                                                     │
│ ┌ Reviewer ───────────────────────────────────────┐ │
│ │ Found missing test for invalid config path       │ │
│ │ [Create follow-up task] [Ask implementer fix]    │ │
│ └─────────────────────────────────────────────────┘ │
│                                                     │
│ Input                                               │
│ ┌─────────────────────────────────────────────────┐ │
│ │ Ask agent / refine task / approve instruction...│ │
│ └─────────────────────────────────────────────────┘ │
│ [Send] [Run] [Stop] [Attach PRD] [Reference File]   │
└─────────────────────────────────────────────────────┘
```

---

## 4.2.4 Key Components

| Component            | 描述                                        |
| -------------------- | ----------------------------------------- |
| Agent Message Stream | 按 Agent 角色分组                              |
| Tool Call Chips      | 展示工具调用                                    |
| Approval Card        | 需要用户确认的操作                                 |
| Task Input Box       | 多模式输入                                     |
| Reference Picker     | 引用文件、规则、MCP 数据                            |
| Run Control Bar      | start / pause / stop / retry              |
| Mode Selector        | plan-only / implement / validate / review |

---

## 4.2.5 State Model

```ts
type AgentInteractionState = {
  activeTaskId?: string
  activeRunId?: string
  messages: AgentMessage[]
  pendingApprovals: ApprovalRequest[]
  inputDraft: string
  selectedReferences: ContextReference[]
  mode: "ask" | "plan" | "implement" | "review" | "debug"
}
```

```ts
type AgentMessage = {
  id: string
  runId: string
  agentRole: "planner" | "implementer" | "reviewer" | "system"
  kind: "text" | "plan" | "tool_call" | "diff" | "error" | "approval"
  content: unknown
  timestamp: string
}
```

---

## 4.2.6 User Interactions

### 输入任务

```text
User enters task
  ↓
System creates draft task
  ↓
Planner agent starts
  ↓
Plan card appears
  ↓
User approves or edits
```

### 审批 patch

```text
Implementer proposes patch
  ↓
UI shows diff summary
  ↓
User opens diff
  ↓
User applies / rejects / asks revision
```

---

## 4.2.7 Data Flow

```text
User Input
  ↓
Command Dispatcher
  ↓
Agent Orchestrator
  ↓
Agent Event Stream
  ↓
Interaction State
  ↓
Agent Panel UI
```

---

## 4.2.8 Integration Points

| 系统             | 集成点                     |
| -------------- | ----------------------- |
| Agent Runtime  | agent event stream      |
| Context Engine | reference picker        |
| Git            | diff preview            |
| Toolchain      | command approval        |
| Provider Proxy | model status            |
| Timeline       | all interactions logged |

---

# 4.3 Task / Plan Management System

## 4.3.1 模块定位

Task / Plan Management System 是 AI Coding 的核心工作单元管理模块。

它负责把需求转成可执行、可追踪、可验证的任务结构。

---

## 4.3.2 Responsibilities

| 职责     | 描述                              |
| ------ | ------------------------------- |
| 创建任务   | 从自然语言、PRD、issue、文件创建任务          |
| 拆解任务   | 生成 task.json                    |
| 任务树管理  | parent / child / dependency     |
| 验收标准   | 管理 acceptance criteria          |
| 执行计划   | 管理 plan steps                   |
| 状态追踪   | todo / running / blocked / done |
| 历史 run | 一个任务可多次执行                       |
| 人工编辑   | 用户可手动修改 task.json               |

---

## 4.3.3 UI Layout

```text
┌──────────────────────────────────────────────────────┐
│ Task / Plan                                          │
├──────────────────────────────────────────────────────┤
│ Task Tree                                            │
│  ▾ PRD: Add provider proxy debugging                 │
│    ▸ T1: Add provider config inspector               │
│    ▸ T2: Add request trace viewer                    │
│    ▸ T3: Add model-not-found diagnosis               │
│    ▸ T4: Add validation tests                        │
│                                                      │
│ Selected Task                                        │
│  ID: T2                                              │
│  Title: Add request trace viewer                     │
│  Status: Running                                     │
│  Owner Agent: Implementer                            │
│                                                      │
│ Acceptance Criteria                                  │
│  ✓ show request payload                              │
│  ✓ show provider response                            │
│  - show proxy route                                  │
│                                                      │
│ Plan Steps                                           │
│  1. Inspect provider module                          │
│  2. Add trace data model                             │
│  3. Build timeline UI                                │
│  4. Add tests                                        │
│                                                      │
│ [Edit task.json] [Run Task] [Validate] [Create Child]│
└──────────────────────────────────────────────────────┘
```

---

## 4.3.4 Key Components

| Component                  | 描述             |
| -------------------------- | -------------- |
| Task Tree                  | 层级任务结构         |
| Task Detail                | 当前任务详情         |
| Plan Step List             | Agent 执行计划     |
| Acceptance Criteria Editor | 验收标准           |
| Dependency Graph           | 任务依赖           |
| Run History                | 每次执行记录         |
| task.json Editor           | 原始 JSON 编辑     |
| Task Importer              | PRD / issue 导入 |

---

## 4.3.5 State Model

```ts
type TaskState = {
  tasks: Record<string, Task>
  selectedTaskId?: string
  taskTree: TaskTreeNode[]
  activeRunByTask: Record<string, string>
}
```

```ts
type Task = {
  id: string
  parentId?: string
  title: string
  description: string
  status: "draft" | "ready" | "running" | "blocked" | "done" | "failed"
  priority: "low" | "medium" | "high" | "critical"
  type: "feature" | "bugfix" | "refactor" | "test" | "docs" | "infra"
  acceptanceCriteria: AcceptanceCriterion[]
  plan: PlanStep[]
  constraints: TaskConstraint[]
  contextRefs: ContextReference[]
  validationRefs: ValidationRequirement[]
  runHistory: string[]
}
```

---

## 4.3.6 task.json 示例

```json
{
  "version": "1.0",
  "project": "opencode-rs",
  "tasks": [
    {
      "id": "T1",
      "title": "Add provider config inspector",
      "type": "feature",
      "priority": "high",
      "status": "ready",
      "description": "Add a UI panel to inspect active provider and model configuration.",
      "acceptanceCriteria": [
        {
          "id": "AC1",
          "description": "User can see active provider id, model id, base URL and auth mode.",
          "required": true
        },
        {
          "id": "AC2",
          "description": "UI highlights missing model id or unsupported provider.",
          "required": true
        }
      ],
      "constraints": [
        {
          "kind": "scope",
          "description": "Only modify desktop UI provider settings modules."
        },
        {
          "kind": "quality",
          "description": "Must include unit tests for provider config parsing."
        }
      ],
      "contextRefs": [
        {
          "type": "file",
          "path": "src/provider/config.rs"
        },
        {
          "type": "rule",
          "id": "rust-error-handling"
        }
      ],
      "validation": [
        {
          "type": "command",
          "command": "cargo test provider_config"
        },
        {
          "type": "command",
          "command": "cargo clippy --all-targets -- -D warnings"
        }
      ]
    }
  ]
}
```

---

## 4.3.7 User Interactions

### 从 PRD 生成任务

```text
User uploads PRD
  ↓
Click "Decompose into Tasks"
  ↓
Planner extracts modules, requirements, acceptance criteria
  ↓
Task tree generated
  ↓
User reviews task.json
  ↓
User approves
```

### 手动拆分任务

```text
User selects large task
  ↓
Click "Split Task"
  ↓
Planner suggests subtasks
  ↓
User accepts / edits
  ↓
Task tree updated
```

---

## 4.3.8 Data Flow

```text
PRD / User Input
  ↓
Planner Agent
  ↓
Task Parser
  ↓
task.json
  ↓
Task State Store
  ↓
Task UI
  ↓
Run Orchestrator
```

---

## 4.3.9 Integration Points

| 系统                  | 集成点                        |
| ------------------- | -------------------------- |
| Planner Agent       | task decomposition         |
| Context Engine      | task context references    |
| Validation Pipeline | task acceptance validation |
| Git                 | run branch / checkpoint    |
| Timeline            | task lifecycle events      |

---

# 4.4 Context Inspection Panel

## 4.4.1 模块定位

Context Inspection Panel 用于回答一个核心问题：

> 当前 Agent 到底看到了什么上下文？

这是 AI Coding Control Center 的关键差异化模块。

---

## 4.4.2 Responsibilities

| 职责          | 描述                         |
| ----------- | -------------------------- |
| 展示注入上下文     | 文件、规则、MCP、知识、历史            |
| 显示来源        | 每条上下文从哪里来                  |
| 显示 token 占用 | 上下文预算透明                    |
| 支持启用 / 禁用   | 用户可调整上下文                   |
| 支持锁定上下文     | 防止 Agent 自行扩展              |
| 支持上下文快照     | 每次 run 保存 context snapshot |
| 支持差异对比      | 比较两次 run 上下文变化             |

---

## 4.4.3 UI Layout

```text
┌──────────────────────────────────────────────────────┐
│ Context Inspection                                   │
├──────────────────────────────────────────────────────┤
│ Context Budget                                       │
│  Used: 78k / 128k tokens                             │
│  Repo: 42k | Rules: 8k | MCP: 16k | Task: 12k        │
│                                                      │
│ Sources                                              │
│  ▾ Repo Files                                        │
│    ✓ src/provider/mod.rs        8.2k                 │
│    ✓ src/agent/runtime.rs       12.1k                │
│    ! src/legacy/debug.rs        excluded             │
│                                                      │
│  ▾ Rules                                             │
│    ✓ rust-error-handling                             │
│    ✓ no-unscoped-file-modification                   │
│                                                      │
│  ▾ MCP Data                                          │
│    ✓ github.issue#123                                │
│    ✓ ci.last_failure                                 │
│                                                      │
│  ▾ Knowledge                                         │
│    ✓ architecture.md                                 │
│    ✓ provider-design.md                              │
│                                                      │
│ Actions                                              │
│  [Lock Context] [Edit Selection] [Compare Snapshot]  │
└──────────────────────────────────────────────────────┘
```

---

## 4.4.4 Key Components

| Component           | 描述                             |
| ------------------- | ------------------------------ |
| Context Budget Bar  | token 分布                       |
| Context Source Tree | repo / MCP / rules / knowledge |
| Context Item Detail | 查看具体内容                         |
| Inclusion Toggle    | include / exclude              |
| Lock Context Switch | 锁定上下文                          |
| Snapshot Selector   | 选择 run snapshot                |
| Diff Viewer         | 对比上下文差异                        |
| Context Search      | 搜索当前注入内容                       |

---

## 4.4.5 State Model

```ts
type ContextState = {
  activeRunId?: string
  snapshot?: ContextSnapshot
  selectedItemId?: string
  tokenBudget: TokenBudget
  lockMode: boolean
}
```

```ts
type ContextSnapshot = {
  id: string
  runId: string
  taskId: string
  createdAt: string
  items: ContextItem[]
  totalTokens: number
  modelContextLimit: number
}
```

```ts
type ContextItem = {
  id: string
  type: "repo_file" | "symbol" | "rule" | "mcp" | "knowledge" | "git" | "task"
  source: string
  title: string
  contentPreview: string
  tokenCount: number
  included: boolean
  required: boolean
  reason: string
}
```

---

## 4.4.6 User Interactions

### 查看上下文来源

```text
User opens Context Panel
  ↓
Selects "MCP Data"
  ↓
Clicks ci.last_failure
  ↓
UI shows:
    source server
    query
    returned data
    token count
    injected prompt section
```

### 排除文件上下文

```text
User unchecks src/legacy/debug.rs
  ↓
Context Engine recalculates token budget
  ↓
Agent run context updated
  ↓
Timeline logs ContextItemExcluded
```

---

## 4.4.7 Data Flow

```text
Task
  ↓
Context Requirement
  ↓
Context Engine
  ↓
Repo Index / Rules / MCP / Knowledge
  ↓
Context Snapshot
  ↓
Context Panel
  ↓
User Modification
  ↓
Updated Context Snapshot
```

---

## 4.4.8 Integration Points

| 系统              | 集成点                 |
| --------------- | ------------------- |
| Repo Indexer    | 文件、符号               |
| Rules Engine    | rules injection     |
| MCP Client      | external data       |
| Knowledge Store | 文档、规范               |
| Agent Runtime   | prompt construction |
| Timeline        | context events      |

---

# 4.5 Generation Constraints Panel

## 4.5.1 模块定位

Generation Constraints Panel 用于定义 AI 生成前必须遵守的约束。

它解决的问题是：

> 不要等 AI 生成完再发现乱改，而是在生成前就限制它的行为边界。

---

## 4.5.2 Responsibilities

| 职责      | 描述                                    |
| ------- | ------------------------------------- |
| 管理生成规则  | coding rules、architecture rules       |
| 管理修改范围  | allowlist / denylist                  |
| 管理依赖策略  | 是否允许新增依赖                              |
| 管理行为策略  | 是否允许执行 shell 命令                       |
| 管理安全策略  | secret、private file、dangerous command |
| 管理任务级约束 | 当前任务的临时约束                             |
| 约束预检    | generation 前检查约束冲突                    |

---

## 4.5.3 UI Layout

```text
┌──────────────────────────────────────────────────────┐
│ Generation Constraints                               │
├──────────────────────────────────────────────────────┤
│ Active Constraint Profile                            │
│  Profile: Rust Safe Refactor                         │
│                                                      │
│ Scope Constraints                                    │
│  ✓ Allow modify: src/provider/**                     │
│  ✓ Allow modify: tests/provider/**                   │
│  ✗ Deny modify: src/core/protocol/**                 │
│                                                      │
│ Coding Rules                                         │
│  ✓ rust-error-handling                               │
│  ✓ no-unwrap-in-production                           │
│  ✓ must-add-tests-for-behavior-change                │
│                                                      │
│ Dependency Policy                                    │
│  Mode: Ask before adding dependency                  │
│                                                      │
│ Tool Policy                                          │
│  Shell command: approval required                    │
│  File deletion: blocked                              │
│                                                      │
│ Preflight                                            │
│  ✓ no conflict                                       │
│  ! task wants to modify denied path                  │
│                                                      │
│ [Edit Rules] [Run Preflight] [Save Profile]          │
└──────────────────────────────────────────────────────┘
```

---

## 4.5.4 Key Components

| Component                   | 描述              |
| --------------------------- | --------------- |
| Constraint Profile Selector | 选择约束模板          |
| Scope Editor                | allow / deny 路径 |
| Rule List                   | 当前启用规则          |
| Dependency Policy Control   | 依赖策略            |
| Tool Policy Control         | 工具权限            |
| Risk Level Indicator        | 当前风险等级          |
| Preflight Result            | 约束预检结果          |

---

## 4.5.5 State Model

```ts
type GenerationConstraintState = {
  activeProfileId: string
  taskOverrides: Constraint[]
  preflightResult?: ConstraintPreflightResult
}
```

```ts
type Constraint = {
  id: string
  kind:
    | "scope"
    | "coding_rule"
    | "architecture_rule"
    | "dependency_policy"
    | "tool_policy"
    | "security_policy"
  severity: "info" | "warning" | "blocking"
  description: string
  enabled: boolean
  source: "workspace" | "task" | "team" | "user"
}
```

---

## 4.5.6 User Interactions

### 添加路径约束

```text
User clicks "Add Scope Constraint"
  ↓
Selects "Deny Modify"
  ↓
Chooses src/core/protocol/**
  ↓
Constraint saved
  ↓
Preflight runs automatically
```

### 生成前预检

```text
User starts run
  ↓
Constraint Engine checks:
    path scope
    dependency policy
    tool permissions
    rule conflicts
  ↓
If blocking issue:
    run blocked
    UI shows reason
```

---

## 4.5.7 Data Flow

```text
Workspace Rules
  +
Task Constraints
  +
User Overrides
  ↓
Constraint Engine
  ↓
Preflight Result
  ↓
Prompt Guardrails
  ↓
Agent Runtime
```

---

## 4.5.8 Integration Points

| 系统                | 集成点                    |
| ----------------- | ---------------------- |
| Rules Engine      | rule loading           |
| Agent Runtime     | prompt guardrails      |
| Tool Router       | tool permission        |
| File Patch Engine | path scope enforcement |
| Timeline          | constraint violations  |

---

# 4.6 Output Validation Panel

## 4.6.1 模块定位

Output Validation Panel 用于验证 AI 生成结果是否满足工程质量要求。

它不是简单展示测试结果，而是 AI Coding 质量门禁中心。

---

## 4.6.2 Responsibilities

| 职责                   | 描述                           |
| -------------------- | ---------------------------- |
| 运行验证命令               | build / test / lint / format |
| 展示测试结果               | pass / fail / skipped        |
| 展示静态分析               | clippy、security scan         |
| 展示 AI review         | reviewer agent 结果            |
| 映射验收标准               | acceptance criteria coverage |
| 支持失败诊断               | 错误归因、建议修复                    |
| 支持自动迭代               | 失败后让 Agent 修复                |
| 输出 validation report | run 级质量报告                    |

---

## 4.6.3 UI Layout

```text
┌──────────────────────────────────────────────────────┐
│ Output Validation                                    │
├──────────────────────────────────────────────────────┤
│ Validation Summary                                   │
│  Status: Failed                                      │
│  Passed: 5 | Failed: 2 | Skipped: 1                  │
│                                                      │
│ Checks                                               │
│  ✓ cargo fmt --check                                 │
│  ✓ cargo check                                       │
│  ✗ cargo test provider_config                        │
│  ✗ cargo clippy --all-targets                        │
│  ✓ AI reviewer                                       │
│                                                      │
│ Failed Test                                          │
│  test_invalid_model_id_returns_error                 │
│  expected ProviderError::ModelNotFound               │
│  got ProviderError::Unknown                          │
│                                                      │
│ Acceptance Criteria                                  │
│  ✓ active provider visible                           │
│  ✗ missing model id highlighted                      │
│  ✓ proxy route shown                                 │
│                                                      │
│ Actions                                              │
│  [Ask Agent Fix] [Open Failure] [Rerun] [Create Task]│
└──────────────────────────────────────────────────────┘
```

---

## 4.6.4 Key Components

| Component              | 描述                 |
| ---------------------- | ------------------ |
| Validation Summary     | 总体状态               |
| Check List             | 每个验证项              |
| Test Failure Detail    | 失败详情               |
| Static Analysis Detail | lint / security    |
| Acceptance Coverage    | 验收标准覆盖             |
| AI Review Result       | reviewer 总结        |
| Fix Action Bar         | 修复 / 重跑 / 创建任务     |
| Report Export          | 导出 markdown / json |

---

## 4.6.5 State Model

```ts
type ValidationState = {
  activeRunId?: string
  status: "idle" | "running" | "passed" | "failed" | "cancelled"
  checks: ValidationCheck[]
  acceptanceCoverage: AcceptanceCoverage[]
  report?: ValidationReport
}
```

```ts
type ValidationCheck = {
  id: string
  name: string
  type: "build" | "test" | "lint" | "format" | "static_analysis" | "ai_review"
  command?: string
  status: "pending" | "running" | "passed" | "failed" | "skipped"
  output?: string
  errorSummary?: string
  startedAt?: string
  endedAt?: string
}
```

---

## 4.6.6 User Interactions

### 执行验证

```text
User clicks "Run Validation"
  ↓
Validation Runner executes configured checks
  ↓
Panel streams outputs
  ↓
Failures grouped by type
  ↓
Reviewer Agent analyzes failure
  ↓
User chooses fix / rerun / ignore
```

### 失败后自动修复

```text
Validation failed
  ↓
User clicks "Ask Agent Fix"
  ↓
Failure context injected
  ↓
Implementer starts fix loop
  ↓
Validation reruns
```

---

## 4.6.7 Data Flow

```text
Code Diff
  ↓
Validation Pipeline
  ↓
Command Runner
  ↓
Raw Output
  ↓
Result Parser
  ↓
Validation State
  ↓
Reviewer Agent
  ↓
Fix Recommendation
```

---

## 4.6.8 Integration Points

| 系统            | 集成点                         |
| ------------- | --------------------------- |
| Cargo         | check / test / clippy / fmt |
| Shell Runner  | custom command              |
| Agent Runtime | AI review                   |
| Task System   | acceptance criteria         |
| Git           | changed file mapping        |
| Timeline      | validation events           |

---

# 4.7 Multi-Agent Orchestration View

## 4.7.1 模块定位

Multi-Agent Orchestration View 用于显式展示 planner / implementer / reviewer 的协作过程。

---

## 4.7.2 Responsibilities

| 职责             | 描述                               |
| -------------- | -------------------------------- |
| 展示 Agent 角色    | planner / implementer / reviewer |
| 展示状态流转         | 每个 Agent 当前状态                    |
| 展示交接关系         | planner 输出给 implementer          |
| 展示冲突           | reviewer reject implementer      |
| 支持重跑单个 Agent   | 只重跑 reviewer                     |
| 支持模型配置         | 不同 agent 使用不同模型                  |
| 支持 Agent trace | 查看每个 agent 的输入输出                 |

---

## 4.7.3 UI Layout

```text
┌──────────────────────────────────────────────────────┐
│ Multi-Agent Orchestration                            │
├──────────────────────────────────────────────────────┤
│                                                      │
│ ┌──────────┐      ┌──────────────┐      ┌──────────┐ │
│ │ Planner  │─────▶│ Implementer  │─────▶│ Reviewer │ │
│ │ Passed   │      │ Running      │      │ Waiting  │ │
│ └──────────┘      └──────────────┘      └──────────┘ │
│      │                  │                    │       │
│      ▼                  ▼                    ▼       │
│  Plan v3            Patch #2              Review     │
│                                                      │
│ Agent Config                                         │
│  Planner:     glm-4.5                                │
│  Implementer: kimi-k2                                │
│  Reviewer:    local-qwen-code                        │
│                                                      │
│ Current Handoff                                      │
│  Planner → Implementer                               │
│  "Implement provider debug trace according to plan"  │
│                                                      │
│ [View Trace] [Rerun Planner] [Pause Implementer]     │
└──────────────────────────────────────────────────────┘
```

---

## 4.7.4 Key Components

| Component          | 描述               |
| ------------------ | ---------------- |
| Agent Graph        | Agent 协作图        |
| Agent Card         | 每个 Agent 状态      |
| Handoff Viewer     | 交接内容             |
| Agent Trace Viewer | 输入、输出、tool call  |
| Model Selector     | 每个 Agent 选择模型    |
| Retry Controls     | 重跑某个 Agent       |
| Conflict Panel     | review reject 原因 |

---

## 4.7.5 State Model

```ts
type MultiAgentState = {
  runId: string
  agents: AgentNode[]
  handoffs: AgentHandoff[]
  activeAgentId?: string
}
```

```ts
type AgentNode = {
  id: string
  role: "planner" | "implementer" | "reviewer"
  status:
    | "idle"
    | "running"
    | "waiting"
    | "completed"
    | "failed"
    | "paused"
  model: string
  provider: string
  currentStep?: string
}
```

---

## 4.7.6 User Interactions

### 重跑 Reviewer

```text
User clicks Reviewer card
  ↓
Clicks "Rerun Reviewer"
  ↓
System reuses current diff + validation result
  ↓
Reviewer generates new review
```

### 替换 Agent 模型

```text
User opens Agent Config
  ↓
Selects Implementer model: kimi
  ↓
Future implementer runs use kimi
  ↓
Timeline logs model change
```

---

## 4.7.7 Data Flow

```text
Run State Machine
  ↓
Agent Orchestrator
  ↓
Agent Events
  ↓
Multi-Agent State
  ↓
Orchestration View
```

---

## 4.7.8 Integration Points

| 系统             | 集成点                    |
| -------------- | ---------------------- |
| Agent Runtime  | role state             |
| Provider Proxy | per-agent model        |
| Timeline       | agent events           |
| Task System    | plan / review relation |
| Context Engine | role-specific context  |

---

# 4.8 Code Explorer

## 4.8.1 模块定位

Code Explorer 是轻量代码浏览器，不是完整 IDE。

它用于：

* 查看 Agent 读取了哪些文件。
* 查看 AI 修改了哪些代码。
* 查看 diff。
* 查看 symbols。
* 查看测试失败关联文件。

---

## 4.8.2 Responsibilities

| 职责              | 描述                        |
| --------------- | ------------------------- |
| 文件浏览            | repo tree                 |
| 代码查看            | read-only first           |
| Diff 查看         | before / after            |
| Agent touch map | 哪些文件被 Agent 访问 / 修改       |
| Symbol 导航       | function / struct / trait |
| 修改范围高亮          | changed lines             |
| 快速引用            | `@file` 添加到上下文            |

---

## 4.8.3 UI Layout

```text
┌──────────────────────────────────────────────────────┐
│ Code Explorer                                        │
├──────────────────────────────────────────────────────┤
│ File Tree                 │ Code Viewer              │
│                           │                          │
│ ▾ src                     │ src/provider/config.rs   │
│   ▾ provider              │                          │
│     mod.rs                │  12 pub struct Provider  │
│     config.rs  modified   │  13   id: String,        │
│     error.rs   read       │  14   model: String,     │
│                           │                          │
│ ▾ tests                   │ Diff                     │
│   provider_test.rs        │ - old                    │
│                           │ + new                    │
│                           │                          │
│                           │ [Add to Context]         │
└──────────────────────────────────────────────────────┘
```

---

## 4.8.4 Key Components

| Component          | 描述                          |
| ------------------ | --------------------------- |
| File Tree          | repo 文件树                    |
| Code Viewer        | 代码阅读                        |
| Diff Viewer        | 变更对比                        |
| Symbol Outline     | 当前文件符号                      |
| Agent Access Badge | read / modified / generated |
| Context Add Button | 添加到上下文                      |
| Open External IDE  | 用 VSCode / Cursor 打开        |

---

## 4.8.5 State Model

```ts
type CodeExplorerState = {
  selectedFile?: string
  openFiles: string[]
  fileAccessMap: Record<string, AgentFileAccess>
  diffMode: "none" | "inline" | "side_by_side"
}
```

```ts
type AgentFileAccess = {
  filePath: string
  readBy: string[]
  modifiedBy: string[]
  generatedBy: string[]
  lastAccessAt: string
}
```

---

## 4.8.6 User Interactions

### 查看 Agent 修改

```text
User opens Code Explorer
  ↓
Modified files highlighted
  ↓
User selects file
  ↓
Diff appears
  ↓
User approves / rejects patch
```

---

## 4.8.7 Integration Points

| 系统             | 集成点                 |
| -------------- | ------------------- |
| File System    | file tree           |
| Git            | diff                |
| Agent Runtime  | file access events  |
| Context Engine | add file to context |
| External IDE   | open file           |

---

# 4.9 Git Integration Panel

## 4.9.1 模块定位

Git Integration Panel 用于管理 AI 修改产生的代码变更。

---

## 4.9.2 Responsibilities

| 职责                | 描述                            |
| ----------------- | ----------------------------- |
| 展示 Git status     | modified / staged / untracked |
| 创建 checkpoint     | run 前保存状态                     |
| 查看 diff           | 文件级、hunk 级                    |
| stage / unstage   | 选择性提交                         |
| commit message 生成 | AI 生成 commit message          |
| branch 管理         | 为任务创建 branch                  |
| rollback          | 回滚某次 run                      |
| PR 支持             | 后续版本支持创建 PR                   |

---

## 4.9.3 UI Layout

```text
┌──────────────────────────────────────────────────────┐
│ Git                                                  │
├──────────────────────────────────────────────────────┤
│ Branch: feature/provider-debug                       │
│ Base: main                                           │
│                                                      │
│ Checkpoints                                          │
│  ✓ before-run-2026-04-26-001                         │
│  ✓ after-plan-v3                                     │
│                                                      │
│ Changes                                              │
│  M src/provider/config.rs                            │
│  M src/provider/error.rs                             │
│  A tests/provider_config_test.rs                     │
│                                                      │
│ Commit Message                                       │
│  Add provider debug trace inspection                 │
│                                                      │
│ [Stage All] [Commit] [Rollback Run] [Create Branch]  │
└──────────────────────────────────────────────────────┘
```

---

## 4.9.4 Key Components

| Component                | 描述                |
| ------------------------ | ----------------- |
| Branch Selector          | 当前分支              |
| Status List              | 文件状态              |
| Diff Viewer              | Git diff          |
| Checkpoint List          | run checkpoints   |
| Commit Message Generator | AI commit message |
| Rollback Control         | 回滚                |
| PR Button                | 创建 PR，未来支持        |

---

## 4.9.5 State Model

```ts
type GitState = {
  branch: string
  baseBranch?: string
  status: GitFileStatus[]
  checkpoints: GitCheckpoint[]
  stagedFiles: string[]
}
```

```ts
type GitCheckpoint = {
  id: string
  name: string
  runId?: string
  commitHash?: string
  createdAt: string
  description: string
}
```

---

## 4.9.6 User Interactions

### Run 前创建 checkpoint

```text
User starts implementation
  ↓
System asks or auto creates checkpoint
  ↓
Git checkpoint stored
  ↓
Agent applies changes
```

### 回滚某次 run

```text
User selects run checkpoint
  ↓
Clicks rollback
  ↓
UI shows affected files
  ↓
User confirms
  ↓
Git reset / restore executed
```

---

## 4.9.7 Integration Points

| 系统                | 集成点                      |
| ----------------- | ------------------------ |
| Git CLI / libgit2 | status、diff、commit       |
| Agent Runtime     | patch attribution        |
| Timeline          | checkpoint events        |
| Validation        | validate before commit   |
| Task System       | task-run-commit relation |

---

# 4.10 MCP Data Sources Panel

## 4.10.1 模块定位

MCP Data Sources Panel 用于管理和调试 Model Context Protocol 数据源。

---

## 4.10.2 Responsibilities

| 职责                 | 描述                     |
| ------------------ | ---------------------- |
| 管理 MCP server      | 添加、启动、停止               |
| 查看 tools/resources | MCP 暴露的能力              |
| 测试 MCP 调用          | 手动执行 tool              |
| 配置任务绑定             | 某些任务启用某些 MCP           |
| 查看调用日志             | MCP request / response |
| 权限控制               | 哪些 Agent 可调用           |
| 健康检查               | server status          |

---

## 4.10.3 UI Layout

```text
┌──────────────────────────────────────────────────────┐
│ MCP Data Sources                                     │
├──────────────────────────────────────────────────────┤
│ Servers                                              │
│  ✓ github-mcp          running                       │
│  ✓ ci-mcp              running                       │
│  ! database-mcp        disconnected                  │
│                                                      │
│ Selected: github-mcp                                 │
│  Command: npx github-mcp                             │
│  Tools:                                               │
│    - get_issue                                        │
│    - list_pull_requests                              │
│    - get_file                                        │
│                                                      │
│ Recent Calls                                         │
│  get_issue #123       success                        │
│  list_pull_requests   success                        │
│                                                      │
│ [Add Server] [Test Tool] [View Logs] [Restart]       │
└──────────────────────────────────────────────────────┘
```

---

## 4.10.4 Key Components

| Component         | 描述                      |
| ----------------- | ----------------------- |
| MCP Server List   | server 状态               |
| Tool Catalog      | tools/resources/prompts |
| MCP Config Editor | server config           |
| Test Console      | 手动调用                    |
| Permission Matrix | Agent / task 访问权限       |
| Call Log          | request / response      |
| Health Indicator  | running / failed        |

---

## 4.10.5 State Model

```ts
type McpState = {
  servers: McpServer[]
  selectedServerId?: string
  callLogs: McpCallLog[]
}
```

```ts
type McpServer = {
  id: string
  name: string
  command: string
  args: string[]
  status: "stopped" | "starting" | "running" | "failed"
  tools: McpTool[]
  resources: McpResource[]
  permissions: McpPermission[]
}
```

---

## 4.10.6 User Interactions

### 添加 MCP server

```text
User clicks Add Server
  ↓
Inputs command and args
  ↓
System validates config
  ↓
Starts MCP server
  ↓
Discovers tools/resources
  ↓
Displays in catalog
```

### 动态注入 MCP

```text
User selects active task
  ↓
Enables github-mcp
  ↓
Selects get_issue #123
  ↓
Context Engine injects returned issue data
  ↓
Context Panel displays MCP item
```

---

## 4.10.7 Integration Points

| 系统                | 集成点                |
| ----------------- | ------------------ |
| MCP Client        | server protocol    |
| Context Engine    | MCP data injection |
| Agent Runtime     | tool call          |
| Permission Engine | access control     |
| Timeline          | MCP calls          |

---

# 4.11 Skills / Commands / Hooks Management UI

## 4.11.1 模块定位

该模块是 opencode-rs 扩展能力的管理中心。

它管理：

* Skills
* Commands
* Hooks
* Rules

其中 Hooks 是 AI Coding 生命周期注入点的关键机制。

---

## 4.11.2 Responsibilities

| 职责          | 描述                       |
| ----------- | ------------------------ |
| 管理 Skills   | 创建、编辑、启用、禁用              |
| 管理 Commands | 定义可执行命令                  |
| 管理 Hooks    | 生命周期事件触发动作               |
| 管理 Rules    | 编码规则、架构规则                |
| 绑定任务        | task-specific skill/rule |
| 查看执行记录      | skill / hook run logs    |
| 调试扩展        | dry run、trace            |

---

## 4.11.3 UI Layout

```text
┌──────────────────────────────────────────────────────┐
│ Skills / Commands / Hooks / Rules                    │
├──────────────────────────────────────────────────────┤
│ Tabs: [Skills] [Commands] [Hooks] [Rules]            │
│                                                      │
│ Hooks                                                │
│  ✓ before_generation: inject-rust-rules              │
│  ✓ after_generation: scan-dangerous-diff             │
│  ✓ before_validation: ensure-tests-exist             │
│  ✓ after_validation: summarize-failures              │
│                                                      │
│ Selected Hook                                        │
│  Name: scan-dangerous-diff                           │
│  Event: after_generation                             │
│  Action: run command dangerous-diff-scan             │
│  Blocking: true                                      │
│                                                      │
│ [Create] [Edit] [Dry Run] [View Logs]                │
└──────────────────────────────────────────────────────┘
```

---

## 4.11.4 Key Components

| Component        | 描述                          |
| ---------------- | --------------------------- |
| Skill Library    | skill 列表                    |
| Command Registry | command 列表                  |
| Hook Registry    | hook 列表                     |
| Rule Library     | rule 列表                     |
| Editor           | YAML / Markdown / JSON 编辑   |
| Dry Run Button   | 测试执行                        |
| Execution Log    | 扩展执行记录                      |
| Binding Matrix   | workspace / task / agent 绑定 |

---

## 4.11.5 Skill Lifecycle

```text
Create Skill
  ↓
Define metadata
  ↓
Define trigger conditions
  ↓
Define prompt/tool behavior
  ↓
Validate skill format
  ↓
Enable for workspace/task
  ↓
Agent runtime discovers skill
  ↓
Skill invoked during run
  ↓
Execution logged
```

---

## 4.11.6 Hook Injection Points

### 必须支持的 Hook

| Hook Point               | 触发时机             | 典型用途              |
| ------------------------ | ---------------- | ----------------- |
| `before_generation`      | Agent 生成代码前      | 注入规则、检查上下文        |
| `after_generation`       | Agent 生成 patch 后 | 扫描 diff、检查风险      |
| `before_validation`      | 执行测试前            | 确保测试文件存在、准备环境     |
| `after_validation`       | 验证完成后            | 总结失败、触发修复         |
| `before_tool_call`       | 调用工具前            | 权限检查              |
| `after_tool_call`        | 工具调用后            | 日志、结果转换           |
| `before_context_resolve` | 上下文选择前           | 添加必要上下文源          |
| `after_context_resolve`  | 上下文生成后           | token 预算检查        |
| `before_patch_apply`     | patch 应用前        | 路径限制、危险变更扫描       |
| `after_patch_apply`      | patch 应用后        | git diff snapshot |

---

## 4.11.7 Hook 示例

```json
{
  "id": "scan-dangerous-diff",
  "name": "Scan Dangerous Diff",
  "event": "after_generation",
  "enabled": true,
  "blocking": true,
  "conditions": [
    {
      "type": "changed_path_matches",
      "pattern": "src/core/**"
    }
  ],
  "actions": [
    {
      "type": "command",
      "commandId": "dangerous-diff-scan"
    },
    {
      "type": "agent_review",
      "agentRole": "reviewer",
      "prompt": "Review this diff for risky architecture changes."
    }
  ]
}
```

---

## 4.11.8 State Model

```ts
type ExtensionState = {
  skills: Skill[]
  commands: Command[]
  hooks: Hook[]
  rules: Rule[]
  executionLogs: ExtensionExecutionLog[]
}
```

```ts
type Hook = {
  id: string
  name: string
  event: HookEvent
  enabled: boolean
  blocking: boolean
  conditions: HookCondition[]
  actions: HookAction[]
}
```

---

## 4.11.9 Integration Points

| 系统                | 集成点               |
| ----------------- | ----------------- |
| Agent Runtime     | skill invocation  |
| Run State Machine | hook events       |
| Command Runner    | command execution |
| Rules Engine      | rule injection    |
| Timeline          | extension events  |

---

# 4.12 Execution Timeline / Event Log Viewer

## 4.12.1 模块定位

Execution Timeline 是整个 AI Coding run 的审计、调试、复盘中心。

---

## 4.12.2 Responsibilities

| 职责           | 描述                                  |
| ------------ | ----------------------------------- |
| 展示 run event | 所有生命周期事件                            |
| 支持过滤         | agent / tool / context / validation |
| 支持 trace     | 展开 request / response               |
| 支持错误定位       | 错误链路                                |
| 支持 replay    | 后续支持                                |
| 支持导出         | JSON / Markdown                     |
| 支持性能分析       | 每一步耗时                               |

---

## 4.12.3 UI Layout

```text
┌──────────────────────────────────────────────────────┐
│ Execution Timeline                                   │
├──────────────────────────────────────────────────────┤
│ Filters: [Agent] [Tool] [Context] [Validation] [Err] │
│                                                      │
│ 10:01:02 TaskCreated                                 │
│ 10:01:05 PlannerStarted                              │
│ 10:01:08 ContextResolved       78k tokens            │
│ 10:01:20 PlanGenerated         5 steps               │
│ 10:01:30 UserApprovedPlan                            │
│ 10:01:31 ImplementerStarted                          │
│ 10:01:34 FileRead              src/provider.rs       │
│ 10:02:11 PatchGenerated        3 files               │
│ 10:02:20 HookExecuted          scan-dangerous-diff   │
│ 10:02:40 ValidationStarted                           │
│ 10:03:12 TestFailed            provider_config_test  │
│                                                      │
│ [Export Trace] [Open Error] [Compare Run]            │
└──────────────────────────────────────────────────────┘
```

---

## 4.12.4 Key Components

| Component           | 描述                 |
| ------------------- | ------------------ |
| Event List          | 时间线                |
| Event Detail Drawer | 事件详情               |
| Trace Viewer        | request / response |
| Error Chain Viewer  | 错误链                |
| Duration Chart      | 耗时                 |
| Filter Bar          | 类型过滤               |
| Export Button       | 导出                 |
| Compare Runs        | 比较两次 run           |

---

## 4.12.5 State Model

```ts
type TimelineState = {
  activeRunId?: string
  events: ExecutionEvent[]
  filters: TimelineFilter
  selectedEventId?: string
}
```

```ts
type ExecutionEvent = {
  id: string
  runId: string
  timestamp: string
  type: ExecutionEventType
  source: "ui" | "agent" | "tool" | "context" | "validation" | "git" | "mcp"
  title: string
  detail?: unknown
  durationMs?: number
  severity: "debug" | "info" | "warning" | "error"
}
```

---

## 4.12.6 Integration Points

| 系统                     | 集成点                    |
| ---------------------- | ---------------------- |
| All Runtime Components | event emit             |
| Provider Proxy         | request trace          |
| MCP Client             | call trace             |
| Git                    | diff/checkpoint events |
| Validation             | validation events      |
| UI                     | user actions           |

---

# 5. AI Coding Lifecycle Visualization

UI 必须围绕三条主线设计：

1. Context Supply
2. Generation Constraints
3. Output Validation

---

## 5.1 生命周期总览

```text
┌───────────────────────────────────────────────────────────────┐
│                     AI Coding Lifecycle                       │
└───────────────────────────────────────────────────────────────┘

User Intent / PRD / Issue
        │
        ▼
┌──────────────────┐
│ Task Modeling    │
│ task.json        │
└──────────────────┘
        │
        ▼
┌──────────────────┐
│ Context Supply   │◀──── Repo / MCP / Rules / Knowledge / Git
└──────────────────┘
        │
        ▼
┌───────────────────────┐
│ Generation Constraints│◀──── Scope / Policies / Guardrails
└───────────────────────┘
        │
        ▼
┌──────────────────┐
│ Agent Generation │◀──── Planner / Implementer / Reviewer
└──────────────────┘
        │
        ▼
┌──────────────────┐
│ Patch / Changes  │
└──────────────────┘
        │
        ▼
┌──────────────────┐
│ Output Validation│◀──── Test / Lint / Static / AI Review
└──────────────────┘
        │
        ▼
Pass ─────────────▶ Commit / PR / Done
Fail ─────────────▶ Iterate / Fix / Retry
```

---

## 5.2 Context Supply

### 5.2.1 UI 位置

Context Supply 主要出现在：

| UI 区域                    | 用途                            |
| ------------------------ | ----------------------------- |
| Context Inspection Panel | 查看实际注入上下文                     |
| Task Detail              | 任务绑定上下文                       |
| Agent Interaction Panel  | `@file` / `@mcp` / `@rule` 引用 |
| Timeline                 | 上下文解析事件                       |
| MCP Panel                | 外部数据源                         |
| Code Explorer            | 文件加入上下文                       |

---

### 5.2.2 用户如何检查

用户可以查看：

* 当前 run 注入了哪些文件。
* 哪些上下文是 Agent 自动选择的。
* 哪些上下文是用户显式指定的。
* 每个上下文项的 token 占用。
* 每个上下文项被选择的原因。
* 哪些上下文被裁剪。
* 哪些上下文来自 MCP。
* 上下文是否被锁定。

---

### 5.2.3 用户如何修改

支持操作：

| 操作      | 描述               |
| ------- | ---------------- |
| Include | 加入上下文            |
| Exclude | 排除上下文            |
| Pin     | 固定某个上下文          |
| Lock    | 锁定整个上下文集合        |
| Replace | 替换 MCP 数据        |
| Refresh | 刷新 repo / MCP 数据 |
| Compare | 比较两次 run 上下文     |

---

### 5.2.4 如何影响 Agent 行为

Context Supply 直接影响：

* Agent 能理解哪些代码。
* Agent 能否感知项目规范。
* Agent 是否知道 CI 失败原因。
* Agent 是否知道历史设计决策。
* Agent 是否有足够信息生成正确 patch。

---

## 5.3 Generation Constraints

### 5.3.1 UI 位置

| UI 区域                        | 用途            |
| ---------------------------- | ------------- |
| Generation Constraints Panel | 管理规则和约束       |
| Task Detail                  | 任务级约束         |
| Skills / Rules UI            | 管理规则源         |
| Patch Approval UI            | 展示违反约束的 patch |
| Timeline                     | 约束检查日志        |

---

### 5.3.2 用户如何检查

用户可以查看：

* 当前启用的规则。
* 当前允许修改的路径。
* 当前禁止修改的路径。
* 是否允许新增依赖。
* 是否允许执行命令。
* 是否存在约束冲突。
* 约束来自 workspace、team 还是 task。

---

### 5.3.3 用户如何修改

支持：

* 修改 constraint profile。
* 添加任务临时约束。
* 禁用非强制规则。
* 切换 strict / normal / exploratory 模式。
* 设置 approval policy。
* 调整 tool permission。

---

### 5.3.4 如何影响 Agent 行为

Generation Constraints 会在多个层面生效：

| 层面                 | 生效方式                |
| ------------------ | ------------------- |
| Prompt             | 注入 guardrails       |
| Tool Router        | 拦截危险工具调用            |
| Patch Engine       | 阻止非法路径修改            |
| Dependency Manager | 拦截新增依赖              |
| Hook System        | after_generation 扫描 |
| Reviewer Agent     | 根据规则审查输出            |

---

## 5.4 Output Validation

### 5.4.1 UI 位置

| UI 区域                   | 用途                  |
| ----------------------- | ------------------- |
| Output Validation Panel | 主验证中心               |
| Task Detail             | acceptance coverage |
| Agent Interaction Panel | 失败修复交互              |
| Timeline                | validation events   |
| Git Panel               | commit 前验证          |

---

### 5.4.2 用户如何检查

用户可以查看：

* 哪些验证通过。
* 哪些验证失败。
* 失败命令输出。
* 失败测试关联文件。
* 哪些 acceptance criteria 未满足。
* Reviewer Agent 的诊断。
* 是否允许进入 commit。

---

### 5.4.3 用户如何修改

支持：

* 添加 validation command。
* 禁用非必要检查。
* 设置 blocking / non-blocking。
* 选择失败后自动修复。
* 创建 follow-up task。
* 标记某项为 accepted risk。

---

### 5.4.4 如何影响 Agent 行为

Validation 结果会反馈给 Agent：

```text
Validation Failure
  ↓
Failure Context Built
  ↓
Injected into Implementer
  ↓
Agent proposes fix
  ↓
Validation reruns
```

---

# 6. Key Workflows

---

# 6.1 Initialize a Repo for AI Coding

## 6.1.1 目标

让普通代码仓变成可被 opencode-rs Agent 安全、可控、可验证执行的 workspace。

---

## 6.1.2 Step-by-step

```text
1. User opens local repo
2. System scans repo
3. System detects language/framework/build/test tools
4. System checks Git status
5. System asks permission boundary
6. System generates project profile
7. System suggests default rules
8. System suggests validation commands
9. System creates .opencode-rs/
10. System builds initial repo index
11. System validates provider config
12. Workspace becomes AI Coding Ready
```

---

## 6.1.3 Sequence

```text
User
  │
  │ Open Repo
  ▼
Workspace Manager
  │
  │ scan
  ▼
Repo Scanner
  │
  │ detect Rust / Cargo / tests
  ▼
Project Profiler
  │
  │ generate profile
  ▼
AI Coding Initializer
  │
  │ create .opencode-rs/
  ▼
Context Engine
  │
  │ build index
  ▼
Validation Runner
  │
  │ check commands
  ▼
Workspace Ready
```

---

## 6.1.4 Generated Files

```text
.opencode-rs/
  config.json
  rules/
    rust.md
    architecture.md
    testing.md
  skills/
    rust-refactor.md
    test-failure-fixer.md
  commands/
    cargo-check.json
    cargo-test.json
  hooks/
    default-hooks.json
  runs/
  context-cache/
```

---

# 6.2 Create a Task from PRD

## 6.2.1 目标

从 PRD 生成结构化 task.json。

---

## 6.2.2 Step-by-step

```text
1. User clicks "Create Task from PRD"
2. User selects PRD file or pastes content
3. Planner Agent extracts features
4. Planner Agent identifies modules
5. Planner Agent creates task hierarchy
6. Planner Agent generates acceptance criteria
7. Planner Agent suggests constraints
8. UI shows task tree preview
9. User edits / approves
10. task.json saved
```

---

## 6.2.3 Sequence

```text
User
  │
  │ Upload PRD
  ▼
Task Manager
  │
  │ send PRD
  ▼
Planner Agent
  │
  │ analyze requirements
  ▼
Task Decomposer
  │
  │ produce task.json
  ▼
Task Manager UI
  │
  │ preview
  ▼
User
  │
  │ approve
  ▼
Task Store
```

---

# 6.3 Decompose into task.json

## 6.3.1 任务拆解原则

拆解结果必须满足：

| 原则   | 描述                       |
| ---- | ------------------------ |
| 可执行  | 每个 task 能被 Agent 单独执行    |
| 可验证  | 每个 task 有验收标准            |
| 可排序  | 有依赖关系                    |
| 可回滚  | 一个 task 对应一个 run 或多个 run |
| 可审计  | 每个 task 有执行历史            |
| 范围清晰 | 明确允许修改范围                 |

---

## 6.3.2 task.json 生成过程

```text
Input PRD
  ↓
Requirement Extraction
  ↓
Module Mapping
  ↓
Task Splitting
  ↓
Acceptance Criteria Generation
  ↓
Constraint Generation
  ↓
Validation Mapping
  ↓
Dependency Ordering
  ↓
task.json
```

---

# 6.4 Run Agent Implementation Loop

## 6.4.1 目标

执行一个任务，从计划到代码修改再到验证。

---

## 6.4.2 Step-by-step

```text
1. User selects task
2. User clicks Run
3. System creates Git checkpoint
4. Planner checks or updates plan
5. Context Engine resolves context
6. Constraint Engine runs preflight
7. Implementer starts generation
8. Implementer reads files and calls tools
9. Implementer proposes patch
10. Hook after_generation runs
11. User approves patch if required
12. Patch applied
13. Validation runs
14. Reviewer reviews result
15. If failed, system starts iteration
16. If passed, task marked done
```

---

## 6.4.3 Sequence

```text
User
  │ Run Task
  ▼
Run Orchestrator
  │ create checkpoint
  ▼
Git Integration
  │
  ▼
Planner Agent
  │ generate plan
  ▼
Context Engine
  │ resolve context
  ▼
Constraint Engine
  │ preflight
  ▼
Implementer Agent
  │ generate patch
  ▼
Hook Engine
  │ after_generation
  ▼
Patch Engine
  │ apply patch
  ▼
Validation Runner
  │ run checks
  ▼
Reviewer Agent
  │ review
  ▼
Task Manager
  │ update status
```

---

# 6.5 Inject Skills / MCP Dynamically

## 6.5.1 目标

根据任务和执行状态动态启用 skills 和 MCP 数据。

---

## 6.5.2 Step-by-step

```text
1. Task starts
2. Context Engine analyzes task type
3. Skill Matcher finds relevant skills
4. MCP Matcher finds useful data sources
5. UI shows proposed injections
6. User approves or system auto-approves
7. MCP tools are called
8. Skill content is injected
9. Context snapshot updated
10. Agent continues
```

---

## 6.5.3 Example

任务：

```text
Debug model not found when using axonhub proxy.
```

动态注入：

| 类型      | 注入内容                       |
| ------- | -------------------------- |
| Skill   | provider-debugging-skill   |
| Rule    | model-provider-config-rule |
| MCP     | local-proxy-log-mcp        |
| File    | provider config            |
| Command | curl provider health check |

---

# 6.6 Debug “model not found” / Bad Output Issues

## 6.6.1 目标

提供专门的 AI 行为与 provider 调试体验。

---

## 6.6.2 model not found Debug Flow

```text
1. User sees model not found error
2. User clicks "Diagnose"
3. UI opens Provider Debug View
4. System checks active provider config
5. System checks model id
6. System checks provider routing
7. System checks proxy request log
8. System checks whether request reached proxy
9. System checks provider response
10. System suggests root cause
11. User applies fix
12. User retries run
```

---

## 6.6.3 Diagnostic Checklist

| 检查项                 | UI 展示       |
| ------------------- | ----------- |
| provider id 是否存在    | pass / fail |
| model id 是否为空       | pass / fail |
| model alias 是否映射    | pass / fail |
| proxy base URL 是否正确 | pass / fail |
| request 是否发出        | pass / fail |
| proxy 是否收到请求        | pass / fail |
| provider 是否返回错误     | pass / fail |
| auth 是否有效           | pass / fail |
| fallback 是否配置       | pass / fail |

---

## 6.6.4 Bad Output Debug Flow

```text
1. User marks output as bad
2. UI asks reason:
   - wrong behavior
   - style issue
   - missing test
   - wrong file modified
   - ignored requirement
3. System opens Run Diagnosis
4. Compare:
   - task
   - context
   - constraints
   - prompt
   - model
   - validation result
5. Reviewer Agent analyzes root cause
6. System suggests:
   - add context
   - strengthen rules
   - switch model
   - split task
   - add validation
```

---

# 6.7 Run Validation and Iterate

## 6.7.1 Step-by-step

```text
1. Patch applied
2. Validation starts automatically
3. Checks run sequentially or parallel
4. Results streamed to UI
5. Failed checks grouped
6. Reviewer Agent analyzes failures
7. User chooses:
   - auto fix
   - manual fix
   - ignore
   - create follow-up task
8. If auto fix:
   - failure context injected
   - implementer generates fix
   - validation reruns
9. Loop until pass or stopped
```

---

# 7. State & Data Model

---

# 7.1 Core Domain Model

```text
Workspace
  └── ProjectProfile
  └── Tasks
        └── Runs
              └── Agents
              └── ContextSnapshot
              └── Constraints
              └── Patches
              └── ValidationReports
              └── TimelineEvents
```

---

# 7.2 Task Model

```ts
type Task = {
  id: string
  workspaceId: string
  parentId?: string
  title: string
  description: string
  type: TaskType
  priority: Priority
  status: TaskStatus
  acceptanceCriteria: AcceptanceCriterion[]
  plan: PlanStep[]
  constraints: Constraint[]
  contextRefs: ContextReference[]
  validation: ValidationRequirement[]
  dependencies: string[]
  runHistory: string[]
  createdAt: string
  updatedAt: string
}
```

```ts
type TaskStatus =
  | "draft"
  | "ready"
  | "running"
  | "blocked"
  | "done"
  | "failed"
  | "cancelled"
```

---

# 7.3 Agent State Machine

```text
Idle
  ↓
Preparing
  ↓
ResolvingContext
  ↓
WaitingForModel
  ↓
Thinking
  ↓
CallingTool
  ↓
GeneratingPatch
  ↓
WaitingApproval
  ↓
ApplyingPatch
  ↓
Validating
  ↓
Reviewing
  ↓
Completed

Failure path:
  Any State → Failed → Retry / Abort / Rollback
```

---

## 7.3.1 Agent State Definition

```ts
type AgentRunState =
  | "idle"
  | "preparing"
  | "resolving_context"
  | "waiting_for_model"
  | "thinking"
  | "calling_tool"
  | "generating_patch"
  | "waiting_approval"
  | "applying_patch"
  | "validating"
  | "reviewing"
  | "completed"
  | "failed"
  | "paused"
  | "cancelled"
```

---

# 7.4 Context Model

```ts
type ContextModel = {
  snapshotId: string
  runId: string
  taskId: string
  model: string
  maxTokens: number
  usedTokens: number
  items: ContextItem[]
  createdAt: string
}
```

```ts
type ContextSource =
  | "repo"
  | "mcp"
  | "knowledge"
  | "rule"
  | "task"
  | "git"
  | "validation"
  | "user"
```

---

# 7.5 Execution Log Model

```ts
type ExecutionLog = {
  runId: string
  events: ExecutionEvent[]
}
```

```ts
type ExecutionEvent = {
  id: string
  timestamp: string
  type: string
  source: string
  severity: "debug" | "info" | "warning" | "error"
  payload: unknown
  traceId?: string
  parentEventId?: string
}
```

---

# 7.6 Run Versioning

每次执行生成一个 Run。

```ts
type Run = {
  id: string
  taskId: string
  workspaceId: string
  version: number
  status: RunStatus
  agentConfig: AgentConfig
  contextSnapshotId: string
  constraintSnapshotId: string
  gitCheckpointId?: string
  patchSetId?: string
  validationReportId?: string
  startedAt: string
  endedAt?: string
}
```

```ts
type RunStatus =
  | "created"
  | "running"
  | "waiting_user"
  | "validating"
  | "passed"
  | "failed"
  | "cancelled"
```

---

# 8. Extensibility Design

---

# 8.1 Plugin System

## 8.1.1 插件类型

| Plugin Type         | 描述            |
| ------------------- | ------------- |
| Context Provider    | 提供上下文         |
| Validation Provider | 提供验证能力        |
| Tool Provider       | 提供工具调用        |
| UI Panel Plugin     | 新增 UI 面板      |
| Agent Plugin        | 新 Agent role  |
| Model Provider      | 新模型接入         |
| MCP Adapter         | MCP server 管理 |
| Rule Pack           | 规则包           |
| Skill Pack          | 技能包           |

---

## 8.1.2 Plugin Manifest

```json
{
  "id": "rust-quality-pack",
  "name": "Rust Quality Pack",
  "version": "0.1.0",
  "type": "skill_pack",
  "contributes": {
    "skills": ["skills/rust-refactor.md"],
    "rules": ["rules/rust-quality.md"],
    "commands": ["commands/cargo-clippy.json"],
    "hooks": ["hooks/rust-validation.json"]
  }
}
```

---

# 8.2 MCP Integration Model

```text
MCP Server
  ↓
MCP Client
  ↓
MCP Tool Registry
  ↓
Permission Engine
  ↓
Context Engine / Agent Tool Router
  ↓
Agent Runtime
```

---

## 8.2.1 MCP Permission Model

| 权限                  | 描述          |
| ------------------- | ----------- |
| `read_context`      | 读取资源作为上下文   |
| `call_tool`         | 调用 MCP tool |
| `write_external`    | 写外部系统       |
| `requires_approval` | 需要用户审批      |
| `task_scoped`       | 仅当前任务可用     |

---

# 8.3 Skill Lifecycle

```text
Author
  ↓
Validate
  ↓
Register
  ↓
Bind to Workspace / Task
  ↓
Match Trigger
  ↓
Inject / Execute
  ↓
Log
  ↓
Evaluate Effectiveness
```

---

## 8.3.1 Skill Format

```markdown
---
id: rust-test-failure-fixer
name: Rust Test Failure Fixer
trigger:
  taskTypes:
    - bugfix
    - test
  validationFailures:
    - cargo test
---

# Purpose

Help the implementer fix Rust test failures.

# Instructions

1. Read the failing test output.
2. Identify the minimal failing behavior.
3. Inspect related source files.
4. Apply smallest safe fix.
5. Do not weaken tests unless explicitly approved.
```

---

# 8.4 Hook Injection Points

## 8.4.1 before_generation

### 触发时机

Agent 开始生成代码前。

### 输入

```ts
{
  task: Task
  contextSnapshot: ContextSnapshot
  constraints: Constraint[]
  agentRole: AgentRole
}
```

### 用途

* 注入规则。
* 检查上下文是否完整。
* 检查任务是否过大。
* 检查是否需要用户确认。
* 加载相关 skill。

### 示例动作

```text
before_generation
  → check required context
  → inject rust coding rules
  → block if no validation command
```

---

## 8.4.2 after_generation

### 触发时机

Agent 生成 patch 之后，应用 patch 之前。

### 用途

* 扫描 diff。
* 检查是否修改 forbidden path。
* 检查是否新增依赖。
* 检查是否删除测试。
* 让 reviewer 预审。

### 示例动作

```text
after_generation
  → run dangerous diff scan
  → run architecture boundary check
  → request user approval if risk high
```

---

## 8.4.3 before_validation

### 触发时机

patch 应用后，验证执行前。

### 用途

* 准备测试环境。
* 检查测试文件是否存在。
* 检查依赖是否安装。
* 生成临时测试配置。

---

## 8.4.4 after_validation

### 触发时机

验证完成后。

### 用途

* 总结失败。
* 创建 failure context。
* 触发修复 loop。
* 生成 validation report。
* 阻止 commit。

---

## 8.4.5 Hook Execution Contract

```ts
type HookExecutionResult = {
  status: "passed" | "failed" | "blocked" | "skipped"
  message?: string
  generatedContext?: ContextItem[]
  suggestedActions?: SuggestedAction[]
  blockingReason?: string
}
```

---

# 9. Non-Functional Requirements

---

# 9.1 Performance

## 9.1.1 Large Repo Handling

| 要求             | 指标                   |
| -------------- | -------------------- |
| 初次 repo scan   | 100k files 内可完成并显示进度 |
| 增量索引           | 文件变更后秒级更新            |
| UI 响应          | 常规操作 < 100ms         |
| 大 diff 展示      | 支持 10k 行 diff 分块加载   |
| Timeline       | 支持单 run 10k events   |
| Context search | 本地索引搜索 < 500ms       |

---

## 9.1.2 策略

* Lazy loading file tree。
* Incremental repo index。
* Ignore `.git/`, `target/`, `node_modules/`, build outputs。
* Token count cache。
* Virtualized list rendering。
* Streaming event processing。
* Background workers。

---

# 9.2 Offline Capability

## 9.2.1 离线支持范围

| 能力             | 离线支持 |
| -------------- | ---- |
| 打开 workspace   | 支持   |
| 查看任务           | 支持   |
| 查看 run history | 支持   |
| 查看日志           | 支持   |
| 查看 diff        | 支持   |
| 本地模型           | 支持   |
| 云模型            | 不支持  |
| MCP 本地 server  | 支持   |
| 远程 MCP         | 不支持  |
| Validation     | 支持   |

---

# 9.3 Security

## 9.3.1 本地代码安全

要求：

* 默认只访问 workspace root。
* 访问 workspace 外文件需要审批。
* secret 文件默认隐藏。
* `.env` 默认不注入上下文。
* API key 不进入 prompt。
* 日志脱敏。
* 可清空 run traces。

---

## 9.3.2 API Key 管理

| 要求              | 描述             |
| --------------- | -------------- |
| 本地安全存储          | 使用系统 keychain  |
| 不写入普通配置文件       | 禁止明文持久化        |
| Provider scoped | 每个 provider 独立 |
| 支持 OAuth        | browser auth   |
| 支持 proxy token  | agent proxy 模式 |

---

## 9.3.3 Dangerous Actions

以下操作默认需要审批：

* 删除文件。
* 修改大量文件。
* 执行 shell command。
* 访问 workspace 外目录。
* 新增依赖。
* 修改 CI/CD 文件。
* 修改 auth/security 相关代码。
* 调用外部写操作 MCP。

---

# 9.4 Observability

## 9.4.1 必须记录

| 类型          | 内容                           |
| ----------- | ---------------------------- |
| Agent event | 状态、输入、输出摘要                   |
| Tool call   | tool name、args、result        |
| Model call  | provider、model、latency、error |
| Context     | snapshot、token               |
| Validation  | command、output、status        |
| Git         | diff、checkpoint              |
| Hook        | trigger、result               |
| MCP         | server、tool、response         |

---

# 9.5 Debuggability of AI Behavior

必须支持：

* 查看 prompt snapshot。
* 查看 context snapshot。
* 查看 model request metadata。
* 查看 tool call chain。
* 查看 Agent handoff。
* 比较两次 run。
* 标记 bad output。
* 生成 diagnosis report。

---

# 9.6 Multi-model Support

## 9.6.1 Provider Requirements

必须支持：

* OpenAI-compatible API。
* Kimi。
* GLM。
* Minimax。
* 本地模型。
* 自定义 proxy。
* Model alias。
* Per-agent model selection。
* Fallback model。
* Provider debug trace。

---

## 9.6.2 Provider Model

```ts
type ProviderConfig = {
  id: string
  name: string
  type: "openai_compatible" | "kimi" | "glm" | "minimax" | "local" | "custom"
  baseUrl?: string
  authMode: "api_key" | "oauth" | "browser" | "none"
  models: ModelConfig[]
  defaultModel?: string
}
```

---

# 10. UX Details

---

# 10.1 Layout

## 10.1.1 主布局

```text
┌──────────────────────────────────────────────────────────────┐
│ Top Bar: Workspace | Branch | Provider | Run Status          │
├───────────────┬──────────────────────────────┬───────────────┤
│ Left Sidebar  │ Main Work Area               │ Right Inspector│
│               │                              │               │
│ Workspace     │ Agent / Task / Code / Git    │ Context       │
│ Tasks         │                              │ Constraints   │
│ Runs          │                              │ Validation    │
│ MCP           │                              │ Timeline      │
│ Skills        │                              │ Details       │
└───────────────┴──────────────────────────────┴───────────────┘
```

---

## 10.1.2 推荐 Pane

| 区域              | 内容                              |
| --------------- | ------------------------------- |
| Top Bar         | 当前项目、分支、模型、状态                   |
| Left Sidebar    | workspace、task、run、extension 导航 |
| Main Area       | 当前主任务：Agent / Task / Code / Git |
| Right Inspector | 上下文、约束、验证、timeline              |
| Bottom Drawer   | logs、terminal、trace             |

---

# 10.2 Keyboard-first Interactions

必须支持：

| 快捷键               | 动作                       |
| ----------------- | ------------------------ |
| `Cmd+K`           | Command palette          |
| `Cmd+Enter`       | Run current task         |
| `Cmd+Shift+Enter` | Run validation           |
| `Cmd+.`           | Stop current run         |
| `Cmd+P`           | Open file                |
| `Cmd+Shift+P`     | Open task                |
| `Cmd+J`           | Toggle bottom log drawer |
| `Cmd+Option+C`    | Open context panel       |
| `Cmd+Option+V`    | Open validation panel    |

---

# 10.3 Visual Hierarchy

优先级：

```text
1. Current task status
2. Blocking error / user approval needed
3. Active Agent state
4. Validation result
5. Context / constraints summary
6. Logs and details
```

---

# 10.4 Error States

## 10.4.1 错误类型

| 错误                     | UI 行为                 |
| ---------------------- | --------------------- |
| Provider error         | 打开 Provider Diagnosis |
| Context too large      | 打开 Context Budget     |
| Constraint violation   | 打开 Constraints        |
| Validation failed      | 打开 Validation         |
| MCP disconnected       | 打开 MCP Panel          |
| Git dirty conflict     | 打开 Git Panel          |
| Tool permission denied | 显示 approval           |
| Patch failed           | 显示 patch conflict     |

---

## 10.4.2 Error Card 示例

```text
Model Not Found

Provider: axonhub
Model: minimax-m2.5-free
Request sent: No
Proxy log found: No

Likely cause:
The request did not reach the proxy. Check provider routing or base URL.

Actions:
[Open Provider Config] [Test Request] [View Trace] [Retry]
```

---

# 10.5 Loading / Streaming Behavior

## 10.5.1 Streaming

必须支持：

* Agent message streaming。
* Tool call status streaming。
* Validation output streaming。
* Timeline real-time append。
* Token usage live update。
* Long command output folding。

---

## 10.5.2 Loading State

| 场景              | UI                          |
| --------------- | --------------------------- |
| Repo scan       | progress bar + current path |
| Context resolve | source-by-source loading    |
| Model response  | streaming text              |
| Validation      | per-check spinner           |
| MCP call        | request card                |
| Git diff        | skeleton + lazy load        |

---

# 11. MVP Scope vs Future Iterations

---

# 11.1 V1 Must Have

## 11.1.1 Core Product

| 模块                      | V1 要求                               |
| ----------------------- | ----------------------------------- |
| Workspace Manager       | 本地 repo 打开、初始化、readiness            |
| Agent Interaction Panel | task-based agent interaction        |
| Task Manager            | task.json 创建、编辑、运行                  |
| Context Inspection      | 查看注入上下文                             |
| Generation Constraints  | 基础 rules、scope constraints          |
| Output Validation       | cargo check/test/clippy/fmt         |
| Multi-Agent View        | planner / implementer / reviewer 状态 |
| Code Explorer           | 文件树、diff、read-only code             |
| Git Panel               | status、diff、checkpoint、rollback     |
| MCP Panel               | server 管理、tool 查看、调用日志              |
| Skills / Hooks / Rules  | 基础管理                                |
| Timeline                | run event log                       |

---

## 11.1.2 V1 Agent Capabilities

| 能力             | 要求                                              |
| -------------- | ----------------------------------------------- |
| Planner        | PRD → plan / task.json                          |
| Implementer    | 根据 task 修改代码                                    |
| Reviewer       | review diff + validation failure                |
| Context Engine | repo + rules + task context                     |
| Validation     | Rust 项目默认命令                                     |
| Provider       | OpenAI-compatible + custom proxy                |
| Hook           | before/after generation、before/after validation |

---

## 11.1.3 V1 平台范围

| 项目             | 要求                                 |
| -------------- | ---------------------------------- |
| OS             | macOS first                        |
| Language focus | Rust first                         |
| Repo           | local Git repo                     |
| UI framework   | Desktop-native or webview-based 均可 |
| Data storage   | local SQLite                       |
| Config         | `.opencode-rs/`                    |
| Logs           | local run store                    |

---

# 11.2 V1 Deferred

| 能力                      | 延后原因                |
| ----------------------- | ------------------- |
| 完整 IDE 编辑器              | 产品不是 IDE            |
| 多人实时协作                  | 复杂度高                |
| Cloud workspace         | 先做本地                |
| PR 自动创建                 | GitHub/GitLab 集成可后置 |
| 高级成本分析                  | provider 稳定后做       |
| Agent marketplace       | 生态成熟后做              |
| Visual workflow builder | 初期用配置即可             |
| 全平台支持                   | macOS 先验证体验         |
| 复杂 DevOps 集成            | 先支持命令级 validation   |

---

# 11.3 V2 Roadmap

## V2.1 Team Mode

* 团队规则包。
* 共享 skill pack。
* 共享 run report。
* 团队级 provider policy。
* Workspace template。

## V2.2 DevOps Integration

* GitHub PR。
* GitLab MR。
* CI failure MCP。
* Issue tracker MCP。
* Release workflow。

## V2.3 Advanced Agent Debugging

* Prompt diff。
* Context diff。
* Model output comparison。
* Agent replay。
* Deterministic run mode。

## V2.4 Remote Control

* Mobile companion。
* Remote run monitor。
* Approve patch from mobile。
* Voice input task creation。

## V2.5 Cross-platform

* Windows。
* Linux。
* Remote SSH workspace。

---

# 12. Implementation Guidance for AI Coding

## 12.1 Recommended Implementation Milestones

### Milestone 1：Workspace + Local State

目标：

* 打开 repo。
* 扫描项目。
* 保存 workspace。
* 展示 readiness。

核心数据：

* Workspace
* ProjectProfile
* GitState

---

### Milestone 2：Task Manager + task.json

目标：

* 创建任务。
* 编辑 task.json。
* 展示任务树。
* 支持任务状态。

核心数据：

* Task
* PlanStep
* AcceptanceCriteria

---

### Milestone 3：Run Orchestrator Skeleton

目标：

* 创建 run。
* 状态机流转。
* Timeline 记录。
* Mock Agent events。

核心数据：

* Run
* AgentState
* ExecutionEvent

---

### Milestone 4：Context Inspection

目标：

* repo 文件加入上下文。
* 展示 context snapshot。
* token 估算。
* include / exclude。

---

### Milestone 5：Agent Interaction

目标：

* 与 Agent Runtime 交互。
* 显示 planner / implementer / reviewer。
* 展示 tool call。
* 支持 pause / stop / retry。

---

### Milestone 6：Validation

目标：

* 执行 cargo commands。
* 解析结果。
* 展示 validation report。
* 失败后生成 fix loop。

---

### Milestone 7：Git + Diff

目标：

* Git status。
* Diff viewer。
* Checkpoint。
* Rollback。

---

### Milestone 8：MCP + Skills + Hooks

目标：

* 管理 MCP server。
* 管理 hooks。
* before/after generation。
* before/after validation。
* hook execution logs。

---

# 13. Acceptance Criteria for V1

## 13.1 Workspace

* 用户可以打开本地 Rust Git repo。
* 系统可以识别 Cargo 项目。
* 系统可以生成 `.opencode-rs/`。
* 系统可以展示 AI Coding readiness。

---

## 13.2 Task

* 用户可以创建 task。
* 用户可以从 PRD 生成 task.json。
* 用户可以编辑 task.json。
* 用户可以运行单个 task。
* task 可以关联 acceptance criteria。

---

## 13.3 Agent

* UI 可以显示 planner / implementer / reviewer。
* 每个 Agent 有明确状态。
* 用户可以暂停、停止、重试 run。
* 用户可以查看 Agent handoff。

---

## 13.4 Context

* 用户可以查看当前注入上下文。
* 用户可以看到 context 来源和 token 占用。
* 用户可以 include / exclude context item。
* 每次 run 保存 context snapshot。

---

## 13.5 Constraints

* 用户可以设置允许 / 禁止修改路径。
* 用户可以启用 / 禁用 rules。
* 生成前执行 constraint preflight。
* 违反 blocking constraint 时 run 被阻止。

---

## 13.6 Validation

* 系统可以运行 `cargo check`。
* 系统可以运行 `cargo test`。
* 系统可以运行 `cargo clippy`。
* 系统可以展示失败详情。
* 失败后可以让 Agent 继续修复。

---

## 13.7 Git

* 系统可以显示 Git status。
* 系统可以显示 diff。
* 系统可以创建 checkpoint。
* 系统可以 rollback run。

---

## 13.8 MCP

* 用户可以添加 MCP server。
* 系统可以显示 MCP tools。
* 系统可以记录 MCP call。
* MCP 数据可以进入 context snapshot。

---

## 13.9 Hooks

* 支持 `before_generation`。
* 支持 `after_generation`。
* 支持 `before_validation`。
* 支持 `after_validation`。
* Hook 执行结果进入 timeline。

---

## 13.10 Timeline

* 每个 run 都有 timeline。
* Timeline 记录 agent、tool、context、validation、git、hook 事件。
* 用户可以展开事件详情。
* 用户可以导出 run trace。

---

# 14. Summary

opencode-rs Desktop UI 的核心不是“让用户和 AI 聊天”，而是构建一个面向真实工程开发的 AI Coding Control Center。

它必须显式管理：

```text
Task
Context
Constraints
Agents
Tools
MCP
Skills
Hooks
Git
Validation
Timeline
```

它的核心竞争力在于：

1. **Agent-first**：围绕任务和 Agent，而不是围绕文件编辑器。
2. **Context Transparency**：让用户知道 AI 到底看到了什么。
3. **Generation Constraints**：让 AI 在工程边界内生成。
4. **Output Validation**：让生成结果必须经过质量门禁。
5. **Multi-Agent Orchestration**：让 planner / implementer / reviewer 协作可见。
6. **Debuggable AI Behavior**：让模型、上下文、工具、规则、输出都可追踪。
7. **Extensible Runtime**：通过 MCP / skills / commands / hooks / rules 扩展系统能力。

最终产品形态应当是：

> 一个桌面化、可观察、可控制、可扩展、可验证的 AI Coding 工程操作系统。
