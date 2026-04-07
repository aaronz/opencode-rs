# TUI (终端用户界面) 产品需求文档

**版本：** 1.4 (基于 OpenTUI)  
**日期：** 2026年4月7日  
**作者：** mycode 产品团队  
**状态：** 已发布

---

## 1. 概述

### 1.1 产品定位

mycode TUI（Terminal User Interface，终端用户界面）是 mycode 提供的一个交互式终端界面，旨在配合大语言模型（LLM）处理项目代码。TUI 为开发者提供了一种在终端环境中与 AI 进行协作的高效方式，无需切换到 Web 界面即可完成代码分析、修改和协作任务。

TUI 基于 **OpenTUI** 核心库构建，OpenTUI 是一个用 Zig 编写的原生终端 UI 核心，具有高性能、跨语言支持（C ABI）和组件化架构等特点。

### 1.2 核心价值

- **终端原生体验**：为习惯命令行工作的开发者提供无缝的 AI 协作体验
- **文件智能引用**：通过 `@` 语法快速引用项目文件
- **Shell 命令集成**：支持直接执行 Bash 命令并获取输出
- **会话管理**：提供完整的会话历史、撤销/重做功能
- **协作分享**：支持会话分享，方便团队协作和代码审查
- **高性能渲染**：基于 OpenTUI 原生 Zig 核心，享受流畅的终端渲染体验

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
│                    OpenTUI 渲染层                         │
│  (@opentui/core - Zig + TypeScript)                     │
│  - 组件系统 (Button, Input, List, Modal, etc.)          │
│  - 布局引擎 (Flex, Grid, Stack)                         │
│  - 事件系统 (键盘、鼠标、滚动)                           │
│  - 主题系统                                             │
├─────────────────────────────────────────────────────────┤
│                    终端适配层                            │
│  (ANSI/VT100 渲染、Tty 控制、输入捕获)                   │
└─────────────────────────────────────────────────────────┘
```

---

## 2. OpenTUI 架构详解

### 2.1 包结构

OpenTUI 采用 Monorepo 结构，包含以下核心包：

| 包名 | 版本 | 描述 |
|------|------|------|
| `@opentui/core` | 0.1.97 | 原生 Zig 核心 + TypeScript 绑定 |
| `@opentui/react` | 0.1.97 | React 19+ 渲染器 |
| `@opentui/solid` | 0.1.97 | SolidJS 1.9+ 渲染器 |

### 2.2 核心模块结构

#### 2.2.1 @opentui/core

```
@opentui/core
├── src/
│   ├── index.ts           # 主入口
│   ├── 3d.ts             # 3D 渲染支持
│   ├── testing.ts        # 测试工具
│   ├── runtime-plugin.ts # 运行时插件
│   ├── zig/              # Zig 原生核心
│   │   ├── src/          # Zig 源代码
│   │   ├── build.zig     # 构建配置
│   │   └── bench.zig     # 基准测试
│   ├── lib/              # 库文件
│   │   └── tree-sitter/  # 语法高亮
│   └── examples/         # 示例
└── package.json
```

#### 2.2.2 @opentui/react

```
@opentui/react
├── src/
│   ├── index.ts           # 主入口
│   ├── test-utils.ts      # 测试工具
│   ├── jsx-runtime.ts    # JSX 运行时
│   └── examples/         # 示例
├── jsx-runtime.js         # JSX 运行时实现
└── package.json
```

#### 2.2.3 @opentui/solid

```
@opentui/solid
├── index.ts               # 主入口
├── jsx-runtime.d.ts       # JSX 类型定义
├── scripts/
│   ├── preload.ts         # Bun 预加载
│   ├── solid-plugin.ts   # Bun.build 插件
│   └── runtime-plugin-support.ts
└── package.json
```

### 2.3 依赖配置

#### 2.3.1 @opentui/core 依赖

| 依赖 | 用途 |
|------|------|
| yoga-layout | Flexbox 布局引擎 |
| web-tree-sitter | 语法高亮解析器 |
| diff | 差异对比 |
| jimp | 图像处理（3D 支持）|
| three | 3D 渲染支持 |

**平台特定依赖：**
- `@opentui/core-darwin-x64` / `@opentui/core-darwin-arm64` (macOS)
- `@opentui/core-linux-x64` / `@opentui/core-linux-arm64` (Linux)
- `@opentui/core-win32-x64` / `@opentui/core-win32-arm64` (Windows)

**运行要求：** Bun >= 1.3.0

#### 2.3.2 @opentui/react 依赖

- React >= 19.0.0
- react-reconciler ^0.32.0
- react-devtools-core ^7.0.1 (可选，用于调试)
- ws ^8.18.0 (可选)

#### 2.3.3 @opentui/solid 依赖

- SolidJS 1.9.11
- babel-preset-solid 1.9.10

### 2.4 快速开始

```bash
# React 模板
bun create tui --template react

# SolidJS 模板
bun create tui --template solid
```

---

## 3. 功能需求

### 3.1 启动与基础交互

| 功能点 | 描述 | 优先级 |
|--------|------|--------|
| 启动 TUI | 运行 `mycode` 启动当前目录的 TUI | P0 |
| 指定目录启动 | 运行 `mycode /path/to/project` 启动指定目录的 TUI | P0 |
| 消息输入提示 | 进入 TUI 后可输入消息进行提示 | P0 |
| AI 响应渲染 | 使用 OpenTUI 组件渲染 AI 响应（消息气泡、代码块等） | P0 |
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

#### 2.4.1 撤销/重做功能说明

- `/undo` 和 `/redo` 使用 Git 来管理文件更改
- 撤销时会移除最近的用户消息、所有后续响应以及所有文件更改
- 使用此功能需要项目是一个 Git 仓库

#### 2.4.2 思考模式说明

- `/thinking` 命令仅控制思考块是否**显示**，不会启用或禁用模型的推理能力
- 要切换实际的推理能力，请使用 `ctrl+t` 循环切换模型变体

### 3.5 组件与交互设计

基于 OpenTUI 的组件系统，TUI 实现以下核心 UI 组件：

#### 3.5.1 布局与显示组件

| 组件 | 描述 | OpenTUI 基础 | JSX 标签 |
|------|------|--------------|----------|
| 消息气泡 | 显示用户和 AI 的对话消息 | TextRenderable | `<text>` |
| 容器 | 带边框的布局容器 | BoxRenderable | `<box>` |
| 滚动容器 | 可滚动的内容区域 | ScrollboxRenderable | `<scrollbox>` |
| ASCII 字形 | ASCII 艺术字渲染 | AsciiFontRenderable | `<ascii-font>` |

#### 3.5.2 输入组件

| 组件 | 描述 | OpenTUI 基础 | JSX 标签 |
|------|------|--------------|----------|
| 输入框 | 单行文本输入 | InputRenderable | `<input>` |
| 文本域 | 多行文本输入 | TextareaRenderable | `<textarea>` |
| 下拉选择 | 下拉选择列表 | SelectRenderable | `<select>` |
| Tab 选择 | Tab 切换选择 | TabSelectRenderable | `<tab-select>` |

#### 3.5.3 代码与差异组件

| 组件 | 描述 | OpenTUI 基础 | JSX 标签 |
|------|------|--------------|----------|
| 代码块 | 语法高亮代码 | CodeRenderable | `<code>` |
| 行号代码 | 带行号的代码，含差异高亮 | LineNumberRenderable | `<line-number>` |
| 差异视图 | 统一/分屏差异查看器 | DiffRenderable | `<diff>` |

#### 3.5.4 文本修饰符

| 组件 | 描述 |
|------|------|
| `<span>` | 内联样式文本 |
| `<strong>` / `<b>` | 粗体 |
| `<em>` / `<i>` | 斜体 |
| `<u>` | 下划线 |
| `<br>` | 换行 |

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
- 使用 OpenTUI 主题系统实现
- 支持自定义主题扩展

---

## 4. 技术实现

### 4.1 OpenTUI 核心依赖

| 包名 | 描述 | 版本要求 |
|------|------|----------|
| `@opentui/core` | OpenTUI TypeScript 核心绑定 | >= 0.1.0 |
| `@opentui/react` | React reconciler (如使用 React) | >= 0.1.0 |
| `@opentui/solid` | SolidJS reconciler (如使用 SolidJS) | >= 0.1.0 |

### 4.2 架构设计

- **底层渲染**：基于 OpenTUI 原生 Zig 核心，提供高性能终端渲染
- **组件层**：使用 OpenTUI 提供的组件系统构建 UI
- **状态管理**：React/Solid 状态管理（根据技术选型）
- **会话管理**：基于 Git 实现文件变更追踪和撤销/重做
- **配置文件**：支持 JSON 格式的配置文件
- **环境集成**：支持 EDITOR 环境变量配置

### 4.3 依赖要求

- **运行时**：
  - Zig 编译器（用于构建 OpenTUI 核心）
  - Node.js/Bun（用于运行 mycode）
  - Git（用于撤销/重做功能）
  
- **终端要求**：
  - 支持 ANSI/VT100 的终端模拟器
  - 建议使用现代终端（iTerm2, Windows Terminal, Alacritty 等）

- **可选依赖**：
  - 外部编辑器（用于 `/editor` 和 `/export` 命令）

### 4.4 性能目标

| 指标 | 目标值 |
|------|--------|
| 启动时间 | < 500ms |
| 消息渲染延迟 | < 50ms |
| 滚动帧率 | >= 60fps |
| 内存占用（空闲） | < 50MB |
| 内存占用（大量消息） | < 200MB |

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

- [ ] TUI 启动时间满足目标（< 500ms）
- [ ] 滚动流畅，无明显卡顿
- [ ] 长时间使用无内存泄漏

### 4.6 用户体验验收

- [ ] 命令面板可正常访问和自定义
- [ ] 用户名显示设置可切换和记忆
- [ ] 主题切换功能正常
- [ ] 打字机效果流畅自然

---

## 5. 竞品分析与行业参考

### 5.1 竞品项目调研

#### 5.1.1 rusty-claude-cli

该项目是 Rust 生态中成熟的 TUI 实现，采用 Ratatui 作为渲染库，其核心架构值得参考：

| 模块 | 职责 | 可借鉴程度 |
|------|------|-----------|
| `args.rs` | CLI 参数解析 | ★★★★★ |
| `app.rs` | 应用状态管理 | ★★★★☆ |
| `render.rs` | 终端渲染、着色器 | ★★★★★ |
| `input.rs` | 输入处理、行编辑器 | ★★★★☆ |
| `init.rs` | 项目初始化 | ★★★☆☆ |

**关键实现细节：**

1. **Spinner 动画** (render.rs)
```rust
const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
// 使用 crossterm 的 queue! 进行非阻塞渲染
```

2. **ColorTheme 系统** (render.rs)
```rust
pub struct ColorTheme {
    heading: Color,
    emphasis: Color,
    strong: Color,
    inline_code: Color,
    // ...
}
```

3. **Markdown 流式渲染** (render.rs)
```rust
use pulldown_cmark::{Parser, Event, Tag, TagEnd};
use syntect::easy::HighlightLines;
// 支持 50+ 语言语法高亮
```

4. **权限系统** (args.rs)
```rust
pub enum PermissionMode {
    ReadOnly,
    WorkspaceWrite,
    DangerFullAccess,
}
```

#### 5.1.2 其他竞品参考

| 项目 | 特点 | 参考点 |
|------|------|--------|
| `aider` | AI 驱动编辑 | 流式输出、undo/redo |
| `ollama run` | 本地 LLM | 模型切换、下载管理 |
| `cursor` | AI IDE | TUI 模式、分享功能 |

### 5.2 行业最佳实践

#### 5.2.1 TUI 渲染模式

| 模式 | 描述 | 适用场景 |
|------|------|---------|
| 块渲染 | 整块更新 | 静态内容 |
| 流式渲染 | 逐字符输出 | AI 响应、日志 |
| 增量渲染 | 部分更新 | 长列表、表格 |

#### 5.2.2 交互模式

| 模式 | 描述 | 优点 |
|------|------|------|
| Line Editor | 行输入 | 简单、兼容性好 |
| Vi Mode | Vi 风格 | 高效、专业 |
| Emacs Mode | Emacs 风格 | 快捷键丰富 |

#### 5.2.3 状态管理

```rust
// 会话状态
pub struct SessionState {
    turns: usize,
    compacted_messages: usize,
    last_model: String,
    last_usage: UsageSummary,
}

// 应用状态
pub struct AppState {
    session: SessionState,
    config: AppConfig,
    renderer: TerminalRenderer,
    conversation: Vec<ConversationMessage>,
}
```

### 5.3 可借鉴的功能特性

| 特性 | 描述 | 实现复杂度 | 优先级 |
|------|------|-----------|--------|
| CLI 参数系统 | 完整的命令行参数支持 | 中 | P0 |
| 权限模式 | 三级权限控制 | 低 | P0 |
| 流式输出 | 打字机效果 | 中 | P0 |
| 语法高亮 | Syntect 集成 | 低 | P1 |
| 会话恢复 | --resume 支持 | 中 | P1 |
| Markdown 渲染 | 完整的 MD 解析 | 中 | P1 |
| Token 统计 | 使用量跟踪 | 低 | P2 |
| 模型别名 | opus/sonnet/haiku | 低 | P2 |

---

## 6. 后续规划

### 6.1 短期规划

- 增加更多快捷键自定义选项
- 优化滚动性能和体验
- 增加更多主题支持
- 改进文件选择器交互
- 增加鼠标支持
- 实现 CLI 参数系统
- 实现权限模式系统
- 实现 Spinner 动画
- 实现 ColorTheme 主题系统

### 6.2 中期规划

- 插件系统支持
- 高级自定义脚本
- 团队协作功能增强
- 性能优化（启动时间、内存占用）
- Syntect 语法高亮集成
- Markdown 流式渲染

### 6.3 长期规划

- 跨平台 TUI 定制开发指南
- OpenTUI 生态集成
- 社区主题和插件市场

---

## 7. 参考文档

### 7.1 mycode 文档

- [mycode TUI 官方文档](https://mycode.ai/docs/zh-cn/tui/)
- [快捷键文档](/docs/zh-cn/keybinds)
- [分享功能文档](/docs/zh-cn/share)
- [AGENTS.md 规则文档](/docs/zh-cn/rules)

### 7.2 OpenTUI 项目

- [OpenTUI GitHub 仓库](https://github.com/anomalyco/opentui)
- [OpenTUI 官方文档](https://opentui.com/docs/getting-started)
- [OpenTUI 示例](https://github.com/anomalyco/opentui/tree/main/packages/core/src/examples)
- [awesome-opentui 资源列表](https://github.com/msmps/awesome-opentui)

### 7.3 竞品项目

- [rusty-claude-cli](https://github.com/claw-code/rusty-claude-cli)
- [aider](https://github.com/paul-gauthier/aider)
- [ollama](https://github.com/ollama/ollama)

### 7.4 技术栈

- [Zig 语言](https://ziglang.org/)
- [TypeScript](https://www.typescriptlang.org/)
- [Bun 运行时](https://bun.sh/)

---

## 8. TUI 增强计划（分阶段）

基于竞品分析，mycode TUI 应按以下阶段实现增强：

### 阶段 0：结构清理（基础）

| 任务 | 描述 | 工作量 |
|------|------|--------|
| 0.1 | **拆分 monolith** - 将核心逻辑拆分为独立模块 | M |
| 0.2 | **移除重复代码** - 清理遗留实现 | S |
| 0.3 | **统一参数解析** - 整合 CLI 参数系统 | S |
| 0.4 | **创建 UI 模块** - 规范化组件结构 | S |

### 阶段 1：状态栏与实时 HUD

| 任务 | 描述 | 工作量 |
|------|------|--------|
| 1.1 | **终端尺寸感知的状态行** - 显示模型、权限、Token 计数 | M |
| 1.2 | **实时 Token 计数器** - 流式传输中实时更新 | M |
| 1.3 | **Turn 计时器** - 显示当前 Turn 耗时 | S |
| 1.4 | **Git 分支指示器** - 显示当前分支 | S |

### 阶段 2：增强流式输出

| 任务 | 描述 | 工作量 |
|------|------|--------|
| 2.1 | **实时 Markdown 渲染** - 增量渲染 Markdown | L |
| 2.2 | **思考指示器** - 独特的推理模式动画 | S |
| 2.3 | **流式进度条** - 基于 Token 的进度指示 | M |
| 2.4 | **移除人工延迟** - 即时流式输出 | S |

### 阶段 3：工具调用可视化

| 任务 | 描述 | 工作量 |
|------|------|--------|
| 3.1 | **可折叠工具输出** - 长输出折叠处理 | M |
| 3.2 | **语法高亮工具结果** - 代码高亮显示 | M |
| 3.3 | **工具调用时间线** - 多工具紧凑摘要 | S |
| 3.4 | **Diff 感知显示** - 带颜色的 diff 输出 | M |
| 3.5 | **权限提示增强** - 样式化批准提示 | S |

### 阶段 4：增强命令与导航

| 任务 | 描述 | 工作量 |
|------|------|--------|
| 4.1 | **着色 /diff** - 红绿 diff 渲染 | M |
| 4.2 | **分页器** - 长输出滚动 | M |
| 4.3 | **/search 命令** - 搜索对话历史 | M |
| 4.4 | **/undo 命令** - 撤销文件编辑 | M |
| 4.5 | **交互式会话选择器** - 模糊过滤列表 | L |
| 4.6 | **工具参数补全** - 文件路径、模型名补全 | M |

### 阶段 5：颜色主题

| 任务 | 描述 | 工作量 |
|------|------|--------|
| 5.1 | **命名颜色主题** - dark/light/solarized/catppuccin | M |
| 5.2 | **终端能力检测** - 优雅降级颜色 | M |
| 5.3 | **可配置 Spinner** - 点状/条状/月相 | S |
| 5.4 | **Banner 自定义** - 可选 ASCII Banner | S |

### 阶段 6：全屏 TUI 模式

| 任务 | 描述 | 工作量 |
|------|------|--------|
| 6.1 | **引入 OpenTUI 全屏** - 可选的全屏模式 | S |
| 6.2 | **分屏布局** - 对话/输入/边栏 | XL |
| 6.3 | **可滚动视图** - PgUp/PgDn 浏览 | L |
| 6.4 | **快捷键面板** - `?` 帮助覆盖 | M |
| 6.5 | **鼠标支持** - 点击/滚动交互 | L |

### 优先级建议

#### 立即（高影响）

1. 阶段 0 - 基础清理
2. 阶段 1.1-1.2 - 状态栏 + 实时 Token
3. 阶段 2.4 - 移除人工延迟
4. 阶段 3.1 - 可折叠工具输出

#### 近期

5. 阶段 2.1 - 实时 Markdown
6. 阶段 3.2 - 语法高亮
7. 阶段 3.4 - Diff 显示
8. 阶段 4.1 - 着色 diff

#### 长期

9. 阶段 5 - 颜色主题
10. 阶段 4.2-4.6 - 增强命令
11. 阶段 6 - 全屏模式

### 架构建议

```
src/
├── main.rs           # 入口
├── app.rs            # 应用核心
├── format.rs         # 格式化
├── session_mgr.rs    # 会话管理
├── ui/               # UI 组件
│   ├── status_bar.rs # 状态栏
│   ├── tool_panel.rs # 工具面板
│   ├── diff_view.rs  # Diff 视图
│   ├── pager.rs      # 分页器
│   └── theme.rs      # 主题
```

### 设计原则

1. 内联 REPL 为默认，全屏模式可选
2. 所有内容可无终端测试
3. 流式优先渲染
4. 使用 OpenTUI 进行终端控制
5. 重依赖特性门控

---

## 9. 附录

### 9.1 OpenTUI 核心特性

根据 OpenTUI 项目文档，其核心特性包括：

1. **原生性能**：Zig 编写的核心，接近 C 的性能
2. **跨语言支持**：C ABI 可从任何语言调用
3. **组件化架构**：Button, Input, List, Modal, Progress, Tabs 等
4. **灵活布局**：Flex, Grid, Stack 等布局系统
5. **事件处理**：键盘、鼠标、滚动等完整事件系统
6. **主题系统**：完整的主题定制能力
7. **可扩展性**：支持自定义组件和扩展

### 9.2 命令面板快捷键

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
