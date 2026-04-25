#[allow(dead_code)]
pub(crate) mod cmd;
#[allow(dead_code)]
pub(crate) mod output;
#[allow(dead_code)]
pub(crate) mod webview;

#[cfg(test)]
mod tests {
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
}
