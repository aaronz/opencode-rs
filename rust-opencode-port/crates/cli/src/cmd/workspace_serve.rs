use clap::Args;

#[derive(Args, Debug)]
pub struct WorkspaceServeArgs {
    #[arg(short, long)]
    pub port: Option<u16>,
}

pub fn run(args: WorkspaceServeArgs) {
    println!("Starting workspace server on port {:?}", args.port);
}
