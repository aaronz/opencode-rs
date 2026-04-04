# Constitution 审计报告 (v1.3 → v1.4)

**日期**: 2026-04-04  
**审计范围**: Constitution v1.3 vs 当前实现状态  
**审计依据**: PRD-OpenCode-Configuration.md, docs/gap-analysis-prd-vs-rust.md, outputs/iteration-3/gap-analysis.md

---

## 一、审计结论

### Constitution v1.3 状态: **基本覆盖，需小幅更新**

| 指标 | 值 |
|------|-----|
| Constitution 条款总数 | 19 (C-001 ~ C-019) |
| 已覆盖 P0/P1 问题 | 12/12 (100%) |
| 新增需覆盖问题 | 3 (来自 PRD 新增章节) |
| 建议新增条款 | C-020, C-021, C-022 |

### 关键发现

1. **Constitution 不存在于项目根目录** — 仅存在于 `outputs/iteration-3/constitution_updates.md`，未被 AGENTS.md 或项目规范引用
2. **Iteration-4 Gap Analysis 未完成** — 仅包含 agent 对话残留，无实际分析内容
3. **PRD 新增 3 个配置领域未被 Constitution 覆盖** — Server/mDNS、Compaction、Watcher

---

## 二、现有条款覆盖度验证

### 2.1 P0 问题覆盖 (v1.3 已覆盖)

| P0 问题 | Constitution 条款 | 覆盖状态 |
|---------|-------------------|----------|
| 缺少 `OPENCODE_TUI_CONFIG` 环境变量 | C-017 §3-4 | ✅ 完整覆盖 |
| TUI 配置未分离为独立 tui.json | C-017 §1-2 | ✅ 完整覆盖 |

### 2.2 P1 问题覆盖 (v1.3 已覆盖)

| P1 问题 | Constitution 条款 | 覆盖状态 |
|---------|-------------------|----------|
| 缺少 `modes/` 目录扫描 | C-013 细化 §1-3 | ✅ 完整覆盖 |
| 配置路径命名 `opencode-rs` vs `opencode` | C-018 §1-3 | ✅ 完整覆盖 |
| `{file:path}` 不支持 `~` 展开 | C-019 §3 | ✅ 完整覆盖 |
| `{file:path}` 不支持相对路径 | C-019 §3 | ✅ 完整覆盖 |
| `.opencode/` 目录扫描未集成 | C-013 细化 §3 | ✅ 完整覆盖 |

### 2.3 P2 技术债务 (v1.3 未约束，正确)

| P2 问题 | Constitution 处理 | 评价 |
|---------|-------------------|------|
| JSON Schema 远程验证未实现 | 建议后续增加 C-020 | ✅ 正确 — 功能缺失非设计约束 |
| AgentMapConfig 固定键 | 建议后续修订 C-011 | ✅ 正确 — 可用 custom flatten 绕过 |
| merge_configs JSON 中转 | 代码重构关注 | ✅ 正确 — 实现细节 |
| fetch_remote_config 同步包装 | 代码重构关注 | ✅ 正确 — 实现细节 |
| 测试覆盖不足 | 工程实践关注 | ✅ 正确 — 工程质量非设计约束 |

---

## 三、新增 Constitution 条款 (v1.4)

以下 3 个配置领域在 PRD 中明确要求，但 Constitution v1.3 未覆盖：

### 3.1 Server 配置规范 (C-020)

**条款 C-020: Server 配置规范**

```
1. Server 配置项 (PRD §5.1):
   a) port: 监听端口号，默认 4096
   b) hostname: 监听地址，默认 "0.0.0.0"
   c) mdns: 是否启用 mDNS 服务发现，默认 true
   d) mdnsDomain: mDNS 域名，可选
   e) cors: CORS 允许的源列表

2. mDNS 规范:
   a) 启用 mdns 时，必须在局域网内广播服务
   b) mdnsDomain 格式必须符合 mDNS 命名规范 (以 .local 结尾)
   c) 未设置 mdnsDomain 时，使用默认格式 "opencode.local"

3. CORS 规范:
   a) cors 为空列表时，允许所有源 (开发模式)
   b) cors 非空时，仅允许列表中的源
   c) 生产环境必须配置 cors 白名单

4. 安全约束:
   a) port 必须在 1024-65535 范围内 (非特权端口)
   b) hostname 禁止设置为 "0.0.0.0" 在生产环境 (应使用 "127.0.0.1" 或具体 IP)
```

### 3.2 Compaction 配置规范 (C-021)

**条款 C-021: 会话 Compaction 配置规范**

```
1. Compaction 配置项 (PRD §5.9):
   a) auto: 是否自动压缩会话，默认 true
   b) prune: 是否移除旧工具输出以节省 token，默认 true
   c) reserved: 压缩时保留的 token 缓冲，默认 10000

2. 自动压缩触发条件:
   a) 会话 token 数接近模型上下文窗口限制时触发
   b) 触发阈值 = 模型最大上下文 - reserved
   c) 压缩必须保持对话语义连贯性

3. Prune 规范:
   a) prune 启用时，移除旧的工具输出内容，保留工具调用记录
   b) 保留最近的 N 个工具调用完整输出 (N 由实现决定，建议 >= 3)
   c) 被 prune 的内容必须标记为 "[content pruned to save tokens]"

4. 安全约束:
   a) reserved 必须 > 0，防止压缩后无剩余 token
   b) reserved 建议 >= 5000，确保模型有足够空间响应
```

### 3.3 Watcher 配置规范 (C-022)

**条款 C-022: 文件 Watcher 配置规范**

```
1. Watcher 配置项 (PRD §5.10):
   a) ignore: glob 模式的忽略列表

2. 忽略模式规范:
   a) 支持 glob 语法 (*, **, ?)
   b) 必须默认忽略: .git/**, node_modules/**, dist/**, build/**
   c) ignore 列表为用户自定义追加，非替换默认值

3. 性能约束:
   a) 单个目录下的监视文件数不超过 10000
   b) 忽略模式必须在文件系统层面生效 (而非事件过滤)
   c) 监视器启动失败 (如 inotify 限制) 应记录 warning 但不阻断启动
```

---

## 四、条款更新映射 (v1.4)

### 新增条款

| 条款 | 模块 | 覆盖 PRD 章节 |
|------|------|---------------|
| C-020 | Server 配置 | PRD §5.1 |
| C-021 | Compaction 配置 | PRD §5.9 |
| C-022 | Watcher 配置 | PRD §5.10 |

### 保持不变

| 条款 | 说明 |
|------|------|
| C-001 | 已废止 (被 C-016 替代) |
| C-002 ~ C-019 | 不受本次更新影响 |

---

## 五、验证清单 (v1.4 新增)

### C-020: Server 配置
- [ ] port 是否在 1024-65535 范围内
- [ ] mdns 启用时是否正确广播服务
- [ ] cors 非空时是否仅允许列表中的源
- [ ] 生产环境 hostname 是否禁止 "0.0.0.0"

### C-021: Compaction 配置
- [ ] reserved 是否 > 0
- [ ] prune 启用时是否正确标记被移除内容
- [ ] 压缩触发阈值是否正确计算

### C-022: Watcher 配置
- [ ] 默认忽略列表是否生效
- [ ] glob 模式是否正确解析
- [ ] 监视器启动失败是否不阻断启动

---

## 六、修订历史

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| 1.0 | 2026-04-04 | 初始版本，覆盖 Context/Plugin/Skills/Commands/MCP |
| 1.1 | 2026-04-04 | 新增 Config System 条款 (C-011, C-012, C-013) |
| 1.2 | 2026-04-04 | 新增 TUI Input Parser (C-014), Session Fork (C-015), Context Token Budget (C-016) |
| 1.3 | 2026-04-04 | 新增 TUI 配置分离 (C-017), 路径命名 (C-018), 文件引用变量 (C-019), 细化 C-013 |
| **1.4** | **2026-04-04** | **新增 Server 配置 (C-020), Compaction (C-021), Watcher (C-022)** |

---

## 七、设计决策约束 (v1.4 更新版)

| 设计决策 | 必须遵循条款 |
|----------|-------------|
| Config 系统实现 | C-011, C-012, C-013 (含 modes/), C-017, C-018, C-019 |
| TUI 配置实现 | C-014, C-017 |
| 变量替换实现 | C-012, C-019 |
| 目录扫描实现 | C-013 (细化版) |
| 路径获取实现 | C-018 |
| **Server 实现** | **C-020** |
| **Compaction 实现** | **C-021** |
| **Watcher 实现** | **C-022** |

---

## 八、建议 (非 Constitution 约束)

### 8.1 Constitution 位置建议

当前 Constitution 仅存在于 `outputs/iteration-3/constitution_updates.md`，建议：

1. **移动到项目根目录**: `.sisyphus/constitution.md` 或 `docs/constitution.md`
2. **在 AGENTS.md 中引用**: 确保 AI agent 启动时加载 Constitution
3. **建立版本控制**: 每次迭代更新时递增版本号

### 8.2 Iteration-4 Gap Analysis 修复

Iteration-4 的 `outputs/iteration-4/gap-analysis.md` 仅包含 agent 对话残留 (5 行)，无实际分析内容。建议：

1. 重新执行完整的 gap analysis
2. 对照 PRD 逐项验证实现状态
3. 更新实现完整度百分比

---

*本文档作为 OpenCode-RS 项目的 Constitution v1.4 更新建议，聚焦 Server/Compaction/Watcher 配置领域的 PRD 覆盖。*
