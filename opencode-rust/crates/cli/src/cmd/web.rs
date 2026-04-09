use clap::Args;

#[derive(Args, Debug)]
pub struct WebArgs {
    #[arg(short, long)]
    pub port: Option<u16>,
}

pub fn run(args: WebArgs) {
    println!("Starting web interface on port {:?}", args.port);
}
