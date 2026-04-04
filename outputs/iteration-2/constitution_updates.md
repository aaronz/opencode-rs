# Constitution 更新建议 (v1.2)

## 背景

Gap Analysis 识别了 5 个 P0 问题，其中:
- **已覆盖**: P0-1 (Commands), P0-4 (Config) → C-007, C-008, C-011, C-012
- **部分覆盖**: P0-5 (Context Token Budget) → C-001, C-002 需细化
- **未覆盖**: P0-2 (TUI Input Parser), P0-3 (Session Fork) → 需新增条款

---

## 一、新增 P0 问题对照

### Gap Analysis P0 问题 vs Constitution

| P0 问题 | Constitution 覆盖 | 状态 |
|---------|------------------|------|
| P0-1: Commands 系统 | C-007, C-008 | ✅ 已覆盖 |
| P0-2: `@file` / `!shell` / `/command` | ❌ | 需新增 C-014 |
| P0-3: Session Fork | ❌ | 需新增 C-015 |
| P0-4: 多层配置合并 | C-011, C-012, C-013 | ✅ 已覆盖 |
| P0-5: Context Token Budget | ⚠️ C-001, C-002 | 需细化 C-016 |

---

## 二、新增 Constitution 条款

### 2.1 TUI Input Parser (C-014)

**条款 C-014: TUI 快捷输入解析器**

```
1. 输入解析器类型:
   a) @file - 文件引用: @path/to/file 或 @line:col
   b) !shell - 内联 Shell: !<command> 或 !<command> | <pipe>
   c) /command - 命令: /<command> [args]

2. 解析优先级:
   - @file 和 !shell 优先于普通文本
   - /command 单独一行或行首触发
   - 混合输入: "修复 @src/main.rs 的 bug" → 解析为 "修复 [file:src/main.rs] 的 bug"

3. 变量替换规则:
   - ${file} - 当前文件路径
   - ${line} - 当前行号
   - ${selected} - 选中文本
   - ${pwd} - 当前工作目录

4. 错误处理:
   - 无效路径: 提示文件不存在，可创建
   - 无效命令: 提示命令未找到
   - 超长输入: 截断并提示

5. 快捷键绑定:
   - Ctrl+G: 打开文件选择器 (@file)
   - Ctrl+!: 打开 shell 行 (!shell)
   - Ctrl+/ 或 Esc+: 打开命令面板 (/command)
```

### 2.2 Session Fork (C-015)

**条款 C-015: Session Fork 与 Lineage 追踪**

```
1. Fork 语义:
   - Fork 创建当前 session 的完整副本
   - Fork 后独立演进，互不影响
   - 支持多级 fork 形成 lineage 图

2. 数据模型:
   - Session 表新增字段: parent_session_id (nullable)
   - Fork 操作: INSERT INTO sessions ... SELECT * WHERE id = ? + 更新 parent_session_id
   - Lineage 查询: 递归查询 parent_session_id 构建树

3. Fork 触发方式:
   - API: POST /sessions/{id}/fork
   - TUI: /fork 命令
   - 自动: 冲突检测后建议 fork

4. Fork 限制:
   - 最大深度: 10 级 (防止恶意滥用)
   - 并发 fork: 同一 session 最多 5 个子 fork
   - 命名规则: {original_name}-fork-{timestamp}

5. Fork 合并:
   - 支持将子 session 合并回父 session
   - 合并策略: 保留父 session + 追加差异
```

### 2.3 Context Token Budget (C-016) - 细化

**条款 C-016: Context Token Budget 压缩机制** (替代/细化 C-001)

```
1. Token Budget 定义:
   - 预算上限: 128K tokens (可配置)
   - 软上限: 85% → 预警
   - 硬上限: 92% → 压缩
   - 强制上限: 95% → 建议新 session

2. 压缩策略 (按优先级):
   a) 合并重复系统消息 (System prompt deduplication)
   b) 压缩长对话摘要 (Conversation summary)
   c) 移除低 ranking 工具结果 (Low-value tool results)
   d) 截断历史消息 (Truncate history)

3. 预警机制:
   - 85% 时: UI 提示 "Session 接近 token 上限"
   - 92% 时: 自动触发压缩 + 提示 "已自动压缩上下文"
   - 95% 时: 阻止继续添加 + 强制要求 fork 或 new session

4. 压缩记录:
   - 每次压缩记录: timestamp, compression_type, tokens_saved
   - 可通过 /stats 查看压缩统计

5. 配置项:
   - context.max_tokens: 128000 (默认)
   - context.compression_threshold: 0.92
   - context.warning_threshold: 0.85
   - context.force_new_session_threshold: 0.95
```

---

## 三、条款更新映射

### 原有条款 (v1.0 ~ v1.1)

| 条款 | 模块 | 状态 |
|------|------|------|
| C-001 | Context Engine - 管理原则 | **已废止** → 被 C-016 替代 |
| C-002 | Context Engine - Ranking | 保留 |
| C-003 | Plugin System - 架构 | 保留 |
| C-004 | Plugin System - Event Bus | 保留 |
| C-005 | Skills System - 加载 | 保留 |
| C-006 | Skills System - Matching | 保留 |
| C-007 | Commands System - 设计 | 保留 |
| C-008 | Commands System - Template | 保留 |
| C-009 | MCP Integration - 接入 | 保留 |
| C-010 | MCP Integration - Permission | 保留 |
| C-011 | Config System - 优先级加载 | 保留 |
| C-012 | Config System - 环境变量 | 保留 |
| C-013 | Config System - 目录结构 | 保留 |

### 新增条款 (v1.2)

| 条款 | 模块 | 说明 |
|------|------|------|
| C-014 | TUI Input Parser | P0-2: @file/!shell/command |
| C-015 | Session Fork | P0-3: Session Fork Lineage |
| C-016 | Context Token Budget | P0-5: 压缩机制 (细化 C-001) |

---

## 四、验证清单 (更新版)

实现新功能时需验证:

### 原有模块
- [ ] C-002: Context Engine - Ranking 机制
- [ ] C-003~C-004: Plugin System
- [ ] C-005~C-006: Skills System
- [ ] C-007~C-008: Commands System
- [ ] C-009~C-010: MCP Integration
- [ ] C-011~C-013: Config System

### 新增模块 (v1.2)
- [ ] **C-014: TUI Input Parser**
  - [ ] @file 解析是否正确处理相对/绝对路径
  - [ ] !shell 解析是否支持管道操作
  - [ ] /command 解析是否支持参数
  - [ ] 变量替换 ${file}, ${line} 等是否正确
  - [ ] 快捷键绑定是否生效

- [ ] **C-015: Session Fork**
  - [ ] Fork 创建是否完整复制 session 数据
  - [ ] parent_session_id 是否正确设置
  - [ ] Lineage 追踪是否可查询
  - [ ] 最大深度限制是否生效
  - [ ] API /sessions/{id}/fork 是否实现

- [ ] **C-016: Context Token Budget**
  - [ ] Token 统计是否准确
  - [ ] 85% 预警是否触发
  - [ ] 92% 压缩是否自动执行
  - [ ] 95% 强制新 session 是否阻止输入

---

## 五、修订历史

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| 1.0 | 2026-04-04 | 初始版本，覆盖 P0 问题 (Context/Plugin/Skills/Commands/MCP) |
| 1.1 | 2026-04-04 | 新增 Config System 条款 (C-011, C-012, C-013)，覆盖 P0 配置问题 |
| 1.2 | 2026-04-04 | 新增 TUI Input Parser (C-014), Session Fork (C-015), Context Token Budget 细化 (C-016) |

---

## 六、实施建议

### 短期 (1-2 周)

1. **实现 C-014: TUI Input Parser**
   - 实现 @file 解析器
   - 实现 !shell 解析器
   - 实现 /command 解析器
   - 添加快捷键绑定

2. **实现 C-015: Session Fork**
   - 修改 Session 数据模型 (添加 parent_session_id)
   - 实现 /sessions/{id}/fork API
   - 实现 Fork UI 命令

### 中期 (2-4 周)

1. **实现 C-016: Context Token Budget**
   - 实现 token 统计模块
   - 实现压缩策略
   - 实现预警机制

2. **完善 C-014: 变量替换**
   - 实现 ${file}, ${line}, ${selected} 替换

### 长期 (4+ 周)

1. Fork 合并功能
2. Lineage 可视化
3. 压缩统计 Dashboard

---

## 七、设计决策约束

以下设计决策必须遵循 Constitution:

| 设计决策 | 必须遵循条款 |
|----------|-------------|
| Commands 系统实现 | C-007, C-008 |
| TUI 快捷输入实现 | C-014 |
| Session Fork 实现 | C-015 |
| Context 管理实现 | C-016 |
| Config 系统实现 | C-011, C-012, C-013 |
| MCP 集成实现 | C-009, C-010 |

---

*本文档作为 OpenCode-RS 项目的 Constitution v1.2，覆盖所有 P0 问题并提供设计约束。*