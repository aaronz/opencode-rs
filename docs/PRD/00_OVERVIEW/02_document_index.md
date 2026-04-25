# PRD: Document Index

## Overview

This document provides the complete index of Product Requirement Documents for OpenCode RS.

---

## Document Structure

```
/PRD
├── 00_OVERVIEW/           # Product overview, glossary, architecture summary
├── 10_CORE/              # Core system PRDs
├── 20_INTEGRATION/        # Integration system PRDs
├── 30_INFRASTRUCTURE/      # Infrastructure PRDs
├── 40_USER_FACING/        # User-facing system PRDs
├── 50_IMPLEMENTATION/      # Implementation tracking
├── 90_REFERENCE/           # Reference documents
└── 99_ARCHIVE/           # Archived documents
```

---

## 00 Overview

| Document | Description | Status |
|----------|-------------|--------|
| [01_glossary.md](./01_glossary.md) | Unified terminology and definitions | ✅ Complete |
| [02_document_index.md](./02_document_index.md) | This file - complete document index | ✅ Complete |
| [03_architecture_overview.md](./03_architecture_overview.md) | System architecture summary | ⚠️ TODO |

---

## 10 Core Systems

| Document | Description | Status |
|----------|-------------|--------|
| [10_core_architecture.md](./10_CORE/10_core_architecture.md) | Core entities: Project, Session, Message, Part | ✅ Complete |
| [11_agent_system.md](./10_CORE/11_agent_system.md) | Agent system: Primary/Subagent, delegation | ✅ Complete |
| [12_tools_system.md](./10_CORE/12_tools_system.md) | Tools system: registration, execution, permissions | ✅ Complete |
| [13_state_machines.md](./10_CORE/13_state_machines.md) | State machine definitions and diagrams | ✅ Complete |
| **Module PRDs** | | |
| [agent.md](./10_CORE/modules/agent.md) | Agent Rust API, 750+ lines | ✅ Complete |
| [session.md](./10_CORE/modules/session.md) | Session Rust API, 500+ lines | ✅ Complete |
| [tool.md](./10_CORE/modules/tool.md) | Tool Rust API, 770+ lines | ✅ Complete |

---

## 20 Integration Systems

| Document | Description | Status |
|----------|-------------|--------|
| [20_lsp_integration.md](./20_INTEGRATION/20_lsp_integration.md) | Language Server Protocol integration | ✅ Complete |
| [21_mcp_integration.md](./20_INTEGRATION/21_mcp_integration.md) | Model Context Protocol integration | ✅ Complete |
| [22_plugin_system.md](./20_INTEGRATION/22_plugin_system.md) | Plugin system: WASM runtime | ✅ Complete |
| [23_provider_model.md](./20_INTEGRATION/23_provider_model.md) | Provider/Model abstraction | ✅ Complete |
| [24_skills_system.md](./20_INTEGRATION/24_skills_system.md) | Skills system: prompt templates | ✅ Complete |
| [25_formatters.md](./20_INTEGRATION/25_formatters.md) | Code formatter integrations | ✅ Complete |
| [26_vcs_integration.md](./20_INTEGRATION/26_vcs_integration.md) | GitHub/GitLab integration | ⚠️ TODO |
| **Module PRDs** | | |
| [provider.md](./20_INTEGRATION/modules/provider.md) | Provider Rust API, 800+ lines | ✅ Complete |
| [lsp.md](./20_INTEGRATION/modules/lsp.md) | LSP Rust API | ✅ Complete |
| [mcp.md](./20_INTEGRATION/modules/mcp.md) | MCP Rust API | ✅ Complete |
| [plugin.md](./20_INTEGRATION/modules/plugin.md) | Plugin Rust API | ✅ Complete |
| [acp.md](./20_INTEGRATION/modules/acp.md) | ACP Rust API | ✅ Complete |
| [skill.md](./20_INTEGRATION/modules/skill.md) | Skill Rust API | ✅ Complete |
| [control-plane.md](./20_INTEGRATION/modules/control-plane.md) | Control plane API | ✅ Complete |

---

## 30 Infrastructure

| Document | Description | Status |
|----------|-------------|--------|
| [30_cli_reference.md](./30_INFRASTRUCTURE/30_cli_reference.md) | CLI commands and arguments | ✅ Complete |
| [31_api_contract.md](./30_INFRASTRUCTURE/31_api_contract.md) | HTTP API schemas | ✅ Complete |
| [32_storage.md](./30_INFRASTRUCTURE/32_storage.md) | SQLite storage | ⚠️ TODO |
| [33_configuration.md](./30_INFRASTRUCTURE/33_configuration.md) | Configuration system | ✅ Complete |
| **Module PRDs** | | |
| [config.md](./30_INFRASTRUCTURE/modules/config.md) | Config Rust API, 780+ lines | ✅ Complete |

---

## 40 User-Facing Systems

| Document | Description | Status |
|----------|-------------|--------|
| [40_tui_system.md](./40_USER_FACING/40_tui_system.md) | Terminal UI system | ✅ Complete |
| [41_desktop_web.md](./40_USER_FACING/41_desktop_web.md) | Desktop/Web interface | ✅ Complete |
| [42_permission_model.md](./40_USER_FACING/42_permission_model.md) | Permission model and RBAC | ✅ Complete |
| [43_tui_plugin_api.md](./40_USER_FACING/43_tui_plugin_api.md) | TUI plugin API | ✅ Complete |

---

## 50 Implementation

| Document | Description | Status |
|----------|-------------|--------|
| [50_implementation_plan.md](./50_IMPLEMENTATION/50_implementation_plan.md) | Phase-based implementation plan | ⚠️ TODO |
| [51_test_plan.md](./50_IMPLEMENTATION/51_test_plan.md) | Test strategy and plan | ⚠️ TODO |
| [52_test_roadmap.md](./50_IMPLEMENTATION/52_test_roadmap.md) | Rust test implementation roadmap | ⚠️ TODO |
| [53_test_backlog.md](./50_IMPLEMENTATION/53_test_backlog.md) | Crate-by-crate test backlog | ⚠️ TODO |

---

## 90 Reference

| Document | Description |
|----------|-------------|
| [ERROR_CODE_CATALOG.md](./90_REFERENCE/ERROR_CODE_CATALOG.md) | Complete error code reference |
| [NON_FUNCTIONAL_REQUIREMENTS.md](./90_REFERENCE/NON_FUNCTIONAL_REQUIREMENTS.md) | Performance, security, usability requirements |
| [AI_CODING_VALIDATION.md](./90_REFERENCE/AI_CODING_VALIDATION.md) | AI coding acceptance validation |
| [COMPREHENSIVE_ANALYSIS_REPORT.md](./90_REFERENCE/COMPREHENSIVE_ANALYSIS_REPORT.md) | Full PRD analysis report |

---

## 99 Archive

Archived documents from previous versions:

| Document | Reason Archived |
|----------|----------------|
| [23-opencode-vs-opencode-rs-gap.md](./99_ARCHIVE/23-opencode-vs-opencode-rs-gap.md) | Gap analysis - outdated |
| [24-cli-contract-gap-from-harness-report.md](./99_ARCHIVE/24-cli-contract-gap-from-harness-report.md) | Gap analysis - outdated |
| [25-parity-gap-analysis-2026-04.md](./99_ARCHIVE/25-parity-gap-analysis-2026-04.md) | Gap analysis - outdated |
| [21-rust-code-refactor.md](./99_ARCHIVE/21-rust-code-refactor.md) | Refactor tracking - obsolete |

---

## Quick Links

- [Glossary](./00_OVERVIEW/01_glossary.md) - Terminology definitions
- [Error Codes](./90_REFERENCE/ERROR_CODE_CATALOG.md) - Error code reference
- [Acceptance Criteria](./90_REFERENCE/AI_CODING_VALIDATION.md) - AI coding readiness
- [State Machines](./10_CORE/13_state_machines.md) - State diagrams

---

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-04-26 | AI | Restructure to 00-99 format |
