use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use opencode_tui::dialogs::credential_select::{CredentialEntry, CredentialSelectDialog};
use opencode_tui::dialogs::Dialog;
use opencode_tui::theme::Theme;
use ratatui::{backend::TestBackend, Frame, Terminal};

fn make_credential(id: &str, name: &str) -> CredentialEntry {
    CredentialEntry {
        id: id.into(),
        name: name.into(),
        created_at: "2024-01-01".into(),
    }
}

#[test]
fn test_credential_select_dialog_renders_empty() {
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let dialog = CredentialSelectDialog::new(Theme::default(), "openai".into());
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
fn test_credential_select_dialog_scrolling_with_many_items() {
    let backend = TestBackend::new(50, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut dialog = CredentialSelectDialog::new(Theme::default(), "openai".into());
    dialog.set_credentials(vec![
        make_credential("cred1", "Work Account"),
        make_credential("cred2", "Personal Account"),
        make_credential("cred3", "Backup Account"),
        make_credential("cred4", "Test Account"),
        make_credential("cred5", "Dev Account"),
        make_credential("cred6", "Prod Account"),
        make_credential("cred7", "Staging Account"),
        make_credential("cred8", "CI Account"),
    ]);

    for _ in 0..8 {
        dialog.handle_input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        terminal
            .draw(|f: &mut Frame| {
                dialog.draw(f, f.area());
            })
            .unwrap();
    }

    let action = dialog.handle_input(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(matches!(
        action,
        opencode_tui::dialogs::DialogAction::Confirm(_)
    ));
}

#[test]
fn test_credential_select_dialog_renders_after_navigation_small_area() {
    let backend = TestBackend::new(50, 6);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut dialog = CredentialSelectDialog::new(Theme::default(), "openai".into());
    dialog.set_credentials(vec![
        make_credential("cred1", "Account 1"),
        make_credential("cred2", "Account 2"),
        make_credential("cred3", "Account 3"),
        make_credential("cred4", "Account 4"),
    ]);

    for _ in 0..3 {
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
