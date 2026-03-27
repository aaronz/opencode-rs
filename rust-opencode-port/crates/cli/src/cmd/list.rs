use clap::Args;

#[derive(Args, Debug)]
pub struct ListArgs {
    #[arg(short, long)]
    pub all: bool,

    #[arg(short, long)]
    pub limit: Option<u32>,

    #[arg(short, long)]
    pub json: bool,
}

pub fn run(args: ListArgs) {
    println!(
        "Listing sessions, all: {}, limit: {:?}",
        args.all, args.limit
    );
}
