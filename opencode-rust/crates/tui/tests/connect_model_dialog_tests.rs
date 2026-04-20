use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use opencode_llm::BrowserAuthModelInfo;
use opencode_tui::dialogs::connect_model::ConnectModelDialog;
use opencode_tui::dialogs::Dialog;
use opencode_tui::theme::Theme;
use ratatui::{backend::TestBackend, Frame, Terminal};

fn make_model(id: &str, name: &str) -> BrowserAuthModelInfo {
    BrowserAuthModelInfo {
        id: id.into(),
        name: name.into(),
        variants: vec![],
    }
}

#[test]
fn test_connect_model_dialog_renders_empty_state() {
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let dialog = ConnectModelDialog::new(Theme::default(), vec![]);
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
fn test_connect_model_dialog_renders_models() {
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let dialog = ConnectModelDialog::new(
                Theme::default(),
                vec![
                    make_model("gpt-4o", "GPT-4o"),
                    make_model("gpt-4o-mini", "GPT-4o Mini"),
                ],
            );
            dialog.draw(f, f.area());
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
    assert!(has_content, "Dialog should render with content");
}

#[test]
fn test_connect_model_dialog_single_model() {
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let dialog =
                ConnectModelDialog::new(Theme::default(), vec![make_model("gpt-4o", "GPT-4o")]);
            dialog.draw(f, f.area());
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
    assert!(has_content, "Dialog with single model should render");
}

#[test]
fn test_connect_model_dialog_small_terminal() {
    let backend = TestBackend::new(20, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let dialog =
                ConnectModelDialog::new(Theme::default(), vec![make_model("gpt-4o", "GPT-4o")]);
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
fn test_connect_model_dialog_many_models() {
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let dialog = ConnectModelDialog::new(
                Theme::default(),
                vec![
                    make_model("gpt-4o", "GPT-4o"),
                    make_model("gpt-4o-mini", "GPT-4o Mini"),
                    make_model("gpt-5", "GPT-5"),
                    make_model("claude-3.5", "Claude 3.5"),
                    make_model("claude-3", "Claude 3"),
                ],
            );
            dialog.draw(f, f.area());
        })
        .unwrap();

    let buffer = terminal.backend().buffer();
    let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");
    assert!(has_content, "Dialog with many models should render");
}

#[test]
fn test_connect_model_dialog_scrolling_with_many_items() {
    let backend = TestBackend::new(70, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut dialog = ConnectModelDialog::new(
        Theme::default(),
        vec![
            make_model("gpt-4o", "GPT-4o"),
            make_model("gpt-4o-mini", "GPT-4o Mini"),
            make_model("gpt-5", "GPT-5"),
            make_model("claude-3.5", "Claude 3.5"),
            make_model("claude-3", "Claude 3"),
            make_model("gemini-pro", "Gemini Pro"),
            make_model("llama-3", "Llama 3"),
            make_model("mistral", "Mistral"),
        ],
    );

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
fn test_connect_model_dialog_renders_after_navigation_small_area() {
    let backend = TestBackend::new(70, 8);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut dialog = ConnectModelDialog::new(
        Theme::default(),
        vec![
            make_model("gpt-4o", "GPT-4o"),
            make_model("gpt-4o-mini", "GPT-4o Mini"),
            make_model("gpt-5", "GPT-5"),
            make_model("claude-3.5", "Claude 3.5"),
            make_model("claude-3", "Claude 3"),
        ],
    );

    for _ in 0..4 {
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
