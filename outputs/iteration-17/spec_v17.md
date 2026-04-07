# OpenCode-RS 规格文档 v17

**版本**: 17  
**日期**: 2026年4月7日  
**作者**: OpenCode Rust Team  
**状态**: 已发布

---

## 变更日志

| 版本 | 日期 | 变更描述 |
|------|------|----------|
| v17 | 2026-04-07 | 基于PRD v2.4 (Rust Edition) 创建，包含完整的TUI功能规格和FR编号 |
| v16 | 2026-04-06 | 添加MCP协议、插件系统规格 |
| v15 | 2026-03-27 | 完成功能对标补充 |
| v14 | 2026-03-26 | Rust OpenCode Port 规格初版 |

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
│  (Build/Plan/Review/Refactor/Debug Agent)                   │
├──────────────────────────────────────────────────────────────┤
│                      opencode-tools                         │
│  (文件操作、Git、Bash、Search等工具)                          │
├──────────────────────────────────────────────────────────────┤
│                      opencode-llm                           │
│  (多Provider支持：OpenAI/Anthropic/Ollama)                   │
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

---

## 2. TUI 架构规格

### 2.1 核心依赖配置

```toml
[dependencies]
ratatui = { version = "0.30", default-features = false, features = ["crossterm"] }
crossterm = { version = "0.28", features = ["event-stream"] }
tokio = { version = "1.45", features = ["full"] }
syntect = "5"
pulldown-cmark = "0.13"
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

| FR-ID | 功能点 | 描述 | 优先级 | 状态 |
|-------|--------|------|--------|------|
| FR-001 | TUI启动 | 运行 `opencode-rs` 启动TUI | P0 | ✅ 已实现 |
| FR-002 | 指定目录启动 | 运行 `opencode-rs /path/to/project` 启动指定目录 | P0 | ✅ 已实现 |
| FR-003 | CLI参数解析 | 支持 `--model`, `--permission-mode`, `--output-format` 等 | P0 | ✅ 已实现 |
| FR-004 | 消息输入 | 在TUI中输入消息进行提示 | P0 | ✅ 已实现 |
| FR-005 | AI响应渲染 | 使用Ratatui组件渲染AI响应 | P0 | ✅ 已实现 |
| FR-006 | 打字机效果 | 流式输出的打字机渲染效果 | P1 | 🔲 待实现 |

### 3.2 文件引用功能

| FR-ID | 功能点 | 描述 | 优先级 | 状态 |
|-------|--------|------|--------|------|
| FR-010 | 文件模糊搜索 | 使用 `@` 在消息中引用文件 | P0 | ✅ 已实现 |
| FR-011 | 文件内容加载 | 引用的文件内容自动添加到上下文 | P0 | ✅ 已实现 |
| FR-012 | 文件选择列表 | 搜索结果通过List组件展示 | P0 | ✅ 已实现 |
| FR-013 | @路径补全 | 输入时自动补全文件路径 | P1 | 🔲 待实现 |

### 3.3 Bash命令执行

| FR-ID | 功能点 | 描述 | 优先级 | 状态 |
|-------|--------|------|--------|------|
| FR-020 | Shell命令执行 | 以 `!` 开头的消息作为shell命令执行 | P0 | ✅ 已实现 |
| FR-021 | 输出结果集成 | 命令输出作为工具结果添加 | P0 | ✅ 已实现 |
| FR-022 | 命令终止 | 支持Ctrl+C终止正在执行的命令 | P1 | 🔲 待实现 |

### 3.4 斜杠命令系统

| FR-ID | 命令 | 别名 | 描述 | 快捷键 | 状态 |
|-------|------|------|------|--------|------|
| FR-030 | `/plan` | `/p` | 切换到plan模式 | - | ✅ 已实现 |
| FR-031 | `/build` | `/b` | 切换到build模式 | - | ✅ 已实现 |
| FR-032 | `/clear` | - | 清空消息 | - | ✅ 已实现 |
| FR-033 | `/help` | `/h`, `/?` | 显示帮助 | `ctrl+x h` | ✅ 已实现 |
| FR-034 | `/timeline` | `/t` | 打开时间线视图 | - | ✅ 已实现 |
| FR-035 | `/fork` | `/f` | 在当前消息处创建分支 | - | ✅ 已实现 |
| FR-036 | `/meta` | `/m` | 切换元数据显示 | - | ✅ 已实现 |
| FR-037 | `/settings` | `/,` | 打开设置对话框 | - | ✅ 已实现 |
| FR-038 | `/models` | - | 打开模型选择 | `ctrl+x m` | ✅ 已实现 |
| FR-039 | `/providers` | - | 打开Provider管理 | - | ✅ 已实现 |
| FR-040 | `/connect` | - | 连接Provider | - | ✅ 已实现 |
| FR-041 | `/files` | - | 切换文件树面板 | - | ✅ 已实现 |
| FR-042 | `/skills` | - | 切换技能面板 | - | ✅ 已实现 |
| FR-043 | `/release-notes` | `/rn` | 打开发布说明 | - | ✅ 已实现 |
| FR-044 | `/compact` | `/c` | 压缩会话 | `ctrl+x c` | ✅ 已实现 |
| FR-045 | `/summarize` | `/s` | 总结当前对话 | - | ✅ 已实现 |
| FR-046 | `/export` | `/e` | 导出会话到Markdown | `ctrl+x x` | ✅ 已实现 |
| FR-047 | `/undo` | `/u` | 撤销最后文件变更 | `ctrl+x u` | ✅ 已实现 |
| FR-048 | `/sessions` | `/ses` | 列出会话并管理 | `ctrl+x l` | ✅ 已实现 |
| FR-049 | `/new` | - | 创建新会话 | `ctrl+x n` | ✅ 已实现 |
| FR-050 | `/details` | `/d` | 切换工具执行详情 | `ctrl+x d` | ✅ 已实现 |
| FR-051 | `/themes` | - | 列出可用主题 | `ctrl+x t` | ✅ 已实现 |
| FR-052 | `/theme` | - | 切换到下一主题 | - | ✅ 已实现 |
| FR-053 | `/exit` | `/q` | 退出应用 | `ctrl+x q` | ✅ 已实现 |
| FR-054 | `/search` | - | 搜索对话历史 | - | ✅ 已实现 |
| FR-055 | `/diff` | - | 显示Git Diff | - | ✅ 已实现 |
| FR-056 | `/memory` | - | 管理记忆条目 | - | ✅ 已实现 |
| FR-057 | `/plugins` | - | 管理插件 | - | ✅ 已实现 |
| FR-058 | `/username` | - | 设置显示用户名 | - | ✅ 已实现 |
| FR-059 | `/thinking` | - | 切换思考模式 | - | ✅ 已实现 |
| FR-060 | `/status` | - | 显示会话状态 | - | ✅ 已实现 |
| FR-061 | `/share` | - | 分享当前会话 | `ctrl+x s` | ✅ 已实现 |
| FR-062 | `/redo` | - | 重做 | `ctrl+x r` | ✅ 已实现 |
| FR-063 | `/editor` | - | 打开外部编辑器编写消息 | `ctrl+x e` | 🔲 待实现 |
| FR-064 | `/init` | - | 创建或更新AGENTS.md | `ctrl+x i` | 🔲 待实现 |

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

---

## 4. 验收标准

### 4.1 核心功能验收

- [x] FR-001: `opencode-rs` 命令可正常启动TUI
- [x] FR-002: 指定目录启动功能正常
- [x] FR-003: CLI参数解析正常工作
- [x] FR-010: `@` 语法可正确引用文件并进行模糊搜索
- [x] FR-020: `!` 语法可正确执行Shell命令并返回输出
- [x] FR-030~FR-062: 所有斜杠命令可正常执行
- [x] FR-130~FR-132: 权限模式切换正常

### 4.2 UI/组件验收

- [x] FR-080: 消息气泡正确渲染
- [x] FR-081: 代码块语法高亮显示
- [x] FR-075: 文件树组件正常工作
- [x] FR-086: 命令面板可正常打开和搜索
- [x] FR-073: 输入组件正常工作

### 4.3 待完成项 (P1)

- [ ] FR-006: 打字机效果优化
- [ ] FR-022: Ctrl+C 命令终止
- [ ] FR-063: `/editor` 命令
- [ ] FR-064: `/init` 命令
- [ ] FR-084: 进度条组件
- [ ] FR-115: 自定义快捷键配置
- [ ] FR-116: Diff样式配置
- [ ] FR-142: NDJSON输出格式

---

## 5. 竞品分析与借鉴

### 5.1 参考项目

| 项目 | 特点 | 借鉴价值 |
|------|------|----------|
| rusty-claude-cli | Ratatui TUI, CLI参数系统, 会话管理 | 高 |
| ratatui | Rust TUI库 | 参考 |

### 5.2 CLI参数设计 (已实现)

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

## 6. 后续规划

### 6.1 短期规划 (v17-v18)

| 任务 | 描述 | 优先级 |
|------|------|--------|
| 实现打字机效果 | 流式输出优化 | P1 |
| 命令终止功能 | Ctrl+C支持 | P1 |
| `/editor` 命令 | 外部编辑器集成 | P1 |
| `/init` 命令 | AGENTS.md初始化 | P1 |

### 6.2 中期规划 (v18-v20)

| 任务 | 描述 | 优先级 |
|------|------|--------|
| 进度条组件 | Gauge组件 | P2 |
| 自定义快捷键 | keybinds配置 | P2 |
| Diff样式配置 | diff_style | P2 |
| NDJSON输出 | 流式JSON | P2 |

### 6.3 长期规划

| 任务 | 描述 | 优先级 |
|------|------|--------|
| 插件系统增强 | WASM插件支持 | P3 |
| 主题市场 | 用户分享主题 | P3 |

---

## 7. 参考文档

### 7.1 内部文档

- [Constitution](../.specify/memory/constitution.md)
- [C-024 Session Tools Permission](../.specify/constitutions/C-024.md)
- [C-055 Test Coverage Requirements](../.specify/constitutions/C-055.md)
- [C-056 Config JSONC Migration](../.specify/constitutions/C-056.md)

### 7.2 外部资源

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

---

**版本**: 17  
**最后更新**: 2026-04-07  
**维护者**: OpenCode Rust Team
