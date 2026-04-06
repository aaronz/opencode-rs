# Constitution 审计报告 (v1.7 → v1.8)

**日期**: 2026-04-06  
**审计范围**: Constitution v1.7 (C-001 ~ C-037) vs 差距分析 iteration-8 新发现的 P0 问题  
**审计依据**: 差距分析报告 iteration-8 (outputs/iteration-8/gap-analysis.md)

---

## 一、审计结论

### Constitution v1.7 状态: ❌ 需要更新

| 指标 | 值 |
|------|-----|
| Constitution 条款总数 | 37 (C-001 ~ C-037, C-001 已废止) |
| iteration-8 新发现 P0 问题数 | **3** |
| **P0 被现有条款覆盖** | **0/3 (0%)** |
| 建议修改条款 | C-024, C-034 (各需扩展覆盖) |
| 建议新增条款 | C-038 (Provider API), C-039 (Permission API), C-040 (Session State) |

### 关键发现

1. **iteration-7 vs iteration-8 P0 问题不相同** — iteration-7 审计的 P0 问题 (OAuth/Device Code, Provider 认证分层) 与 iteration-8 新增的 P0 问题 (Provider API, Permission API, Session State) 是完全不同的新问题
2. **C-024 (Permission) 仅覆盖评估层** — 包含权限级别 (allow/ask/deny) 和审计日志，但**不包含**审批 API (POST /permissions/{id}/reply)
3. **C-034 (TUI 状态机) 与 Session State 无关** — 定义的是 TUI 交互状态 (如 idle/thinking/awaiting_permission)，不是 Session 业务状态机
4. **Provider API 完全无约束** — C-030 定义了 Provider 配置，但**不包含** REST API 端点 (credential/test/revoke)

---

## 二、iteration-8 P0 问题详细分析

### 2.1 P0-1: Provider 管理 API 缺失

| 检查项 | 说明 | Constitution 覆盖 |
|--------|------|-------------------|
| POST /providers/{id}/credentials | 添加/更新 provider 凭证 | ❌ 无 |
| POST /providers/{id}/test | 测试凭证有效性 | ❌ 无 |
| POST /providers/{id}/revoke | 撤销凭证 | ❌ 无 |

**根本原因**: C-030 (Provider) 定义了配置模型和认证机制，但未定义 Server API 层。

### 2.2 P0-2: Permission 审批 API 缺失

| 检查项 | 说明 | Constitution 覆盖 |
|--------|------|-------------------|
| POST /permissions/{req_id}/reply | 审批请求回复 (approve/deny) | ❌ 无 |
| GET /permissions/pending | 查询待审批请求 | ❌ 无 |
| GET /permissions/history | 查询审批历史 | ❌ 无 |

**根本原因**: C-024 (Permission) 定义了权限评估 (PermissionEvaluator)，但未定义审批工作流 API。

### 2.3 P0-3: Session State 状态机不完整

| 检查项 | 说明 | Constitution 覆盖 |
|--------|------|-------------------|
| 12 种业务状态 | idle/thinking/awaiting_permission/paused/completed/error 等 | ❌ 无 |
| 状态转换规则 | 状态间合法转换矩阵 | ❌ 无 |
| 状态持久化 | 状态恢复机制 | ❌ 无 |

**根本原因**: C-034 §3-4 定义的是 TUI 交互状态机，不是 Session 业务状态机。两者是不同概念：

- **TUI 状态机**: 用户可见的 UI 状态 (输入模式、面板可见性)
- **Session 状态机**: 业务逻辑状态 (AI 是否正在思考、是否等待权限、是否暂停)

---

## 三、差距分析 P0 问题映射

| iteration-8 P0 问题 | Constitution 覆盖 | 验证结论 |
|---------------------|-------------------|----------|
| **P0-1: Provider 管理 API** | ❌ 无条款覆盖 | **需新增 C-038** |
| **P0-2: Permission 审批 API** | ⚠️ C-024 仅覆盖评估，未覆盖审批工作流 | **需扩展 C-024 或新增 C-039** |
| **P0-3: Session State 12 状态** | ⚠️ C-034 仅覆盖 TUI 状态，非 Session 状态 | **需新增 C-040** |

---

## 四、Constitution v1.8 修订建议

### 4.1 扩展 C-024: Permission 审批工作流 (C-024 §6 新增)

```markdown
### §6. Permission 审批工作流 API

1. API 端点:
   a) POST /permissions/{req_id}/reply — 审批请求回复
      - body: { "decision": "approve|deny", "note": "optional" }
      - response: { "status": "processed", "request_id": "..." }
   b) GET /permissions/pending — 查询待审批请求
      - response: [{ "id": "...", "tool": "...", "reason": "...", "timestamp": "..." }]
   c) GET /permissions/history — 查询审批历史
      - query: ?limit=50&offset=0
      - response: [{ "id": "...", "decision": "...", "timestamp": "..." }]

2. 审批语义:
   a) approve — 授权本次请求，允许后续同类请求 (基于 tool + 路径)
   b) deny — 拒绝本次请求，记录拒绝原因

3. 审批影响:
   a) 用户批准后，请求执行并记录到 AuditLog
   b) 用户拒绝后，请求终止并记录到 AuditLog
   c) 审批结果可设置 "记住决定" (save_decision=true) 自动应用于后续请求

4. 安全约束:
   a) 审批 API 需认证 (C-026 §5: Runtime Access Control)
   b) deny 决策不可被覆盖
   c) 审批超时默认拒绝 (timeout: 5min)
```

### 4.2 新增 C-038: Provider 管理 API

```markdown
### 条款 C-038: Provider 管理 API 规范

1. API 端点:
   a) POST /providers — 注册新 provider
      - body: { "type": "openai|anthropic|ollama|...", "config": {...} }
      - response: { "id": "...", "status": "created" }
   
   b) GET /providers — 列表所有 provider
      - query: ?enabled=true|false (过滤)
      - response: [{ "id": "...", "type": "...", "enabled": true }]
   
   c) GET /providers/{id} — 获取 provider 详情
      - response: { "id": "...", "type": "...", "enabled": true, "auth_method": "..." }
   
   d) PUT /providers/{id} — 更新 provider 配置
      - body: { "enabled": false, "config": {...} }
      - response: { "status": "updated" }
   
   e) DELETE /providers/{id} — 删除 provider
      - response: { "status": "deleted" }
   
   f) POST /providers/{id}/credentials — 添加/更新凭证
      - body: { "credential": "api_key|oauth|refresh_token|...", "value": "..." }
      - response: { "status": "stored" }
   
   g) POST /providers/{id}/test — 测试凭证有效性
      - response: { "valid": true|false, "message": "..." }
   
   h) POST /providers/{id}/revoke — 撤销凭证
      - response: { "status": "revoked" }

2. 凭证存储:
   a) 凭证加密存储 (C-026 §6)
   b) 凭证不通过 API 返回 (敏感信息掩码)
   c) 支持多套凭证切换

3. 认证要求:
   a) 所有端点需认证 (C-026 §5: Runtime Access Control)
   b) credential 端点需 admin 角色

4. 审计要求:
   a) 凭证操作记录到 AuditLog
   b) test 操作记录成功/失败
```

### 4.3 新增 C-040: Session 业务状态机

```markdown
### 条款 C-040: Session 业务状态机规范

1. 状态定义 (12 状态):
   a) idle — 空闲，等待用户输入
   b) thinking — AI 正在处理请求
   c) awaiting_permission — 等待用户授权工具执行
   d) executing — 工具执行中
   e) awaiting_response — 等待用户响应 (如 confirmDialog)
   f) paused — 暂停 (用户中断或 session 恢复)
   g) completed — 完成 (正常结束)
   h) error — 错误 (执行失败)
   i) interrupted — 中断 (Ctrl+C 或强制终止)
   j) transferring — 传输中 (如 session 迁移)
   k) reconnecting — 重连中 (网络中断恢复)
   l) waiting_tool — 等待工具结果

2. 状态转换规则:
   a) idle → thinking (用户发送消息)
   b) thinking → awaiting_permission (需要授权)
   c) awaiting_permission → executing (用户批准)
   d) awaiting_permission → error (用户拒绝)
   e) executing → waiting_tool (工具调用)
   f) waiting_tool → thinking (工具返回结果)
   g) thinking → idle (生成回复完成)
   h) any → paused (用户暂停)
   i) paused → idle (用户恢复)
   j) any → error (执行异常)
   k) any → interrupted (用户中断)
   l) waiting_tool → error (工具执行失败)

3. 状态持久化:
   a) 每次状态转换记录到 storage
   b) Session 恢复时恢复最后状态
   c) 状态历史可通过 API 查询

4. 状态可见性:
   a) 状态通过 SSE 推送到 TUI/Web UI
   b) 状态通过 GET /sessions/{id}/status API 可查询
   c) 状态变化触发对应 UI 更新

5. 安全约束:
   a) error 状态需记录错误详情 (不包含敏感信息)
   b) interrupted 状态需保存 checkpoint
   c) paused 状态需释放资源但保留上下文
```

---

## 五、与 iteration-7 对比

| 指标 | iteration-7 (v1.7) | iteration-8 (v1.8) | 变化 |
|------|-------------------|---------------------|------|
| P0 覆盖率 | 100% (OAuth/Device Code) | 0% (Provider/Permission/Session API) | ❌ 新 P0 未覆盖 |
| Constitution 版本 | v1.7 | v1.7 | → 需更新 |
| 需修订条款 | 无 | C-024, C-034 | ✅ 增加 |
| 需新增条款 | 无 | C-038, C-039, C-040 | ✅ 增加 |

**注意**: iteration-7 审计结论声称 "P0 100% 覆盖" 是因为当时审计的 P0 问题与 iteration-8 不同。两次审计针对的是不同的 P0 问题集合，不能简单认为 Constitution 已完备。

---

## 六、审计结论

### ❌ Constitution v1.8 需要更新

**理由**:
1. **iteration-8 新增 3 个 P0 问题完全未被覆盖** — Provider API、Permission 审批 API、Session State 状态机在现有 C-001~C-037 中无对应条款
2. **C-024 需扩展** — 当前仅定义权限评估，需新增 §6 定义审批工作流 API
3. **C-034 被误用于 Session 状态机** — 实际定义的是 TUI 状态机，与 Session 业务状态机不同
4. **Provider API 无任何约束** — C-030 仅定义配置层，未定义 Server API

### 修订计划

| 条款 | 操作 | 说明 |
|------|------|------|
| C-024 | 扩展 | 新增 §6: Permission 审批工作流 API |
| C-038 | 新增 | Provider 管理 API 规范 |
| C-039 | 合并 | 可合并到 C-024 §6 |
| C-040 | 新增 | Session 业务状态机规范 |

---

## 七、修订历史

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| 1.7 | 2026-04-05 | 审计确认: OAuth/Device Code P0 100% 覆盖 |
| **1.8** | **2026-04-06** | **新增 P0 问题: Provider API, Permission API, Session State** |

---

*本文档识别 iteration-8 差距分析中的 3 个新 P0 问题需要 Constitution v1.8 新增条款覆盖。*
