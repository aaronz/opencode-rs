use clap::Args;

#[derive(Args, Debug)]
pub struct StatsArgs {
    #[arg(short, long)]
    pub json: bool,
}

pub fn run(args: StatsArgs) {
    println!("Showing stats, json: {}", args.json);
}
