# OpenCode-RS 规格文档 v10

**版本**: 10.0
**日期**: 2026-04-06
**基于**: spec_v9.md + iteration-9 完成后的剩余差距分析
**状态**: 已完成

---

## 1. 文档概述

### 1.1 背景

iteration-9 已完成所有 7 项任务（33/33 子任务），但仍有部分 P2/P3 差距未解决。本规格文档定义 iteration-10 的目标范围。

### 1.2 范围

| 类别 | 包含 | 排除 |
|------|------|------|
| **包含** | TUI E2E 测试框架、Agent 集成测试、Provider 协议测试、环境变量覆盖完善 | GitHub 集成、IDE 插件、Desktop Shell |
| **优先级** | P2 (4项) + P3 (2项) | P3 (OAuth Browser) 推迟 |

---

## 2. 功能需求

### FR-105: TUI E2E 测试框架

**描述**: 实现 ratatui testing framework 支持

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-105.1 | TUI 组件单元测试框架 | P2 |
| FR-105.2 | 输入事件模拟 | P2 |
| FR-105.3 | 渲染输出断言 | P2 |

### FR-106: Agent 集成测试

**描述**: 补充 build/plan 闭环测试

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-106.1 | Build Agent 集成测试 | P2 |
| FR-106.2 | Plan Agent 只读约束测试 | P2 |
| FR-106.3 | Agent 切换测试 | P2 |

### FR-107: Provider 协议测试

**描述**: 补充 mock provider 测试

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-107.1 | Mock Provider 实现 | P3 |
| FR-107.2 | Provider 协议一致性测试 | P3 |
| FR-107.3 | 认证流程测试 | P2 |

### FR-108: 环境变量覆盖完善

**描述**: 补充 provider-specific 变量绑定

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-108.1 | Provider-specific 环境变量解析 | P3 |
| FR-108.2 | 环境变量优先级测试 | P3 |

### FR-109: OAuth Browser 登录

**描述**: 实现完整 PKCE 流程

| 子需求 | 描述 | 优先级 |
|--------|------|--------|
| FR-109.1 | PKCE code verifier/challenge | P3 |
| FR-109.2 | localhost callback server | P3 |
| FR-109.3 | Browser 自动打开 | P3 |

---

## 3. 验收标准

- [ ] TUI 组件测试覆盖率 ≥ 50%
- [ ] Agent 集成测试全部通过
- [ ] Provider 协议测试覆盖主要 provider
- [ ] 环境变量覆盖完整
- [ ] OAuth PKCE 流程可用

---

**文档状态**: 草稿
**下一步**: 创建 plan_v10.md 实施计划
