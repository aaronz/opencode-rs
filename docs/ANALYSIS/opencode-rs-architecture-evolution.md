# opencode-rs 架构演进分析：一场"假 Runtime"到"真 Runtime"的重构之旅

## 开篇：项目定位与演进概览

opencode-rs 是一个用 Rust 重写 TypeScript 版 opencode 的项目，目标是为 AI Coding Agent 提供一个生产级别的运行时基础设施。项目起始于 2025 年，历经一年多的演进，从早期的"功能 parity 追赶"逐步走向"架构深度重构"阶段。

当前项目处于**架构重组的技术成熟期**。过去三周（2026 年 4 月下旬至 5 月初），项目集中爆发了一次大规模重构（commit range: `27930e2b...c3c2223e`，约 82 个文件、7500+ 行变更），核心任务是解决一个根本性架构问题：**项目的 TUI 曾经就是 Runtime，而 Runtime 只是一个任务存储**。

这次重构标志着项目从"功能可用"向"架构健康"的关键转折。

---

## 主线一：功能迭代脉络

### Phase 1: Parity 阶段（2025 年初 - 2025 年中）

**动机**：用 Rust 重写 TypeScript 项目，首要目标是功能对等。

这一阶段的 commit 历史清晰地记录了追赶痕迹：

```
7e1f1041 initial version
79f31155 2nd
ae1f00c7 feat: complete feature parity gap fill between TS and Rust port
2630d00c feat: implement advanced tools, tui components, and e2e test harness for full parity
65b0a897 chore: archive sync-with-opencode-target change
f2dcd11c feat: Achieve full parity with OpenCode
7a10fac7 feat: complete full feature parity between TS and Rust port
0ed6238f chore: archive achieve-full-parity-with-opencode change
b2c40eec feat: implement TUI dialogs (settings, model selection, providers, file/directory selection, release notes)
1d5e27b5 feat: add File Tree component with toggle (Ctrl+Shift+F)
7facf634 feat: add Title Bar with session history dropdown (Ctrl+H)
c3e7f745 feat: add Status Bar with popover system
44979f2d feat: add comprehensive E2E test suite
c2630c51 chore: update tasks.md marking completed work
42e0128a feat: add Terminal Panel component (Ctrl+~)
8f11a891 feat: add InputWidget component with multiline and history support
c56f7473 feat: add VirtualList component for efficient large list rendering
aa952647 feat(config): add KeybindConfig and ThemeConfig to Config struct
cf2f58de feat(config): add validation, save, and TS migration support
e9008038 chore(openspec): archive completed bridge-remaining-gaps-with-opencode change
```

**特点**：
- 以 `feat:` 为主要 commit 类型，聚焦 TUI 组件、工具实现、测试框架
- 大量 `feat: add FileTree`, `feat: add Terminal Panel`, `feat: add StatusBar` 这类具体功能 commit
- 设计决策：优先实现 TypeScript 版本已有的功能，架构问题暂时搁置

**功能里程碑分析**：

| 里程碑 | Commit | 动机 | 方案选择 |
|--------|--------|------|----------|
| 初始版本 | `7e1f1041` | 建立 Rust 版本基础 | 最小化可运行版本 |
| 功能 parity | `ae1f00c7`, `f2dcd11c` | 与 TS 版本功能对等 | 逐功能翻译 |
| TUI 组件 | `b2c40eec`, `1d5e27b5` | 提供交互界面 | Ratatui + 虚拟列表 |
| 测试框架 | `44979f2d` | 验证功能正确性 | E2E 测试 + TUI testing |

**反直觉发现 #1**：项目在功能 parity 阶段刻意保留了 TypeScript 版本的架构设计模式，即使知道这些模式在 Rust 中不是最佳实践。这是因为团队策略是"先迁移后优化"，避免在追赶阶段引入不必要的重构风险。

这种策略在短期内是合理的：维持功能 parity 是首要目标，架构优化是锦上添花。但代价是积累技术债务——后续三周的重构工作量（82 文件、7500+ 行变更）某种程度上是这段"快速迁移期"的机会成本。

### Phase 2: 稳定性工程（2025 年中 - 2026 年初）

**动机**：功能 parity 完成后，开始处理积累的技术债务和稳定性问题。

这一阶段 commit 风格发生变化，越来越多 `fix:` 和 `chore:` 类型的出现：

```
0392ff74 fix: resolve startup slowness, screen clear on exit, and path conflicts
d702f1b0 fix(tui): limit sidebar file tree depth to fix 2+ minute startup
8a4eb9f9 fix(logging): disable ANSI color codes in log file output
4bf14d00 fix(logging): fix RollingFileAppender arguments - log dir was created as file
f070b094 fix: clippy lint and formatting fixes across agent, config, logging crates
945292bb fix(llm): strip models-dev- prefix in ModelRegistry and update stale tests
20b24a85 fix(llm): add 30s timeout to MiniMax and Ollama providers
027eeffd fix(cli,tui): complete runtime streaming and Ollama flows
```

**关键观察**：这个阶段开始出现"启动慢"和"目录冲突"这类问题——它们是架构问题的症状，而非根本原因。sidebar 文件树深度导致 2+ 分钟启动时间，暗示着低效的目录遍历或不必要的阻塞操作。

**稳定性问题的分类**：

1. **性能问题**：`d702f1b0` 揭示的 2+ 分钟启动问题
2. **资源清理问题**：`0392ff74` 中的 screen clear on exit
3. **路径冲突**：`0392ff74` 中的 path conflicts
4. **日志损坏**：`4bf14d00` 中 RollingFileAppender 将目录创建为文件
5. **超时问题**：`20b24a85` 为 MiniMax 和 Ollama 添加 30s 超时

这些问题的共同特点是：它们是"最后一步"才暴露出来的边缘情况。在快速开发阶段，代码路径假设happy path，忽略错误处理和边界条件。

**Phase 2 的另一个重要工作**是 LLM Provider 的完善：

```
945292bb fix(llm): strip models-dev- prefix in ModelRegistry and update stale tests
20b24a85 fix(llm): add 30s timeout to MiniMax and Ollama providers
027eeffd fix(cli,tui): complete runtime streaming and Ollama flows
```

**Provider 架构的演进**：从 TypeScript 版本继承的 provider 层，在 Rust 版本中经历了从"直接实现"到"抽象 Gateway"的转变。`8b971f8a` (normalize ProviderGateway trait) 是这个转变的里程碑。

### Phase 3: 架构重构（2026 年 4 月 - 至今）

**动机**：一个深度的 design document 分析揭示了核心架构问题。

这是当前正在进行的关键阶段。`docs/DESIGN/agent-runtime-design.md` 定义了 Reference Architecture，而 `docs/ISSUE/ARCHITECTURE_ISSUES.md` 则记录了对现有代码的审查结果。

**核心发现**（commit `d9037809` 引入了实现计划）：

```
The TUI is the runtime, and Runtime is just a task store.

The TUI:
- Instantiates AgentRuntime directly
- Handles LLM events via callbacks
- Executes shell commands directly
- Has #[allow(dead_code)] runtime field that was never wired up

The Runtime struct:
- Creates tasks and sessions
- Saves state
- Returns "accepted" without executing
```

这解释了为什么之前修修补补总是治标不治本——TUI 和 Runtime 的边界混乱导致每次修复都只是把问题移到另一个地方。`d702f1b0` 修复 2+ 分钟启动问题只是症状治疗，`0392ff74` 修复 path conflicts 也是局部优化，真正的病因是 TUI 越权承担了 Runtime 的职责。

**这个发现的重要性**：它不是某一次 commit 的产物，而是通过系统性的代码审查（ISSUE-001 到 ISSUE-014）累积得出的结论。单独看任何一个 issue，可能只是中等问题；但把它们放在一起，揭示的是一个方向性的架构错误。

### Phase 3 的 commit 序列分析

这一阶段的关键 commit 按照依赖关系可以分为几个批次：

**批次一：Runtime Facade 基础**（4月28日-29日）

```
27930e2b feat(runtime): add runtime facade and turn-aware session bridge
2841beab feat(server): wire server state through runtime facade
fae1340a feat(server): route stream messages through runtime events
b166af7e feat(server): bridge execute events into runtime events
```

这个批次的共同主题是"桥接"——将之前散落在 TUI 和 Server 中的 Runtime 相关逻辑集中到 Runtime Facade 层面。`RuntimeFacade` 是一个新的抽象层，它接收来自 UI 和 Server 的命令，委托给 Runtime Core 处理。

**批次二：本地 Provider 连接**（4月29日-30日）

```
24dfa5a8 feat(llm): add local auth for local model providers
b54467fb feat(tui): add local connect method for local providers
e53d0011 feat(tui): add local model connection flow
```

这是用户可见的功能改进，支持本地模型（如 Ollama）的连接。需要特别指出的是，`e53d0011` 添加的"本地模型连接流程"不仅仅是一个 UI 改进——它涉及到 Provider Gateway 如何处理本地认证（local auth）与远程 OAuth 的差异。

**批次三：核心重构**（4月30日-5月1日）

```
8b971f8a feat(runtime): normalize ProviderGateway trait
dea2fd50 feat(runtime): add explicit RuntimeStatus state machine
85f0be8a feat(core): add hook engine for lifecycle event processing
ba9130a5 feat(runtime): add HookEngine to RuntimeFacadeServices
46895689 feat(core): add CommandWorkflow and CommandStep types
b95955d3 feat(agent): use chat_with_tools() with tool schemas in agents
5000fa32 feat(llm): add ToolSchema and chat_with_tools for provider-native tool calling
5869e956 feat(cli): add context inspectability commands
321cb70e fix(core): properly compute preserve_from in trim_to_budget ranking
```

这个批次是重构的核心。`8b971f8a` 的 ProviderGateway 规范化是 ISSUE-005 的修复基础；`dea2fd50` 的状态机是 ISSUE-013 的修复基础；`5000fa32` 的 ToolSchema 则是 ISSUE-005 的完整解决方案。

**批次四：完善基础设施**（5月1日）

```
56c299c6 feat(runtime): add TraceStore and StructuredLog events
c3cd92e1 feat(runtime): add PathResolver trait and ToolRouter validation
2898c972 feat(runtime): add testing infrastructure
613b7d36 chore(runtime): add DomainEvent variants and error types
```

这是重构的收尾阶段，添加可观测性（TraceStore）、可测试性（testing infrastructure）和边界验证（PathResolver）。这些不是核心功能，但为未来的稳定性提供了基础设施。

---

## 主线二：架构优化与重构

### 转折点：从设计文档到实施计划

2026 年 4 月 28 日的 commit `d9037809` 引入了一个关键文档：`docs/DESIGN/agent-runtime-design.md`。这个 Reference Architecture 不是凭空设计的，而是基于对现有代码的批判性分析。

**架构设计原则**（Section 3.1）定义了 7 个核心原则：

1. **UI 与 Runtime 严格分离** - Runtime 不应该知道自己运行在 CLI/TUI/Desktop 中
2. **Runtime Core 应该事件驱动** - 关键动作都应发布事件
3. **Lifecycle 必须显式** - 明确的状态机（14 个状态）
4. **Deterministic Core, Non-deterministic Boundary** - 核心确定，边界可替换
5. **Provider Complexity 不应泄漏到 Runtime Core** - Provider 差异在 Gateway 层消化
6. **Tool Execution 必须安全、可审计、可回滚**
7. **Context 必须可解释、可检查、可重放**

### 重构前的技术债务清单

`docs/ISSUE/ARCHITECTURE_ISSUES.md` 记录了 14 个确认问题，按严重性和类别分组：

| Issue | Severity | 问题描述 | 分类 |
|-------|----------|----------|------|
| ISSUE-001 | HIGH | TUI 直接实例化 AgentRuntime，违反设计原则 | TUI/Runtime |
| ISSUE-002 | HIGH | Shell 执行绕过 Runtime 的 ToolRouter | TUI/Runtime |
| ISSUE-005 | HIGH | Tool schemas 以明文嵌入，而非通过 Provider API | Provider |
| ISSUE-013 | HIGH | Runtime 不执行 Agent Loop，Loop 在 TUI 的线程中运行 | Runtime |
| ISSUE-012 | MEDIUM | AgentRuntime 使用回调而非 EventBus | Events |
| ISSUE-014 | MEDIUM | RuntimeHandle 每次 execute 都克隆 Runtime | Runtime |
| ISSUE-008 | MEDIUM | trim_to_budget 边缘情况错误 | Context |
| ISSUE-009 | MEDIUM | CLI 缺少 context inspect 命令 | Context |
| ISSUE-011 | MEDIUM | PermissionResponse 命令未实现 | Runtime |
| ISSUE-003 | INFO | TUI 写入 config 文件 | TUI/Runtime |
| ISSUE-004 | INFO | TUI 进行 provider API 调用 | TUI/Runtime |
| ISSUE-007 | INFO | LlmError 结构问题 | Provider |

**ISSUE-001 和 ISSUE-013 的根因分析**：

这两个 HIGH severity 问题指向同一个根本原因：架构边界的倒置。正常的设计应该是：

```
UI Layer -> Runtime Core -> Infrastructure
```

但实际架构是：

```
TUI (includes AgentRuntime) -> Runtime (task store only)
```

TUI 不应该实例化 AgentRuntime，AgentRuntime 应该由 Runtime 创建并管理。但当时的实现中，TUI 直接创建 AgentRuntime 并在线程中运行它的循环，而 Runtime 只是存储任务状态。

### 重构的具体实施

**第一步：建立测试基础设施**（commit `2898c972`）

```rust
// crates/runtime/src/testing/ 目录结构
testing/
├── mod.rs
├── fake_provider_gateway.rs    // Provider Gateway 的假实现
├── fake_shell_executor.rs     // Shell 执行器的假实现
├── in_memory_state_store.rs   // 内存状态存储（测试用）
└── recording_event_sink.rs    // 事件记录器（用于测试断言）
```

引入测试替身，使得后续重构可以验证行为不变性。这是一个经典的"测试先行"策略——在重构前建立回归测试网络。

**第二步：添加 HookEngine**（commit `85f0be8a`, `ba9130a5`）

HookEngine 是生命周期事件处理的基础设施，让 Runtime Core 可以被扩展而不修改核心代码。

```rust
// HookEngine 的核心抽象
pub trait HookHandler: Send + Sync {
    fn on_agent_start(&self, session: &Session);
    fn on_agent_complete(&self, session: &Session, result: &Result);
    fn on_tool_call(&self, tool: &str, args: &Value);
    fn on_llm_request(&self, prompt: &str);
    fn on_llm_response(&self, response: &str);
}
```

这种设计允许：
- 日志系统挂接 Hook 来记录完整执行轨迹
- 监控系用挂接 Hook 来收集指标
- 测试系统挂接 Hook 来验证特定事件发生

**第三步：规范化 ProviderGateway**（commit `8b971f8a`）

统一 Provider 接口，隐藏不同 LLM 提供商（OpenAI, Anthropic, Ollama, MiniMax）的差异。

```rust
// ProviderGateway trait 的规范化签名
pub trait ProviderGateway: Send + Sync {
    async fn complete(&self, request: ProviderRequest) -> Result<ProviderResponse, ProviderError>;
    async fn complete_streaming(&self, request: ProviderRequest) -> Result<ProviderStream, ProviderError>;
    fn get_capabilities(&self) -> ModelCapabilities;
}
```

之前的实现中，各个 Provider 的接口不统一，导致 Runtime Core 需要知道 provider-specific 的细节。规范化后，Runtime 只需要面对统一的 Gateway 接口。

**第四步：添加显式 RuntimeStatus 状态机**（commit `dea2fd50`）

从隐式状态转为显式状态机，使得 TUI 可以正确显示进度，日志可以清晰记录，测试可以断言。

```rust
pub enum RuntimeStatus {
    Idle,
    Preparing,
    BuildingContext,
    CallingModel,
    WaitingForPermission,
    ExecutingTool,
    ApplyingPatch,
    RunningCommand,
    Validating,
    Summarizing,
    Persisting,
    Completed,
    Failed,
    Cancelled,
    Interrupted,
}
```

**为什么需要显式状态机**：

AI Coding Agent 的执行流程比普通应用复杂得多——它涉及 LLM 调用、用户交互（权限审批）、工具执行、文件修改等阶段。如果状态不明确：

- TUI 无法准确显示当前进度
- 日志无法提供有意义的上下文
- 测试无法断言特定状态是否达成
- "恢复执行"功能无法确定从哪个状态恢复

**第五步：添加 TraceStore 和 CheckpointStore**（commit `56c299c6`, `d0019343`）

可观测性和可恢复性的基础设施。

```rust
// TraceStore 提供了执行轨迹的持久化
pub trait TraceStore: Send + Sync {
    async fn record_event(&self, event: &DomainEvent) -> Result<(), TraceError>;
    async fn get_trace(&self, session_id: &str) -> Result<Vec<DomainEvent>, TraceError>;
    async fn get_events_since(&self, session_id: &str, after: EventId) -> Result<Vec<DomainEvent>, TraceError>;
}

// CheckpointStore 提供了状态快照
pub trait CheckpointStore: Send + Sync {
    async fn save_checkpoint(&self, session: &Session) -> Result<CheckpointId, CheckpointError>;
    async fn load_checkpoint(&self, id: &CheckpointId) -> Result<Session, CheckpointError>;
}
```

这两个 trait 的组合是实现"可恢复执行"的基础——当 Agent 因故中断后，可以从最近的 checkpoint 恢复，并从 Trace 重放后续操作。

**第六步：Runtime 真正执行 Agent Loop**（最关键的修复）

这是整个重构的核心。修改前的代码路径：

```
1. TUI 在 app.rs 中创建 AgentRuntime 实例
2. TUI 在自己的线程中调用 runtime.run_loop_streaming()
3. Runtime::execute(RuntimeCommand::SubmitUserInput) 只存储任务，返回 "accepted"
4. AgentRuntime 通过 EventCallback 回调向 TUI 报告 LLM 事件
```

修改后的代码路径：

```
1. TUI 发送 RuntimeFacadeCommand::RunAgent(RunAgentCommand { session, agent_type })
2. Runtime 的 RunAgent handler 执行：
   - 创建 AgentRuntime
   - 调用 with_event_bus() 连接 EventBus
   - 调用 runtime.run_loop_streaming()
3. RuntimeFacadeCommand::ExecuteShell(cmd) 路由到 ToolRouter 执行
4. TUI 只接收事件，渲染视图
```

**代码证据**（来自 `crates/runtime/src/runtime.rs:242-279`）：

```rust
RuntimeFacadeCommand::RunAgent(cmd) => {
    let runtime = AgentRuntime::new(cmd.session, cmd.agent_type)
        .with_event_bus(services.event_bus.clone());
    // ...
    runtime.run_loop_streaming(&*agent, p, t, None).await
}
```

而之前（重构前）的代码路径是 TUI 直接创建 AgentRuntime 并在自己的线程中运行循环。

**第七步：Shell 执行路由到 ToolRouter**（与第六步并行）

```rust
// crates/runtime/src/commands.rs 新增 ExecuteShellCommand
pub struct ExecuteShellCommand {
    pub command: String,
    pub timeout_secs: Option<u64>,
    pub workdir: Option<String>,
}

// crates/runtime/src/runtime.rs 中的 handler
RuntimeFacadeCommand::ExecuteShell(cmd) => {
    let args = serde_json::json!({
        "command": cmd.command,
        "timeout": cmd.timeout_secs,
        "workdir": cmd.workdir,
    });
    services.tool_router.execute("bash", args, None).await
}
```

之前的实现是 TUI 直接调用 `std::process::Command`，绕过了 Runtime 的 ToolRouter 和 PermissionManager。重构后，所有 shell 执行都通过 Runtime 的标准路径。

### 重构前后对比

| 指标 | 重构前 | 重构后 |
|------|--------|--------|
| Agent Loop 执行位置 | TUI 线程 | Runtime |
| 事件传递 | 回调（EventCallback） | EventBus |
| Shell 执行 | 直接 process::Command | 通过 ToolRouter |
| Provider 差异 | 泄漏到 Core | 在 Gateway 层消化 |
| 状态管理 | 隐式 | 显式状态机（14 个状态） |
| 可测试性 | 低（紧耦合） | 高（依赖注入 + 假实现） |
| CLI 命令 | 不存在 | context inspect/explain/dump/why |
| Context 截断 | ranking 未使用 | trim_to_budget 正确处理边缘情况 |

---

## 主线二（续）：关键架构问题的深度分析

### ISSUE-001 深度分析：TUI 直接实例化 AgentRuntime

**问题代码位置**（重构前）：`crates/tui/src/app.rs:4755-4813`

```rust
// 重构前的代码（TUI 直接创建 AgentRuntime）
let runtime = AgentRuntime::new(session.clone(), agent_type);
// TUI spawns a thread to run the agent loop
let handle = std::thread::spawn({
    let runtime = runtime.clone();
    move || {
        runtime.run_loop_streaming(&agent, &provider, &tools, None)
    }
});
```

**问题本质**：违反单一职责原则。TUI 的职责是"展示和交互"，不应该承担"运行 Agent"的责任。当 TUI 直接控制 Agent 生命周期时，它需要理解 Agent 的所有内部细节（回调、线程模型、状态转换）。

**修复后代码**：

```rust
// TUI 现在只发送命令
let cmd = RuntimeFacadeCommand::RunAgent(Box::new(RunAgentCommand {
    session,
    agent_type: AgentType::Build,
}));
match runtime.execute(cmd).await { ... }
```

TUI 现在完全不知道 AgentRuntime 的存在——它只知道 RuntimeFacadeCommand。

### ISSUE-005 深度分析：Tool Schema 以明文嵌入

**问题代码位置**（重构前）：`crates/agent/src/build_agent.rs:22-36`

```rust
system_prompt: r#"You are OpenCode...
You have access to tools to help you complete coding tasks:
- file_read: Read file contents
- file_write: Write content to files
...
When you need to use a tool, respond with a JSON object containing tool_calls."#.to_string(),
```

**问题本质**：
1. 模型只能看到工具的文本描述，无法理解参数类型
2. 没有使用 Provider-native 的 tool calling API
3. 工具调用结果需要手动解析为 JSON，而不是 Provider 自动处理

**修复**：添加了 `ToolSchema` 和 `chat_with_tools()` 到 Provider trait：

```rust
// crates/llm/src/provider.rs 新增
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: Schema,
}

pub trait Provider: Send + Sync {
    async fn chat_with_tools(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolSchema],
    ) -> Result<ChatResponse, OpenCodeError>;
}
```

### ISSUE-013 深度分析：Runtime 不执行 Agent Loop

**问题本质**：Runtime 应该是"执行者"，而不是"存储者"。但之前的 Runtime 只负责：
- 创建 Session
- 保存 Session 状态
- 返回 "accepted"

真正的执行发生在 TUI 的线程中。

**修复后的架构**：

```
┌─────────────────────────────────────────────────────────────┐
│                         TUI                                 │
│  - 渲染视图                                                 │
│  - 接收用户输入                                             │
│  - 发送 RuntimeFacadeCommand                               │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Runtime Facade                           │
│  - 接收命令                                                 │
│  - 委托给 Services                                         │
│  - 返回结果                                                 │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Runtime Core                             │
│  - 创建 AgentRuntime                                       │
│  - 执行 run_loop_streaming()                               │
│  - 路由 Shell 命令到 ToolRouter                            │
│  - 通过 EventBus 发布事件                                   │
└─────────────────────────────────────────────────────────────┘
```

---

## 方法论提炼

### 1. "假 Runtime" 模式：一个常见的架构异味

**反直觉发现 #2**：这个项目不是从零开始设计 Runtime，而是经历了一个"功能先导、架构跟进"的迭代路径。大多数 Coding Agent 框架（包括一些知名开源项目）在早期都经历过类似的"UI 吞噬 Runtime"阶段。

**为什么这会发生**：
- 早期快速迭代时，让 UI 直接控制 Runtime 可以减少接口定义的工作量
- "运行时"和"展示层"的边界在 AI Agent 场景中比其他应用更模糊（状态机的每一步都需要 UI 反馈）
- 测试困难：没有足够测试覆盖的代码容易积累架构债务
- 回调满天飞：EventCallback 是一种快速传递事件的方式，但会导致隐式耦合

**行业对比**：LangChain 的早期版本也有类似问题——Chain 的执行有时发生在 Python 对象内部而非一个明确的执行器。这导致可观测性和可组合性都很差。Cowait 的设计者后来专门写了一篇文章反思这个问题。

**opencode-rs 的独特之处**：大多数项目是先有设计再有实现，opencode-rs 是反过来的——先有实现，通过架构审查发现问题，再建立 Reference Architecture 来指导重构。这种"从现有代码推导理想设计"的模式，对于 Rust 重写项目来说是合理的，因为代码已经在生产环境运行，问题已经暴露。

### 2. 设计文档作为架构改进的催化剂

这个项目的转折点是 `agent-runtime-design.md` 的引入。这是一个**Reference Architecture**文档——不是描述现有系统，而是定义一个理想目标架构。

关键洞察：**Reference Architecture 的价值不在于它描述了什么，而在于它揭示了什么**。

通过对比现有代码和理想设计，项目识别出了 14 个架构问题，其中 8 个是高严重性。这种系统性审查在快速迭代阶段是不可能的。

**Reference Architecture 的有效性来源**：

1. **具体性**：不是泛泛的原则，而是具体的接口定义和状态机
2. **可验证性**：定义了 14 个状态，测试可以验证状态转换
3. **分层性**：UI/Runtime/Provider/Storage 每层职责清晰

**如果重来**：项目可以在功能 parity 完成后立即进行一次"架构评估冲刺"，而不是等到问题积累到影响开发效率。但这种建议是事后诸葛亮——在当时的情况下，保持开发速度是首要任务。

### 3. 渐进式重构策略

项目没有采用"大爆炸"重构（big bang rewrite），而是采用了渐进式策略：

1. **测试基础设施优先**（`2898c972`）- 确保重构可验证
   - 引入 fake 实现替换真实组件
   - 建立 recording event sink 捕获事件序列
   - 允许单元测试验证行为不变性

2. **外围系统先行**（HookEngine, ToolRouter）- 降低风险
   - HookEngine 是可选扩展，不影响核心路径
   - ToolRouter 隔离 shell 执行，是独立模块

3. **核心问题最后**（Runtime 执行 Agent Loop）- 最大风险变更最后处理
   - 等所有其他变更稳定后，才修改 Runtime 的核心职责
   - 留下最多的时间窗口来验证稳定性

4. **文档同步更新**（ISSUE-001 到 ISSUE-014 逐一标记 FIXED）
   - 文档即代码——架构问题列表本身就是进度追踪

**这种策略的优势**：
- 每个步骤都可以单独验证和回滚
- 如果某个变更引入问题，可以快速定位
- 团队可以在重构期间继续交付小功能

**这种策略的代价**：
- 总重构时间拉长（从 4 月 28 日到 5 月 1 日仍在进行）
- 需要维护多个"中间状态"的代码
- 文档需要持续更新以反映最新状态

### 4. 事件驱动架构的务实实现

项目没有采用复杂的事件溯源（Event Sourcing）模式，而是选择了**轻量级事件总线**（EventBus + HookEngine）。这是一个务实的选择：

- **适合场景**：AI Agent 的核心状态（Session, Turn）需要确定性，不适合 event sourcing 的不可变性保证
- **足够能力**：HookEngine 提供了扩展点，TraceStore 提供了可观测性
- **降低复杂度**：不需要处理 event replay 的复杂性

**EventBus 的设计选择**：

```rust
// crates/core/src/bus.rs
pub trait EventBus: Send + Sync {
    fn publish(&self, event: DomainEvent);
    fn subscribe(&self, handler: EventHandler);
}
```

这是一个简单的发布-订阅模型，不是完整的 event sourcing。每个事件被记录到 TraceStore，但不要求每个状态变更都必须通过事件驱动。

### 5. 架构债务的识别时机

**反直觉发现 #3**：架构问题的症状早在 2025 年底就出现了，但直到 2026 年 4 月才被系统性识别。

症状：
- 2025 年中：`d702f1b0` 修复的 2+ 分钟启动问题（文件树遍历效率）
- 2025 年底：`f070b094` 修复的大量 clippy lint 问题（代码质量）
- 2026 年初：`20b24a85` 添加的 provider 超时（可靠性）

但这些都是"症状治疗"，不是"病因诊断"。真正的架构问题（UI 和 Runtime 边界混乱）直到 4 月底引入设计文档后才被完整识别。

**为什么会延迟**：
- 架构问题需要距离才能看清（太近了看不清）
- 功能交付压力下，架构审查被延后
- 没有明确的"架构问题"追踪（只有 bug 追踪）

---

## 结语：未来演进方向预测

基于当前的技术债务和未解决问题，预测下一步演进重点：

### 短期（1-3 个月）

1. **ISSUE-005 深入**：ToolSchema 的 Provider-native 实现目前已添加（`5000fa32`），但 ISSUE-005 指出"模型无法看到完整参数类型"的问题尚未完全解决。Provider-native tool calling 需要每个 Provider 分别实现正确的 schema 格式（OpenAI 的 function_call vs Anthropic 的 tool_use vs Google 的 function_declarations）。

2. **ContextRanking 激活**：ISSUE-008 提到 `trim_to_budget()` 的 ranking 边缘情况已修复（`321cb70e`），但整个 ranking 系统仍然是"定义但未充分使用"状态。Context 的 relevance 和 importance 计算可能需要更精细的算法。

3. **Server 集成完善**：当前 server 通过 runtime facade 桥接事件（`2841beab`, `1f03678f`），但"远程控制"模式尚未完全实现。Server 应该能够：
   - 接收远程用户的命令
   - 将命令路由到 Runtime
   - 将 Runtime 事件转发回远程客户端

### 中期（3-6 个月）

1. **Core crate 拆分**：根据 `docs/FINDINGS/findings.md` 的建议，`core` crate（64 个文件）应该被拆分为 session、project、skill 等独立 crate。这是一个高风险操作，需要：
   - 识别所有跨 crate 依赖
   - 设计新的 public API surface
   - 确保所有 consumers 兼容

2. **Provider 抽象层强化**：需要更好地处理 provider 之间的能力差异。目前的 ProviderGateway 是统一的，但：
   - OpenAI 支持 function calling
   - Anthropic 支持 tool_use
   - Ollama 只支持 JSON mode，没有原生 tool calling
   - MiniMax 的 API 可能完全不同

   未来的 ProviderGateway 需要表达"能力协商"的概念——Runtime 根据 Provider 的能力选择合适的执行路径。

3. **可恢复性增强**：TraceStore + CheckpointStore 的组合应该支持"中断后从检查点恢复"而不仅是"可观测性"。这需要：
   - 定义检查点频率策略
   - 实现状态序列化/反序列化
   - 设计恢复后的状态一致性保证

### 反直觉预测

**预测 #1**：项目的下一个大重构可能不是技术性的，而是组织性的。当前的"单人维护者"模式（从 commit 作者可以看出）在这种规模的架构重构中会成为瓶颈。可能会引入更正式的 RFC 流程或架构决策记录（ADR）。

**理由**：架构重构需要一致性决策，单人维护的优势是决策快速，劣势是视角单一。当项目从"功能开发"转向"架构优化"时，需要更多的设计讨论和权衡记录。RFC/ADR 流程可以：
- 记录架构决策的历史和理由
- 帮助新贡献者理解设计选择
- 为未来重构提供参考

**预测 #2**：项目的测试策略可能从"集成测试为主"转向"单元测试+Contract Testing"。

当前的 hook_engine_tests.rs 是单元测试的尝试，但整体测试覆盖率仍然偏低。Contract Testing 可以用来验证 ProviderGateway 的不同实现（OpenAI、Anthropic、Ollama）满足相同的契约。

**预测 #3**：项目可能引入"行为版本化"机制。

当前 Session 的状态是单一的，但随着 Agent 能力的增强，Session 的数据结构可能需要版本化。例如，当 Context 截断策略改变时，旧的 Session 可能需要特殊的迁移路径。

### 关键结论

opencode-rs 的演进展示了 AI Coding Agent 基础设施开发的一个典型路径——从快速功能实现到架构健康度提升。这个路径不是线性的，而是需要多次"架构评估-识别问题-渐进重构"的循环。

**项目的成熟度标志**：

1. **当前标志（已达成）**：能够识别并修复架构问题
2. **下一个标志（进行中）**：能够在一个独立的 PR 中完成一个完整的架构改进，而不需要跨越 80+ 个文件和数周时间
3. **最终标志（待达成）**：架构决策被正式记录，重构可以在小范围内快速验证和交付

**核心教训**：AI Coding Agent 的"Runtime"概念比其他应用更复杂——它需要处理 LLM 的非确定性、工具执行的安全性、用户交互的异步性、状态的持久化等多个维度。在早期快速迭代阶段将 UI 和 Runtime 合并是常见的起步策略，但长期健康度需要及时拆分。这个"拆分"的时机选择至关重要——太早影响功能交付速度，太晚则技术债务累积成山。

---

## 附录：关键 commit 索引

| Commit | 日期 | 描述 |
|--------|------|------|
| `7e1f1041` | 2025 | initial version |
| `f2dcd11c` | 2025 | Achieve full parity with OpenCode |
| `d702f1b0` | 2026-04-28 | fix(tui): limit sidebar file tree depth to fix 2+ minute startup |
| `d9037809` | 2026-04-29 | docs: add runtime unification implementation plan |
| `27930e2b` | 2026-04-29 | feat(runtime): add runtime facade and turn-aware session bridge |
| `85f0be8a` | 2026-04-30 | feat(core): add hook engine for lifecycle event processing |
| `8b971f8a` | 2026-04-30 | feat(runtime): normalize ProviderGateway trait |
| `dea2fd50` | 2026-04-30 | feat(runtime): add explicit RuntimeStatus state machine |
| `5000fa32` | 2026-04-30 | feat(llm): add ToolSchema and chat_with_tools for provider-native tool calling |
| `5869e956` | 2026-04-30 | feat(cli): add context inspectability commands |
| `2898c972` | 2026-05-01 | feat(runtime): add testing infrastructure |
| `c3c2223e` | 2026-05-01 | chore: update Cargo.lock and fix minor issues |
