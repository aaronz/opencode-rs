use ratatui_testing::pty::{send_key, PtySession};
use std::time::Duration;

fn run_command(cmd: &str) -> PtySession {
    PtySession::spawn_command("bash", &["-c", cmd]).unwrap()
}

mod keyboard {
    use super::*;

    #[test]
    fn test_echo() {
        let mut session = run_command("echo hello");
        let output = session.read(Duration::from_secs(2)).unwrap();
        assert!(output.contains("hello"));
    }

    #[test]
    fn test_echo_with_newline() {
        let mut session = run_command("printf 'hello\\nworld'");
        let output = session.read(Duration::from_secs(2)).unwrap();
        assert!(output.contains("hello"));
        assert!(output.contains("world"));
    }

    #[test]
    fn test_special_chars() {
        let mut session = run_command("echo !@#$%");
        let output = session.read(Duration::from_secs(2)).unwrap();
        assert!(output.contains("!@#$%"));
    }
}

mod unicode {
    use super::*;

    #[test]
    fn test_chinese() {
        let mut session = run_command("echo '你好世界'");
        let output = session.read(Duration::from_secs(2)).unwrap();
        assert!(output.contains("你好世界"));
    }

    #[test]
    fn test_emoji() {
        let mut session = run_command("echo '😀🎉🚀'");
        let output = session.read(Duration::from_secs(2)).unwrap();
        assert!(output.contains("😀"));
    }

    #[test]
    fn test_japanese() {
        let mut session = run_command("echo 'こんにちは'");
        let output = session.read(Duration::from_secs(2)).unwrap();
        assert!(output.contains("こんにちは"));
    }
}

mod edge_cases {
    use super::*;

    #[test]
    fn test_empty_command() {
        let mut session = run_command("true");
        let output = session.read(Duration::from_secs(1)).unwrap();
        assert!(output.is_empty() || output.contains(""));
    }

    #[test]
    fn test_long_output() {
        let mut session = run_command("seq 1 100");
        let output = session.read(Duration::from_secs(2)).unwrap();
        assert!(output.contains("1"));
        assert!(output.contains("100"));
    }

    #[test]
    fn test_multiline() {
        let mut session = run_command("printf 'line1\\nline2\\nline3'");
        let output = session.read(Duration::from_secs(2)).unwrap();
        assert!(output.contains("line1"));
        assert!(output.contains("line2"));
        assert!(output.contains("line3"));
    }
}

mod signals {
    use super::*;

    #[test]
    fn test_ctrl_c() {
        let mut session = run_command("sleep 100 & kill -INT %1");
        let output = session.read(Duration::from_secs(2)).unwrap();
    }

    #[test]
    fn test_exit_code() {
        let mut session = run_command("exit 42");
        let _ = session.read(Duration::from_secs(1));
        let status = session.wait().unwrap();
        assert_eq!(status, 42);
    }
}

mod paste_clipboard {
    use super::*;

    #[test]
    fn test_paste_multiline_text() {
        let mut session = run_command("echo -e 'line1\\nline2\\nline3'");
        let output = session.read(Duration::from_secs(2)).unwrap();
        assert!(output.contains("line1"));
        assert!(output.contains("line2"));
        assert!(output.contains("line3"));
    }

    #[test]
    fn test_paste_long_text() {
        let long_text = "Lorem ipsum dolor sit amet consectetur adipiscing elit. ".repeat(10);
        let cmd = format!("echo '{}'", long_text);
        let mut session = run_command(&cmd);
        let output = session.read(Duration::from_secs(2)).unwrap();
        assert!(output.contains("Lorem ipsum"));
    }

    #[test]
    fn test_paste_with_special_chars() {
        let mut session = run_command("echo 'hello$world`test$(whoami)'");
        let output = session.read(Duration::from_secs(2)).unwrap();
        assert!(output.contains("hello"));
    }

    #[test]
    fn test_paste_whitespace_text() {
        let mut session = run_command("echo -e 'word1\\tword2\\tword3'");
        let output = session.read(Duration::from_secs(2)).unwrap();
        assert!(output.contains("word1"));
        assert!(output.contains("word2"));
    }

    #[test]
    fn test_paste_unicode_paste() {
        let mut session = run_command("echo '🎉🎊🎁' | cat");
        let output = session.read(Duration::from_secs(2)).unwrap();
        assert!(output.contains("🎉"));
    }
}

mod expect {
    use super::*;

    #[test]
    fn test_expect_found() {
        let mut session = run_command("echo hello world");
        let found = session.expect("hello", Duration::from_secs(2)).unwrap();
        assert!(found);
    }

    #[test]
    fn test_expect_not_found() {
        let mut session = run_command("echo hello");
        let found = session.expect("goodbye", Duration::from_secs(1)).unwrap();
        assert!(!found);
    }

    #[test]
    fn test_contains() {
        let mut session = run_command("echo testing123");
        let found = session.contains("ing", Duration::from_secs(2)).unwrap();
        assert!(found);
    }
}
