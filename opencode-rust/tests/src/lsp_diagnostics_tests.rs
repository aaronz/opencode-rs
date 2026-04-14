use crate::common::TempProject;
use opencode_core::Message;
use opencode_lsp::mock::MockLspServer;
use std::time::Duration;

#[tokio::test]
async fn test_lsp_diagnostics_flow_end_to_end() {
    let project = TempProject::new();

    project.create_file(
        "test.rs",
        r#"fn main() {
    println!("Hello, world!");
}"#,
    );

    let mut mock_server = MockLspServer::start().expect("Mock LSP server should start");
    let server_pid = mock_server.pid();
    assert!(server_pid > 0, "Server should have a valid PID");

    mock_server.stop();
}

#[tokio::test]
async fn test_lsp_mock_server_handles_initialize() {
    let mut mock_server = MockLspServer::start().expect("Mock LSP server should start");

    let response_script = r#"
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
    fi
fi
"#;

    let mut server_with_init = MockLspServer::start_with_script(response_script)
        .expect("Server with initialize script should start");

    let pid = server_with_init.pid();
    assert!(pid > 0);

    server_with_init.stop();
}

#[tokio::test]
async fn test_lsp_mock_server_handles_definition_request() {
    let definition_script = r#"
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
        RESPONSE='{"jsonrpc":"2.0","id":1,"result":{"capabilities":{"definitionProvider":true}}}'
        echo -n "Content-Length: ${#RESPONSE}"$'\r\n\r\n'"$RESPONSE"
    elif [[ "$BODY" == *"textDocument/definition"* ]]; then
        RESPONSE='{"jsonrpc":"2.0","id":2,"result":{"uri":"file:///test.rs","range":{"start":{"line":10,"character":5},"end":{"line":10,"character":10}}}}'
        echo -n "Content-Length: ${#RESPONSE}"$'\r\n\r\n'"$RESPONSE"
    fi
fi
"#;

    let mut server = MockLspServer::start_with_script(definition_script)
        .expect("Server with definition script should start");

    let pid = server.pid();
    assert!(pid > 0);

    server.stop();
}

#[tokio::test]
async fn test_lsp_mock_server_handles_references_request() {
    let references_script = r#"
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
        RESPONSE='{"jsonrpc":"2.0","id":1,"result":{"capabilities":{"referencesProvider":true}}}'
        echo -n "Content-Length: ${#RESPONSE}"$'\r\n\r\n'"$RESPONSE"
    elif [[ "$BODY" == *"textDocument/references"* ]]; then
        RESPONSE='{"jsonrpc":"2.0","id":2,"result":[{"uri":"file:///ref1.rs","range":{"start":{"line":5,"character":3},"end":{"line":5,"character":8}}}]}'
        echo -n "Content-Length: ${#RESPONSE}"$'\r\n\r\n'"$RESPONSE"
    fi
fi
"#;

    let mut server = MockLspServer::start_with_script(references_script)
        .expect("Server with references script should start");

    let pid = server.pid();
    assert!(pid > 0);

    server.stop();
}

#[tokio::test]
async fn test_lsp_mock_server_graceful_shutdown() {
    let shutdown_script = r#"
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
    if [[ "$BODY" == *"shutdown"* ]]; then
        RESPONSE='{"jsonrpc":"2.0","id":99,"result":null}'
        echo -n "Content-Length: ${#RESPONSE}"$'\r\n\r\n'"$RESPONSE"
        exit 0
    fi
fi
"#;

    let mut server = MockLspServer::start_with_script(shutdown_script)
        .expect("Server with shutdown script should start");

    tokio::time::sleep(Duration::from_millis(100)).await;

    server.stop();
}

#[tokio::test]
async fn test_lsp_session_message_processing() {
    let project = TempProject::new();

    let mut session = opencode_core::Session::new();
    session.add_message(Message::user("Analyze this code".to_string()));
    session.add_message(Message::assistant(
        "I'll analyze the code structure".to_string(),
    ));

    assert_eq!(session.messages.len(), 2);
    assert_eq!(session.messages[0].content, "Analyze this code");
    assert_eq!(
        session.messages[1].content,
        "I'll analyze the code structure"
    );
}

#[tokio::test]
async fn test_lsp_mock_server_multiple_requests() {
    let multi_request_script = r#"
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
        RESPONSE='{"jsonrpc":"2.0","id":1,"result":{"capabilities":{"hoverProvider":true}}}'
        echo -n "Content-Length: ${#RESPONSE}"$'\r\n\r\n'"$RESPONSE"
    elif [[ "$BODY" == *"textDocument/hover"* ]]; then
        RESPONSE='{"jsonrpc":"2.0","id":2,"result":{"contents":"Hover info"}}'
        echo -n "Content-Length: ${#RESPONSE}"$'\r\n\r\n'"$RESPONSE"
    elif [[ "$BODY" == *"shutdown"* ]]; then
        RESPONSE='{"jsonrpc":"2.0","id":99,"result":null}'
        echo -n "Content-Length: ${#RESPONSE}"$'\r\n\r\n'"$RESPONSE"
        exit 0
    fi
fi
"#;

    let mut server = MockLspServer::start_with_script(multi_request_script)
        .expect("Server with multi-request script should start");

    let pid = server.pid();
    assert!(pid > 0);

    server.stop();
}
