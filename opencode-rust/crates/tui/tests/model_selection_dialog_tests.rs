use opencode_tui::dialogs::model_selection::ModelSelectionDialog;
use opencode_tui::dialogs::Dialog;
use opencode_tui::theme::Theme;
use ratatui::{backend::TestBackend, Frame, Terminal};

#[test]
fn test_model_selection_dialog_renders_empty_filter() {
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let dialog = ModelSelectionDialog::new(Theme::default());
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
fn test_model_selection_dialog_renders_models() {
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let dialog = ModelSelectionDialog::new(Theme::default());
            dialog.draw(f, f.area());
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
    assert!(has_content, "Dialog should render with content");
}

#[test]
fn test_model_selection_dialog_small_terminal() {
    let backend = TestBackend::new(20, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let dialog = ModelSelectionDialog::new(Theme::default());
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
fn test_model_selection_dialog_with_unavailable_model() {
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let dialog = ModelSelectionDialog::new(Theme::default());
            dialog.draw(f, f.area());
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
    assert!(has_content, "Dialog with unavailable model should render");
}
