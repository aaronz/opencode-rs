# OpenCode-RS v18 Implementation Plan

**Version**: 18  
**Date**: 2026-04-07  
**Status**: Draft

---

## 1. Overview

This plan addresses the remaining implementation gaps identified in Spec v18. Unlike v17, v18 gap analysis reveals **3 P0阻断性问题** that must be prioritized immediately, plus **6 P1高优先级问题** and **8 P2中优先级问题**.

## 2. Current Status Summary

Based on Spec v18 gap analysis:

| Category | Total | Completed | Pending | Blocked |
|----------|-------|-----------|---------|---------|
| P0 Features | 3 | 0 | 3 | 3 |
| P1 Features | 6 | 0 | 6 | 0 |
| P2 Features | 8 | 0 | 8 | 0 |

**Overall Progress**: ~60% (down from claimed 80% in v17)

### 2.1 Critical Findings from Gap Analysis

1. **GAP-P0-002**: FileRefHandler存在但未与LLM上下文集成 - @语法不完整
2. **GAP-P0-003**: ShellHandler已实现但未与消息处理流程集成 - !命令输出不显示
3. **Architecture ADR-001**: 已通过ADR-001解决OpenTUI vs Ratatui问题

---

## 3. P0 Tasks (This Week - MUST FIX)

### 3.1 FR-014: File Reference Context Integration

- **Gap ID**: GAP-P0-002
- **Description**: 将FileRefHandler结果集成到LLM请求上下文，实现@引用的文件内容自动添加到AI对话
- **Priority**: P0
- **Estimated Effort**: Medium
- **Files Affected**: 
  - `crates/tui/src/app.rs`
  - `crates/tui/src/file_ref_handler.rs`
- **Status**: ❌ 未实现
- **Dependencies**: None
- **Acceptance Criteria**:
  - [ ] `@file` 引用的文件内容自动添加到消息上下文
  - [ ] `FileRefHandler.format_for_context()` 被调用
  - [ ] 上下文大小合理限制 (MAX_CONTENT_SIZE)

### 3.2 FR-023: Shell Output UI Rendering

- **Gap ID**: GAP-P0-003
- **Description**: 将ShellHandler结果渲染到UI，实现!命令的完整执行和结果展示流程
- **Priority**: P0
- **Estimated Effort**: Medium
- **Files Affected**:
  - `crates/tui/src/app.rs`
  - `crates/tui/src/shell_handler.rs`
  - `crates/tui/src/components/terminal_panel.rs`
- **Status**: ❌ 未实现
- **Dependencies**: None
- **Acceptance Criteria**:
  - [ ] `!command` 执行后输出渲染到TerminalPanel
  - [ ] Ctrl+C 终止功能完整 (InterruptibleHandle UI绑定)
  - [ ] 命令执行状态显示

---

## 4. P1 Tasks (This Week to Next Week)

### 4.1 FR-061/FR-153: Session Share

- **Gap ID**: GAP-P1-001
- **Description**: 实现/share和取消分享功能
- **Priority**: P1
- **Estimated Effort**: Medium
- **Files Affected**:
  - `crates/tui/src/command.rs`
  - `crates/tui/src/dialogs/session_share.rs` (new)
- **Status**: ❌ 未实现
- **Acceptance Criteria**:
  - [ ] `/share` 命令生成可分享链接
  - [ ] `/unshare` 取消分享
  - [ ] SessionShare对话框 (FR-101) 实现

### 4.2 FR-059/FR-087: Thinking Mode Toggle

- **Gap ID**: GAP-P1-003
- **Description**: 实现思考块的可见性切换
- **Priority**: P1
- **Estimated Effort**: Small
- **Files Affected**:
  - `crates/tui/src/command.rs`
  - `crates/tui/src/widgets/thinking_block.rs` (new)
- **Status**: ❌ 未实现
- **Acceptance Criteria**:
  - [ ] `/thinking` 切换思考模式
  - [ ] ThinkingBlock可见性可控制
  - [ ] 状态持久化

### 4.3 FR-006: Typewriter Effect

- **Gap ID**: GAP-P1-004
- **Description**: 实现流式输出的打字机效果
- **Priority**: P1
- **Estimated Effort**: Medium
- **Files Affected**:
  - `crates/tui/src/components/input_widget.rs`
  - `crates/tui/src/config.rs`
- **Status**: ❌ 未实现 (config存在但未使用)
- **Acceptance Criteria**:
  - [ ] typewriter_speed配置生效
  - [ ] 流式输出逐字显示
  - [ ] 可跳过打字效果

### 4.4 FR-160: Model Alias Resolution

- **Gap ID**: GAP-P1-005
- **Description**: 在发送请求前解析模型别名
- **Priority**: P1
- **Estimated Effort**: Small
- **Files Affected**:
  - `crates/tui/src/app.rs`
  - `crates/llm/` (model alias mapping)
- **Status**: ⚠️ 部分实现 (映射存在但未使用)
- **Acceptance Criteria**:
  - [ ] opus/sonnet/haiku 别名解析生效
  - [ ] 请求发送前解析为完整模型名

### 4.5 FR-007: Real-time Token Counter

- **Gap ID**: GAP-P1-006
- **Description**: 在状态栏显示实时Token使用量
- **Priority**: P1
- **Estimated Effort**: Medium
- **Files Affected**:
  - `crates/tui/src/components/status_bar.rs`
- **Status**: ❌ 未实现 (TokenCounter存在但未显示)
- **Acceptance Criteria**:
  - [ ] StatusBar显示当前token计数
  - [ ] 请求/响应时更新计数
  - [ ] 总计显示

### 4.6 FR-062/FR-102: Session Unshare

- **Gap ID**: GAP-P1-002
- **Description**: 取消会话分享
- **Priority**: P1
- **Estimated Effort**: Medium
- **Files Affected**:
  - `crates/tui/src/command.rs`
  - `crates/tui/src/dialogs/session_unshare.rs` (new)
- **Status**: ❌ 未实现
- **Acceptance Criteria**:
  - [ ] `/unshare` 命令工作
  - [ ] SessionUnshare对话框 (FR-102) 实现

---

## 5. P2 Tasks (Next Week to Two Weeks)

### 5.1 Slash Commands Implementation

| FR-ID | Command | Description | Effort | Status |
|-------|---------|-------------|--------|--------|
| FR-054 | /search | 对话历史搜索 | Medium | ❌ 占位符 |
| FR-055 | /diff | Git差异显示 | Medium | ❌ 占位符 |
| FR-056 | /memory | 记忆管理 | Medium | ❌ 占位符 |
| FR-057 | /plugins | 插件管理 | Large | ❌ 占位符 |
| FR-058 | /username | 用户名设置 | Small | ❌ 占位符 |
| FR-060 | /status | 会话状态显示 | Small | ❌ 占位符 |
| FR-064 | /editor | 外部编辑器 | Medium | ⚠️ 未实现 |
| FR-065 | /init | AGENTS.md初始化 | Medium | ⚠️ 未实现 |

### 5.2 UI Components

| FR-ID | Component | Description | Effort | Status |
|-------|-----------|-------------|--------|--------|
| FR-084 | ProgressBar | 进度条组件 | Low | 🔲 待实现 |
| FR-087 | ThinkingBlock | AI思考过程显示 | Small | ❌ 未实现 |
| FR-101 | SessionShare | 分享对话框 | Medium | ❌ 未实现 |
| FR-102 | SessionUnshare | 取消分享对话框 | Medium | ❌ 未实现 |

### 5.3 Configuration

| FR-ID | Config | Description | Effort | Status |
|-------|--------|-------------|--------|--------|
| FR-115 | keybinds | 自定义快捷键 | High | 🔲 待实现 |
| FR-116 | diff_style | Diff样式配置 | Low | ⚠️ 部分实现 |
| FR-142 | NDJSON | NDJSON输出格式 | Medium | 🔲 待实现 |
| FR-122 | custom themes | 自定义主题 | Medium | 🔲 待实现 |

---

## 6. Technical Debt

### 6.1 High Priority

| Debt ID | Description | Risk | Action |
|---------|-------------|------|--------|
| TECH-001 | Custom(String)命令占位符20+个 | High | 逐一实现或移除 |
| TECH-002 | 硬编码值(MAX_OUTPUT_SIZE=100KB等) | Medium | 移至配置 |
| TECH-003 | 重复命令定义(undo定义两次) | Medium | 清理重复 |

### 6.2 Medium Priority

| Debt ID | Description | Risk |
|---------|-------------|------|
| TECH-004 | Error处理不一致 | Medium |
| TECH-005 | 魔法数字(100, 5000等) | Low |

---

## 7. Implementation Order

### Week 1 (P0 Focus)

1. **FR-014** - File reference context integration (BLOCKING)
2. **FR-023** - Shell output UI rendering (BLOCKING)

### Week 1-2 (P1 High Visibility)

3. **FR-006** - Typewriter effect
4. **FR-007** - Token counting display
5. **FR-160** - Model alias resolution
6. **FR-059/087** - Thinking mode toggle

### Week 2-3 (P1 Session Features)

7. **FR-061/153** - Session share
8. **FR-062/102** - Session unshare

### Week 3-4 (P2 Commands)

9. **FR-055** - /diff
10. **FR-054** - /search
11. **FR-064** - /editor
12. **FR-065** - /init
13. **FR-056** - /memory
14. **FR-057** - /plugins
15. **FR-058** - /username
16. **FR-060** - /status

---

## 8. Dependencies

- FR-101 depends on FR-061 (SessionShare dialog)
- FR-102 depends on FR-062 (SessionUnshare dialog)
- FR-014 requires understanding of LLM request flow
- FR-023 requires TerminalPanel component

---

## 9. Testing Requirements

Per Constitution Article 4:
- Unit tests for FileRefHandler integration
- Integration tests for ShellHandler UI flow
- Update TEST_MAPPING.md with new test cases
- Coverage targets per Constitution C-055

---

## 10. Risks

1. **FileRefHandler integration** may require changes to LLM request serialization
2. **Shell output threading** needs careful handling to avoid UI blocking
3. **Session share** requires backend service (may need server component)

---

**Next Review**: After P0 items implementation  
**Blocking Issues**: GAP-P0-002, GAP-P0-003