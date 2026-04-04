pub mod app;
pub mod command;
pub mod components;
pub mod dialogs;
pub mod input;
pub mod input_parser;
pub mod session;
pub mod shell_handler;
pub mod file_ref_handler;
pub mod layout;
pub mod patch_preview;
pub mod right_panel;
pub mod theme;

pub use app::{App, AppMode, MessageMeta};
pub use command::{Command, CommandAction, CommandRegistry};
pub use components::{
    ConnectionStatus, FileTree, InputAction, InputElement, InputWidget, SkillInfo, SkillsPanel,
    StatusBar, StatusPopoverType, TerminalPanel, TitleBar, TitleBarAction, VirtualList,
};
pub use dialogs::*;
pub use input::{CommandCompleter, FileCompleter, InputBox, InputParser, InputProcessor, InputResult, InputToken};
pub use session::{Session, SessionManager};
pub use shell_handler::{ExecuteResult, ShellHandler};
pub use file_ref_handler::{FileRefHandler, FileRefResult};
pub use layout::{LayoutManager, LayoutPreset, LayoutProportions};
pub use patch_preview::{PatchDecision, PatchPreview};
pub use right_panel::{RightPanel, RightPanelContent, RightPanelRenderData};
pub use theme::{Theme, ThemeManager};
