# OpenCode-RS 规格文档 v19

**版本**: 19  
**日期**: 2026年4月8日  
**作者**: OpenCode Rust Team  
**状态**: 已发布

---

## 变更日志

| 版本 | 日期 | 变更描述 |
|------|------|----------|
| v19 | 2026-04-08 | 基于PRD v2.4 (Rust Edition) 和差距分析更新；总体进度提升至85%；解决OpenTUI架构误解；聚焦P1/P2功能缺口 |
| v18 | 2026-04-07 | 基于PRD v2.4 (Rust Edition) 和差距分析更新；修正多个功能的实现状态；新增缺失功能规格 |
| v17 | 2026-04-07 | 基于PRD v2.4 (Rust Edition) 创建，包含完整的TUI功能规格和FR编号 |
| v16 | 2026-04-06 | 添加MCP协议、插件系统规格 |
| v15 | 2026-03-27 | 完成功能对标补充 |

---

## 1. 概述

### 1.1 产品定位

OpenCode-RS 是 OpenCode 的 Rust 实现，是一个配合大语言模型（LLM）处理项目代码的 AI 编码助手。TUI（Terminal User Interface）为开发者提供了一种在终端环境中与 AI 进行协作的高效方式。

### 1.2 核心架构

```
┌──────────────────────────────────────────────────────────────┐
│                      opencode-cli                           │
│  (命令行入口、命令分发、REPL交互)                              │
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
│                      opencode-llm                           │
│  (多Provider支持：OpenAI/Anthropic/Ollama)                  │
├──────────────────────────────────────────────────────────────┤
│                      opencode-server                        │
│  (Actix-web HTTP服务、WebSocket流式传输)                      │
├──────────────────────────────────────────────────────────────┤
│                      opencode-storage                       │
│  (SQLite持久化、会话管理)                                     │
├──────────────────────────────────────────────────────────────┤
│                      opencode-permission                    │
│  (权限评估、批准队列、审计日志)                               │
└──────────────────────────────────────────────────────────────┘
```

### 1.3 技术栈

| 组件 | 技术选型 | 版本 |
|------|----------|------|
| TUI渲染 | Ratatui | 0.30 |
| 终端控制 | Crossterm | 0.28 |
| 异步运行时 | Tokio | 1.45 |
| HTTP服务 | Actix-web | 4 |
| 语法高亮 | Syntect | 5 |
| Markdown解析 | pulldown-cmark | 0.13 |
| 配置文件 | JSONC/JSON/TOML | - |

### 1.4 架构决策记录 (ADR)

#### ADR-001: TUI 框架选择 Ratatui

**状态**: 已批准  
**日期**: 2026-04-07

**背景**: PRD v2.4 存在两个版本：
- 原始版本要求使用 OpenTUI (Zig + TypeScript)
- Rust Edition 要求使用 Ratatui (Rust)

**决策**: 采用 Ratatui 作为 TUI 渲染框架

**理由**:
1. Rust Edition PRD 明确要求 Ratatui
2. 完全使用 Rust 实现，无外部 C 依赖或 Bun 运行时要求
3. 完全使用 Rust 实现，享受 Rust 的安全性、高性能和零成本抽象
4. Ratatui 是 Rust 生态最成熟的 TUI 库（19401+ 星标，3000万+ 下载量）
5. 与现有架构（Tokio 异步、Crossterm 终端控制）无缝集成

**影响**: OpenTUI 的某些特定功能（如 3D 渲染）不适用于 Rust 版本

---

## 2. TUI 架构规格

### 2.1 核心依赖配置

```toml
[dependencies]
ratatui = { version = "0.30", default-features = false, features = ["crossterm"] }
crossterm = { version = "0.28", features = ["event-stream"] }
tokio = { version = "1.45", features = ["full"] }
syntect = "5"
pulldown_cmark = "0.13"
```

### 2.2 Crate 结构

```
crates/tui/
├── src/
│   ├── lib.rs                 # 库入口，导出公共API
│   ├── app.rs                 # 应用状态机、TuiState
│   ├── cli/args.rs            # CLI参数定义
│   ├── command.rs             # 命令注册表
│   ├── components/            # UI组件
│   │   ├── banner.rs          # 启动横幅
│   │   ├── diff_view.rs       # Diff视图
│   │   ├── file_tree.rs       # 文件树
│   │   ├── input_widget.rs    # 输入组件
│   │   ├── right_panel.rs     # 右侧面板
│   │   ├── skills_panel.rs    # 技能面板
│   │   ├── status_bar.rs      # 状态栏
│   │   ├── terminal_panel.rs  # 终端面板
│   │   ├── title_bar.rs       # 标题栏
│   │   └── virtual_list.rs    # 虚拟列表
│   ├── config.rs              # TUI配置
│   ├── dialogs/               # 对话框
│   │   ├── connect_method.rs  # 连接方式选择
│   │   ├── connect_model.rs   # 模型选择
│   │   ├── connect_provider.rs# Provider选择
│   │   ├── diff_review.rs     # Diff审查
│   │   ├── directory_selection.rs
│   │   ├── file_selection.rs  # 文件选择
│   │   ├── model_selection.rs # 模型选择
│   │   ├── provider_management.rs
│   │   ├── release_notes.rs   # 发布说明
│   │   ├── settings.rs        # 设置
│   │   ├── slash_command.rs   # 斜杠命令
│   │   └── mod.rs
│   ├── input/                 # 输入处理
│   │   ├── completer.rs       # 自动补全
│   │   ├── editor.rs          # 外部编辑器
│   │   ├── history.rs         # 历史记录
│   │   ├── input_box.rs       # 输入框
│   │   ├── parser.rs          # 输入解析
│   │   └── processor.rs       # 输入处理
│   ├── layout.rs              # 布局管理
│   ├── patch_preview.rs       # 补丁预览
│   ├── render/                # 渲染
│   │   ├── markdown.rs        # Markdown渲染
│   │   └── syntax_highlight.rs# 语法高亮
│   ├── right_panel.rs         # 右侧面板管理
│   ├── session.rs             # 会话管理
│   ├── server_protocol.rs     # 服务协议
│   ├── server_ws.rs           # WebSocket客户端
│   ├── shell_handler.rs       # Shell命令执行
│   ├── file_ref_handler.rs    # 文件引用处理
│   ├── theme.rs               # 主题管理
│   └── widgets/               # Widget组件
│       ├── code_block.rs      # 代码块
│       ├── command_palette.rs # 命令面板
│       ├── file_selection.rs  # 文件选择
│       ├── indicators.rs      # 指示器
│       ├── message_bubble.rs  # 消息气泡
│       ├── scrollbar.rs       # 滚动条
│       ├── spinner.rs         # 加载动画
│       └── mod.rs
└── Cargo.toml
```

### 2.3 应用状态机

```rust
pub enum TuiState {
    Idle,              // 空闲状态
    Streaming,         // 流式输出中
    ExecutingTool,     // 工具执行中
    Submitting,        // 提交中
    Aborting,          // 中止中
    Reconnecting,      // 重连中
    WaitingForPermission, // 等待权限批准
}
```

---

## 3. 功能需求规格 (FR-XXX)

### 3.1 启动与基础交互

| FR-ID | 功能点 | 描述 | 优先级 | 状态 | 差距分析 |
|-------|--------|------|--------|------|----------|
| FR-001 | TUI启动 | 运行 `opencode-rs` 启动TUI | P0 | ✅ 已实现 | - |
| FR-002 | 指定目录启动 | 运行 `opencode-rs /path/to/project` 启动指定目录 | P0 | ✅ 已实现 | - |
| FR-003 | CLI参数解析 | 支持 `--model`, `--permission-mode`, `--output-format` 等 | P0 | ✅ 已实现 | - |
| FR-004 | 消息输入 | 在TUI中输入消息进行提示 | P0 | ✅ 已实现 | - |
| FR-005 | AI响应渲染 | 使用Ratatui组件渲染AI响应 | P0 | ✅ 已实现 | - |
| FR-006 | 打字机效果 | 流式输出的打字机渲染效果 | P1 | 🔲 进行中 | GAP-P1-001: `input_widget.start_typewriter()` 被调用但增量渲染未实现 |
| FR-007 | 实时Token计数 | 在状态栏显示实时Token使用量 | P1 | 🔲 进行中 | GAP-P1-002: `status_bar.update_usage()` 调用存在但实际渲染需验证 |

### 3.2 文件引用功能

| FR-ID | 功能点 | 描述 | 优先级 | 状态 | 差距分析 |
|-------|--------|------|--------|------|----------|
| FR-010 | 文件模糊搜索 | 使用 `@` 在消息中引用文件 | P0 | ✅ 已实现 | - |
| FR-011 | 文件内容加载 | 引用的文件内容自动添加到上下文 | P0 | ✅ 已实现 | FileRefHandler.resolve() 已与 LLM 上下文集成 |
| FR-012 | 文件选择列表 | 搜索结果通过List组件展示 | P0 | ✅ 已实现 | - |
| FR-013 | @路径补全 | 输入时自动补全文件路径 | P1 | 🔲 待实现 | - |
| FR-014 | 文件引用上下文集成 | 将FileRefHandler结果集成到LLM请求上下文 | P0 | ✅ 已实现 | GAP-P0-002 已修复 |

### 3.3 Bash命令执行

| FR-ID | 功能点 | 描述 | 优先级 | 状态 | 差距分析 |
|-------|--------|------|--------|------|----------|
| FR-020 | Shell命令执行 | 以 `!` 开头的消息作为shell命令执行 | P0 | ✅ 已实现 | - |
| FR-021 | 输出结果集成 | 命令输出作为工具结果添加 | P0 | ✅ 已实现 | ShellHandler 已与消息处理流程集成 |
| FR-022 | 命令终止 | 支持Ctrl+C终止正在执行的命令 | P1 | ✅ 已实现 | InterruptibleHandle + UI绑定完整 |
| FR-023 | Shell输出UI渲染 | 将ShellHandler结果渲染到UI | P0 | ✅ 已实现 | GAP-P0-003 已修复 |

### 3.4 斜杠命令系统

| FR-ID | 命令 | 别名 | 描述 | 快捷键 | 状态 | 差距分析 |
|-------|------|------|------|--------|------|----------|
| FR-030 | `/plan` | `/p` | 切换到plan模式 | - | ✅ 已实现 | - |
| FR-031 | `/build` | `/b` | 切换到build模式 | - | ✅ 已实现 | - |
| FR-032 | `/clear` | - | 清空消息 | - | ✅ 已实现 | - |
| FR-033 | `/help` | `/h`, `/?` | 显示帮助 | `ctrl+x h` | ✅ 已实现 | - |
| FR-034 | `/timeline` | `/t` | 打开时间线视图 | - | ✅ 已实现 | - |
| FR-035 | `/fork` | `/f` | 在当前消息处创建分支 | - | ✅ 已实现 | - |
| FR-036 | `/meta` | `/m` | 切换元数据显示 | - | ✅ 已实现 | - |
| FR-037 | `/settings` | `/,` | 打开设置对话框 | - | ✅ 已实现 | - |
| FR-038 | `/models` | - | 打开模型选择 | `ctrl+x m` | ✅ 已实现 | - |
| FR-039 | `/providers` | - | 打开Provider管理 | - | ✅ 已实现 | - |
| FR-040 | `/connect` | - | 连接Provider | - | ✅ 已实现 | - |
| FR-041 | `/files` | - | 切换文件树面板 | - | ✅ 已实现 | - |
| FR-042 | `/skills` | - | 切换技能面板 | - | ✅ 已实现 | - |
| FR-043 | `/release-notes` | `/rn` | 打开发布说明 | - | ✅ 已实现 | - |
| FR-044 | `/compact` | `/c` | 压缩会话 | `ctrl+x c` | ⚠️ 部分实现 | GAP-P2-003: 仅显示消息，未实际调用 SummaryAgent |
| FR-045 | `/summarize` | `/s` | 总结当前对话 | - | ✅ 已实现 | - |
| FR-046 | `/export` | `/e` | 导出会话到Markdown | `ctrl+x x` | ✅ 已实现 | - |
| FR-047 | `/undo` | `/u` | 撤销最后文件变更 | `ctrl+x u` | ✅ 已实现 | - |
| FR-048 | `/sessions` | `/ses` | 列出会话并管理 | `ctrl+x l` | ✅ 已实现 | - |
| FR-049 | `/new` | - | 创建新会话 | `ctrl+x n` | ✅ 已实现 | - |
| FR-050 | `/details` | `/d` | 切换工具执行详情 | `ctrl+x d` | ✅ 已实现 | - |
| FR-051 | `/themes` | - | 列出可用主题 | `ctrl+x t` | ✅ 已实现 | - |
| FR-052 | `/theme` | - | 切换到下一主题 | - | ✅ 已实现 | - |
| FR-053 | `/exit` | `/q` | 退出应用 | `ctrl+x q` | ✅ 已实现 | - |
| FR-054 | `/search` | - | 搜索对话历史 | - | 🔲 待实现 | GAP-P2-002: 只有Custom占位符 |
| FR-055 | `/diff` | - | 显示Git Diff | - | ⚠️ 部分实现 | GAP-P2-001: diff视图存在但未集成到命令 |
| FR-056 | `/memory` | - | 管理记忆条目 | - | 🔲 待实现 | GAP-P2-003: 只有Custom占位符 |
| FR-057 | `/plugins` | - | 管理插件 | - | 🔲 待实现 | GAP-P2-004: 只有Custom占位符 |
| FR-058 | `/username` | - | 设置显示用户名 | - | 🔲 待实现 | GAP-P2-005: 只有Custom占位符 |
| FR-059 | `/thinking` | - | 切换思考模式 | - | 🔲 进行中 | GAP-P1-004: thinking_mode标志存在但未传递给LLM provider |
| FR-060 | `/status` | - | 显示会话状态 | - | ⚠️ 部分实现 | GAP-P2-006: 框架存在，完整状态显示待实现 |
| FR-061 | `/share` | - | 分享当前会话 | `ctrl+x s` | ⚠️ 部分实现 | GAP-P1-003: 只导出到临时文件，未实现远程分享 |
| FR-062 | `/unshare` | - | 取消分享当前会话 | - | 🔲 待实现 | GAP-P2-001: 只有占位消息 |
| FR-063 | `/redo` | - | 重做 | `ctrl+x r` | ✅ 已实现 | - |
| FR-064 | `/editor` | - | 打开外部编辑器编写消息 | `ctrl+x e` | ✅ 已实现 | - |
| FR-065 | `/init` | - | 创建或更新AGENTS.md | `ctrl+x i` | ✅ 已实现 | - |

### 3.5 UI组件规格

#### 3.5.1 布局组件

| FR-ID | 组件 | 描述 | Ratatui基础 | 状态 |
|-------|------|------|-------------|------|
| FR-070 | Banner | 启动横幅 | Paragraph | ✅ 已实现 |
| FR-071 | TitleBar | 标题栏 | Paragraph + Block | ✅ 已实现 |
| FR-072 | StatusBar | 状态栏 | Paragraph | ✅ 已实现 |
| FR-073 | InputWidget | 输入组件 | Paragraph | ✅ 已实现 |
| FR-074 | VirtualList | 虚拟列表 | 自定义 | ✅ 已实现 |
| FR-075 | FileTree | 文件树 | List + TreeState | ✅ 已实现 |
| FR-076 | TerminalPanel | 终端面板 | Paragraph | ✅ 已实现 |
| FR-077 | RightPanel | 右侧面板 | 复合组件 | ✅ 已实现 |
| FR-078 | DiffView | Diff视图 | 自定义渲染 | ✅ 已实现 |
| FR-079 | SkillsPanel | 技能面板 | List | ✅ 已实现 |

#### 3.5.2 消息组件

| FR-ID | 组件 | 描述 | Ratatui基础 | 状态 |
|-------|------|------|-------------|------|
| FR-080 | MessageBubble | 消息气泡 | Block + Paragraph | ✅ 已实现 |
| FR-081 | CodeBlock | 代码块 | Paragraph + Block | ✅ 已实现 |
| FR-082 | ThinkingIndicator | 思考指示器 | 自定义动画 | ✅ 已实现 |
| FR-083 | Spinner | 加载动画 | 自定义渲染 | ✅ 已实现 |
| FR-084 | ProgressBar | 进度条 | Gauge | 🔲 待实现 |
| FR-085 | Scrollbar | 滚动条 | 自定义渲染 | ✅ 已实现 |
| FR-086 | Indicators | 状态指示器 | Span + Style | ✅ 已实现 |
| FR-087 | ThinkingBlock | AI思考过程显示 | 自定义 | 🔲 进行中 | GAP-P1-004 |

#### 3.5.3 对话框组件

| FR-ID | 组件 | 描述 | 状态 |
|-------|------|------|------|
| FR-090 | ModelSelection | 模型选择对话框 | ✅ 已实现 |
| FR-091 | ProviderManagement | Provider管理对话框 | ✅ 已实现 |
| FR-092 | ConnectMethod | 连接方式选择 | ✅ 已实现 |
| FR-093 | ConnectProvider | 连接Provider | ✅ 已实现 |
| FR-094 | ConnectModel | 连接模型 | ✅ 已实现 |
| FR-095 | Settings | 设置对话框 | ✅ 已实现 |
| FR-096 | DiffReview | Diff审查对话框 | ✅ 已实现 |
| FR-097 | FileSelection | 文件选择对话框 | ✅ 已实现 |
| FR-098 | DirectorySelection | 目录选择对话框 | ✅ 已实现 |
| FR-099 | ReleaseNotes | 发布说明对话框 | ✅ 已实现 |
| FR-100 | SlashCommand | 斜杠命令面板 | ✅ 已实现 |
| FR-101 | SessionShare | 会话分享对话框 | ⚠️ 部分实现 | GAP-P1-003: 本地导出完成，远程分享待实现 |
| FR-102 | SessionUnshare | 取消分享对话框 | 🔲 待实现 | GAP-P2-001 |

### 3.6 配置功能

| FR-ID | 配置项 | 类型 | 默认值 | 状态 |
|-------|--------|------|--------|------|
| FR-110 | scroll_speed | number | 3 | ✅ 已实现 |
| FR-111 | scroll_acceleration.enabled | boolean | true | ✅ 已实现 |
| FR-112 | theme | string | - | ✅ 已实现 |
| FR-113 | show_file_tree | boolean | - | ✅ 已实现 |
| FR-114 | show_skills_panel | boolean | - | ✅ 已实现 |
| FR-115 | keybinds | object | - | 🔲 待实现 |
| FR-116 | diff_style | string | - | 🔲 待实现 |
| FR-117 | typewriter_speed | number | 20 | 🔲 进行中 | GAP-P1-001 |

### 3.7 主题系统

| FR-ID | 功能 | 描述 | 状态 |
|-------|------|------|------|
| FR-120 | 主题切换 | 深色/浅色/自动主题切换 | ✅ 已实现 |
| FR-121 | ThemeManager | 主题管理器 | ✅ 已实现 |
| FR-122 | 自定义主题 | 用户自定义主题 | 🔲 待实现 |

### 3.8 权限模式

| FR-ID | 模式 | 描述 | 状态 |
|-------|------|------|------|
| FR-130 | ReadOnly | 只读模式 | ✅ 已实现 |
| FR-131 | WorkspaceWrite | 工作区写权限 | ✅ 已实现 |
| FR-132 | DangerFullAccess | 完全访问 | ✅ 已实现 |

### 3.9 输出格式

| FR-ID | 格式 | 描述 | 状态 |
|-------|------|------|------|
| FR-140 | Text | 纯文本输出 | ✅ 已实现 |
| FR-141 | Json | JSON输出 | ✅ 已实现 |
| FR-142 | Ndjson | NDJSON输出 | 🔲 待实现 |

### 3.10 会话管理

| FR-ID | 功能 | 描述 | 优先级 | 状态 | 差距分析 |
|-------|------|------|--------|------|----------|
| FR-150 | 会话创建 | 创建新会话 | P0 | ✅ 已实现 | - |
| FR-151 | 会话恢复 | 恢复指定会话 | P0 | ✅ 已实现 | - |
| FR-152 | 会话列表 | 列出会话历史 | P0 | ✅ 已实现 | - |
| FR-153 | 会话分享 | 分享当前会话 | P1 | ⚠️ 部分实现 | GAP-P1-003: 本地文件导出完成，远程分享待实现 |
| FR-154 | 取消分享 | 取消会话分享 | P1 | 🔲 待实现 | GAP-P2-001 |
| FR-155 | 会话压缩 | 压缩会话上下文 | P1 | ⚠️ 部分实现 | GAP-P2-003: 未实际调用 SummaryAgent |
| FR-156 | 会话导出 | 导出会话为Markdown | P0 | ✅ 已实现 | - |
| FR-157 | 会话分叉 | Fork当前会话创建分支 | P0 | ✅ 已实现 | - |
| FR-158 | 会话中止 | Abort正在进行的生成 | P0 | ✅ 已实现 | - |

### 3.11 模型别名

| FR-ID | 功能 | 描述 | 优先级 | 状态 | 差距分析 |
|-------|------|------|--------|------|----------|
| FR-160 | 别名解析 | 解析opus/sonnet/haiku等别名 | P1 | ✅ 已实现 | 模型别名在请求前正确解析 |
| FR-161 | 别名补全 | 在模型选择时提供别名补全 | P1 | 🔲 待实现 | - |

### 3.12 上下文预算控制

| FR-ID | 功能 | 描述 | 优先级 | 状态 | 差距分析 |
|-------|------|------|--------|------|----------|
| FR-170 | Token预算计算 | 计算并跟踪Token使用量 | P0 | ✅ 已实现 | - |
| FR-171 | 85%阈值触发 | 上下文达到85%时触发compact | P1 | 🔲 待实现 | GAP-P1-005: ContextBudget存在但未启用 |
| FR-172 | 92%阈值触发 | 上下文达到92%时触发compact | P1 | 🔲 待实现 | GAP-P1-005 |
| FR-173 | 95%阈值触发 | 上下文达到95%时触发compact | P1 | 🔲 待实现 | GAP-P1-005 |
| FR-174 | Relevance排序 | 根据相关性排序上下文 | P1 | ❌ 未实现 | GAP-P2-006: L0-L4层次部分实现 |

### 3.13 LSP集成

| FR-ID | 功能 | 描述 | 优先级 | 状态 | 差距分析 |
|-------|------|------|--------|------|----------|
| FR-180 | 诊断信息 | 显示LSP诊断信息 | P0 | ✅ 已实现 | - |
| FR-181 | 工作区符号 | 搜索工作区符号 | P0 | ✅ 已实现 | - |
| FR-182 | 文档符号 | 搜索文档内符号 | P0 | ✅ 已实现 | - |
| FR-183 | 定义跳转 | 跳转到定义 | P1.1 | ❌ 未实现 | GAP-P2-007 |
| FR-184 | 引用查找 | 查找引用 | P1.1 | ❌ 未实现 | GAP-P2-007 |
| FR-185 | 悬停信息 | 显示悬停信息 | P1.1 | ❌ 未实现 | GAP-P2-007 |
| FR-186 | 代码动作 | 显示代码动作 | P1.1 | ❌ 未实现 | GAP-P2-007 |

### 3.14 MCP集成

| FR-ID | 功能 | 描述 | 优先级 | 状态 | 差距分析 |
|-------|------|------|--------|------|----------|
| FR-190 | 本地MCP | 连接本地MCP服务器 | P0 | ✅ 已实现 | - |
| FR-191 | 远程MCP | 连接远程MCP服务器 | P0 | ✅ 已实现 | - |
| FR-192 | 工具发现 | 自动发现MCP工具 | P0 | ✅ 已实现 | - |
| FR-193 | 工具执行 | 通过Agent执行MCP工具 | P1 | 🔲 进行中 | GAP-P2-008: McpToolAdapter存在但集成待验证 |
| FR-194 | Token成本控制 | 控制MCP调用的Token成本 | P1 | ⚠️ 部分实现 | GAP-P2-008 |

### 3.15 插件系统

| FR-ID | 功能 | 描述 | 优先级 | 状态 | 差距分析 |
|-------|------|------|--------|------|----------|
| FR-200 | WASM插件 | 加载和执行WASM插件 | P0 | ✅ 已实现 | - |
| FR-201 | Sidecar插件 | Sidecar模式插件支持 | P0 | ⚠️ 部分实现 | GAP-P2-009: 框架存在 |
| FR-202 | 事件总线 | 插件间事件通信 | P0 | ✅ 已实现 | - |
| FR-203 | 插件管理 | 插件注册、启动、关闭 | P0 | ✅ 已实现 | - |

### 3.16 数据模型规格

| FR-ID | 数据模型 | 描述 | 状态 | 差距分析 |
|-------|----------|------|------|----------|
| FR-210 | ShareStatus | 会话分享状态管理 | 🔲 待实现 | GAP-P2-001: ShareStatus缺失 |
| FR-211 | ThinkingMode | 思考模式配置传递 | 🔲 进行中 | GAP-P1-004: thinking_mode存在但传递链路未完成 |
| FR-212 | BudgetLimit | 预算限制配置 | 🔲 待实现 | GAP-P1-005: ContextBudget存在但触发机制缺失 |
| FR-213 | UsageStats | 使用统计聚合 | 🔲 待实现 | GAP-P2-010: UsageStats缺失 |

### 3.17 Web UI

| FR-ID | 功能 | 描述 | 优先级 | 状态 | 差距分析 |
|-------|------|------|--------|------|----------|
| FR-220 | Web UI基础 | Web界面基础框架 | P1 | 🔲 进行中 | GAP-P2-011: web_ui.rs存在但功能不完整 |
| FR-221 | 会话管理UI | Web界面会话管理 | P1 | 🔲 待实现 | PRD v1.5目标 |
| FR-222 | 聊天界面UI | Web界面聊天交互 | P1 | 🔲 待实现 | PRD v1.5目标 |

---

## 4. 差距分析汇总

### 4.1 P0 阻断性问题 (必须修复)

| 差距ID | 模块 | 描述 | 影响 | 修复FR |
|--------|------|------|------|--------|
| **无P0阻断性问题** | - | 核心架构稳定，主要功能已实现 | - | - |

> **说明**: 相比 iteration-18，iteration-19 已解决之前的 P0 问题（OpenTUI 架构误解已澄清，实际采用 Ratatui 是正确选择）。

### 4.2 P1 高优先级问题

| 差距ID | 模块 | 描述 | 影响 | 修复FR |
|--------|------|------|------|--------|
| GAP-P1-001 | TUI | 打字机效果未实现 | 流式输出体验差 | FR-006, FR-117 |
| GAP-P1-002 | TUI | Token实时显示未验证 | 无法确认用量显示 | FR-007 |
| GAP-P1-003 | Session | /share仅本地导出 | 无法远程分享 | FR-153, FR-101 |
| GAP-P1-004 | Agent | /thinking模式未传递给LLM | 思考块可见性切换不完整 | FR-059, FR-087, FR-211 |
| GAP-P1-005 | Context | 上下文预算触发未实现 | 85%/92%/95%阈值不生效 | FR-171, FR-172, FR-173, FR-212 |

### 4.3 P2 中优先级问题

| 差距ID | 模块 | 描述 | 影响 | 修复FR |
|--------|------|------|------|--------|
| GAP-P2-001 | Session | /unshare未实现 | 取消分享功能缺失 | FR-062, FR-102, FR-154, FR-210 |
| GAP-P2-002 | Search | /search未实现 | 无法搜索对话历史 | FR-054 |
| GAP-P2-003 | Session | /compact未调用SummaryAgent | 压缩功能不完整 | FR-044, FR-155 |
| GAP-P2-004 | Plugin | /plugins未实现 | 插件管理缺失 | FR-057 |
| GAP-P2-005 | Config | /username未实现 | 用户名设置缺失 | FR-058 |
| GAP-P2-006 | Context | relevance ranking未实现 | 上下文排序不完整 | FR-174 |
| GAP-P2-007 | LSP | definition/references/hover未实现 | LSP 1.1能力缺失 | FR-183, FR-184, FR-185, FR-186 |
| GAP-P2-008 | MCP | MCP工具执行集成未验证 | Agent调用链路不完整 | FR-193, FR-194 |
| GAP-P2-009 | Plugin | Sidecar插件支持不完整 | 插件模式受限 | FR-201 |
| GAP-P2-010 | Stats | UsageStats未实现 | 使用统计缺失 | FR-213 |
| GAP-P2-011 | Web UI | Web UI功能不完整 | PRD v1.5目标未达成 | FR-220, FR-221, FR-222 |

### 4.4 技术债务

| 债务ID | 描述 | 风险 | 修复建议 |
|--------|------|------|----------|
| TECH-001 | Custom(String)命令占位符10+个 | 高 | 逐一实现或移除 |
| TECH-002 | 硬编码值(MAX_OUTPUT_SIZE=100KB等) | 中 | 移至配置 |
| TECH-003 | 重复命令定义(undo定义两次) | 中 | 清理重复 |
| TECH-004 | Error处理不一致 | 中 | 统一Result类型 |
| TECH-005 | 魔法数字(100, 5000等) | 低 | 命名常量 |
| TECH-006 | `working_dir` 字段未使用 | 低 | 编译警告 |
| TECH-007 | 注释代码散布 | 低 | 清理 |

---

## 5. 验收标准

### 5.1 核心功能验收

- [x] FR-001: `opencode-rs` 命令可正常启动TUI
- [x] FR-002: 指定目录启动功能正常
- [x] FR-003: CLI参数解析正常工作
- [x] FR-010: `@` 语法可正确引用文件并进行模糊搜索
- [x] FR-011: `@` 引用的文件内容自动添加到AI上下文 (GAP-P0-002 已修复)
- [x] FR-020: `!` 语法可正确执行Shell命令
- [x] FR-023: `!` 命令输出结果渲染到UI (GAP-P0-003 已修复)
- [ ] FR-030~FR-065: 斜杠命令实现状态见FR表
- [x] FR-130~FR-132: 权限模式切换正常

### 5.2 UI/组件验收

- [x] FR-080: 消息气泡正确渲染
- [x] FR-081: 代码块语法高亮显示
- [x] FR-075: 文件树组件正常工作
- [x] FR-086: 命令面板可正常打开和搜索
- [x] FR-073: 输入组件正常工作
- [ ] FR-087: 思考块可见性可切换 (GAP-P1-004 进行中)

### 5.3 待完成项

| FR-ID | 功能 | 优先级 | 差距ID |
|-------|------|--------|--------|
| FR-006 | 打字机效果 | P1 | GAP-P1-001 |
| FR-007 | 实时Token计数 | P1 | GAP-P1-002 |
| FR-054 | /search对话历史搜索 | P2 | GAP-P2-002 |
| FR-055 | /diff Git差异 | P2 | GAP-P2-001 |
| FR-056 | /memory记忆管理 | P2 | GAP-P2-003 |
| FR-057 | /plugins插件管理 | P2 | GAP-P2-004 |
| FR-058 | /username用户名设置 | P2 | GAP-P2-005 |
| FR-059 | /thinking思考模式切换 | P1 | GAP-P1-004 |
| FR-060 | /status会话状态 | P2 | GAP-P2-006 |
| FR-061 | /share会话分享 | P1 | GAP-P1-003 |
| FR-062 | /unshare取消分享 | P2 | GAP-P2-001 |
| FR-064 | /editor外部编辑器 | P2 | GAP-P2-007 |
| FR-065 | /init AGENTS.md初始化 | P2 | GAP-P2-008 |
| FR-084 | ProgressBar进度条 | P2 | - |
| FR-087 | ThinkingBlock思考块 | P1 | GAP-P1-004 |
| FR-101 | SessionShare分享对话框 | P1 | GAP-P1-003 |
| FR-102 | SessionUnshare取消对话框 | P2 | GAP-P2-001 |
| FR-115 | 自定义快捷键配置 | P2 | - |
| FR-116 | Diff样式配置 | P2 | - |
| FR-122 | 自定义主题 | P2 | - |
| FR-142 | NDJSON输出格式 | P2 | - |
| FR-161 | 模型别名补全 | P1 | - |
| FR-171-173 | 上下文预算触发 | P1 | GAP-P1-005 |
| FR-174 | Relevance排序 | P1 | GAP-P2-006 |
| FR-183-186 | LSP 1.1能力 | P1.1 | GAP-P2-007 |
| FR-193-194 | MCP工具执行 | P1 | GAP-P2-008 |
| FR-201 | Sidecar插件 | P1 | GAP-P2-009 |
| FR-210 | ShareStatus | P2 | GAP-P2-001 |
| FR-211 | ThinkingMode | P1 | GAP-P1-004 |
| FR-212 | BudgetLimit | P1 | GAP-P1-005 |
| FR-213 | UsageStats | P2 | GAP-P2-010 |
| FR-220-222 | Web UI | P1 | GAP-P2-011 |

---

## 6. 短期修复计划 (v19)

### 6.1 P1 修复 (本周)

| 任务 | FR-ID | 描述 | 预计工时 |
|------|-------|------|----------|
| 打字机效果 | FR-006, FR-117 | 在 `check_llm_events()` 中实现增量文本渲染 | M |
| Token实时显示验证 | FR-007 | 确认 `StatusBar.update_usage()` 实际渲染 | S |
| /thinking模式传递 | FR-059, FR-211 | 在消息构建时传递thinking标志到LLM provider | M |
| 上下文预算触发 | FR-171-173, FR-212 | 实现85%/92%/95%阈值自动compact | M |
| /share远程分享 | FR-153, FR-101 | 实现远程分享服务或连接到外部服务 | M |

### 6.2 P2 修复 (下周~下两周)

| 任务 | FR-ID | 描述 | 预计工时 |
|------|-------|------|----------|
| /unshare实现 | FR-062, FR-102, FR-154, FR-210 | 实现本地分享状态管理 | S |
| /compact摘要 | FR-044, FR-155 | 实现CompactionAgent调用 | M |
| LSP 1.1能力 | FR-183-186 | 实现definition/references/hover/code actions | L |
| MCP工具集成 | FR-193, FR-194 | 验证register_mcp_tools()调用链路 | M |
| /search实现 | FR-054 | 对话历史搜索 | M |
| /diff集成 | FR-055 | Git差异显示集成 | M |
| Web UI | FR-220-222 | 完成Web前端实现 | L |
| 技术债务清理 | TECH-001~007 | 清理未使用常量、魔法数字、重复定义 | M |

---

## 7. 竞品分析与借鉴

### 7.1 参考项目

| 项目 | 特点 | 借鉴价值 |
|------|------|----------|
| rusty-claude-cli | Ratatui TUI, CLI参数系统, 会话管理 | 高 |
| ratatui | Rust TUI库 | 参考 |

### 7.2 CLI参数设计 (已实现)

```bash
# 基础使用
opencode-rs                          # 启动交互式TUI
opencode-rs /path/to/project        # 指定项目目录

# 模型选择
opencode-rs --model claude-opus-4   # 指定模型
opencode-rs -m opus                 # 使用模型别名

# 权限控制
opencode-rs --permission-mode read-only
opencode-rs --permission-mode workspace-write
opencode-rs --dangerously-skip-permissions

# 输出格式
opencode-rs --output-format text
opencode-rs --output-format json

# 会话管理
opencode-rs --session-id <id>       # 恢复会话
```

---

## 8. 参考文档

### 8.1 内部文档

- [Constitution](../.specify/memory/constitution.md)
- [C-024 Session Tools Permission](../.specify/constitutions/C-024.md)
- [C-055 Test Coverage Requirements](../.specify/constitutions/C-055.md)
- [C-056 Config JSONC Migration](../.specify/constitutions/C-056.md)

### 8.2 外部资源

- [Ratatui GitHub](https://github.com/ratatui/ratatui)
- [Crossterm Documentation](https://github.com/crossterm-rs/crossterm)
- [Tokio Documentation](https://tokio.rs)

---

## 附录 A: 组件清单

| 组件名 | 文件 | FR-ID | 状态 |
|--------|------|-------|------|
| App | app.rs | - | ✅ |
| Banner | components/banner.rs | FR-070 | ✅ |
| TitleBar | components/title_bar.rs | FR-072 | ✅ |
| StatusBar | components/status_bar.rs | FR-072 | ✅ |
| InputWidget | components/input_widget.rs | FR-073 | ✅ |
| VirtualList | components/virtual_list.rs | FR-074 | ✅ |
| FileTree | components/file_tree.rs | FR-075 | ✅ |
| TerminalPanel | components/terminal_panel.rs | FR-076 | ✅ |
| RightPanel | right_panel.rs | FR-077 | ✅ |
| DiffView | components/diff_view.rs | FR-078 | ✅ |
| SkillsPanel | components/skills_panel.rs | FR-079 | ✅ |
| MessageBubble | widgets/message_bubble.rs | FR-080 | ✅ |
| CodeBlock | widgets/code_block.rs | FR-081 | ✅ |
| ThinkingIndicator | widgets/indicators.rs | FR-082 | ✅ |
| Spinner | widgets/spinner.rs | FR-083 | ✅ |
| Scrollbar | widgets/scrollbar.rs | FR-085 | ✅ |
| Indicators | widgets/indicators.rs | FR-086 | ✅ |
| CommandPalette | widgets/command_palette.rs | - | ✅ |
| FileSelectionList | widgets/file_selection.rs | - | ✅ |
| ProgressBar | - | FR-084 | 🔲 |
| ThinkingBlock | - | FR-087 | 🔲 |
| SessionShare | - | FR-101 | ⚠️ |
| SessionUnshare | - | FR-102 | 🔲 |

---

## 附录 B: 差距分析原始数据

### B.1 差距分析来源

- **文件**: outputs/iteration-19/gap-analysis.md
- **分析日期**: 2026-04-08
- **分析版本**: 19.0

### B.2 总体实现进度

```
█████████████████████░░░░░░░░░ 85% 完成
```

### B.3 按模块进度

| 模块 | 完成度 | 说明 |
|------|--------|------|
| Core 领域模型 | ████████████ 95% | 核心类型定义完整 |
| Config 系统 | ████████████ 95% | JSONC + 多层合并 |
| Storage 层 | █████████░░ 85% | SQLite + FS |
| LLM Provider | ████████████ 95% | 20+ providers |
| Agent 系统 | ████████████ 95% | 7 种 agent 类型 |
| Tool Runtime | █████████░░ 85% | 核心工具完整 |
| Permission | ████████████ 90% | 评估器 + 队列 + 审计 |
| TUI | ████████░░░ 80% | 核心完成，缺流式渲染 |
| CLI | ████████████ 95% | 35+ 子命令 |
| Server | ████████░░░ 80% | REST 端点完整，Web UI 缺 |
| LSP | ██████░░░░░ 60% | diagnostics 完整，其他 LSP 1.1 |
| MCP | ███████░░░░ 70% | 客户端完整，工具集成缺 |
| Plugin | ███████░░░░ 70% | WASM runtime 完整 |
| Skills | █████████░░ 85% | 发现/匹配/状态管理 |
| Git | ███████░░░░ 70% | 基础操作完整，集成缺 |
| Session | █████████░░ 85% | fork/summarize 未完全实现 |

---

**版本**: 19  
**最后更新**: 2026-04-08  
**维护者**: OpenCode Rust Team