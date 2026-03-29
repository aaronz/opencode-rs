# TUI V2 交互效果实现设计

## 概述

基于 `docs/design-tui-v2.md` 设计文档，为 rust-opencode-port TUI 添加完整的沉浸式交互体验。

## 设计文档映射

本文档实现 `docs/design-tui-v2.md` 中的全部 5 个章节：
1. 整体布局与视觉感知
2. 光标交互效果
3. 输入与反馈效果
4. 代码块与结构感知
5. 错误与警告交互

---

## 第一部分：整体布局与视觉感知

### 1.1 侧边栏抽屉式设计

**目标**: 鼠标悬停时图标微亮，点击后以平滑的侧滑动画开启

**修改文件**: `crates/tui/src/components/file_tree.rs`

```rust
// 新增动画状态
pub struct FileTreeAnimation {
    pub is_expanding: bool,
    pub is_collapsing: bool,
    pub progress: f32,  // 0.0 - 1.0
    pub target_width: u16,
    pub current_width: u16,
}

impl FileTree {
    // 修改 show 方法，触发动画
    pub fn toggle(&mut self) {
        if self.visible {
            self.animation.is_expanding = false;
            self.animation.is_collapsing = true;
        } else {
            self.animation.is_expanding = true;
            self.animation.is_collapsing = false;
            self.visible = true;
        }
        self.animation.progress = 0.0;
    }
    
    // 修改 draw 方法，支持动画宽度
    pub fn draw(&mut self, f: &mut Frame, area: Rect) {
        let width = if self.animation.is_expanding || self.animation.is_collapsing {
            self.update_animation();
            self.animation.current_width
        } else {
            self.width
        };
        
        let tree_area = Rect::new(area.x, area.y, width, area.height);
        self.render_tree(f, tree_area);
    }
    
    fn update_animation(&mut self) {
        const ANIMATION_SPEED: f32 = 0.1;
        
        if self.animation.is_expanding {
            self.animation.progress += ANIMATION_SPEED;
            if self.animation.progress >= 1.0 {
                self.animation.progress = 1.0;
                self.animation.is_expanding = false;
            }
            self.animation.current_width = (self.animation.target_width as f32 * self.animation.progress) as u16;
        }
        
        if self.animation.is_collapsing {
            self.animation.progress -= ANIMATION_SPEED;
            if self.animation.progress <= 0.0 {
                self.animation.progress = 0.0;
                self.animation.is_collapsing = false;
                self.visible = false;
            }
            self.animation.current_width = (self.animation.target_width as f32 * self.animation.progress) as u16;
        }
    }
}
```

**面包屑导航**:
```rust
pub struct Breadcrumb {
    pub segments: Vec<PathBuf>,
}

impl FileTree {
    pub fn render_breadcrumb(&self, f: &mut Frame, area: Rect) {
        let path_str = self.current_dir.display().to_string();
        let text = Text::from(Span::raw(path_str));
        f.render_widget(text, area);
    }
}
```

### 1.2 毛玻璃效果 (Acrylic/Mica)

**说明**: 受限于终端能力，使用深色渐变和阴影模拟毛玻璃

**修改文件**: `crates/tui/src/themes/`

在主题中添加 backdrop 样式：
```json
// themes/backdrop.json
{
    "name": "backdrop",
    "block": {
        "bg": "dark_gray",
        "fg": "white"
    },
    "title": {
        "bg": "dark_gray", 
        "fg": "light_gray",
        "modifiers": ["bold"]
    },
    "border": {
        "fg": "gray",
        "bg": "dark_gray"
    }
}
```

在 app.rs 中应用：
```rust
fn draw_backdrop(&self, f: &mut Frame) {
    let area = f.area();
    
    // 添加微妙的背景色块
    let backdrop = Block::default()
        .style(Style::default().bg(Color::Rgb(20, 20, 25)));
    
    f.render_widget(backdrop, area);
}
```

### 1.3 动态状态栏颜色

**目标**: 根据当前语言模式改变主题颜色

**修改文件**: `crates/tui/src/components/status_bar.rs`

```rust
pub struct StatusBar {
    // ... existing fields
    pub language_mode: LanguageMode,
}

#[derive(Clone, Copy, PartialEq)]
pub enum LanguageMode {
    None,
    Python,
    Rust,
    JavaScript,
    TypeScript,
    Go,
    // ... more languages
}

impl StatusBar {
    pub fn theme_color(&self) -> Color {
        match self.language_mode {
            LanguageMode::None => Color::Gray,
            LanguageMode::Python => Color::Rgb(55, 118, 171),    // 蓝色
            LanguageMode::Rust => Color::Rgb(222, 165, 132),     // 橙色
            LanguageMode::JavaScript => Color::Rgb(247, 223, 30), // 黄色
            LanguageMode::TypeScript => Color::Rgb(49, 120, 198), // 蓝色
            LanguageMode::Go => Color::Rgb(0, 173, 216),          // 青色
        }
    }
    
    pub fn draw(&self, f: &mut Frame, area: Rect) {
        let color = self.theme_color();
        
        // 渲染带颜色的状态栏
        let block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(color))
            .bg(Color::Rgb(25, 25, 30));
            
        f.render_widget(block, area);
        // ... 其余渲染
    }
}
```

---

## 第二部分：光标交互效果

### 2.1 平滑移动 (Smooth Caret)

**目标**: 光标带有线性插值平移动画

**修改文件**: `crates/tui/src/components/input_widget.rs`

```rust
// 新增光标动画状态
pub struct CursorAnimation {
    pub target_x: u16,
    pub target_y: u16,
    pub current_x: f32,
    pub current_y: f32,
    pub is_animating: bool,
}

impl InputWidget {
    // 修改光标位置设置
    pub fn set_cursor_position(&mut self, x: u16, y: u16) {
        self.cursor_animation.target_x = x;
        self.cursor_animation.target_y = y;
        self.cursor_animation.is_animating = true;
    }
    
    // 每帧更新
    pub fn update_cursor_animation(&mut self) {
        if !self.cursor_animation.is_animating {
            return;
        }
        
        const INTERPOLATION_SPEED: f32 = 0.3;
        
        let dx = self.cursor_animation.target_x as f32 - self.cursor_animation.current_x;
        let dy = self.cursor_animation.target_y as f32 - self.cursor_animation.current_y;
        
        if dx.abs() < 0.5 && dy.abs() < 0.5 {
            self.cursor_animation.current_x = self.cursor_animation.target_x as f32;
            self.cursor_animation.current_y = self.cursor_animation.target_y as f32;
            self.cursor_animation.is_animating = false;
        } else {
            self.cursor_animation.current_x += dx * INTERPOLATION_SPEED;
            self.cursor_animation.current_y += dy * INTERPOLATION_SPEED;
        }
    }
    
    // 渲染时使用动画位置
    fn render_cursor(&self, f: &mut Frame, area: Rect) {
        let x = self.cursor_animation.current_x as u16;
        let y = self.cursor_animation.current_y as u16;
        f.set_cursor(area.x + x, area.y + y);
    }
}
```

### 2.2 脉冲呼吸感

**目标**: 光标静止时有透明度渐变的呼吸效果

```rust
pub struct CursorAnimation {
    // ... existing fields
    pub breathe_phase: f32,  // 呼吸相位 0-2π
}

impl InputWidget {
    pub fn update_cursor_animation(&mut self, delta_ms: u64) {
        // 更新呼吸相位
        const BREATHE_SPEED: f32 = 0.003;  // ms^-1
        self.cursor_animation.breathe_phase += BREATHE_SPEED * delta_ms as f32;
        if self.cursor_animation.breathe_phase > std::f32::consts::PI * 2.0 {
            self.cursor_animation.breathe_phase -= std::f32::consts::PI * 2.0;
        }
    }
    
    fn render_cursor(&self, f: &mut Frame, area: Rect) {
        // 计算呼吸透明度
        let breathe = (self.cursor_animation.breathe_phase.sin() + 1.0) / 2.0;  // 0-1
        let alpha = (breathe * 0.5 + 0.5) as u8;  // 50%-100%
        
        let cursor_style = Style::default()
            .bg(Color::LightBlue)
            .add_modifier(Modifier::DIM);
            
        f.set_cursor(area.x + x, area.y + y);
    }
}
```

### 2.3 扩展选中圆角

**目标**: 选中文字时背景高亮块有圆角和柔和边缘

```rust
fn render_selection(&self, f: &mut Frame, area: Rect, start: usize, end: usize) {
    // 计算选中区域的字符位置
    let (start_x, start_y) = self.position_to_coords(start);
    let (end_x, end_y) = self.position_to_coords(end);
    
    // 渲染带圆角的高亮块
    // ratatui 不直接支持圆角，使用多个 block 模拟
    for y in start_y..=end_y {
        let y_offset = area.y + y;
        
        if start_y == end_y {
            // 单行选中
            let x_start = area.x + start_x;
            let width = end_x - start_x;
            let line = Line::from(vec![Span::styled(
                " ".repeat(width.max(1)),
                Style::default()
                    .bg(Color::Rgb(50, 50, 80))
                    .add_modifier(Modifier::ROUNDED)
            )]);
            f.render_widget(line, Rect::new(x_start, y_offset, width, 1));
        } else {
            // 多行选中 - 首行、末行、中间行
            // ...
        }
    }
}
```

---

## 第三部分：输入与反馈效果

### 3.1 字符淡入

**目标**: 每个敲击出的字符有零点几秒的淡入动画

**修改文件**: `crates/tui/src/components/input_widget.rs`

```rust
// 新增字符动画状态
pub struct CharAnimation {
    pub char: char,
    pub x: u16,
    pub y: u16,
    pub fade_progress: f32,  // 0.0 = 完全透明, 1.0 = 完全显示
    pub created_at: Instant,
}

pub struct InputWidget {
    // ... existing fields
    pub char_animations: Vec<CharAnimation>,
}

impl InputWidget {
    pub fn insert_char(&mut self, c: char, x: u16, y: u16) {
        self.content.insert(self.cursor, c.to_string());
        self.cursor += 1;
        
        // 添加字符动画
        self.char_animations.push(CharAnimation {
            char: c,
            x,
            y,
            fade_progress: 0.0,
            created_at: Instant::now(),
        });
    }
    
    pub fn update_char_animations(&mut self) {
        const FADE_DURATION_MS: u64 = 200;
        
        let now = Instant::now();
        self.char_animations.retain(|anim| {
            let elapsed = now.duration_since(anim.created_at).as_millis() as f32;
            anim.fade_progress = (elapsed / FADE_DURATION_MS as f32).min(1.0);
            anim.fade_progress < 1.0
        });
    }
    
    fn render_char_animations(&self, f: &mut Frame, area: Rect) {
        for anim in &self.char_animations {
            let alpha = (anim.fade_progress * 255.0) as u8;
            let style = Style::default()
                .fg(Color::LightWhite)
                .bg(Color::Rgb(30, 30, 40));
            
            let span = Span::styled(anim.char.to_string(), style);
            f.render_widget(
                span,
                Rect::new(area.x + anim.x, area.y + anim.y, 1, 1)
            );
        }
    }
}
```

### 3.2 符号配对

**目标**: 输入 `(` 时预渲染 `)`，Enter 自动补全

```rust
pub struct BracketPair {
    pub left: char,
    pub right: char,
}

const BRACKET_PAIRS: &[(char, char)] = &[
    ('(', ')'),
    ('[', ']'),
    ('{', '}'),
    ('<', '>'),
    ('"', '"'),
    ('\'', '\''),
    ('`', '`'),
];

pub struct InputWidget {
    // ... existing fields
    pub pending_bracket: Option<BracketPair>,
    pub show_ghost_bracket: bool,
}

impl InputWidget {
    pub fn insert_char(&mut self, c: char) {
        // 检查是否是左括号
        if let Some((_, right)) = BRACKET_PAIRS.iter().find(|(l, _)| *l == c) {
            self.pending_bracket = Some(BracketPair {
                left: c,
                right: *right,
            });
            self.show_ghost_bracket = true;
        }
        
        // ... 插入字符逻辑
    }
    
    fn render_ghost_bracket(&self, f: &mut Frame, area: Rect) {
        if !self.show_ghost_bracket {
            return;
        }
        
        let Some(ref pair) = self.pending_bracket else { return };
        
        let ghost_x = self.cursor_x + 1;  // 光标右侧
        let ghost_style = Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::DIM);
            
        f.render_widget(
            Span::styled(pair.right.to_string(), ghost_style),
            Rect::new(area.x + ghost_x, area.y + self.cursor_y, 1, 1)
        );
    }
    
    pub fn handle_enter(&mut self) -> String {
        let mut result = "\n".to_string();
        
        // 如果有 pending bracket，添加缩进
        if let Some(ref pair) = self.pending_bracket {
            result.push_str(&"  ".repeat(self.indent_level));
            result.push(pair.right);  // 补全右括号
            result.push('\n');
            result.push_str(&"  ".repeat(self.indent_level));
            
            self.pending_bracket = None;
            self.show_ghost_bracket = false;
        }
        
        result
    }
}
```

### 3.3 魔法联想 (IntelliSense)

**目标**: 复用现有 Dialog 系统实现补全弹窗

**修改文件**: `crates/tui/src/dialogs/completion.rs (新建)`

```rust
use ratatui::{
    widgets::{Block, List, ListItem, Paragraph},
    Frame,
};

pub struct CompletionDialog {
    pub items: Vec<CompletionItem>,
    pub selected_index: usize,
    pub visible: bool,
}

pub struct CompletionItem {
    pub label: String,
    pub detail: Option<String>,
    pub insert_text: String,
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
                Style::default()
                    .fg(Color::LightGray)
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
                DialogAction::Complete(self.items[self.selected_index].insert_text.clone())
            }
            KeyCode::Esc => {
                self.visible = false;
                DialogAction::Close
            }
            _ => DialogAction::Continue,
        }
    }
    
    fn is_modal(&self) -> bool {
        false  // 非模态，显示在输入框上方
    }
}
```

---

## 第四部分：代码块与结构感知

### 4.1 彩虹括号/缩进线

**目标**: 光标进入区域时缩进线加粗变亮

**修改文件**: `crates/tui/src/components/code_highlight.rs (新建)`

```rust
pub struct IndentGuide {
    pub indent_level: usize,
    pub is_active: bool,
    pub active_color: Color,
}

pub struct CodeHighlight {
    pub indent_guides: Vec<IndentGuide>,
    pub matching_brackets: Vec<BracketMatch>,
}

impl CodeHighlight {
    pub fn update_cursor_position(&mut self, line: usize, col: usize, content: &str) {
        // 重置所有活跃状态
        for guide in &mut self.indent_guides {
            guide.is_active = false;
        }
        
        // 计算当前缩进级别
        let current_indent = self.calculate_indent(line, content);
        
        // 激活当前及更深级别的缩进线
        for guide in &mut self.indent_guides {
            if guide.indent_level >= current_indent {
                guide.is_active = true;
            }
        }
        
        // 查找匹配的括号
        self.find_matching_brackets(line, col, content);
    }
    
    fn calculate_indent(&self, line: usize, content: &str) -> usize {
        let line_content = content.lines().nth(line).unwrap_or("");
        line_content.len() - line_content.trim_start().len()
    }
}
```

### 4.2 悬浮文档

**目标**: Hover 时弹出支持 Markdown 的文档框

**修改文件**: `crates/tui/src/components/hover_popup.rs (新建)`

```rust
pub struct HoverPopup {
    pub visible: bool,
    pub content: String,
    pub position: (u16, u16),  // x, y
    pub size: (u16, u16),
}

impl HoverPopup {
    pub fn show(&mut self, content: String, x: u16, y: u16) {
        self.visible = true;
        self.content = content;
        self.position = (x, y);
        
        // 计算尺寸
        let lines = content.lines().count();
        let max_width = content.lines().map(|l| l.len()).max().unwrap_or(0);
        self.size = ((max_width + 2).min(60) as u16, (lines + 2) as u16);
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
        
        // 支持 Markdown 渲染
        let markdown = self.render_markdown();
        let paragraph = Paragraph::new(markdown)
            .wrap(Wrap { trim: true })
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .bg(Color::Rgb(30, 30, 40)));
        
        f.render_widget(paragraph, area);
    }
    
    fn render_markdown(&self) -> Text {
        // 简化的 Markdown 渲染
        // 实际可使用 markdown-rs 库
        let mut result = Vec::new();
        
        for line in self.content.lines() {
            let processed = if line.starts_with("```") {
                // 代码块
                Span::raw(line)
            } else if line.starts_with('#') {
                // 标题
                Span::styled(line, Style::default().add_modifier(Modifier::BOLD))
            } else {
                Span::raw(line)
            };
            result.push(processed);
        }
        
        Text::from(result)
    }
}
```

### 4.3 折叠动画

**目标**: 代码块折叠时有平滑的消失动画

**修改文件**: `crates/tui/src/components/fold_indicator.rs`

```rust
pub struct FoldIndicator {
    pub line_start: usize,
    pub line_end: usize,
    pub folded: bool,
    pub fold_progress: f32,  // 0.0 = 完全展开, 1.0 = 完全折叠
    pub is_animating: bool,
}

impl FoldIndicator {
    pub fn toggle(&mut self) {
        self.is_animating = true;
        self.folded = !self.folded;
    }
    
    pub fn update(&mut self) {
        const FOLD_SPEED: f32 = 0.1;
        
        if !self.is_animating {
            return;
        }
        
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
        // 根据折叠进度渲染省略号
        let dots = match (self.fold_progress * 3.0) as u8 {
            0 => "",
            1 => ".",
            2 => "..",
            _ => "...",
        };
        
        let text = Paragraph::new(dots)
            .style(Style::default().fg(Color::DarkGray));
            
        f.render_widget(text, area);
    }
}
```

---

## 第五部分：错误与警告交互

### 5.1 波浪线律动

**目标**: 报错波浪线有微弱位移动画

**修改文件**: `crates/tui/src/components/error_underline.rs`

```rust
pub struct ErrorUnderline {
    pub line: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub wave_offset: f32,  // 波动偏移
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
            // 计算波动位移
            let wave = (self.wave_offset + x as f32 * 0.3).sin();
            let display_x = x + (wave * 0.5) as usize;
            
            // 渲染波浪字符
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

### 5.2 边缘预警 (热力图滚动条)

**目标**: 滚动条显示错误位置热力图

**修改文件**: `crates/tui/src/components/heatmap_scrollbar.rs (新建)`

```rust
use ratatui::widgets::ScrollbarState;

pub struct HeatmapScrollbar {
    pub total_lines: usize,
    pub visible_lines: usize,
    pub viewport_start: usize,
    pub errors: Vec<ErrorMarker>,  // 错误位置
    pub warnings: Vec<WarningMarker>,  // 警告位置
}

pub struct ErrorMarker {
    pub line: usize,
    pub severity: ErrorSeverity,
}

#[derive(Clone, Copy)]
pub enum ErrorSeverity {
    Error,
    Warning,
    Info,
}

impl HeatmapScrollbar {
    pub fn new() -> Self {
        Self {
            total_lines: 0,
            visible_lines: 0,
            viewport_start: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    pub fn set_content(&mut self, total: usize, visible: usize) {
        self.total_lines = total;
        self.visible_lines = visible;
    }
    
    pub fn add_error(&mut self, line: usize, severity: ErrorSeverity) {
        match severity {
            ErrorSeverity::Error => self.errors.push(ErrorMarker { line, severity }),
            ErrorSeverity::Warning | ErrorSeverity::Info => 
                self.warnings.push(ErrorMarker { line, severity }),
        }
    }
    
    pub fn render(&self, f: &mut Frame, area: Rect) {
        // 计算热力图颜色
        let colors: Vec<Color> = (0..area.height as usize)
            .map(|i| {
                let line = self.viewport_start + 
                    (i as f32 * self.total_lines as f32 / area.height as f32) as usize;
                
                // 检查是否有错误/警告
                if self.errors.iter().any(|e| e.line == line) {
                    Color::Red
                } else if self.warnings.iter().any(|w| w.line == line) {
                    Color::Yellow
                } else {
                    Color::DarkGray
                }
            })
            .collect();
        
        // 渲染热力图块
        for (i, color) in colors.iter().enumerate() {
            let block = Block::default()
                .bg(*color)
                .fg(*color);
                
            f.render_widget(
                block,
                Rect::new(area.x, area.y + i as u16, 1, 1)
            );
        }
    }
}
```

---

## 任务分解

### 阶段 1: 动画系统基础设施

- [ ] 1.1 创建 `crates/tui/src/animation/mod.rs`
- [ ] 1.2 实现 `Animator` trait
- [ ] 1.3 添加动画状态管理到 App

### 阶段 2: 整体布局

- [ ] 2.1 扩展 FileTree 动画支持
- [ ] 2.2 添加面包屑导航
- [ ] 2.3 实现毛玻璃背景模拟
- [ ] 2.4 动态状态栏颜色

### 阶段 3: 光标与输入

- [ ] 3.1 实现平滑光标移动
- [ ] 3.2 添加脉冲呼吸效果
- [ ] 3.3 实现选中文字圆角
- [ ] 3.4 添加字符淡入动画
- [ ] 3.5 实现符号配对
- [ ] 3.6 创建补全弹窗

### 阶段 4: 代码感知

- [ ] 4.1 实现彩虹缩进线
- [ ] 4.2 创建悬浮文档弹窗
- [ ] 4.3 添加折叠动画

### 阶段 5: 错误警告

- [ ] 5.1 实现波浪线动画
- [ ] 5.2 创建热力图滚动条

---

## 技术细节

### 相关文件

- `crates/tui/src/app.rs`
- `crates/tui/src/components/input_widget.rs`
- `crates/tui/src/components/file_tree.rs`
- `crates/tui/src/components/status_bar.rs`
- `crates/tui/src/components/code_highlight.rs` (新建)
- `crates/tui/src/components/hover_popup.rs` (新建)
- `crates/tui/src/components/fold_indicator.rs` (新建)
- `crates/tui/src/components/error_underline.rs` (新建)
- `crates/tui/src/components/heatmap_scrollbar.rs` (新建)
- `crates/tui/src/dialogs/completion.rs` (新建)
- `crates/tui/src/animation/mod.rs` (新建)
- `crates/tui/src/themes/`

### 依赖

- `ratatui` - 已有
- `crossterm` - 已有
- 无需新增外部依赖

### 限制说明

1. **毛玻璃效果**: 终端不支持真实透明，使用深色渐变模拟
2. **圆角效果**: ratatui 不直接支持，使用多个 block 模拟
3. **动画性能**: 需注意终端刷新率，避免过度渲染
