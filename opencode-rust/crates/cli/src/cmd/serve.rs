use clap::Args;

#[derive(Args, Debug)]
pub struct ServeArgs {
    #[arg(short, long)]
    pub port: Option<u16>,

    #[arg(short, long)]
    pub hostname: Option<String>,
}

pub fn run(args: ServeArgs) {
    println!("Starting server on {:?}:{:?}", args.hostname, args.port);
}
