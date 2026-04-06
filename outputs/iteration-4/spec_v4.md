# OpenCode-RS 规格文档 v4

**版本**: 4.0
**日期**: 2026-04-04
**基于**: PRD-OpenCode-Configuration.md (v1.0) + Gap Analysis (iteration-3) + PRD vs Rust Gap Analysis (docs/) + Constitution v1.4
**状态**: 已完成

---

## 1. 文档概述

### 1.1 背景

本规格文档基于以下文档综合生成：
- **PRD-OpenCode-Configuration.md**: 配置系统产品需求文档
- **docs/gap-analysis-prd-vs-rust.md**: PRD vs Rust 实现差距分析 (~85-90% 完整度)
- **outputs/iteration-3/gap-analysis.md**: 配置系统专项差距分析 (13 项差距)
- **outputs/iteration-4/constitution_updates.md**: Constitution v1.4 更新 (Server/Compaction/Watcher 配置)
- **spec_v3.md**: 上一版规格文档 (FR-001 ~ FR-043)

### 1.2 目标

- 为每个差距项分配唯一的需求编号 (FR-XXX)
- 按优先级组织需求
- 为每个需求定义验收标准
- 确保新功能有对应的规格定义
- 整合 PRD vs Rust 全局差距分析中的遗漏项

### 1.3 参考文档

| 文档 | 路径 | 说明 |
|------|------|------|
| PRD-配置系统 | `PRD-OpenCode-Configuration.md` | 配置系统产品需求 |
| PRD-主文档 | `docs/PRD.md` | 产品需求文档 v1.1 |
| PRD-Providers | `docs/PRD-providers.md` | Provider 详细规格 |
| PRD-TUI | `docs/PRD-tui.md` | TUI 产品需求 |
| Gap Analysis (PRD vs Rust) | `docs/gap-analysis-prd-vs-rust.md` | 全局实现差距 |
| Gap Analysis (配置专项) | `outputs/iteration-3/gap-analysis.md` | 配置系统差距 |
| Constitution v1.4 | `outputs/iteration-4/constitution_updates.md` | 设计约束条款 |

### 1.4 与 v3 的关系

v4 保留 v3 的所有需求 (FR-001 ~ FR-043)，并新增：
- **FR-044 ~ FR-049**: PRD vs Rust 差距分析中的遗漏项 (session_load/save, OAuth, GitHub, 剩余 skills/commands, HuggingFace/AI21 providers)
- **FR-050 ~ FR-052**: Constitution v1.4 新增配置领域 (Server mDNS, Compaction, Watcher)

---

## 2. 需求总览

| 优先级 | 数量 | 说明 |
|--------|------|------|
| P0 | 12 | 阻断性问题 (v3: 12, v4新增: 0) |
| P1 | 20 | 核心功能缺失 (v3: 16, v4新增: 4) |
| P2 | 20 | 完善性问题 (v3: 15, v4新增: 5) |

---

## 3. P0 - 阻断性问题

> FR-001 ~ FR-010, FR-033, FR-034 继承自 v3，内容不变。

### FR-001: Context Engine

**模块**: core
**严重程度**: P0
**来源**: v2

Context 构建引擎，负责管理对话上下文、token 预算、上下文窗口。

#### 验收标准

- [ ] Context 构建正确包含文件/工具/会话信息
- [ ] Token 预算计算准确
- [ ] 上下文窗口管理符合模型限制

---

### FR-002: Plugin System

**模块**: plugin
**严重程度**: P0
**来源**: v2

插件系统，支持扩展 OpenCode 功能。

#### 验收标准

- [ ] 插件注册/发现机制工作正常
- [ ] 插件生命周期管理 (加载/卸载/重载)
- [ ] 插件间通信机制

---

### FR-003: Skills 系统

**模块**: core/skills
**严重程度**: P0
**来源**: v2

Skills 系统，为 Agent 提供专业能力。

#### 验收标准

- [ ] Skill 注册与发现
- [ ] Skill 加载与执行
- [ ] 内建 Skills + 自定义 Skills 支持

---

### FR-004: Commands 系统

**模块**: core/commands
**严重程度**: P0
**来源**: v2

Commands 系统，支持自定义命令和 TUI 斜杠命令。

#### 验收标准

- [ ] 命令注册与执行
- [ ] TUI 斜杠命令集成
- [ ] 命令模板变量替换

---

### FR-005: MCP 工具接入

**模块**: mcp
**严重程度**: P0
**来源**: v2

MCP (Model Context Protocol) 工具接入。

#### 验收标准

- [ ] MCP 服务器连接管理
- [ ] MCP 工具发现与调用
- [ ] MCP 资源访问

---

### FR-006: TUI 快捷输入解析器

**模块**: server
**严重程度**: P0
**来源**: v2

TUI 输入语法解析器 (@file, !shell, /command)。

#### 验收标准

- [ ] @file 语法解析与文件选择
- [ ] !shell 语法解析与命令预览
- [ ] /command 斜杠命令解析

---

### FR-007: Session Fork

**模块**: server
**严重程度**: P0
**来源**: v2

会话分叉功能，从历史消息点创建新会话。

#### 验收标准

- [ ] POST /sessions/{id}/fork API 实现
- [ ] 父会话关系记录 (parent_session_id)
- [ ] 分叉点消息正确复制

---

### FR-008: 多层配置合并

**模块**: core/config
**严重程度**: P0
**来源**: v2

多层配置加载与合并机制。

#### 验收标准

- [ ] 6 层配置按优先级加载
- [ ] 配置合并逻辑正确 (deep_merge)
- [ ] 合并后配置结构完整

---

### FR-009: .opencode 目录加载

**模块**: core/config
**严重程度**: P0
**来源**: v2

.opencode/ 目录配置自动加载。

#### 验收标准

- [ ] .opencode/config.json 自动加载
- [ ] 子目录 (agents/commands/skills/tools/themes/modes) 扫描
- [ ] 目录内容与主配置正确合并

---

### FR-010: Provider 环境变量约定

**模块**: core/config
**严重程度**: P0
**来源**: v2

Provider 配置中的环境变量引用约定。

#### 验收标准

- [ ] {env:VAR} 在 Provider 配置中正确替换
- [ ] API Key 等敏感信息支持环境变量引用

---

### FR-033: OPENCODE_TUI_CONFIG 环境变量支持

**模块**: core/config
**严重程度**: P0
**来源**: v3 (配置差距分析 Gap #1)

#### 需求描述

实现 `OPENCODE_TUI_CONFIG` 环境变量，允许用户自定义 TUI 配置文件路径。

#### 详细规格

1. **环境变量定义**
   - 变量名: `OPENCODE_TUI_CONFIG`
   - 类型: 文件路径 (绝对路径或 `~` 开头的路径)
   - 优先级: 高于默认 TUI 配置路径

2. **加载逻辑**
   ```
   1. 检查 OPENCODE_TUI_CONFIG 环境变量
   2. 若设置，使用该路径作为 TUI 配置文件
   3. 若未设置，使用默认路径: ~/.config/opencode/tui.json
   4. 若文件不存在，使用内建默认 TUI 配置
   ```

3. **路径展开**
   - 支持 `~` 展开为用户主目录
   - 支持绝对路径

#### 验收标准

- [ ] `OPENCODE_TUI_CONFIG` 环境变量可自定义 TUI 配置路径
- [ ] 路径支持 `~` 展开
- [ ] 未设置时使用默认路径
- [ ] 文件不存在时降级到内建默认

---

### FR-034: TUI 配置分离为独立 tui.json 文件

**模块**: core/config, tui
**严重程度**: P0
**来源**: v3 (配置差距分析 Gap #2)

#### 需求描述

将 TUI 相关配置从主配置 (opencode.json) 分离到独立的 tui.json 文件。

#### 详细规格

1. **独立文件**
   - 文件名: `tui.json` 或 `tui.jsonc`
   - Schema: `$schema: "https://opencode.ai/tui.json"`
   - 默认路径: `~/.config/opencode/tui.json`

2. **TUI 配置项 (应移至 tui.json)**
   - `scroll_speed`: 滚动速度
   - `scroll_acceleration`: 滚动加速
   - `diff_style`: diff 显示风格
   - `theme`: 主题名称
   - `keybinds`: 自定义快捷键绑定

3. **加载优先级**
   ```
   1. OPENCODE_TUI_CONFIG 环境变量指定路径
   2. ~/.config/opencode/tui.json
   3. 项目目录 tui.json
   4. 内建默认 TUI 配置
   ```

#### 验收标准

- [ ] TUI 配置使用独立 tui.json 文件
- [ ] `$schema` 声明正确
- [ ] 主配置中旧 TUI 项发出废弃警告
- [ ] TUI 配置加载优先级正确

---

## 4. P1 - 核心功能缺失

> FR-011 ~ FR-020, FR-032, FR-035 ~ FR-039 继承自 v3，内容不变。以下为 v4 新增 P1 需求。

### FR-011: Server API 完善

**模块**: server
**严重程度**: P1
**来源**: v2

补全 REST API 端点。

#### 验收标准

- [ ] 所有 PRD 定义的 API 端点实现
- [ ] 认证/权限检查
- [ ] 错误响应格式统一

---

### FR-012: Share 功能

**模块**: core
**严重程度**: P1
**来源**: v2

会话分享功能。

#### 验收标准

- [ ] Session.shared_id 字段实现
- [ ] 分享链接生成
- [ ] 分享权限控制

---

### FR-013: LSP 功能增强

**模块**: lsp
**严重程度**: P1
**来源**: v2

LSP 诊断与代码分析增强。

#### 验收标准

- [ ] LSP 诊断信息准确
- [ ] 多语言服务器支持
- [ ] 诊断结果集成到 TUI

---

### FR-014: 插件事件总线

**模块**: plugin
**严重程度**: P1
**来源**: v2

插件间事件通信机制。

#### 验收标准

- [ ] 事件注册与订阅
- [ ] 事件路由与分发
- [ ] 事件过滤

---

### FR-015: 凭证加密存储

**模块**: auth
**严重程度**: P1
**来源**: v2

API Key 等凭证的安全存储。

#### 验收标准

- [ ] 凭证加密存储
- [ ] 凭证读取解密
- [ ] 凭证删除

---

### FR-016: Permission 审计记录

**模块**: storage/permission
**严重程度**: P1
**来源**: v2

权限决策的审计日志。

#### 验收标准

- [ ] 权限决策记录存储
- [ ] 审计日志查询
- [ ] 审计日志清理

---

### FR-017: TUI Token/Cost 显示

**模块**: tui
**严重程度**: P1
**来源**: v2

TUI 中显示 token 使用量和成本。

#### 验收标准

- [ ] Token 计数准确
- [ ] 成本计算正确
- [ ] TUI 状态栏显示

---

### FR-018: TUI Schema 验证

**模块**: config/tui
**严重程度**: P1
**来源**: v2

TUI 配置的 JSON Schema 验证。

#### 验收标准

- [ ] tui.json 格式验证
- [ ] 验证错误提示清晰
- [ ] 内建默认值验证

---

### FR-019: scroll_acceleration 结构修复

**模块**: config/tui
**严重程度**: P1
**来源**: v2

滚动加速配置的类型结构修复。

#### 验收标准

- [ ] scroll_acceleration 为对象类型 `{"enabled": true}`
- [ ] 序列化/反序列化正确

---

### FR-020: keybinds 自定义绑定

**模块**: config/tui
**严重程度**: P1
**来源**: v2

自定义键盘快捷键绑定。

#### 验收标准

- [ ] keybinds 配置解析
- [ ] 快捷键覆盖默认行为
- [ ] 冲突检测

---

### FR-032: Snapshot 元数据完善

**模块**: storage
**严重程度**: P1
**来源**: v2

Snapshot 元数据字段完善。

#### 验收标准

- [ ] Snapshot 包含完整元数据
- [ ] 元数据查询 API
- [ ] 快照恢复功能

---

### FR-035: modes/ 目录扫描

**模块**: core/config/directory_scanner
**严重程度**: P1
**来源**: v3 (配置差距分析 Gap #3)

#### 验收标准

- [ ] `.opencode/modes/` 目录被正确扫描
- [ ] `~/.config/opencode/modes/` 目录被正确扫描
- [ ] 模式定义文件格式正确解析
- [ ] 扫描结果注册到配置系统

---

### FR-036: 配置路径命名统一为 opencode

**模块**: core/config
**严重程度**: P1
**来源**: v3 (配置差距分析 Gap #4)

#### 验收标准

- [ ] 配置目录路径为 `~/.config/opencode/`
- [ ] 使用 `directories` crate 管理路径
- [ ] 无硬编码路径字符串
- [ ] 旧路径迁移提示正常

---

### FR-037: {file:path} 支持 ~ 路径展开

**模块**: core/config
**严重程度**: P1
**来源**: v3 (配置差距分析 Gap #5)

#### 验收标准

- [ ] `{file:~/.secrets/api-key}` 正确读取文件
- [ ] `~` 展开为当前用户主目录
- [ ] 展开失败时有明确错误提示

---

### FR-038: {file:path} 支持相对于配置文件目录

**模块**: core/config
**严重程度**: P1
**来源**: v3 (配置差距分析 Gap #6)

#### 验收标准

- [ ] `{file:./instructions.md}` 相对于配置文件目录解析
- [ ] `{file:../shared/config.md}` 支持上级目录引用
- [ ] 相对路径解析在 `load_multi()` 中正确工作

---

### FR-039: .opencode/ 目录扫描集成到配置加载

**模块**: core/config
**严重程度**: P1
**来源**: v3 (配置差距分析 Gap #12)

#### 验收标准

- [ ] `load_multi()` 自动调用 `.opencode/` 目录扫描
- [ ] agents/commands/modes/plugins/skills/tools/themes 内容被加载
- [ ] 目录内容与主配置正确合并
- [ ] 同名配置按优先级覆盖

---

### FR-044: session_load/session_save 工具实现

**模块**: core/tools
**严重程度**: P1
**来源**: v4 (PRD vs Rust Gap Analysis §2.2)

#### 需求描述

实现 session_load 和 session_save 工具，支持会话的持久化加载和保存。

#### 详细规格

1. **session_load 工具**
   - 从存储加载指定会话
   - 恢复会话消息历史
   - 恢复会话上下文状态

2. **session_save 工具**
   - 保存当前会话到存储
   - 包含完整消息历史
   - 包含会话元数据

3. **API 集成**
   - 与现有 session 存储系统对接
   - 支持会话 ID 引用

#### 验收标准

- [ ] session_load 工具可加载历史会话
- [ ] session_save 工具可保存当前会话
- [ ] 加载后会话上下文完整恢复
- [ ] 与现有 session 存储系统兼容

---

### FR-045: 剩余内建 Skills 补全

**模块**: core/skills
**严重程度**: P1
**来源**: v4 (PRD vs Rust Gap Analysis §2.6, 5/10 内建 Skills 缺失)

#### 需求描述

补全剩余 5 个内建 Skills，达到 10/10 完整覆盖。

#### 详细规格

1. **当前状态**: 5/10 内建 Skills 已实现
2. **缺失 Skills**: 根据 PRD 定义补全剩余 5 个
3. **Skill 格式**: 遵循现有 Skill 定义规范 (SKILL.md 格式)

#### 验收标准

- [ ] 10/10 内建 Skills 全部实现
- [ ] 每个 Skill 有完整的 SKILL.md 定义
- [ ] Skills 可在 TUI 中被 Agent 发现和调用

---

### FR-046: 剩余 Commands 补全

**模块**: core/commands
**严重程度**: P1
**来源**: v4 (PRD vs Rust Gap Analysis §2.6, 3/8 Commands 已实现)

#### 需求描述

补全剩余 5 个 Commands，达到 8/8 完整覆盖。

#### 详细规格

1. **当前状态**: 3/8 Commands 已实现
2. **缺失 Commands**: 根据 PRD 定义补全剩余 5 个
3. **Command 格式**: 遵循现有 Command 定义规范

#### 验收标准

- [ ] 8/8 Commands 全部实现
- [ ] 每个 Command 有完整的定义
- [ ] Commands 可在 TUI 中通过 /cmd 触发

---

### FR-047: OAuth 登录支持

**模块**: auth
**严重程度**: P1
**来源**: v4 (PRD vs Rust Gap Analysis §2.4, v1.5+ 功能)

#### 需求描述

实现 OAuth 登录流程，支持用户认证。

#### 详细规格

1. **OAuth Flow**
   - 浏览器重定向登录
   - Token 交换与存储
   - Token 刷新机制

2. **集成点**
   - Provider 认证集成
   - 用户身份管理

#### 验收标准

- [ ] OAuth 登录流程可完成
- [ ] Token 安全存储
- [ ] Token 自动刷新
- [ ] 登录状态持久化

---

### FR-048: GitHub 集成

**模块**: git
**严重程度**: P1
**来源**: v4 (PRD vs Rust Gap Analysis §2.4, v1.5+ 功能)

#### 需求描述

实现 GitHub 集成，支持仓库操作。

#### 详细规格

1. **功能范围**
   - PR 创建/查看
   - Issue 管理
   - Code Review 辅助
   - Repository 信息获取

2. **认证**
   - 使用 OAuth 或 PAT 认证
   - GitHub API 集成

#### 验收标准

- [ ] GitHub API 连接正常
- [ ] PR 操作 (创建/查看/评论)
- [ ] Issue 操作 (创建/查看/更新)
- [ ] 认证流程完整

---

## 5. P2 - 完善性问题

> FR-021 ~ FR-031, FR-040 ~ FR-043 继承自 v3，内容不变。以下为 v4 新增 P2 需求。

### FR-021: 配置系统完善

**模块**: core/config
**严重程度**: P2
**来源**: v2

配置系统整体完善。

#### 验收标准

- [ ] merge_configs 避免 JSON 中转 (直接使用原生 deep_merge)
- [ ] 所有配置路径统一
- [ ] 配置验证完善

---

### FR-022: Session Summarize

**模块**: core
**严重程度**: P2
**来源**: v2

会话自动摘要功能。

#### 验收标准

- [ ] POST /sessions/{id}/summarize API 实现
- [ ] 摘要生成逻辑
- [ ] 摘要存储与检索

---

### FR-023: TUI 布局切换

**模块**: tui
**严重程度**: P2
**来源**: v2

TUI 布局动态切换。

#### 验收标准

- [ ] 多种布局预设
- [ ] 布局切换快捷键
- [ ] 布局状态持久化

---

### FR-024: TUI 右栏功能完善

**模块**: tui
**严重程度**: P2
**来源**: v2

TUI 右侧面板功能完善。

#### 验收标准

- [ ] 右栏内容可配置
- [ ] 面板折叠/展开
- [ ] 面板内容实时更新

---

### FR-025: TUI Patch 预览展开

**模块**: tui
**严重程度**: P2
**来源**: v2

TUI 中 Patch/Diff 预览交互。

#### 验收标准

- [ ] Patch 内容可展开查看
- [ ] Diff 高亮显示
- [ ] 接受/拒绝操作

---

### FR-026: Web UI

**模块**: tui
**严重程度**: P2
**来源**: v2

Web 界面支持。

#### 验收标准

- [ ] Web UI 基本功能
- [ ] 与 Server API 对接
- [ ] 实时通信 (WebSocket/SSE)

---

### FR-027: IDE 扩展预留

**模块**: core
**严重程度**: P2
**来源**: v2

IDE 扩展接口预留。

#### 验收标准

- [ ] IDE 扩展 API 定义
- [ ] 扩展点预留
- [ ] 文档说明

---

### FR-028: GitHub 集成预留

**模块**: git
**严重程度**: P2
**来源**: v2

GitHub 集成接口预留。

#### 验收标准

- [ ] GitHub API 接口定义
- [ ] 认证接口预留
- [ ] 扩展点文档

---

### FR-029: OAuth 登录预留

**模块**: auth
**严重程度**: P2
**来源**: v2

OAuth 登录接口预留。

#### 验收标准

- [ ] OAuth 接口定义
- [ ] Token 存储接口
- [ ] 扩展点文档

---

### FR-030: 废弃字段清理

**模块**: core/config
**严重程度**: P2
**来源**: v2

清理 PRD §9.1 声明的废弃字段。

#### 验收标准

- [ ] 废弃字段标记 #[deprecated]
- [ ] 加载时发出 warning
- [ ] 迁移文档更新

---

### FR-031: theme 路径解析增强

**模块**: config/tui
**严重程度**: P2
**来源**: v2

主题路径解析增强。

#### 验收标准

- [ ] 主题路径支持 ~ 展开
- [ ] 主题路径支持相对路径
- [ ] 主题文件不存在时降级

---

### FR-040: 变量替换覆盖完整性

**模块**: core/config
**严重程度**: P2
**来源**: v3 (配置差距分析 Gap #7)

#### 验收标准

- [ ] 所有配置加载路径执行变量替换
- [ ] 未设置变量替换为空字符串 (非保留原字符串)
- [ ] 变量替换在原始字符串层面进行
- [ ] 嵌套/复杂变量替换正确处理

---

### FR-041: theme/keybinds 从主配置迁移

**模块**: core/config
**严重程度**: P2
**来源**: v3 (配置差距分析 Gap #8)

#### 验收标准

- [ ] 主配置中 theme/keybinds 发出废弃警告
- [ ] 旧配置值自动迁移到 TUI 配置
- [ ] TUI 配置正确加载 theme/keybinds

---

### FR-042: AgentMapConfig 完全动态 HashMap

**模块**: core/config
**严重程度**: P2
**来源**: v3 (配置差距分析 Gap #10)

#### 验收标准

- [ ] 支持任意 agent 名称作为 key
- [ ] 无固定键限制
- [ ] 自定义 agent 正确加载

---

### FR-043: JSON Schema 远程验证实现

**模块**: core/config/schema
**严重程度**: P2
**来源**: v3 (配置差距分析 Gap #11)

#### 验收标准

- [ ] 从远程 URL 拉取 JSON Schema
- [ ] 本地缓存机制正常
- [ ] 内建 fallback schema 可用
- [ ] 验证错误提示详细
- [ ] 离线模式不阻断配置加载

---

### FR-049: HuggingFace 和 AI21 Provider 补全

**模块**: core/llm
**严重程度**: P2
**来源**: v4 (PRD vs Rust Gap Analysis §2.3, 16/18 providers)

#### 需求描述

补全 HuggingFace 和 AI21 LLM Provider，达到 18/18 完整覆盖。

#### 详细规格

1. **HuggingFace Provider**
   - API 端点配置
   - 模型列表支持
   - 认证方式 (API Key / Token)

2. **AI21 Provider**
   - API 端点配置
   - Jurassic 模型支持
   - 认证方式 (API Key)

3. **统一接口**
   - 遵循现有 Provider 接口规范
   - 支持 streaming 和非 streaming 模式

#### 验收标准

- [ ] HuggingFace Provider 可调用
- [ ] AI21 Provider 可调用
- [ ] 18/18 providers 完整覆盖
- [ ] 配置格式与其他 Provider 一致

---

### FR-050: Server mDNS 服务发现

**模块**: server
**严重程度**: P2
**来源**: v4 (Constitution v1.4 C-020)

#### 需求描述

实现 Server 的 mDNS 服务发现功能，允许局域网内自动发现 OpenCode 服务。

#### 详细规格

1. **配置项 (PRD §5.1)**
   - `port`: 监听端口号，默认 4096
   - `hostname`: 监听地址，默认 "0.0.0.0"
   - `mdns`: 是否启用 mDNS 服务发现，默认 true
   - `mdnsDomain`: mDNS 域名，可选
   - `cors`: CORS 允许的源列表

2. **mDNS 规范**
   - 启用 mdns 时，必须在局域网内广播服务
   - mdnsDomain 格式必须符合 mDNS 命名规范 (以 .local 结尾)
   - 未设置 mdnsDomain 时，使用默认格式 "opencode.local"

3. **CORS 规范**
   - cors 为空列表时，允许所有源 (开发模式)
   - cors 非空时，仅允许列表中的源
   - 生产环境必须配置 cors 白名单

4. **安全约束**
   - port 必须在 1024-65535 范围内 (非特权端口)
   - hostname 禁止设置为 "0.0.0.0" 在生产环境

#### 验收标准

- [ ] mDNS 服务发现可启用/禁用
- [ ] 默认 mdnsDomain 为 "opencode.local"
- [ ] 自定义 mdnsDomain 符合命名规范
- [ ] CORS 白名单机制工作正常
- [ ] 端口范围验证生效

---

### FR-051: Compaction 会话压缩

**模块**: core/session
**严重程度**: P2
**来源**: v4 (Constitution v1.4 C-021)

#### 需求描述

实现会话自动压缩 (Compaction) 功能，在上下文接近模型限制时自动压缩历史。

#### 详细规格

1. **配置项 (PRD §5.9)**
   - `auto`: 是否自动压缩会话，默认 true
   - `prune`: 是否移除旧工具输出以节省 token，默认 true
   - `reserved`: 压缩时保留的 token 缓冲，默认 10000

2. **自动压缩触发条件**
   - 会话 token 数接近模型上下文窗口限制时触发
   - 触发阈值 = 模型最大上下文 - reserved
   - 压缩必须保持对话语义连贯性

3. **Prune 规范**
   - prune 启用时，移除旧的工具输出内容，保留工具调用记录
   - 保留最近的 N 个工具调用完整输出 (N >= 3)
   - 被 prune 的内容必须标记为 "[content pruned to save tokens]"

4. **安全约束**
   - reserved 必须 > 0，防止压缩后无剩余 token
   - reserved 建议 >= 5000，确保模型有足够空间响应

#### 验收标准

- [ ] auto 启用时自动触发压缩
- [ ] prune 启用时正确移除旧工具输出
- [ ] 被 prune 内容标记正确
- [ ] reserved 配置生效
- [ ] 压缩后对话语义连贯

---

### FR-052: 文件 Watcher 配置

**模块**: core/watcher
**严重程度**: P2
**来源**: v4 (Constitution v1.4 C-022)

#### 需求描述

实现文件变更监视器 (Watcher) 的配置支持，包括忽略模式和性能约束。

#### 详细规格

1. **配置项 (PRD §5.10)**
   - `ignore`: glob 模式的忽略列表

2. **忽略模式规范**
   - 支持 glob 语法 (*, **, ?)
   - 必须默认忽略: `.git/**`, `node_modules/**`, `dist/**`, `build/**`
   - ignore 列表为用户自定义追加，非替换默认值

3. **性能约束**
   - 单个目录下的监视文件数不超过 10000
   - 忽略模式必须在文件系统层面生效 (而非事件过滤)
   - 监视器启动失败 (如 inotify 限制) 应记录 warning 但不阻断启动

#### 验收标准

- [ ] 默认忽略列表生效
- [ ] glob 模式正确解析
- [ ] 用户自定义 ignore 追加生效
- [ ] 监视器启动失败不阻断启动
- [ ] 文件数限制生效

---

## 6. 技术债务清单

| 债务项 | 位置 | 描述 | 关联 FR |
|--------|------|------|---------|
| **TOML vs JSON 格式分裂** | `config.rs:1012-1031` | `config_path()` 默认返回 `.toml`，但 PRD 要求 JSON/JSONC | FR-036 |
| **硬编码路径** | `config.rs:1031` | `"~/.config/opencode-rs/config.toml"` 硬编码 | FR-036 |
| **变量替换实现粗糙** | `config.rs:972-1009` | 字符串替换对嵌套/复杂情况可能出错 | FR-040 |
| **merge_configs 通过 JSON 中转** | `merge.rs:22-29` | 序列化→deep_merge→反序列化，丢失类型信息 | FR-021 |
| **fetch_remote_config 同步包装异步** | `config.rs:1107-1109` | 同步函数中创建 tokio runtime | - |
| **TimeoutConfig 枚举命名** | `config.rs:469-474` | `Disabled(bool)` 语义不清 | - |
| **PermissionConfig 大量重复字段** | `config.rs:628-697` | 应考虑宏生成或统一结构 | - |
| **Schema 验证空壳** | `schema.rs:5-40` | 只检查 2 个字段 | FR-043 |
| **DirectoryScanner 未使用 glob** | `directory_scanner.rs` | 手动 read_dir，不支持 glob 模式 | FR-035 |
| **测试覆盖不足** | `core/tests/` | 仅 2 个测试文件，缺少集成测试 | - |

---

## 7. 验收标准对照 (PRD §10)

| 验收项 | PRD § | 状态 | 关联 FR | 备注 |
|--------|-------|------|---------|------|
| JSON/JSONC 格式支持 | 10.1 | ✅ | - | `jsonc.rs` 完整实现 |
| 配置合并逻辑正确 | 10.1 | ✅ | FR-021 | `merge.rs` deep_merge 实现 |
| 6 个配置位置按优先级加载 | 10.1 | ⚠️ | FR-033, FR-039 | 缺少 OPENCODE_TUI_CONFIG，.opencode 目录扫描未集成 |
| `{env:VARIABLE_NAME}` 正确替换 | 10.2 | ✅ | - | 实现正确 |
| `{file:path}` 正确读取文件 | 10.2 | ⚠️ | FR-037, FR-038 | 不支持 `~` 和相对路径 |
| 未设置变量替换为空字符串 | 10.2 | ❌ | FR-040 | 当前保留原字符串 |
| TUI 配置与 runtime 分离 | 10.3 | ❌ | FR-034 | 未实现独立 tui.json |
| `OPENCODE_TUI_CONFIG` 自定义路径 | 10.3 | ❌ | FR-033 | 完全缺失 |
| Provider timeout/chunkTimeout/setCacheKey | 10.4 | ✅ | - | `ProviderOptions` 完整 |
| Amazon Bedrock 配置 | 10.4 | ✅ | - | awsRegion/awsProfile/awsEndpoint |
| disabled_providers 优先级 | 10.4 | ✅ | - | `is_provider_enabled()` 正确 |
| 自定义 agent 配置 | 10.5 | ✅ | FR-042 | AgentConfig 完整，但 AgentMapConfig 需改为动态 |
| default_agent 设置 | 10.5 | ✅ | - | 字段存在且被 env 覆盖 |
| 命令模板变量替换 | 10.5 | ⚠️ | FR-004 | 命令模板变量替换未明确实现 |
| permission 配置 | 10.6 | ✅ | - | `PermissionConfig` 完整 |
| API Key 文件引用 | 10.6 | ⚠️ | FR-037, FR-038 | 依赖 `{file:path}`，但该功能不完整 |

---

## 8. 功能需求清单汇总

### 按模块分组

| 模块 | FR 编号 | 需求名称 | 优先级 | 来源 |
|------|---------|----------|--------|------|
| core | FR-001 | Context Engine | P0 | v2 |
| core | FR-003 | Skills 系统 | P0 | v2 |
| core | FR-004 | Commands 系统 | P0 | v2 |
| core | FR-012 | Share 功能 | P1 | v2 |
| core | FR-014 | 插件事件总线 | P1 | v2 |
| core | FR-022 | Session Summarize | P2 | v2 |
| core/tools | FR-044 | session_load/session_save | P1 | v4 |
| core/skills | FR-045 | 剩余内建 Skills 补全 | P1 | v4 |
| core/commands | FR-046 | 剩余 Commands 补全 | P1 | v4 |
| core/session | FR-051 | Compaction 会话压缩 | P2 | v4 |
| core/watcher | FR-052 | 文件 Watcher 配置 | P2 | v4 |
| config | FR-008 | 多层配置合并 | P0 | v2 |
| config | FR-009 | .opencode 目录加载 | P0 | v2 |
| config | FR-010 | Provider 环境变量约定 | P0 | v2 |
| config | FR-021 | 配置系统完善 | P2 | v2 |
| config | FR-030 | 废弃字段清理 | P2 | v2 |
| config | FR-033 | OPENCODE_TUI_CONFIG 环境变量 | P0 | v3 |
| config | FR-034 | TUI 配置分离为独立文件 | P0 | v3 |
| config | FR-035 | modes/ 目录扫描 | P1 | v3 |
| config | FR-036 | 配置路径命名统一 | P1 | v3 |
| config | FR-037 | {file:path} ~ 路径展开 | P1 | v3 |
| config | FR-038 | {file:path} 相对路径支持 | P1 | v3 |
| config | FR-039 | .opencode/ 目录扫描集成 | P1 | v3 |
| config | FR-040 | 变量替换覆盖完整性 | P2 | v3 |
| config | FR-041 | theme/keybinds 迁移到 TUI | P2 | v3 |
| config | FR-042 | AgentMapConfig 动态 HashMap | P2 | v3 |
| config | FR-043 | JSON Schema 远程验证 | P2 | v3 |
| config/tui | FR-019 | scroll_acceleration 结构修复 | P1 | v2 |
| config/tui | FR-020 | keybinds 自定义绑定 | P1 | v2 |
| config/tui | FR-031 | theme 路径解析增强 | P2 | v2 |
| schema | FR-018 | TUI Schema 验证 | P1 | v2 |
| plugin | FR-002 | Plugin System | P0 | v2 |
| mcp | FR-005 | MCP 工具接入 | P0 | v2 |
| server | FR-006 | TUI 快捷输入解析器 | P0 | v2 |
| server | FR-007 | Session Fork | P0 | v2 |
| server | FR-011 | Server API 完善 | P1 | v2 |
| server | FR-050 | Server mDNS 服务发现 | P2 | v4 |
| storage | FR-032 | Snapshot 元数据完善 | P1 | v2 |
| storage/permission | FR-016 | Permission 审计记录 | P1 | v2 |
| lsp | FR-013 | LSP 功能增强 | P1 | v2 |
| auth | FR-015 | 凭证加密存储 | P1 | v2 |
| auth | FR-029 | OAuth 登录预留 | P2 | v2 |
| auth | FR-047 | OAuth 登录支持 | P1 | v4 |
| tui | FR-017 | TUI Token/Cost 显示 | P1 | v2 |
| tui | FR-023 | TUI 布局切换 | P2 | v2 |
| tui | FR-024 | TUI 右栏功能完善 | P2 | v2 |
| tui | FR-025 | TUI Patch 预览展开 | P2 | v2 |
| tui | FR-026 | Web UI | P2 | v2 |
| git | FR-028 | GitHub 集成预留 | P2 | v2 |
| git | FR-048 | GitHub 集成 | P1 | v4 |
| llm | FR-049 | HuggingFace/AI21 Provider | P2 | v4 |
| - | FR-027 | IDE 扩展预留 | P2 | v2 |

### 按优先级分组

| 优先级 | FR 编号 |
|--------|---------|
| P0 | FR-001, FR-002, FR-003, FR-004, FR-005, FR-006, FR-007, FR-008, FR-009, FR-010, FR-033, FR-034 |
| P1 | FR-011, FR-012, FR-013, FR-014, FR-015, FR-016, FR-017, FR-018, FR-019, FR-020, FR-032, FR-035, FR-036, FR-037, FR-038, FR-039, FR-044, FR-045, FR-046, FR-047, FR-048 |
| P2 | FR-021, FR-022, FR-023, FR-024, FR-025, FR-026, FR-027, FR-028, FR-029, FR-030, FR-031, FR-040, FR-041, FR-042, FR-043, FR-049, FR-050, FR-051, FR-052 |

---

## 9. 实施建议

### Phase 1: P0 阻断性问题 (当前优先级)

1. **FR-033 OPENCODE_TUI_CONFIG 环境变量** - 配置系统基础
2. **FR-034 TUI 配置分离** - 核心架构要求
3. **FR-001 Context Engine** - 核心依赖
4. **FR-005 MCP 工具接入** - 工具系统扩展
5. **FR-004 Commands 系统** - TUI 输入增强
6. **FR-006 TUI 快捷输入解析器** - 核心交互
7. **FR-003 Skills 系统** - 上下文增强
8. **FR-002 Plugin System** - 扩展性基础
9. **FR-007 Session Fork** - 会话分叉
10. **FR-008 多层配置合并** - 配置管理
11. **FR-009 .opencode 目录加载** - 模块化配置支持
12. **FR-010 Provider 环境变量约定** - 环境变量绑定

### Phase 2: P1 核心功能

1. **FR-039 .opencode/ 目录扫描集成** - 配置加载完整性
2. **FR-037 {file:path} ~ 路径展开** - 变量替换完整性
3. **FR-038 {file:path} 相对路径支持** - 变量替换完整性
4. **FR-035 modes/ 目录扫描** - 目录结构完整性
5. **FR-036 配置路径命名统一** - 生态兼容性
6. **FR-044 session_load/session_save** - 会话持久化
7. **FR-045 剩余内建 Skills 补全** - 能力扩展
8. **FR-046 剩余 Commands 补全** - 命令完整性
9. **FR-011 Server API** - API 完整性
10. **FR-013 LSP 功能增强** - 开发体验
11. **FR-012 Share 功能** - 协作能力
12. **FR-015 凭证加密存储** - 安全合规
13. **FR-014 插件事件总线** - 事件系统
14. **FR-016 Permission 审计记录** - 权限追踪
15. **FR-017 TUI Token/Cost 显示** - 成本感知
16. **FR-018 TUI Schema 验证** - 配置验证增强
17. **FR-019 scroll_acceleration 结构修复** - 类型修正
18. **FR-020 keybinds 自定义绑定** - 绑定扩展
19. **FR-032 Snapshot 元数据完善** - 数据完整性
20. **FR-047 OAuth 登录支持** - 用户认证 (v1.5+)
21. **FR-048 GitHub 集成** - DevOps 集成 (v1.5+)

### Phase 3: P2 完善性

1. **FR-040 变量替换覆盖完整性** - 配置系统完善
2. **FR-041 theme/keybinds 迁移** - 废弃声明一致性
3. **FR-042 AgentMapConfig 动态 HashMap** - 灵活性
4. **FR-043 JSON Schema 远程验证** - 配置校验
5. **FR-049 HuggingFace/AI21 Provider** - LLM 覆盖完整性
6. **FR-050 Server mDNS 服务发现** - 局域网发现
7. **FR-051 Compaction 会话压缩** - 上下文管理
8. **FR-052 文件 Watcher 配置** - 文件监视
9. **FR-021 配置系统** - 配置灵活性
10. **FR-022 Session Summarize** - 会话管理
11. **FR-023 TUI 布局切换** - UI 增强
12. **FR-024 TUI 右栏功能完善** - 面板功能
13. **FR-025 TUI Patch 预览展开** - Diff 交互
14. **FR-026 Web UI** - 多端支持
15. **FR-027 IDE 扩展预留** - 生态扩展
16. **FR-028 GitHub 集成预留** - DevOps 集成
17. **FR-029 OAuth 登录预留** - 认证扩展
18. **FR-030 废弃字段清理** - 代码清理
19. **FR-031 theme 路径解析增强** - 主题功能增强

---

## 10. 附录

### A. 数据模型状态

| PRD 数据模型 | 实现状态 | 备注 |
|--------------|----------|------|
| Session | ✅ 完整 | |
| Session.parent_session_id | ❌ 未实现 | FR-007 |
| Session.shared_id | ❌ 未实现 | FR-012 |
| Message | ✅ 完整 | |
| ToolCall | ✅ 完整 | |
| Snapshot | ⚠️ 部分 | FR-032 需完善 |
| PermissionDecision | ⚠️ 部分 | FR-016 需完善 |
| Provider/Credential | ✅ 完整 | |
| Project | ✅ 完整 | |
| Checkpoint | ✅ 完整 | |

### B. API 状态

| PRD API 路径 | 实现状态 | FR 编号 |
|--------------|----------|---------|
| POST /sessions | ⚠️ 部分 | FR-011 |
| GET /sessions | ⚠️ 部分 | FR-011 |
| GET /sessions/{id} | ⚠️ 部分 | FR-011 |
| POST /sessions/{id}/fork | ❌ 未实现 | FR-007, FR-011 |
| POST /sessions/{id}/summarize | ⚠️ 部分 | FR-011, FR-022 |
| POST /sessions/{id}/abort | ✅ 已实现 | |
| POST /sessions/{id}/prompt | ⚠️ 部分 | FR-011 |
| GET /sessions/{id}/messages | ⚠️ 部分 | FR-011 |
| POST /sessions/{id}/shell | ✅ 已实现 | |
| POST /sessions/{id}/command | ❌ 未实现 | FR-004, FR-011 |
| POST /sessions/{id}/permissions/{req_id}/reply | ✅ 已实现 | |
| GET /sessions/{id}/diff | ⚠️ 部分 | FR-011 |
| GET /sessions/{id}/snapshots | ⚠️ 部分 | FR-011, FR-032 |
| POST /sessions/{id}/revert | ⚠️ 部分 | FR-011 |
| GET /providers | ⚠️ 部分 | FR-011 |
| GET /models | ⚠️ 部分 | FR-011 |
| POST /providers/{id}/credentials | ❌ 未实现 | FR-011 |
| POST /providers/{id}/credentials/test | ❌ 未实现 | FR-011 |
| DELETE /providers/{id}/credentials | ❌ 未实现 | FR-011 |

### C. 配置系统状态

| 配置项 | 实现状态 | 关联 FR | 备注 |
|--------|----------|---------|------|
| JSON/JSONC 格式 | ✅ 完整 | - | jsonc.rs |
| 配置合并 | ✅ 完整 | FR-021 | merge.rs |
| Remote Config | ⚠️ 部分 | FR-008 | fetch_remote_config 同步包装异步 |
| Global Config | ⚠️ 部分 | FR-036 | 路径使用 opencode-rs |
| OPENCODE_CONFIG | ✅ 完整 | - | 环境变量支持 |
| OPENCODE_TUI_CONFIG | ❌ 未实现 | FR-033 | 完全缺失 |
| OPENCODE_CONFIG_CONTENT | ✅ 完整 | - | 内联配置 |
| Project Config | ✅ 完整 | - | .opencode/config.json |
| .opencode/ 目录扫描 | ⚠️ 部分 | FR-035, FR-039 | 缺少 modes/，未集成到 load_multi |
| {env:VAR} 变量替换 | ✅ 完整 | - | |
| {file:path} 变量替换 | ⚠️ 部分 | FR-037, FR-038 | 不支持 ~ 和相对路径 |
| TUI 配置分离 | ❌ 未实现 | FR-034 | 内嵌在主配置中 |
| Schema 验证 | ⚠️ 空壳 | FR-043 | 只检查 2 个字段 |
| Agent 配置 | ✅ 完整 | FR-042 | AgentMapConfig 需改为动态 |
| Command 配置 | ✅ 完整 | FR-004 | |
| Permission 配置 | ✅ 完整 | - | |
| Provider 配置 | ✅ 完整 | - | |
| MCP 配置 | ⚠️ 部分 | FR-005 | |
| theme 配置 | ⚠️ 部分 | FR-031, FR-041 | 未迁移到 tui.json |
| keybinds 配置 | ⚠️ 部分 | FR-020, FR-041 | 未迁移到 tui.json |
| Server 配置 (mDNS/CORS) | ⚠️ 部分 | FR-050 | 基础实现存在，mDNS 待完善 |
| Compaction 配置 | ⚠️ 部分 | FR-051 | 基础结构存在，自动压缩待实现 |
| Watcher 配置 | ⚠️ 部分 | FR-052 | 基础监视存在，ignore 配置待完善 |

### D. Agent/Tool/Provider 实现状态

| 类别 | PRD 要求 | 已实现 | 缺失 | 关联 FR |
|------|----------|--------|------|---------|
| Agent Types | 10 | 10 | 0 | - |
| Tools | 35 | 33 | session_load, session_save | FR-044 |
| LLM Providers | 18 | 16 | HuggingFace, AI21 | FR-049 |
| Built-in Skills | 10 | 5 | 5 | FR-045 |
| Commands | 8 | 3 | 5 | FR-046 |
| Permission System | 95% | ✅ | OAuth, GitHub | FR-047, FR-048 |
| Server APIs | 全部 | ✅ | - | - |
| MCP/LSP | 全部 | ✅ | - | - |

### E. Constitution 条款映射

| Constitution 条款 | 覆盖领域 | 关联 FR |
|-------------------|----------|---------|
| C-001 | 已废止 (被 C-016 替代) | - |
| C-002 ~ C-010 | 基础架构 | FR-001, FR-002 |
| C-011 | Config Schema 设计 | FR-008, FR-021 |
| C-012 | 变量替换规范 | FR-037, FR-038, FR-040 |
| C-013 | 目录扫描规范 (含 modes/) | FR-009, FR-035, FR-039 |
| C-014 | TUI Input Parser | FR-006 |
| C-015 | Session Fork | FR-007 |
| C-016 | Context Token Budget | FR-001 |
| C-017 | TUI 配置分离 | FR-033, FR-034 |
| C-018 | 路径命名统一 | FR-036 |
| C-019 | 文件引用变量 | FR-037, FR-038 |
| **C-020** | **Server 配置规范** | **FR-050** |
| **C-021** | **Compaction 配置规范** | **FR-051** |
| **C-022** | **Watcher 配置规范** | **FR-052** |

### F. v3 → v4 变更摘要

| 变更项 | 说明 |
|--------|------|
| 新增 FR-044 | session_load/session_save 工具 (P1) |
| 新增 FR-045 | 剩余内建 Skills 补全 (P1) |
| 新增 FR-046 | 剩余 Commands 补全 (P1) |
| 新增 FR-047 | OAuth 登录支持 (P1) |
| 新增 FR-048 | GitHub 集成 (P1) |
| 新增 FR-049 | HuggingFace/AI21 Provider (P2) |
| 新增 FR-050 | Server mDNS 服务发现 (P2) |
| 新增 FR-051 | Compaction 会话压缩 (P2) |
| 新增 FR-052 | 文件 Watcher 配置 (P2) |
| 新增 §8.D | Agent/Tool/Provider 实现状态表 |
| 新增 §8.E | Constitution 条款映射表 |
| 新增 §8.F | v3 → v4 变更摘要 |
| 更新 §2 | 需求总览 (P1: 16→20, P2: 15→20) |
| 更新 §8 | 功能需求清单汇总 (新增 9 项) |
| 更新 §9 | 实施建议 (Phase 2/3 新增项) |
| 更新 §10.C | 配置系统状态 (新增 Server/Compaction/Watcher) |

---

**文档状态**: 草稿
**下一步**: 基于本规格文档创建迭代 4 实施计划
