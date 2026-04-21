# Gap Analysis Report: Module `global`

**Iteration**: 37
**Date**: 2026-04-21
**Module**: `opencode-core::global`
**Source File**: `opencode-rust/crates/core/src/global.rs`

---

## 1. 差距列表 (Gap List)

| 差距项 | 严重程度 | 模块 | 修复建议 |
|--------|----------|------|----------|
| **缺失测试模块** - PRD 规定了 6 个测试用例，但 `global.rs` 中完全没有 `#[cfg(test)]` 模块 | P0 | `global.rs` | 添加完整的 `mod tests` 模块，实现所有 PRD 指定的测试用例 |
| **GlobalState 未公开** - `lib.rs:178` 使用 `pub(crate)` 而非 `pub`，外部 crate 无法使用 `GlobalState` | P0 | `lib.rs` | 将 `pub(crate) use global::GlobalState;` 改为 `pub use global::GlobalState;` |
| **缺失 subscriber_count 方法** - `GlobalState` 没有提供获取 `event_bus` 订阅者数量的方法 | P1 | `global.rs` | 添加 `pub fn subscriber_count(&self) -> usize { self.event_bus.subscriber_count() }` |
| **缺失 Session::new() 方法** - 测试用例使用 `Session::new()`，但 session 模块可能没有这个方法 | P1 | `session.rs` | 检查 `Session::new()` 是否存在，如不存在则添加 |
| **未实现扩展模式** - PRD 描述了 `tool_registry`, `plugin_manager`, `lsp_manager` 扩展字段，但代码中不存在 | P2 | `global.rs` | 如需要这些扩展字段，按 PRD 示例添加到结构体中 |
| **文档缺失** - 缺少 `GlobalState` 的 doc comments | P2 | `global.rs` | 添加模块级和结构体级文档注释 |
| **Usage pattern 未实现** - PRD 中的使用示例代码没有实际实现/测试 | P2 | `global.rs` | 添加文档注释展示正确的使用模式 |

---

## 2. P0/P1/P2 问题分类

### P0 阻断性问题 (2项)

1. **缺失测试模块**
   - 状态: 未实现
   - 影响: 无法验证 `GlobalState` 的核心功能正确性
   - 修复方案:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       use crate::config::Config;

       #[test]
       fn new_global_state_has_no_session() {
           let state = GlobalState::new(Config::default());
           assert!(state.current_session.is_none());
       }

       #[test]
       fn event_bus_is_initialized() {
           let state = GlobalState::new(Config::default());
           assert_eq!(state.event_bus.subscriber_count(), 0);
       }

       // ... 其他 4 个测试
   }
   ```

2. **GlobalState 未公开**
   - 当前: `pub(crate) use global::GlobalState;`
   - 应为: `pub use global::GlobalState;`
   - 影响: 其他 crate 无法使用 `GlobalState`，违反了设计意图

### P1 高优先级问题 (2项)

1. **subscriber_count 访问受限**
   - 需要通过 `event_bus.subscriber_count()` 间接访问
   - 应在 `GlobalState` 层直接暴露

2. **Session::new() 方法验证**
   - 需要确认 `Session::new()` 存在且行为符合测试预期

### P2 中优先级问题 (3项)

1. 扩展模式字段（tool_registry, plugin_manager, lsp_manager）
2. 文档注释
3. Usage pattern 文档

---

## 3. 技术债务清单

| 技术债务项 | 描述 | 修复成本 |
|------------|------|----------|
| **测试债务** | `global.rs` 完全没有任何测试覆盖 | 低 |
| **可见性债务** | `GlobalState` 被限制为 crate-internal，无法被外部使用 | 低 |
| **文档债务** | 缺少 API 文档和使用示例 | 低 |
| **扩展性债务** | 未实现 PRD 中描述的扩展模式 | 中 |

---

## 4. 实现进度总结

### 总体进度

| 维度 | 状态 | 说明 |
|------|------|------|
| 功能完整性 | ⚠️ 部分实现 | 核心结构存在，但缺少测试 |
| 接口完整性 | ⚠️ 不完整 | GlobalState 未公开，关键方法缺失 |
| 前端完整性 | N/A | 本模块为后端核心库，无前端组件 |
| 数据模型 | ✅ 已实现 | GlobalState 包含 config, event_bus, current_session |
| 配置管理 | ✅ 已实现 | 通过 Config 类型支持 |
| 测试覆盖 | ❌ 缺失 | 0% 覆盖 |

### 当前实现（20行）

```rust
// opencode-rust/crates/core/src/global.rs (当前)
pub struct GlobalState {
    pub config: Config,
    pub event_bus: Arc<EventBus>,
    pub current_session: Option<Session>,
}

impl GlobalState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            event_bus: Arc::new(EventBus::new()),
            current_session: None,
        }
    }
}
```

### PRD 要求（完整实现）

```rust
// 包含所有 6 个测试 + 扩展字段 + 文档 + 公开 API
```

### 进度百分比

| 项目 | 当前 | 目标 | 进度 |
|------|------|------|------|
| 核心结构 | ✅ 100% | 100% | 完成 |
| 测试覆盖 | ❌ 0% | 100% | 缺失 |
| 公开 API | ⚠️ 50% | 100% | 部分完成 |
| 文档 | ❌ 0% | 100% | 缺失 |
| **总体** | **~35%** | **100%** | **待完善** |

---

## 5. 修复优先级

1. **立即修复 (P0)**:
   - 将 `GlobalState` 公开为 `pub use`
   - 添加完整的测试模块

2. **短期修复 (P1)**:
   - 添加 `subscriber_count()` 方法
   - 验证 `Session::new()` 存在

3. **中期修复 (P2)**:
   - 添加文档注释
   - 实现扩展模式字段（如果需要）

---

## 6. 建议行动

1. **添加测试** - 按照 PRD 指定的 6 个测试用例实现测试
2. **修复可见性** - 修改 `lib.rs` 中 `GlobalState` 的可见性
3. **添加辅助方法** - 在 `GlobalState` 上暴露常用操作的快捷方法
4. **完善文档** - 添加模块和结构体文档

---

*Report generated: 2026-04-21*
*Next iteration: 38*