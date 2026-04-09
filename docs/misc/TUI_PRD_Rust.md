# TUI (终端用户界面) 产品需求文档

**版本：** 2.4 (Rust Edition)  
**日期：** 2026年4月7日  
**作者：** mycode 产品团队  
**状态：** 已发布

---

## 1. 概述

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

### 1.4 技术架构概览

```
┌─────────────────────────────────────────────────────────┐
│                    mycode TUI 应用层                    │
│  (会话管理、命令处理、LLM集成、协作分享)                   │
├─────────────────────────────────────────────────────────┤
│                    Rust TUI 渲染层                        │
│  (Ratatui + 自定义组件)                                  │
│  - 组件系统 (Paragraph, List, Table, Modal, etc.)        │
│  - 布局系统 (Flex, Grid, Stack)                         │
│  - 事件系统 (键盘、鼠标、滚动)                           │
│  - 主题系统                                             │
├─────────────────────────────────────────────────────────┤
│                    Crossterm 终端层                       │
│  (跨平台终端控制、ANSI 渲染、输入捕获)                    │
├─────────────────────────────────────────────────────────┤
│                    Tokio 异步运行时                       │
│  (异步任务调度、并发处理)                                 │
└─────────────────────────────────────────────────────────┘
```

---

## 2. Ratatui 架构详解

### 2.1 核心库概览

Ratatui 是 Rust 生态最成熟的 TUI 库，采用模块化架构设计。

| 特性 | 说明 |
|------|------|
| 许可证 | MIT |
| 下载量 | 3000万+ |
| MSRV | 1.59.0 (v0.30) |
| 活跃度 | 85+ releases，高活跃开发 |
| 星标 | 19401+ |
| 架构 | 模块化 crate 设计 |

### 2.2 包结构

Ratatui v0.30.0 采用多 crate 架构：

```
ratatui
├── ratatui          # 核心库，重新导出所有模块
├── ratatui-core     # 核心 trait 和类型
├── ratatui-boolean - 布尔值 widget
├── ratatui-widgets # 内置 widgets
└── ratatui-macros  # 宏定义
```

### 2.3 核心依赖配置

```toml
[dependencies]
# 核心库
ratatui = "0.30"

# 终端控制
crossterm = "0.28"

# 异步运行时
tokio = { version = "1", features = ["full"] }

# 错误处理
anyhow = "1"
thiserror = "2"

# 日志
tracing = "0.1"
tracing-subscriber = "0.3"

# 可选：语法高亮
syntect = "5"
tree-sitter = "0.24"
```

### 2.4 组件系统

Ratatui 提供丰富的内置 Widget：

#### 2.4.1 布局组件

| Widget | 描述 |
|--------|------|
| `Block` | 带边框的容器组件 |
| `Clear` | 清除屏幕区域 |
| `Split` | 分割屏幕区域 |

#### 2.4.2 文本组件

| Widget | 描述 |
|--------|------|
| `Paragraph` | 多行文本渲染，支持滚动 |
| `Scrollbar` | 滚动条 |
| `Sparkline` | 数据可视化 |

#### 2.4.3 数据展示组件

| Widget | 描述 |
|--------|------|
| `BarChart` | 柱状图 |
| `Chart` | 图表 |
| `Gauge` | 仪表盘 |
| `List` | 列表 |
| `Table` | 表格 |
| `Tabs` | Tab 切换 |

#### 2.4.4 输入组件

| Widget | 描述 |
|--------|------|
| `Checkbox` | 复选框 |
| `Input` | 文本输入 |
| `Radio` | 单选按钮 |
| `Select` | 下拉选择 |

#### 2.4.5 其他组件

| Widget | 描述 |
|--------|------|
| `Calendar` | 日历 |
| `Canvas` | 画布 |
| `LineGauge` | 行式仪表 |

### 2.5 布局系统

Ratatui 支持多种布局方式：

```rust
use ratatui::layout::{Constraint, Direction, Layout};

// 垂直布局
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(area);

// 水平布局
let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
        Constraint::Percentage(30),
        Constraint::Percentage(70),
    ])
    .split(area);
```

**Constraint 类型：**
- `Length(n)` - 固定长度
- `Percentage(n)` - 百分比
- `Ratio(n, d)` - 比例
- `Min(n)` - 最小值
- `Max(n)` - 最大值
- `Fill` - 填充剩余空间

### 2.6 样式系统

```rust
use ratatui::style::{Color, Style, Stylize};

let style = Style::default()
    .fg(Color::White)
    .bg(Color::DarkGray)
    .add_modifier(Modifier::BOLD)
    .add_modifier(Modifier::ITALIC);
```

**颜色支持：**
- `Color::Reset` - 重置为默认
- `Color::Black` / `White` - 标准颜色
- `Color::Indexed(n)` - 256 色索引
- `Color::Rgb(r, g, b)` - 24 位真彩色
- `Color::LightBlue` / `DarkGray` - 亮色/暗色变体

**修饰符：**
- `Modifier::BOLD` - 粗体
- `Modifier::DIM` - 暗淡
- `Modifier::ITALIC` - 斜体
- `Modifier::UNDERLINED` - 下划线
- `Modifier::REVERSED` - 反色
- `Modifier::CROSSED_OUT` - 删除线
- `Modifier::BLINK` - 闪烁

### 2.7 事件处理

```rust
use ratatui::event::{Event, EventHandler, KeyEvent, MouseEvent};

// 键盘事件
match event {
    Event::Key(KeyEvent { code, modifiers, .. }) => {
        match (code, modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
            (KeyCode::Char('q'), KeyModifiers::NONE) => break,
            _ => {}
        }
    }
    Event::Mouse(MouseEvent { kind, column, row, .. }) => {
        // 鼠标事件处理
    }
    Event::Resize(width, height) => {
        // 窗口大小调整
    }
}
```

### 2.8 新版 run() API

Ratatui v0.30 引入了 `ratatui::run()` API：

```rust
use ratatui::prelude::*;
use ratatui::Terminal;

fn main() -> Result<(), Box<dyn Error>> {
    ratatui::run(|terminal| {
        let mut app = App::new();
        loop {
            terminal.draw(|f| app.render(f))?;
            if app.should_exit() {
                return Ok(());
            }
        }
    })?;
    Ok(())
}
```

### 2.9 与其他 Rust TUI 库对比

| 库 | 星标 | 下载量 | 特点 |
|----|------|--------|------|
| **Ratatui** | 19401 | 3000万+ | 最成熟，功能全面，模块化架构 |
| **r3bl_tui** | - | 5906/月 | Rust 2024，活跃开发 |
| **Iocraft** | - | - | React-like API |
| **feather_tui** | - | - | 轻量级 |
| **MinUI** | - | - | 游戏导向，轻量 |

---

## 3. 功能需求

### 3.1 启动与基础交互

| 功能点 | 描述 | 优先级 |
|--------|------|--------|
| 启动 TUI | 运行 `mycode` 启动当前目录的 TUI | P0 |
| 指定目录启动 | 运行 `mycode /path/to/project` 启动指定目录的 TUI | P0 |
| 消息输入提示 | 进入 TUI 后可输入消息进行提示 | P0 |
| AI 响应渲染 | 使用 Ratatui 组件渲染 AI 响应（消息气泡、代码块等） | P0 |
| 打字机效果 | 支持流式输出的打字机渲染效果 | P1 |

### 3.2 文件引用功能

| 功能点 | 描述 | 优先级 |
|--------|------|--------|
| 文件模糊搜索 | 使用 `@` 在消息中引用文件，进行模糊文件搜索 | P0 |
| 文件内容自动加载 | 引用的文件内容会自动添加到对话上下文中 | P0 |
| 引用列表选择 | 文件搜索结果通过 TUI 列表组件展示 | P0 |

**使用示例：**

```
How is auth handled in @packages/functions/src/api/index.ts?
```

### 3.3 Bash 命令执行

| 功能点 | 描述 | 优先级 |
|--------|------|--------|
| Shell 命令执行 | 以 `!` 开头的消息会作为 shell 命令执行 | P0 |
| 输出结果集成 | 命令的输出会作为工具结果添加到对话中 | P0 |
| 命令终止 | 支持 Ctrl+C 终止正在执行的命令 | P1 |

**使用示例：**

```
!ls -la
```

### 3.4 斜杠命令

| 命令 | 别名 | 描述 | 快捷键 |
|------|------|------|--------|
| `/connect` | - | 将提供商添加到 mycode，选择并添加 API 密钥 | - |
| `/compact` | `/summarize` | 压缩当前会话 | `ctrl+x c` |
| `/details` | - | 切换工具执行详情的显示 | `ctrl+x d` |
| `/editor` | - | 打开外部编辑器编写消息 | `ctrl+x e` |
| `/exit` | `/quit`, `/q` | 退出 mycode | `ctrl+x q` |
| `/export` | - | 将当前对话导出为 Markdown 并在默认编辑器中打开 | `ctrl+x x` |
| `/help` | - | 显示帮助对话框 | `ctrl+x h` |
| `/init` | - | 创建或更新 `AGENTS.md` 文件 | `ctrl+x i` |
| `/models` | - | 列出可用模型 | `ctrl+x m` |
| `/new` | `/clear` | 开始新的会话 | `ctrl+x n` |
| `/redo` | - | 重做之前撤销的消息（需先使用 `/undo`） | `ctrl+x r` |
| `/sessions` | `/resume`, `/continue` | 列出会话并在会话之间切换 | `ctrl+x l` |
| `/share` | - | 分享当前会话 | `ctrl+x s` |
| `/themes` | - | 列出可用主题 | `ctrl+x t` |
| `/thinking` | - | 切换对话中思考/推理块的可见性 | - |
| `/undo` | - | 撤销对话中的最后一条消息及所有文件更改 | `ctrl+x u` |
| `/unshare` | - | 取消分享当前会话 | - |

#### 3.4.1 撤销/重做功能说明

- `/undo` 和 `/redo` 使用 Git 来管理文件更改
- 撤销时会移除最近的用户消息、所有后续响应以及所有文件更改
- 使用此功能需要项目是一个 Git 仓库

#### 3.4.2 思考模式说明

- `/thinking` 命令仅控制思考块是否**显示**，不会启用或禁用模型的推理能力
- 要切换实际的推理能力，请使用 `ctrl+t` 循环切换模型变体

### 3.5 组件与交互设计

基于 Ratatui 的组件系统，TUI 实现以下核心 UI 组件：

#### 3.5.1 布局与容器组件

| 组件 | 描述 | Ratatui 基础 |
|------|------|--------------|
| 消息气泡 | 显示用户和 AI 的对话消息 | `Paragraph` + `Block` |
| 代码块 | 语法高亮显示代码 | `Paragraph` + 自定义样式 |
| 容器 | 带边框的内容区域 | `Block` |

#### 3.5.2 列表与选择组件

| 组件 | 描述 | Ratatui 基础 |
|------|------|--------------|
| 文件选择器 | 文件搜索结果选择列表 | `List` + `StatefulList` |
| 命令面板 | 快捷命令访问面板 | `List` + `Input` |
| 会话列表 | 会话历史管理界面 | `Table` |

#### 3.5.3 进度与指示组件

| 组件 | 描述 | Ratatui 基础 |
|------|------|--------------|
| 进度指示 | 显示工具执行进度 | `Gauge` / `LineGauge` |
| 滚动条 | 内容滚动控制 | `Scrollbar` |
| Tab 切换 | 多个面板切换 | `Tabs` |

#### 3.5.4 数据展示组件

| 组件 | 描述 | Ratatui 基础 |
|------|------|--------------|
| 柱状图 | 统计数据可视化 | `BarChart` |
| 图表 | 折线图、散点图 | `Chart` |
| 仪表盘 | 百分比/进度显示 | `Gauge` |

### 3.6 编辑器设置

#### 3.6.1 支持的编辑器

| 编辑器 | 命令 | 备注 |
|--------|------|------|
| Visual Studio Code | `code` | 需加 `--wait` 参数 |
| Cursor | `cursor` | 需加 `--wait` 参数 |
| Windsurf | `windsurf` | 需加 `--wait` 参数 |
| Neovim | `nvim` | - |
| Vim | `vim` | - |
| Nano | `nano` | - |
| Sublime Text | `subl` | - |
| Notepad | `notepad` | Windows 默认 |

#### 3.6.2 环境变量配置

- **Linux/macOS**: 设置 `EDITOR` 环境变量
- **Windows CMD**: 使用 `set EDITOR=xxx`
- **Windows PowerShell**: 使用 `$env:EDITOR = "xxx"`

### 3.7 配置功能

| 配置项 | 类型 | 默认值 | 描述 |
|--------|------|--------|------|
| `scroll_acceleration.enabled` | boolean | `true` | 启用 macOS 风格的滚动加速 |
| `scroll_speed` | number | `3` | 滚动速度（最小值：1） |

**配置文件示例 (mycode.json)：**

```json
{
  "$schema": "https://mycode.ai/config.json",
  "tui": {
    "scroll_speed": 3,
    "scroll_acceleration": {
      "enabled": true
    }
  }
}
```

**说明：**
- `scroll_acceleration` 启用后，优先于 `scroll_speed` 设置
- 快速滚动时速度增加，慢速移动时保持精确

### 3.8 自定义功能

#### 3.8.1 用户名显示

- 可切换用户名是否显示在聊天消息中
- 设置通过命令面板（`ctrl+x h` 或 `/help`）访问
- 设置自动保存并在各 TUI 会话中记忆

#### 3.8.2 主题支持

- 浅色/深色/自动主题切换
- 使用 Ratatui 主题系统实现
- 支持自定义主题扩展

---

## 4. 技术实现

### 4.1 技术选型

#### 4.1.1 核心库：Ratatui

| 特性 | 说明 |
|------|------|
| 许可证 | MIT |
| 下载量 | 3000万+ |
| MSRV | 1.59.0 |
| 活跃度 | 85 releases，高活跃开发 |
| 星标 | 19401 |

**选择理由：**
- Rust 生态最成熟的 TUI 库
- 完全使用 Rust 实现，无外部 C 依赖
- 丰富的内置组件和布局系统
- 良好的主题支持
- 活跃的社区和维护

#### 4.1.2 终端层：Crossterm

| 特性 | 说明 |
|------|------|
| 功能 | 跨平台终端控制 |
| 平台 | Windows, macOS, Linux, BSD |

#### 4.1.3 异步运行时：Tokio

| 特性 | 说明 |
|------|------|
| 功能 | 异步任务调度、并发处理 |
| 集成 | 与 Ratatui 事件循环配合 |

### 4.2 依赖配置 (Cargo.toml)

```toml
[dependencies]
ratatui = "0.30"
crossterm = "0.28"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = "0.3"
```

### 4.3 架构设计

```
┌──────────────────────────────────────────────────────────────┐
│                      mycode TUI 应用层                      │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐              │
│  │  会话管理   │ │  命令处理   │ │ LLM 集成   │              │
│  └─────────────┘ └─────────────┘ └─────────────┘              │
├──────────────────────────────────────────────────────────────┤
│                      Rust TUI 渲染层                          │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐              │
│  │  Widgets   │ │  Layout     │ │  Theme     │              │
│  │  (组件)    │ │  (布局)     │ │  (主题)    │              │
│  └─────────────┘ └─────────────┘ └─────────────┘              │
├──────────────────────────────────────────────────────────────┤
│                      Terminal Adapter                        │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐              │
│  │ Crossterm  │ │  ANSI 渲染  │ │  输入捕获  │              │
│  └─────────────┘ └─────────────┘ └─────────────┘              │
├──────────────────────────────────────────────────────────────┤
│                      Async Runtime                           │
│  ┌─────────────────────────────────────────────────┐        │
│  │  Tokio (任务调度、并发、文件/网络 IO)              │        │
│  └─────────────────────────────────────────────────┘        │
└──────────────────────────────────────────────────────────────┘
```

### 4.4 模块设计

#### 3.4.1 核心模块

| 模块 | 职责 |
|------|------|
| `tui::app` | 应用状态管理、事件循环 |
| `tui::widgets` | 自定义 TUI 组件 |
| `tui::layout` | 布局管理 |
| `tui::theme` | 主题系统 |
| `tui::input` | 输入处理 |
| `tui::events` | 事件系统 |

#### 3.4.2 自定义组件

| 组件 | 描述 |
|------|------|
| `MessageBubble` | 对话消息气泡 |
| `CodeBlock` | 代码块（语法高亮） |
| `FilePicker` | 文件选择器 |
| `CommandPalette` | 命令面板 |
| `ThinkingBlock` | AI 思考过程显示 |
| `ToolDetail` | 工具执行详情 |
| `SessionList` | 会话列表 |

### 4.5 依赖要求

- **运行时**：
  - Rust 1.59+ (MSRV)
  - Cargo 包管理器
  - Git（用于撤销/重做功能）
  
- **终端要求**：
  - 支持 ANSI/VT100 的终端模拟器
  - 建议使用现代终端（iTerm2, Windows Terminal, Alacritty 等）

- **可选依赖**：
  - 外部编辑器（用于 `/editor` 和 `/export` 命令）

### 4.6 性能目标

| 指标 | 目标值 |
|------|--------|
| 启动时间 | < 300ms |
| 消息渲染延迟 | < 16ms (60fps) |
| 滚动帧率 | >= 60fps |
| 内存占用（空闲） | < 30MB |
| 内存占用（大量消息） | < 100MB |
| 二进制大小 | < 10MB |

### 4.7 详细模块设计

参考 rusty-claude-cli 的模块设计，mycode TUI Rust 版本应包含以下模块：

#### 4.7.1 核心模块结构

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
│   ├── help.rs           # 帮助命令
│   ├── status.rs        # 状态命令
│   ├── compact.rs       # 压缩会话
│   ├── model.rs         # 模型切换
│   └── session.rs       # 会话管理
│
├── input/
│   ├── mod.rs           # 输入模块入口
│   ├── editor.rs        # 行编辑器
│   ├── completion.rs    # 命令补全
│   └── history.rs       # 历史记录
│
├── auth/
│   ├── mod.rs           # 认证模块
│   ├── oauth.rs         # OAuth 登录
│   └── credentials.rs   # 凭证管理
│
└── plugins/
    ├── mod.rs           # 插件模块
    ├── registry.rs      # 插件注册表
    └── manager.rs        # 插件管理器
```

#### 4.7.2 核心数据结构

```rust
// 应用配置
pub struct CliConfig {
    pub model: String,
    pub permission_mode: PermissionMode,
    pub output_format: OutputFormat,
    pub session_path: Option<PathBuf>,
    pub config_file: Option<PathBuf>,
}

// 权限模式
pub enum PermissionMode {
    ReadOnly,           // 只读模式
    WorkspaceWrite,     // 工作区写权限
    DangerFullAccess,  // 完全访问
}

// 输出格式
pub enum OutputFormat {
    Text,   // 纯文本
    Json,   // JSON
    Ndjson, // NDJSON (每行一个 JSON)
}

// 会话状态
pub struct SessionState {
    pub id: String,
    pub path: PathBuf,
    pub turns: usize,
    pub compacted_messages: usize,
    pub last_model: String,
    pub token_usage: TokenUsage,
}

// Token 使用量
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_creation: u32,
    pub cache_read: u32,
}

// 应用状态
pub struct AppState {
    pub config: CliConfig,
    pub session: SessionState,
    pub runtime: RuntimeState,
    pub renderer: TerminalRenderer,
}
```

#### 4.7.3 Spinner 实现参考

```rust
const SPINNER_FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub struct Spinner {
    frame_index: usize,
}

impl Spinner {
    pub fn tick(&mut self, label: &str, theme: &ColorTheme, out: &mut impl Write) -> io::Result<()> {
        let frame = SPINNER_FRAMES[self.frame_index % SPINNER_FRAMES.len()];
        self.frame_index += 1;
        queue!(
            out,
            SavePosition,
            MoveToColumn(0),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(theme.spinner_active),
            Print(format!("{frame} {label}")),
            ResetColor,
            RestorePosition
        )?;
        out.flush()
    }

    pub fn finish(&mut self, label: &str, theme: &ColorTheme, out: &mut impl Write) -> io::Result<()> {
        self.frame_index = 0;
        execute!(
            out,
            MoveToColumn(0),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(theme.spinner_done),
            Print(format!("✔ {label}\n")),
            ResetColor
        )?;
        out.flush()
    }
}
```

#### 4.7.4 Markdown 渲染实现参考

```rust
use pulldown_cmark::{Parser, Event, Tag, TagEnd};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;

pub struct TerminalRenderer {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    theme: Theme,
}

impl TerminalRenderer {
    pub fn render_markdown(&self, content: &str, out: &mut impl Write) -> io::Result<()> {
        let mut parser = Parser::new_ext(content, Options::all());
        let mut highlighter = HighlightLines::new(&self.theme, &self.syntax_set);

        while let Some(event) = parser.next() {
            match event {
                Event::Text(text) => write!(out, "{}", text)?,
                Event::Code(code) => self.render_code(&code, &mut highlighter, out)?,
                Event::Start(Tag::Heading { .. }) => write!(out, "\n{}", self.theme.heading)?,
                Event::End(TagEnd::Heading) => write!(out, "\n")?,
                _ => {}
            }
        }
        Ok(())
    }
}
```

---

## 4. 验收标准

### 4.1 核心功能验收

- [ ] `mycode` 命令可正常启动 TUI
- [ ] 指定目录启动功能正常：`mycode /path/to/project`
- [ ] `@` 语法可正确引用文件并进行模糊搜索
- [ ] `!` 语法可正确执行 Shell 命令并返回输出
- [ ] 所有斜杠命令可正常执行并返回预期结果
- [ ] 快捷键绑定正常工作
- [ ] 撤销/重做功能正常（需要 Git 仓库）
- [ ] 会话列表和切换功能正常
- [ ] 会话分享和取消分享功能正常

### 4.2 UI/组件验收

- [ ] 消息气泡正确渲染（用户和 AI 消息区分）
- [ ] 代码块语法高亮显示
- [ ] 文件选择器列表组件正常工作
- [ ] 命令面板可正常打开和搜索
- [ ] 进度指示器正确显示
- [ ] 工具详情面板可展开/收起

### 4.3 配置验收

- [ ] 配置文件 `mycode.json` 中的 TUI 配置生效
- [ ] 滚动加速功能按预期工作
- [ ] 滚动速度设置按预期工作
- [ ] 主题切换功能正常

### 4.4 集成验收

- [ ] `/editor` 命令可调用外部编辑器
- [ ] `/export` 命令可导出对话为 Markdown
- [ ] 环境变量 EDITOR 配置正确生效

### 4.5 性能验收

- [ ] TUI 启动时间满足目标（< 300ms）
- [ ] 滚动流畅，无明显卡顿
- [ ] 长时间使用无内存泄漏
- [ ] 二进制大小满足目标（< 10MB）

### 4.6 用户体验验收

- [ ] 命令面板可正常访问和自定义
- [ ] 用户名显示设置可切换和记忆
- [ ] 主题切换功能正常
- [ ] 打字机效果流畅自然

---

## 5. 竞品分析与借鉴

### 5.1 参考项目：rusty-claude-cli

[ruby-claude-cli](https://github.com/anomalyco/opentui) 是 Rust 生态中的一个成熟 TUI 实现，可以作为很好的参考。

**项目特点：**
- 使用 Ratatui 作为 TUI 渲染库
- 完整的 CLI 参数解析系统
- 完善的斜杠命令实现
- 会话管理（Session / Resume）
- OAuth 登录流程
- 权限模式系统
- 流式输出处理

### 5.2 可借鉴的功能特性

| 特性 | 描述 | 优先级 |
|------|------|--------|
| **CLI 参数系统** | 支持 `--model`, `--permission-mode`, `--output-format` 等参数 | P0 |
| **权限模式** | `read-only`, `workspace-write`, `danger-full-access` 三级权限控制 | P0 |
| **输出格式** | 支持 Text, JSON, NDJSON 三种输出格式 | P1 |
| **会话恢复** | 支持 `--resume` 恢复会话，继续之前的对话 | P0 |
| **登录/登出** | OAuth 登录流程，凭证管理 | P1 |
| **版本信息** | 显示版本号、构建目标、Git SHA | P1 |
| **状态报告** | 详细的会话状态、Token 使用量统计 | P1 |
| **Cost 统计** | 显示输入/输出 Token 数量、缓存使用情况 | P1 |
| **模型别名** | 支持 `opus`, `sonnet`, `haiku` 等模型别名 | P1 |
| **插件系统** | Plugin Manager, Plugin Registry | P2 |
| **记忆系统** | Memory 文件管理 | P2 |
| **Git 集成** | Diff 视图、Commit 消息生成 | P2 |

### 5.3 命令行参数设计

参考 rusty-claude-cli 的设计，mycode TUI 应支持以下 CLI 参数：

```bash
# 基础使用
mycode                          # 启动交互式 TUI
mycode /path/to/project        # 指定项目目录

# 模型选择
mycode --model claude-opus-4-6 # 指定模型
mycode -m opus                 # 使用模型别名

# 权限控制
mycode --permission-mode read-only        # 只读模式
mycode --permission-mode workspace-write  # 工作区写权限
mycode --dangerously-skip-permissions     # 跳过权限检查

# 输出格式
mycode --output-format text   # 文本输出
mycode --output-format json   # JSON 输出

# 会话管理
mycode --resume <session-path> # 恢复会话

# 工具控制
mycode --allowed-tools Bash,Edit,Read  # 允许特定工具

# 其他
mycode --version              # 显示版本
mycode --help                 # 显示帮助
```

### 5.4 斜杠命令扩展

参考 rusty-claude-cli 的命令实现，扩展 mycode TUI 的斜杠命令：

| 命令 | 描述 | 状态 |
|------|------|------|
| `/help` | 显示帮助 | 已有 |
| `/status` | 显示会话状态 | 已有 |
| `/compact` | 压缩会话 | 已有 |
| `/model` | 切换模型 | 已有 |
| `/permissions` | 切换权限模式 | **新增** |
| `/session` | 会话管理（列表/切换/导出）| **新增** |
| `/cost` | 显示 Token 使用量 | **新增** |
| `/resume` | 恢复会话 | **新增** |
| `/clear` | 清空会话 | 已有 |
| `/export` | 导出会话 | 已有 |
| `/init` | 初始化项目 | 已有 |
| `/diff` | 显示 Git 差异 | **新增** |
| `/version` | 显示版本信息 | **新增** |
| `/config` | 查看配置 | **新增** |
| `/memory` | 记忆文件管理 | **新增** |
| `/plugins` | 插件管理 | **新增** |

### 5.5 启动 Banner 设计

参考 rusty-claude-cli 的 ASCII Art Banner 设计：

```
███╗   ███╗██╗   ██╗ ██████╗ ██████╗ ██████╗ ███████╗
████╗ ████║╚██╗ ██╔╝██╔════╝██╔═══██╗██╔══██╗██╔════╝
██╔████╔██║ ╚████╔╝ ██║     ██║   ██║██║  ██║█████╗  
██║╚██╔╝██║  ╚██╔╝  ██║     ██║   ██║██║  ██║██╔══╝  
██║ ╚═╝ ██║   ██║   ╚██████╗╚██████╔╝██████╔╝███████╗
╚═╝     ╚═╝   ╚═╝    ╚═════╝ ╚═════╝ ╚═════╝ ╚══════╝

  Model            claude-opus-4-6
  Permissions      danger-full-access
  Directory        /path/to/project
  Session          abc123

  Type /help for commands · Shift+Enter for newline
```

### 5.6 会话状态显示

```
Model
  Current model    claude-opus-4-6
  Session messages 42
  Session turns    15

Usage
  Input tokens     125,000
  Output tokens   45,000
  Cache create    30,000
  Cache read      80,000
  Total tokens    250,000

Permissions
  Active mode      danger-full-access
  Mode status      live session default

Modes
  read-only        ○ available   Read/search tools only
  workspace-write  ○ available   Edit files inside the workspace
  danger-full-access ● current   Unrestricted tool access
```

### 5.7 权限模式系统

参考 rusty-claude-cli 实现三级权限模式：

| 模式 | 描述 | 工具可用性 |
|------|------|-----------|
| `read-only` | 只读模式 | 仅搜索、读取工具 |
| `workspace-write` | 工作区写权限 | 编辑文件工具 |
| `danger-full-access` | 完全访问 | 所有工具（包括 Shell） |

### 5.8 TUI 增强计划（详细）

基于 rusty-claude-cli 的 TUI 增强计划，mycode TUI 应分阶段实现以下增强：

#### 阶段 0：结构清理（基础）

| 任务 | 描述 | 工作量 |
|------|------|--------|
| 0.1 | **拆分 monolith** - 将 `main.rs` 中的 `LiveCli` 提取到 `app.rs`，分离 `format.rs`（报告格式化）、`session_mgr.rs`（会话 CRUD） | M |
| 0.2 | **移除重复代码** - 清理 `app.rs` 中的遗留 `CliApp`，或合并其独特功能 | S |
| 0.3 | **统一参数解析** - 整合手写解析器和 clap-based 解析器，采用更完整的版本 | S |
| 0.4 | **创建 tui/ 模块** - 引入 `src/tui/mod.rs` 作为 TUI 组件命名空间：`status_bar.rs`, `layout.rs`, `tool_panel.rs` 等 | S |

#### 阶段 1：状态栏与实时 HUD

| 任务 | 描述 | 工作量 |
|------|------|--------|
| 1.1 | **终端尺寸感知的状态行** - 使用 `crossterm::terminal::size()` 渲染底部状态栏，显示：模型名称、权限模式、会话 ID、累计 Token 计数、预估费用 | M |
| 1.2 | **实时 Token 计数器** - 在流式传输过程中根据 `Usage` 事件实时更新状态栏 | M |
| 1.3 | **Turn 计时器** - 显示当前 Turn 的耗时 | S |
| 1.4 | **Git 分支指示器** - 在状态栏显示当前 Git 分支 | S |

#### 阶段 2：增强流式输出

| 任务 | 描述 | 工作量 |
|------|------|--------|
| 2.1 | **实时 Markdown 渲染** - 不再只是原始文本流式输出，而是增量渲染 Markdown（标题检测、粗体/斜体、内联代码） | L |
| 2.2 | **思考指示器** - 当模型处于思考/推理模式时，显示独特的动画指示器（如 `🧠 Reasoning...`） | S |
| 2.3 | **流式进度条** - 在 Spinner 下方添加可选的水平进度指示器（基于 max_tokens vs. output_tokens） | M |
| 2.4 | **移除人工延迟** - 当前 `stream_markdown` 每块睡眠 8ms，主响应流应即时或可配置 | S |

#### 阶段 3：工具调用可视化

| 任务 | 描述 | 工作量 |
|------|------|--------|
| 3.1 | **可折叠的工具输出** - 对于长于 N 行的工具结果（可配置，默认 15 行），显示摘要和 `[+] Expand` 提示 | M |
| 3.2 | **语法高亮的工具结果** - 当工具结果包含代码时（根据工具名检测），使用 syntect 高亮 | M |
| 3.3 | **工具调用时间线** - 多工具 Turn，显示紧凑摘要：`🔧 bash → ✓ | read_file → ✓ | edit_file → ✓ (3 tools, 1.2s)` | S |
| 3.4 | **Diff 感知的 edit_file 显示** - 当 `edit_file` 成功时，显示带颜色的统一 diff 而不是仅 `✓ edit_file: path` | M |
| 3.5 | **权限提示增强** - 使用 Box Drawing 样式化批准提示，用颜色标注工具名称，显示工具将执行的操作的一行摘要 | S |

#### 阶段 4：增强斜杠命令与导航

| 任务 | 描述 | 工作量 |
|------|------|--------|
| 4.1 | **着色的 /diff 输出** - 解析 git diff 并用红/绿着色渲染删除/新增，类似 `delta` 或 `diff-so-fancy` | M |
| 4.2 | **长输出分页器** - 当 `/status`, `/config`, `/memory`, `/diff` 产生长于终端高度的输出时，通过内部分页器（j/k/q 滚动）或外部 `$PAGER` | M |
| 4.3 | **`/search` 命令** - 添加按关键字搜索对话历史的新命令 | M |
| 4.4 | **`/undo` 命令** - 通过从 `write_file`/`edit_file` 工具结果中的 `originalFile` 数据恢复来撤销最后文件编辑 | M |
| 4.5 | **交互式会话选择器** - 将文本式的 `/session list` 替换为可交互的模糊可过滤列表（上下箭头选择，回车切换）| L |
| 4.6 | **工具参数补全** - 扩展 `SlashCommandHelper` 以在 `/export` 后补全文件路径，在 `/model` 后补全模型名称 | M |

#### 阶段 5：颜色主题与配置

| 任务 | 描述 | 工作量 |
|------|------|--------|
| 5.1 | **命名颜色主题** - 添加 `dark`（当前默认）、`light`、`solarized`、`catppuccin` 主题。连接到现有 Config 工具的 `theme` 设置 | M |
| 5.2 | **ANSI-256 / 真彩色检测** - 检测终端能力并优雅降级（无颜色 → 16 色 → 256 → 真彩色）| M |
| 5.3 | **可配置 Spinner 样式** - 允许在点状、条状、月相等之间选择 | S |
| 5.4 | **Banner 自定义** - 通过设置使 ASCII 艺术 Banner 可选或可配置 | S |

#### 阶段 6：全屏 TUI 模式（扩展）

| 任务 | 描述 | 工作量 |
|------|------|--------|
| 6.1 | **添加 ratatui 依赖** - 引入 `ratatui`（终端 UI 框架）作为全屏模式的可选依赖 | S |
| 6.2 | **分屏布局** - 顶部窗格：带回滚的对话；底部窗格：输入区域；右侧边栏（可选）：工具状态/待办列表 | XL |
| 6.3 | **可滚动对话视图** - 使用 PgUp/PgDn 浏览过往消息，在对话中搜索 | L |
| 6.4 | **键盘快捷键面板** - 显示 `?` 帮助覆盖层，显示所有键绑定 | M |
| 6.5 | **鼠标支持** - 点击展开工具结果，滚动对话，选择文本复制 | L |

### 5.9 优先级建议

#### 立即（高影响，中等工作量）

1. **阶段 0** - 基础清理。3,159 行的 `main.rs` 是 #1 维护风险
2. **阶段 1.1–1.2** - 带实时 Token 的状态栏。最高影响的 UX 改进
3. **阶段 2.4** - 移除人工延迟。低工作量，立即可感知的改进
4. **阶段 3.1** - 可折叠的工具输出。大型 bash 输出目前严重影响可读性

#### 近期（下一个 Sprint）

5. **阶段 2.1** - 实时 Markdown 渲染。核心交互体验更流畅
6. **阶段 3.2** - 语法高亮的工具结果
7. **阶段 3.4** - Diff 感知的 edit 显示
8. **阶段 4.1** - `/diff` 的着色输出

#### 长期

9. **阶段 5** - 颜色主题（用户需求驱动）
10. **阶段 4.2–4.6** - 增强导航和命令
11. **阶段 6** - 全屏模式（主要工作，在早期阶段发布后评估）

### 5.10 架构建议

#### Phase 0 后的模块结构

```
src/
├── main.rs              # 入口，参数分发 (~100 lines)
├── args.rs              # CLI 参数解析
├── app.rs               # LiveCli 结构，REPL 循环，Turn 执行
├── format.rs            # 所有报告格式化 (status, cost, model, permissions)
├── session_mgr.rs       # 会话 CRUD: 创建、恢复、列表、切换、持久化
├── init.rs              # 仓库初始化
├── input.rs             # 行编辑器
├── render.rs            # TerminalRenderer, Spinner (扩展)
└── tui/
    ├── mod.rs           # TUI 模块根
    ├── status_bar.rs    # 持久底部状态行
    ├── tool_panel.rs    # 工具调用可视化 (boxes, timelines, collapsible)
    ├── diff_view.rs    # 着色 diff 渲染
    ├── pager.rs        # 长输出的内部分页器
    └── theme.rs        # 颜色主题定义和选择
```

#### 关键设计原则

1. **保持内联 REPL 为默认** - 全屏 TUI 应为可选（`--tui` 标志）
2. **所有内容可无终端测试** - 所有格式化函数接受 `&mut impl Write`，不直接假定 stdout
3. **流式优先** - 渲染应该增量工作，而不是缓冲整个响应
4. **所有终端控制使用 crossterm** - 不要混合原始 ANSI 转义序列
5. **特性门控重依赖** - `ratatui` 应在 `full-tui` 特性标志后面

### 5.11 风险评估

| 风险 | 缓解措施 |
|------|----------|
| 在重构期间破坏工作的 REPL | 阶段 0 是纯结构重组，现有测试覆盖率作为安全网 |
| 终端兼容性问题 (tmux, SSH, Windows) | 依赖 crossterm 的抽象；在降级环境中测试 |
| 丰富渲染的性能回归 | 前后性能分析；始终保留快速路径（原始流式传输）|
| 阶段 6 的范围蔓延 | 将阶段 0–3 作为连贯版本发布后再开始阶段 6 |
| `app.rs` vs `main.rs` 混淆 | 阶段 0.2 通过移除遗留 `CliApp` 明确解决 |

---

## 7. 后续规划

### 7.1 短期规划

- 增加更多快捷键自定义选项
- 优化滚动性能和体验
- 增加更多主题支持
- 改进文件选择器交互
- 增加鼠标支持
- 实现 CLI 参数系统
- 实现权限模式系统

### 7.2 中期规划

- 插件系统支持
- 高级自定义脚本
- 团队协作功能增强
- 性能优化（启动时间、内存占用）
- 语法高亮增强（Tree-sitter 集成）
- 实现 OAuth 登录流程

### 7.3 长期规划

- 跨平台 TUI 定制开发指南
- Rust TUI 生态集成
- 社区主题和插件市场

---

## 8. 参考文档

### 8.1 mycode 文档

- [mycode TUI 官方文档](https://mycode.ai/docs/zh-cn/tui/)
- [快捷键文档](/docs/zh-cn/keybinds)
- [分享功能文档](/docs/zh-cn/share)
- [AGENTS.md 规则文档](/docs/zh-cn/rules)

### 8.2 Rust TUI 生态

- [Ratatui GitHub](https://github.com/ratatui/ratatui)
- [Ratatui 官方文档](https://ratatui.rs)
- [Crossterm 文档](https://github.com/crossterm-rs/crossterm)
- [Tokio 文档](https://tokio.rs)

### 8.3 竞品项目

- [rusty-claude-cli](https://github.com/claw-code/rusty-claude-cli)
- [rusty-claude-cli 源码](https://github.com/claw-code/rust/tree/main/crates/rusty-claude-cli)

### 8.4 技术栈

- [Rust 语言](https://www.rust-lang.org/)
- [Ratatui 库](https://crates.io/crates/ratatui)
- [Crossterm](https://crates.io/crates/crossterm)
- [Tokio](https://crates.io/crates/tokio)

---

## 9. 附录

### 7.1 Ratatui 核心特性

根据 Ratatui 官方文档，其核心特性包括：

1. **丰富组件**：Paragraph, List, Table, Tabs, Sparkline, Gauge, ProgressBar 等
2. **布局系统**：Block, Flex, Grid, Split, Stack 等
3. **事件处理**：键盘、鼠标、窗口大小调整、焦点等
4. **主题系统**：内置多种主题，支持自定义
5. **样式系统**：全面的文本样式（颜色、粗体、斜体、下划线等）
6. **异步支持**：与 Tokio 良好集成

### 7.2 命令面板快捷键

| 快捷键 | 功能 |
|--------|------|
| `ctrl+x h` | 打开帮助/命令面板 |
| `ctrl+x c` | 压缩会话 |
| `ctrl+x d` | 切换详情显示 |
| `ctrl+x e` | 打开编辑器 |
| `ctrl+x q` | 退出 |
| `ctrl+x x` | 导出 |
| `ctrl+x i` | 初始化 |
| `ctrl+x m` | 模型列表 |
| `ctrl+x n` | 新建会话 |
| `ctrl+x r` | 重做 |
| `ctrl+x l` | 会话列表 |
| `ctrl+x s` | 分享 |
| `ctrl+x t` | 主题 |
| `ctrl+x u` | 撤销 |
| `ctrl+t` | 切换模型变体 |

### 7.3 Rust TUI 库对比

| 库 | 星标 | 下载量 | 特点 |
|----|------|--------|------|
| **Ratatui** | 19401 | 3000万+ | 最成熟，功能全面 |
| **r3bl_tui** | - | 5906/月 | Rust 2024，活跃开发 |
| **Iocraft** | - | - | React-like API |
| **feather_tui** | - | - | 轻量级 |
| **MinUI** | - | - | 游戏导向，轻量 |

### 7.4 迁移说明

从 Zig/TypeScript 版本迁移到 Rust 版本的主要变更：

| 方面 | 原版本 (OpenTUI) | 新版本 (Rust/Ratatui) |
|------|------------------|----------------------|
| 核心语言 | Zig | Rust |
| 绑定语言 | TypeScript | 原生 Rust |
| TUI 库 | @opentui/core | Ratatui |
| 终端层 | 自定义 | Crossterm |
| 异步 | Node.js | Tokio |
| 编译 | Bun/Node | Cargo |
| 二进制 | 多文件 | 单文件 |
