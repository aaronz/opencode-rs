use clap::Args;

#[derive(Args, Debug)]
pub(crate) struct ExportArgs {
    #[arg(short, long)]
    pub output: Option<String>,

    #[arg(short, long)]
    pub format: Option<String>,
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_args_default() {
        let args = ExportArgs {
            output: None,
            format: None,
        };
        assert!(args.output.is_none());
        assert!(args.format.is_none());
    }

    #[test]
    fn test_export_args_with_output() {
        let args = ExportArgs {
            output: Some("export.json".to_string()),
            format: None,
        };
        assert_eq!(args.output.as_deref(), Some("export.json"));
    }

    #[test]
    fn test_export_args_with_format() {
        let args = ExportArgs {
            output: None,
            format: Some("json".to_string()),
        };
        assert_eq!(args.format.as_deref(), Some("json"));
    }

    #[test]
    fn test_export_args_full() {
        let args = ExportArgs {
            output: Some("sessions.json".to_string()),
            format: Some("json".to_string()),
        };
        assert_eq!(args.output.as_deref(), Some("sessions.json"));
        assert_eq!(args.format.as_deref(), Some("json"));
    }
}

pub(crate) fn run(args: ExportArgs) {
    println!("Exporting to {:?}, format: {:?}", args.output, args.format);
}
