# OpenCode-RS 实施计划 v7

**版本**: 7.0  
**日期**: 2026-04-05  
**基于**: gap-analysis.md + spec_v7.md + constitution_updates.md  
**状态**: 已完成

---

## 1. 计划概述

### 1.1 背景

本计划基于 iteration-7 差距分析编写，旨在解决以下核心问题：
- **P0 (2项)**: OAuth/Device Code 缺失 + Provider 认证分层未完成
- **P1 (9项)**: Remote Config、disabled_providers 优先级、TUI Inspector、MCP OAuth、云厂商认证、Plugin WASM、凭证加密等
- **P2 (10项)**: Share 服务、OpenAPI、LSP 扩展、session 工具等

### 1.2 目标

1. **立即目标**: 解决 2 个 P0 阻断性问题
2. **短期目标**: 完成 9 个 P1 核心功能
3. **中期目标**: 实现 10 个 P2 增强功能
4. **验收标准**: 满足 Constitution v1.7 + PRD v1.1 的所有要求

---

## 2. 实施阶段

### Phase 1: P0 阻断性问题 (最高优先级)

**目标**: 解决 2 个 P0 阻断性问题，为后续所有认证相关功能奠定基础

| 序号 | 任务 | 模块 | FR | 依赖 | 预计工时 |
|------|------|------|-----|------|----------|
| P0-1 | 实现 OAuth/Device Code 浏览器登录流程 | llm/auth | FR-064 | FR-063 | 3d |
| P0-2 | 实现 Provider 认证协议分层抽象 | llm/auth | FR-063 | - | 5d |

#### P0-1: OAuth/Device Code 浏览器登录流程

**详细任务**:
1. 实现 OAuth Browser Flow 状态机
   - 本地 HTTP 回调服务器 (localhost:PORT)
   - 浏览器自动打开 (xdg-open/open)
   - Token 持久化到 auth.json
   - Token 自动刷新 (refresh_token)
2. 实现 Device Code Flow 状态机
   - code 获取与展示
   - 轮询授权状态 (polling)
   - 超时与取消处理
3. 集成到现有 auth crate

**验收标准**:
- [ ] OAuth Browser Flow 可完成 GitHub Copilot 登录
- [ ] Device Code Flow 可完成 GitLab Duo 登录
- [ ] Token 自动刷新正常工作

#### P0-2: Provider 认证协议分层抽象

**详细任务**:
1. 实现 Layer 1: Credential Source (环境变量/API Key/云厂商凭证/OAuth)
2. 实现 Layer 2: Auth Mechanism (Bearer/API Key/OAuth/Device Code)
3. 实现 Layer 3: Provider Transport (OpenAI-compatible/自定义)
4. 实现 Layer 4: Runtime Access Control (权限控制)
5. 重构现有 Provider 实现以复用分层架构

**验收标准**:
- [ ] 4 层架构完整实现
- [ ] 75+ providers 可通过分层架构配置
- [ ] Provider 认证与 Runtime 认证分离

---

### Phase 2: P1 核心功能

**目标**: 完成 9 个 P1 核心功能

| 序号 | 任务 | 模块 | FR | 依赖 | 预计工时 |
|------|------|------|-----|------|----------|
| P1-1 | 云厂商原生认证 (AWS/Bedrock/Vertex AI) | llm | FR-065 | P0-2 | 2d |
| P1-2 | Remote Config 自动发现 (.well-known) | core/config | FR-066 | - | 1d |
| P1-3 | disabled_providers 优先级实现 | core/config | FR-067 | - | 0.5d |
| P1-4 | MCP OAuth 独立 token store | mcp | FR-068 | P0-1 | 1d |
| P1-5 | TUI 三栏布局 + Inspector 面板 | tui | FR-069 | - | 3d |
| P1-6 | TUI 状态机完整实现 | tui | FR-070 | P1-5 | 2d |
| P1-7 | Context Engine 分层上下文 (L0-L4) | core | FR-071 | - | 2d |
| P1-8 | Plugin WASM 运行时 | plugin | FR-072 | - | 2d |
| P1-9 | 凭证加密存储 | auth | FR-073 | P0-1 | 1d |

#### P1-1: 云厂商原生认证

**详细任务**:
1. AWS Bedrock / Credential Chain 优先级实现
2. Vertex AI (GOOGLE_APPLICATION_CREDENTIALS) 支持
3. Bearer Token > Credential Chain 优先级
4. 测试用例覆盖

**验收标准**:
- [ ] AWS Bedrock 可用
- [ ] Vertex AI 可用
- [ ] 凭证链优先级正确

#### P1-2: Remote Config 自动发现

**详细任务**:
1. 实现 .well-known/opencode 自动发现
2. 支持 HTTP/HTTPS URL 配置
3. 自动合并远程配置到本地配置

**验收标准**:
- [ ] 可通过 .well-known 发现远程配置
- [ ] 远程配置优先级低于本地配置

#### P1-3: disabled_providers 优先级

**详细任务**:
1. 修改配置合并逻辑
2. disabled_providers 优先级高于 enabled_providers

**验收标准**:
- [ ] disabled_providers 可禁用任何 enabled_providers

#### P1-4: MCP OAuth 独立 token store

**详细任务**:
1. 创建 mcp-auth.json 独立存储
2. 实现 MCP OAuth token 生命周期管理

**验收标准**:
- [ ] MCP OAuth token 独立存储
- [ ] Token 刷新/失效处理正常

#### P1-5: TUI 三栏布局 + Inspector 面板

**详细任务**:
1. 实现 Sidebar/Timeline/Inspector 三栏布局
2. Inspector 面板 6 个 tab: Messages / Files / Tools / Sessions / Config / Debug
3. 虚拟滚动支持大消息列表

**验收标准**:
- [ ] 三栏布局正确显示
- [ ] Inspector 6 个 tab 可切换

#### P1-6: TUI 状态机完整实现

**详细任务**:
1. 实现 PRD-tui 定义的 10 种状态机
2. 状态流转正确性验证

**验收标准**:
- [ ] 10 种状态机完整实现
- [ ] 状态流转正确

#### P1-7: Context Engine 分层上下文

**详细任务**:
1. 实现 L0 (System) / L1 (User) / L2 (Session) / L3 (Project) / L4 (File) 分层
2. Token Budget 按层分配

**验收标准**:
- [ ] 5 层上下文正确实现
- [ ] Token Budget 分配正确

#### P1-8: Plugin WASM 运行时

**详细任务**:
1. 集成 WASM 执行引擎 (wasmer/wasmtime)
2. 实现 Plugin 加载/卸载/隔离

**验收标准**:
- [ ] WASM 插件可加载执行
- [ ] 插件隔离正确

#### P1-9: 凭证加密存储

**详细任务**:
1. 接入系统密钥链 (macOS Keychain / Windows Credential Manager)
2. 实现凭证加密/解密

**验收标准**:
- [ ] 凭证加密存储
- [ ] 密钥链集成成功

---

### Phase 3: P2 完善性

**目标**: 完成 10 个 P2 增强功能

| 序号 | 任务 | 模块 | FR | 依赖 | 预计工时 |
|------|------|------|-----|------|----------|
| P2-1 | Share 服务层 | core/share | FR-075 | - | 2d |
| P2-2 | OpenAPI 文档自动生成 | server | FR-077 | - | 1d |
| P2-3 | LSP definition/references/hover | lsp | FR-078 | - | 2d |
| P2-4 | session_load/session_save 工具 | tools | FR-079 | - | 1d |
| P2-5 | Formatters 接入 agent 执行循环 | core/formatter | FR-081 | - | 1d |
| P2-6 | Compaction 自动触发阈值 | core | FR-082 | - | 1d |
| P2-7 | Server HTTP Basic Auth | server | FR-084 | P0-1 | 0.5d |
| P2-8 | 观测性 (tracing/crash recovery/token cost) | core | FR-085 | - | 2d |
| P2-9 | 自定义 Commands 目录实现 | .opencode | FR-004 | - | 1d |
| P2-10 | HuggingFace + AI21 Provider 完整实现 | llm | FR-080 | P0-2 | 1d |

---

### Phase 4: P3 远期规划

| 序号 | 任务 | 模块 | FR | 预计工时 |
|------|------|------|-----|----------|
| P3-1 | GitHub Integration (v2) | git | FR-087 | 待定 |
| P3-2 | Enterprise 配置 (Central Config + SSO) | core | FR-088 | 待定 |

---

## 3. 里程碑

| 里程碑 | 日期 | 完成条件 |
|--------|------|----------|
| M1: P0 完成 | T+8d | ✅ P0-1 + P0-2 验收通过 |
| M2: P1 完成 | T+18d | ✅ P1-1 ~ P1-9 验收通过 |
| M3: P2 完成 | T+30d | ✅ P2-1 ~ P2-10 验收通过 |
| M4: Alpha Release | T+35d | ✅ 全部 P0/P1/P2+P3 验收通过 |

---

## 4. 资源分配

### 4.1 人力投入

| 角色 | 投入比例 |
|------|----------|
| 架构师 | 20% (P0 架构设计) |
| 后端工程师 | 60% (llm/core/server) |
| 前端工程师 | 15% (tui) |
| 测试工程师 | 5% (关键路径) |

### 4.2 技术依赖

| 依赖 | 版本 | 用途 |
|------|------|------|
| rust | 1.75+ | 编译环境 |
| tokio | 1.x | 异步 runtime |
| wasmer | 2.x | WASM 执行引擎 |
| serde | 1.x | 序列化 |
| tracing | 0.1 | 观测性 |

---

## 5. 风险与缓解

| 风险 | 等级 | 缓解措施 |
|------|------|----------|
| P0-2 重构复杂度高 | 高 | 分层实现，每层独立验证 |
| WASM 运行时兼容性 | 中 | 多引擎支持 (wasmer/wasmtime) |
| 测试覆盖不足 | 高 | 增加 TDD 实践 |

---

## 6. 验收标准

### 6.1 P0 验收

- [ ] OAuth Browser Flow 可完成 GitHub Copilot 登录
- [ ] Device Code Flow 可完成 GitLab Duo 登录
- [ ] 4 层认证架构完整实现
- [ ] 75+ providers 可通过分层架构配置

### 6.2 P1 验收

- [ ] AWS Bedrock / Vertex AI 可用
- [ ] Remote Config 自动发现工作
- [ ] disabled_providers 优先级正确
- [ ] MCP OAuth 独立存储
- [ ] TUI 三栏布局 + Inspector 6 tabs
- [ ] TUI 10 种状态机完整
- [ ] Context Engine 5 层上下文
- [ ] Plugin WASM 运行时可用
- [ ] 凭证加密存储

### 6.3 P2 验收

- [ ] Share 服务层可用
- [ ] OpenAPI 文档生成
- [ ] LSP definition/references/hover 实现
- [ ] session_load/session_save 工具可用
- [ ] Formatters 接入 agent 执行
- [ ] Compaction 自动触发阈值
- [ ] Server HTTP Basic Auth
- [ ] 观测性完整
- [ ] Commands 目录可用
- [ ] HuggingFace + AI21 Provider 可用

---

## 7. 追溯链

```
gap-analysis.md
    │
    ├── P0: 2 项 → FR-063, FR-064
    ├── P1: 9 项 → FR-065 ~ FR-073
    └── P2: 10 项 → FR-074 ~ FR-085
            │
            ▼
constitution_updates.md (v1.7)
    │
    └── P0/P1 100% 覆盖确认
            │
            ▼
spec_v7.md
    │
    └── FR-001 ~ FR-088 (88 项)
            │
            ▼
本计划 (plan_v7.md)
    │
    └── Phase 1-4 实施任务
```

---

**文档状态**: 草稿  
**下一步**: 创建 tasks_v7.md 并开始 P0 任务实现
