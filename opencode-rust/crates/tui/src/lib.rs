pub mod app;
pub mod cli;
pub mod command;
pub mod components;
pub mod config;
pub mod dialogs;
pub mod file_ref_handler;
pub mod input;
pub mod input_parser;
pub mod keybinding;
pub mod layout;
pub mod patch_preview;
pub mod plugin;
pub mod plugin_api;
pub mod render;
pub mod right_panel;
pub mod server_protocol;
pub mod server_ws;
pub mod session;
pub mod shell_handler;
pub mod theme;
pub mod widgets;

pub use app::{App, AppMode, MessageMeta};
pub use cli::{CliArgs, OutputFormat, PermissionMode};
pub use command::{Command, CommandAction, CommandRegistry};
pub use components::{
    Banner, ConnectionStatus, FileTree, InputAction, InputElement, InputWidget, Sidebar,
    SidebarSection, SidebarSectionState, SidebarSectionType, SkillInfo, SkillsPanel, StartupInfo,
    StatusBar, StatusPopoverType, TerminalPanel, TitleBar, TitleBarAction, VirtualList,
};
pub use config::{
    Config, CustomTheme, DiffStyle, KeybindConfig, ProviderConfig, TuiConfig, UserConfig,
};
pub use dialogs::*;
pub use file_ref_handler::{FileRefHandler, FileRefResult};
pub use input::{
    CommandCompleter, EditorLauncher, FileCompleter, FileSuggestion, InputBox, InputHistory,
    InputParser, InputProcessor, InputResult, InputToken,
};
pub use keybinding::{
    DefaultKeybindings, Key, Keybinding, KeybindingAction, KeybindingConfig, KeybindingRegistry,
    KeyCode, KeyModifiers,
};
pub use layout::{LayoutManager, LayoutPreset, LayoutProportions};
pub use patch_preview::{PatchDecision, PatchPreview};
pub use plugin::{TuiPluginEntry, TuiPluginError, TuiPluginManager};
pub use plugin_api::{
    ApiVersion, CommandContext, CommandContextState, CommandMessage, CommandResult, PluginCommand,
    PluginCommandError, PluginCommandRegistry, PluginEvent, PluginEventData, PluginEventError,
    PluginEventRegistry, PluginRoute, PluginRouteError, PluginRouteRegistry, PluginStateError,
    PluginStateRegistry, PluginTheme, PluginThemeError, PluginThemeRegistry, RegisteredCommand,
    RegisteredEvent, RegisteredRoute, RegisteredTheme, RouteContext, RouteParams, RouteResult,
    ThemeColors, VERSION,
};
pub use render::{MarkdownRenderer, SyntaxHighlighter};
pub use right_panel::{RightPanel, RightPanelContent, RightPanelRenderData};
pub use session::{Session, SessionManager};
pub use shell_handler::{ExecuteResult, InterruptibleHandle, ShellHandler};
pub use theme::{Theme, ThemeManager};
pub use widgets::{
    CodeBlock, CommandItem, CommandPalette, FileItem, FileSelectionList, MessageBubble,
    MessageRole, ProgressBar, Scrollbar, Spinner, SpinnerState, ThinkingIndicator,
};
