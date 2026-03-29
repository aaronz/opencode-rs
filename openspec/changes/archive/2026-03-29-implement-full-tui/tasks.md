## 1. Leader Key Foundation

- [x] 1.1 Add `LeaderKeyState` enum to `App` struct
- [x] 1.2 Implement leader key timeout logic (2000ms)
- [x] 1.3 Migrate Ctrl+P to leader key `ctrl+x p`
- [x] 1.4 Migrate Ctrl+T to leader key `ctrl+x t`
- [x] 1.5 Migrate Ctrl+, to leader key `ctrl+x ,`
- [x] 1.6 Migrate Ctrl+M to leader key `ctrl+x m`
- [x] 1.7 Migrate Ctrl+Shift+F to leader key `ctrl+x f`
- [x] 1.8 Migrate Ctrl+H to leader key `ctrl+x h`
- [x] 1.9 Migrate Ctrl+1/2/3 to leader key `ctrl+x 1/2/3`
- [x] 1.10 Migrate Ctrl+` to leader key `ctrl+x `` ``
- [x] 1.11 Add fallback aliases for backward compatibility
- [x] 1.12 Add leader key visual indicator in status bar

## 2. Input Widget Enhancement

- [x] 2.1 Create `InputElement` enum with `Text` and `Chip` variants
- [x] 2.2 Extend `InputWidget` to store `Vec<InputElement>`
- [x] 2.3 Implement chip rendering with background color
- [x] 2.4 Implement atomic deletion for chips
- [x] 2.5 Implement chip navigation with arrow keys
- [x] 2.6 Add shell command prefix highlighting (`!`)
- [x] 2.7 Integrate leader key indicator in input widget

## 3. Slash Command System

- [x] 3.1 Create `Command` struct with name, aliases, description
- [x] 3.2 Create `CommandRegistry` to manage commands
- [x] 3.3 Migrate existing commands from `execute_command()` to registry
- [x] 3.4 Create `SlashCommandOverlay` component
- [x] 3.5 Implement command filtering by input
- [x] 3.6 Implement command selection and execution
- [x] 3.7 Replace CommandPalette with slash command autocomplete
- [x] 3.8 Add `/help` command that shows all available commands

## 4. File Chip Integration

- [x] 4.1 Extend file picker to return selected file as chip
- [x] 4.2 Insert chip into input widget on file selection
- [x] 4.3 Attach file content to context when chip is present
- [x] 4.4 Limit chips to 20 per input

## 5. Dual Mode System

- [x] 5.1 Rename `agent` field to `mode` with `Plan`/`Build` enum
- [x] 5.2 Update Tab key to toggle mode
- [x] 5.3 Update status bar to display current mode
- [x] 5.4 Add mode color indicators (muted for Plan, accent for Build)
- [x] 5.5 Restrict file modifications in Plan mode

## 6. Diff Review Loop

- [x] 6.1 Create `DiffState` enum (Pending, Accepted, Rejected, Editing)
- [x] 6.2 Create `DiffReviewOverlay` component
- [x] 6.3 Implement side-by-side diff layout
- [x] 6.4 Implement stacked diff layout
- [x] 6.5 Add Y/N/E key handling during diff review
- [x] 6.6 Integrate with file modification system
- [x] 6.7 Add syntax highlighting for diff content

## 7. Tool Accordion UI

- [x] 7.1 Create `ToolCall` struct with name, status, output
- [x] 7.2 Create `ToolStatus` enum (Running, Success, Failed)
- [x] 7.3 Replace `tool_output: Vec<String>` with `Vec<ToolCall>`
- [x] 7.4 Implement accordion rendering (collapsed by default)
- [x] 7.5 Add loading animation for running tools
- [x] 7.6 Add success/failure indicators
- [x] 7.7 Implement `/details` command to toggle expansion
- [x] 7.8 Preserve ANSI colors in tool output

## 8. External Editor Integration

- [x] 8.1 Add `ctrl+x e` key binding
- [x] 8.2 Create temporary file in system temp directory
- [x] 8.3 Spawn `$EDITOR` process
- [x] 8.4 Read file content after editor closes
- [x] 8.5 Insert content into input widget
- [x] 8.6 Clean up temporary file
- [x] 8.7 Handle missing `$EDITOR` gracefully

## 9. Theme System

- [x] 9.1 Create `ThemeManager` with preset storage
- [x] 9.2 Add catppuccin theme preset
- [x] 9.3 Add tokyonight theme preset
- [x] 9.4 Add nord theme preset
- [x] 9.5 Add gruvbox theme preset
- [x] 9.6 Create `/themes` command
- [x] 9.7 Create theme selection (via /theme and ctrl+x t)
- [x] 9.8 Persist theme selection to config
- [x] 9.9 Add truecolor detection and fallback

## 10. Smooth Scrolling

- [x] 10.1 Create `ScrollState` struct with velocity
- [x] 10.2 Implement scroll acceleration
- [x] 10.3 Add max velocity cap
- [x] 10.4 Add deceleration on key release
- [x] 10.5 Add `scroll_speed` config option
- [x] 10.6 Add `scroll_acceleration.enabled` config option
- [x] 10.7 Replace PageUp/PageDown with smooth scrolling

## 11. SIGINT Handler

- [x] 11.1 Capture Ctrl+C during LLM generation
- [x] 11.2 Cancel pending HTTP request
- [x] 11.3 Preserve partial response
- [x] 11.4 Display "[Interrupted]" marker
- [x] 11.5 Return control to input field

## 12. History Roaming

- [x] 12.1 Make Up/Down arrows context-aware
- [x] 12.2 Navigate history when input is empty
- [x] 12.3 Navigate cursor when input has content
- [x] 12.4 Persist history to disk
- [x] 12.5 Load history on startup
- [x] 12.6 Limit history to 100 entries

## 13. Session Manager

- [x] 13.1 Create `Session` struct with ID, messages, timestamp
- [x] 13.2 Create `SessionManager` to manage sessions
- [x] 13.3 Create `/sessions` command
- [x] 13.4 Create session list overlay
- [x] 13.5 Implement session selection
- [x] 13.6 Implement session creation (`/new`)
- [x] 13.7 Implement session persistence to disk
- [x] 13.8 Implement session search
- [x] 13.9 Add `ctrl+x l` key binding

## 14. Status Bar Enhancement

- [x] 14.1 Add mode indicator (Plan/Build)
- [x] 14.2 Add model and provider display
- [x] 14.3 Add leader key hints
- [x] 14.4 Add connection status indicator
- [x] 14.5 Update status bar on mode change

## 15. Drag and Drop (Low Priority)

- [x] 15.1 Detect file drop events
- [x] 15.2 Insert dropped file path as chip
- [x] 15.3 Validate dropped files
- [x] 15.4 Handle image drops for multimodal models
