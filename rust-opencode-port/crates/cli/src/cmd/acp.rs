use clap::Args;

#[derive(Args, Debug)]
pub struct AcpArgs {
    #[arg(short, long)]
    pub url: Option<String>,
}

pub fn run(args: AcpArgs) {
    println!("ACP protocol manager, url: {:?}", args.url);
}
