# OpenCode-RS 实现计划 v4

**版本**: 4.0
**日期**: 2026-04-04
**基于**: spec_v4.md + constitution_updates.md
**状态**: 草稿

---

## 1. 计划概述

### 1.1 目标

基于 iteration-4 规格文档 (FR-001 ~ FR-052)，在 iteration-3 已完成的 12 项配置任务基础上，继续推进剩余 40 项需求。重点关注：
- **新增 v4 需求** (FR-044 ~ FR-052): session 工具、Skills/Commands 补全、OAuth/GitHub、新 Provider、Server mDNS、Compaction、Watcher
- **v3 遗留需求** (FR-001 ~ FR-043): 配置系统已 100% 完成，剩余为非配置领域的 P0/P1 需求

### 1.2 实现完整度

| 模块 | v3 完成度 | v4 目标 | 备注 |
|------|-----------|---------|------|
| 配置系统 | ✅ 100% | ✅ 100% | 12/12 tasks 已完成 |
| 非配置 P0 | ❌ 0% | ⬜ 待启动 | Context/Plugin/Skills/Commands/MCP/InputParser/SessionFork |
| 非配置 P1 | ❌ 0% | ⬜ 待启动 | ServerAPI/Share/LSP/Events/Credential/Audit/TUI-Token/Schema验证 |
| v4 新增 P1 | ❌ 0% | ⬜ 待启动 | session工具/Skills补全/Commands补全/OAuth/GitHub |
| v4 新增 P2 | ❌ 0% | ⬜ 待启动 | Provider补全/mDNS/Compaction/Watcher |

### 1.3 阶段划分

| Phase | 优先级 | 任务数 | 目标 | 预计工期 |
|-------|--------|--------|------|----------|
| Phase 0 | P0 非配置 | 6 | Context/Plugin/Skills/Commands/MCP/InputParser | 12 天 |
| Phase 1 | P1 核心 | 8 | ServerAPI/Share/LSP/Credential/Audit/session工具/Skills补全/Commands补全 | 14 天 |
| Phase 2 | P1 v4新增 | 3 | OAuth/GitHub/SessionFork | 8 天 |
| Phase 3 | P2 完善性 | 6 | Provider补全/mDNS/Compaction/Watcher/Schema验证/配置完善 | 12 天 |

---

## 2. Phase 0: P0 非配置阻断性问题

> 配置系统 P0 (FR-033/034) 已在 iteration-3 完成。本阶段处理剩余 6 项 P0。

### 2.1 目标

实现系统核心架构组件：Context Engine、Plugin System、Skills、Commands、MCP、TUI Input Parser。

### 2.2 任务清单

#### Task 0.1: Context Engine 实现 (FR-001)

**目标**: 实现上下文构建引擎，管理对话上下文、token 预算、上下文窗口。

**子任务**:
1. 创建 `ContextBuilder` 结构体
   - 收集文件上下文 (已打开/已引用文件)
   - 收集工具上下文 (可用工具列表)
   - 收集会话上下文 (历史消息摘要)
2. 实现 token 预算计算
   - 使用 tiktoken 或等效库计算 token 数
   - 根据模型上下文窗口限制设定预算
3. 实现上下文窗口管理
   - 当 token 数接近限制时，自动截断/压缩旧消息
   - 保留系统提示和最近 N 条消息
4. 集成到 session 处理流程
5. 添加单元测试

**验收标准**:
- [ ] Context 构建正确包含文件/工具/会话信息
- [ ] Token 预算计算准确 (误差 < 5%)
- [ ] 上下文窗口管理符合模型限制

**预计工期**: 3 天
**依赖**: 无

---

#### Task 0.2: Plugin System 实现 (FR-002)

**目标**: 实现插件系统，支持扩展 OpenCode 功能。

**子任务**:
1. 定义 Plugin trait 接口
   - `name()`, `version()`, `init()`, `shutdown()`
   - 插件生命周期管理 (加载/卸载/重载)
2. 实现插件注册与发现机制
   - 扫描 `~/.config/opencode/plugins/` 和 `.opencode/plugins/`
   - 支持 .wasm 插件加载 (使用 wasmtime)
3. 实现插件间通信机制
   - 事件总线 (预留，FR-014 详细实现)
   - 共享状态管理
4. 集成到主程序启动流程
5. 添加单元测试

**验收标准**:
- [ ] 插件注册/发现机制工作正常
- [ ] 插件生命周期管理 (加载/卸载/重载)
- [ ] 插件间通信机制

**预计工期**: 3 天
**依赖**: 无

---

#### Task 0.3: Skills 系统实现 (FR-003)

**目标**: 实现 Skills 系统，为 Agent 提供专业能力。

**子任务**:
1. 完善 Skill 注册与发现
   - 当前已有 5/10 内建 Skills (skill.rs 中有基础结构)
   - 扫描内置 skills 目录和自定义 skills 路径
2. 实现 Skill 加载与执行
   - 解析 SKILL.md frontmatter
   - 将 skill 内容注入到 agent 上下文
3. 实现 5 个缺失的内建 Skills (FR-045 合并到此)
   - 根据 PRD 定义补全剩余 5 个
4. 集成到 TUI 和 Agent 系统
5. 添加单元测试

**验收标准**:
- [ ] 10/10 内建 Skills 全部实现
- [ ] Skill 注册与发现
- [ ] Skill 加载与执行
- [ ] 内建 Skills + 自定义 Skills 支持

**预计工期**: 3 天
**依赖**: 无

---

#### Task 0.4: Commands 系统实现 (FR-004)

**目标**: 实现 Commands 系统，支持自定义命令和 TUI 斜杠命令。

**子任务**:
1. 完善命令注册与执行
   - 当前已有 3/8 Commands (command.rs 中有 Help/Test/Debug)
   - 补全剩余 5 个命令 (FR-046 合并到此)
2. 实现 TUI 斜杠命令集成
   - `/help`, `/clear`, `/models`, `/agents`, `/share` 等
3. 实现命令模板变量替换
   - 支持 `{input}`, `{file}`, `{selection}` 等变量
4. 集成到 TUI 输入处理
5. 添加单元测试

**验收标准**:
- [ ] 8/8 Commands 全部实现
- [ ] 命令注册与执行
- [ ] TUI 斜杠命令集成
- [ ] 命令模板变量替换

**预计工期**: 2 天
**依赖**: 无

---

#### Task 0.5: MCP 工具接入完善 (FR-005)

**目标**: 完善 MCP (Model Context Protocol) 工具接入。

**子任务**:
1. 实现 MCP 服务器连接管理
   - 本地进程启动/停止
   - 远程 SSE 连接
   - 连接健康检查与自动重连
2. 实现 MCP 工具发现与调用
   - 从 MCP 服务器获取工具列表
   - 工具参数验证
   - 工具执行与结果返回
3. 实现 MCP 资源访问
   - 资源 URI 解析
   - 资源内容读取
4. 集成到工具系统
5. 添加单元测试

**验收标准**:
- [ ] MCP 服务器连接管理
- [ ] MCP 工具发现与调用
- [ ] MCP 资源访问

**预计工期**: 2 天
**依赖**: 无

---

#### Task 0.6: TUI 快捷输入解析器 (FR-006)

**目标**: 实现 TUI 输入语法解析器 (@file, !shell, /command)。

**子任务**:
1. 实现 `@file` 语法解析与文件选择
   - 支持 `@` 后接文件路径自动补全
   - 支持模糊匹配文件搜索
   - 多文件选择 (`@file1 @file2`)
2. 实现 `!shell` 语法解析与命令预览
   - 支持 `!` 后接 shell 命令
   - 命令预览与确认执行
3. 实现 `/command` 斜杠命令解析
   - 与 Commands 系统对接
   - 自动补全命令列表
4. 集成到 TUI 输入框
5. 添加单元测试

**验收标准**:
- [ ] @file 语法解析与文件选择
- [ ] !shell 语法解析与命令预览
- [ ] /command 斜杠命令解析

**预计工期**: 2 天
**依赖**: Task 0.4 (Commands 系统)

---

## 3. Phase 1: P1 核心功能

### 3.1 目标

实现核心功能缺失项：Server API、Share、LSP、凭证存储、审计日志、session 工具、Skills/Commands 补全。

### 3.2 任务清单

#### Task 1.1: Server API 完善 (FR-011)

**目标**: 补全 REST API 端点。

**子任务**:
1. 实现缺失的 API 端点:
   - `POST /sessions/{id}/fork` (与 FR-007 合并)
   - `POST /sessions/{id}/command`
   - `POST /providers/{id}/credentials`
   - `DELETE /providers/{id}/credentials`
   - `POST /providers/{id}/credentials/test`
2. 完善已有端点的部分实现:
   - `POST /sessions`, `GET /sessions`, `GET /sessions/{id}`
   - `POST /sessions/{id}/prompt`, `GET /sessions/{id}/messages`
   - `GET /sessions/{id}/diff`, `GET /sessions/{id}/snapshots`
   - `POST /sessions/{id}/revert`, `GET /providers`, `GET /models`
3. 实现认证/权限检查中间件
4. 统一错误响应格式
5. 添加集成测试

**验收标准**:
- [ ] 所有 PRD 定义的 API 端点实现
- [ ] 认证/权限检查
- [ ] 错误响应格式统一

**预计工期**: 3 天
**依赖**: 无

---

#### Task 1.2: Share 功能 (FR-012)

**目标**: 实现会话分享功能。

**子任务**:
1. 实现 `Session.shared_id` 字段
2. 实现分享链接生成
   - 生成唯一分享 ID
   - 创建只读分享视图
3. 实现分享权限控制
   - 公开/私有分享
   - 分享过期时间
4. 实现分享 API 端点
5. 添加单元测试

**验收标准**:
- [ ] Session.shared_id 字段实现
- [ ] 分享链接生成
- [ ] 分享权限控制

**预计工期**: 2 天
**依赖**: 无

---

#### Task 1.3: LSP 功能增强 (FR-013)

**目标**: LSP 诊断与代码分析增强。

**子任务**:
1. 完善 LSP 诊断信息
   - 诊断信息聚合与去重
   - 诊断 severity 分类
2. 实现多语言服务器支持
   - 按文件类型自动选择 LSP
   - 多 LSP 并行运行
3. 集成诊断结果到 TUI
   - 状态栏显示诊断数量
   - 诊断列表面板
4. 添加单元测试

**验收标准**:
- [ ] LSP 诊断信息准确
- [ ] 多语言服务器支持
- [ ] 诊断结果集成到 TUI

**预计工期**: 2 天
**依赖**: 无

---

#### Task 1.4: 凭证加密存储 (FR-015)

**目标**: API Key 等凭证的安全存储。

**子任务**:
1. 实现凭证加密存储
   - 使用 keyring 或等效库
   - AES-256-GCM 加密
2. 实现凭证读取解密
3. 实现凭证删除
4. 集成到 Provider 认证流程
5. 添加单元测试

**验收标准**:
- [ ] 凭证加密存储
- [ ] 凭证读取解密
- [ ] 凭证删除

**预计工期**: 2 天
**依赖**: 无

---

#### Task 1.5: Permission 审计记录 (FR-016)

**目标**: 权限决策的审计日志。

**子任务**:
1. 实现权限决策记录存储
   - 记录决策时间、工具名、决策结果
   - 存储到 SQLite
2. 实现审计日志查询
   - 按时间范围查询
   - 按工具名过滤
3. 实现审计日志清理
   - 自动清理超过 N 天的记录
   - 手动清理 API
4. 添加单元测试

**验收标准**:
- [ ] 权限决策记录存储
- [ ] 审计日志查询
- [ ] 审计日志清理

**预计工期**: 1 天
**依赖**: 无

---

#### Task 1.6: session_load/session_save 工具 (FR-044)

**目标**: 实现 session_load 和 session_save 工具。

**子任务**:
1. 实现 session_load 工具
   - 从 SQLite 存储加载指定会话
   - 恢复会话消息历史
   - 恢复会话上下文状态
2. 实现 session_save 工具
   - 保存当前会话到 SQLite
   - 包含完整消息历史
   - 包含会话元数据
3. 注册到工具系统
4. 与现有 session 存储系统对接
5. 添加单元测试

**验收标准**:
- [ ] session_load 工具可加载历史会话
- [ ] session_save 工具可保存当前会话
- [ ] 加载后会话上下文完整恢复
- [ ] 与现有 session 存储系统兼容

**预计工期**: 2 天
**依赖**: 无

---

#### Task 1.7: TUI Token/Cost 显示 (FR-017)

**目标**: TUI 中显示 token 使用量和成本。

**子任务**:
1. 实现 Token 计数
   - 每次请求统计 input/output tokens
   - 累计会话总 token 数
2. 实现成本计算
   - 根据模型定价计算成本
   - 支持自定义定价表
3. TUI 状态栏显示
   - 实时显示 token 数和成本
   - 预算警告
4. 添加单元测试

**验收标准**:
- [ ] Token 计数准确
- [ ] 成本计算正确
- [ ] TUI 状态栏显示

**预计工期**: 1 天
**依赖**: 无

---

#### Task 1.8: TUI Schema 验证 + scroll_acceleration 修复 + keybinds (FR-018/019/020)

**目标**: TUI 配置验证、滚动加速结构修复、快捷键绑定。

**子任务**:
1. TUI Schema 验证 (FR-018)
   - tui.json 格式验证
   - 验证错误提示
2. scroll_acceleration 结构修复 (FR-019)
   - 已实现 (iteration-3 中 ScrollAccelerationConfig 已修复)
   - 验证测试
3. keybinds 自定义绑定 (FR-020)
   - keybinds 配置解析
   - 快捷键覆盖默认行为
   - 冲突检测 (KeybindConfig.merge_with_defaults 已实现)
4. 添加集成测试

**验收标准**:
- [ ] tui.json 格式验证
- [ ] scroll_acceleration 为对象类型
- [ ] keybinds 配置解析与冲突检测

**预计工期**: 1 天
**依赖**: 无 (FR-019 已实现，FR-020 部分实现)

---

## 4. Phase 2: P1 v4 新增 + P0 Session Fork

### 4.1 目标

实现 OAuth 登录、GitHub 集成、Session Fork。

### 4.2 任务清单

#### Task 2.1: Session Fork (FR-007)

**目标**: 实现会话分叉功能。

**子任务**:
1. 添加 `Session.parent_session_id` 字段到数据模型
2. 实现 `POST /sessions/{id}/fork` API
3. 实现分叉点消息复制
   - 复制分叉点之前的所有消息
   - 设置 parent_session_id 关系
4. 实现 TUI 分叉操作
5. 添加单元测试

**验收标准**:
- [ ] POST /sessions/{id}/fork API 实现
- [ ] 父会话关系记录 (parent_session_id)
- [ ] 分叉点消息正确复制

**预计工期**: 2 天
**依赖**: Task 1.1 (Server API)

---

#### Task 2.2: OAuth 登录支持 (FR-047)

**目标**: 实现 OAuth 登录流程。

**子任务**:
1. 实现 OAuth Flow
   - 浏览器重定向登录
   - PKCE 流程实现
   - Token 交换与存储
2. 实现 Token 刷新机制
   - refresh_token 自动刷新
   - Token 过期检测
3. 集成到 Provider 认证
4. 实现登录状态持久化
5. 添加单元测试

**验收标准**:
- [ ] OAuth 登录流程可完成
- [ ] Token 安全存储
- [ ] Token 自动刷新
- [ ] 登录状态持久化

**预计工期**: 3 天
**依赖**: Task 1.4 (凭证加密存储)

---

#### Task 2.3: GitHub 集成 (FR-048)

**目标**: 实现 GitHub 集成。

**子任务**:
1. 实现 GitHub API 客户端
   - REST API 封装
   - GraphQL API 封装
2. 实现 PR 操作
   - 创建/查看/评论 PR
   - PR 状态查询
3. 实现 Issue 操作
   - 创建/查看/更新 Issue
   - Issue 列表查询
4. 实现 Repository 信息获取
5. 认证集成 (OAuth 或 PAT)
6. 添加单元测试

**验收标准**:
- [ ] GitHub API 连接正常
- [ ] PR 操作 (创建/查看/评论)
- [ ] Issue 操作 (创建/查看/更新)
- [ ] 认证流程完整

**预计工期**: 3 天
**依赖**: Task 2.2 (OAuth 登录)

---

## 5. Phase 3: P2 完善性问题

### 5.1 目标

解决 Provider 补全、mDNS、Compaction、Watcher 等完善性问题。

### 5.2 任务清单

#### Task 3.1: HuggingFace/AI21 Provider 补全 (FR-049)

**目标**: 补全 HuggingFace 和 AI21 LLM Provider。

**子任务**:
1. 实现 HuggingFace Provider
   - Inference API 端点
   - 模型列表支持
   - API Key 认证
   - streaming 支持
2. 实现 AI21 Provider
   - Jurassic 模型支持
   - API Key 认证
   - streaming 支持
3. 注册到 Provider 系统
4. 添加单元测试

**验收标准**:
- [ ] HuggingFace Provider 可调用
- [ ] AI21 Provider 可调用
- [ ] 18/18 providers 完整覆盖
- [ ] 配置格式与其他 Provider 一致

**预计工期**: 2 天
**依赖**: 无

---

#### Task 3.2: Server mDNS 服务发现 (FR-050)

**目标**: 实现 Server 的 mDNS 服务发现。

**子任务**:
1. 添加 `mdns` crate 依赖
2. 实现 mDNS 服务注册
   - 服务类型: `_opencode._tcp.local`
   - 默认域名: `opencode.local`
   - 自定义 mdnsDomain 支持
3. 实现 mDNS 启用/禁用
   - 基于 `server.mdns` 配置
4. 实现 CORS 白名单机制
   - 空列表 = 允许所有源
   - 非空列表 = 仅允许指定源
5. 实现端口范围验证 (1024-65535)
6. 添加单元测试

**验收标准**:
- [ ] mDNS 服务发现可启用/禁用
- [ ] 默认 mdnsDomain 为 "opencode.local"
- [ ] CORS 白名单机制工作正常
- [ ] 端口范围验证生效

**预计工期**: 2 天
**依赖**: 无

---

#### Task 3.3: Compaction 会话压缩 (FR-051)

**目标**: 实现会话自动压缩功能。

**子任务**:
1. 实现自动压缩触发
   - 监控会话 token 数
   - 触发阈值 = 模型最大上下文 - reserved
   - 使用 LLM 生成摘要压缩历史
2. 实现 Prune 功能
   - 移除旧工具输出
   - 保留最近 N 个工具调用 (N >= 3)
   - 标记 "[content pruned to save tokens]"
3. 实现 reserved 配置
   - 验证 reserved > 0
   - 默认 10000
4. 保持对话语义连贯性
5. 添加单元测试

**验收标准**:
- [ ] auto 启用时自动触发压缩
- [ ] prune 启用时正确移除旧工具输出
- [ ] 被 prune 内容标记正确
- [ ] reserved 配置生效

**预计工期**: 3 天
**依赖**: Task 0.1 (Context Engine)

---

#### Task 3.4: 文件 Watcher 配置 (FR-052)

**目标**: 实现文件变更监视器的配置支持。

**子任务**:
1. 实现 WatcherConfig 加载
   - `ignore` 字段解析
   - 默认忽略列表: `.git/**`, `node_modules/**`, `dist/**`, `build/**`
   - 用户自定义 ignore 追加
2. 实现 glob 模式解析
   - 使用 `globset` crate
   - 支持 `*`, `**`, `?` 语法
3. 实现文件监视器
   - 使用 `notify` crate
   - 忽略模式在文件系统层面生效
   - 文件数限制 (<= 10000)
4. 实现启动失败容忍
   - 记录 warning 但不阻断
5. 添加单元测试

**验收标准**:
- [ ] 默认忽略列表生效
- [ ] glob 模式正确解析
- [ ] 用户自定义 ignore 追加生效
- [ ] 监视器启动失败不阻断启动

**预计工期**: 2 天
**依赖**: 无

---

#### Task 3.5: 配置系统完善 (FR-021/030/031)

**目标**: 配置系统整体完善。

**子任务**:
1. merge_configs 优化 (FR-021)
   - iteration-3 已添加文档说明
   - 可选: 实现直接 struct-level merge
2. 废弃字段清理 (FR-030)
   - iteration-3 已标记 `#[deprecated]`
   - 验证 warning 日志
3. theme 路径解析增强 (FR-031)
   - 主题路径支持 ~ 展开
   - 主题路径支持相对路径
   - 主题文件不存在时降级
4. 添加集成测试

**验收标准**:
- [ ] 废弃字段标记与 warning
- [ ] theme 路径支持 ~ 和相对路径
- [ ] 主题文件不存在时降级

**预计工期**: 1 天
**依赖**: 无 (大部分已在 iteration-3 完成)

---

#### Task 3.6: 其他 P2 需求 (FR-022/023/024/025/026/027/028/029)

**目标**: 实现剩余 P2 完善性需求。

**子任务**:
1. Session Summarize (FR-022)
   - `POST /sessions/{id}/summarize` API
   - 使用 LLM 生成会话摘要
2. TUI 布局切换 (FR-023)
   - 多种布局预设
   - 切换快捷键
3. TUI 右栏功能完善 (FR-024)
   - 右栏内容可配置
   - 面板折叠/展开
4. TUI Patch 预览展开 (FR-025)
   - Patch 内容可展开
   - Diff 高亮显示
   - 接受/拒绝操作
5. Web UI (FR-026) — 预留接口
6. IDE 扩展预留 (FR-027)
7. GitHub 集成预留 (FR-028) — 已被 FR-048 覆盖
8. OAuth 登录预留 (FR-029) — 已被 FR-047 覆盖

**验收标准**:
- [ ] Session Summarize API 实现
- [ ] TUI 布局切换
- [ ] TUI 右栏功能
- [ ] TUI Patch 预览

**预计工期**: 4 天
**依赖**: Task 1.1 (Server API), Task 0.6 (TUI Input Parser)

---

## 6. 资源分配

### 6.1 人力估算

| Phase | 任务数 | 预计工期 | 总人天 |
|-------|--------|----------|--------|
| Phase 0 | 6 | 15 天 | 15 人天 |
| Phase 1 | 8 | 14 天 | 14 人天 |
| Phase 2 | 3 | 8 天 | 8 人天 |
| Phase 3 | 6 | 12 天 | 12 人天 |
| **总计** | **23** | **49 天** | **49 人天** |

### 6.2 并行策略

**Phase 0 并行**:
```
Task 0.1 (Context) ──────┐
Task 0.2 (Plugin) ───────┤
Task 0.3 (Skills) ───────┤  全部可并行
Task 0.4 (Commands) ─────┤
Task 0.5 (MCP) ──────────┤
Task 0.6 (InputParser) ──┘ 依赖 Task 0.4
```

**Phase 1 并行**:
```
Task 1.1 (ServerAPI) ────┐
Task 1.2 (Share) ────────┤
Task 1.3 (LSP) ──────────┤  全部可并行
Task 1.4 (Credential) ───┤
Task 1.5 (Audit) ────────┤
Task 1.6 (session工具) ──┤
Task 1.7 (Token/Cost) ───┤
Task 1.8 (TUI验证) ──────┘
```

**Phase 2 依赖链**:
```
Task 1.4 (Credential) → Task 2.2 (OAuth) → Task 2.3 (GitHub)
Task 1.1 (ServerAPI) → Task 2.1 (SessionFork)
```

**Phase 3 依赖链**:
```
Task 0.1 (Context) → Task 3.3 (Compaction)
Task 1.1 + Task 0.6 → Task 3.6 (TUI增强)
Task 3.1/3.2/3.4/3.5 可并行
```

---

## 7. 里程碑

| Milestone | Phase | 任务 | 预计完成 |
|-----------|-------|------|----------|
| M0 | Phase 0 | P0 核心架构 (Context/Plugin/Skills/Commands/MCP) | Week 3 |
| M1 | Phase 1 | P1 核心功能 (ServerAPI/Share/LSP/Credential) | Week 5 |
| M2 | Phase 2 | P1 v4新增 (OAuth/GitHub/SessionFork) | Week 7 |
| M3 | Phase 3 | P2 完善性 (Provider/mDNS/Compaction/Watcher) | Week 9 |

---

## 8. Constitution 合规性

本计划遵循 Constitution v1.4 全部条款:

| 条款 | 覆盖任务 | 说明 |
|------|----------|------|
| C-011 | Task 0.1 | Config Schema 设计 |
| C-012 | Task 0.4 | 变量替换规范 |
| C-013 | Task 0.2, 0.3 | 目录扫描规范 (含 modes/) |
| C-014 | Task 0.6 | TUI Input Parser |
| C-015 | Task 2.1 | Session Fork |
| C-016 | Task 0.1 | Context Token Budget |
| C-017 | (已完成) | TUI 配置分离 |
| C-018 | (已完成) | 路径命名统一 |
| C-019 | (已完成) | 文件引用变量 |
| **C-020** | **Task 3.2** | **Server 配置规范** |
| **C-021** | **Task 3.3** | **Compaction 配置规范** |
| **C-022** | **Task 3.4** | **Watcher 配置规范** |

---

## 9. 风险与缓解

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| MCP 协议变更 | 高 | 中 | 使用官方 SDK，定期同步 |
| OAuth 提供商差异 | 中 | 高 | 抽象 OAuth 接口，适配各提供商 |
| Compaction 语义连贯性 | 高 | 高 | 使用 LLM 摘要，人工验证质量 |
| mDNS 跨平台兼容性 | 中 | 中 | 使用成熟 crate (mdns-sd)，多平台测试 |
| WASM 插件性能 | 中 | 低 | 性能基准测试，设定超时 |

---

**文档状态**: 草稿
**下一步**: 基于本计划创建 tasks_v4.md 任务清单
