use clap::Args;

#[derive(Args, Debug)]
pub(crate) struct WorkspaceServeArgs {
    #[arg(short, long)]
    pub port: Option<u16>,
}

#[allow(clippy::items_after_test_module)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_serve_args_default() {
        let args = WorkspaceServeArgs { port: None };
        assert!(args.port.is_none());
    }

    #[test]
    fn test_workspace_serve_args_with_port() {
        let args = WorkspaceServeArgs { port: Some(8080) };
        assert_eq!(args.port, Some(8080));
    }

    #[test]
    fn test_workspace_serve_args_with_different_port() {
        let args = WorkspaceServeArgs { port: Some(3000) };
        assert_eq!(args.port, Some(3000));
    }
}

pub(crate) fn run(args: WorkspaceServeArgs) {
    println!("Starting workspace server on port {:?}", args.port);
}
