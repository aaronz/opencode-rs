use clap::{Args, Subcommand};

#[derive(Args, Debug)]
pub struct ModelsArgs {
    #[arg(short, long)]
    pub provider: Option<String>,

    #[arg(short, long)]
    pub json: bool,

    #[arg(short, long)]
    pub visibility: Option<String>,

    #[command(subcommand)]
    pub action: Option<ModelsAction>,
}

#[derive(Subcommand, Debug)]
pub enum ModelsAction {
    Visibility {
        #[arg(short, long)]
        hide: Option<String>,

        #[arg(short, long)]
        show: Option<String>,

        #[arg(long)]
        list_hidden: bool,
    },
}

pub fn run(args: ModelsArgs) {
    match args.action {
        Some(ModelsAction::Visibility {
            hide,
            show,
            list_hidden,
        }) => {
            if let Some(model_id) = hide {
                println!("Hiding model: {}", model_id);
            } else if let Some(model_id) = show {
                println!("Showing model: {}", model_id);
            } else if list_hidden {
                println!("Listing hidden models");
            } else {
                println!("Visibility action requires --hide, --show, or --list-hidden");
            }
        }
        None => {
            println!(
                "Listing models for provider: {:?}, visibility: {:?}",
                args.provider, args.visibility
            );
        }
    }
}
