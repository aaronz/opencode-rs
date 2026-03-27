use clap::Args;

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
}

pub fn run(args: RunArgs) {
    println!("Running with prompt: {:?}", args.prompt);
}
