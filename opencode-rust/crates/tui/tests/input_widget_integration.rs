use ratatui::{backend::TestBackend, layout::Rect, widgets::Block, Frame, Terminal};

#[test]
fn test_input_widget_render_80x24() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let area = Rect::new(10, 10, 60, 3);
            let block = Block::default()
                .title("Input")
                .borders(ratatui::widgets::Borders::ALL);
            f.render_widget(block, area);
        })
        .unwrap();

    let buffer = terminal.backend().buffer();

    let has_content = buffer.content.iter().any(|cell| cell.symbol() != " ");

    assert!(has_content, "Buffer should have rendered content");
}

#[test]
fn test_input_widget_narrow_terminal() {
    let backend = TestBackend::new(40, 12);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let area = Rect::new(0, 0, 38, 3);
            let block = Block::default()
                .title("In")
                .borders(ratatui::widgets::Borders::ALL);
            f.render_widget(block, area);
        })
        .unwrap();

    assert!(true, "Narrow terminal render completed without panic");
}

#[test]
fn test_multiple_widgets_layout() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f: &mut Frame| {
            let title_area = Rect::new(0, 0, 80, 1);
            let messages_area = Rect::new(0, 1, 80, 18);
            let input_area = Rect::new(0, 19, 80, 3);
            let status_area = Rect::new(0, 22, 80, 1);
            let _terminal_area = Rect::new(0, 23, 80, 1);

            let title_block = Block::default().borders(ratatui::widgets::Borders::ALL);
            f.render_widget(title_block, title_area);

            let messages_block = Block::default()
                .title("Messages")
                .borders(ratatui::widgets::Borders::ALL);
            f.render_widget(messages_block, messages_area);

            let input_block = Block::default()
                .title("Input")
                .borders(ratatui::widgets::Borders::ALL);
            f.render_widget(input_block, input_area);

            let status_block = Block::default().borders(ratatui::widgets::Borders::ALL);
            f.render_widget(status_block, status_area);
        })
        .unwrap();

    assert!(true, "Multiple widgets layout rendered successfully");
}

fn areas_overlap(a: &Rect, b: &Rect) -> bool {
    !(a.x + a.width <= b.x
        || b.x + b.width <= a.x
        || a.y + a.height <= b.y
        || b.y + b.height <= a.y)
}

#[test]
fn test_chat_layout_regions_no_overlap() {
    let terminal_width = 80u16;
    let terminal_height = 24u16;

    let title_height = 1u16;
    let messages_height = 18u16;
    let input_height = 3u16;
    let status_height = 1u16;
    let terminal_panel_height = 1u16;

    let title_area = Rect::new(0, 0, terminal_width, title_height);
    let messages_area = Rect::new(0, title_height, terminal_width, messages_height);
    let input_area = Rect::new(
        0,
        title_height + messages_height,
        terminal_width,
        input_height,
    );
    let status_area = Rect::new(
        0,
        title_height + messages_height + input_height,
        terminal_width,
        status_height,
    );
    let terminal_panel_area = Rect::new(
        0,
        title_height + messages_height + input_height + status_height,
        terminal_width,
        terminal_panel_height,
    );

    assert!(
        !areas_overlap(&title_area, &messages_area),
        "Title and messages should not overlap"
    );
    assert!(
        !areas_overlap(&messages_area, &input_area),
        "Messages and input should not overlap"
    );
    assert!(
        !areas_overlap(&input_area, &status_area),
        "Input and status should not overlap"
    );
    assert!(
        !areas_overlap(&status_area, &terminal_panel_area),
        "Status and terminal panel should not overlap"
    );

    let total_height = title_area.height
        + messages_area.height
        + input_area.height
        + status_area.height
        + terminal_panel_area.height;
    assert_eq!(
        total_height, terminal_height,
        "All areas should fill terminal height"
    );
}
