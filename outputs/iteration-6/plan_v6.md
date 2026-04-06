# OpenCode-RS 实施计划 v6

**版本**: 6.0
**日期**: 2026-04-05
**基于**: spec_v6.md 拆分 — 计划与架构部分
**状态**: 已完成

---

## 1. 文档概述

### 1.1 背景

本计划文档从 spec_v6.md 拆分而来，聚焦于实施架构、阶段规划与状态追踪。
详细的单个功能需求规格 (FR-001 ~ FR-088) 请参见 `tasks_v6.md`。

本规格文档基于以下文档综合生成：
- **spec_v5.md**: 上一版规格文档 (FR-001 ~ FR-062)
- **PRD-OpenCode-Configuration.md**: 配置系统产品需求文档 (v1.0)
- **docs/PRD.md**: 产品需求文档 v1.1 (完整系统级 PRD)
- **docs/PRD-providers.md**: Provider 与认证协议详细规格 (75+ providers, 5 类认证层)
- **docs/PRD-tui.md**: TUI 产品需求 (三栏布局/10 种状态机/Inspector 面板/虚拟滚动)
- **outputs/iteration-6/constitution_updates.md**: Constitution v1.6 更新 (C-026 重写, C-030 修订, C-033~C-037 新增)
- **outputs/iteration-6/gap-analysis.md**: 差距分析报告 (25 项差距, 75-80% 完成度)

### 1.2 目标

- 基于差距分析新发现的 25 项差距，新增 26 项功能需求 (FR-063 ~ FR-088)
- 将 PRD-providers.md 的 4 层认证架构纳入实施计划
- 将 PRD-tui.md 的三栏布局/10 种状态机/Inspector 面板纳入实施计划
- 将 Constitution v1.6 的新条款映射到功能需求
- 确保 P0 任务 (FR-063, FR-064) 优先于其他所有任务

### 1.3 参考文档

| 文档 | 路径 | 说明 |
|------|------|------|
| PRD-主文档 | `docs/PRD.md` | 产品需求文档 v1.1 |
| PRD-Providers | `docs/PRD-providers.md` | Provider 与认证协议详细规格 |
| PRD-TUI | `docs/PRD-tui.md` | TUI 产品需求详细设计 |
| PRD-配置系统 | `PRD-OpenCode-Configuration.md` | 配置系统产品需求 |
| Constitution v1.6 | `outputs/iteration-6/constitution_updates.md` | 设计约束条款 (C-001 ~ C-037) |
| spec_v6 | `outputs/iteration-6/spec_v6.md` | 当前版规格文档 |
| tasks_v6 | `outputs/iteration-6/tasks_v6.md` | 功能需求详细规格 |
| 差距分析 | `outputs/iteration-6/gap-analysis.md` | 差距分析报告 (2026-04-05) |

### 1.4 与 v5 的关系

v6 保留 v5 的所有需求 (FR-001 ~ FR-062)，并新增：
- **FR-063 ~ FR-064**: P0 认证架构缺陷 (Provider 认证分层 / OAuth/Device Code)
- **FR-065 ~ FR-073**: P1 重要功能缺失 (云厂商认证 / Remote Config / MCP OAuth / TUI 三栏 / TUI 状态机 / Context Engine 分层 / WASM 运行时 / 凭证加密 / Compaction 阈值)
- **FR-074 ~ FR-085**: P2 增强功能 (Event Bus 完整 / Share 服务 / SDK 输出 / OpenAPI 文档 / LSP 扩展 / session_load/save / HF+AI21 / Formatters 接入 / 虚拟滚动 / Server Basic Auth / 观测性)
- **FR-087 ~ FR-088**: P3 远期规划 (GitHub Integration / Enterprise 配置)

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

## 3. 技术债务清单

| 债务项 | 位置 | 描述 | 关联 FR |
|--------|------|------|---------|
| **ProviderConfig 平面结构** | `crates/llm/src/provider.rs` | 使用 `{model, api_key, temperature}` 平面结构，与 PRD 要求的 4 层认证架构严重不符 | FR-063 |
| **Provider 实现重复** | `crates/llm/src/*.rs` | 18 个 provider 实现可能存在大量重复代码，缺少统一的 OpenAI-compatible adapter 复用 | FR-063 |
| **Config 结构体过大** | `crates/core/src/config.rs` | 配置结构体字段极多 (1000+ 行)，merge 逻辑复杂，变量替换逻辑耦合在 Config impl 内 | FR-021 |
| **TOML vs JSON 格式分裂** | `config.rs:1012-1031` | `config_path()` 默认返回 `.toml`，但 PRD 要求 JSON/JSONC | FR-036 |
| **硬编码路径** | `config.rs:1031` | `"~/.config/opencode-rs/config.toml"` 硬编码 | FR-036 |
| **变量替换实现粗糙** | `config.rs:972-1009` | 字符串替换对嵌套/复杂情况可能出错 | FR-040 |
| **merge_configs 通过 JSON 中转** | `merge.rs:22-29` | 序列化→deep_merge→反序列化，丢失类型信息 | FR-021 |
| **fetch_remote_config 同步包装异步** | `config.rs:1107-1109` | 同步函数中创建 tokio runtime | FR-066 |
| **Auth 模块孤岛** | `crates/auth/` | auth crate 存在但未见被 llm/core 广泛引用，可能存在未连接的模块 | FR-063, FR-064 |
| **浏览器认证单独实现** | `crates/core/src/openai_browser_auth.rs` | 浏览器认证单独实现而非复用 auth crate | FR-064 |
| **TUI 测试覆盖** | `crates/tui/` | 未见 TUI 组件测试文件，PRD-tui.md 要求单元/组件/集成/快照四类测试 | FR-069, FR-070 |
| **总测试文件仅 8 个** | 全项目 | 对 15 crates 的大型项目而言测试覆盖严重不足 | 全局 |
| **invalid.rs 残留** | `crates/tools/src/invalid.rs` | 存在名为 "invalid" 的工具文件，可能是残留代码 | - |
| **测试文件位置不当** | `crates/tools/src/grep_tool_test.rs` | 测试文件放在 src/ 而非 tests/ 目录 | - |
| **Schema 验证空壳** | `schema.rs:5-40` | 只检查 2 个字段 | FR-043 |
| **DirectoryScanner 未用 glob** | `directory_scanner.rs` | 手动 read_dir，不支持 glob 模式 | FR-035, FR-056 |
| **Workspace version 0.1.0** | `Cargo.toml` | 与 PRD 版本规划 (v0.1 MVP → v0.2 平台化 → v0.3 扩展化 → v1.0 生产可用) 未对齐 | - |
| **log_schema_validation 仅警告** | `crates/core/src/config.rs` | 仅警告不阻断，配置错误可能被静默忽略 | FR-043 |

---

## 4. 验收标准对照 (PRD §10)

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
| disabled_providers 优先级 | 10.4 | ⚠️ | FR-067 | 基础存在，需完善优先级逻辑 |
| 自定义 agent 配置 | 10.5 | ✅ | FR-042 | AgentConfig 完整，但 AgentMapConfig 需改为动态 |
| default_agent 设置 | 10.5 | ✅ | - | 字段存在且被 env 覆盖 |
| 命令模板变量替换 | 10.5 | ⚠️ | FR-004 | 命令模板变量替换未明确实现 |
| permission 配置 | 10.6 | ✅ | - | `PermissionConfig` 完整 |
| API Key 文件引用 | 10.6 | ⚠️ | FR-037, FR-038 | 依赖 `{file:path}`，但该功能不完整 |
| **Tools 配置禁用** | **§5.2** | **❌** | **FR-053** | **完全缺失** |
| **Formatters 配置** | **§5.7** | **❌** | **FR-055** | **完全缺失** |
| **Instructions 配置** | **§5.13** | **❌** | **FR-056** | **完全缺失** |
| **Provider 控制** | **§5.14** | **⚠️** | **FR-054** | **基础存在，需完善** |
| **Provider 认证分层** | **§7.x.5** | **❌** | **FR-063** | **完全缺失，架构级缺陷** |
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

## 5. 实施阶段规划

### Phase 1: P0 阻断性问题 (最高优先级)

> **FR-063 和 FR-064 必须在其他认证相关需求之前完成**，因为它们定义了认证架构的基础。

1. **FR-063 Provider 认证协议分层抽象** - 架构级重构，必须在其他认证相关需求之前完成
2. **FR-064 OAuth/Device Code 浏览器登录流程** - 依赖 FR-063 的分层架构
3. **FR-033 OPENCODE_TUI_CONFIG 环境变量** - 配置系统基础
4. **FR-034 TUI 配置分离** - 核心架构要求
5. **FR-001 Context Engine** - 核心依赖
6. **FR-005 MCP 工具接入** - 工具系统扩展
7. **FR-004 Commands 系统** - TUI 输入增强
8. **FR-006 TUI 快捷输入解析器** - 核心交互
9. **FR-003 Skills 系统** - 上下文增强
10. **FR-002 Plugin System** - 扩展性基础
11. **FR-007 Session Fork** - 会话分叉
12. **FR-008 多层配置合并** - 配置管理
13. **FR-009 .opencode 目录加载** - 模块化配置支持
14. **FR-010 Provider 环境变量约定** - 环境变量绑定

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
10. **FR-039 .opencode/ 目录扫描集成** - 配置加载完整性
11. **FR-037 {file:path} ~ 路径展开** - 变量替换完整性
12. **FR-038 {file:path} 相对路径支持** - 变量替换完整性
13. **FR-035 modes/ 目录扫描** - 目录结构完整性
14. **FR-036 配置路径命名统一** - 生态兼容性
15. **FR-044 session_load/session_save** - 会话持久化
16. **FR-045 剩余内建 Skills 补全** - 能力扩展
17. **FR-046 剩余 Commands 补全** - 命令完整性
18. **FR-011 Server API** - API 完整性
19. **FR-013 LSP 功能增强** - 开发体验
20. **FR-012 Share 功能** - 协作能力
21. **FR-015 凭证加密存储** - 安全合规
22. **FR-014 插件事件总线** - 事件系统
23. **FR-016 Permission 审计记录** - 权限追踪
24. **FR-017 TUI Token/Cost 显示** - 成本感知
25. **FR-018 TUI Schema 验证** - 配置验证增强
26. **FR-019 scroll_acceleration 结构修复** - 类型修正
27. **FR-020 keybinds 自定义绑定** - 绑定扩展
28. **FR-032 Snapshot 元数据完善** - 数据完整性
29. **FR-047 OAuth 登录支持** - 用户认证 (v1.5+)
30. **FR-048 GitHub 集成** - DevOps 集成 (v1.5+)
31. **FR-053 Tools 配置禁用机制** - 工具控制
32. **FR-054 Provider 控制** - Provider 管理
33. **FR-055 Formatters 自动格式化** - 代码格式化
34. **FR-056 Instructions 指令文件加载** - 上下文注入

### Phase 3: P2 完善性

1. **FR-074 Event Bus 事件类型完整性** - 事件通信
2. **FR-075 Share 服务层** - 协作能力
3. **FR-076 SDK 输出** - 开发者体验
4. **FR-077 OpenAPI 文档自动生成** - 文档
5. **FR-078 LSP 扩展** - 开发体验
6. **FR-079 session_load/session_save 工具** - 会话管理
7. **FR-080 HuggingFace + AI21 Provider** - LLM 覆盖
8. **FR-081 Formatters 接入 agent 执行循环** - 格式化
9. **FR-082 Compaction 自动触发阈值** - 上下文管理
10. **FR-083 TUI 虚拟滚动** - 性能
11. **FR-084 Server HTTP Basic Auth** - 安全
12. **FR-085 观测性** - 可观测性
13. **FR-040 变量替换覆盖完整性** - 配置系统完善
14. **FR-041 theme/keybinds 迁移** - 废弃声明一致性
15. **FR-042 AgentMapConfig 动态 HashMap** - 灵活性
16. **FR-043 JSON Schema 远程验证** - 配置校验
17. **FR-049 HuggingFace/AI21 Provider** - LLM 覆盖完整性
18. **FR-050 Server mDNS 服务发现** - 局域网发现
19. **FR-051 Compaction 会话压缩** - 上下文管理
20. **FR-052 文件 Watcher 配置** - 文件监视
21. **FR-057 Event Bus 事件总线** - 事件通信
22. **FR-058 Effect System 效果系统** - 副作用管理
23. **FR-059 Streaming 消息架构** - 流式消息标准化
24. **FR-060 Control Plane / ACP 协议** - Agent 通信
25. **FR-061 CLI 命令架构完善** - CLI 架构
26. **FR-062 Remote Config 安全验证** - 远程配置安全
27. **FR-021 配置系统** - 配置灵活性
28. **FR-022 Session Summarize** - 会话管理
29. **FR-023 TUI 布局切换** - UI 增强
30. **FR-024 TUI 右栏功能完善** - 面板功能
31. **FR-025 TUI Patch 预览展开** - Diff 交互
32. **FR-026 Web UI** - 多端支持
33. **FR-027 IDE 扩展预留** - 生态扩展
34. **FR-028 GitHub 集成预留** - DevOps 集成
35. **FR-029 OAuth 登录预留** - 认证扩展
36. **FR-030 废弃字段清理** - 代码清理
37. **FR-031 theme 路径解析增强** - 主题功能增强

### Phase 4: P3 远期规划

1. **FR-087 GitHub Integration (v2)** - DevOps 集成
2. **FR-088 Enterprise 配置 (Central Config + SSO)** - 企业版

---

## 6. 配置系统状态

| 配置项 | 实现状态 | 关联 FR | 备注 |
|--------|----------|---------|------|
| JSON/JSONC 格式 | ✅ 完整 | - | jsonc.rs |
| 配置合并 | ✅ 完整 | FR-021 | merge.rs |
| Remote Config | ❌ 未实现 | FR-066, FR-062 | 自动发现机制完全缺失 |
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
| MCP 配置 | ⚠️ 部分 | FR-005, FR-068 | OAuth 独立存储缺失 |
| theme 配置 | ⚠️ 部分 | FR-031, FR-041 | 未迁移到 tui.json |
| keybinds 配置 | ⚠️ 部分 | FR-020, FR-041 | 未迁移到 tui.json |
| Server 配置 (mDNS/CORS) | ⚠️ 部分 | FR-050 | 基础实现存在，mDNS 待完善 |
| Compaction 配置 | ⚠️ 部分 | FR-051, FR-082 | 自动触发阈值缺失 |
| Watcher 配置 | ⚠️ 部分 | FR-052 | 基础监视存在，ignore 配置待完善 |
| Tools 配置 | ❌ 未实现 | FR-053 | 完全缺失 |
| Formatters 配置 | ❌ 未实现 | FR-055, FR-081 | 未接入 agent 执行循环 |
| Instructions 配置 | ❌ 未实现 | FR-056 | 完全缺失 |
| disabled_providers | ⚠️ 部分 | FR-054, FR-067 | 优先级逻辑需完善 |
| **Provider 认证分层** | **❌ 未实现** | **FR-063** | **架构级缺陷** |
| **OAuth/Device Code** | **❌ 未实现** | **FR-064** | **完全缺失** |
| **云厂商认证** | **❌ 未实现** | **FR-065** | **完全缺失** |

---

## 7. Constitution 条款映射

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
| **C-026 (v1.6 重写)** | **Auth 系统规范** | **FR-015, FR-047, FR-063, FR-064, FR-065, FR-073, FR-084** |
| C-027 | Share 系统规范 | FR-012, FR-075 |
| C-028 | Storage 系统规范 | FR-032 |
| C-029 | Tools 配置规范 | FR-053 |
| **C-030 (v1.6 修订)** | **Provider 控制规范** | **FR-054, FR-065, FR-067** |
| C-031 | Formatters 规范 | FR-055, FR-081 |
| C-032 | Instructions 规范 | FR-056 |
| **C-033** | **MCP OAuth 独立存储** | **FR-068** |
| **C-034** | **TUI 布局与状态机** | **FR-069, FR-070, FR-083** |
| **C-035** | **Context Engine 分层** | **FR-071, FR-082** |
| **C-036** | **Plugin WASM 运行时** | **FR-072** |
| **C-037** | **Remote Config 自动发现** | **FR-066** |

---

## 8. v5 → v6 变更摘要

| 变更项 | 说明 |
|--------|------|
| 新增 FR-063 | Provider 认证协议分层抽象 (P0) — 架构级重构 |
| 新增 FR-064 | OAuth/Device Code 浏览器登录流程 (P0) |
| 新增 FR-065 | 云厂商原生认证 (P1) |
| 新增 FR-066 | Remote Config 自动发现 (P1) |
| 新增 FR-067 | disabled_providers 优先级 (P1) |
| 新增 FR-068 | MCP OAuth 独立 token store (P1) |
| 新增 FR-069 | TUI 三栏布局与 Inspector 面板 (P1) |
| 新增 FR-070 | TUI 状态机完整实现 (P1) |
| 新增 FR-071 | Context Engine 分层上下文 L0-L4 (P1) |
| 新增 FR-072 | Plugin WASM 运行时 (P1) |
| 新增 FR-073 | 凭证加密存储完善 (P1) |
| 新增 FR-074 | Event Bus 事件类型完整性 (P2) |
| 新增 FR-075 | Share 服务层 (P2) |
| 新增 FR-076 | SDK 输出 (Rust + TypeScript) (P2) |
| 新增 FR-077 | OpenAPI 文档自动生成 (P2) |
| 新增 FR-078 | LSP definition/references/hover/code actions (P2) |
| 新增 FR-079 | session_load/session_save 工具 (P2) |
| 新增 FR-080 | HuggingFace + AI21 Provider 完整实现 (P2) |
| 新增 FR-081 | Formatters 接入 agent 执行循环 (P2) |
| 新增 FR-082 | Compaction 自动触发阈值 (P2) |
| 新增 FR-083 | TUI 虚拟滚动 (P2) |
| 新增 FR-084 | Server 认证 HTTP Basic Auth (P2) |
| 新增 FR-085 | 观测性 tracing/crash recovery/token cost (P2) |
| 新增 FR-087 | GitHub Integration v2 (P3) |
| 新增 FR-088 | Enterprise 配置 Central Config + SSO (P3) |
| 更新 §2 | 需求总览 (P0: 12→14, P1: 25→34, P2: 26→37, P3: 0→2) |
| 更新 §3 | 技术债务清单 (扩展至 18 项) |
| 更新 §4 | 验收标准对照 (新增 12 项 PRD 配置项) |
| 更新 §5 | 实施阶段规划 (Phase 1 新增 FR-063/064, Phase 2 新增 9 项, Phase 3 新增 12 项, Phase 4 新增 2 项) |
| 更新 §6 | 配置系统状态 (新增认证分层/OAuth/云厂商认证) |
| 更新 §7 | Constitution 条款映射 (C-026 重写, C-030 修订, C-033~C-037 新增) |

---

**文档状态**: 草稿
**下一步**: 基于本计划文档 + tasks_v6.md 创建迭代 6 实施计划
