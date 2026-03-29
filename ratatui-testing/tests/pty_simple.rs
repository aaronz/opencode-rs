use ratatui_testing::PtySession;
use std::time::Duration;

#[test]
fn test_simple_pty() {
    let mut session = PtySession::spawn_command("bash", &["-c", "echo hello"]).unwrap();
    let output = session.read(Duration::from_secs(2)).unwrap();
    println!("Output: {:?}", output);
    assert!(output.contains("hello"), "Should contain hello");
}
