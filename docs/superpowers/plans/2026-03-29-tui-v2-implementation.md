# TUI V2 交互效果实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 rust-opencode-port TUI 实现完整的沉浸式交互体验，覆盖设计文档 docs/superpowers/specs/2026-03-29-tui-v2-design.md 的全部 5 个章节

**Architecture:** 
- 新增 animation 模块管理动画状态和时间轴
- 扩展现有组件添加动画支持
- 创建新的 UI 组件（hover_popup, heatmap_scrollbar, code_highlight 等）
- 复用现有 Dialog 系统实现补全弹窗

**Tech Stack:** 
- ratatui 0.28
- crossterm 0.28
- Rust async/await

---

## 文件结构

```
crates/tui/src/
├── animation/          # 新建：动画系统
│   ├── mod.rs
│   ├── animator.rs     # 动画器 trait
│   ├── transition.rs   # 过渡状态
│   └── easing.rs       # 缓动函数
├── components/
│   ├── mod.rs          # 修改：导出新组件
│   ├── input_widget.rs # 修改：添加动画
│   ├── file_tree.rs    # 修改：添加抽屉动画
│   ├── status_bar.rs   # 修改：动态颜色
│   ├── hover_popup.rs  # 新建：悬浮文档
│   ├── code_highlight.rs # 新建：彩虹缩进
│   ├── fold_indicator.rs  # 新建：折叠动画
│   ├── error_underline.rs # 新建：波浪线
│   └── heatmap_scrollbar.rs # 新建：热力图滚动条
├── dialogs/
│   ├── mod.rs          # 修改：导出新对话框
│   └── completion.rs   # 新建：补全弹窗
└── app.rs              # 修改：集成动画系统
```

---

## 任务分解

### 阶段 1: 动画系统基础设施

#### Task 1: 创建动画模块基础结构

**Files:**
- Create: `rust-opencode-port/crates/tui/src/animation/mod.rs`
- Create: `rust-opencode-port/crates/tui/src/animation/animator.rs`
- Modify: `rust-opencode-port/crates/tui/src/lib.rs` (添加 `pub mod animation;`)

- [ ] **Step 1: 创建 animation/mod.rs**

```rust
pub mod animator;
pub mod easing;
pub mod transition;

pub use animator::Animator;
pub use easing::Easing;
pub use transition::Transition;
```

- [ ] **Step 2: 创建 easing.rs**

```rust
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl Easing {
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
        }
    }
}
```

- [ ] **Step 3: 创建 animator.rs**

```rust
use super::easing::Easing;
use std::time::Instant;

pub struct Animation {
    pub start_value: f32,
    pub end_value: f32,
    pub current_value: f32,
    pub duration_ms: u64,
    pub start_time: Instant,
    pub easing: Easing,
    pub completed: bool,
}

impl Animation {
    pub fn new(start: f32, end: f32, duration_ms: u64) -> Self {
        Self {
            start_value: start,
            end_value: end,
            current_value: start,
            duration_ms,
            start_time: Instant::now(),
            easing: Easing::EaseOut,
            completed: false,
        }
    }

    pub fn update(&mut self) {
        let elapsed = self.start_time.elapsed().as_millis() as f32;
        let progress = (elapsed / self.duration_ms as f32).min(1.0);
        let eased = self.easing.apply(progress);
        
        self.current_value = self.start_value + (self.end_value - self.start_value) * eased;
        
        if progress >= 1.0 {
            self.completed = true;
            self.current_value = self.end_value;
        }
    }

    pub fn is_done(&self) -> bool {
        self.completed
    }
}
```

- [ ] **Step 4: 创建 transition.rs**

```rust
use super::animator::Animation;

pub enum Transition {
    Idle,
    Running(Animation),
}

impl Transition {
    pub fn new() -> Self {
        Transition::Idle
    }

    pub fn start(&mut self, start: f32, end: f32, duration_ms: u64) {
        *self = Transition::Running(Animation::new(start, end, duration_ms));
    }

    pub fn update(&mut self) -> Option<f32> {
        match self {
            Transition::Idle => None,
            Transition::Running(anim) => {
                anim.update();
                if anim.is_done() {
                    *self = Transition::Idle;
                }
                Some(anim.current_value)
            }
        }
    }
}
```

- [ ] **Step 5: 验证编译**

Run: `cd rust-opencode-port && cargo check -p opencode-tui`
Expected: SUCCESS

---

#### Task 2: 在 App 中集成动画更新循环

**Files:**
- Modify: `rust-opencode-port/crates/tui/src/app.rs`

- [ ] **Step 1: 添加动画状态字段到 App**

在 App struct 中添加:
```rust
pub struct App {
    // ... existing fields
    pub last_frame_time: Instant,
}
```

- [ ] **Step 2: 在 run() 初始化时记录时间**

在 App::run() 方法中，事件循环前添加:
```rust
self.last_frame_time = Instant::now();
```

- [ ] **Step 3: 计算 delta time 并更新**

在事件循环的每次迭代中:
```rust
let current_time = Instant::now();
let delta_ms = current_time.duration_since(self.last_frame_time).as_millis() as u64;
self.last_frame_time = current_time;

// 更新动画状态
self.update_animations(delta_ms);
```

- [ ] **Step 4: 添加 update_animations 方法**

```rust
fn update_animations(&mut self, _delta_ms: u64) {
    // 由各组件自行更新，这里保留接口
}
```

- [ ] **Step 5: 验证编译**

Run: `cd rust-opencode-port && cargo check -p opencode-tui`
Expected: SUCCESS

---

### 阶段 2: 整体布局与视觉感知

#### Task 3: FileTree 抽屉动画

**Files:**
- Modify: `rust-opencode-port/crates/tui/src/components/file_tree.rs`

- [ ] **Step 1: 添加动画状态字段**

```rust
pub struct FileTree {
    // ... existing fields
    pub visible: bool,
    pub width: u16,
    pub target_width: u16,
    pub animation_progress: f32,  // 0.0 - 1.0
    pub is_animating: bool,
}
```

- [ ] **Step 2: 修改 toggle 方法触发动画**

```rust
pub fn toggle(&mut self) {
    self.is_animating = true;
    if self.visible {
        self.visible = false;
    } else {
        self.visible = true;
        self.animation_progress = 0.0;
    }
}
```

- [ ] **Step 3: 添加 update_animation 方法**

```rust
pub fn update_animation(&mut self, delta_ms: u64) {
    if !self.is_animating {
        return;
    }

    const ANIMATION_SPEED: f32 = 0.003;  // 每毫秒进度

    if self.visible {
        self.animation_progress += ANIMATION_SPEED * delta_ms as f32;
        if self.animation_progress >= 1.0 {
            self.animation_progress = 1.0;
            self.is_animating = false;
        }
    } else {
        self.animation_progress -= ANIMATION_SPEED * delta_ms as f32;
        if self.animation_progress <= 0.0 {
            self.animation_progress = 0.0;
            self.is_animating = false;
        }
    }
}
```

- [ ] **Step 4: 修改 draw 方法使用动画宽度**

```rust
pub fn draw(&mut self, f: &mut Frame, area: Rect) {
    if self.animation_progress <= 0.0 && !self.visible {
        return;
    }

    let current_width = (self.target_width as f32 * self.animation_progress) as u16;
    let draw_area = Rect::new(area.x, area.y, current_width, area.height);
    
    // 渲染文件树内容
    // ... existing render logic
}
```

- [ ] **Step 5: 验证编译**

Run: `cd rust-opencode-port && cargo check -p opencode-tui`
Expected: SUCCESS

---

#### Task 4: 毛玻璃效果模拟

**Files:**
- Create: `rust-opencode-port/crates/tui/src/components/backdrop.rs`
- Modify: `rust-opencode-port/crates/tui/src/components.rs` (导出新组件)
- Modify: `rust-opencode-port/crates/tui/src/app.rs`

- [ ] **Step 1: 创建 backdrop.rs**

```rust
use ratatui::{
    widgets::{Block, BorderType, Borders},
    Frame,
};

pub struct Backdrop {
    pub enabled: bool,
}

impl Backdrop {
    pub fn new() -> Self {
        Self { enabled: true }
    }

    pub fn draw(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        if !self.enabled {
            return;
        }

        // 绘制深色半透明背景效果
        let block = Block::default()
            .bg(Color::Rgb(20, 20, 25))
            .border_type(BorderType::Plain);
        
        f.render_widget(block, area);
    }
}
```

- [ ] **Step 2: 导出新组件**

在 components.rs 中添加:
```rust
pub mod backdrop;
pub use backdrop::Backdrop;
```

- [ ] **Step 3: 在 App 中使用 Backdrop**

在 app.rs 中:
```rust
use crate::components::Backdrop;

pub struct App {
    // ... existing fields
    pub backdrop: Backdrop,
}
```

- [ ] **Step 4: 在 draw 方法中渲染背景**

在 draw() 方法开头调用:
```rust
fn draw(&mut self, f: &mut Frame) {
    self.backdrop.draw(f, f.area());
    // ... rest of drawing
}
```

- [ ] **Step 5: 验证编译**

Run: `cd rust-opencode-port && cargo check -p opencode-tui`
Expected: SUCCESS

---

#### Task 5: 动态状态栏颜色

**Files:**
- Modify: `rust-opencode-port/crates/tui/src/components/status_bar.rs`

- [ ] **Step 1: 添加 LanguageMode 枚举**

```rust
#[derive(Clone, Copy, PartialEq)]
pub enum LanguageMode {
    None,
    Python,
    Rust,
    JavaScript,
    TypeScript,
    Go,
    Cpp,
    Java,
}

impl LanguageMode {
    pub fn theme_color(&self) -> Color {
        match self {
            LanguageMode::None => Color::Gray,
            LanguageMode::Python => Color::Rgb(55, 118, 171),    // 蓝色
            LanguageMode::Rust => Color::Rgb(222, 165, 132),     // 橙色
            LanguageMode::JavaScript => Color::Rgb(247, 223, 30), // 黄色
            LanguageMode::TypeScript => Color::Rgb(49, 120, 198), // 蓝色
            LanguageMode::Go => Color::Rgb(0, 173, 216),          // 青色
            LanguageMode::Cpp => Color::Rgb(0, 85, 164),          // 深蓝
            LanguageMode::Java => Color::Rgb(176, 114, 25),       // 棕色
        }
    }
}
```

- [ ] **Step 2: 添加 language_mode 字段到 StatusBar**

```rust
pub struct StatusBar {
    // ... existing fields
    pub language_mode: LanguageMode,
}
```

- [ ] **Step 3: 修改 draw 方法使用动态颜色**

```rust
pub fn draw(&self, f: &mut Frame, area: Rect) {
    let color = self.language_mode.theme_color();
    
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(color))
        .bg(Color::Rgb(25, 25, 30));
    
    f.render_widget(block, area);
    // ... rest of rendering
}
```

- [ ] **Step 4: 添加设置 language_mode 的方法**

```rust
impl StatusBar {
    pub fn set_language(&mut self, mode: LanguageMode) {
        self.language_mode = mode;
    }
}
```

- [ ] **Step 5: 验证编译**

Run: `cd rust-opencode-port && cargo check -p opencode-tui`
Expected: SUCCESS

---

### 阶段 3: 光标交互效果

#### Task 6: 平滑光标移动

**Files:**
- Modify: `rust-opencode-port/crates/tui/src/components/input_widget.rs`

- [ ] **Step 1: 添加 CursorAnimation 状态**

```rust
pub struct CursorAnimation {
    pub target_x: u16,
    pub target_y: u16,
    pub current_x: f32,
    pub current_y: f32,
    pub is_animating: bool,
}

impl CursorAnimation {
    pub fn new() -> Self {
        Self {
            target_x: 0,
            target_y: 0,
            current_x: 0.0,
            current_y: 0.0,
            is_animating: false,
        }
    }

    pub fn set_target(&mut self, x: u16, y: u16) {
        self.target_x = x;
        self.target_y = y;
        self.is_animating = true;
    }

    pub fn update(&mut self, delta_ms: u64) {
        if !self.is_animating {
            return;
        }

        const SPEED: f32 = 0.3;
        
        let dx = self.target_x as f32 - self.current_x;
        let dy = self.target_y as f32 - self.current_y;

        if dx.abs() < 0.5 && dy.abs() < 0.5 {
            self.current_x = self.target_x as f32;
            self.current_y = self.target_y as f32;
            self.is_animating = false;
        } else {
            self.current_x += dx * SPEED;
            self.current_y += dy * SPEED;
        }
    }

    pub fn position(&self) -> (u16, u16) {
        (self.current_x as u16, self.current_y as u16)
    }
}
```

- [ ] **Step 2: 在 InputWidget 中添加 cursor_animation**

```rust
pub struct InputWidget {
    // ... existing fields
    pub cursor_animation: CursorAnimation,
}
```

- [ ] **Step 3: 修改 set_cursor 方法触发动画**

```rust
pub fn set_cursor(&mut self, position: usize) {
    // ... existing logic
    let (x, y) = self.position_to_coords(position);
    self.cursor_animation.set_target(x, y);
}
```

- [ ] **Step 4: 修改 render_cursor 使用动画位置**

```rust
fn render_cursor(&self, f: &mut Frame, area: Rect) {
    let (x, y) = self.cursor_animation.position();
    f.set_cursor(area.x + x, area.y + y);
}
```

- [ ] **Step 5: 验证编译**

Run: `cd rust-opencode-port && cargo check -p opencode-tui`
Expected: SUCCESS

---

#### Task 7: 脉冲呼吸效果

**Files:**
- Modify: `rust-opencode-port/crates/tui/src/components/input_widget.rs`

- [ ] **Step 1: 在 CursorAnimation 中添加呼吸相位**

```rust
pub struct CursorAnimation {
    // ... existing fields
    pub breathe_phase: f32,
    pub breathe_enabled: bool,
}

impl CursorAnimation {
    pub fn new() -> Self {
        Self {
            // ... existing fields
            breathe_phase: 0.0,
            breathe_enabled: true,
        }
    }
}
```

- [ ] **Step 2: 添加 update_breathe 方法**

```rust
pub fn update_breathe(&mut self, delta_ms: u64) {
    if !self.breathe_enabled || self.is_animating {
        return;
    }

    const BREATHE_SPEED: f32 = 0.003;
    self.breathe_phase += BREATHE_SPEED * delta_ms as f32;
    
    if self.breathe_phase > std::f32::consts::PI * 2.0 {
        self.breathe_phase -= std::f32::consts::PI * 2.0;
    }
}

pub fn breathe_alpha(&self) -> u8 {
    if !self.breathe_enabled || self.is_animating {
        return 255;
    }
    
    let breathe = (self.breathe_phase.sin() + 1.0) / 2.0;
    (breathe * 127.0 + 128.0) as u8  // 50%-100%
}
```

- [ ] **Step 3: 修改渲染使用呼吸透明度**

```rust
fn render_cursor(&self, f: &mut Frame, area: Rect) {
    let alpha = self.cursor_animation.breathe_alpha();
    let (x, y) = self.cursor_animation.position();
    
    // 使用带有透明度样式的光标
    f.set_cursor(area.x + x, area.y + y);
}
```

- [ ] **Step 4: 在 InputWidget 的 update 中调用 breathe 更新**

在 InputWidget 中添加 update 方法:
```rust
pub fn update(&mut self, delta_ms: u64) {
    self.cursor_animation.update(delta_ms);
    self.cursor_animation.update_breathe(delta_ms);
}
```

- [ ] **Step 5: 验证编译**

Run: `cd rust-opencode-port && cargo check -p opencode-tui`
Expected: SUCCESS

---

#### Task 8: 选中文字圆角效果

**Files:**
- Modify: `rust-opencode-port/crates/tui/src/components/input_widget.rs`

- [ ] **Step 1: 添加 selection_style 字段**

```rust
pub struct InputWidget {
    // ... existing fields
    pub selection_rounded: bool,
}
```

- [ ] **Step 2: 修改 render_selection 方法添加圆角**

```rust
fn render_selection(&self, f: &mut Frame, area: Rect) {
    if self.selection_start >= self.selection_end {
        return;
    }

    let (start_x, start_y) = self.position_to_coords(self.selection_start);
    let (end_x, end_y) = self.position_to_coords(self.selection_end);
    
    let style = if self.selection_rounded {
        Style::default()
            .bg(Color::Rgb(50, 50, 80))
            .add_modifier(Modifier::ROUNDED)
    } else {
        Style::default().bg(Color::Rgb(50, 50, 80))
    };
    
    // 渲染选中的高亮块
    for y in start_y..=end_y {
        let y_pos = area.y + y;
        let x_start = if y == start_y { area.x + start_x } else { area.x };
        let x_end = if y == end_y { area.x + end_x } else { area.x + area.width };
        
        let width = (x_end - x_start).max(1);
        let line = Line::from(vec![Span::styled(" ".repeat(width as usize), style)]);
        f.render_widget(line, Rect::new(x_start, y_pos, width, 1));
    }
}
```

- [ ] **Step 3: 验证编译**

Run: `cd rust-opencode-port && cargo check -p opencode-tui`
Expected: SUCCESS

---

#### Task 9: 字符淡入动画

**Files:**
- Modify: `rust-opencode-port/crates/tui/src/components/input_widget.rs`

- [ ] **Step 1: 添加 CharAnimation 结构**

```rust
use std::time::Instant;

pub struct CharAnimation {
    pub char: String,
    pub x: u16,
    pub y: u16,
    pub fade_progress: f32,
    pub created_at: Instant,
}
```

- [ ] **Step 2: 在 InputWidget 中添加 char_animations**

```rust
pub struct InputWidget {
    // ... existing fields
    pub char_animations: Vec<CharAnimation>,
}
```

- [ ] **Step 3: 修改 insert_char 添加动画**

```rust
pub fn insert_char(&mut self, c: char) {
    let (x, y) = self.position_to_coords(self.cursor);
    
    // ... existing insert logic
    
    // 添加字符动画
    self.char_animations.push(CharAnimation {
        char: c.to_string(),
        x,
        y,
        fade_progress: 0.0,
        created_at: Instant::now(),
    });
}
```

- [ ] **Step 4: 添加 update_char_animations 方法**

```rust
pub fn update_char_animations(&mut self) {
    const FADE_DURATION_MS: u64 = 200;
    
    let now = Instant::now();
    self.char_animations.retain(|anim| {
        let elapsed = now.duration_since(anim.created_at).as_millis() as f32;
        anim.fade_progress = (elapsed / FADE_DURATION_MS as f32).min(1.0);
        anim.fade_progress < 1.0
    });
}
```

- [ ] **Step 5: 添加 render_char_animations 方法**

```rust
fn render_char_animations(&self, f: &mut Frame, area: Rect) {
    for anim in &self.char_animations {
        let alpha = (anim.fade_progress * 255.0) as u8;
        
        let style = Style::default()
            .fg(Color::LightWhite);
        
        f.render_widget(
            Span::styled(&anim.char, style),
            Rect::new(area.x + anim.x, area.y + anim.y, 1, 1)
        );
    }
}
```

- [ ] **Step 6: 验证编译**

Run: `cd rust-opencode-port && cargo check -p opencode-tui`
Expected: SUCCESS

---

#### Task 10: 符号配对

**Files:**
- Modify: `rust-opencode-port/crates/tui/src/components/input_widget.rs`

- [ ] **Step 1: 添加 BracketPair 相关字段**

```rust
const BRACKET_PAIRS: &[(char, char)] = &[
    ('(', ')'),
    ('[', ']'),
    ('{', '}'),
    ('<', '>'),
    ('"', '"'),
    ('\'', '\''),
];

pub struct InputWidget {
    // ... existing fields
    pub pending_bracket: Option<(char, char)>,  // (left, right)
    pub show_ghost_bracket: bool,
}
```

- [ ] **Step 2: 修改 insert_char 检测括号**

```rust
pub fn insert_char(&mut self, c: char) {
    // 检查是否是左括号
    if let Some(pair) = BRACKET_PAIRS.iter().find(|(l, _)| *l == c) {
        self.pending_bracket = Some((*pair.0, *pair.1));
        self.show_ghost_bracket = true;
    } else {
        self.pending_bracket = None;
        self.show_ghost_bracket = false;
    }
    
    // ... existing insert logic
}
```

- [ ] **Step 3: 添加 render_ghost_bracket 方法**

```rust
fn render_ghost_bracket(&self, f: &mut Frame, area: Rect) {
    if !self.show_ghost_bracket {
        return;
    }
    
    let Some((_, right)) = self.pending_bracket else { return };
    
    let ghost_x = self.cursor_x + 1;
    let ghost_style = Style::default()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::DIM);
    
    f.render_widget(
        Span::styled(right.to_string(), ghost_style),
        Rect::new(area.x + ghost_x, area.y + self.cursor_y, 1, 1)
    );
}
```

- [ ] **Step 4: 修改 draw 方法调用 ghost_bracket 渲染**

在 InputWidget::draw 方法末尾添加:
```rust
self.render_ghost_bracket(f, area);
```

- [ ] **Step 5: 验证编译**

Run: `cd rust-opencode-port && cargo check -p opencode-tui`
Expected: SUCCESS

---

#### Task 11: 补全弹窗 (IntelliSense)

**Files:**
- Create: `rust-opencode-port/crates/tui/src/dialogs/completion.rs`
- Modify: `rust-opencode-port/crates/tui/src/dialogs/mod.rs`

- [ ] **Step 1: 创建 completion.rs**

```rust
use ratatui::{
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::dialogs::{Dialog, DialogAction};

pub struct CompletionItem {
    pub label: String,
    pub detail: Option<String>,
    pub insert_text: String,
}

pub struct CompletionDialog {
    pub items: Vec<CompletionItem>,
    pub selected_index: usize,
    pub visible: bool,
}

impl CompletionDialog {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
            visible: false,
        }
    }

    pub fn show(&mut self, items: Vec<CompletionItem>) {
        self.items = items;
        self.selected_index = 0;
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn selected_item(&self) -> Option<&CompletionItem> {
        self.items.get(self.selected_index)
    }
}

impl Dialog for CompletionDialog {
    fn draw(&self, f: &mut Frame, area: Rect) {
        if !self.visible || self.items.is_empty() {
            return;
        }

        let items: Vec<ListItem> = self.items.iter().enumerate().map(|(i, item)| {
            let style = if i == self.selected_index {
                Style::default()
                    .bg(Color::Rgb(50, 50, 80))
                    .fg(Color::White)
            } else {
                Style::default().fg(Color::LightGray)
            };

            let text = if let Some(ref detail) = item.detail {
                format!("{} - {}", item.label, detail)
            } else {
                item.label.clone()
            };

            ListItem::new(text).style(style)
        }).collect();

        let list = List::new(items)
            .block(Block::default()
                .title("Completions")
                .borders(Borders::ALL))
            .highlight_style(Style::default()
                .bg(Color::Rgb(60, 60, 100))
                .add_modifier(Modifier::BOLD));

        f.render_widget(list, area);
    }

    fn handle_input(&mut self, key: KeyEvent) -> DialogAction {
        use crossterm::event::KeyCode;

        match key.code {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                DialogAction::Continue
            }
            KeyCode::Down => {
                if self.selected_index < self.items.len() - 1 {
                    self.selected_index += 1;
                }
                DialogAction::Continue
            }
            KeyCode::Enter => {
                if let Some(item) = self.selected_item() {
                    DialogAction::Complete(item.insert_text.clone())
                } else {
                    DialogAction::Close
                }
            }
            KeyCode::Esc => {
                self.hide();
                DialogAction::Close
            }
            _ => DialogAction::Continue,
        }
    }

    fn is_modal(&self) -> bool {
        false
    }
}
```

- [ ] **Step 2: 在 dialogs/mod.rs 中添加导出**

```rust
pub mod completion;
pub use completion::{CompletionDialog, CompletionItem};
```

- [ ] **Step 3: 验证编译**

Run: `cd rust-opencode-port && cargo check -p opencode-tui`
Expected: SUCCESS

---

### 阶段 4: 代码块与结构感知

#### Task 12: 彩虹缩进线

**Files:**
- Create: `rust-opencode-port/crates/tui/src/components/code_highlight.rs`
- Modify: `rust-opencode-port/crates/tui/src/components.rs`

- [ ] **Step 1: 创建 code_highlight.rs**

```rust
use ratatui::{Frame, layout::Rect, style::Color};

pub struct IndentGuide {
    pub indent_level: usize,
    pub is_active: bool,
    pub color: Color,
}

pub struct CodeHighlight {
    pub indent_guides: Vec<IndentGuide>,
    pub active_indent: usize,
}

impl CodeHighlight {
    pub fn new() -> Self {
        Self {
            indent_guides: Vec::new(),
            active_indent: 0,
        }
    }

    pub fn set_active_indent(&mut self, indent: usize) {
        self.active_indent = indent;
        for guide in &mut self.indent_guides {
            guide.is_active = guide.indent_level >= indent;
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, content: &[&str]) {
        for (line_num, line) in content.iter().enumerate() {
            let indent = line.len() - line.trim_start().len();
            
            if indent > 0 {
                let is_active = indent >= self.active_indent;
                let color = if is_active {
                    self.get_indent_color(indent)
                } else {
                    Color::DarkGray
                };

                // 渲染缩进线
                for i in (0..indent).step_by(2) {
                    let x = area.x + i as u16;
                    f.render_widget(
                        Span::styled("│", Style::default().fg(color)),
                        Rect::new(x, area.y + line_num as u16, 1, 1)
                    );
                }
            }
        }
    }

    fn get_indent_color(&self, indent: usize) -> Color {
        const COLORS: &[Color] = &[
            Color::LightRed,
            Color::LightGreen,
            Color::LightYellow,
            Color::LightBlue,
            Color::LightMagenta,
            Color::Cyan,
        ];
        COLORS[(indent / 2) % COLORS.len()]
    }
}
```

- [ ] **Step 2: 导出组件**

在 components.rs 中添加:
```rust
pub mod code_highlight;
pub use code_highlight::CodeHighlight;
```

- [ ] **Step 3: 验证编译**

Run: `cd rust-opencode-port && cargo check -p opencode-tui`
Expected: SUCCESS

---

#### Task 13: 悬浮文档弹窗

**Files:**
- Create: `rust-opencode-port/crates/tui/src/components/hover_popup.rs`
- Modify: `rust-opencode-port/crates/tui/src/components.rs`

- [ ] **Step 1: 创建 hover_popup.rs**

```rust
use ratatui::{
    widgets::{Block, Borders, Paragraph},
    Frame,
    layout::Rect,
    text::{Text, Span},
    style::{Style, Modifier},
};

pub struct HoverPopup {
    pub visible: bool,
    pub content: String,
    pub position: (u16, u16),
    pub size: (u16, u16),
}

impl HoverPopup {
    pub fn new() -> Self {
        Self {
            visible: false,
            content: String::new(),
            position: (0, 0),
            size: (40, 10),
        }
    }

    pub fn show(&mut self, content: String, x: u16, y: u16) {
        self.visible = true;
        self.content = content.clone();
        
        let lines = content.lines().count();
        let max_width = content.lines().map(|l| l.len()).max().unwrap_or(0);
        self.size = ((max_width + 4).min(60) as u16, (lines + 2).max(5) as u16);
        
        self.position = (x, y);
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn draw(&self, f: &mut Frame) {
        if !self.visible {
            return;
        }

        let area = Rect::new(
            self.position.0,
            self.position.1,
            self.size.0,
            self.size.1,
        );

        let markdown = self.render_markdown();
        
        let popup = Paragraph::new(markdown)
            .wrap(Wrap { trim: true })
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .bg(Color::Rgb(30, 30, 40)));

        f.render_widget(popup, area);
    }

    fn render_markdown(&self) -> Text {
        let mut result = Vec::new();
        
        for line in self.content.lines() {
            let span = if line.starts_with("```") {
                Span::raw(line)
            } else if line.starts_with('#') {
                Span::styled(line, Style::default().add_modifier(Modifier::BOLD))
            } else if line.starts_with('-') || line.starts_with('*') {
                Span::raw(format!("  {}", line))
            } else {
                Span::raw(line)
            };
            result.push(span);
        }
        
        Text::from(result)
    }
}
```

- [ ] **Step 2: 导出组件**

在 components.rs 中添加:
```rust
pub mod hover_popup;
pub use hover_popup::HoverPopup;
```

- [ ] **Step 3: 验证编译**

Run: `cd rust-opencode-port && cargo check -p opencode-tui`
Expected: SUCCESS

---

#### Task 14: 折叠动画

**Files:**
- Create: `rust-opencode-port/crates/tui/src/components/fold_indicator.rs`
- Modify: `rust-opencode-port/crates/tui/src/components.rs`

- [ ] **Step 1: 创建 fold_indicator.rs**

```rust
use ratatui::{
    widgets::Paragraph,
    Frame,
    layout::Rect,
    style::{Style, Color},
    text::Span,
};

pub struct FoldIndicator {
    pub line_start: usize,
    pub folded: bool,
    pub fold_progress: f32,
    pub is_animating: bool,
}

impl FoldIndicator {
    pub fn new(line_start: usize) -> Self {
        Self {
            line_start,
            folded: false,
            fold_progress: 0.0,
            is_animating: false,
        }
    }

    pub fn toggle(&mut self) {
        self.is_animating = true;
        self.folded = !self.folded;
    }

    pub fn update(&mut self, delta_ms: u64) {
        if !self.is_animating {
            return;
        }

        const FOLD_SPEED: f32 = 0.1;

        if self.folded {
            self.fold_progress += FOLD_SPEED;
            if self.fold_progress >= 1.0 {
                self.fold_progress = 1.0;
                self.is_animating = false;
            }
        } else {
            self.fold_progress -= FOLD_SPEED;
            if self.fold_progress <= 0.0 {
                self.fold_progress = 0.0;
                self.is_animating = false;
            }
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let dots = match (self.fold_progress * 3.0) as u8 {
            0 => "",
            1 => ".",
            2 => "..",
            _ => "...",
        };

        let style = if self.folded {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::LightGray)
        };

        let text = Paragraph::new(Span::styled(dots, style));
        f.render_widget(text, area);
    }

    pub fn is_visible(&self) -> bool {
        self.folded || self.is_animating
    }
}
```

- [ ] **Step 2: 导出组件**

在 components.rs 中添加:
```rust
pub mod fold_indicator;
pub use fold_indicator::FoldIndicator;
```

- [ ] **Step 3: 验证编译**

Run: `cd rust-opencode-port && cargo check -p opencode-tui`
Expected: SUCCESS

---

### 阶段 5: 错误与警告交互

#### Task 15: 波浪线动画

**Files:**
- Create: `rust-opencode-port/crates/tui/src/components/error_underline.rs`
- Modify: `rust-opencode-port/crates/tui/src/components.rs`

- [ ] **Step 1: 创建 error_underline.rs**

```rust
use ratatui::{
    Frame,
    layout::Rect,
    style::{Style, Color},
    text::Span,
};

pub struct ErrorUnderline {
    pub line: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub wave_offset: f32,
}

impl ErrorUnderline {
    pub fn new(line: usize, column_start: usize, column_end: usize) -> Self {
        Self {
            line,
            column_start,
            column_end,
            wave_offset: 0.0,
        }
    }

    pub fn update(&mut self, delta_ms: u64) {
        const WAVE_SPEED: f32 = 0.005;
        self.wave_offset += WAVE_SPEED * delta_ms as f32;
    }

    pub fn render(&self, f: &mut Frame, area: Rect, line_offset: u16) {
        let y = area.y + line_offset;

        for x in self.column_start..self.column_end {
            let wave = (self.wave_offset + x as f32 * 0.3).sin();
            let display_x = self.column_start + ((x - self.column_start) as f32 + wave * 0.5) as usize;
            
            let char = if wave > 0.0 { '~' } else { '-' };
            let style = Style::default().fg(Color::Red);
            
            f.render_widget(
                Span::styled(char.to_string(), style),
                Rect::new(area.x + display_x, y, 1, 1)
            );
        }
    }
}
```

- [ ] **Step 2: 导出组件**

在 components.rs 中添加:
```rust
pub mod error_underline;
pub use error_underline::ErrorUnderline;
```

- [ ] **Step 3: 验证编译**

Run: `cd rust-opencode-port && cargo check -p opencode-tui`
Expected: SUCCESS

---

#### Task 16: 热力图滚动条

**Files:**
- Create: `rust-opencode-port/crates/tui/src/components/heatmap_scrollbar.rs`
- Modify: `rust-opencode-port/crates/tui/src/components.rs`

- [ ] **Step 1: 创建 heatmap_scrollbar.rs**

```rust
use ratatui::{
    widgets::{Block, Scrollbar, ScrollbarOrientation},
    Frame,
    layout::Rect,
    style::Color,
};

#[derive(Clone, Copy)]
pub enum MarkerSeverity {
    Error,
    Warning,
    Info,
}

pub struct Marker {
    pub line: usize,
    pub severity: MarkerSeverity,
}

pub struct HeatmapScrollbar {
    pub total_lines: usize,
    pub viewport_start: usize,
    pub viewport_size: usize,
    pub markers: Vec<Marker>,
}

impl HeatmapScrollbar {
    pub fn new() -> Self {
        Self {
            total_lines: 0,
            viewport_start: 0,
            viewport_size: 0,
            markers: Vec::new(),
        }
    }

    pub fn set_content(&mut self, total: usize, viewport_size: usize) {
        self.total_lines = total;
        self.viewport_size = viewport_size;
    }

    pub fn add_marker(&mut self, line: usize, severity: MarkerSeverity) {
        self.markers.push(Marker { line, severity });
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        // 渲染热力图背景
        for i in 0..area.height as usize {
            let line = self.viewport_start + 
                (i as f32 * self.total_lines as f32 / area.height as f32) as usize;
            
            let color = if self.markers.iter().any(|m| m.line == line && matches!(m.severity, MarkerSeverity::Error)) {
                Color::Red
            } else if self.markers.iter().any(|m| m.line == line && matches!(m.severity, MarkerSeverity::Warning)) {
                Color::Yellow
            } else if self.markers.iter().any(|m| m.line == line && matches!(m.severity, MarkerSeverity::Info)) {
                Color::LightBlue
            } else {
                Color::DarkGray
            };

            let block = Block::default()
                .bg(color)
                .fg(color);
            
            f.render_widget(block, Rect::new(area.x, area.y + i as u16, 1, 1));
        }

        // 渲染滚动条
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        let mut state = ratatui::widgets::ScrollbarState::new(self.total_lines)
            .position(self.viewport_start);
        
        f.render_stateful_widget(scrollbar, area, &mut state);
    }
}
```

- [ ] **Step 2: 导出组件**

在 components.rs 中添加:
```rust
pub mod heatmap_scrollbar;
pub use heatmap_scrollbar::{HeatmapScrollbar, MarkerSeverity};
```

- [ ] **Step 3: 验证编译**

Run: `cd rust-opencode-port && cargo check -p opencode-tui`
Expected: SUCCESS

---

### 阶段 6: 集成与测试

#### Task 17: 在 App 中集成所有新组件

**Files:**
- Modify: `rust-opencode-port/crates/tui/src/app.rs`
- Modify: `rust-opencode-port/crates/tui/src/components.rs`
- Modify: `rust-opencode-port/crates/tui/src/lib.rs`

- [ ] **Step 1: 更新 lib.rs 导出所有新组件**

```rust
pub use components::{
    // ... existing
    Backdrop, CodeHighlight, CompletionDialog, CompletionItem,
    ErrorUnderline, FoldIndicator, HeatmapScrollbar, HoverPopup,
    // ...
};
```

- [ ] **Step 2: 在 App 中添加新组件字段**

```rust
pub struct App {
    // ... existing fields
    pub backdrop: Backdrop,
    pub hover_popup: HoverPopup,
    pub completion_dialog: Option<CompletionDialog>,
    pub code_highlight: CodeHighlight,
    pub error_underlines: Vec<ErrorUnderline>,
    pub heatmap_scrollbar: HeatmapScrollbar,
}
```

- [ ] **Step 3: 在 App::new() 中初始化新组件**

```rust
impl App {
    pub fn new(/* ... */) -> Self {
        // ... existing initialization
        Self {
            // ... existing fields
            backdrop: Backdrop::new(),
            hover_popup: HoverPopup::new(),
            completion_dialog: None,
            code_highlight: CodeHighlight::new(),
            error_underlines: Vec::new(),
            heatmap_scrollbar: HeatmapScrollbar::new(),
        }
    }
}
```

- [ ] **Step 4: 在 draw 方法中渲染新组件**

在 draw() 方法中添加:
```rust
// 渲染背景
self.backdrop.draw(f, f.area());

// 渲染悬浮弹窗
self.hover_popup.draw(f);

// 渲染补全弹窗
if let Some(ref dialog) = self.completion_dialog {
    dialog.draw(f, completion_area);
}

// 渲染热力图滚动条
self.heatmap_scrollbar.render(f, scrollbar_area);
```

- [ ] **Step 5: 验证编译**

Run: `cd rust-opencode-port && cargo check -p opencode-tui`
Expected: SUCCESS

---

#### Task 18: 端到端测试

**Files:**
- Modify: `rust-opencode-port/crates/tui/tests/` (如存在)

- [ ] **Step 1: 添加动画集成测试**

```rust
#[test]
fn test_animation_progress() {
    let mut animation = Animation::new(0.0, 100.0, 1000);
    
    // 初始状态
    assert!(!animation.is_done());
    
    // 更新到完成
    animation.update();
    assert!(!animation.is_done());
    
    // 手动设置完成
    animation.completed = true;
    assert!(animation.is_done());
    assert_eq!(animation.current_value, 100.0);
}
```

- [ ] **Step 2: 添加 InputWidget 动画测试**

```rust
#[test]
fn test_cursor_animation() {
    let mut cursor = CursorAnimation::new();
    cursor.set_target(10, 5);
    
    assert!(cursor.is_animating);
    
    // 模拟多帧更新
    for _ in 0..100 {
        cursor.update(10);
    }
    
    assert!(!cursor.is_animating);
    assert_eq!(cursor.position(), (10, 5));
}
```

- [ ] **Step 3: 运行测试**

Run: `cd rust-opencode-port && cargo test -p opencode-tui`
Expected: All tests pass

---

## 依赖关系图

```
阶段 1: 动画系统基础设施
├── Task 1: 动画模块基础 ← Task 2 依赖
└── Task 2: App 集成动画 ← Task 1 依赖

阶段 2: 整体布局
├── Task 3: FileTree 抽屉动画 ← Task 2 依赖
├── Task 4: 毛玻璃效果 ← Task 2 依赖
└── Task 5: 动态状态栏 ← Task 2 依赖

阶段 3: 光标与输入
├── Task 6: 平滑光标 ← Task 2 依赖
├── Task 7: 脉冲呼吸 ← Task 6 依赖
├── Task 8: 选中圆角 ← Task 2 依赖
├── Task 9: 字符淡入 ← Task 2 依赖
├── Task 10: 符号配对 ← Task 2 依赖
└── Task 11: 补全弹窗 ← Task 2 依赖

阶段 4: 代码感知
├── Task 12: 彩虹缩进 ← Task 2 依赖
├── Task 13: 悬浮文档 ← Task 2 依赖
└── Task 14: 折叠动画 ← Task 2 依赖

阶段 5: 错误警告
├── Task 15: 波浪线动画 ← Task 2 依赖
└── Task 16: 热力图滚动条 ← Task 2 依赖

阶段 6: 集成测试
└── Task 17: App 集成 ← 所有任务完成
    └── Task 18: 测试 ← Task 17 依赖
```

---

## 并行执行机会

1. **Task 1-2** (动画基础设施) 可并行执行
2. **Task 3-5** (布局) 可并行执行
3. **Task 6-11** (输入体验) 可并行执行
4. **Task 12-14** (代码感知) 可并行执行
5. **Task 15-16** (错误警告) 可并行执行

---

## 验收标准

- [ ] 所有 18 个任务完成
- [ ] `cargo check -p opencode-tui` 无错误
- [ ] `cargo test -p opencode-tui` 全部通过
- [ ] 动画系统运行流畅 (60fps)
- [ ] 内存使用无明显增长
- [ ] 向后兼容：现有功能不受影响
