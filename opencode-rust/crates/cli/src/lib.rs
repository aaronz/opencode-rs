#[allow(dead_code)]
pub(crate) mod cmd;
#[allow(dead_code)]
pub(crate) mod output;
#[allow(dead_code)]
pub(crate) mod webview;

pub fn finalize_tui_run_result<F>(
    run_result: std::io::Result<()>,
    restore_terminal: F,
) -> Result<(), String>
where
    F: FnOnce() -> std::io::Result<()>,
{
    let restore_result = restore_terminal();

    match run_result {
        Ok(()) => restore_result.map_err(|restore_error| {
            format!("Terminal restore failed after TUI exit: {}", restore_error)
        }),
        Err(error) => match restore_result {
            Ok(()) => Err(format!("Error running TUI: {}", error)),
            Err(restore_error) => Err(format!(
                "Error running TUI: {} (terminal restore failed: {})",
                error, restore_error
            )),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::finalize_tui_run_result;
    use super::output::NdjsonSerializer;

    #[test]
    fn test_ndjson_serializer_write_message() {
        let mut buffer = Vec::new();
        {
            let mut serializer = NdjsonSerializer::new(&mut buffer);
            serializer.write_message("user", "Hello").unwrap();
        }
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"event\":\"message\""));
        assert!(output.contains("\"role\":\"user\""));
        assert!(output.contains("\"content\":\"Hello\""));
    }

    #[test]
    fn test_ndjson_serializer_write_start() {
        let mut buffer = Vec::new();
        {
            let mut serializer = NdjsonSerializer::new(&mut buffer);
            serializer.write_start("gpt-4").unwrap();
        }
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"event\":\"start\""));
        assert!(output.contains("\"model\":\"gpt-4\""));
    }

    #[test]
    fn test_ndjson_serializer_write_error() {
        let mut buffer = Vec::new();
        {
            let mut serializer = NdjsonSerializer::new(&mut buffer);
            serializer.write_error("something went wrong").unwrap();
        }
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"event\":\"error\""));
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["event"], "error");
        assert_eq!(parsed["error"], "something went wrong");
    }

    #[test]
    fn test_ndjson_serializer_write_chunk() {
        let mut buffer = Vec::new();
        {
            let mut serializer = NdjsonSerializer::new(&mut buffer);
            serializer.write_chunk("partial content").unwrap();
        }
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"event\":\"chunk\""));
        assert!(output.contains("\"content\":\"partial content\""));
    }

    #[test]
    fn test_ndjson_serializer_write_done() {
        let mut buffer = Vec::new();
        {
            let mut serializer = NdjsonSerializer::new(&mut buffer);
            serializer.write_done().unwrap();
        }
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"event\":\"done\""));
    }

    #[test]
    fn test_ndjson_serializer_write_tool_call() {
        let mut buffer = Vec::new();
        {
            let mut serializer = NdjsonSerializer::new(&mut buffer);
            serializer
                .write_tool_call("read", r#"{"path": "foo.txt"}"#)
                .unwrap();
        }
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"event\":\"tool_call\""));
        assert!(output.contains("\"tool\":\"read\""));
        assert!(output.contains("\"args\""));
    }

    #[test]
    fn test_ndjson_serializer_write_tool_result() {
        let mut buffer = Vec::new();
        {
            let mut serializer = NdjsonSerializer::new(&mut buffer);
            serializer
                .write_tool_result("read", "file contents here")
                .unwrap();
        }
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("\"event\":\"tool_result\""));
        assert!(output.contains("\"tool\":\"read\""));
        assert!(output.contains("\"result\":\"file contents here\""));
    }

    #[test]
    fn test_ndjson_serializer_flush() {
        let buffer = Vec::new();
        let mut serializer = NdjsonSerializer::new(buffer);
        serializer.flush().unwrap();
    }

    #[test]
    fn test_finalize_tui_run_result_returns_ok_on_success() {
        let restore_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let restore_called_clone = restore_called.clone();
        let result = finalize_tui_run_result(Ok(()), move || {
            restore_called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        });
        assert!(result.is_ok());
        assert!(
            restore_called.load(std::sync::atomic::Ordering::SeqCst),
            "terminal restore should still run after a clean TUI exit"
        );
    }

    #[test]
    fn test_finalize_tui_run_result_formats_run_error_after_cleanup() {
        let result = finalize_tui_run_result(Err(std::io::Error::other("draw failed")), || Ok(()));

        assert_eq!(
            result.unwrap_err(),
            "Error running TUI: draw failed",
            "run errors should be returned after terminal recovery"
        );
    }

    #[test]
    fn test_finalize_tui_run_result_reports_cleanup_failure() {
        let result = finalize_tui_run_result(Err(std::io::Error::other("draw failed")), || {
            Err(std::io::Error::other("restore failed"))
        });

        assert_eq!(
            result.unwrap_err(),
            "Error running TUI: draw failed (terminal restore failed: restore failed)",
            "cleanup failure should be included in the returned message"
        );
    }
}
