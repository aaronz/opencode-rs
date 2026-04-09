use clap::Args;
use opencode_tui::App;

#[derive(Args, Debug)]
pub struct ThreadArgs {
    #[arg(short, long)]
    pub session_id: Option<String>,
}

pub fn run(_args: ThreadArgs) {
    let mut app = App::new();
    if let Err(e) = app.run() {
        eprintln!("Error running TUI: {}", e);
    }
}
