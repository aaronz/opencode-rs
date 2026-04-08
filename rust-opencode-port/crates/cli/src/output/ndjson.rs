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

    fn timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

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
