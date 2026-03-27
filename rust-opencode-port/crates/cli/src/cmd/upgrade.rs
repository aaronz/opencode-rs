use clap::Args;

#[derive(Args, Debug)]
pub struct UpgradeArgs {
    #[arg(short, long)]
    pub version: Option<String>,

    #[arg(short, long)]
    pub force: bool,
}

pub fn run(args: UpgradeArgs) {
    println!(
        "Upgrading opencode to version {:?}, force: {}",
        args.version, args.force
    );
}
