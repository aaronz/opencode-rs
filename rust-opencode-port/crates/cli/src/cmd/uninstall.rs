use clap::Args;

#[derive(Args, Debug)]
pub struct UninstallArgs {
    #[arg(short, long)]
    pub force: bool,
}

pub fn run(args: UninstallArgs) {
    println!("Uninstalling opencode, force: {}", args.force);
}
