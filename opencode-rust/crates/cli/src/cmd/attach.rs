use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub(crate) struct AttachArgs {
    #[arg(short, long)]
    pub session_id: Option<String>,

    #[arg(short, long = "dir")]
    pub directory: Option<PathBuf>,

    pub url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attach_args_default() {
        let args = AttachArgs {
            session_id: None,
            directory: None,
            url: None,
        };
        assert!(args.session_id.is_none());
        assert!(args.directory.is_none());
        assert!(args.url.is_none());
    }

    #[test]
    fn test_attach_args_with_session_id() {
        let args = AttachArgs {
            session_id: Some("session-123".to_string()),
            directory: None,
            url: None,
        };
        assert_eq!(args.session_id.as_deref(), Some("session-123"));
    }

    #[test]
    fn test_attach_args_with_url() {
        let args = AttachArgs {
            session_id: None,
            directory: None,
            url: Some("wss://example.com".to_string()),
        };
        assert_eq!(args.url.as_deref(), Some("wss://example.com"));
    }

    #[test]
    fn test_attach_args_with_directory() {
        let args = AttachArgs {
            session_id: None,
            directory: Some(PathBuf::from("/tmp")),
            url: None,
        };
        assert_eq!(
            args.directory.as_ref().map(|p| p.as_os_str()),
            Some(std::ffi::OsStr::new("/tmp"))
        );
    }

    #[test]
    fn test_attach_args_full() {
        let args = AttachArgs {
            session_id: Some("session-456".to_string()),
            directory: Some(PathBuf::from("/home/user/project")),
            url: Some("wss://example.com/session".to_string()),
        };
        assert_eq!(args.session_id.as_deref(), Some("session-456"));
        assert_eq!(
            args.directory
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
            Some("/home/user/project".to_string())
        );
        assert_eq!(args.url.as_deref(), Some("wss://example.com/session"));
    }
}

pub(crate) fn run(args: AttachArgs) {
    if let Some(url) = args.url {
        println!("Attaching to URL: {}", url);
    } else if let Some(session_id) = args.session_id {
        println!("Attaching to session: {}", session_id);
    } else {
        println!("Attaching to local session");
    }
}
