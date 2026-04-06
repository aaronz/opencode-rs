# OpenCode-RS 规格文档 v12

**版本**: 12.0
**日期**: 2026-04-06
**基于**: spec_v10.md + 2026-03-31 openspec 完成后的最终差距
**状态**: 草稿

---

## 1. 文档概述

### 1.1 背景

iteration-10 已完成所有 P2/P3 任务。2026-03-31 的 openspec 变更实现了 WebSocket/SSE 流式、MCP 协议、TUI 输入语法和 LSP 诊断。当前代码库达到 ~85-90% 完整度，15 个 crate 均有实质性实现和测试覆盖。

本规格文档定义 iteration-12 的最终 v1.0 目标范围。

### 1.2 范围

| 类别 | 包含 | 排除 |
|------|------|------|
| **包含** | 集成测试框架、HuggingFace/AI21 Provider、Session 工具、OAuth Browser 登录完善、代码质量与性能优化 | GitHub 深度集成 (v1.5)、IDE 插件、Desktop Shell |
| **优先级** | P1 (集成测试) + P2 (Provider 补全 + 工具完善) | P3 (GitHub Integration v2) 推迟 |

### 1.3 与 v10 的关系

v10 保留了 FR-001 ~ FR-109 的所有需求。

v12 新增需求：
- 1 项 P1 (集成测试框架) → FR-110
- 2 项 P2 (Provider 补全 + Session 工具) → FR-111, FR-112
- 2 项 P2 (代码质量 + 性能基准) → FR-113, FR-114

---

## 2. 功能需求

### FR-110: 集成测试框架

**描述**: 建立端到端集成测试，验证 crate 间协作

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-110.1 | 集成测试基础设施 (TempProject + MockServer) | P1 |
| FR-110.2 | Agent ↔ LLM Provider 集成测试 | P1 |
| FR-110.3 | Agent ↔ Tool 集成测试 | P1 |
| FR-110.4 | Server ↔ Session ↔ Storage 集成测试 | P1 |
| FR-110.5 | TUI ↔ Server WebSocket 集成测试 | P2 |
| FR-110.6 | MCP 外部工具集成测试 | P2 |

### FR-111: HuggingFace + AI21 Provider 完整实现

**描述**: 补全剩余 2/18 LLM Provider

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-111.1 | HuggingFace Inference API Provider | P2 |
| FR-111.2 | HuggingFace 流式响应支持 | P2 |
| FR-111.3 | AI21 Studio API Provider | P2 |
| FR-111.4 | AI21 工具调用支持 | P2 |

### FR-112: Session Load/Save 工具

**描述**: 实现会话导入导出工具

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-112.1 | session_load 工具实现 | P2 |
| FR-112.2 | session_save 工具实现 | P2 |
| FR-112.3 | 导出格式 (JSON/Markdown) | P2 |
| FR-112.4 | 导出时敏感信息脱敏 | P2 |

### FR-113: 代码质量完善

**描述**: 提升代码可维护性和安全性

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-113.1 | clippy 零警告 | P2 |
| FR-113.2 | 文档注释覆盖率 ≥ 80% | P2 |
| FR-113.3 | 错误消息用户友好化 | P2 |

### FR-114: 性能基准测试

**描述**: 建立性能基准确保 v1.0 质量

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-114.1 | LLM 请求延迟基准 | P2 |
| FR-114.2 | 工具执行延迟基准 | P2 |
| FR-114.3 | Session 加载时间基准 | P2 |
| FR-114.4 | TUI 渲染帧率基准 | P2 |

---

## 3. 非功能需求

### 3.1 性能

| 指标 | 目标 |
|------|------|
| API 响应时间 (p95) | < 200ms |
| Session 加载时间 | < 500ms |
| Tool 执行延迟 | < 50ms |
| TUI 渲染帧率 | ≥ 30fps |

### 3.2 质量

| 要求 | 描述 |
|------|------|
| clippy 警告 | 0 |
| 文档覆盖率 | ≥ 80% |
| 集成测试覆盖 | 核心路径 100% |
| 单元测试覆盖率 | ≥ 70% |

---

## 4. 当前实现状态总览

### 4.1 Crate 完整度

| Crate | 文件数 | 模块数 | 测试 | 完整度 |
|-------|--------|--------|------|--------|
| core | 64 | 56 | ✅ | 95% |
| cli | 63 | 50+ | ✅ (25+ e2e) | 90% |
| llm | 40 | 31 | ✅ | 89% (16/18 providers) |
| tools | 37 | 35 | ✅ | 94% (33/35 tools) |
| tui | 40 | 13 | ✅ | 85% |
| agent | 12 | 10 | ✅ | 100% (10/10 agents) |
| lsp | 8 | 7 | ✅ | 90% |
| storage | 5 | 5 | ✅ | 90% |
| server | 19 | 17 | ✅ | 85% |
| permission | 5 | 4 | ✅ | 95% |
| auth | 7 | 5 | ✅ | 85% |
| control-plane | 7 | 6 | ✅ | 80% |
| plugin | 5 | 4 | ✅ | 80% |
| git | 2 | 1 | ✅ | 70% |
| mcp | 8 | 7 | ✅ | 90% |

### 4.2 已完成的主要能力

- ✅ 10 种 Agent 类型 (Build, Plan, General, Explore, Review, Refactor, Debug)
- ✅ 33/35 内置工具
- ✅ 16/18 LLM Provider
- ✅ 完整 Server API (REST + WebSocket + SSE + MCP)
- ✅ TUI 输入语法 (@file, !shell, /command)
- ✅ LSP 诊断集成
- ✅ 权限系统 (95%)
- ✅ 存储层 (SQLite)
- ✅ Plugin WASM 运行时框架

### 4.3 剩余差距

| 差距 | 优先级 | 影响 | 修复方案 |
|------|--------|------|----------|
| 无集成测试 | P1 | 无法验证端到端流程 | 建立集成测试框架 |
| HuggingFace Provider 缺失 | P2 | 16/18 providers | 实现 HF Inference API |
| AI21 Provider 缺失 | P2 | 16/18 providers | 实现 AI21 Studio API |
| session_load/save 工具缺失 | P2 | 会话管理不完整 | 实现导入导出工具 |
| clippy 警告 | P2 | 代码质量 | 修复所有警告 |
| 文档注释不足 | P2 | 可维护性 | 补充文档注释 |
| 性能基准缺失 | P2 | 无法验证性能目标 | 建立基准测试 |

---

## 5. 验收标准

- [ ] 集成测试框架可用，核心路径覆盖 100%
- [ ] 18/18 LLM Provider 完整实现
- [ ] 35/35 内置工具完整实现
- [ ] session_load/save 工具可用
- [ ] clippy 零警告
- [ ] 文档注释覆盖率 ≥ 80%
- [ ] 性能基准测试通过
- [ ] cargo test 全部通过
- [ ] cargo build --release 成功

---

## 6. 追溯链

```
iteration-10 完成 (spec_v10, plan_v10, tasks_v10)
    │
    ├── 2026-03-31: WebSocket/SSE 流式实现
    ├── 2026-03-31: MCP 协议实现
    ├── 2026-03-31: TUI 输入语法实现
    └── 2026-03-31: LSP 诊断完善
            │
            ▼
    最终差距分析 (~85-90% 完整度)
            │
            ├── P1: 集成测试框架 → FR-110
            ├── P2: HuggingFace/AI21 Provider → FR-111
            ├── P2: Session Load/Save 工具 → FR-112
            ├── P2: 代码质量完善 → FR-113
            └── P2: 性能基准测试 → FR-114
                    │
                    ▼
            spec_v12.md (本文档)
```

---

**文档状态**: 草稿
**下一步**: 创建 plan_v12.md 实施计划
