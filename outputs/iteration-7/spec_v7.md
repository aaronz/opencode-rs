# OpenCode-RS 规格文档 v7

**版本**: 7.0
**日期**: 2026-04-05
**基于**: spec_v6.md + iteration-7 差距分析 + Constitution v1.7 审计确认
**状态**: 草稿

---

## 1. 文档概述

### 1.1 背景

本规格文档基于以下文档综合生成：
- **spec_v6.md**: 上一版规格文档 (FR-001 ~ FR-088)
- **outputs/iteration-7/gap-analysis.md**: iteration-7 差距分析报告 (2 P0, 9 P1, 10 P2)
- **outputs/iteration-7/constitution_updates.md**: Constitution v1.7 审计确认 (P0/P1 100% 覆盖，无需修订)
- **PRD-OpenCode-Configuration.md**: 配置系统产品需求文档 (v1.0)
- **docs/PRD.md**: 产品需求文档 v1.1 (完整系统级 PRD)
- **docs/PRD-providers.md**: Provider 与认证协议详细规格 (75+ providers, 5 类认证层)
- **docs/PRD-tui.md**: TUI 产品需求 (三栏布局/10 种状态机/Inspector 面板/虚拟滚动)

### 1.2 目标

- 基于 iteration-7 差距分析，确认现有 88 项 FR 覆盖所有识别的问题
- Constitution v1.7 审计确认 P0/P1 问题 100% 覆盖，无需修订
- 记录差距分析→Constitution→spec 的完整追溯链
- 为 iteration-8 提供实施基线

### 1.3 参考文档

| 文档 | 路径 | 说明 |
|------|------|------|
| PRD-主文档 | `docs/PRD.md` | 产品需求文档 v1.1 |
| PRD-Providers | `docs/PRD-providers.md` | Provider 与认证协议详细规格 |
| PRD-TUI | `docs/PRD-tui.md` | TUI 产品需求详细设计 |
| PRD-配置系统 | `PRD-OpenCode-Configuration.md` | 配置系统产品需求 |
| Constitution v1.7 | `outputs/iteration-7/constitution_updates.md` | 设计约束条款 (C-001 ~ C-037) |
| spec_v6 | `outputs/iteration-6/spec_v6.md` | 上一版规格文档 |
| 差距分析 | `outputs/iteration-7/gap-analysis.md` | 差距分析报告 (2026-04-05) |

### 1.4 与 v6 的关系

v7 保留 v6 的所有需求 (FR-001 ~ FR-088)，无新增 FR。

v7 变更：
- 确认 iteration-7 差距分析中的 19 项问题 (2 P0 + 9 P1 + 10 P2) 已被 FR-063 ~ FR-088 完整覆盖
- Constitution v1.7 审计确认 P0/P1 问题 100% 覆盖
- 记录完整的追溯链：gap analysis → Constitution → spec → FR

---

## 2. 需求总览

| 优先级 | 数量 | 说明 |
|--------|------|------|
| P0 | 14 | 阻断性问题 (v5: 12, v6新增: 2) |
| P1 | 34 | 核心功能缺失 (v5: 25, v6新增: 9) |
| P2 | 37 | 完善性问题 (v5: 25, v6新增: 12) |
| P3 | 2 | 远期规划 (v5: 0, v6新增: 2) |

**总计**: 88 项功能需求 (v5: 62 项)

---

## 3. 差距分析 → FR 追溯链

### 3.1 P0 问题追溯

| 差距分析 P0 问题 | FR 编号 | 覆盖状态 | Constitution 条款 |
|-----------------|---------|----------|------------------|
| OAuth/Device Code 浏览器登录未实现 | FR-064 | ✅ 已覆盖 | C-026 §3c-d |
| Provider 认证分层架构未完成 | FR-063 | ✅ 已覆盖 | C-026 §1-5 |

### 3.2 P1 问题追溯

| 差距分析 P1 问题 | FR 编号 | 覆盖状态 | Constitution 条款 |
|-----------------|---------|----------|------------------|
| Remote Config 自动发现未实现 | FR-066 | ✅ 已覆盖 | C-037 |
| disabled_providers 优先级未实现 | FR-067 | ✅ 已覆盖 | C-030 §1c |
| TUI 三栏布局 Inspector 面板不完整 | FR-069 | ✅ 已覆盖 | C-034 §1-2 |
| MCP OAuth 独立 token 存储缺失 | FR-068 | ✅ 已覆盖 | C-033 |
| 云厂商原生认证不完整 | FR-065 | ✅ 已覆盖 | C-026 §3e + C-030 §2b-e |
| Plugin WASM 运行时缺失 | FR-072 | ✅ 已覆盖 | C-036 |
| 凭证加密存储未实现 | FR-073 | ✅ 已覆盖 | C-026 §6 + C-028 §4b |

### 3.3 P2 问题追溯

| 差距分析 P2 问题 | FR 编号 | 覆盖状态 |
|-----------------|---------|----------|
| Share 服务层未实现 | FR-075 | ✅ 已覆盖 |
| OpenAPI 文档自动生成缺失 | FR-077 | ✅ 已覆盖 |
| LSP definition/references/hover 未实现 | FR-078 | ✅ 已覆盖 |
| session_load/session_save 工具缺失 | FR-079 | ✅ 已覆盖 |
| Formatters 配置未完全接入 | FR-081 | ✅ 已覆盖 |
| Compaction 自动触发阈值未实现 | FR-082 | ✅ 已覆盖 |
| Server HTTP Basic Auth 未实现 | FR-084 | ✅ 已覆盖 |
| 观测性不完整 | FR-085 | ✅ 已覆盖 |
| 自定义 Commands 目录空置 | FR-004 | ✅ 已覆盖 |
| HuggingFace/AI21 Provider 状态存疑 | FR-080 | ✅ 已覆盖 |

### 3.4 Constitution v1.7 审计确认

| 指标 | 状态 |
|------|------|
| P0 问题覆盖率 | **100%** (2/2) |
| P1 问题覆盖率 | **100%** (9/9) |
| P2 问题覆盖率 | **100%** (10/10) |
| 需修订 Constitution 条款 | **无** |
| 审计结论 | **Constitution 已完备，重点应转向实现验证** |

---

## 4. FR 状态汇总

### 4.1 P0 - 阻断性问题

> FR-001 ~ FR-010, FR-033, FR-034 继承自 v4/v5，内容不变。

| FR 编号 | 需求名称 | 覆盖差距 | Constitution 条款 | 状态 |
|--------|----------|----------|-------------------|------|
| FR-063 | Provider 认证协议分层抽象 | P0-2 | C-026 §1-5 | 待实现 |
| FR-064 | OAuth/Device Code 浏览器登录流程 | P0-1 | C-026 §3c-d | 待实现 |

### 4.2 P1 - 核心功能缺失

> FR-011 ~ FR-020, FR-032, FR-035 ~ FR-039, FR-044 ~ FR-048, FR-053 ~ FR-056 继承自 v5，内容不变。

| FR 编号 | 需求名称 | 覆盖差距 | Constitution 条款 | 状态 |
|--------|----------|----------|-------------------|------|
| FR-065 | 云厂商原生认证 | P1-7 | C-026 §3e + C-030 §2b-e | 待实现 |
| FR-066 | Remote Config 自动发现 | P1-3 | C-037 | 待实现 |
| FR-067 | disabled_providers 优先级 | P1-4 | C-030 §1c | 待实现 |
| FR-068 | MCP OAuth 独立 token store | P1-6 | C-033 | 待实现 |
| FR-069 | TUI 三栏布局与 Inspector 面板 | P1-5 | C-034 §1-2 | 待实现 |
| FR-070 | TUI 状态机完整实现 | P1 关联 | C-034 §3-4 | 待实现 |
| FR-071 | Context Engine 分层上下文 | P1 关联 | C-035 | 待实现 |
| FR-072 | Plugin WASM 运行时 | P1-8 | C-036 | 待实现 |
| FR-073 | 凭证加密存储 | P1-9 | C-026 §6 + C-028 §4b | 待实现 |

### 4.3 P2 - 增强功能

> FR-021 ~ FR-031, FR-040 ~ FR-052, FR-057 ~ FR-062 继承自 v5，内容不变。

| FR 编号 | 需求名称 | 覆盖差距 | 状态 |
|--------|----------|----------|------|
| FR-074 | Event Bus 事件类型完整性 | P2 | 待实现 |
| FR-075 | Share 服务层 | P2-1 | 待实现 |
| FR-076 | SDK 输出 (Rust + TypeScript) | P2 | 待实现 |
| FR-077 | OpenAPI 文档自动生成 | P2-2 | 待实现 |
| FR-078 | LSP definition/references/hover | P2-3 | 待实现 |
| FR-079 | session_load/session_save 工具 | P2-4 | 待实现 |
| FR-080 | HuggingFace + AI21 Provider | P2-10 | 待实现 |
| FR-081 | Formatters 接入 agent 执行循环 | P2-5 | 待实现 |
| FR-082 | Compaction 自动触发阈值 | P2-6 | 待实现 |
| FR-083 | TUI 虚拟滚动 | P2 | 待实现 |
| FR-084 | Server HTTP Basic Auth | P2-7 | 待实现 |
| FR-085 | 观测性 (tracing/crash recovery/token cost) | P2-8 | 待实现 |

### 4.4 P3 - 远期规划

| FR 编号 | 需求名称 | 状态 |
|--------|----------|------|
| FR-087 | GitHub Integration (v2) | 规划中 |
| FR-088 | Enterprise 配置 (Central Config + SSO) | 规划中 |

---

## 5. 技术债务清单

*(同 v6，内容不变)*

| 债务项 | 位置 | 描述 | 关联 FR | 风险 |
|--------|------|------|---------|------|
| **ProviderConfig 平面结构** | `crates/llm/src/provider.rs` | 使用 `{model, api_key, temperature}` 平面结构，与 PRD 要求的 4 层认证架构严重不符 | FR-063 | 高 - 重构成本高 |
| **Provider 实现重复** | `crates/llm/src/*.rs` | 18 个 provider 实现可能存在大量重复代码，缺少统一的 OpenAI-compatible adapter 复用 | FR-063 | 中 - 维护成本高 |
| **Config 结构体过大** | `crates/core/src/config.rs` | 配置结构体字段极多 (1000+ 行)，merge 逻辑复杂，变量替换逻辑耦合在 Config impl 内 | FR-021 | 中 - 可测试性差 |
| **Auth 模块孤岛** | `crates/auth/` | auth crate 存在但未见被 llm/core 广泛引用，可能存在未连接的模块 | FR-063, FR-064 | 高 - 功能孤岛 |
| **TUI 测试覆盖** | `crates/tui/` | 未见 TUI 组件测试文件，PRD-tui.md 要求单元/组件/集成/快照四类测试 | FR-069, FR-070 | 高 - 回归风险 |
| **总测试文件仅 8 个** | 全项目 | 对 15 crates 的大型项目而言测试覆盖严重不足 | 全局 | 高 - 质量风险 |

---

## 6. 验收标准对照 (PRD §10)

| 验收项 | PRD § | 状态 | 关联 FR | 备注 |
|--------|-------|------|---------|------|
| JSON/JSONC 格式支持 | 10.1 | ✅ | - | `jsonc.rs` 完整实现 |
| **Provider 认证分层** | **§7.x.5** | **❌** | **FR-063** | **架构级缺陷** |
| **OAuth/Device Code** | **§7.x.5.B** | **❌** | **FR-064** | **完全缺失** |
| **云厂商原生认证** | **§7.x.5.C** | **❌** | **FR-065** | **完全缺失** |
| **Remote Config 自动发现** | **§1** | **❌** | **FR-066** | **完全缺失** |
| **MCP OAuth 独立存储** | **§7.x.5.D** | **❌** | **FR-068** | **完全缺失** |
| **TUI 三栏布局** | **§7.15.3** | **❌** | **FR-069** | **完全缺失** |
| **TUI 状态机** | **§7.15.7** | **❌** | **FR-070** | **完全缺失** |
| **Context Engine 分层** | **§6** | **❌** | **FR-071** | **完全缺失** |
| **Plugin WASM 运行时** | **§8** | **❌** | **FR-072** | **完全缺失** |
| **凭证加密存储** | **安全要求** | **❌** | **FR-073** | **完全缺失** |

---

## 7. 功能需求清单汇总

### 7.1 按模块分组

| 模块 | FR 编号 | 需求名称 | 优先级 | 覆盖差距 |
|------|---------|----------|--------|----------|
| core | FR-001 | Context Engine | P0 | - |
| core | FR-003 | Skills 系统 | P0 | - |
| core | FR-004 | Commands 系统 | P0 | P2-9 |
| core | FR-012 | Share 功能 | P1 | - |
| core | FR-014 | 插件事件总线 | P1 | - |
| core | FR-022 | Session Summarize | P2 | - |
| core | FR-051 | Compaction 会话压缩 | P2 | - |
| core | FR-052 | 文件 Watcher 配置 | P2 | - |
| core | FR-057 | Event Bus 事件总线 | P2 | - |
| core | FR-058 | Effect System 效果系统 | P2 | - |
| core | FR-071 | Context Engine 分层上下文 (L0-L4) | P1 | P1 关联 |
| core | FR-074 | Event Bus 事件类型完整性 | P2 | P2 |
| core | FR-075 | Share 服务层 | P2 | P2-1 |
| core | FR-082 | Compaction 自动触发阈值 | P2 | P2-6 |
| core | FR-085 | 观测性 (tracing/crash recovery/token cost) | P2 | P2-8 |
| core | FR-088 | Enterprise 配置 (Central Config + SSO) | P3 | - |
| core/config | FR-008 | 多层配置合并 | P0 | - |
| core/config | FR-009 | .opencode 目录加载 | P0 | - |
| core/config | FR-010 | Provider 环境变量约定 | P0 | - |
| core/config | FR-021 | 配置系统完善 | P2 | - |
| core/config | FR-030 | 废弃字段清理 | P2 | - |
| core/config | FR-033 | OPENCODE_TUI_CONFIG 环境变量 | P0 | - |
| core/config | FR-034 | TUI 配置分离为独立文件 | P0 | - |
| core/config | FR-035 | modes/ 目录扫描 | P1 | - |
| core/config | FR-036 | 配置路径命名统一 | P1 | - |
| core/config | FR-037 | {file:path} ~ 路径展开 | P1 | - |
| core/config | FR-038 | {file:path} 相对路径支持 | P1 | - |
| core/config | FR-039 | .opencode/ 目录扫描集成 | P1 | - |
| core/config | FR-040 | 变量替换覆盖完整性 | P2 | - |
| core/config | FR-041 | theme/keybinds 迁移到 TUI | P2 | - |
| core/config | FR-042 | AgentMapConfig 动态 HashMap | P2 | - |
| core/config | FR-043 | JSON Schema 远程验证 | P2 | - |
| core/config | FR-054 | Provider 控制 (disabled/enabled) | P1 | - |
| core/config | FR-055 | Formatters 自动格式化 | P1 | - |
| core/config | FR-056 | Instructions 指令文件加载 | P1 | - |
| core/config | FR-066 | Remote Config 自动发现 | P1 | P1-3 |
| core/config | FR-067 | disabled_providers 优先级 | P1 | P1-4 |
| core/formatter | FR-055 | Formatters 自动格式化 | P1 | - |
| core/formatter | FR-081 | Formatters 接入 agent 执行循环 | P2 | P2-5 |
| core/share | FR-075 | Share 服务层 | P2 | P2-1 |
| core/tools | FR-044 | session_load/session_save | P1 | - |
| core/tools | FR-053 | Tools 配置禁用机制 | P1 | - |
| core/tools | FR-079 | session_load/session_save 工具 | P2 | P2-4 |
| llm | FR-049 | HuggingFace/AI21 Provider | P2 | - |
| llm | FR-063 | Provider 认证协议分层抽象 | P0 | P0-2 |
| llm | FR-065 | 云厂商原生认证 | P1 | P1-7 |
| llm | FR-080 | HuggingFace + AI21 Provider 完整实现 | P2 | P2-10 |
| auth | FR-015 | 凭证加密存储 | P1 | - |
| auth | FR-029 | OAuth 登录预留 | P2 | - |
| auth | FR-047 | OAuth 登录支持 | P1 | - |
| auth | FR-064 | OAuth/Device Code 浏览器登录流程 | P0 | P0-1 |
| auth | FR-073 | 凭证加密存储 (完善) | P1 | P1-9 |
| tui | FR-017 | TUI Token/Cost 显示 | P1 | - |
| tui | FR-023 | TUI 布局切换 | P2 | - |
| tui | FR-024 | TUI 右栏功能完善 | P2 | - |
| tui | FR-025 | TUI Patch 预览展开 | P2 | - |
| tui | FR-026 | Web UI | P2 | - |
| tui | FR-069 | TUI 三栏布局与 Inspector 面板 | P1 | P1-5 |
| tui | FR-070 | TUI 状态机完整实现 | P1 | P1 关联 |
| tui | FR-083 | TUI 虚拟滚动 | P2 | P2 |
| plugin | FR-002 | Plugin System | P0 | - |
| plugin | FR-072 | Plugin WASM 运行时 | P1 | P1-8 |
| mcp | FR-005 | MCP 工具接入 | P0 | - |
| mcp | FR-068 | MCP OAuth 独立 token store | P1 | P1-6 |
| server | FR-006 | TUI 快捷输入解析器 | P0 | - |
| server | FR-007 | Session Fork | P0 | - |
| server | FR-011 | Server API 完善 | P1 | - |
| server | FR-050 | Server mDNS 服务发现 | P2 | - |
| server | FR-059 | Streaming 消息架构 | P2 | - |
| server | FR-076 | SDK 输出 (Rust + TypeScript) | P2 | - |
| server | FR-077 | OpenAPI 文档自动生成 | P2 | P2-2 |
| server | FR-084 | Server 认证 (HTTP Basic Auth) | P2 | P2-7 |
| storage | FR-032 | Snapshot 元数据完善 | P1 | - |
| storage/permission | FR-016 | Permission 审计记录 | P1 | - |
| lsp | FR-013 | LSP 功能增强 | P1 | - |
| lsp | FR-078 | LSP definition/references/hover/code actions | P2 | P2-3 |
| git | FR-028 | GitHub 集成预留 | P2 | - |
| git | FR-048 | GitHub 集成 | P1 | - |
| git | FR-087 | GitHub Integration (v2) | P3 | - |
| control-plane | FR-060 | Control Plane / ACP 协议 | P2 | - |
| cli | FR-061 | CLI 命令架构完善 | P2 | - |
| schema | FR-018 | TUI Schema 验证 | P1 | - |
| - | FR-027 | IDE 扩展预留 | P2 | - |

### 7.2 按优先级分组

| 优先级 | FR 编号 | 覆盖差距 |
|--------|---------|----------|
| P0 | FR-001, FR-002, FR-003, FR-004, FR-005, FR-006, FR-007, FR-008, FR-009, FR-010, FR-033, FR-034, **FR-063 (P0-2), FR-064 (P0-1)** | 2 |
| P1 | FR-011, FR-012, FR-013, FR-014, FR-015, FR-016, FR-017, FR-018, FR-019, FR-020, FR-032, FR-035, FR-036, FR-037, FR-038, FR-039, FR-044, FR-045, FR-046, FR-047, FR-048, FR-053, FR-054, FR-055, FR-056, **FR-065 (P1-7), FR-066 (P1-3), FR-067 (P1-4), FR-068 (P1-6), FR-069 (P1-5), FR-070, FR-071, FR-072 (P1-8), FR-073 (P1-9)** | 9 |
| P2 | FR-021, FR-022, FR-023, FR-024, FR-025, FR-026, FR-027, FR-028, FR-029, FR-030, FR-031, FR-040, FR-041, FR-042, FR-043, FR-049, FR-050, FR-051, FR-052, FR-057, FR-058, FR-059, FR-060, FR-061, FR-062, **FR-074, FR-075 (P2-1), FR-076, FR-077 (P2-2), FR-078 (P2-3), FR-079 (P2-4), FR-080 (P2-10), FR-081 (P2-5), FR-082 (P2-6), FR-083, FR-084 (P2-7), FR-085 (P2-8)** | 10 |
| P3 | **FR-087, FR-088** | - |

---

## 8. 实施建议

### Phase 1: P0 阻断性问题 (最高优先级)

1. **FR-063 Provider 认证协议分层抽象** - 架构级重构，必须在其他认证相关需求之前完成
2. **FR-064 OAuth/Device Code 浏览器登录流程** - 依赖 FR-063 的分层架构

### Phase 2: P1 核心功能

1. **FR-065 云厂商原生认证** - 依赖 FR-063
2. **FR-066 Remote Config 自动发现** - 企业部署
3. **FR-067 disabled_providers 优先级** - 配置冲突处理
4. **FR-068 MCP OAuth 独立存储** - 依赖 FR-063
5. **FR-069 TUI 三栏布局与 Inspector 面板** - UX 核心
6. **FR-070 TUI 状态机完整实现** - 状态流转
7. **FR-071 Context Engine 分层上下文** - 依赖 FR-001
8. **FR-072 Plugin WASM 运行时** - 依赖 FR-002
9. **FR-073 凭证加密存储** - 安全合规

### Phase 3: P2 完善性

1. **FR-074 Event Bus 事件类型完整性**
2. **FR-075 Share 服务层**
3. **FR-076 SDK 输出**
4. **FR-077 OpenAPI 文档自动生成**
5. **FR-078 LSP 扩展**
6. **FR-079 session_load/session_save 工具**
7. **FR-080 HuggingFace + AI21 Provider**
8. **FR-081 Formatters 接入 agent 执行循环**
9. **FR-082 Compaction 自动触发阈值**
10. **FR-083 TUI 虚拟滚动**
11. **FR-084 Server HTTP Basic Auth**
12. **FR-085 观测性**

### Phase 4: P3 远期规划

1. **FR-087 GitHub Integration (v2)**
2. **FR-088 Enterprise 配置 (Central Config + SSO)**

---

## 9. 附录

### A. Constitution 条款映射 (v1.7)

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
| C-020 | Server 配置规范 | FR-050 |
| C-021 | Compaction 配置规范 | FR-051, FR-082 |
| C-022 | Watcher 配置规范 | FR-052 |
| C-023 | Agent 系统规范 | FR-045, FR-046 |
| C-024 | Permission 系统规范 | FR-016 |
| C-025 | Plugin 系统规范 | FR-002, FR-014, FR-072, FR-074 |
| C-026 (v1.6 重写) | Auth 系统规范 | FR-015, FR-047, FR-063, FR-064, FR-065, FR-073, FR-084 |
| C-027 | Share 系统规范 | FR-012, FR-075 |
| C-028 | Storage 系统规范 | FR-032 |
| C-029 | Tools 配置规范 | FR-053 |
| C-030 (v1.6 修订) | Provider 控制规范 | FR-054, FR-065, FR-067 |
| C-031 | Formatters 规范 | FR-055, FR-081 |
| C-032 | Instructions 规范 | FR-056 |
| C-033 | MCP OAuth 独立存储 | FR-068 |
| C-034 | TUI 布局与状态机 | FR-069, FR-070, FR-083 |
| C-035 | Context Engine 分层 | FR-071, FR-082 |
| C-036 | Plugin WASM 运行时 | FR-072 |
| C-037 | Remote Config 自动发现 | FR-066 |

### B. 追溯链确认

```
gap-analysis.md (iteration-7)
    │
    ├── P0: 2 项 → FR-063, FR-064
    ├── P1: 9 项 → FR-065 ~ FR-073
    └── P2: 10 项 → FR-074 ~ FR-085, FR-004
            │
            ▼
constitution_updates.md (v1.7)
    │
    ├── P0 覆盖率: 100% (2/2)
    ├── P1 覆盖率: 100% (9/9)
    └── P2 覆盖率: 100% (10/10)
            │
            ▼
spec_v7.md
    │
    └── FR-001 ~ FR-088 (88 项)
```

### C. v6 → v7 变更摘要

| 变更项 | 说明 |
|--------|------|
| 新增 FR | **无** (v7 为审计确认版) |
| 更新 §1 | 新增 iteration-7 差距分析参考 |
| 更新 §3 | 新增差距分析→FR 追溯链 (P0/P1/P2 完整映射) |
| 更新 §4 | 新增 FR 状态汇总表 (覆盖差距列) |
| 更新 §7 | 补充按优先级分组的覆盖差距列 |
| 更新 §8.A | Constitution 条款映射 (C-001~C-037) |
| 更新 §8.B | 新增追溯链确认图 |
| 更新 §9 | v6 → v7 变更摘要 |

---

**文档状态**: 草稿
**下一步**: 基于本规格文档创建 iteration-8 实施计划

