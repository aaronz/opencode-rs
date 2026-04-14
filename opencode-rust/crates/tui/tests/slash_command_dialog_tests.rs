use opencode_tui::command::CommandRegistry;
use opencode_tui::dialogs::slash_command::SlashCommandOverlay;
use opencode_tui::dialogs::Dialog;
use opencode_tui::theme::Theme;
use ratatui::{backend::TestBackend, Frame, Terminal};

#[test]
fn test_slash_command_dialog_renders_with_commands() {
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let dialog = SlashCommandOverlay::new(Theme::default());
            dialog.draw(f, f.area());
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let has_border = buffer
        .content
        .iter()
        .any(|cell| cell.symbol() == "─" || cell.symbol() == "│");
    assert!(has_border, "Dialog should render with border");
}

#[test]
fn test_slash_command_dialog_small_terminal() {
    let backend = TestBackend::new(20, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let dialog = SlashCommandOverlay::new(Theme::default());
            dialog.draw(f, f.area());
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let has_border = buffer
        .content
        .iter()
        .any(|cell| cell.symbol() == "─" || cell.symbol() == "│");
    assert!(
        has_border,
        "Small terminal should still render dialog border"
    );
}

#[test]
fn test_slash_command_dialog_with_filtered_commands() {
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let mut dialog = SlashCommandOverlay::new(Theme::default());
            let registry = CommandRegistry::new();
            dialog.update_input(&registry, "p");
            dialog.draw(f, f.area());
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
    assert!(
        has_content,
        "Dialog with filtered commands should render content"
    );
}
