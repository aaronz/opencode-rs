use clap::Args;

#[derive(Args, Debug)]
pub struct ModelsArgs {
    #[arg(short, long)]
    pub provider: Option<String>,

    #[arg(short, long)]
    pub json: bool,
}

pub fn run(args: ModelsArgs) {
    println!("Listing models for provider: {:?}", args.provider);
}
