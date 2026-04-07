pub mod app;
pub mod cli;
pub mod command;
pub mod components;
pub mod config;
pub mod dialogs;
pub mod input;
pub mod input_parser;
pub mod layout;
pub mod patch_preview;
pub mod render;
pub mod right_panel;
pub mod session;
pub mod server_protocol;
pub mod server_ws;
pub mod shell_handler;
pub mod file_ref_handler;
pub mod theme;
pub mod widgets;

pub use app::{App, AppMode, MessageMeta};
pub use cli::{CliArgs, OutputFormat, PermissionMode};
pub use command::{Command, CommandAction, CommandRegistry};
pub use components::{
    Banner, ConnectionStatus, FileTree, InputAction, InputElement, InputWidget, SkillInfo,
    SkillsPanel, StartupInfo, StatusBar, StatusPopoverType, TerminalPanel, TitleBar,
    TitleBarAction, VirtualList,
};
pub use dialogs::*;
pub use input::{CommandCompleter, EditorLauncher, FileCompleter, InputHistory, InputBox, InputParser, InputProcessor, InputResult, InputToken};
pub use render::{MarkdownRenderer, SyntaxHighlighter};
pub use config::{Config, TuiConfig, UserConfig, ProviderConfig};
pub use session::{Session, SessionManager};
pub use shell_handler::{ExecuteResult, ShellHandler};
pub use file_ref_handler::{FileRefHandler, FileRefResult};
pub use layout::{LayoutManager, LayoutPreset, LayoutProportions};
pub use patch_preview::{PatchDecision, PatchPreview};
pub use right_panel::{RightPanel, RightPanelContent, RightPanelRenderData};
pub use theme::{Theme, ThemeManager};
pub use widgets::{
    CodeBlock, CommandPalette, CommandItem, FileItem, FileSelectionList, ProgressBar, ThinkingIndicator,
    MessageBubble, MessageRole, Scrollbar, Spinner, SpinnerState,
};
