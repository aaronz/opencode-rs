# TUI 输入框布局修复与视觉回归测试设计

## 概述

修复 rust-opencode-port TUI 应用中输入框/聊天区域的布局问题，并添加基于 BufferDiff 的视觉回归测试。

## 问题分析

### 当前问题

1. **InputWidget 文本溢出**
   - 位置：`crates/tui/src/components/input_widget.rs`
   - 问题：`draw()` 方法不处理换行，长文本直接超出边界
   - 影响：终端宽度较小时，内容被截断或溢出

2. **硬编码布局值**
   - 位置：`crates/tui/src/app.rs` `draw_chat()`
   - 问题：状态栏宽度硬编码为 `30`，文件树宽度硬编码为 `30`
   - 影响：终端尺寸变化时布局不正确

3. **缺少边界检查**
   - 组件区域计算未考虑最小尺寸保护
   - 可能导致负数尺寸或零尺寸区域

4. **组件重叠**
   - messages_area、input_area、status_area 在小终端上可能重叠

## 解决方案

### 1. InputWidget 布局修复

**修改文件**: `crates/tui/src/components/input_widget.rs`

#### 1.1 添加滚动状态

```rust
pub struct InputWidget {
    // ... existing fields
    pub scroll_x: usize,        // 水平滚动偏移
    pub overflow_mode: OverflowMode,
}

pub enum OverflowMode {
    Truncate,    // 截断超出部分
    Scroll,      // 水平滚动
    Wrap,        // 自动换行（多行模式）
}
```

#### 1.2 修复 draw 方法

- 添加 `scroll_x` 偏移计算
- 添加边界检查，确保光标在可视区域内
- 多行模式支持（`multiline=true` 时启用）

```rust
pub fn draw(&self, f: &mut Frame, area: Rect, title: &str) {
    // 1. 计算可见区域
    let visible_width = area.width as usize;
    let content = self.get_content();
    
    // 2. 应用滚动偏移
    let visible_content = if self.scroll_x < content.len() {
        &content[self.scroll_x..]
    } else {
        ""
    };
    
    // 3. 渲染（保持现有逻辑，添加 scroll_x 支持）
}
```

### 2. draw_chat 布局修复

**修改文件**: `crates/tui/src/app.rs`

#### 2.1 使用 Constraint 替代硬编码

```rust
use ratatui::layout::{Constraint, Direction, Layout};

fn draw_chat(&mut self, f: &mut Frame) {
    let area = f.area();
    let theme = self.theme();
    
    // 使用百分比约束
    let constraints = match (self.show_file_tree, self.show_terminal) {
        (true, true) => vec![
            Constraint::Length(1),  // title
            Constraint::Min(10),   // messages + tools
            Constraint::Length(3), // input
            Constraint::Length(1), // status
            Constraint::Length(10), // terminal
        ],
        (true, false) => vec![
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(3),
            Constraint::Length(1),
        ],
        (false, true) => vec![
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(10),
        ],
        (false, false) => vec![
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(3),
            Constraint::Length(1),
        ],
    };
    
    let chunks = Layout::vertical(constraints)
        .split(main_area);
    
    // 使用 chunks[index] 替代手动 Rect 计算
}
```

#### 2.2 添加最小尺寸保护

```rust
fn calculate_min_size(area: Rect) -> (u16, u16) {
    let min_width = 40.min(area.width);
    let min_height = 10.min(area.height);
    (min_width, min_height)
}
```

### 3. 视觉回归测试

**修改文件**: `ratatui-testing/tests/visual_regression.rs`

#### 3.1 BufferDiff 集成测试

```rust
use ratatui_testing::{TestTerminal, BufferDiff};

#[test]
fn test_input_widget_layout_80x24() {
    let terminal = TestTerminal::with_size(80, 24);
    let theme = Theme::default();
    let input = InputWidget::new(theme);
    
    // 渲染到 buffer
    terminal.draw(|f| {
        input.draw(f, Rect::new(0, 0, 80, 3), "Input");
    });
    
    // 预期输出（简化的字符表示）
    let expected = vec![
        "┌────────────────────────────────────────────────────────────────┐",
        "│Input                                                         │",
        "└────────────────────────────────────────────────────────────────┘",
    ];
    
    // 比较
    let buffer = terminal.buffer();
    let result = BufferDiff::compare_buffer(&expected, buffer);
    
    assert!(result.stats.similarity > 95.0, 
        "Similarity: {}%", result.stats.similarity);
}
```

#### 3.2 测试场景

| 测试名称 | 终端尺寸 | 输入内容 | 预期 |
|---------|---------|---------|------|
| `input_empty_80x24` | 80x24 | 空 | 正确渲染空边框 |
| `input_short_text_80x24` | 80x24 | "hello" | 文本正确显示 |
| `input_long_text_80x24` | 80x24 | 超过80字符 | 正确截断/滚动 |
| `input_narrow_terminal_40x12` | 40x12 | "test" | 窄终端正确渲染 |
| `input_multiline_80x24` | 80x24 | 多行文本 | 换行正确 |

#### 3.3 聊天区域布局测试

```rust
#[test]
fn test_chat_layout_80x24() {
    let terminal = TestTerminal::with_size(80, 24);
    let mut app = App::new();
    app.add_message("Hello, world!".to_string(), true);
    app.add_message("This is a test message.".to_string(), false);
    
    terminal.draw(|f| app.draw(f));
    
    let buffer = terminal.buffer();
    // 验证各区域不重叠
    assert!(!areas_overlap(&messages_area, &input_area));
    assert!(!areas_overlap(&input_area, &status_area));
}
```

## 任务分解

### 阶段 1: InputWidget 修复

- [ ] 1.1 添加 scroll_x 和 OverflowMode 到 InputWidget
- [ ] 1.2 修改 draw() 方法支持滚动
- [ ] 1.3 添加多行模式支持
- [ ] 1.4 更新单元测试

### 阶段 2: draw_chat 修复

- [ ] 2.1 重构使用 Constraint-based Layout
- [ ] 2.2 移除硬编码值
- [ ] 2.3 添加最小尺寸保护
- [ ] 2.4 验证各组件不重叠

### 阶段 3: 视觉回归测试

- [ ] 3.1 创建 visual_regression.rs 测试文件
- [ ] 3.2 实现 BufferDiff 比较函数
- [ ] 3.3 添加 InputWidget 布局测试
- [ ] 3.4 添加聊天区域布局测试
- [ ] 3.5 运行测试验证

## 验收标准

1. **InputWidget**
   - [ ] 80x24 终端上长文本正确显示
   - [ ] 40x12 窄终端上不崩溃
   - [ ] 光标位置正确

2. **draw_chat**
   - [ ] 无硬编码值
   - [ ] 各组件不重叠
   - [ ] 最小终端尺寸 (40x10) 可用

3. **测试**
   - [ ] 视觉回归测试通过
   - [ ] 相似度 >= 95%
   - [ ] 布局正确性断言通过

## 技术细节

### 相关文件

- `rust-opencode-port/crates/tui/src/components/input_widget.rs`
- `rust-opencode-port/crates/tui/src/app.rs`
- `ratatui-testing/src/diff.rs`
- `ratatui-testing/tests/visual_regression.rs` (新建)

### 依赖

- `ratatui` - TUI 库
- `ratatui-testing` - 测试框架
- 无需新增外部依赖
