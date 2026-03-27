use clap::Args;

#[derive(Args, Debug)]
pub struct AttachArgs {
    #[arg(short, long)]
    pub session_id: Option<String>,
}

pub fn run(args: AttachArgs) {
    println!("Attaching to session {:?}", args.session_id);
}
