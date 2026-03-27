use clap::Args;

#[derive(Args, Debug)]
pub struct ThreadArgs {
    #[arg(short, long)]
    pub session_id: Option<String>,
}

pub fn run(args: ThreadArgs) {
    println!("Starting TUI thread mode for session {:?}", args.session_id);
}
