use serde::Serialize;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
struct NdjEvent<'a> {
    event: &'a str,
    timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    args: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<&'a str>,
}

pub struct NdjsonSerializer<W: Write> {
    writer: W,
}

impl<W: Write> NdjsonSerializer<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    #[expect(
        clippy::expect_used,
        reason = "System time should always be after UNIX EPOCH on valid systems"
    )]
    fn timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time is before UNIX epoch")
            .as_millis() as u64
    }

    #[allow(dead_code)]
    pub fn write_chunk(&mut self, chunk: &str) -> io::Result<()> {
        let event = NdjEvent {
            event: "chunk",
            timestamp: Self::timestamp(),
            content: Some(chunk),
            role: None,
            model: None,
            tool: None,
            args: None,
            result: None,
            error: None,
        };
        serde_json::to_writer(&mut self.writer, &event)?;
        writeln!(self.writer)
    }

    #[allow(dead_code)]
    pub fn write_done(&mut self) -> io::Result<()> {
        let event = NdjEvent {
            event: "done",
            timestamp: Self::timestamp(),
            role: None,
            content: None,
            model: None,
            tool: None,
            args: None,
            result: None,
            error: None,
        };
        serde_json::to_writer(&mut self.writer, &event)?;
        writeln!(self.writer)
    }

    #[allow(dead_code)]
    pub fn write_error(&mut self, error: &str) -> io::Result<()> {
        let event = NdjEvent {
            event: "error",
            timestamp: Self::timestamp(),
            error: Some(error),
            role: None,
            content: None,
            model: None,
            tool: None,
            args: None,
            result: None,
        };
        serde_json::to_writer(&mut self.writer, &event)?;
        writeln!(self.writer)
    }

    pub fn write_message(&mut self, role: &str, content: &str) -> io::Result<()> {
        let event = NdjEvent {
            event: "message",
            timestamp: Self::timestamp(),
            role: Some(role),
            content: Some(content),
            model: None,
            tool: None,
            args: None,
            result: None,
            error: None,
        };
        serde_json::to_writer(&mut self.writer, &event)?;
        writeln!(self.writer)
    }

    pub fn write_start(&mut self, model: &str) -> io::Result<()> {
        let event = NdjEvent {
            event: "start",
            timestamp: Self::timestamp(),
            model: Some(model),
            role: None,
            content: None,
            tool: None,
            args: None,
            result: None,
            error: None,
        };
        serde_json::to_writer(&mut self.writer, &event)?;
        writeln!(self.writer)
    }

    #[allow(dead_code)]
    pub fn write_tool_call(&mut self, tool_name: &str, args: &str) -> io::Result<()> {
        let event = NdjEvent {
            event: "tool_call",
            timestamp: Self::timestamp(),
            tool: Some(tool_name),
            args: Some(args),
            role: None,
            content: None,
            model: None,
            result: None,
            error: None,
        };
        serde_json::to_writer(&mut self.writer, &event)?;
        writeln!(self.writer)
    }

    #[allow(dead_code)]
    pub fn write_tool_result(&mut self, tool_name: &str, result: &str) -> io::Result<()> {
        let event = NdjEvent {
            event: "tool_result",
            timestamp: Self::timestamp(),
            tool: Some(tool_name),
            result: Some(result),
            role: None,
            content: None,
            model: None,
            args: None,
            error: None,
        };
        serde_json::to_writer(&mut self.writer, &event)?;
        writeln!(self.writer)
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl NdjsonSerializer<std::io::Stdout> {
    pub fn stdout() -> Self {
        Self::new(std::io::stdout())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ndjson_serializer_write_start() {
        let mut buffer = Vec::new();
        let mut serializer = NdjsonSerializer::new(&mut buffer);

        serializer.write_start("gpt-4").unwrap();
        serializer.flush().unwrap();

        let output = String::from_utf8(buffer).unwrap();
        let line = output.lines().next().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(line).unwrap();

        assert_eq!(parsed["event"], "start");
        assert_eq!(parsed["model"], "gpt-4");
        assert!(parsed["timestamp"].as_u64().is_some());
    }

    #[test]
    fn test_ndjson_serializer_write_message() {
        let mut buffer = Vec::new();
        let mut serializer = NdjsonSerializer::new(&mut buffer);

        serializer
            .write_message("assistant", "Hello world")
            .unwrap();
        serializer.flush().unwrap();

        let output = String::from_utf8(buffer).unwrap();
        let line = output.lines().next().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(line).unwrap();

        assert_eq!(parsed["event"], "message");
        assert_eq!(parsed["role"], "assistant");
        assert_eq!(parsed["content"], "Hello world");
    }

    #[test]
    fn test_ndjson_serializer_write_chunk() {
        let mut buffer = Vec::new();
        let mut serializer = NdjsonSerializer::new(&mut buffer);

        serializer.write_chunk("partial content").unwrap();
        serializer.flush().unwrap();

        let output = String::from_utf8(buffer).unwrap();
        let line = output.lines().next().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(line).unwrap();

        assert_eq!(parsed["event"], "chunk");
        assert_eq!(parsed["content"], "partial content");
    }

    #[test]
    fn test_ndjson_serializer_write_done() {
        let mut buffer = Vec::new();
        let mut serializer = NdjsonSerializer::new(&mut buffer);

        serializer.write_done().unwrap();
        serializer.flush().unwrap();

        let output = String::from_utf8(buffer).unwrap();
        let line = output.lines().next().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(line).unwrap();

        assert_eq!(parsed["event"], "done");
        assert!(parsed["content"].is_null());
    }

    #[test]
    fn test_ndjson_serializer_write_error() {
        let mut buffer = Vec::new();
        let mut serializer = NdjsonSerializer::new(&mut buffer);

        serializer.write_error("Something went wrong").unwrap();
        serializer.flush().unwrap();

        let output = String::from_utf8(buffer).unwrap();
        let line = output.lines().next().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(line).unwrap();

        assert_eq!(parsed["event"], "error");
        assert_eq!(parsed["error"], "Something went wrong");
    }

    #[test]
    fn test_ndjson_serializer_multiple_events() {
        let mut buffer = Vec::new();
        let mut serializer = NdjsonSerializer::new(&mut buffer);

        serializer.write_start("gpt-4").unwrap();
        serializer.write_message("assistant", "Hi").unwrap();
        serializer.write_chunk("more").unwrap();
        serializer.write_done().unwrap();
        serializer.flush().unwrap();

        let output = String::from_utf8(buffer).unwrap();
        let lines: Vec<&str> = output.lines().collect();

        assert_eq!(lines.len(), 4);
        assert!(lines
            .iter()
            .all(|l| serde_json::from_str::<serde_json::Value>(l).is_ok()));
    }

    #[test]
    fn test_ndjson_serializer_tool_call() {
        let mut buffer = Vec::new();
        let mut serializer = NdjsonSerializer::new(&mut buffer);

        serializer
            .write_tool_call("bash", r#"{"command": "ls"}"#)
            .unwrap();
        serializer.flush().unwrap();

        let output = String::from_utf8(buffer).unwrap();
        let line = output.lines().next().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(line).unwrap();

        assert_eq!(parsed["event"], "tool_call");
        assert_eq!(parsed["tool"], "bash");
        assert_eq!(parsed["args"], r#"{"command": "ls"}"#);
    }

    #[test]
    fn test_ndjson_serializer_tool_result() {
        let mut buffer = Vec::new();
        let mut serializer = NdjsonSerializer::new(&mut buffer);

        serializer
            .write_tool_result("bash", "file1.txt\nfile2.txt")
            .unwrap();
        serializer.flush().unwrap();

        let output = String::from_utf8(buffer).unwrap();
        let line = output.lines().next().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(line).unwrap();

        assert_eq!(parsed["event"], "tool_result");
        assert_eq!(parsed["tool"], "bash");
        assert_eq!(parsed["result"], "file1.txt\nfile2.txt");
    }

    #[test]
    fn test_ndjson_each_event_on_separate_line() {
        let mut buffer = Vec::new();
        let mut serializer = NdjsonSerializer::new(&mut buffer);

        serializer.write_start("model").unwrap();
        serializer.write_message("user", "test").unwrap();
        serializer.write_done().unwrap();
        serializer.flush().unwrap();

        let output = String::from_utf8(buffer).unwrap();
        let lines: Vec<&str> = output.lines().collect();

        assert_eq!(lines.len(), 3);
        for line in &lines {
            assert!(serde_json::from_str::<serde_json::Value>(line).is_ok());
        }
    }

    #[test]
    fn test_ndjson_valid_json_per_line() {
        let mut buffer = Vec::new();
        let mut serializer = NdjsonSerializer::new(&mut buffer);

        let events = vec![
            ("start", Some("model")),
            ("message", Some("content")),
            ("chunk", Some("data")),
            ("done", None),
            ("error", Some("err")),
        ];

        for (event_type, content) in events {
            match event_type {
                "start" => serializer.write_start(content.unwrap()).unwrap(),
                "message" => serializer.write_message("role", content.unwrap()).unwrap(),
                "chunk" => serializer.write_chunk(content.unwrap()).unwrap(),
                "done" => serializer.write_done().unwrap(),
                "error" => serializer.write_error(content.unwrap()).unwrap(),
                _ => unreachable!(),
            }
        }
        serializer.flush().unwrap();

        let output = String::from_utf8(buffer).unwrap();
        for line in output.lines() {
            let parsed = serde_json::from_str::<serde_json::Value>(line);
            assert!(parsed.is_ok(), "Failed to parse: {}", line);
        }
    }
}
