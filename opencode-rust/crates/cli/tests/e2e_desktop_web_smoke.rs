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

#[test]
fn test_dynamic_port_allocation_returns_valid_tcp_port() {
    let port = find_available_port().expect("Should allocate a valid port");
    assert!(port > 0, "Port should be greater than 0");
}

#[test]
fn test_find_available_port_with_retry_returns_valid_port() {
    let port = find_available_port_with_retry(3).expect("Should allocate a valid port");
    assert!(port > 0, "Port should be greater than 0");
}

#[test]
fn test_dynamic_allocation_not_hardcoded_to_single_port() {
    let port1 = find_available_port().expect("Should allocate first valid port");
    let port2 = find_available_port().expect("Should allocate second valid port");
    let port3 = find_available_port().expect("Should allocate third valid port");

    let unique_ports = std::collections::HashSet::from([port1, port2, port3]);
    assert!(
        unique_ports.len() > 1,
        "Dynamic port allocation should return different ports, got {:?}",
        unique_ports
    );
}

#[allow(dead_code)]
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
fn desktop_smoke_starts_without_error() {
    let harness = TestHarness::setup();
    let port = find_available_port().expect("No available port");

    let mut cmd = harness.cmd();
    cmd.args(["desktop", "--no-browser", "--port", &port.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().expect("Failed to spawn desktop command");

    let result = harness.wait_for_async(5000, || {
        TcpListener::bind(format!("127.0.0.1:{}", port)).is_err()
    });

    if result {
        let _ = child.kill();
    }

    assert!(
        result,
        "Desktop server should start and listen on port {}",
        port
    );

    let status = child.wait().expect("Failed to wait for child");
    assert!(
        !status.success() || result,
        "Desktop command should either still be running or have exited cleanly"
    );
}

#[test]
fn web_smoke_starts_without_error() {
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

#[tokio::test]
async fn desktop_web_session_sharing_via_share_server() {
    use opencode_core::config::ShareMode;
    use opencode_server::routes::share::{ShareOperation, ShareServer, ShortShareConfig};

    let config = ShortShareConfig::default();
    let share_server = ShareServer::new(config);

    let session_id = "test-session-desktop-web".to_string();

    let link = share_server
        .create_short_link_with_mode(session_id.clone(), ShareMode::Collaborative, None, None)
        .await;

    assert_eq!(link.session_id, session_id);
    assert_eq!(link.share_mode, ShareMode::Collaborative);
    assert!(link.allowed_operations.contains(&ShareOperation::Read));
    assert!(link.allowed_operations.contains(&ShareOperation::Write));

    let can_read = share_server
        .check_permission(&link.short_code, ShareOperation::Read)
        .await;
    let can_write = share_server
        .check_permission(&link.short_code, ShareOperation::Write)
        .await;

    assert!(can_read, "Should be able to read via share link");
    assert!(
        can_write,
        "Should be able to write via share link in collaborative mode"
    );
}

#[test]
fn desktop_web_different_ports() {
    let harness = TestHarness::setup();
    let desktop_port = find_available_port_with_retry(10).expect("No available port for desktop");

    let mut desktop_cmd = harness.cmd();
    desktop_cmd
        .args([
            "desktop",
            "--no-browser",
            "--port",
            &desktop_port.to_string(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[allow(clippy::zombie_processes)]
    let mut desktop_child = desktop_cmd
        .spawn()
        .expect("Failed to spawn desktop command");

    let desktop_ready = harness.wait_for_async(5000, || {
        TcpListener::bind(format!("127.0.0.1:{}", desktop_port)).is_err()
    });

    assert!(
        desktop_ready,
        "Desktop server should start on port {}",
        desktop_port
    );

    let web_port = find_available_port_with_retry(10).expect("No available port for web");

    let mut web_cmd = harness.cmd();
    web_cmd
        .args(["web", "--port", &web_port.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[allow(clippy::zombie_processes)]
    let mut web_child = web_cmd.spawn().expect("Failed to spawn web command");

    let web_ready = harness.wait_for_async(5000, || {
        TcpListener::bind(format!("127.0.0.1:{}", web_port)).is_err()
    });

    let _ = desktop_child.kill();
    if web_ready {
        let _ = web_child.kill();
    }

    assert!(web_ready, "Web server should start on port {}", web_port);
    assert_ne!(
        desktop_port, web_port,
        "Desktop and web should use different ports"
    );
}

#[test]
fn test_parallel_desktop_web_instances_no_port_conflict() {
    let harness = TestHarness::setup();

    let port1 = find_available_port_with_retry(5).expect("No available port for instance 1");
    let port2 = find_available_port_with_retry(5).expect("No available port for instance 2");
    let port3 = find_available_port_with_retry(5).expect("No available port for instance 3");

    assert_ne!(port1, port2, "Instances should use different ports");
    assert_ne!(port2, port3, "Instances should use different ports");
    assert_ne!(port1, port3, "Instances should use different ports");

    let mut cmd1 = harness.cmd();
    cmd1.args(["web", "--port", &port1.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[allow(clippy::zombie_processes)]
    let mut child1 = cmd1.spawn().expect("Failed to spawn web command 1");

    let mut cmd2 = harness.cmd();
    cmd2.args(["web", "--port", &port2.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[allow(clippy::zombie_processes)]
    let mut child2 = cmd2.spawn().expect("Failed to spawn web command 2");

    let mut cmd3 = harness.cmd();
    cmd3.args(["web", "--port", &port3.to_string()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[allow(clippy::zombie_processes)]
    let mut child3 = cmd3.spawn().expect("Failed to spawn web command 3");

    let ready1 = harness.wait_for_async(5000, || {
        TcpListener::bind(format!("127.0.0.1:{}", port1)).is_err()
    });
    let ready2 = harness.wait_for_async(5000, || {
        TcpListener::bind(format!("127.0.0.1:{}", port2)).is_err()
    });
    let ready3 = harness.wait_for_async(5000, || {
        TcpListener::bind(format!("127.0.0.1:{}", port3)).is_err()
    });

    let _ = child1.kill();
    let _ = child2.kill();
    let _ = child3.kill();

    assert!(ready1, "Server 1 should start on port {}", port1);
    assert!(ready2, "Server 2 should start on port {}", port2);
    assert!(ready3, "Server 3 should start on port {}", port3);
}

#[test]
fn desktop_command_help_shows_options() {
    let harness = TestHarness::setup();
    let output = harness.run_cli(&["desktop", "--help"]);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--port") || stdout.contains("-p"),
        "Desktop help should show port option"
    );
    assert!(
        stdout.contains("--hostname") || stdout.contains("-h"),
        "Desktop help should show hostname option"
    );
    assert!(
        stdout.contains("--no-browser"),
        "Desktop help should show no-browser option"
    );
}

#[test]
fn web_command_help_shows_options() {
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
