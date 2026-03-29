use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io::stdout;

#[derive(Default)]
struct App {
    counter: i32,
    quit: bool,
}

fn main() -> std::io::Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut app = App::default();

    loop {
        terminal.draw(|f| ui(f, &app))?;
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('j') => app.counter += 1,
                        KeyCode::Char('k') => app.counter -= 1,
                        KeyCode::Char('q') => app.quit = true,
                        _ => {}
                    }
                }
            }
        }
        if app.quit {
            break;
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn ui(frame: &mut Frame, app: &App) {
    let area = Rect::new(0, 0, 80, 24);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!("Counter: {}", app.counter));
    let paragraph = Paragraph::new("Press j/k to adjust, q to quit").block(block);
    frame.render_widget(paragraph, area);
}
