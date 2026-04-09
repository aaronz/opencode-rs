use clap::Args;

#[derive(Args, Debug)]
pub struct ImportArgs {
    #[arg(short, long)]
    pub input: String,

    #[arg(short, long)]
    pub format: Option<String>,
}

pub fn run(args: ImportArgs) {
    println!("Importing from {}, format: {:?}", args.input, args.format);
}
