# OpenCode-RS 任务清单 v15 — TUI 完整实现

**版本**: 15.0
**日期**: 2026-04-07
**基于**: plan_v15.md + spec_v15.md
**状态**: 已发布

---

## 阶段 1: P0 核心功能 (Week 1-2)

### T1.1 CLI 与启动完善

| 任务 ID | 描述 | FR | 优先级 | 文件 | 状态 |
|---------|------|-----|--------|------|------|
| T1.1.1 | 完善 CLI 参数验证：路径遍历检查、友好错误提示 | FR-201.2, FR-201.3, FR-212.6 | P0 | `cli/args.rs` | ⬜ |
| T1.1.2 | 配置文件 mycode.json 读取与 TUI 配置段解析 | FR-220.1, FR-220.2 | P0 | `config.rs` | ⬜ |
| T1.1.3 | Banner 组件实现：ASCII Art + 模型/权限/目录/会话 ID/快捷键提示 | FR-226.1 ~ FR-226.6 | P1 | `components/banner.rs` | ⬜ |

### T1.2 消息系统完善

| 任务 ID | 描述 | FR | 优先级 | 文件 | 状态 |
|---------|------|-----|--------|------|------|
| T1.2.1 | 消息气泡视觉区分：用户/AI 背景色/边框/对齐差异 | FR-202.2, FR-202.3 | P0 | `widgets/message_bubble.rs` | ⬜ |
| T1.2.2 | Markdown 基础渲染：标题/粗体/斜体/列表/链接 → ratatui Line/Span | FR-221.1, FR-221.2 | P0 | `render/markdown.rs` | ⬜ |
| T1.2.3 | Spinner 主题适配：颜色跟随当前主题 | FR-222.4 | P1 | `widgets/spinner.rs` | ⬜ |
| T1.2.4 | 消息滚动自动跟随：流式输出时自动滚动到底部 | FR-202.5, FR-211.5 | P0 | `app.rs` (滚动逻辑) | ⬜ |

### T1.3 文件引用 (@ 语法)

| 任务 ID | 描述 | FR | 优先级 | 文件 | 状态 |
|---------|------|-----|--------|------|------|
| T1.3.1 | @ 触发检测集成到 InputProcessor | FR-204.1 | P0 | `input/processor.rs`, `input_parser.rs` | ⬜ |
| T1.3.2 | 模糊文件搜索：ignore crate 遍历 + fuzzy-matcher 匹配 | FR-204.2 | P0 | `file_ref_handler.rs` | ⬜ |
| T1.3.3 | 搜索结果 TUI List 展示与选择 | FR-204.3 | P0 | `widgets/file_selection.rs` | ⬜ |
| T1.3.4 | 选中文件内容加载到对话上下文 | FR-204.4 | P0 | `file_ref_handler.rs` | ⬜ |
| T1.3.5 | 安全约束：.git/ 排除、项目外文件排除、大文件截断、二进制处理 | FR-204 (安全) | P0 | `file_ref_handler.rs` | ⬜ |

### T1.4 Shell 命令执行 (! 语法)

| 任务 ID | 描述 | FR | 优先级 | 文件 | 状态 |
|---------|------|-----|--------|------|------|
| T1.4.1 | ! 前缀检测与命令执行 | FR-205.1 | P0 | `shell_handler.rs`, `input_parser.rs` | ⬜ |
| T1.4.2 | 命令执行结果集成到对话气泡 | FR-205.2 | P0 | `shell_handler.rs` | ⬜ |
| T1.4.3 | 安全约束：工作目录限制、危险命令警告、权限模式约束 | FR-205 (安全) | P0 | `shell_handler.rs` | ⬜ |

### T1.5 斜杠命令补齐

| 任务 ID | 描述 | FR | 优先级 | 文件 | 状态 |
|---------|------|-----|--------|------|------|
| T1.5.1 | 补齐 P0 命令到 CommandRegistry：/connect, /compact, /details, /help, /models, /new, /sessions, /exit(已有) | FR-206.1~206.3, 206.5~206.7, 206.9~206.10, 206.12 | P0 | `command.rs` | ⬜ |
| T1.5.2 | 命令执行逻辑实现：每个命令的实际行为（已有部分 stub） | FR-206 (P0) | P0 | `app.rs` (execute_slash_command) | ⬜ |
| T1.5.3 | /sessions 会话列表对话框与切换 | FR-206.12 | P0 | `dialogs/` (新增 session_list.rs) | ⬜ |

### T1.6 命令面板实现

| 任务 ID | 描述 | FR | 优先级 | 文件 | 状态 |
|---------|------|-----|--------|------|------|
| T1.6.1 | 命令面板 Widget render 实现（当前为空） | FR-224.1, FR-224.2 | P0 | `widgets/command_palette.rs` | ⬜ |
| T1.6.2 | 命令面板模糊搜索/过滤 | FR-224.3 | P1 | `widgets/command_palette.rs` | ⬜ |
| T1.6.3 | 命令面板快捷键提示展示 | FR-224.4 | P1 | `widgets/command_palette.rs` | ⬜ |

### T1.7 快捷键系统 (Ctrl+X Leader Key)

| 任务 ID | 描述 | FR | 优先级 | 文件 | 状态 |
|---------|------|-----|--------|------|------|
| T1.7.1 | Leader Key 按键映射表：ctrl+x → {c,d,q,h,m,n,l} | FR-207.1/2/4/6/8/9/11 | P0 | `app.rs` (handle_input) | ⬜ |
| T1.7.2 | Enter 发送消息 / Shift+Enter 换行 | FR-207.16, FR-207.17 | P0 | `app.rs` (handle_input) | ⬜ |
| T1.7.3 | PgUp/PgDn 对话滚动 | FR-207.19 | P0 | `app.rs` (handle_input) | ⬜ |
| T1.7.4 | Up/Down 输入历史导航 | FR-207.18 | P1 | `app.rs` + `input/history.rs` | ⬜ |

### T1.8 会话管理 (SQLite)

| 任务 ID | 描述 | FR | 优先级 | 文件 | 状态 |
|---------|------|-----|--------|------|------|
| T1.8.1 | 集成 opencode-storage crate，添加 session 表 schema | FR-208.2 | P0 | `storage/src/` (新增 session schema) | ⬜ |
| T1.8.2 | SessionManager 改为 SQLite 后端（替换文件持久化） | FR-208.2 | P0 | `session.rs` | ⬜ |
| T1.8.3 | 会话创建自动保存 | FR-208.1 | P0 | `session.rs` | ⬜ |
| T1.8.4 | 会话恢复（--resume 或 /sessions） | FR-208.3 | P0 | `session.rs`, `cli/args.rs` | ⬜ |
| T1.8.5 | 会话列表 Table 组件展示 | FR-208.4 | P0 | `dialogs/` (新增 session_list.rs) | ⬜ |
| T1.8.6 | 会话间切换（加载历史消息） | FR-208.5 | P0 | `session.rs`, `app.rs` | ⬜ |

### T1.9 权限模式集成

| 任务 ID | 描述 | FR | 优先级 | 文件 | 状态 |
|---------|------|-----|--------|------|------|
| T1.9.1 | 三级权限模式与 opencode-permission crate 集成 | FR-213.1 ~ FR-213.3 | P0 | `permission/`, `app.rs` | ⬜ |
| T1.9.2 | 运行时权限切换 UI | FR-213.4 | P1 | `app.rs` | ⬜ |
| T1.9.3 | 权限状态栏显示 | FR-213.5 | P1 | `components/status_bar.rs` | ⬜ |

### T1.10 布局系统完善

| 任务 ID | 描述 | FR | 优先级 | 文件 | 状态 |
|---------|------|-----|--------|------|------|
| T1.10.1 | 主布局：对话区 + 输入区（已有基础，需完善 draw 逻辑） | FR-209.1 | P0 | `app.rs` (draw), `layout.rs` | ⬜ |
| T1.10.2 | 终端尺寸自适应（已有基础，需集成到 draw 循环） | FR-209.4 | P0 | `app.rs` (draw) | ⬜ |
| T1.10.3 | 最小终端尺寸检测与提示（已有基础） | FR-209.5 | P2 | `app.rs` (run) | ⬜ |

### T1.11 TUI-Server 接口契约

| 任务 ID | 描述 | FR | 优先级 | 文件 | 状态 |
|---------|------|-----|--------|------|------|
| T1.11.1 | WebSocket 连接管理模块 | FR-240.1 | P0 | `tui/src/` (新增 `server_ws.rs`) | ⬜ |
| T1.11.2 | 消息格式统一：请求/响应/流式 JSON 结构 | FR-240.2 | P0 | `tui/src/` (新增 `server_protocol.rs`) | ⬜ |
| T1.11.3 | 错误码统一处理 | FR-240.3 | P0 | `tui/src/` (新增 `server_errors.rs`) | ⬜ |
| T1.11.4 | 会话状态同步（TUI ↔ Server） | FR-240.4 | P0 | `server_ws.rs` | ⬜ |
| T1.11.5 | 工具调用结果回传 | FR-240.6 | P0 | `server_ws.rs` | ⬜ |
| T1.11.6 | Token 使用量实时上报 | FR-240.5 | P1 | `server_ws.rs` | ⬜ |
| T1.11.7 | 断线重连机制 | FR-240.7 | P1 | `server_ws.rs` | ⬜ |
| T1.11.8 | 心跳保活（30s） | FR-240.8 | P1 | `server_ws.rs` | ⬜ |

---

## 阶段 2: P1 增强体验 (Week 3-4)

### T2.1 流式输出与 Markdown 增强

| 任务 ID | 描述 | FR | 优先级 | 文件 | 状态 |
|---------|------|-----|--------|------|------|
| T2.1.1 | 增量 Markdown 渲染（流式中实时更新） | FR-215.1 | P1 | `render/markdown.rs` | ⬜ |
| T2.1.2 | 打字机效果（移除人工延迟，即时流式） | FR-202.4, FR-215.4 | P1 | `app.rs` (check_llm_events) | ⬜ |
| T2.1.3 | 思考指示器动画（🧠 Reasoning...） | FR-215.2, FR-234.4 | P1 | `widgets/` (新增 `thinking_indicator.rs`) | ⬜ |
| T2.1.4 | 代码块语法高亮（syntect 50+ 语言） | FR-203.2 | P1 | `render/syntax_highlight.rs`, `widgets/code_block.rs` | ⬜ |
| T2.1.5 | 代码块语言标签显示 | FR-203.3 | P1 | `widgets/code_block.rs` | ⬜ |
| T2.1.6 | 代码块可滚动（超出宽度时） | FR-203.4 | P0 | `widgets/code_block.rs` | ⬜ |

### T2.2 主题与状态栏

| 任务 ID | 描述 | FR | 优先级 | 文件 | 状态 |
|---------|------|-----|--------|------|------|
| T2.2.1 | 浅色主题完善与切换 | FR-210.2 | P1 | `theme.rs` | ⬜ |
| T2.2.2 | 主题切换命令（/themes + ctrl+x t） | FR-210.4 | P1 | `command.rs`, `app.rs` | ⬜ |
| T2.2.3 | 主题持久化到配置文件 | FR-210.5 | P1 | `theme.rs` (save_to_config 已有基础) | ⬜ |
| T2.2.4 | 底部状态栏：模型/权限/Token/分支 | FR-214.1 ~ FR-214.5 | P1 | `components/status_bar.rs` | ⬜ |
| T2.2.5 | 实时 Token 更新（流式中） | FR-214.7 | P1 | `components/status_bar.rs` | ⬜ |

### T2.3 工具调用可视化

| 任务 ID | 描述 | FR | 优先级 | 文件 | 状态 |
|---------|------|-----|--------|------|------|
| T2.3.1 | 可折叠工具输出（>15 行折叠） | FR-216.1 | P1 | `app.rs` (ToolCall), `components/` | ⬜ |
| T2.3.2 | 语法高亮工具结果 | FR-216.2 | P1 | `render/syntax_highlight.rs` | ⬜ |
| T2.3.3 | 工具调用时间线（紧凑摘要） | FR-216.3 | P2 | `components/` (新增 `tool_timeline.rs`) | ⬜ |

### T2.4 编辑器与输入增强

| 任务 ID | 描述 | FR | 优先级 | 文件 | 状态 |
|---------|------|-----|--------|------|------|
| T2.4.1 | 外部编辑器集成：EDITOR 环境变量、--wait 参数、内容回填 | FR-219.1 ~ FR-219.4 | P1 | `input/editor.rs` | ⬜ |
| T2.4.2 | 编辑器安全约束：工作目录限制、超时、临时文件清理 | FR-219 (安全) | P1 | `input/editor.rs` | ⬜ |
| T2.4.3 | 输入历史持久化到文件 | FR-223.1 | P1 | `input/history.rs` | ⬜ |
| T2.4.4 | 历史记录安全：不包含敏感信息、文件权限 600 | FR-223 (安全) | P1 | `input/history.rs` | ⬜ |
| T2.4.5 | 用户名显示设置（切换/保存/记忆） | FR-235.1 ~ FR-235.3 | P1 | `config.rs`, `app.rs` | ⬜ |
| T2.4.6 | 滚动配置系统（scroll_acceleration, scroll_speed） | FR-236.1 ~ FR-236.4 | P1 | `config.rs`, `app.rs` (ScrollState) | ⬜ |

### T2.5 会话增强

| 任务 ID | 描述 | FR | 优先级 | 文件 | 状态 |
|---------|------|-----|--------|------|------|
| T2.5.1 | /undo 使用 Git 回退文件更改 | FR-237.1 | P1 | `command.rs`, `app.rs` | ⬜ |
| T2.5.2 | /redo 重做撤销的更改 | FR-237.2 | P1 | `command.rs`, `app.rs` | ⬜ |
| T2.5.3 | 非 Git 仓库友好提示 | FR-237.3 | P1 | `app.rs` | ⬜ |
| T2.5.4 | 撤销时移除最后用户消息及所有后续响应 | FR-237.4 | P1 | `app.rs` | ⬜ |
| T2.5.5 | /export 导出 Markdown + 默认编辑器打开 | FR-238.1, FR-238.2 | P1 | `app.rs` (execute_command) | ⬜ |
| T2.5.6 | 导出时 API Key 自动脱敏 | FR-238.3 | P1 | `app.rs` | ⬜ |
| T2.5.7 | /share 分享当前会话 + 链接生成 + 脱敏 | FR-239.1, FR-239.3, FR-239.4 | P1 | `command.rs`, `app.rs` | ⬜ |

### T2.6 新增 FR 实现

| 任务 ID | 描述 | FR | 优先级 | 文件 | 状态 |
|---------|------|-----|--------|------|------|
| T2.6.1 | 会话状态面板（/status）：Model/Usage/Permissions 区域 | FR-227.1 ~ FR-227.4 | P1 | `dialogs/` (新增 `status_panel.rs`) | ⬜ |
| T2.6.2 | 模型别名系统（opus/sonnet/haiku 映射） | FR-228.1 ~ FR-228.4 | P1 | `cli/args.rs`, `app.rs` | ⬜ |
| T2.6.3 | Cost 统计显示（Input/Output/Cache tokens + 预估费用） | FR-229.1 ~ FR-229.4 | P1 | `components/status_bar.rs`, `app.rs` | ⬜ |
| T2.6.4 | 思考模式 UI 控制（/thinking + ctrl+t + 视觉区分） | FR-234.1 ~ FR-234.3 | P1 | `app.rs`, `render/markdown.rs` | ⬜ |

---

## 阶段 3: P2 高级功能 (Week 5-6)

### T3.1 高级导航

| 任务 ID | 描述 | FR | 优先级 | 文件 | 状态 |
|---------|------|-----|--------|------|------|
| T3.1.1 | 着色 diff 输出（红绿 diff） | FR-231.1 ~ FR-231.3 | P2 | `components/diff_view.rs` | ⬜ |
| T3.1.2 | 长输出分页器（j/k/q 滚动） | FR-217.2 | P2 | `widgets/` (新增 `pager.rs`) | ⬜ |
| T3.1.3 | /search 搜索对话历史 | FR-217.3 | P2 | `command.rs`, `app.rs` | ⬜ |
| T3.1.4 | 交互式会话选择器（模糊过滤列表） | FR-217.5 | P2 | `dialogs/session_list.rs` | ⬜ |
| T3.1.5 | 工具参数补全（文件路径/模型名） | FR-217.6 | P2 | `input/completer.rs` | ⬜ |

### T3.2 高级主题

| 任务 ID | 描述 | FR | 优先级 | 文件 | 状态 |
|---------|------|-----|--------|------|------|
| T3.2.1 | Solarized 主题 | FR-218.3 | P2 | `theme.rs` | ⬜ |
| T3.2.2 | ANSI-256 / 真彩色检测与降级 | FR-218.5 | P2 | `theme.rs` (supports_truecolor 已有基础) | ⬜ |
| T3.2.3 | Spinner 样式配置 | FR-218.6 | P2 | `widgets/spinner.rs` | ⬜ |
| T3.2.4 | Banner 自定义 | FR-218.7 | P2 | `components/banner.rs` | ⬜ |

### T3.3 高级功能

| 任务 ID | 描述 | FR | 优先级 | 文件 | 状态 |
|---------|------|-----|--------|------|------|
| T3.3.1 | 版本信息（--version, 构建目标, Git SHA） | FR-230.1 ~ FR-230.4 | P2 | `cli/args.rs`, `app.rs` | ⬜ |
| T3.3.2 | Git diff 视图（/diff） | FR-231.1 ~ FR-231.3 | P2 | `command.rs`, `components/diff_view.rs` | ⬜ |
| T3.3.3 | Git 分支指示器（状态栏） | FR-231.4 | P1 | `components/status_bar.rs` | ⬜ |
| T3.3.4 | 记忆系统 UI（/memory） | FR-232.1 ~ FR-232.3 | P2 | `command.rs`, `dialogs/` (新增 `memory.rs`) | ⬜ |
| T3.3.5 | 插件系统 UI（/plugins） | FR-233.1 ~ FR-233.3 | P2 | `command.rs`, `dialogs/` (新增 `plugins.rs`) | ⬜ |

### T3.4 性能与质量

| 任务 ID | 描述 | FR | 优先级 | 文件 | 状态 |
|---------|------|-----|--------|------|------|
| T3.4.1 | 启动时间优化（< 300ms）：懒加载、异步初始化 | FR-225 | P1 | `app.rs` (new), `cli/mod.rs` | ⬜ |
| T3.4.2 | 滚动帧率优化（>= 60fps）：虚拟滚动、渲染优化 | FR-225 | P1 | `app.rs` (draw), `components/virtual_list.rs` | ⬜ |
| T3.4.3 | 内存占用优化（空闲 < 30MB, 大量消息 < 100MB） | FR-225 | P2 | 全局 | ⬜ |
| T3.4.4 | clippy 警告清零 | FR-225 | P1 | 全局 | ⬜ |
| T3.4.5 | 文档覆盖率 >= 80% | FR-225 | P1 | 全局 | ⬜ |
| T3.4.6 | 单元测试覆盖率 >= 70% | FR-225 | P1 | `tests/` | ⬜ |
| T3.4.7 | 二进制大小优化（< 10MB） | FR-225 | P2 | `Cargo.toml` (features) | ⬜ |

---

## 任务统计

### 按优先级

| 优先级 | 任务数 | 占比 |
|--------|--------|------|
| P0 | 38 | 43% |
| P1 | 38 | 43% |
| P2 | 12 | 14% |
| **总计** | **88** | **100%** |

### 按阶段

| 阶段 | 任务数 | 周数 |
|------|--------|------|
| 阶段 1 (P0 核心) | 38 | Week 1-2 |
| 阶段 2 (P1 增强) | 38 | Week 3-4 |
| 阶段 3 (P2 高级) | 12 | Week 5-6 |

### 按模块

| 模块 | 任务数 | 关键文件 |
|------|--------|----------|
| CLI 与启动 | 3 | `cli/args.rs`, `config.rs`, `components/banner.rs` |
| 消息系统 | 4 | `widgets/message_bubble.rs`, `render/markdown.rs` |
| 文件引用 | 5 | `file_ref_handler.rs`, `widgets/file_selection.rs` |
| Shell 执行 | 3 | `shell_handler.rs` |
| 斜杠命令 | 3 | `command.rs`, `app.rs`, `dialogs/` |
| 命令面板 | 3 | `widgets/command_palette.rs` |
| 快捷键 | 4 | `app.rs` |
| 会话管理 | 6 | `session.rs`, `storage/src/`, `dialogs/` |
| 权限模式 | 3 | `permission/`, `app.rs`, `components/status_bar.rs` |
| 布局系统 | 3 | `app.rs`, `layout.rs` |
| TUI-Server | 8 | `server_ws.rs`, `server_protocol.rs`, `server_errors.rs` |
| 流式与 Markdown | 6 | `render/markdown.rs`, `render/syntax_highlight.rs` |
| 主题与状态栏 | 5 | `theme.rs`, `components/status_bar.rs` |
| 工具可视化 | 3 | `app.rs`, `components/` |
| 编辑器与输入 | 6 | `input/editor.rs`, `input/history.rs`, `config.rs` |
| 会话增强 | 7 | `command.rs`, `app.rs` |
| 新增 FR | 4 | 多个 |
| 高级导航 | 5 | `components/diff_view.rs`, `input/completer.rs` |
| 高级主题 | 4 | `theme.rs`, `widgets/spinner.rs` |
| 高级功能 | 5 | 多个 |
| 性能与质量 | 7 | 全局 |

---

## 依赖关系

### 关键路径

```
T1.1 (CLI) → T1.2 (消息) → T1.5 (命令) → T1.7 (快捷键)
                    ↓
T1.3 (文件引用) ← T1.2
T1.4 (Shell) ← T1.2
T1.8 (会话) ← T1.1
T1.11 (Server) → 独立并行
```

### 可并行任务组

| 组 | 任务 | 并行度 |
|----|------|--------|
| G1 | T1.3 (文件引用), T1.4 (Shell), T1.11 (Server) | 3 |
| G2 | T2.1 (流式), T2.2 (主题), T2.4 (编辑器) | 3 |
| G3 | T3.1 (导航), T3.2 (主题), T3.3 (高级) | 3 |

---

## 执行顺序建议

1. **Week 1 Day 1-2**: T1.1 (CLI) → T1.10 (布局) → T1.9 (权限)
2. **Week 1 Day 3-4**: T1.2 (消息) → T1.3 (文件引用) → T1.4 (Shell)
3. **Week 1 Day 5**: T1.5 (命令补齐) → T1.6 (命令面板)
4. **Week 2 Day 1-2**: T1.7 (快捷键) → T1.8 (会话 SQLite)
5. **Week 2 Day 3-4**: T1.11 (TUI-Server 接口)
6. **Week 2 Day 5**: 阶段 1 验收 + clippy 检查

7. **Week 3**: T2.1 (流式) → T2.2 (主题) → T2.3 (工具可视化)
8. **Week 4**: T2.4 (编辑器) → T2.5 (会话增强) → T2.6 (新增 FR)

9. **Week 5**: T3.1 (导航) → T3.2 (主题) → T3.3 (高级功能)
10. **Week 6**: T3.4 (性能与质量) → 最终验收

---

**文档状态**: 已发布
**总任务数**: 88
**P0 任务**: 38 (优先执行)
**下一步**: 开始阶段 1 实施
