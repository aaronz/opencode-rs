# OpenCode-RS 规格文档 v22

**版本**: 22  
**日期**: 2026年4月8日  
**作者**: OpenCode Rust Team  
**状态**: Draft

---

## 变更日志

| 版本 | 日期 | 变更描述 |
|------|------|----------|
| v22 | 2026-04-08 | 承接v19未完成任务：LSP语言服务后端集成、MCP工具注册、TUI架构完善 |
| v19 | 2026-04-08 | 基于PRD v2.4 (Rust Edition) 和差距分析更新；总体进度提升至81% |

---

## 概述

### 产品定位

OpenCode-RS 是 OpenCode 的 Rust 实现，是一个配合大语言模型（LLM）处理项目代码的 AI 编码助手。TUI（Terminal User Interface）为开发者提供了一种在终端环境中与 AI 进行协作的高效方式。

### 核心架构

```
┌──────────────────────────────────────────────────────────────┐
│                      opencode-cli                           │
├──────────────────────────────────────────────────────────────┤
│                      opencode-tui                           │
│  (Ratatui + Crossterm 终端UI、组件系统、对话框)               │
├──────────────────────────────────────────────────────────────┤
│                      opencode-agent                         │
│  (Build/Plan/Review/Refactor/Debug Agent)                  │
├──────────────────────────────────────────────────────────────┤
│                      opencode-tools                         │
│  (文件操作、Git、Bash、Search等工具)                         │
├──────────────────────────────────────────────────────────────┤
│                      opencode-mcp                           │
│  (Model Context Protocol工具发现与执行)                       │
├──────────────────────────────────────────────────────────────┤
│                      opencode-lsp                           │
│  (Language Server Protocol语言服务)                          │
├──────────────────────────────────────────────────────────────┤
│                      opencode-llm                           │
│  (多Provider支持：OpenAI/Anthropic/Ollama)                  │
├──────────────────────────────────────────────────────────────┤
│                      opencode-server                        │
│  (Actix-web HTTP服务、WebSocket流式传输)                     │
├──────────────────────────────────────────────────────────────┤
│                      opencode-storage                       │
│  (SQLite持久化、会话管理)                                     │
├──────────────────────────────────────────────────────────────┤
│                      opencode-permission                    │
│  (权限评估、批准队列、审计日志)                               │
└──────────────────────────────────────────────────────────────┘
```

---

## 1. LSP语言服务架构

### 1.1 问题陈述

当前LSP服务器声明了能力(capabilities)但所有处理器返回None，原因是缺少语言解析后端。

### 1.2 目标

实现真正的代码导航功能：
- `textDocument/definition` - 跳转到定义
- `textDocument/references` - 查找引用
- `textDocument/hover` - 悬停信息
- `textDocument/codeAction` - 代码动作

### 1.3 技术方案

#### 方案A: 集成tree-sitter

**优点**:
- 支持30+语言
- 增量解析适合编辑场景
- 活跃维护

**缺点**:
- 重量级依赖
- 需要为每种语言加载语法

#### 方案B: 使用syn/rust-analyzer风格

**优点**:
- 对Rust支持好
- 不需要运行时解析

**缺点**:
- 仅支持Rust
- 其他语言需另外方案

### 1.4 推荐方案

**方案A (tree-sitter)** - 支持多语言

---

## 2. MCP工具注册架构

### 2.1 问题陈述

`McpManager` 和 `bridge_to_tool_registry()` 存在但：
- `build_default_executor()` 从未被调用
- TUI的`App`结构没有`ToolRegistry`字段
- MCP工具从未被注册到任何工具注册表

### 2.2 目标

让MCP工具可被Agent执行

### 2.3 技术方案

1. 在TUI App中添加`ToolRegistry`字段
2. 在App初始化时调用`McpManager::bridge_to_tool_registry()`
3. 将ToolRegistry传递给AgentExecutor

```rust
// app.rs
pub struct App {
    // ... existing fields
    tool_registry: ToolRegistry,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            // ... existing initialization
            tool_registry: ToolRegistry::new(),
        };
        
        // Bridge MCP tools if configured
        #[cfg(feature = "mcp")]
        {
            let mcp_manager = McpManager::global();
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(mcp_manager.bridge_to_tool_registry(&mut app.tool_registry));
        }
        
        app
    }
}
```

---

## 3. TUI插件架构 (Sidecar)

### 3.1 问题陈述

无插件架构设计，Sidecar模式未实现

### 3.2 目标

支持插件扩展系统

### 3.3 技术方案

参考现有`PluginManager`设计：

```rust
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&mut self, config: &PluginConfig) -> Result<(), OpenCodeError>;
    fn execute(&self, action: &str, params: serde_json::Value) -> Result<serde_json::Value, OpenCodeError>;
}

pub enum PluginCapability {
    ListenEvents,
    RewritePrompt,
    InjectShellEnv,
    AddTools,      // <-- 关键：允许插件添加工具
    AddContextSources,
}
```

---

## 4. CLI流式输出 (NDJSON)

### 4.1 问题陈述

CLI无流式输出模式，NDJSON格式未实现

### 4.2 目标

支持CLI模式下的流式JSON输出

### 4.3 技术方案

```rust
// 添加 --format ndjson 选项
#[derive(Args)]
pub struct RunArgs {
    // ...
    #[arg(long, default_value = "default")]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
pub enum OutputFormat {
    Default,  // 格式化输出
    Json,     // 原始JSON
    Ndjson,   // 逐行JSON
}
```

---

## 验收清单

### LSP功能
- [ ] FR-183: goto_definition 返回真实位置
- [ ] FR-184: find_references 返回引用列表
- [ ] FR-185: hover 显示类型信息
- [ ] FR-186: codeAction 提供可用动作

### MCP功能
- [ ] FR-193: MCP工具可被发现
- [ ] FR-194: MCP工具可执行

### 插件功能
- [ ] FR-201: Sidecar插件支持

### CLI功能
- [ ] FR-142: NDJSON输出格式

---

**版本**: 22  
**最后更新**: 2026-04-08  
**维护者**: OpenCode Rust Team
