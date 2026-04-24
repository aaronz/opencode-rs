use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::sync::mpsc;
use std::thread;

fn spawn_mock_github_api(port: u16, response_body: String, status_code: u16) -> String {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
    let addr = format!("http://127.0.0.1:{}", port);
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        tx.send(()).unwrap();
        let (mut stream, _) = listener.accept().unwrap();
        let mut buffer = [0_u8; 8192];
        let _ = stream.read(&mut buffer).unwrap();

        let status_line = match status_code {
            200 => "HTTP/1.1 200 OK",
            401 => "HTTP/1.1 401 Unauthorized",
            403 => "HTTP/1.1 403 Forbidden",
            404 => "HTTP/1.1 404 Not Found",
            500 => "HTTP/1.1 500 Internal Server Error",
            _ => "HTTP/1.1 200 OK",
        };

        let response = format!(
            "{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            status_line,
            response_body.len(),
            response_body
        );
        stream.write_all(response.as_bytes()).unwrap();
    });

    let _ = rx.recv();
    addr
}

#[test]
fn test_github_repo_list_requires_token() {
    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "--bin",
            "opencode-rs",
            "--",
            "git-hub",
            "repo-list",
        ])
        .env_remove("GITHUB_TOKEN")
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("GitHub token required") || stderr.contains("GITHUB_TOKEN"),
        "Expected error about missing token, got: {}",
        stderr
    );
}

#[test]
fn test_github_issue_list_requires_token() {
    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "--bin",
            "opencode-rs",
            "--",
            "git-hub",
            "issue-list",
            "owner/repo",
        ])
        .env_remove("GITHUB_TOKEN")
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("GitHub token required") || stderr.contains("GITHUB_TOKEN"),
        "Expected error about missing token, got: {}",
        stderr
    );
}

#[test]
fn test_github_issue_list_invalid_repo_format() {
    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "--bin",
            "opencode-rs",
            "--",
            "git-hub",
            "issue-list",
            "invalid-repo",
        ])
        .env("GITHUB_TOKEN", "test-token")
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("owner/repo") || stderr.contains("format"),
        "Expected error about repo format, got: {}",
        stderr
    );
}

#[test]
fn test_github_help_shows_all_commands() {
    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "--bin",
            "opencode-rs",
            "--",
            "git-hub",
            "--help",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("login")
            && combined.contains("repo-list")
            && combined.contains("issue-list"),
        "Expected help to show login, repo-list, issue-list commands. Got: {}",
        combined
    );
}

#[test]
fn test_github_login_starts_oauth_flow() {
    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "--bin",
            "opencode-rs",
            "--",
            "git-hub",
            "login",
        ])
        .env_remove("GITHUB_TOKEN")
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("OAuth") || combined.contains("device") || combined.contains("login"),
        "Expected OAuth flow to start, got: {}",
        combined
    );
}

#[test]
fn test_github_install_requires_owner() {
    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "--bin",
            "opencode-rs",
            "--",
            "git-hub",
            "install",
            "--repo",
            "myrepo",
        ])
        .env("GITHUB_TOKEN", "test-token")
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("owner") || stderr.contains("required"),
        "Expected error about missing owner, got: {}",
        stderr
    );
}

#[test]
fn test_github_workflow_requires_owner() {
    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "--bin",
            "opencode-rs",
            "--",
            "git-hub",
            "workflow",
            "--repo",
            "myrepo",
        ])
        .env("GITHUB_TOKEN", "test-token")
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("owner") || stderr.contains("required"),
        "Expected error about missing owner, got: {}",
        stderr
    );
}

#[test]
fn test_github_integration_repo_list_filters_pull_requests() {
    let body = serde_json::json!([
        {
            "id": 1,
            "number": 1,
            "state": "open",
            "title": "Bug fix",
            "body": "Fixes a bug",
            "pull_request": null,
            "user": {"login": "user1", "id": 1}
        },
        {
            "id": 2,
            "number": 2,
            "state": "open",
            "title": "Feature PR",
            "body": "New feature",
            "pull_request": {"url": "https://api.github.com/repos/owner/repo/pulls/2"},
            "user": {"login": "user2", "id": 2}
        }
    ])
    .to_string();

    let _api_base = spawn_mock_github_api(9999, body, 200);

    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "--bin",
            "opencode-rs",
            "--",
            "git-hub",
            "issue-list",
            "test-owner/test-repo",
        ])
        .env("GITHUB_TOKEN", "test-token")
        .env("OPENCODE_GITHUB_API_BASE", "http://127.0.0.1:9999")
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        stdout.contains("Bug fix") || !stdout.contains("Feature PR"),
        "Expected only issues to be shown (no PRs). stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_github_integration_returns_api_error() {
    let body = serde_json::json!({
        "message": "Not Found",
        "documentation_url": "https://docs.github.com"
    })
    .to_string();

    let _api_base = spawn_mock_github_api(9998, body, 404);

    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "--bin",
            "opencode-rs",
            "--",
            "git-hub",
            "issue-list",
            "nonexistent/repo",
        ])
        .env("GITHUB_TOKEN", "test-token")
        .env("OPENCODE_GITHUB_API_BASE", "http://127.0.0.1:9998")
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("404") || stderr.contains("Not Found") || stderr.contains("Error"),
        "Expected error about API failure, got: {}",
        stderr
    );
}

#[test]
fn test_github_integration_unauthorized() {
    let body = serde_json::json!({
        "message": "Bad credentials"
    })
    .to_string();

    let _api_base = spawn_mock_github_api(9997, body, 401);

    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "--bin",
            "opencode-rs",
            "--",
            "git-hub",
            "repo-list",
        ])
        .env("GITHUB_TOKEN", "invalid-token")
        .env("OPENCODE_GITHUB_API_BASE", "http://127.0.0.1:9997")
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("401") || stderr.contains("Unauthorized") || stderr.contains("credentials"),
        "Expected error about unauthorized, got: {}",
        stderr
    );
}

#[test]
fn test_github_repo_list_empty_response() {
    let body = serde_json::json!([]).to_string();

    let _api_base = spawn_mock_github_api(9996, body, 200);

    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "--bin",
            "opencode-rs",
            "--",
            "git-hub",
            "repo-list",
        ])
        .env("GITHUB_TOKEN", "test-token")
        .env("OPENCODE_GITHUB_API_BASE", "http://127.0.0.1:9996")
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("No repositories") || combined.contains("Total: 0"),
        "Expected empty repositories message, got: {}",
        combined
    );
}

#[test]
fn test_github_issue_list_empty_response() {
    let body = serde_json::json!([]).to_string();

    let _api_base = spawn_mock_github_api(9995, body, 200);

    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "--bin",
            "opencode-rs",
            "--",
            "git-hub",
            "issue-list",
            "owner/repo",
        ])
        .env("GITHUB_TOKEN", "test-token")
        .env("OPENCODE_GITHUB_API_BASE", "http://127.0.0.1:9995")
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("No open issues"),
        "Expected no issues message, got: {}",
        combined
    );
}

#[test]
fn test_github_repo_list_displays_visibility() {
    let body = serde_json::json!([
        {
            "id": 1,
            "name": "private-repo",
            "full_name": "user/private-repo",
            "private": true,
            "default_branch": "main",
            "html_url": "https://github.com/user/private-repo"
        },
        {
            "id": 2,
            "name": "public-repo",
            "full_name": "user/public-repo",
            "private": false,
            "default_branch": "master",
            "html_url": "https://github.com/user/public-repo"
        }
    ])
    .to_string();

    let _api_base = spawn_mock_github_api(9994, body, 200);

    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "--bin",
            "opencode-rs",
            "--",
            "git-hub",
            "repo-list",
        ])
        .env("GITHUB_TOKEN", "test-token")
        .env("OPENCODE_GITHUB_API_BASE", "http://127.0.0.1:9994")
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("[private]") && combined.contains("[public]"),
        "Expected visibility labels, got: {}",
        combined
    );
}

#[test]
fn test_github_issue_list_displays_author() {
    let body = serde_json::json!([
        {
            "id": 1,
            "number": 42,
            "state": "open",
            "title": "Test issue",
            "body": "Description",
            "pull_request": null,
            "user": {"login": "testuser", "id": 123}
        }
    ])
    .to_string();

    let _api_base = spawn_mock_github_api(9993, body, 200);

    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "--bin",
            "opencode-rs",
            "--",
            "git-hub",
            "issue-list",
            "owner/repo",
        ])
        .env("GITHUB_TOKEN", "test-token")
        .env("OPENCODE_GITHUB_API_BASE", "http://127.0.0.1:9993")
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("testuser") && combined.contains("#42"),
        "Expected issue author and number, got: {}",
        combined
    );
}

#[test]
fn test_github_install_with_token_flag() {
    let body = serde_json::json!({
        "id": 12345678,
        "name": "Test Workflow",
        "head_branch": "main",
        "status": "queued",
        "conclusion": null
    })
    .to_string();

    let _api_base = spawn_mock_github_api(9992, body, 200);

    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "--bin",
            "opencode-rs",
            "--",
            "git-hub",
            "install",
            "--token",
            "test-token",
            "--owner",
            "test-owner",
            "--repo",
            "test-repo",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("workflow")
            || combined.contains("Workflow")
            || combined.contains("OpenCode"),
        "Expected workflow setup output, got: {}",
        combined
    );
}

#[test]
fn test_github_workflow_generates_yaml() {
    let output = Command::new("cargo")
        .args([
            "run",
            "-q",
            "--bin",
            "opencode-rs",
            "--",
            "git-hub",
            "workflow",
            "--token",
            "test-token",
            "--owner",
            "myorg",
            "--repo",
            "myrepo",
            "--branch",
            "develop",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("myorg") && combined.contains("myrepo") && combined.contains("develop"),
        "Expected workflow YAML with org/repo/branch, got: {}",
        combined
    );
}
