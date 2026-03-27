use clap::Args;

#[derive(Args, Debug)]
pub struct ExportArgs {
    #[arg(short, long)]
    pub output: Option<String>,

    #[arg(short, long)]
    pub format: Option<String>,
}

pub fn run(args: ExportArgs) {
    println!("Exporting to {:?}, format: {:?}", args.output, args.format);
}
