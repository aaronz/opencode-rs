use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

pub struct MockLspServer {
    child: Child,
    running: Arc<AtomicBool>,
}

impl MockLspServer {
    pub fn start() -> Result<Self, std::io::Error> {
        Self::start_with_script(SIMPLE_LSP_MOCK_SCRIPT)
    }

    pub fn start_with_script(script: &str) -> Result<Self, std::io::Error> {
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let stdin = child.stdin.take();
        let stdout = child.stdout.take();

        thread::spawn(move || {
            if let (Some(mut stdin), Some(stdout)) = (stdin, stdout) {
                let mut reader = BufReader::new(stdout);
                let mut current_header = String::new();

                loop {
                    if !running_clone.load(Ordering::SeqCst) {
                        break;
                    }

                    current_header.clear();
                    let mut found_end = false;

                    while !found_end {
                        let mut line = String::new();
                        match reader.read_line(&mut line) {
                            Ok(0) => break,
                            Ok(_) => {
                                if line == "\r\n" || line == "\n" {
                                    found_end = true;
                                } else {
                                    current_header.push_str(&line);
                                }
                            }
                            Err(_) => break,
                        }
                    }

                    if !found_end {
                        break;
                    }

                    let content_length = current_header
                        .lines()
                        .find(|l| l.starts_with("Content-Length: "))
                        .and_then(|l| l.strip_prefix("Content-Length: "))
                        .and_then(|v| v.trim().parse::<usize>().ok());

                    let Some(len) = content_length else {
                        break;
                    };

                    let mut body = vec![0u8; len];
                    let mut pos = 0;
                    while pos < len {
                        match reader.read(&mut body[pos..]) {
                            Ok(0) => break,
                            Ok(n) => pos += n,
                            Err(_) => break,
                        }
                    }

                    let request = String::from_utf8_lossy(&body).to_string();
                    let response = handle_jsonrpc_request(&request);

                    if let Some(resp) = response {
                        let formatted = format_response(&resp);
                        if stdin.write_all(formatted.as_bytes()).is_err() {
                            break;
                        }
                        if stdin.flush().is_err() {
                            break;
                        }
                    }
                }
            }
        });

        Ok(Self { child, running })
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        let _ = self.child.kill();
    }

    pub fn pid(&self) -> u32 {
        self.child.id()
    }
}

impl Drop for MockLspServer {
    fn drop(&mut self) {
        self.stop();
    }
}

const SIMPLE_LSP_MOCK_SCRIPT: &str = r#"
while IFS= read -r line; do
    if [[ "$line" == $'\r' ]] || [[ "$line" == $'\n' ]] || [[ -z "$line" ]]; then
        break
    fi
    if [[ "$line" == Content-Length:\ * ]]; then
        LEN="${line#Content-Length: }"
        LEN="${LEN%$'\r'}"
        LEN="${LEN%$'\n'}"
    fi
done

if [[ -n "$LEN" ]]; then
    read -n "$LEN" BODY
    if [[ "$BODY" == *"initialize"* ]]; then
        RESPONSE='{"jsonrpc":"2.0","id":1,"result":{"capabilities":{"definitionProvider":true,"referencesProvider":true}}}'
        echo -n "Content-Length: ${#RESPONSE}"$'\r\n\r\n'"$RESPONSE"
    elif [[ "$BODY" == *"textDocument/definition"* ]]; then
        RESPONSE='{"jsonrpc":"2.0","id":2,"result":{"uri":"file:///target.rs","range":{"start":{"line":10,"character":5},"end":{"line":10,"character":10}}}}'
        echo -n "Content-Length: ${#RESPONSE}"$'\r\n\r\n'"$RESPONSE"
    elif [[ "$BODY" == *"textDocument/references"* ]]; then
        RESPONSE='{"jsonrpc":"2.0","id":3,"result":[{"uri":"file:///ref1.rs","range":{"start":{"line":5,"character":3},"end":{"line":5,"character":8}}},{"uri":"file:///ref2.rs","range":{"start":{"line":10,"character":7},"end":{"line":10,"character":12}}}]}'
        echo -n "Content-Length: ${#RESPONSE}"$'\r\n\r\n'"$RESPONSE"
    elif [[ "$BODY" == *"shutdown"* ]]; then
        RESPONSE='{"jsonrpc":"2.0","id":99,"result":null}'
        echo -n "Content-Length: ${#RESPONSE}"$'\r\n\r\n'"$RESPONSE"
        exit 0
    fi
fi
"#;

fn handle_jsonrpc_request(msg: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(msg).ok()?;
    let method = value.get("method")?.as_str()?;
    let id = value.get("id");

    match method {
        "initialize" => {
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "capabilities": {
                        "definitionProvider": true,
                        "referencesProvider": true,
                        "hoverProvider": true,
                    }
                }
            });
            Some(response.to_string())
        }
        "initialized" => None,
        "shutdown" => {
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": null
            });
            Some(response.to_string())
        }
        "textDocument/definition" => {
            let params = value.get("params")?;
            let position = params.get("position")?;
            let line = position.get("line")?.as_u64().unwrap_or(0) as u32;
            let character = position.get("character")?.as_u64().unwrap_or(0) as u32;

            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "uri": "file:///mock_target.rs",
                    "range": {
                        "start": { "line": line + 10, "character": character + 5 },
                        "end": { "line": line + 10, "character": character + 10 }
                    }
                }
            });
            Some(response.to_string())
        }
        "textDocument/references" => {
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": [
                    {
                        "uri": "file:///mock_ref1.rs",
                        "range": {
                            "start": { "line": 5, "character": 3 },
                            "end": { "line": 5, "character": 8 }
                        }
                    },
                    {
                        "uri": "file:///mock_ref2.rs",
                        "range": {
                            "start": { "line": 10, "character": 7 },
                            "end": { "line": 10, "character": 12 }
                        }
                    }
                ]
            });
            Some(response.to_string())
        }
        "textDocument/hover" => {
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "contents": "Mock hover info"
                }
            });
            Some(response.to_string())
        }
        "window/showMessageRequest" => {
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": null
            });
            Some(response.to_string())
        }
        _ => None,
    }
}

fn format_response(response: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{}", response.len(), response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_jsonrpc_initialize() {
        let msg = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let response = handle_jsonrpc_request(msg);
        assert!(response.is_some());
        let resp: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert_eq!(resp.get("id").unwrap(), 1);
        assert!(resp.get("result").is_some());
    }

    #[test]
    fn test_handle_jsonrpc_definition() {
        let msg = r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/definition","params":{"textDocument":{"uri":"file:///test.rs"},"position":{"line":10,"character":5}}}"#;
        let response = handle_jsonrpc_request(msg);
        assert!(response.is_some());
        let resp: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert_eq!(resp.get("id").unwrap(), 2);
        assert!(resp.get("result").is_some());
    }

    #[test]
    fn test_handle_jsonrpc_references() {
        let msg = r#"{"jsonrpc":"2.0","id":3,"method":"textDocument/references","params":{"textDocument":{"uri":"file:///test.rs"},"position":{"line":10,"character":5}}}"#;
        let response = handle_jsonrpc_request(msg);
        assert!(response.is_some());
        let resp: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert_eq!(resp.get("id").unwrap(), 3);
        let result = resp.get("result").unwrap();
        assert!(result.is_array());
        assert_eq!(result.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_handle_jsonrpc_unknown_method() {
        let msg = r#"{"jsonrpc":"2.0","id":4,"method":"unknown/method","params":{}}"#;
        let response = handle_jsonrpc_request(msg);
        assert!(response.is_none());
    }

    #[test]
    fn test_format_response() {
        let response = r#"{"jsonrpc":"2.0","id":1,"result":null}"#;
        let formatted = format_response(response);
        assert!(formatted.starts_with("Content-Length: "));
        assert!(formatted.contains("\r\n\r\n"));
        assert!(formatted.contains(response));
    }

    #[test]
    fn test_mock_server_start_stop() {
        let mut server = MockLspServer::start().expect("Failed to start mock server");
        let pid = server.pid();
        assert!(pid > 0);
        server.stop();
    }
}
