use clap::Args;
use opencode_tui::App;

#[derive(Args, Debug)]
pub struct RunArgs {
    #[arg(short, long)]
    pub prompt: Option<String>,

    #[arg(short, long)]
    pub agent: Option<String>,

    #[arg(short, long)]
    pub model: Option<String>,

    #[arg(short, long)]
    pub continue_session: Option<String>,

    #[arg(short, long)]
    pub attach: Option<String>,

    #[arg(short = 'y', long)]
    pub yes: bool,

    #[arg(long)]
    pub title: Option<String>,
}

pub fn run(args: RunArgs) {
    eprintln!("Running with prompt: {:?}", args.prompt);

    let mut app = App::new();

    if let Some(prompt) = args.prompt {
        app.input = prompt;
    }

    if let Some(agent) = args.agent {
        app.agent = agent;
    }

    if let Err(e) = app.run() {
        eprintln!("Error running TUI: {}", e);
    }
}
