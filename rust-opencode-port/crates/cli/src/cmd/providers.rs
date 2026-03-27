use clap::Args;

#[derive(Args, Debug)]
pub struct ProvidersArgs {
    #[arg(short, long)]
    pub json: bool,
}

pub fn run(args: ProvidersArgs) {
    println!("Listing providers, json: {}", args.json);
}
