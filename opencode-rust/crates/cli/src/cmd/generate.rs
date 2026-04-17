use clap::Args;

#[derive(Args, Debug)]
pub struct GenerateArgs {
    #[arg(short, long)]
    pub template: Option<String>,

    #[arg(short, long)]
    pub output: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_args_default() {
        let args = GenerateArgs {
            template: None,
            output: None,
        };
        assert!(args.template.is_none());
        assert!(args.output.is_none());
    }

    #[test]
    fn test_generate_args_with_template() {
        let args = GenerateArgs {
            template: Some("rust-crate".to_string()),
            output: None,
        };
        assert_eq!(args.template.as_deref(), Some("rust-crate"));
    }

    #[test]
    fn test_generate_args_with_output() {
        let args = GenerateArgs {
            template: None,
            output: Some("my-project".to_string()),
        };
        assert_eq!(args.output.as_deref(), Some("my-project"));
    }

    #[test]
    fn test_generate_args_full() {
        let args = GenerateArgs {
            template: Some("react-component".to_string()),
            output: Some("components/Button.tsx".to_string()),
        };
        assert_eq!(args.template.as_deref(), Some("react-component"));
        assert_eq!(args.output.as_deref(), Some("components/Button.tsx"));
    }
}

pub fn run(args: GenerateArgs) {
    println!(
        "Generating with template {:?}, output: {:?}",
        args.template, args.output
    );
}
