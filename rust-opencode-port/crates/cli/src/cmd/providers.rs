use clap::Args;
use serde_json::json;

#[derive(Args, Debug)]
pub struct ProvidersArgs {
    #[arg(short, long)]
    pub json: bool,
}

pub fn run(args: ProvidersArgs) {
    if args.json {
        let result = json!({
            "action": "list",
            "providers": [
                { "id": "openai", "name": "OpenAI" },
                { "id": "anthropic", "name": "Anthropic" },
                { "id": "ollama", "name": "Ollama" }
            ]
        });
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
        return;
    }

    println!("Listing providers, json: {}", args.json);
}
