下面是可直接替换进原文档的 **“7.15 TUI 设计”细化版**。

---

## 7.15 TUI 设计

公开资料表明 TUI 是当前 OpenCode 的默认使用方式。([opencode.ai](https://opencode.ai/docs/cli/))

### 7.15.1 TUI 设计目标

TUI 不是“终端聊天框”，而是 Runtime 的可视化控制台。它的目标是：

1. **让用户始终知道系统在做什么**
   需要清晰展示：思考中、调用工具、等待授权、编辑文件、运行测试、生成 diff、总结结果。

2. **让用户随时能接管流程**
   用户必须可以中断、拒绝权限、查看改动、切换 agent/model、回滚当前轮修改。

3. **让长任务可跟踪**
   多文件修改、测试修复、LSP 诊断、MCP 查询结果必须有明确的信息层级，而不是混在普通聊天文本里。

4. **让 TUI 成为 Runtime 的标准前端之一**
   TUI 不应内嵌大量独立业务逻辑，而应通过统一的 Session/Server 协议驱动。

---

### 7.15.2 设计原则

#### 原则 A：消息流与执行流分离

普通对话、工具调用、Shell 输出、权限请求、Diff 结果必须分别展示。

#### 原则 B：任务优先于对话

用户真正关心的是：

* 正在改哪些文件
* 当前跑了什么命令
* 哪一步失败了
* 还剩哪些待办

不是单纯看“assistant 回复了一大段文字”。

#### 原则 C：输入不阻塞观察

即使模型在流式输出，用户仍应能：

* 滚动查看历史
* 打开 diff
* 查看 diagnostics
* 处理权限请求

#### 原则 D：高风险操作必须显眼

权限确认不能埋在文本里，必须是明确的交互节点。

#### 原则 E：终端环境必须可降级

在窄终端、低色彩、SSH、Windows Terminal 场景中仍然可用。

---

### 7.15.3 布局设计

#### 默认双栏布局

适用于 100~160 列终端：

* 左栏：Session 列表 / 项目状态
* 右栏：主消息流 + 输入区

#### 三栏布局

适用于 >= 160 列终端：

* 左栏：Session / Project
* 中栏：Timeline
* 右栏：Inspector（Todo / Diff / Diagnostics / Permissions）

#### 窄屏布局

适用于 < 100 列终端：

* 单主栏
* 侧边面板改为 Tab 弹出式
* 保证输入区、权限区始终可达

---

### 7.15.4 视觉区域划分

#### Header 顶部状态栏

显示：

* 当前项目名
* Git 分支
* 当前 session 名称
* 当前 agent
* 当前 model
* token / cost 摘要
* server 状态
* LSP 状态
* MCP 状态

#### Sidebar 左侧导航区

显示：

* session 列表
* 每个 session 的标题
* 最近更新时间
* 活跃状态
* 最近文件 / 最近命令

#### Timeline 主时间线

显示：

* 用户消息
* assistant 消息
* tool 调用块
* shell 输出块
* permission 块
* diff 块
* summary 块
* error 块

#### Inspector 右侧辅助区

以标签页切换：

* Todo
* Diff
* Diagnostics
* Context
* Permissions
* Files

#### Composer 底部输入区

显示：

* 当前输入内容
* 自动补全
* 模式提示（普通 / @ / / / !）
* 发送提示

#### Footer 底部状态区

显示：

* 快捷键提示
* 最近错误摘要
* toast / 系统通知

---

### 7.15.5 信息架构

TUI 中最重要的对象分为 6 类：

1. Session
2. Message
3. Tool Call
4. Permission Request
5. Artifact（Diff / Patch / Summary）
6. Runtime State

#### 映射关系

* Session → 左栏
* Message → 主时间线
* Tool Call → 时间线中的结构化块
* Permission Request → 高亮块 + 右栏队列
* Artifact → 时间线摘要 + 右栏详情
* Runtime State → Header / Footer / Spinner

#### 用户关注优先级

界面刷新优先顺序：

1. 正在等待用户决定的内容
2. 正在执行的工具
3. 当前轮新增的改动
4. 当前失败的测试 / diagnostics
5. 历史对话

---

### 7.15.6 时间线设计

时间线不是纯文本流，而是“强类型消息流”。

#### Block 类型

1. `UserMessageBlock`
2. `AssistantTextBlock`
3. `ToolCallBlock`
4. `ToolResultBlock`
5. `ShellOutputBlock`
6. `PermissionBlock`
7. `DiffBlock`
8. `SummaryBlock`
9. `ErrorBlock`
10. `SystemNoticeBlock`

#### AssistantTextBlock

要求：

* 支持流式增量渲染
* 支持基础 markdown
* 代码块可折叠
* 长文本自动收起后续段落
* 不展示模型私有推理，只展示可见输出状态

#### ToolCallBlock

显示字段：

* 工具名
* 参数摘要
* 开始时间
* 当前状态（queued/running/done/error）
* 执行耗时
* 可展开查看原始参数和结果

#### ShellOutputBlock

要求：

* stdout/stderr 区分
* 超过阈值自动折叠
* 能跳到最后一行
* 保留退出码
* 支持独立滚动，不拖慢主时间线

#### DiffBlock

默认显示：

* 本轮修改文件数
* 新增/删除行数
* 主要改动摘要

展开后显示：

* 文件级 diff
* hunk 导航
* 重命名/删除/新增标记

---

### 7.15.7 交互状态机

TUI 必须有明确状态机。

#### 顶层状态

```text id="4gu24v"
idle
composing
submitting
streaming
executing_tool
awaiting_permission
showing_diff
showing_error
aborting
reconnecting
```

#### 状态说明

* `idle`：等待用户输入
* `composing`：正在编辑输入
* `submitting`：消息已提交，等待 server 确认
* `streaming`：模型正在输出
* `executing_tool`：工具执行中
* `awaiting_permission`：等待用户审批
* `showing_diff`：当前聚焦 patch/diff
* `showing_error`：当前有错误需要确认
* `aborting`：终止当前轮
* `reconnecting`：TUI 与 server 重连中

#### 关键事件

* send_prompt
* stream_started
* tool_started
* permission_requested
* permission_resolved
* diff_ready
* task_finished
* abort_requested
* server_disconnected
* reconnect_success

---

### 7.15.8 输入区设计

输入区是高频操作核心。

#### 输入模式

1. 普通 prompt
2. `@` 文件引用模式
3. `/` 命令模式
4. `!` shell 模式
5. 多行输入模式

#### 输入行为

* Enter：发送
* Shift+Enter：换行
* 上下键：输入历史
* 支持粘贴长文本
* 支持混合输入，如：
  `修复 @src/main.rs 的报错，然后执行 !cargo test`

#### 自动补全

##### `@` 补全

显示：

* 文件路径
* 命中方式（exact / recent / fuzzy）
* 文件大小提示
* 是否已在本轮上下文中

##### `/` 补全

显示：

* 命令名
* 简介
* 来源（builtin / project / global）
* 绑定的 agent / model

##### `!` 补全

显示：

* 最近命令
* 最近成功命令优先

#### 提交前解析

输入提交后应先转为结构化对象：

```json id="nkwx8i"
{
  "raw": "修复 @src/main.rs 中的问题并执行 !cargo test",
  "mentions": ["src/main.rs"],
  "commands": [],
  "shell_inline": ["cargo test"],
  "mode": "prompt"
}
```

---

### 7.15.9 权限交互设计

权限是 TUI 中最重要的风险控制节点。

#### 展示位置

* 时间线插入高亮权限卡片
* 右栏 `Permissions` 面板同步出现
* Header 显示 pending count

#### 权限卡片内容

* 工具名
* 风险等级
* 参数摘要
* 影响范围
* 来源（模型 / command / plugin / MCP）

#### 用户动作

* Allow once
* Allow for session
* Allow for project
* Deny
* Deny and remember
* Inspect details

#### 高风险操作二次确认

以下操作需要更强提示：

* 删除文件
* 执行 `rm`, `git reset --hard`, `curl | sh`
* 读取 `.env`, `id_rsa`, `credentials`
* 远程 MCP 写操作

#### 键盘操作建议

* `a`：allow once
* `s`：allow session
* `p`：allow project
* `d`：deny
* `i`：inspect

---

### 7.15.10 Diff / Patch 查看设计

#### 目标

让用户在终端中也能准确判断这次改动是否合理。

#### 三层视图

##### 摘要视图

显示：

* 修改文件数
* 新增/删除行数
* 变更类型概览

##### 文件列表视图

显示：

* 文件路径
* 变更类型
* 风险标记（迁移/配置/删除/测试）

##### 单文件视图

显示：

* unified diff
* hunk 导航
* 折叠上下文
* 跳转原文件位置

#### 支持操作

* 接受当前改动
* 导出 patch
* 回滚本轮改动
* 复制摘要

#### 快捷键建议

* `g d`：打开 diff
* `j/k`：切换 hunk
* `Enter`：展开/折叠
* `u`：回滚本轮
* `y`：复制摘要

---

### 7.15.11 Todo 面板

长任务必须有任务跟踪。

#### 数据来源

* 模型显式写入 todo
* 系统从计划中自动提取
* 用户手动编辑

#### Todo 状态

```text id="cq80vv"
pending
in_progress
blocked
done
dropped
```

#### 每项显示

* 标题
* 状态
* 关联文件
* 最近更新时间
* 所属轮次

#### 作用

* 给用户提供进度感
* 给模型提供中间目标
* 降低长会话失控风险

---

### 7.15.12 Diagnostics 面板

聚焦 LSP 和测试失败。

#### 数据源

* LSP diagnostics
* cargo / npm / pytest 等测试输出摘要
* lint 结果

#### 展示方式

按级别分组：

* error
* warning
* info

#### 每条记录显示

* 文件
* 行号
* 规则/错误码
* 摘要
* 是否已在当前轮修复

#### 操作

* Enter：跳转文件或 diff
* `/`：过滤
* `r`：刷新

---

### 7.15.13 Files 与 Context 面板

#### Files 面板

显示：

* 最近访问文件
* 当前轮修改文件
* 用户 pin 的上下文文件
* 推荐相关文件

#### Context 面板

显示：

* 本轮送入模型的上下文对象
* token 预算占比
* summary / skills / MCP / LSP 占用情况

#### 价值

提升系统可解释性，方便用户判断“模型为什么会这么做”。

---

### 7.15.14 快捷键设计

#### 全局

* `q`：退出
* `?`：帮助
* `Ctrl+c`：中断当前生成/工具执行
* `Ctrl+l`：清屏
* `Tab`：切换主面板
* `Shift+Tab`：反向切换

#### Session

* `g s`：聚焦 session 列表
* `n`：新建 session
* `f`：fork session
* `r`：重载当前 session

#### 输入

* `i`：聚焦输入框
* `Esc`：退出输入框
* `Ctrl+r`：搜索历史输入
* `Ctrl+u`：清空当前输入

#### 面板跳转

* `g t`：todo
* `g p`：permissions
* `g x`：context
* `g l`：diagnostics
* `g d`：diff
* `g f`：files

#### 视图控制

* `z`：折叠/展开当前块
* `[` / `]`：跳到上/下一个重要事件
* `Home/End`：顶部/底部

---

### 7.15.15 组件树设计

建议组件拆分如下：

```text id="2rk54t"
App
 ├─ Header
 ├─ Sidebar
 │   ├─ ProjectCard
 │   ├─ SessionList
 │   └─ SessionFilter
 ├─ Timeline
 │   ├─ MessageBlock
 │   ├─ ToolBlock
 │   ├─ PermissionBlock
 │   ├─ DiffBlock
 │   └─ ErrorBlock
 ├─ Inspector
 │   ├─ TodoPanel
 │   ├─ DiffPanel
 │   ├─ DiagnosticsPanel
 │   ├─ ContextPanel
 │   ├─ PermissionsPanel
 │   └─ FilesPanel
 ├─ Composer
 │   ├─ InputEditor
 │   ├─ SuggestionMenu
 │   └─ ModeIndicator
 └─ Footer
```

#### 组件原则

* 组件只负责渲染和局部交互
* 业务状态集中在统一 Store
* 网络事件统一从 Runtime Event Bus 注入

---

### 7.15.16 状态管理

建议使用集中状态管理。

#### 状态分层

##### 持久状态

* 当前 session id
* 布局模式
* inspector 当前 tab

##### 短期状态

* 输入框内容
* 自动补全候选
* 当前选中 block
* 当前待审批权限

##### 派生状态

* 是否可发送
* 是否可中断
* 是否存在未处理权限
* 是否有 diff 可查看

#### Store 示例

```rust id="xqvda4"
pub struct AppState {
    pub runtime: RuntimeState,
    pub ui: UiState,
    pub sessions: SessionState,
    pub timeline: TimelineState,
    pub inspector: InspectorState,
    pub composer: ComposerState,
}
```

---

### 7.15.17 事件驱动与流式渲染

TUI 必须采用事件驱动，而不是轮询刷新整屏。

#### 事件类型

```text id="eipq7f"
ServerConnected
ServerDisconnected
SessionLoaded
MessageAppended
MessagePatched
ToolStarted
ToolUpdated
ToolFinished
PermissionRequested
PermissionResolved
DiffReady
DiagnosticsUpdated
ToastRaised
```

#### 渲染策略

* 事件先写入 Store
* UI 按帧率节流刷新
* streaming 文本用 append/patch，不整块重绘
* shell 大输出单独缓存
* 大 diff 延迟渲染可见区域

#### 性能目标

* 空闲刷新 10~20 FPS
* 流式输出 20~30 FPS 即可
* 优先稳定与低闪烁，不追求花哨动画

---

### 7.15.18 性能与大会话处理

#### 目标

* 10k+ 消息 session 可打开
* 10k+ 行 shell 输出不阻塞输入
* 大 diff 能渐进展示

#### 优化策略

1. Timeline 虚拟滚动
2. 长 block 延迟渲染
3. shell 输出和普通消息分缓存
4. Inspector 按需计算
5. markdown 渲染缓存
6. diff 只渲染可视区

---

### 7.15.19 错误态与恢复态

#### 错误分类

* provider 错误
* tool 错误
* server 断连
* 配置错误
* 插件错误

#### 展示要求

* 错误进入时间线
* Footer 给简洁摘要
* 可展开详情
* 明确是否可重试

#### 恢复能力

* server 重连后恢复订阅
* TUI 重启后恢复最近 session
* 输入草稿自动保存

---

### 7.15.20 可访问性与终端兼容

#### 要求

* 不依赖纯颜色传达信息
* 高亮必须伴随标签/图标
* 全部操作可键盘完成
* 支持 256 色与 truecolor 降级
* 窄屏时关键交互不可丢失

#### 标签建议

* `[ok]`
* `[run]`
* `[warn]`
* `[err]`
* `[ask]`

---

### 7.15.21 TUI 与 Server 的关系

TUI 应是 **Server-first client**。

#### 原则

* TUI 可以 attach 到已有 runtime server
* Session 状态以 server 为准
* 前端只做乐观更新，不做最终裁决

#### 好处

* 后续 Web / Desktop / IDE 复用同一协议
* 降低多前端状态不一致风险
* 测试边界清晰

---

### 7.15.22 测试策略

#### 单元测试

* 输入解析
* 状态机切换
* reducer/store
* 快捷键映射

#### 组件测试

* Timeline block 渲染
* 权限卡片渲染
* Diff 面板导航

#### 集成测试

* 模拟 server 流事件
* 长会话滚动
* 权限处理
* 断线重连

#### 快照测试

* 窄屏/宽屏
* Unicode 文件名
* 大 diff
* shell 错误输出

---

### 7.15.23 Rust 实现建议

#### 建议技术栈

* `ratatui`：渲染
* `crossterm`：终端控制与输入事件
* `tokio`：异步任务
* `tokio::sync::mpsc`：事件流
* `serde`：协议反序列化

#### 推荐任务划分

1. UI 渲染主循环
2. server 事件接收任务
3. 用户输入事件采集任务
4. 后台缓存/索引任务

#### 避免事项

* 不要在 render 阶段做 IO
* 不要让每个组件直接连 server
* 不要把大日志一次性拼成大字符串反复重绘

---

### 7.15.24 MVP 与迭代范围

#### TUI MVP 必做

* Header / Sidebar / Timeline / Composer / Footer
* 基础消息流
* 工具执行块
* 权限卡片
* Diff 查看
* Session 切换
* `@` 与 `/` 基础补全

#### v0.2 增强

* Todo panel
* Diagnostics panel
* Context panel
* Attach existing server
* 输入历史搜索

#### v0.3 增强

* 鼠标支持
* 多会话分屏
* 内联 patch 接受/拒绝
* 主题系统
* 更强日志过滤

---

如果你愿意，我下一步可以继续把这一节再下钻成 **“TUI 页面线框图 + 状态流转图 + Rust 模块接口定义”**。
