use clap::Args;
use opencode_core::Config;
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

fn load_config() -> Config {
    let path = Config::config_path();
    Config::load(&path).unwrap_or_default()
}

pub fn run(args: RunArgs) {
    if let Some(prompt) = args.prompt.clone() {
        let config = load_config();
        let model = args
            .model
            .clone()
            .or(config.model)
            .unwrap_or_else(|| "gpt-4o".to_string());

        println!("Mode: non-interactive");
        println!("Model: {}", model);
        println!("Prompt: {}", prompt);
        return;
    }

    let mut app = App::new();

    if let Some(agent) = args.agent {
        app.agent = agent;
    }

    if let Err(e) = app.run() {
        eprintln!("Error running TUI: {}", e);
    }
}
