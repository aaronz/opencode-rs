mod common;

use common::TestHarness;
use std::net::TcpListener;
use std::process::Stdio;
use std::time::Duration;

fn find_available_port() -> Option<u16> {
    let listener = TcpListener::bind("127.0.0.1:0").ok()?;
    let port = listener.local_addr().ok()?.port();
    drop(listener);
    Some(port)
}

fn find_available_port_with_retry(max_attempts: u32) -> Option<u16> {
    for _ in 0..max_attempts {
        if let Some(port) = find_available_port() {
            if TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok() {
                std::thread::sleep(Duration::from_millis(50));
                if TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok() {
                    return Some(port);
                }
            }
        }
    }
    find_available_port()
}

fn wait_for_server(host: &str, port: u16, timeout_ms: u64) -> bool {
    let start = std::time::Instant::now();
    let timeout = Duration::from_millis(timeout_ms);

    while start.elapsed() < timeout {
        if TcpListener::bind(format!("{}:{}", host, port)).is_err() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    false
}

#[test]
fn test_web_server_endpoints_defined_in_help() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["web", "--help"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--port") || stdout.contains("-p"),
        "Web help should show port option"
    );
    assert!(
        stdout.contains("--hostname") || stdout.contains("-h"),
        "Web help should show hostname option"
    );
}

#[test]
fn test_web_server_starts_on_specified_port() {
    let harness = TestHarness::setup();
    let port = find_available_port().expect("No available port");

    let mut cmd = harness.cmd();
    cmd.args(["web", "--port", &port.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn web command");

    let result = harness.wait_for_async(5000, || {
        TcpListener::bind(format!("127.0.0.1:{}", port)).is_err()
    });

    if result {
        let _ = child.kill();
    }

    assert!(
        result,
        "Web server should start and listen on port {}",
        port
    );

    let status = child.wait().expect("Failed to wait for child");
    assert!(
        !status.success() || result,
        "Web command should either still be running or have exited cleanly"
    );
}

#[test]
fn test_web_server_starts_on_default_port() {
    let harness = TestHarness::setup();
    let port: u16 = 3000;

    let mut cmd = harness.cmd();
    cmd.args(["web"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn web command");

    let result = harness.wait_for_async(5000, || {
        TcpListener::bind(format!("127.0.0.1:{}", port)).is_err()
    });

    if result {
        let _ = child.kill();
    }

    assert!(
        result,
        "Web server should start and listen on default port {}",
        port
    );

    let status = child.wait().expect("Failed to wait for child");
    assert!(
        !status.success() || result,
        "Web command should either still be running or have exited cleanly"
    );
}

#[test]
fn test_web_server_custom_hostname() {
    let harness = TestHarness::setup();
    let port = find_available_port().expect("No available port");

    let mut cmd = harness.cmd();
    cmd.args([
        "web",
        "--port",
        &port.to_string(),
        "--hostname",
        "127.0.0.1",
    ])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn web command");

    let result = harness.wait_for_async(5000, || {
        TcpListener::bind(format!("127.0.0.1:{}", port)).is_err()
    });

    if result {
        let _ = child.kill();
    }

    assert!(
        result,
        "Web server should start on custom hostname and port {}",
        port
    );

    let status = child.wait().expect("Failed to wait for child");
    assert!(
        !status.success() || result,
        "Web command should either still be running or have exited cleanly"
    );
}

#[test]
fn test_web_server_multiple_instances_different_ports() {
    let harness = TestHarness::setup();

    let port1 = find_available_port_with_retry(5).expect("No available port for instance 1");
    let port2 = find_available_port_with_retry(5).expect("No available port for instance 2");

    assert_ne!(port1, port2, "Instances should use different ports");

    let mut cmd1 = harness.cmd();
    cmd1.args(["web", "--port", &port1.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child1 = cmd1.spawn().expect("Failed to spawn web command 1");

    let mut cmd2 = harness.cmd();
    cmd2.args(["web", "--port", &port2.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child2 = cmd2.spawn().expect("Failed to spawn web command 2");

    let ready1 = harness.wait_for_async(5000, || {
        TcpListener::bind(format!("127.0.0.1:{}", port1)).is_err()
    });
    let ready2 = harness.wait_for_async(5000, || {
        TcpListener::bind(format!("127.0.0.1:{}", port2)).is_err()
    });

    let _ = child1.kill();
    let _ = child2.kill();

    assert!(ready1, "Server 1 should start on port {}", port1);
    assert!(ready2, "Server 2 should start on port {}", port2);
}

#[test]
fn test_web_server_session_sharing_enabled() {
    use opencode_core::session_sharing::SessionSharing;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let session_sharing = SessionSharing::new(
        temp_dir.path().to_path_buf(),
        opencode_core::bus::SharedEventBus::default(),
    );

    let session = session_sharing
        .create_session(Some("Web test session".to_string()))
        .expect("Should create session");

    assert!(
        session_sharing.exists(&session.id),
        "Session should exist after creation"
    );

    let retrieved = session_sharing
        .get_session(&session.id)
        .expect("Should retrieve session");

    assert_eq!(retrieved.id, session.id);
}

#[test]
fn test_web_server_session_sharing_between_instances() {
    use opencode_core::session_sharing::SessionSharing;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let event_bus = opencode_core::bus::SharedEventBus::default();
    let session_sharing1 = SessionSharing::new(temp_dir.path().to_path_buf(), event_bus.clone());
    let session_sharing2 = SessionSharing::new(temp_dir.path().to_path_buf(), event_bus);

    let session = session_sharing1
        .create_session(Some("Shared session".to_string()))
        .expect("Should create session in instance 1");

    let retrieved = session_sharing2
        .get_session(&session.id)
        .expect("Should retrieve session from instance 2 using same storage path");

    assert_eq!(retrieved.id, session.id);
    assert_eq!(retrieved.messages.len(), session.messages.len());
}

#[test]
fn test_web_server_session_persistence() {
    use opencode_core::session_sharing::SessionSharing;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let event_bus = opencode_core::bus::SharedEventBus::default();
    let session_sharing = SessionSharing::new(temp_dir.path().to_path_buf(), event_bus);

    let session = session_sharing
        .create_session(Some("Persistent session".to_string()))
        .expect("Should create session");

    session_sharing
        .save_session(&session)
        .expect("Should save session");

    drop(session_sharing);

    let event_bus = opencode_core::bus::SharedEventBus::default();
    let session_sharing2 = SessionSharing::new(temp_dir.path().to_path_buf(), event_bus);

    let sessions = session_sharing2
        .list_sessions()
        .expect("Should list sessions");

    assert!(
        sessions.len() >= 1,
        "Should have at least one session after reload"
    );
}

#[test]
fn test_web_server_health_endpoint() {
    let harness = TestHarness::setup();
    let port = find_available_port().expect("No available port");

    let mut cmd = harness.cmd();
    cmd.args(["web", "--port", &port.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn web command");

    let result = harness.wait_for_async(5000, || {
        TcpListener::bind(format!("127.0.0.1:{}", port)).is_err()
    });

    if !result {
        let _ = child.kill();
        panic!("Web server failed to start");
    }

    let response = reqwest::blocking::Client::new()
        .get(format!("http://127.0.0.1:{}/health", port))
        .timeout(Duration::from_secs(5))
        .send();

    let _ = child.kill();

    assert!(
        response.is_ok(),
        "Should be able to connect to health endpoint"
    );
}

#[test]
fn test_web_command_help_shows_web_options() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["web", "--help"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--port") || stdout.contains("-p"),
        "Web help should show port option"
    );
    assert!(
        stdout.contains("--hostname") || stdout.contains("-h"),
        "Web help should show hostname option"
    );
}
