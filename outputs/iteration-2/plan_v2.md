# OpenCode-RS 实现计划 v2.2

**版本**: 2.2  
**日期**: 2026-04-04  
**基于**: gap-analysis.md (完整差距分析) + spec_v2.md  
**状态**: 草稿

---

## 1. 计划概述

### 1.1 目标

基于 gap-analysis.md 识别的所有差距，制定修复计划，确保 P0 阻断性问题优先解决。

### 1.2 差距优先级总览

| Priority | 差距数 | 目标 |
|----------|--------|------|
| P0 | 5 | 本周 (配置系统 + 核心功能) |
| P1 | 5 | 下周 |
| P2 | 8 | Sprint 内 |

### 1.3 阶段划分

| Phase | 优先级 | 任务数 | 目标 |
|-------|--------|--------|------|
| Phase 0 | P0 | 5 | 阻断性问题 - Commands/Skills/Context/Fork/配置 |
| Phase 1 | P1 | 5 | 重要功能 - Share/Provider API/TUI 完善 |
| Phase 2 | P2 | 8 | 优化项 - 清理/测试/增强 |

---

## 2. Phase 0: P0 阻断性问题

### 2.1 目标

修复核心阻断性问题，确保基础功能可用。

### 2.2 任务清单

#### Task 0.1: Commands 系统 - 宏命令定义与执行

**目标**: 实现 `.opencode/commands/*.md` 加载，支持 `${file}` 等变量替换

**子任务**:
1. 定义命令格式规范 (YAML frontmatter + Markdown body)
2. 实现 `CommandLoader` 扫描 commands 目录
3. 实现变量替换引擎:
   - `${file}` - 当前文件
   - `${selected}` - 选中文本
   - `${cursor}` - 光标位置
   - `${env:VAR}` - 环境变量
4. 实现命令执行器
5. 实现内置命令 (/help, /test, /debug)

**验收标准**:
- [ ] commands/*.md 可被扫描和加载
- [ ] 变量替换正确工作
- [ ] 内置命令可用

**预计工期**: 3 天

---

#### Task 0.2: Skills 系统 - 延迟加载与语义匹配

**目标**: 实现 `.opencode/skills/<name>/SKILL.md` 按需发现与语义匹配

**子任务**:
1. 定义 Skill 结构 (name, triggers, description, content)
2. 实现 `SkillLoader` 延迟加载
3. 实现语义匹配引擎:
   - 触发词匹配 (精确/模糊)
   - 描述相关性评分
4. 实现 Skills 缓存机制
5. 实现全局/项目级别覆盖

**验收标准**:
- [ ] skills/*/SKILL.md 可被按需加载
- [ ] 触发词匹配正常工作
- [ ] 全局/项目覆盖正确

**预计工期**: 3 天

---

#### Task 0.3: Context Engine - Token Budget 压缩

**目标**: 实现 85% 预警 / 92% compact / 95% 强制新 session 机制

**子任务**:
1. 实现 Token Budget 计算模块
2. 实现 Context Ranking (按重要性排序)
3. 实现 Context Compaction:
   - 85% 阈值前触发预警
   - 92% 阈值触发自动 compact
   - 95% 强制新建 session
4. 实现 Checkpoint 持久化
5. 实现摘要生成 (Summarization)

**验收标准**:
- [ ] Token 预算计算误差 < 5%
- [ ] 85% 预警正常触发
- [ ] 92% 自动 compact 正常工作
- [ ] Checkpoint 可恢复

**预计工期**: 5 天

---

#### Task 0.4: 多层配置合并

**目标**: 实现 global/project/env/CLI 多层配置优先级合并

**子任务**:
1. 实现配置来源优先级:
   - CLI 参数 > 环境变量 > 项目配置 > 全局配置
2. 实现 JSONC 解析器 (支持注释)
3. 实现 `CredentialRef` 引用机制
4. 实现 Provider-specific 环境变量绑定:
   - `OPENAI_API_KEY` → openai provider
   - `ANTHROPIC_API_KEY` → anthropic provider
5. 实现 TUI 独立配置路径 (`OPENCODE_TUI_CONFIG`)

**验收标准**:
- [ ] 多层配置正确合并
- [ ] JSONC 格式正确解析
- [ ] 环境变量优先级高于文件
- [ ] Provider credential 引用正常工作

**预计工期**: 3 天

---

#### Task 0.5: .opencode 目录加载实现

**目标**: 实现 `.opencode/` 目录下的 agents、commands、modes、plugins 加载

**子任务**:
1. 定义目录结构扫描逻辑:
   ```
   .opencode/
   ├── config.jsonc
   ├── agents/<name>/AGENT.md
   ├── commands/<name>.md
   ├── modes/<name>.json
   ├── plugins/<name>.wasm
   ├── skills/<name>/SKILL.md
   ├── tools/<name>/TOOL.md
   └── themes/<name>.json
   ```
2. 实现 DirectoryScanner
3. 实现各模块加载器 (Agent/Command/Plugin/Skill/Theme/Tool)
4. 实现加载优先级: 项目 > 全局

**验收标准**:
- [ ] 各子目录可扫描并加载
- [ ] 项目目录覆盖全局目录

**预计工期**: 3 天

---

### 2.3 Phase 0 并行策略

**并行执行**:
- Task 0.1, 0.2, 0.4, 0.5 可并行进行
- Task 0.3 依赖 Task 0.4 (配置系统)

---

## 3. Phase 1: P1 重要功能

### 3.1 目标

完善核心功能，修复 API 和 Share 功能。

### 3.2 任务清单

#### Task 1.1: Session Fork 实现

**目标**: 实现 session 分叉功能

**子任务**:
1. 添加 `parent_session_id` 字段到 Storage
2. 实现 `/sessions/{id}/fork` API
3. 实现 Fork 时的上下文复制
4. 实现 Fork Lineage 追踪

**验收标准**:
- [ ] Session 可分叉
- [ ] 分叉后独立可编辑
- [ ] Fork lineage 正确追踪

**预计工期**: 2 天

---

#### Task 1.2: Share 功能 - 导出与短链

**目标**: 实现 session JSON/Markdown 导出和 self-hosted share 服务层

**子任务**:
1. 实现 JSON 导出
2. 实现 Markdown 导出
3. 实现 Patch Bundle 导出
4. 实现敏感信息自动脱敏
5. 实现分享服务层 (可选)

**验收标准**:
- [ ] JSON 格式可导出
- [ ] Markdown 格式可导出
- [ ] Patch bundle 可导出
- [ ] 敏感信息自动脱敏

**预计工期**: 3 天

---

#### Task 1.3: Provider API 完善

**目标**: 完善 Provider 管理和凭证 API

**子任务**:
1. 实现 `/providers/{id}/credentials` API:
   - 设置凭证
   - 测试连通性
   - 撤销凭证
2. 实现凭证过期状态检测
3. 实现 Provider-specific header 注入

**验收标准**:
- [ ] 凭证可设置/测试/撤销
- [ ] 连通性测试正常工作
- [ ] 凭证过期有状态显示

**预计工期**: 2 天

---

#### Task 1.4: Session Summarize API

**目标**: 实现自动摘要端点

**子任务**:
1. 实现 `/sessions/{id}/summarize` API
2. 实现自动 Compact 阈值 (85%/92%)
3. 实现摘要生成逻辑

**验收标准**:
- [ ] Summarize API 可用
- [ ] 自动 Compact 正常工作

**预计工期**: 2 天

---

#### Task 1.5: TUI 快捷输入完善

**目标**: 完整实现 `@file` `/command` `!shell` 三种快捷输入解析器

**子任务**:
1. 实现 `@file` 解析器 - 文件引用
2. 实现 `/command` 解析器 - 命令触发
3. 实现 `!shell` 解析器 - Shell 执行
4. 实现快捷输入 UI 提示

**验收标准**:
- [ ] @file 正确解析
- [ ] /command 正确解析
- [ ] !shell 正确解析
- [ ] 快捷提示可用

**预计工期**: 3 天

---

### 3.3 Phase 1 并行策略

**并行执行**: Task 1.1, 1.2, 1.3, 1.4, 1.5 可并行

---

## 4. Phase 2: P2 优化项

### 4.1 目标

清理废弃字段，补充测试，完善 TUI 功能。

### 4.2 任务清单

#### Task 2.1: 废弃字段清理

**目标**: 清理 mode 和 layout 废弃字段

**子任务**:
1. 标记 `mode` 字段为 `#[deprecated]`
2. 标记 `layout` 字段为 `#[deprecated]`
3. 添加废弃警告日志
4. 实现自动迁移提示

**预计工期**: 1 天

---

#### Task 2.2: TUI 三栏/双栏切换

**目标**: 实现可切换布局

**子任务**:
1. 实现布局切换逻辑
2. 实现三栏布局
3. 实现双栏布局
4. 保存布局偏好

**预计工期**: 2 天

---

#### Task 2.3: TUI 右栏功能完善

**目标**: 完善右栏面板 (diagnostics/todo/权限队列)

**子任务**:
1. 实现 Diagnostics 面板
2. 实现 Todo 面板
3. 实现权限队列面板

**预计工期**: 2 天

---

#### Task 2.4: TUI Token/Cost 显示

**目标**: 实现 token 统计与成本显示

**子任务**:
1. 实现 token 统计
2. 实现成本计算
3. 实现 UI 显示

**预计工期**: 1 天

---

#### Task 2.5: TUI Patch 预览展开

**目标**: 实现 diff 展开/收起交互

**子任务**:
1. 实现 patch 预览
2. 实现展开/收起交互
3. 实现批量操作

**预计工期**: 2 天

---

#### Task 2.6: scroll_acceleration 结构修复

**目标**: 修复 scroll_acceleration 配置结构

**子任务**:
1. 修改结构为 `{ enabled: bool, speed: Option<f32> }`
2. 兼容旧格式
3. 添加测试

**预计工期**: 1 天

---

#### Task 2.7: keybinds 自定义绑定支持

**目标**: 实现自定义 keybinds 支持

**子任务**:
1. 扩展 KeybindsConfig 结构
2. 实现自定义绑定解析
3. 实现冲突检测

**预计工期**: 2 天

---

#### Task 2.8: 测试覆盖补充

**目标**: 补充关键路径测试

**子任务**:
1. 添加 Skills 加载与匹配 E2E 测试
2. 添加 Command 模板展开测试
3. 添加 MCP 工具发现与调用测试
4. 添加 Session 并发与分叉测试

**预计工期**: 3 天

---

### 4.3 Phase 2 并行策略

**可并行**: Task 2.1 可单独进行，2.2-2.7 可并行，2.8 依赖功能实现

---

## 5. 资源分配

### 5.1 人力估算

| Phase | 任务数 | 预计工期 | 总人天 |
|-------|--------|----------|--------|
| Phase 0 | 5 | 17 天 | 17 人天 |
| Phase 1 | 5 | 12 天 | 12 人天 |
| Phase 2 | 8 | 14 天 | 14 人天 |
| **总计** | **18** | **43 天** | **43 人天** |

### 5.2 并行机会

- Phase 0: Task 0.1, 0.2, 0.4, 0.5 可并行
- Phase 1: 所有任务可并行
- Phase 2: Task 2.2-2.7 可并行

---

## 6. 里程碑

| Milestone | Phase | 任务 | 预计完成 |
|-----------|-------|------|----------|
| M0.1 | Phase 0 | P0 阻断性问题 | Week 3 |
| M1 | Phase 1 | P1 重要功能 | Week 5 |
| M2 | Phase 2 | P2 优化项 | Week 7 |

---

## 7. 验收流程

### 7.1 每个任务的验收标准

1. 功能正常运行
2. 错误处理正确
3. 性能满足要求
4. 文档完整
5. 测试覆盖

### 7.2 阶段验收

- Phase 0 完成后进行核心功能评审
- Phase 1 完成后进行 API 评审
- Phase 2 完成后进行代码质量评审

---

**文档状态**: 草稿  
**下一步**: 更新 tasks_v2.md 任务清单

