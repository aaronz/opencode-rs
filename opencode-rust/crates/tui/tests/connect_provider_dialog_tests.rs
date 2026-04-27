use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use opencode_tui::dialogs::connect_provider::ConnectProviderDialog;
use opencode_tui::dialogs::Dialog;
use opencode_tui::theme::Theme;
use ratatui::{backend::TestBackend, Frame, Terminal};

#[test]
fn test_connect_provider_dialog_renders() {
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let dialog = ConnectProviderDialog::new(Theme::default());
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
fn test_connect_provider_dialog_renders_small_terminal() {
    let backend = TestBackend::new(20, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let dialog = ConnectProviderDialog::new(Theme::default());
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
fn test_connect_provider_dialog_scrolling_with_many_items() {
    let backend = TestBackend::new(60, 12);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut dialog = ConnectProviderDialog::new(Theme::default());

    for _ in 0..18 {
        dialog.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        terminal
            .draw(|f: &mut Frame| {
                dialog.draw(f, f.area());
            })
            .unwrap();
    }

    let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    match action {
        opencode_tui::dialogs::DialogAction::Confirm(provider_id) => {
            assert_eq!(provider_id, "minimax", "After 18 Down presses from index 0: (0 + 18) % 21 = 18 = minimax");
        }
        _ => panic!("Expected Confirm action, got {:?}", action),
    }
}

#[test]
fn test_connect_provider_dialog_renders_after_navigation_small_area() {
    let backend = TestBackend::new(60, 8);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut dialog = ConnectProviderDialog::new(Theme::default());

    for _ in 0..10 {
        dialog.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
    }

    terminal
        .draw(|f: &mut Frame| {
            dialog.draw(f, f.area());
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let has_border = buffer
        .content
        .iter()
        .any(|cell| cell.symbol() == "─" || cell.symbol() == "│");
    assert!(has_border, "Dialog should render with border in small area");
}
