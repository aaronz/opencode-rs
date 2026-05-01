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

## 主线二（续）：踩坑编年史：特性迭代中的错误与反思

### 踩坑一：FileTree 深度导致 2+ 分钟启动（commit `d702f1b0`）

**问题**：用户反馈 TUI 启动后_sidebar 文件树加载需要 2+ 分钟。

**原因**：FileTree 组件递归遍历目录时没有深度限制，也没有异步加载机制。当项目目录层级深、文件多时，主线程被阻塞。

**错误类型**：happy path 假设——假设用户项目的目录层级不会太深，文件不会太多。

**教训**：UI 组件中的目录遍历必须考虑：
- 深度限制（max depth）
- 异步加载 + loading placeholder
- 懒加载（只加载可见部分）
- 缓存机制

**如果重来**：在实现 FileTree 组件时，应该从一开始就设置合理的 max_depth 和懒加载策略，而不是等到用户投诉才修复。

### 踩坑二：RollingFileAppender 把目录当文件创建（commit `4bf14d00`）

**问题**：日志系统报错，原因是 RollingFileAppender 的目录创建逻辑把应该创建目录的地方创建成了文件。

**原因**：`log_dir` 参数被错误处理——代码中 `log_dir` 应该是一个目录路径，但实际传递时被当作文件路径处理。

**错误类型**：路径处理的边界条件错误。这是一个常见的路径操作 bug：`path.join()` vs `path.push()` 的误用，以及缺少 `is_dir()` 前置检查。

**教训**：
- 路径操作必须明确语义：`path.join(a, b)` 是拼接，`path.push(a)` 是添加
- 创建目录前必须检查目标是否已存在以及其类型
- 路径处理的单元测试必须覆盖边界条件

**如果重来**：应该使用标准库的 `std::fs::create_dir_all()` 并配合适当的错误处理，而不是自己实现目录创建逻辑。

### 踩坑三：trim_to_budget 的 preserve_from 边缘情况（commit `321cb70e`）

**问题**：`trim_to_budget()` 函数的 `preserve_from` 参数在特定情况下计算错误，导致重要的上下文消息被错误截断。

**原因**：`preserve_from` 的 ranking 算法在消息数量为边界值时计算异常。

**错误类型**：算法实现只考虑了常规情况，没有考虑边缘情况（zero, one, exactly two 等）。

**教训**：
- 排序/选择算法必须用边界值测试（0, 1, 2, n-1, n, n+1）
- Preserve-from 的语义要明确：是"保留最近 N 条"还是"保留优先级最高的 N 条"？
- 测试用例需要覆盖随机场景

**如果重来**：在实现 `trim_to_budget()` 时，应该先用 property-based testing 生成各种边界输入，验证输出正确性。

### 踩坑四：Ollama JSONL streaming 解析失败（commit `9d684cf9`）

**问题**：Ollama Provider 的 streaming 模式解析 JSONL 数据时失败。

**原因**：Ollama 的 streaming 输出是 JSON Lines 格式（每行一个 JSON 对象），但解析器没有正确处理行的边界。增量解码器在遇到不完整的 JSON 时没有缓存剩余数据。

**错误类型**：流式解析的状态机实现不完整。JSONL 解析需要：
1. 按行分割
2. 每行单独解析为 JSON
3. 处理"行不完整"的边界情况

**教训**：
- 流式解析必须处理行边界
- 增量解码器必须在数据不完整时缓存 Partial 结果
- 不同 Provider 的 streaming 格式可能不同，需要分别处理

**如果重来**：应该为每个 Provider 实现一个流式解析的单元测试套件，覆盖正常数据、截断数据、畸形数据等场景。

### 踩坑五：TUI 越权创建 AgentRuntime（ISSUE-001, ISSUE-013）

**问题**：这是最大的踩坑——是架构级别的设计错误，而非实现 bug。

**原因**：在快速开发阶段，"让 TUI 直接控制 Agent"是最快速的实现方式：
- 不需要定义 Runtime Facade 接口
- 不需要处理命令路由
- 直接回调，事件传递简单

这种模式在初期效率很高，但代价是架构边界混乱。当功能增多、代码变复杂后：
- TUI 需要知道 AgentRuntime 的所有内部细节
- 每个新功能都需要修改 TUI 和 AgentRuntime 两端
- 测试几乎不可能（紧耦合）

**错误类型**：架构决策错误。"快速实现"和"长期健康"之间的权衡失衡。

**教训**：
- UI 和 Runtime 的边界应该在项目初期就明确定义，而不是等出了问题再重构
- 即使是"快速原型"，也要遵守基本的分层原则
- "等技术债积累够了再还"是谎言——技术债只会越滚越大

**如果重来**：在 Phase 1 开始时，就应该在 design doc 中定义 TUI 和 Runtime 的边界，并在代码中使用接口隔离。即使 AgentRuntime 的实现还很简陋，Facade 接口应该先存在。

### 踩坑六：Provider 接口不一致泄漏 Provider 细节到 Core（ISSUE-005）

**问题**：Tool schema 以明文形式嵌入 system prompt，而不是通过 Provider 的 `tools` 参数传递。

**原因**：
1. 早期 Provider 实现时，只实现了 `complete()` 和 `chat()`，没有 `tools` 参数
2. 为了快速支持 tool calling，采用了"明文嵌入 prompt"的 workaround
3. 这个 workaround 成了默认实现，Provider-native 的 tool calling 被搁置

**错误类型**：Workaround 成为默认实现。"够用就好"的心态导致技术债积累。

**教训**：
- Workaround 必须标记为临时方案，并设置移除日期
- Provider 接口的设计应该在第一个 Provider 实现之前完成，而不是之后补加
- Provider 能力（tool calling, streaming, vision 等）应该在接口层面表达，而不是用 boolean flag

**如果重来**：应该在 `Provider` trait 设计阶段就包含 `chat_with_tools()` 方法，并实现第一个 Provider（OpenAI）时就使用它。明文嵌入 prompt 应该被明确禁止。

## 主线三：方法论提炼——从踩坑中生长的工程原则

### 原则一："快速实现"是谎言，技术债只会越滚越大

**来源**：ISSUE-001（TUI 越权）和 ISSUE-005（明文嵌入 schema）

这是 opencode-rs 最重要的教训。在 Phase 1 开发阶段，"让 TUI 直接控制 Runtime"是最快的实现方式——不需要定义接口，不需要路由命令，不需要处理异步。好处是立竿见影的：功能在几天内就能跑起来。

但代价是隐蔽的、延迟的、累积的：

| 时间点 | 技术债表现 |
|--------|------------|
| Phase 1 结束时 | 代码能跑，架构"凑合能用" |
| Phase 2 开始时 | 修一个 bug 引出另一个（TUI 和 Runtime 紧耦合） |
| Phase 2 中期 | 2+ 分钟启动、screen clear 失败（边界条件没处理） |
| Phase 3 重构时 | 80+ 文件、7500+ 行变更，持续数周 |

**反直觉结论**：架构问题越早发现越好，而不是"等有精力了再重构"。因为技术债有复利效应——债务越久，利息越高。

**可操作建议**：
1. 在项目初期就用 30 分钟写下 UI/Runtime/Provider 的边界，作为"架构宪法"
2. 任何违背这条边界的代码都必须有 `// HACK: 临时方案 - YYYY-MM-DD 移除` 注释
3. 每加入一个 workaround，就在 backlog 里加一个对应的技术债 ticket

### 原则二：症状和病因要区分，治疗症状会掩盖病因

**来源**：`d702f1b0`（2+ 分钟启动）、`0392ff74`（path conflicts）、`4bf14d00`（RollingFileAppender bug）

这三个 fix 有一个共同模式：它们修复的都是症状，不是病因。

| Commit | 修复的症状 | 真正的病因 |
|--------|------------|------------|
| `d702f1b0` | 文件树加载慢 | 目录遍历没有深度限制、没有异步 |
| `0392ff74` | path conflicts | 路径处理混乱（join vs push） |
| `4bf14d00` | log dir 创建失败 | 目录/文件类型检查缺失 |

**为什么会这样**：在快速开发阶段，"头痛医头"是最省力的策略。修复症状快，修复病因需要深入理解代码，往往涉及多个模块。

**可操作建议**：
1. 当同一个"症状"出现 3 次以上时，要停下来问："这是不是一个更深层问题的症状？"
2. 建立"症状日志"，记录每次 fix 的 commit 和对应的深层原因
3. 每月进行一次"症状回顾"，看是否有重复出现的模式

### 原则三：Workaround 必须有 TTL（Time To Live）

**来源**：ISSUE-005（明文嵌入 schema）、Provider 接口不一致

在 Issue-005 的复盘中我们看到：
- 第一个 Provider 实现时跳过了 `tools` 参数
- 用"明文嵌入 prompt"作为 workaround
- 这个 workaround 一用就是几个月，成了"事实标准"

**Workaround 的生命周期**：
```
Day 1: 实施 workaround → 解决了紧急问题
Day 7: workaround 成为默认实现
Day 30: 没人记得这是 workaround
Day 90: 移除 workaround 需要重写大量代码
```

**可操作建议**：
1. 任何 workaround 必须附带 `// TTL: 2026-Q2 - 必须在 X 月 Y 日前移除`
2. 建立技术债的"优先级排序"，不是所有债都要还，但要清楚哪些债拒绝积累
3. Provider 接口必须先设计再实现，不要"先用后补"

### 原则四：渐进式重构优于大爆炸重构

**来源**：Phase 3 的实施策略

项目在 Phase 3 重构时采用了：
1. 测试基础设施优先（`2898c972`）
2. 外围系统先行（HookEngine, ToolRouter）
3. 核心问题最后（Runtime 执行 Agent Loop）
4. 文档同步更新

**为什么这不是过度谨慎**：

大爆炸重构的风险在于：
- 没有回归测试网络，无法验证行为不变性
- 改动范围太大，引入新 bug 无法定位
- 团队在重构期间无法交付价值，压力巨大

渐进式重构的优势在于：
- 每个步骤可单独验证和回滚
- 功能可以持续交付（而非 freeze 整个开发）
- 风险分散到多个小步骤，总风险更低

**可操作建议**：
1. 大型重构的第一步永远是"建立测试网络"
2. 重构前问自己："如果这个重构需要回滚，最小回滚单位是什么？"
3. 重构期间保持功能交付——不要为了重构而重构，要让团队看到进展

### 原则五：Reference Architecture 的价值在于"揭示"而非"描述"

**来源**：`agent-runtime-design.md` 的引入

这个文档之所以有效，是因为它：
- 不是描述现有系统（那会是架构文档）
- 而是定义理想目标（reference architecture）
- 通过对比揭示差距（14 个 ISSUE）

**为什么这种文档有效**：当团队已经"身在庐山中"时，需要一个外部的、理想化的标准来衡量现状。没有这个标准，所有的"架构问题"都只是个人 opinion，无法形成建设性讨论。

**可操作建议**：
1. 每 6 个月进行一次"架构自我审查"
2. 用 Reference Architecture 的框架来评估现有系统
3. 识别的 ISSUE 要量化严重性，而不是只描述问题

---

## 主线四：反直觉发现——与常见假设相悖的演进决策

### 反直觉发现一：功能 parity 阶段刻意保留 TypeScript 架构是合理的

**常见假设**：迁移项目应该趁这个机会重构架构

**实际决策**：保留 TypeScript 架构，先完成功能 parity

**理由**：
1. 架构重构需要时间，期间无法交付价值
2. 功能 parity 是验证 Rust 版本正确性的基础
3. 如果架构重构和功能迁移同时进行，出问题时无法判断原因

**这是不是说我们应该永远不重构**：不是。技术债会累积，但"什么时候重构"是一个时机问题，不是"要不要重构"的问题。

### 反直觉发现二："假 Runtime" 是常见架构异味，不是 opencode-rs 独有的问题

**常见假设**：架构问题说明团队能力不行

**实际发现**：LangChain、Cowait 等知名项目早期都经历过类似问题

**原因**：AI Agent 应用的 UI 和 Runtime 边界天然模糊——状态机的每一步都需要 UI 反馈，这让"让 UI 直接控制 Runtime"成了一个自然的起点。

**启发**：架构问题的价值在于识别和修复，不在于避免。没有人能在项目初期就设计出完美架构。

### 反直觉发现三：架构问题症状在 2025 年底就出现，但 2026 年 4 月才被系统性识别

**常见假设**：架构问题是新问题

**实际发现**：2+ 分钟启动问题（2025 年底）、provider 超时（2026 年初）都是架构问题的症状

**为什么会延迟识别**：
1. 架构问题需要距离才能看清（太近了看不清）
2. 功能交付压力下，架构审查被延后
3. 没有明确的"架构问题"追踪（只有 bug 追踪）

**启发**：建议团队在每个季度结束时进行一次"架构健康度检查"，即使没有明显的 bug。

---

## 主线五：如果重来：复盘视角的决策推演

### 决策点一：Phase 1 开始时应该先定义架构边界

**如果重来**：

在 Phase 1 的第一天，花 30 分钟写下这个架构宪法：

```text
# opencode-rs 架构宪法

1. TUI 只负责：渲染视图、接收用户输入、发送命令、订阅事件
2. Runtime Facade 只负责：接收命令、委托给 Services、返回结果
3. Runtime Core 只负责：创建 AgentRuntime、执行 Agent Loop、路由 Tool 命令、发布事件
4. Provider Gateway 只负责：统一 Provider 接口，隐藏 provider-specific 细节
5. 任何违背以上边界的代码必须标记 `// HACK: TTL-Q3-2025`
```

**预期效果**：避免 ISSUE-001 和 ISSUE-013，至少减少 50% 的 Phase 3 重构工作量。

### 决策点二：Provider trait 应该从一开始就包含 tools 参数

**如果重来**：

在实现第一个 Provider（OpenAI）之前，先定义 Provider trait：

```rust
pub trait Provider: Send + Sync {
    async fn complete(&self, prompt: &str, context: Option<&str>) -> Result<String, OpenCodeError>;
    async fn chat(&self, messages: &[ChatMessage]) -> Result<ChatResponse, OpenCodeError>;
    async fn chat_with_tools(&self, messages: &[ChatMessage], tools: &[ToolSchema]) -> Result<ChatResponse, OpenCodeError>;
}
```

**预期效果**：
- 避免 ISSUE-005（明文嵌入 schema）
- Provider-native tool calling 从第一天就能用
- 不需要后续的 `8b971f8a`（normalize ProviderGateway）和 `5000fa32`（add ToolSchema）

### 决策点三：FileTree 组件应该从一开始就设置深度限制

**如果重来**：

在 `d702f1b0` 之前实现 FileTree 时：

```rust
pub struct FileTreeConfig {
    max_depth: usize,        // 设置合理的默认值，如 3
    lazy_load: bool,         // 默认开启
    cache_enabled: bool,    // 默认开启
}

impl Default for FileTreeConfig {
    fn default() -> Self {
        Self {
            max_depth: 3,
            lazy_load: true,
            cache_enabled: true,
        }
    }
}
```

**预期效果**：2+ 分钟启动问题不会出现在用户反馈中，因为这是内部限制而非外部 bug。

### 决策点四：每个 Provider 实现应该有自己的 streaming 测试套件

**如果重来**：

在实现 Ollama provider 的 streaming 支持时：

```rust
#[cfg(test)]
mod streaming_tests {
    use super::*;

    #[tokio::test]
    async fn test_ollama_streaming_complete_json() {
        // 正常数据测试
    }

    #[tokio::test]
    async fn test_ollama_streaming_truncated_json() {
        // 截断数据测试 - 验证增量解析
    }

    #[tokio::test]
    async fn test_ollama_streaming_malformed_json() {
        // 畸形数据测试 - 验证错误处理
    }
}
```

**预期效果**：`9d684cf9`（fix Ollama JSONL streaming）不会成为一个生产环境 bug。

---

## 结语：未来演进方向预测

基于当前的技术债务和未解决问题，预测下一步演进重点：

### 短期（1-3 个月）

1. **ISSUE-005 深入完成**：ToolSchema 的 Provider-native 实现目前只完成了一半（接口已添加，但每个 Provider 的实现还需要适配 OpenAI/Anthropic/Ollama 的不同格式）
2. **ContextRanking 激活**：trim_to_budget 的 ranking 算法需要更精细的权重调整
3. **Server 远程控制模式**：当前 server 已桥接事件，但完整的远程控制需要更多工作

### 中期（3-6 个月）

1. **Core crate 拆分**：64 个文件的 `core` crate 是下一个需要解决的大型技术债
2. **Provider 抽象层强化**：需要处理 OpenAI/Anthropic/Ollama 的能力差异
3. **可恢复性增强**：TraceStore + CheckpointStore 应该支持"中断后恢复"

### 反直觉预测

**预测 #1**：项目会引入 RFC/ADR 流程来记录架构决策。当前的"单人维护者"模式在架构重构后会暴露决策视角单一的问题。

**预测 #2**：项目会采用 Contract Testing 来验证 Provider Gateway 的不同实现。这是从"集成测试为主"转向更精细测试策略的第一步。

**关键结论**：opencode-rs 的演进是 AI Coding Agent 基础设施开发的一个缩影——从快速功能实现到架构健康度提升，从"先跑起来"到"跑得长久"。这个路径不是线性的，而是需要多次"架构评估-识别问题-渐进重构"的循环。项目的下一个成熟度标志是：能够在一个独立的 PR 中完成一个完整的架构改进，而不需要跨越数周时间和 80+ 个文件。

**最终建议**：对于正在进行 Rust 重写的项目，建议在完成功能 parity 后立即进行为期一周的"架构评估冲刺"，识别和量化技术债务，制定重构路线图。这比"等技术债积累够了再重构"要高效得多——因为技术债的利息是复利的。

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
