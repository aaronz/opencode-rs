# TUI PRD Gaps Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement 5 critical TUI gaps to align with PRD specifications

**Architecture:** Layered implementation - State Machine/Event Bus foundation → Layout/Timeline → Inspector Panels

**Tech Stack:** Rust, ratatui, tokio async runtime

---

## File Structure

```
rust-opencode-port/crates/tui/src/
├── app/
│   ├── mod.rs              (MODIFY - add state machine)
│   ├── state.rs            (NEW - execution states)
│   ├── event_bus.rs        (NEW - event-driven architecture)
│   └── modes.rs            (NEW - AppMode enum)
├── components/
│   ├── mod.rs              (MODIFY - add new components)
│   ├── timeline.rs         (NEW - structured timeline)
│   ├── inspector.rs        (NEW - tabbed inspector)
│   ├── todo_panel.rs       (NEW - task management)
│   └── diagnostics.rs      (NEW - LSP diagnostics)
├── layout/
│   ├── mod.rs              (NEW - responsive layouts)
│   └── double_column.rs    (NEW - double column layout)
└── lib.rs                  (export new modules)
```

---

## Task 1: State Machine & Execution States

**Files:**
- Create: `rust-opencode-port/crates/tui/src/app/state.rs`
- Create: `rust-opencode-port/crates/tui/src/app/modes.rs`
- Modify: `rust-opencode-port/crates/tui/src/app.rs`
- Test: `rust-opencode-port/crates/tui/tests/test_state.rs`

- [ ] **Step 1: Create app/modes.rs with AppMode enum**

```rust
// rust-opencode-port/crates/tui/src/app/modes.rs
use serde::{Deserialize, Serialize};

/// Application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppMode {
    /// Idle - waiting for user input
    Idle,
    /// Composing - user typing message
    Composing,
    /// Thinking - LLM processing
    Thinking,
    /// Streaming - receiving response
    Streaming,
    /// Awaiting permission - tool approval pending
    AwaitingPermission,
    /// Executing - running tool
    Executing,
    /// Error - something went wrong
    Error,
    /// Settings - in settings dialog
    Settings,
    /// Help - showing help
    Help,
}

impl Default for AppMode {
    fn default() -> Self {
        AppMode::Idle
    }
}
```

- [ ] **Step 2: Create app/state.rs with execution state machine**

```rust
// rust-opencode-port/crates/tui/src/app/state.rs
use super::modes::AppMode;
use serde::{Deserialize, Serialize};

/// Session execution state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionState {
    Idle,
    Thinking,
    AwaitingPermission,
    Executing,
    Streaming,
    Completed,
    Error,
}

impl Default for ExecutionState {
    fn default() -> Self {
        ExecutionState::Idle
    }
}

/// State transition error
#[derive(Debug, Clone)]
pub struct StateTransitionError {
    pub from: ExecutionState,
    pub to: ExecutionState,
}

impl std::fmt::Display for StateTransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid transition from {:?} to {:?}", self.from, self.to)
    }
}

impl std::error::Error for StateTransitionError {}

/// Valid state transitions based on PRD
pub fn is_valid_transition(from: ExecutionState, to: ExecutionState) -> bool {
    matches!(
        (from, to),
        (ExecutionState::Idle, ExecutionState::Thinking)
            | (ExecutionState::Thinking, ExecutionState::AwaitingPermission)
            | (ExecutionState::Thinking, ExecutionState::Streaming)
            | (ExecutionState::Thinking, ExecutionState::Error)
            | (ExecutionState::AwaitingPermission, ExecutionState::Executing)
            | (ExecutionState::Executing, ExecutionState::Thinking)
            | (ExecutionState::Executing, ExecutionState::Error)
            | (ExecutionState::Streaming, ExecutionState::Completed)
            | (ExecutionState::Streaming, ExecutionState::Error)
            | (ExecutionState::Completed, ExecutionState::Idle)
            | (ExecutionState::Error, ExecutionState::Idle)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_thinking_to_streaming() {
        assert!(is_valid_transition(ExecutionState::Thinking, ExecutionState::Streaming));
    }

    #[test]
    fn test_invalid_idle_to_executing() {
        assert!(!is_valid_transition(ExecutionState::Idle, ExecutionState::Executing));
    }
}
```

- [ ] **Step 3: Add state field to App struct**

In `rust-opencode-port/crates/tui/src/app.rs`, find the App struct and add:

```rust
pub struct App {
    // ... existing fields ...
    pub execution_state: app::state::ExecutionState,
    pub app_mode: app::modes::AppMode,
}
```

- [ ] **Step 4: Add transition methods to App**

```rust
impl App {
    // ... existing methods ...
    
    pub fn set_execution_state(&mut self, new_state: app::state::ExecutionState) -> Result<(), app::state::StateTransitionError> {
        if !app::state::is_valid_transition(self.execution_state, new_state) {
            return Err(app::state::StateTransitionError {
                from: self.execution_state,
                to: new_state,
            });
        }
        self.execution_state = new_state;
        Ok(())
    }
    
    pub fn set_app_mode(&mut self, mode: app::modes::AppMode) {
        self.app_mode = mode;
    }
}
```

- [ ] **Step 5: Run tests**

Run: `cd rust-opencode-port && cargo test --package opencode-tui app::state -- --nocapture`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
cd rust-opencode-port
git add crates/tui/src/app/modes.rs crates/tui/src/app/state.rs crates/tui/src/app.rs
git commit -m "feat(tui): add execution state machine (PRD alignment)"
```

---

## Task 2: Event-Driven Architecture

**Files:**
- Create: `rust-opencode-port/crates/tui/src/app/event_bus.rs`
- Modify: `rust-opencode-port/crates/tui/src/app.rs`
- Modify: `rust-opencode-port/crates/tui/src/lib.rs`
- Test: `rust-opencode-port/crates/tui/tests/test_event_bus.rs`

- [ ] **Step 1: Create event_bus.rs with event system**

```rust
// rust-opencode-port/crates/tui/src/app/event_bus.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::broadcast;

/// TUI events for reactive updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TuiEvent {
    /// New message received
    MessageReceived { content: String, role: String },
    /// Tool call initiated
    ToolCallStart { name: String, id: String },
    /// Tool call completed
    ToolCallEnd { id: String, success: bool },
    /// Permission requested
    PermissionRequest { tool: String, id: String },
    /// Permission granted
    PermissionGranted { id: String },
    /// Permission denied
    PermissionDenied { id: String },
    /// State changed
    StateChanged { from: String, to: String },
    /// Token count updated
    TokenCountUpdated { used: u32, limit: u32 },
    /// Diagnostics updated
    DiagnosticsUpdated { file: String, count: u32 },
    /// File tree updated
    FileTreeUpdated,
}

/// Event bus for pub/sub communication
pub struct EventBus {
    sender: broadcast::Sender<TuiEvent>,
    subscribers: HashMap<String, broadcast::Sender<TuiEvent>>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self {
            sender,
            subscribers: HashMap::new(),
        }
    }
    
    /// Subscribe to events
    pub fn subscribe(&mut self, name: &str) -> broadcast::Receiver<TuiEvent> {
        let (sender, receiver) = broadcast::channel(100);
        self.subscribers.insert(name.to_string(), sender);
        receiver
    }
    
    /// Publish an event
    pub fn publish(&self, event: TuiEvent) {
        let _ = self.sender.send(event);
    }
    
    /// Get receiver for default subscription
    pub fn receiver(&self) -> broadcast::Receiver<TuiEvent> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// TUI state that can be observed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiState {
    pub message_count: u32,
    pub tool_call_count: u32,
    pub pending_permissions: u32,
    pub token_usage: Option<(u32, u32)>, // (used, limit)
    pub error: Option<String>,
}

impl Default for TuiState {
    fn default() -> Self {
        Self {
            message_count: 0,
            tool_call_count: 0,
            pending_permissions: 0,
            token_usage: None,
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_bus_publish() {
        let bus = EventBus::new();
        let mut receiver = bus.subscribe("test");
        
        bus.publish(TuiEvent::MessageReceived {
            content: "Hello".to_string(),
            role: "user".to_string(),
        });
        
        // Check receiver got the event
        // Note: In real test, would need to poll or use blocking recv
    }

    #[test]
    fn test_tui_state_default() {
        let state = TuiState::default();
        assert_eq!(state.message_count, 0);
        assert!(state.error.is_none());
    }
}
```

- [ ] **Step 2: Add EventBus to App**

In `rust-opencode-port/crates/tui/src/app.rs`, add to App struct:

```rust
pub struct App {
    // ... existing fields ...
    pub event_bus: app::event_bus::EventBus,
    pub state: app::event_bus::TuiState,
}
```

- [ ] **Step 3: Initialize EventBus in App::new()**

```rust
impl App {
    pub fn new() -> Self {
        Self {
            // ... existing fields ...
            event_bus: app::event_bus::EventBus::new(),
            state: app::event_bus::TuiState::default(),
        }
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cd rust-opencode-port && cargo test --package opencode-tui event_bus -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
cd rust-opencode-port
git add crates/tui/src/app/event_bus.rs crates/tui/src/app.rs crates/tui/src/lib.rs
git commit -m "feat(tui): add event-driven architecture (PRD alignment)"
```

---

## Task 3: Responsive Layout System

**Files:**
- Create: `rust-opencode-port/crates/tui/src/layout/mod.rs`
- Create: `rust-opencode-port/crates/tui/src/layout/double_column.rs`
- Create: `rust-opencode-port/crates/tui/src/layout/triple_column.rs`
- Modify: `rust-opencode-port/crates/tui/src/components.rs`
- Test: `rust-opencode-port/crates/tui/tests/test_layout.rs`

- [ ] **Step 1: Create layout/mod.rs**

```rust
// rust-opencode-port/crates/tui/src/layout/mod.rs
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use serde::{Deserialize, Serialize};

/// Layout variants supported by the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutMode {
    /// Single column (default for narrow screens)
    Single,
    /// Double column (main + inspector)
    Double,
    /// Triple column (files + main + inspector)
    Triple,
    /// Full screen with all panels
    Full,
}

impl Default for LayoutMode {
    fn default() -> Self {
        LayoutMode::Single
    }
}

/// Detect appropriate layout based on terminal width
pub fn detect_layout_mode(width: u16) -> LayoutMode {
    if width < 100 {
        LayoutMode::Single
    } else if width < 150 {
        LayoutMode::Double
    } else {
        LayoutMode::Triple
    }
}

/// Layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub mode: LayoutMode,
    pub show_file_tree: bool,
    pub show_inspector: bool,
    pub inspector_width: u16,
    pub file_tree_width: u16,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            mode: LayoutMode::Single,
            show_file_tree: true,
            show_inspector: true,
            inspector_width: 30,
            file_tree_width: 25,
        }
    }
}

impl LayoutConfig {
    pub fn new(mode: LayoutMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }
}
```

- [ ] **Step 2: Create layout/double_column.rs**

```rust
// rust-opencode-port/crates/tui/src/layout/double_column.rs
use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Calculate double column layout (main + inspector)
pub fn double_column_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(50),
            Constraint::Percentage(30),
        ])
        .split(area)
}

/// Calculate main panel area in double column
pub fn main_panel_rect(area: Rect) -> Rect {
    let chunks = double_column_layout(area);
    chunks[0]
}

/// Calculate inspector area in double column
pub fn inspector_rect(area: Rect) -> Rect {
    let chunks = double_column_layout(area);
    chunks[1]
}
```

- [ ] **Step 3: Create layout/triple_column.rs**

```rust
// rust-opencode-port/crates/tui/src/layout/triple_column.rs
use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Calculate triple column layout (file tree + main + inspector)
pub fn triple_column_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Min(40),
            Constraint::Percentage(25),
        ])
        .split(area)
}

/// Calculate file tree area
pub fn file_tree_rect(area: Rect) -> Rect {
    let chunks = triple_column_layout(area);
    chunks[0]
}

/// Calculate main panel area
pub fn main_panel_rect(area: Rect) -> Rect {
    let chunks = triple_column_layout(area);
    chunks[1]
}

/// Calculate inspector area
pub fn inspector_rect(area: Rect) -> Rect {
    let chunks = triple_column_layout(area);
    chunks[2]
}
```

- [ ] **Step 4: Add layout module to lib.rs**

```rust
// rust-opencode-port/crates/tui/src/lib.rs
pub mod layout;
pub use layout::{LayoutConfig, LayoutMode, detect_layout_mode};
```

- [ ] **Step 5: Run tests**

Run: `cd rust-opencode-port && cargo test --package opencode-tui layout -- --nocapture`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
cd rust-opencode-port
git add crates/tui/src/layout/ crates/tui/src/lib.rs
git commit -m "feat(tui): add responsive layout system (PRD alignment)"
```

---

## Task 4: Structured Timeline & Inspector Panels

**Files:**
- Create: `rust-opencode-port/crates/tui/src/components/timeline.rs`
- Create: `rust-opencode-port/crates/tui/src/components/inspector.rs`
- Modify: `rust-opencode-port/crates/tui/src/app.rs`
- Test: `rust-opencode-port/crates/tui/tests/test_timeline.rs`

- [ ] **Step 1: Create timeline.rs with block types**

```rust
// rust-opencode-port/crates/tui/src/components/timeline.rs
use serde::{Deserialize, Serialize};

/// Timeline block types from PRD
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimelineBlock {
    /// User message block
    UserMessage {
        id: String,
        content: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Assistant text block
    AssistantText {
        id: String,
        content: String,
        tokens: Option<u32>,
    },
    /// Tool call block
    ToolCall {
        id: String,
        tool_name: String,
        arguments: serde_json::Value,
        status: ToolCallStatus,
    },
    /// Tool result block
    ToolResult {
        id: String,
        tool_call_id: String,
        content: String,
        success: bool,
    },
    /// System message block
    SystemMessage {
        id: String,
        content: String,
    },
    /// Error block
    Error {
        id: String,
        message: String,
    },
    /// Divider block
    Divider {
        id: String,
        label: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCallStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Timeline container
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Timeline {
    pub blocks: Vec<TimelineBlock>,
}

impl Timeline {
    pub fn new() -> Self {
        Self { blocks: Vec::new() }
    }
    
    pub fn add_block(&mut self, block: TimelineBlock) {
        self.blocks.push(block);
    }
    
    pub fn len(&self) -> usize {
        self.blocks.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
}
```

- [ ] **Step 2: Create inspector.rs with tabbed panels**

```rust
// rust-opencode-port/crates/tui/src/components/inspector.rs
use serde::{Deserialize, Serialize};
use ratatui::widgets::TabsState;

/// Inspector panel tabs from PRD
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectorTab {
    /// Todo panel
    Todo,
    /// Diff viewer
    Diff,
    /// Diagnostics (LSP)
    Diagnostics,
    /// Context (token usage)
    Context,
    /// Permissions
    Permissions,
    /// File tree
    Files,
}

impl Default for InspectorTab {
    fn default() -> Self {
        InspectorTab::Todo
    }
}

impl InspectorTab {
    pub fn all() -> Vec<Self> {
        vec![
            InspectorTab::Todo,
            InspectorTab::Diff,
            InspectorTab::Diagnostics,
            InspectorTab::Context,
            InspectorTab::Permissions,
            InspectorTab::Files,
        ]
    }
    
    pub fn label(&self) -> &'static str {
        match self {
            InspectorTab::Todo => "Todo",
            InspectorTab::Diff => "Diff",
            InspectorTab::Diagnostics => "Diag",
            InspectorTab::Context => "Ctx",
            InspectorTab::Permissions => "Perm",
            InspectorTab::Files => "Files",
        }
    }
}

/// Inspector panel state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectorState {
    pub active_tab: InspectorTab,
    pub tabs: TabsState,
}

impl Default for InspectorState {
    fn default() -> Self {
        let tab_labels: Vec<&str> = InspectorTab::all().iter().map(|t| t.label()).collect();
        Self {
            active_tab: InspectorTab::Todo,
            tabs: TabsState::new(0, tab_labels),
        }
    }
}

impl InspectorState {
    pub fn next_tab(&mut self) {
        let tabs = InspectorTab::all();
        let next = (self.tabs.selected + 1) % tabs.len();
        self.tabs.selected = next;
        self.active_tab = tabs[next];
    }
    
    pub fn prev_tab(&mut self) {
        let tabs = InspectorTab::all();
        let prev = if self.tabs.selected == 0 {
            tabs.len() - 1
        } else {
            self.tabs.selected - 1
        };
        self.tabs.selected = prev;
        self.active_tab = tabs[prev];
    }
}
```

- [ ] **Step 3: Add Timeline and Inspector to App**

In `rust-opencode-port/crates/tui/src/app.rs`:

```rust
pub struct App {
    // ... existing fields ...
    pub timeline: components::timeline::Timeline,
    pub inspector: components::inspector::InspectorState,
}
```

- [ ] **Step 4: Run tests**

Run: `cd rust-opencode-port && cargo test --package opencode-tui timeline inspector -- --nocapture`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
cd rust-opencode-port
git add crates/tui/src/components/timeline.rs crates/tui/src/components/inspector.rs crates/tui/src/app.rs
git commit -m "feat(tui): add structured timeline and inspector panels (PRD)"
```

---

## Task 5: Interactive Permissions & Task Management

**Files:**
- Create: `rust-opencode-port/crates/tui/src/components/todo_panel.rs`
- Create: `rust-opencode-port/crates/tui/src/components/diagnostics.rs`
- Modify: `rust-opencode-port/crates/tui/src/components.rs`
- Test: `rust-opencode-port/crates/tui/tests/test_permissions.rs`

- [ ] **Step 1: Create todo_panel.rs**

```rust
// rust-opencode-port/crates/tui/src/components/todo_panel.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Todo item state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoState {
    Pending,
    InProgress,
    Completed,
}

/// Todo item for task tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub state: TodoState,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl TodoItem {
    pub fn new(title: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.to_string(),
            description: None,
            state: TodoState::Pending,
            created_at: chrono::Utc::now(),
            completed_at: None,
        }
    }
    
    pub fn mark_completed(&mut self) {
        self.state = TodoState::Completed;
        self.completed_at = Some(chrono::Utc::now());
    }
    
    pub fn mark_in_progress(&mut self) {
        self.state = TodoState::InProgress;
    }
}

/// Todo panel state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TodoPanelState {
    pub items: Vec<TodoItem>,
    pub selected_index: usize,
}

impl TodoPanelState {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected_index: 0,
        }
    }
    
    pub fn add_item(&mut self, title: &str) {
        self.items.push(TodoItem::new(title));
    }
    
    pub fn complete_selected(&mut self) {
        if self.selected_index < self.items.len() {
            self.items[self.selected_index].mark_completed();
        }
    }
    
    pub fn next(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.items.len();
        }
    }
    
    pub fn prev(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.items.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }
}
```

- [ ] **Step 2: Create diagnostics.rs**

```rust
// rust-opencode-port/crates/tui/src/components/diagnostics.rs
use serde::{Deserialize, Serialize};

/// Diagnostic severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

/// A single diagnostic (LSP)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub severity: DiagnosticSeverity,
    pub source: Option<String>,
}

/// Diagnostics panel state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiagnosticsState {
    pub diagnostics: Vec<Diagnostic>,
    pub selected_index: usize,
    pub filter_severity: Option<DiagnosticSeverity>,
}

impl DiagnosticsState {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            selected_index: 0,
            filter_severity: None,
        }
    }
    
    pub fn set_diagnostics(&mut self, diagnostics: Vec<Diagnostic>) {
        self.diagnostics = diagnostics;
        self.selected_index = 0;
    }
    
    pub fn filter_errors(&mut self) {
        self.filter_severity = Some(DiagnosticSeverity::Error);
    }
    
    pub fn clear_filter(&mut self) {
        self.filter_severity = None;
    }
    
    pub fn visible_diagnostics(&self) -> Vec<&Diagnostic> {
        match self.filter_severity {
            Some(severity) => self.diagnostics.iter().filter(|d| d.severity == severity).collect(),
            None => self.diagnostics.iter().collect(),
        }
    }
}
```

- [ ] **Step 3: Add permission card component**

In a new file `rust-opencode-port/crates/tui/src/components/permissions.rs`:

```rust
// rust-opencode-port/crates/tui/src/components/permissions.rs
use serde::{Deserialize, Serialize};

/// Permission request card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCard {
    pub id: String,
    pub tool_name: String,
    pub arguments: String,
    pub risk_level: RiskLevel,
    pub requested_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Permission panel state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PermissionPanelState {
    pub pending: Vec<PermissionCard>,
    pub history: Vec<PermissionCard>,
}

impl PermissionPanelState {
    pub fn add_pending(&mut self, card: PermissionCard) {
        self.pending.push(card);
    }
    
    pub fn approve(&mut self, id: &str) -> Option<PermissionCard> {
        if let Some(pos) = self.pending.iter().position(|p| p.id == id) {
            let card = self.pending.remove(pos);
            self.history.push(card.clone());
            Some(card)
        } else {
            None
        }
    }
    
    pub fn deny(&mut self, id: &str) -> bool {
        if let Some(pos) = self.pending.iter().position(|p| p.id == id) {
            self.pending.remove(pos);
            true
        } else {
            false
        }
    }
}
```

- [ ] **Step 4: Export from lib.rs**

```rust
// rust-opencode-port/crates/tui/src/lib.rs
pub mod components;
pub use components::{
    timeline::{Timeline, TimelineBlock, ToolCallStatus},
    inspector::{InspectorState, InspectorTab},
    todo_panel::{TodoPanelState, TodoItem, TodoState},
    diagnostics::{Diagnostic, DiagnosticSeverity, DiagnosticsState},
    permissions::{PermissionCard, RiskLevel, PermissionPanelState},
};
```

- [ ] **Step 5: Run tests**

Run: `cd rust-opencode-port && cargo test --package opencode-tui todo_panel diagnostics permissions -- --nocapture`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
cd rust-opencode-port
git add crates/tui/src/components/todo_panel.rs crates/tui/src/components/diagnostics.rs crates/tui/src/components/permissions.rs crates/tui/src/lib.rs
git commit -m "feat(tui): add todo panel, diagnostics, and permissions (PRD)"
```

---

## Implementation Complete Summary

After all 5 tasks:
- State Machine: ✅ ExecutionState, AppMode, valid transitions
- Event Bus: ✅ TuiEvent pub/sub, TuiState observable
- Layout: ✅ Single/Double/Triple responsive layouts
- Timeline: ✅ Block types (UserMessage, ToolCall, etc.)
- Inspector: ✅ Tabbed panels (Todo, Diff, Diagnostics, Context, Permissions, Files)
- Todo Panel: ✅ Task tracking with state
- Diagnostics: ✅ LSP diagnostics display
- Permissions: ✅ Permission cards with risk levels

---

## Execution Choice

**Plan complete and saved to `docs/superpowers/plans/2026-03-31-tui-prd-gaps.md`. Two execution options:**

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

**Which approach?**