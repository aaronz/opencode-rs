# OpenCode-RS 实施计划 v15 — TUI 完整实现

**版本**: 15.0
**日期**: 2026-04-07
**基于**: spec_v15.md (FR-201 ~ FR-240) + constitution_updates.md (v2.0) + 代码库审计
**状态**: 已发布

---

## 1. 实施策略

### 1.1 优先级分层

| 阶段 | 优先级 | 需求范围 | 目标 |
|------|--------|----------|------|
| **阶段 1** | P0 核心 | FR-201/202/204/205/206(P0)/207(P0)/208/209/212(P0)/213/220(P0)/221(P0)/222/224/240(P0) | TUI 可启动、可交互、可通信 |
| **阶段 2** | P1 增强 | FR-202.4/203/206(P1)/207(P1)/209(P1)/210/211(P1)/214/215/216/218(P1)/219/223/234/235/236/237/238/239 | 完整用户体验 |
| **阶段 3** | P2 高级 | FR-206(P2)/207(P2部分)/209(P2)/211(P2部分)/212(P2)/213(P2)/215(P2部分)/216(P2部分)/217/218(P2部分)/223(P2部分)/225/228/229(P2部分)/230(P2部分)/231/232/233 | 高级功能与优化 |

### 1.2 实施原则

1. **P0 优先**: 确保核心功能可用，每个阶段结束时可构建可运行版本
2. **增量交付**: 每个阶段完成后进行 clippy + 编译验证
3. **复用现有**: 最大化利用已有代码（app.rs、command.rs、theme.rs、layout.rs 等）
4. **匹配模式**: 新代码严格遵循现有 Rust 代码风格（错误处理、模块组织、测试结构）

---

## 2. 当前实现状态审计

### 2.1 已实现模块（55 个 Rust 文件）

| 模块 | 文件 | 实现度 | 备注 |
|------|------|--------|------|
| 应用核心 | `app.rs` | ~60% | 事件循环、状态机、16 种 AppMode、11 种 TuiState 完整 |
| 命令系统 | `command.rs` | ~50% | 25 个命令已注册，但缺少 /redo, /thinking, /status, /permissions, /share, /unshare, /diff, /search, /init, /editor |
| 组件系统 | `components.rs` + `components/` | ~55% | StatusBar, TitleBar, FileTree, TerminalPanel, SkillsPanel, RightPanel, VirtualList, InputWidget, DiffView 存在 |
| 布局系统 | `layout.rs` | ~80% | 响应式布局、4 种预设、最小尺寸检测已完成 |
| 主题系统 | `theme.rs` | ~70% | 7 个主题预设、ThemeManager、真彩色检测已完成，缺少 solarized |
| 输入处理 | `input/` | ~65% | InputBox, InputParser, InputProcessor, InputHistory, Completer, Editor 存在 |
| 文件引用 | `file_ref_handler.rs` | ~40% | @ 语法基础存在，缺少模糊搜索集成 |
| Shell执行 | `shell_handler.rs` | ~40% | ! 语法基础存在，缺少超时/危险命令检测 |
| 会话管理 | `session.rs` | ~35% | 内存会话 + 文件持久化，**缺少 SQLite** |
| 对话框 | `dialogs/` | ~60% | SlashCommand, Settings, ModelSelection, ProviderManagement, Connect*, FileSelection, DirectorySelection, ReleaseNotes, DiffReview |
| 右侧面板 | `right_panel.rs` | ~50% | 基础面板存在 |
| Patch预览 | `patch_preview.rs` | ~50% | Diff 预览基础存在 |
| Spinner | `widgets/spinner.rs` | ~70% | 动画帧、状态指示存在，缺少主题适配 |
| 命令面板 | `widgets/command_palette.rs` | ~30% | 数据结构存在，**render 为空实现** |
| 消息气泡 | `widgets/message_bubble.rs` | ~60% | 基础气泡存在 |
| 代码块 | `widgets/code_block.rs` | ~50% | 基础代码块存在 |
| 滚动条 | `widgets/scrollbar.rs` | ~60% | 基础滚动条存在 |
| Markdown渲染 | `render/markdown.rs` | ~50% | pulldown-cmark 解析存在，**缺少增量渲染** |
| 语法高亮 | `render/syntax_highlight.rs` | ~50% | syntect 基础存在 |
| 输入历史 | `input/history.rs` | ~70% | VecDeque 实现完整，**缺少文件持久化** |
| CLI参数 | `cli/args.rs` | ~75% | clap 参数定义完整，缺少 --help 自定义 |
| 配置文件 | `config.rs` | ~40% | 基础配置存在 |
| Banner | `components/banner.rs` | ~30% | 基础组件存在 |

### 2.2 ratatui 版本

- **当前**: ratatui 0.30 ✅ (PRD 要求 0.30)
- syntect 5 ✅
- pulldown-cmark 0.13 ✅
- fuzzy-matcher 0.3 ✅
- ansi-to-tui 8.0 ✅

### 2.3 关键差距总结

| 差距类别 | 严重程度 | 涉及 FR |
|----------|----------|---------|
| SQLite 会话持久化 | 🔴 高 | FR-208.2 |
| 命令面板 render 为空 | 🔴 高 | FR-224 |
| 斜杠命令缺失 10+ 个 | 🔴 高 | FR-206.4/11/13/14/15/16/17/18/19/20/21 |
| 快捷键 Ctrl+X 体系缺失 | 🔴 高 | FR-207 (leader key 存在但绑定不完整) |
| 会话管理无 SQLite | 🔴 高 | FR-208 |
| 增量 Markdown 渲染缺失 | 🟡 中 | FR-215.1 |
| 思考块 UI 缺失 | 🟡 中 | FR-234 |
| 用户名显示设置缺失 | 🟡 中 | FR-235 |
| 滚动配置系统缺失 | 🟡 中 | FR-236 |
| Undo/Redo Git 集成缺失 | 🟡 中 | FR-237 |
| 导出/分享功能不完整 | 🟡 中 | FR-238/239 |
| 模型别名缺失 | 🟡 中 | FR-228 |
| Cost 统计缺失 | 🟡 中 | FR-229 |
| TUI-Server WebSocket 接口缺失 | 🔴 高 | FR-240 |
| Banner 规范不完整 | 🟡 中 | FR-226 |
| 状态面板缺失 | 🟡 中 | FR-227 |
| 输入历史无持久化 | 🟡 中 | FR-223.1 |
| 编辑器集成不完整 | 🟡 中 | FR-219 |

---

## 3. 实施计划

### 阶段 1: P0 核心功能 (Week 1-2)

**目标**: TUI 可启动、核心交互可用、Server 通信正常

#### 1.1 CLI 与启动 (FR-201, FR-212 P0, FR-220 P0)
- [ ] 完善 CLI 参数验证与错误提示
- [ ] 目录不存在友好提示（已有基础，需增强）
- [ ] 配置文件 mycode.json 的 TUI 配置段解析
- [ ] Banner 显示（ASCII Art + 状态信息）

#### 1.2 消息系统 (FR-202 P0, FR-221 P0, FR-222)
- [ ] 消息气泡完善（用户/AI 视觉区分）
- [ ] Markdown 基础渲染（标题/粗体/斜体/列表）
- [ ] Spinner 动画完善（主题适配）
- [ ] 消息滚动自动跟随

#### 1.3 文件引用与 Shell 执行 (FR-204 P0, FR-205 P0)
- [ ] @ 语法模糊文件搜索集成（fuzzy-matcher + TUI List）
- [ ] 文件内容加载到对话上下文
- [ ] ! 语法命令执行与结果集成
- [ ] 安全约束：工作目录限制、.git/ 排除

#### 1.4 斜杠命令核心 (FR-206 P0)
- [ ] 补齐缺失的 P0 命令：/connect, /compact, /details, /exit, /help, /models, /new, /sessions
- [ ] 命令面板 render 实现（当前为空）
- [ ] 命令面板模糊搜索

#### 1.5 快捷键核心 (FR-207 P0)
- [ ] Ctrl+X leader key 体系完善
- [ ] P0 快捷键绑定：ctrl+x c/d/q/h/m/n/l, Enter, Shift+Enter, PgUp/PgDn
- [ ] 输入历史 Up/Down 导航

#### 1.6 会话管理 (FR-208 P0)
- [ ] **SQLite 集成**（替换当前文件持久化）
- [ ] 会话创建/持久化/恢复
- [ ] 会话列表展示与切换

#### 1.7 权限与布局 (FR-209 P0, FR-213 P0)
- [ ] 布局系统完善（主布局：对话区 + 输入区）
- [ ] 三级权限模式集成
- [ ] 终端尺寸自适应（已有基础）

#### 1.8 TUI-Server 接口 (FR-240 P0)
- [ ] WebSocket 连接管理
- [ ] 消息格式统一（请求/响应/流式）
- [ ] 错误码统一处理
- [ ] 会话状态同步
- [ ] 工具调用结果回传

### 阶段 2: P1 增强体验 (Week 3-4)

**目标**: 完整用户体验，主题、流式、编辑器集成

#### 2.1 流式与 Markdown (FR-202.4, FR-215, FR-203)
- [ ] 增量 Markdown 渲染
- [ ] 打字机效果（无人工延迟）
- [ ] 思考指示器动画
- [ ] 代码块语法高亮（syntect）
- [ ] 代码块语言标签显示

#### 2.2 主题与状态栏 (FR-210, FR-214, FR-218 P1)
- [ ] 浅色主题完善
- [ ] 主题切换命令与快捷键
- [ ] 底部状态栏（模型/权限/Token/分支）
- [ ] 实时 Token 更新

#### 2.3 工具调用可视化 (FR-216)
- [ ] 可折叠工具输出
- [ ] 语法高亮工具结果
- [ ] 工具调用时间线

#### 2.4 编辑器与输入 (FR-219, FR-223, FR-235, FR-236)
- [ ] 外部编辑器集成（EDITOR 环境变量）
- [ ] 输入历史持久化
- [ ] 用户名显示设置
- [ ] 滚动配置系统

#### 2.5 会话增强 (FR-237, FR-238, FR-239)
- [ ] Undo/Redo Git 集成
- [ ] 导出为 Markdown + 编辑器打开
- [ ] 分享功能（链接生成、脱敏）

#### 2.6 新增 FR-226 ~ FR-234 P1
- [ ] FR-226: Banner 规范实现
- [ ] FR-227: 会话状态面板
- [ ] FR-228: 模型别名
- [ ] FR-229: Cost 统计
- [ ] FR-234: 思考模式 UI 控制

### 阶段 3: P2 高级功能 (Week 5-6)

**目标**: 高级功能、性能优化、质量达标

#### 3.1 高级导航 (FR-217)
- [ ] 着色 diff 输出
- [ ] 长输出分页器
- [ ] /search 搜索对话历史
- [ ] 交互式会话选择器
- [ ] 工具参数补全

#### 3.2 高级主题 (FR-218 P2)
- [ ] Solarized 主题
- [ ] ANSI-256 降级
- [ ] Spinner 样式配置
- [ ] Banner 自定义

#### 3.3 高级功能 (FR-230 P2, FR-231, FR-232, FR-233)
- [ ] 版本信息（构建目标、Git SHA）
- [ ] Git diff 视图
- [ ] 记忆系统 UI
- [ ] 插件系统 UI

#### 3.4 性能与质量 (FR-225)
- [ ] 启动时间优化（< 300ms）
- [ ] 滚动帧率优化（>= 60fps）
- [ ] 内存占用优化
- [ ] clippy 警告清零
- [ ] 文档覆盖率 >= 80%
- [ ] 单元测试覆盖率 >= 70%

---

## 4. 技术决策

### 4.1 SQLite vs 文件持久化

**决策**: 采用 SQLite 替换当前文件持久化（FR-208.2）

**理由**:
- Constitution C-028 要求 SQLite
- 支持复杂查询（会话搜索、排序、过滤）
- 事务安全，避免数据损坏
- 已有 `storage` crate 可作为依赖

**实施**:
- 复用 `opencode-storage` crate（已有 database.rs, models.rs, service.rs）
- SessionManager 改为 SQLite 后端
- 保留文件持久化作为降级方案

### 4.2 WebSocket 通信

**决策**: TUI 通过 WebSocket 与 Server 通信（FR-240）

**理由**:
- Constitution C-056 要求统一接口契约
- 流式输出天然适合 WebSocket
- 断线重连机制

**实施**:
- 使用 `tokio-tungstenite` 或 `async-tungstenite`
- 消息格式 JSON-RPC 风格
- 心跳保活 30s

### 4.3 Leader Key 体系

**决策**: 完善现有 Leader Key 机制（已有基础）

**理由**:
- `app.rs` 已有 `LeaderKeyState`, `activate_leader_key()`, `check_leader_key_timeout()`
- 超时 2s 已配置
- 只需补充 Ctrl+X 后的按键映射

---

## 5. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| SQLite 集成复杂度高 | 阶段 1 延期 | 复用已有 storage crate，最小化 schema |
| WebSocket 与现有 LLM 提供者冲突 | 通信层混乱 | 抽象统一消息层，WebSocket 作为传输层之一 |
| 增量 Markdown 渲染性能 | 帧率下降 | 异步渲染，主循环不阻塞 |
| 大量 P2 功能挤压 | 质量不达标 | 严格优先级，P2 可延期 |

---

## 6. 验收标准

### 6.1 阶段 1 验收
- [ ] `mycode` 可正常启动 TUI
- [ ] 核心斜杠命令可执行
- [ ] 核心快捷键绑定正常
- [ ] 消息气泡正确渲染
- [ ] 会话创建/持久化/恢复正常
- [ ] WebSocket 连接与消息格式正确

### 6.2 阶段 2 验收
- [ ] 代码块语法高亮
- [ ] 主题切换正常
- [ ] 状态栏显示完整
- [ ] 编辑器集成正确
- [ ] 导出/分享功能正常

### 6.3 阶段 3 验收
- [ ] 启动时间 < 300ms
- [ ] 滚动流畅 >= 60fps
- [ ] clippy 警告: 0
- [ ] 测试覆盖率 >= 70%

---

## 7. 追溯链

```
spec_v15.md (FR-201 ~ FR-240)
    │
    ├── 40 项功能需求
    ├── 14 项 P0 + 24 项 P1 + 13 项 P2
    ├── Constitution v2.0 (C-011 ~ C-056)
    │
    ▼
plan_v15.md (本文档) — 3 阶段实施计划
    │
    ├── 阶段 1: P0 核心 (Week 1-2)
    ├── 阶段 2: P1 增强 (Week 3-4)
    └── 阶段 3: P2 高级 (Week 5-6)
    │
    ▼
tasks_v15.md — 原子任务清单
```

---

**文档状态**: 已发布
**下一步**: 创建 tasks_v15.md 原子任务清单
