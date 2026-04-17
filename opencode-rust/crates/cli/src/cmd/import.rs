use clap::Args;

#[derive(Args, Debug)]
pub(crate) struct ImportArgs {
    #[arg(short, long)]
    pub input: String,

    #[arg(short, long)]
    pub format: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_args_with_input() {
        let args = ImportArgs {
            input: "import.json".to_string(),
            format: None,
        };
        assert_eq!(args.input, "import.json");
        assert!(args.format.is_none());
    }

    #[test]
    fn test_import_args_with_format() {
        let args = ImportArgs {
            input: "data.json".to_string(),
            format: Some("json".to_string()),
        };
        assert_eq!(args.input, "data.json");
        assert_eq!(args.format.as_deref(), Some("json"));
    }

    #[test]
    fn test_import_args_full() {
        let args = ImportArgs {
            input: "/path/to/import.json".to_string(),
            format: Some("json".to_string()),
        };
        assert_eq!(args.input, "/path/to/import.json");
        assert_eq!(args.format.as_deref(), Some("json"));
    }
}

pub(crate) fn run(args: ImportArgs) {
    println!("Importing from {}, format: {:?}", args.input, args.format);
}
