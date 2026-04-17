use opencode_tui::dialogs::connect_method::ConnectMethodDialog;
use opencode_tui::dialogs::Dialog;
use opencode_tui::theme::Theme;
use ratatui::{backend::TestBackend, Frame, Terminal};

#[test]
fn test_connect_method_dialog_renders_empty_state() {
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let dialog = ConnectMethodDialog::new(Theme::default(), "google".into());
            dialog.draw(f, f.area());
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let has_border = buffer
        .content
        .iter()
        .any(|cell| cell.symbol() == "─" || cell.symbol() == "│");
    assert!(has_border, "Dialog should render with border");

    let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
    assert!(has_content, "Dialog should render with content");
}

#[test]
fn test_connect_method_dialog_renders_with_api_key_method() {
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let dialog = ConnectMethodDialog::new(Theme::default(), "anthropic".into());
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
fn test_connect_method_dialog_renders_small_terminal() {
    let backend = TestBackend::new(20, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let dialog = ConnectMethodDialog::new(Theme::default(), "google".into());
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
fn test_connect_method_dialog_renders_openai_methods() {
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let dialog = ConnectMethodDialog::new(Theme::default(), "openai".into());
            dialog.draw(f, f.area());
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
    assert!(
        has_content,
        "Dialog with OpenAI methods should render content"
    );
}
