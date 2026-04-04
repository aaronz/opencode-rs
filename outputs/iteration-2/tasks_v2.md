# OpenCode-RS 任务清单 v2.2

**版本**: 2.2  
**日期**: 2026-04-04  
**基于**: gap-analysis.md (完整差距分析) + plan_v2.md  
**状态**: 草稿

---

## 1. 任务总览

| Phase | 优先级 | 任务数 | 状态 |
|-------|--------|--------|------|
| Phase 0 | P0 | 5 | 待开始 |
| Phase 1 | P1 | 5 | 待开始 |
| Phase 2 | P2 | 8 | 待开始 |

---

## 2. Phase 0: P0 阻断性问题

### Task 0.1: Commands 系统 - 宏命令定义与执行

**ID**: TASK-0.1  
**优先级**: P0  
**模块**: core/tui  
**状态**: completed  
**预计工期**: 3 天

**目标**: 实现 `.opencode/commands/*.md` 加载，支持 `${file}` 等变量替换

**子任务**:
- [x] TASK-0.1.1: 定义命令格式规范
  - YAML frontmatter 定义 (name, description, triggers, variables)
  - Markdown body 定义命令逻辑
- [x] TASK-0.1.2: 实现 CommandLoader
  - 扫描 commands 目录
  - 解析 YAML frontmatter
  - 注册命令到命令注册表
- [x] TASK-0.1.3: 实现变量替换引擎
  - `${file}` - 当前文件路径
  - `${selected}` - 选中文本
  - `${cursor}` - 光标位置
  - `${env:VAR}` - 环境变量
  - `{file:path}` - 文件内容
- [x] TASK-0.1.4: 实现命令执行器
  - 命令解析与路由
  - 变量展开
  - 执行与结果返回
- [x] TASK-0.1.5: 实现内置命令
  - /help - 帮助命令
  - /test - 测试命令
  - /debug - 调试命令

**验收标准**:
- [x] commands/*.md 可被扫描和加载
- [x] 变量替换正确工作
- [x] 内置命令可用

**依赖**: 无

---

### Task 0.2: Skills 系统 - 延迟加载与语义匹配

**ID**: TASK-0.2  
**优先级**: P0  
**模块**: core  
**状态**: completed  
**预计工期**: 3 天

**目标**: 实现 `.opencode/skills/<name>/SKILL.md` 按需发现与语义匹配

**子任务**:
- [x] TASK-0.2.1: 定义 Skill 结构
  - name, triggers, description, content, category
  - 触发词 (exact/fuzzy match)
- [x] TASK-0.2.2: 实现 SkillLoader 延迟加载
  - 扫描 skills/ 目录
  - 按需加载 (lazy load)
  - 缓存机制
- [x] TASK-0.2.3: 实现语义匹配引擎
  - 触发词精确匹配
  - 触发词模糊匹配
  - 描述相关性评分
- [x] TASK-0.2.4: 实现全局/项目级别覆盖
  - 全局 .opencode/skills/
  - 项目 .opencode/skills/
  - 同名覆盖原则

**验收标准**:
- [x] skills/*/SKILL.md 可被按需加载
- [x] 触发词匹配正常工作
- [x] 全局/项目覆盖正确

**依赖**: 无

---

### Task 0.3: Context Engine - Token Budget 压缩

**ID**: TASK-0.3  
**优先级**: P0  
**模块**: core  
**状态**: completed  
**预计工期**: 5 天

**目标**: 实现 85% 预警 / 92% compact / 95% 强制新 session 机制

**子任务**:
- [x] TASK-0.3.1: 实现 Token Budget 计算模块
  - Token 计数算法
  - 预算分配策略
  - 误差控制在 5% 以内
- [x] TASK-0.3.2: 实现 Context Ranking
  - 消息重要性评分
  - 上下文优先级排序
  - 历史消息压缩策略
- [x] TASK-0.3.3: 实现 Context Compaction
  - 85% 阈值前触发预警 (warning)
  - 92% 阈值触发自动 compact (aggressive)
  - 95% 阈值强制新建 session (force new session)
- [x] TASK-0.3.4: 实现 Checkpoint 持久化
  - 每次消息后 checkpoint 持久化
  - 可恢复到历史 checkpoint
  - Checkpoint 粒度控制
- [x] TASK-0.3.5: 实现摘要生成 (Summarization)
  - LLM 摘要压缩
  - 关键信息提取
  - 摘要缓存

**验收标准**:
- [x] Token 预算计算误差 < 5%
- [x] 85% 预警正常触发
- [x] 92% 自动 compact 正常工作
- [x] Checkpoint 可恢复

**依赖**: TASK-0.4 (配置系统)

---

### Task 0.4: 多层配置合并

**ID**: TASK-0.4  
**优先级**: P0  
**模块**: config  
**状态**: completed  
**预计工期**: 3 天

**目标**: 实现 global/project/env/CLI 多层配置优先级合并

**子任务**:
- [x] TASK-0.4.1: 实现配置来源优先级
  - CLI 参数 > 环境变量 > 项目配置 > 全局配置
  - 配置合并逻辑
- [x] TASK-0.4.2: 实现 JSONC 解析器
  - 支持注释 // 和 /* */
  - 兼容标准 JSON
- [x] TASK-0.4.3: 实现 CredentialRef 引用机制
  - 引用而非明文存储
  - 密钥链集成
- [x] TASK-0.4.4: 实现 Provider-specific 环境变量绑定
  - `OPENAI_API_KEY` → openai provider
  - `ANTHROPIC_API_KEY` → anthropic provider
  - `GOOGLE_API_KEY` → google provider
- [x] TASK-0.4.5: 实现 TUI 独立配置路径
  - `OPENCODE_TUI_CONFIG` 环境变量
  - TUI 配置独立加载

**验收标准**:
- [x] 多层配置正确合并
- [x] JSONC 格式正确解析
- [x] 环境变量优先级高于文件
- [x] Provider credential 引用正常工作

**依赖**: 无

---

### Task 0.5: .opencode 目录加载实现

**ID**: TASK-0.5  
**优先级**: P0  
**模块**: config  
**状态**: completed  
**预计工期**: 3 天

**目标**: 实现 `.opencode/` 目录下的 agents、commands、modes、plugins 加载

**子任务**:
- [x] TASK-0.5.1: 定义目录结构扫描逻辑
  - 定义扫描目录结构
  - 定义各类型文件格式
- [x] TASK-0.5.2: 实现 DirectoryScanner
  - 扫描 .opencode 子目录
  - 解析各类型配置文件
- [x] TASK-0.5.3: 实现 AgentLoader
  - 加载 .opencode/agents/*.md
  - 解析 YAML frontmatter
- [x] TASK-0.5.4: 实现 CommandLoader (复用 TASK-0.1)
  - 加载 .opencode/commands/*.md
  - 解析命令模板
- [x] TASK-0.5.5: 实现 PluginLoader
  - 加载 .opencode/plugins/*.wasm
  - 验证插件格式
- [x] TASK-0.5.6: 实现 SkillLoader (复用 TASK-0.2)
  - 加载 .opencode/skills/*/SKILL.md
  - 解析 skill 元信息
- [x] TASK-0.5.7: 实现 ThemeLoader
  - 加载 .opencode/themes/*.json
  - 验证 theme 格式
- [x] TASK-0.5.8: 实现加载优先级
  - 全局 `.opencode/` 目录 (优先级低)
  - 项目 `.opencode/` 目录 (优先级高)
  - 同名覆盖原则

**验收标准**:
- [ ] .opencode/agents 目录可扫描并加载 Agent
- [ ] .opencode/commands 目录可扫描并加载 Command
- [ ] .opencode/plugins 目录可扫描并发现 Plugin
- [ ] 项目目录覆盖全局目录

**依赖**: 无

---

## 3. Phase 1: P1 重要功能

### Task 1.1: Session Fork 实现

**ID**: TASK-1.1  
**优先级**: P1  
**模块**: storage/server  
**状态**: pending  
**预计工期**: 2 天

**目标**: 实现 session 分叉功能

**子任务**:
- [ ] TASK-1.1.1: 添加 Storage 模型字段
  - 添加 parent_session_id 字段
  - 添加 fork lineage 追踪
- [ ] TASK-1.1.2: 实现 Fork API
  - POST /sessions/{id}/fork
  - 上下文复制逻辑
- [ ] TASK-1.1.3: 实现 Fork Lineage 追踪
  - 父子关系记录
  - 分叉历史展示

**验收标准**:
- [ ] Session 可分叉
- [ ] 分叉后独立可编辑
- [ ] Fork lineage 正确追踪

**依赖**: 无

---

### Task 1.2: Share 功能 - 导出与短链

**ID**: TASK-1.2  
**优先级**: P1  
**模块**: core/server  
**状态**: pending  
**预计工期**: 3 天

**目标**: 实现 session JSON/Markdown 导出和 self-hosted share 服务层

**子任务**:
- [ ] TASK-1.2.1: 实现 JSON 导出
  - 完整 session 序列化
  - 元数据包含
- [ ] TASK-1.2.2: 实现 Markdown 导出
  - 对话格式转换
  - 代码块保留
- [ ] TASK-1.2.3: 实现 Patch Bundle 导出
  - diff 打包
  - 批量应用
- [ ] TASK-1.2.4: 实现敏感信息自动脱敏
  - API Key 移除
  - 凭证信息过滤
- [ ] TASK-1.2.5: 实现分享服务层 (可选)
  - 短链生成
  - 访问控制

**验收标准**:
- [ ] JSON 格式可导出
- [ ] Markdown 格式可导出
- [ ] Patch bundle 可导出
- [ ] 敏感信息自动脱敏

**依赖**: TASK-1.1

---

### Task 1.3: Provider API 完善

**ID**: TASK-1.3  
**优先级**: P1  
**模块**: server  
**状态**: pending  
**预计工期**: 2 天

**目标**: 完善 Provider 管理和凭证 API

**子任务**:
- [ ] TASK-1.3.1: 实现凭证设置 API
  - POST /providers/{id}/credentials
  - 加密存储
- [ ] TASK-1.3.2: 实现连通性测试 API
  - POST /providers/{id}/credentials/test
  - 返回连接状态
- [ ] TASK-1.3.3: 实现凭证撤销 API
  - DELETE /providers/{id}/credentials
  - 清理存储
- [ ] TASK-1.3.4: 实现凭证过期状态检测
  - 过期时间记录
  - 状态显示

**验收标准**:
- [ ] 凭证可设置/测试/撤销
- [ ] 连通性测试正常工作
- [ ] 凭证过期有状态显示

**依赖**: 无

---

### Task 1.4: Session Summarize API

**ID**: TASK-1.4  
**优先级**: P1  
**模块**: server/core  
**状态**: pending  
**预计工期**: 2 天

**目标**: 实现自动摘要端点

**子任务**:
- [ ] TASK-1.4.1: 实现 Summarize API
  - POST /sessions/{id}/summarize
  - 返回摘要内容
- [ ] TASK-1.4.2: 实现自动 Compact 阈值
  - 85% 预警触发
  - 92% Compact 触发
- [ ] TASK-1.4.3: 实现摘要生成逻辑
  - 消息压缩
  - 关键信息提取

**验收标准**:
- [ ] Summarize API 可用
- [ ] 自动 Compact 正常工作

**依赖**: TASK-0.3

---

### Task 1.5: TUI 快捷输入完善

**ID**: TASK-1.5  
**优先级**: P1  
**模块**: tui  
**状态**: pending  
**预计工期**: 3 天

**目标**: 完整实现 `@file` `/command` `!shell` 三种快捷输入解析器

**子任务**:
- [ ] TASK-1.5.1: 实现 @file 解析器
  - 文件路径解析
  - 文件内容加载
  - @file 快捷提示
- [ ] TASK-1.5.2: 实现 /command 解析器
  - 命令触发
  - 参数解析
  - /command 快捷提示
- [ ] TASK-1.5.3: 实现 !shell 解析器
  - Shell 命令执行
  - 输出捕获
  - !shell 快捷提示
- [ ] TASK-1.5.4: 实现快捷输入 UI 提示
  - 自动补全
  - 语法高亮

**验收标准**:
- [ ] @file 正确解析
- [ ] /command 正确解析
- [ ] !shell 正确解析
- [ ] 快捷提示可用

**依赖**: TASK-0.1

---

## 4. Phase 2: P2 优化项

### Task 2.1: 废弃字段清理

**ID**: TASK-2.1  
**优先级**: P2  
**模块**: config  
**状态**: pending  
**预计工期**: 1 天

**目标**: 清理 mode 和 layout 废弃字段

**子任务**:
- [ ] TASK-2.1.1: 标记 mode 字段为 #[deprecated]
  - 添加 deprecation 属性
  - 添加废弃日志
- [ ] TASK-2.1.2: 标记 layout 字段为 #[deprecated]
  - 添加 deprecation 属性
  - 添加废弃日志
- [ ] TASK-2.1.3: 实现自动迁移提示
  - 检测到废弃字段时提示迁移方案

**验收标准**:
- [ ] 废弃字段有 deprecation 标记
- [ ] 编译时产生废弃警告
- [ ] 运行时产生迁移提示

**依赖**: 无

---

### Task 2.2: TUI 三栏/双栏切换

**ID**: TASK-2.2  
**优先级**: P2  
**模块**: tui  
**状态**: completed  
**预计工期**: 2 天

**目标**: 实现可切换布局

**子任务**:
- [x] TASK-2.2.1: 实现布局切换逻辑
  - Ctrl+L 快捷键绑定
  - 状态保存到 layout.txt
- [x] TASK-2.2.2: 实现三栏布局
  - 左: 文件树
  - 中: 编辑器/对话
  - 右: 诊断/工具 (RightPanel)
- [x] TASK-2.2.3: 实现双栏布局
  - 左: 文件树/对话
  - 右: 编辑器
- [x] TASK-2.2.4: 保存布局偏好
  - 写入 ~/.config/opencode-rs/layout.txt
  - 启动时恢复

**验收标准**:
- [x] 可切换三栏/双栏布局
- [x] 布局偏好可保存和恢复

**依赖**: 无

---

### Task 2.3: TUI 右栏功能完善

**ID**: TASK-2.3  
**优先级**: P2  
**模块**: tui  
**状态**: completed  
**预计工期**: 2 天

**目标**: 完善右栏面板 (diagnostics/todo/权限队列)

**子任务**:
- [x] TASK-2.3.1: 实现 Diagnostics 面板
  - LSP 诊断显示
  - 错误/警告分类
- [x] TASK-2.3.2: 实现 Todo 面板
  - TodoWrite 列表显示
  - 完成标记
- [x] TASK-2.3.3: 实现权限队列面板
  - 待审批操作列表
  - 快速审批/拒绝

**验收标准**:
- [x] Diagnostics 面板正常显示
- [x] Todo 面板正常显示
- [x] 权限队列面板正常显示

**依赖**: 无

---

### Task 2.4: TUI Token/Cost 显示

**ID**: TASK-2.4  
**优先级**: P2  
**模块**: tui  
**状态**: completed  
**预计工期**: 1 天

**目标**: 实现 token 统计与成本显示

**子任务**:
- [x] TASK-2.4.1: 实现 token 统计
  - 输入/输出 token 计数
  - 会话总 token
- [x] TASK-2.4.2: 实现成本计算
  - Provider 定价计算
  - 成本显示
- [x] TASK-2.4.3: 实现 UI 显示
  - 状态栏显示 tokens + cost
  - Token popover 详细信息

**验收标准**:
- [x] Token 统计正常显示
- [x] Cost 统计正常显示

**依赖**: TASK-0.3

---

### Task 2.5: TUI Patch 预览展开

**ID**: TASK-2.5  
**优先级**: P2  
**模块**: tui  
**状态**: completed  
**预计工期**: 2 天

**目标**: 实现 diff 展开/收起交互

**子任务**:
- [x] TASK-2.5.1: 实现 patch 预览
  - Diff 渲染为 hunk 结构
  - 语法高亮
- [x] TASK-2.5.2: 实现展开/收起交互
  - e: 全部展开
  - c: 全部收起
  - space: 切换当前 hunk
  - 折叠时显示摘要 (文件名, +/-行数)
- [x] TASK-2.5.3: 实现批量操作
  - 全部展开/收起快捷键

**验收标准**:
- [x] Patch 预览正常显示
- [x] 展开/收起交互正常

**依赖**: 无

---

### Task 2.6: scroll_acceleration 结构修复

**ID**: TASK-2.6  
**优先级**: P2  
**模块**: config  
**状态**: completed  
**预计工期**: 1 天

**当前实现**: `f32` 类型  
**PRD 要求**: `{ enabled: true, speed: 1.0 }`

**子任务**:
- [x] TASK-2.6.1: 修改 ScrollAccelerationConfig 结构
- [x] TASK-2.6.2: 更新序列化/反序列化逻辑 - 自定义 Deserialize 支持旧格式 `1.0` 和新格式 `{"enabled": true, "speed": 1.0}`
- [x] TASK-2.6.3: 添加单元测试 - 3个测试覆盖新旧格式

**验收标准**:
- [x] 新配置格式 `{ enabled: true }` 可正确解析
- [x] 旧配置格式 `1.0` 自动转换为 `{ enabled: true, speed: 1.0 }`
- [x] 单元测试覆盖新旧格式

**依赖**: 无

---

### Task 2.7: keybinds 自定义绑定支持

**ID**: TASK-2.7  
**优先级**: P2  
**模块**: config  
**状态**: completed  
**预计工期**: 2 天

**子任务**:
- [x] TASK-2.7.1: 扩展 KeybindConfig 结构 - 已有 custom 字段
- [x] TASK-2.7.2: 实现自定义绑定解析 - merge_with_defaults()
- [x] TASK-2.7.3: 实现绑定冲突检测 - 返回 conflicts Vec<String>

**验收标准**:
- [x] 支持自定义 keybinds 配置
- [x] 自定义绑定可覆盖默认绑定
- [x] 冲突绑定给出警告

**依赖**: 无

---

### Task 2.8: 测试覆盖补充

**ID**: TASK-2.8  
**优先级**: P2  
**模块**: test  
**状态**: completed  
**预计工期**: 3 天

**子任务**:
- [x] TASK-2.8.1: 添加 Skills 加载与匹配 E2E 测试
- [x] TASK-2.8.2: 添加 Command 模板展开测试 - env vars, cursor
- [x] TASK-2.8.3: 添加 Session 分叉测试 - fork, token estimation
- [x] TASK-2.8.4: 添加 Share 导出测试 - JSON/Markdown/sanitize
- [x] TASK-2.8.5: 添加 scroll_acceleration 双格式测试
- [x] TASK-2.8.6: 添加 keybinds merge + conflict 测试

**验收标准**:
- [x] 所有新增测试通过
- [x] cargo check --workspace 无错误

**依赖**: 相关功能实现

---

## 5. 任务状态追踪

| Phase | Task ID | 任务名称 | 状态 | 优先级 | 预计工期 |
|-------|---------|----------|------|--------|----------|
| 0 | TASK-0.1 | Commands 系统 | ✅ completed | P0 | 3d |
| 0 | TASK-0.2 | Skills 系统 | ✅ completed | P0 | 3d |
| 0 | TASK-0.3 | Context Engine | ✅ completed | P0 | 5d |
| 0 | TASK-0.4 | 多层配置合并 | ✅ completed | P0 | 3d |
| 0 | TASK-0.5 | .opencode 目录加载 | ✅ completed | P0 | 3d |
| 1 | TASK-1.1 | Session Fork | ✅ completed | P1 | 2d |
| 1 | TASK-1.2 | Share 功能 | ✅ completed | P1 | 3d |
| 1 | TASK-1.3 | Provider API 完善 | ✅ completed | P1 | 2d |
| 1 | TASK-1.4 | Session Summarize | ✅ completed | P1 | 2d |
| 1 | TASK-1.5 | TUI 快捷输入 | ✅ completed | P1 | 3d |
| 2 | TASK-2.1 | 废弃字段清理 | ✅ completed | P2 | 1d |
| 2 | TASK-2.2 | TUI 三栏/双栏切换 | ✅ completed | P2 | 2d |
| 2 | TASK-2.3 | TUI 右栏功能 | ✅ completed | P2 | 2d |
| 2 | TASK-2.4 | TUI Token/Cost 显示 | ✅ completed | P2 | 1d |
| 2 | TASK-2.5 | TUI Patch 预览展开 | ✅ completed | P2 | 2d |
| 2 | TASK-2.6 | scroll_acceleration 修复 | ✅ completed | P2 | 1d |
| 2 | TASK-2.7 | keybinds 自定义绑定 | ✅ completed | P2 | 2d |
| 2 | TASK-2.8 | 测试覆盖补充 | ✅ completed | P2 | 3d |

**总计**: 18 tasks, 43 人天 | ✅ 18 completed | 0 pending

---

## 6. 验收检查清单

每个任务完成后需满足:

- [ ] 功能正常运行
- [ ] 错误处理正确
- [ ] 性能满足要求
- [ ] 文档完整
- [ ] 测试覆盖

---

**文档状态**: 已更新  
**下一步**: 等待 TUI 三栏/右栏/Patch预览 实现完成 (bg_60364929)

