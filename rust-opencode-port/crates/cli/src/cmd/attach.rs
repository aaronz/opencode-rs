use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct AttachArgs {
    #[arg(short, long)]
    pub session_id: Option<String>,

    #[arg(short, long = "dir")]
    pub directory: Option<PathBuf>,

    pub url: Option<String>,
}

pub fn run(args: AttachArgs) {
    if let Some(url) = args.url {
        println!("Attaching to URL: {}", url);
    } else if let Some(session_id) = args.session_id {
        println!("Attaching to session: {}", session_id);
    } else {
        println!("Attaching to local session");
    }
}
