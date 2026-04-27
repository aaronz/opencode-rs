use std::io::{Read, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

static PORT_COUNTER: AtomicUsize = AtomicUsize::new(9000);

fn next_port() -> u16 {
    let port = PORT_COUNTER.fetch_add(1, Ordering::SeqCst) as u16;
    if port > 60000 {
        PORT_COUNTER.store(9000, Ordering::SeqCst);
    }
    port
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ReceivedRequest {
    pub method: String,
    pub path: String,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct MockResponse {
    pub status: u16,
    pub body: String,
    pub content_type: String,
}

struct MockServerInner {
    port: u16,
    running: Arc<Mutex<bool>>,
    received_requests: Arc<Mutex<Vec<ReceivedRequest>>>,
    responses: Arc<Mutex<Vec<(String, String, MockResponse)>>>,
}

pub struct MockServer {
    inner: Arc<MockServerInner>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockServer {
    pub fn start() -> Self {
        let port = next_port();
        let running = Arc::new(Mutex::new(true));
        let received = Arc::new(Mutex::new(Vec::new()));
        let responses = Arc::new(Mutex::new(Vec::new()));

        let inner = Arc::new(MockServerInner {
            port,
            running: running.clone(),
            received_requests: received.clone(),
            responses: responses.clone(),
        });

        let listener = std::net::TcpListener::bind(("127.0.0.1", port)).unwrap_or_else(|_| {
            let fallback = next_port();
            std::net::TcpListener::bind(("127.0.0.1", fallback))
                .expect("Failed to bind mock server")
        });

        let inner_for_thread = inner.clone();
        let handle = thread::spawn(move || {
            listener.set_nonblocking(true).ok();
            loop {
                if let Ok(running) = inner_for_thread.running.lock() {
                    if !*running {
                        break;
                    }
                }

                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let received = inner_for_thread.received_requests.clone();
                        let responses = inner_for_thread.responses.clone();

                        let mut buf = [0u8; 4096];
                        if stream
                            .set_read_timeout(Some(Duration::from_secs(1)))
                            .is_err()
                        {
                            continue;
                        }
                        let n = match stream.read(&mut buf) {
                            Ok(n) => n,
                            Err(_) => continue,
                        };

                        let request_str = String::from_utf8_lossy(&buf[..n]).to_string();

                        let (method, path) = Self::parse_request_line(&request_str);
                        let body = Self::parse_body(&request_str);
                        let method_for_response = method.clone();
                        let path_for_response = path.clone();

                        if let Ok(mut reqs) = received.lock() {
                            reqs.push(ReceivedRequest { method, path, body });
                        }

                        let response = {
                            let guard = responses.lock().unwrap();
                            guard
                                .iter()
                                .find(|(m, p, _)| {
                                    (m == "*" || m == &method_for_response)
                                        && (p == "*" || p == &path_for_response)
                                })
                                .map(|(_, _, r)| r.clone())
                                .unwrap_or_else(|| MockResponse {
                                    status: 200,
                                    body: "{}".to_string(),
                                    content_type: "application/json".to_string(),
                                })
                        };

                        let http_response = format!(
                            "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            response.status,
                            response.content_type,
                            response.body.len(),
                            response.body
                        );
                        stream.write_all(http_response.as_bytes()).ok();
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => {
                        thread::sleep(Duration::from_millis(10));
                    }
                }
            }
        });

        Self {
            inner,
            handle: Some(handle),
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{}", self.inner.port, path)
    }

    pub fn port(&self) -> u16 {
        self.inner.port
    }

    pub fn mock(&self, method: &str, path: &str, status: u16, body: &str) {
        let mut responses = self.inner.responses.lock().unwrap();
        responses.push((
            method.to_string(),
            path.to_string(),
            MockResponse {
                status,
                body: body.to_string(),
                content_type: "application/json".to_string(),
            },
        ));
    }

    #[allow(dead_code)]
    pub fn received_requests(&self) -> Vec<ReceivedRequest> {
        self.inner
            .received_requests
            .lock()
            .map(|r| r.clone())
            .unwrap_or_default()
    }

    #[allow(dead_code)]
    pub fn request_count(&self) -> usize {
        self.inner
            .received_requests
            .lock()
            .map(|r| r.len())
            .unwrap_or(0)
    }

    pub fn stop(&mut self) {
        if let Ok(mut running) = self.inner.running.lock() {
            *running = false;
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    fn parse_request_line(request: &str) -> (String, String) {
        let first_line = request.lines().next().unwrap_or("");
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() >= 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("GET".to_string(), "/".to_string())
        }
    }

    fn parse_body(request: &str) -> String {
        if let Some(pos) = request.find("\r\n\r\n") {
            request[pos + 4..].to_string()
        } else if let Some(pos) = request.find("\n\n") {
            request[pos + 2..].to_string()
        } else {
            String::new()
        }
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpStream;

    #[test]
    fn test_mock_server_start_and_url() {
        let server = MockServer::start();
        let url = server.url("/api/test");
        assert!(url.contains("127.0.0.1"));
        assert!(url.contains("/api/test"));
    }

    #[test]
    #[ignore]
    fn test_mock_server_mock_response() {
        let server = MockServer::start();
        server.mock("POST", "/chat", 200, r#"{"content":"hello"}"#);

        let mut stream = TcpStream::connect(("127.0.0.1", server.port())).unwrap();
        stream
            .write_all(b"POST /chat HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .unwrap();
        let mut buf = [0u8; 1024];
        let n = stream.read(&mut buf).unwrap();
        let response = String::from_utf8_lossy(&buf[..n]);
        assert!(response.contains("HTTP/1.1 200"));
        assert!(response.contains("hello"));
    }

    #[test]
    fn test_mock_server_stop() {
        let mut server = MockServer::start();
        server.stop();
        assert!(server.handle.is_none());
    }
}
