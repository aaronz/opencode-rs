use clap::Args;

#[derive(Args, Debug)]
pub struct GenerateArgs {
    #[arg(short, long)]
    pub template: Option<String>,

    #[arg(short, long)]
    pub output: Option<String>,
}

pub fn run(args: GenerateArgs) {
    println!(
        "Generating with template {:?}, output: {:?}",
        args.template, args.output
    );
}
