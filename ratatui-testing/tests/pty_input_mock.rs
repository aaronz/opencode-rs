//! Mock-based TUI Input Testing (for development/debugging)
//!
//! This test module uses mock implementations to test input handling
//! without requiring full PTY functionality. Use this to verify test logic.
//!
//! Run with: cargo test --test pty_input_mock

mod mocks {
    use std::collections::VecDeque;
    use std::sync::Mutex;

    #[derive(Debug, Clone, PartialEq)]
    pub enum InputEvent {
        Key(String),
        Text(String),
        Signal(u8),
    }

    pub struct MockInputHandler {
        input_buffer: Mutex<VecDeque<InputEvent>>,
        output_buffer: Mutex<String>,
    }

    impl MockInputHandler {
        pub fn new() -> Self {
            Self {
                input_buffer: Mutex::new(VecDeque::new()),
                output_buffer: Mutex::new(String::new()),
            }
        }

        pub fn send_key(&self, key: &str) {
            let mut buffer = self.input_buffer.lock().unwrap();
            buffer.push_back(InputEvent::Key(key.to_string()));
        }

        pub fn send_text(&self, text: &str) {
            let mut buffer = self.input_buffer.lock().unwrap();
            buffer.push_back(InputEvent::Text(text.to_string()));
        }

        pub fn send_signal(&self, sig: u8) {
            let mut buffer = self.input_buffer.lock().unwrap();
            buffer.push_back(InputEvent::Signal(sig));
        }

        pub fn write_output(&self, text: &str) {
            let mut output = self.output_buffer.lock().unwrap();
            output.push_str(text);
        }

        pub fn get_output(&self) -> String {
            let output = self.output_buffer.lock().unwrap();
            output.clone()
        }

        pub fn pop_input(&self) -> Option<InputEvent> {
            let mut buffer = self.input_buffer.lock().unwrap();
            buffer.pop_front()
        }

        pub fn has_input(&self) -> bool {
            let buffer = self.input_buffer.lock().unwrap();
            !buffer.is_empty()
        }
    }

    impl Default for MockInputHandler {
        fn default() -> Self {
            Self::new()
        }
    }

    pub fn send_key(key: &str) -> Vec<u8> {
        match key {
            "enter" => vec![b'\n'],
            "escape" => vec![b'\x1b'],
            "tab" => vec![b'\t'],
            "up" => b"\x1b[A".to_vec(),
            "down" => b"\x1b[B".to_vec(),
            "right" => b"\x1b[C".to_vec(),
            "left" => b"\x1b[D".to_vec(),
            _ => key.as_bytes().to_vec(),
        }
    }
}

mod tests {
    use super::mocks::*;

    #[test]
    fn test_mock_input_handler_basic() {
        let handler = MockInputHandler::new();

        handler.send_key("enter");
        handler.send_text("hello");

        assert!(handler.has_input());

        let event = handler.pop_input();
        assert!(matches!(event, Some(InputEvent::Key(k)) if k == "enter"));

        let event = handler.pop_input();
        assert!(matches!(event, Some(InputEvent::Text(t)) if t == "hello"));
    }

    #[test]
    fn test_send_key_function() {
        assert_eq!(send_key("enter"), vec![b'\n']);
        assert_eq!(send_key("escape"), vec![b'\x1b']);
        assert_eq!(send_key("tab"), vec![b'\t']);
        assert_eq!(send_key("up"), b"\x1b[A".to_vec());
        assert_eq!(send_key("down"), b"\x1b[B".to_vec());
    }

    #[test]
    fn test_signal_handling() {
        let handler = MockInputHandler::new();

        handler.send_signal(3); // Ctrl+C
        handler.send_signal(26); // Ctrl+Z

        assert!(handler.has_input());

        let event = handler.pop_input();
        assert!(matches!(event, Some(InputEvent::Signal(3))));

        let event = handler.pop_input();
        assert!(matches!(event, Some(InputEvent::Signal(26))));
    }

    #[test]
    fn test_output_buffer() {
        let handler = MockInputHandler::new();

        handler.write_output("line1\n");
        handler.write_output("line2\n");

        let output = handler.get_output();
        assert!(output.contains("line1"));
        assert!(output.contains("line2"));
    }

    #[test]
    fn test_unicode_input() {
        let handler = MockInputHandler::new();

        handler.send_text("你好");
        handler.send_text("🎉");

        let event = handler.pop_input();
        assert!(matches!(event, Some(InputEvent::Text(t)) if t == "你好"));

        let event = handler.pop_input();
        assert!(matches!(event, Some(InputEvent::Text(t)) if t == "🎉"));
    }

    #[test]
    fn test_multiline_input() {
        let handler = MockInputHandler::new();

        let multiline = "line1\nline2\nline3";
        handler.send_text(multiline);

        let event = handler.pop_input();
        assert!(matches!(event, Some(InputEvent::Text(t)) if t == "line1\nline2\nline3"));
    }

    #[test]
    fn test_special_characters() {
        let handler = MockInputHandler::new();

        handler.send_text("test with $HOME and `backticks`");
        handler.send_text("quotes: \"double\" and 'single'");
        handler.send_text("backslash: \\");

        assert!(handler.has_input());
    }

    #[test]
    fn test_empty_input() {
        let handler = MockInputHandler::new();

        handler.send_text("");
        handler.send_key("enter");

        assert!(handler.has_input());
    }

    #[test]
    fn test_long_input() {
        let handler = MockInputHandler::new();

        let long_text = "x".repeat(10000);
        handler.send_text(&long_text);

        let event = handler.pop_input();
        assert!(matches!(event, Some(InputEvent::Text(t)) if t.len() == 10000));
    }

    #[test]
    fn test_key_sequence() {
        let handler = MockInputHandler::new();

        // Simulate typing "hello<enter>"
        handler.send_text("hello");
        handler.send_key("enter");

        // Simulate up arrow to recall history
        handler.send_key("up");
        handler.send_key("enter");

        // Simulate Ctrl+C
        handler.send_signal(3);

        assert_eq!(
            handler.pop_input(),
            Some(InputEvent::Text("hello".to_string()))
        );
        assert_eq!(
            handler.pop_input(),
            Some(InputEvent::Key("enter".to_string()))
        );
        assert_eq!(handler.pop_input(), Some(InputEvent::Key("up".to_string())));
        assert_eq!(
            handler.pop_input(),
            Some(InputEvent::Key("enter".to_string()))
        );
        assert_eq!(handler.pop_input(), Some(InputEvent::Signal(3)));
    }

    #[test]
    fn test_tab_completion_simulation() {
        let handler = MockInputHandler::new();

        handler.send_text("cat test");
        handler.send_key("tab");

        let event = handler.pop_input();
        assert!(matches!(event, Some(InputEvent::Text(t)) if t == "cat test"));

        let event = handler.pop_input();
        assert!(matches!(event, Some(InputEvent::Key(k)) if k == "tab"));
    }

    #[test]
    fn test_home_end_keys() {
        let handler = MockInputHandler::new();

        handler.send_text("echo test");
        handler.send_key("home");
        handler.send_text("start_");
        handler.send_key("end");
        handler.send_text("_end");

        let mut expected = vec![
            InputEvent::Text("echo test".to_string()),
            InputEvent::Key("home".to_string()),
            InputEvent::Text("start_".to_string()),
            InputEvent::Key("end".to_string()),
            InputEvent::Text("_end".to_string()),
        ];

        for exp in expected.drain(..) {
            assert_eq!(handler.pop_input(), Some(exp));
        }
    }

    #[test]
    fn test_function_keys() {
        let handler = MockInputHandler::new();

        handler.send_key("f1");
        handler.send_key("f2");
        handler.send_key("f12");

        assert!(handler.has_input());
    }

    #[test]
    fn test_ctrl_combinations() {
        let handler = MockInputHandler::new();

        // Ctrl+A (1)
        handler.send_signal(1);
        // Ctrl+E (5)
        handler.send_signal(5);
        // Ctrl+U (21)
        handler.send_signal(21);
        // Ctrl+K (11)
        handler.send_signal(11);
        // Ctrl+W (23)
        handler.send_signal(23);

        assert_eq!(handler.pop_input(), Some(InputEvent::Signal(1)));
        assert_eq!(handler.pop_input(), Some(InputEvent::Signal(5)));
        assert_eq!(handler.pop_input(), Some(InputEvent::Signal(21)));
        assert_eq!(handler.pop_input(), Some(InputEvent::Signal(11)));
        assert_eq!(handler.pop_input(), Some(InputEvent::Signal(23)));
    }

    #[test]
    fn test_escape_sequences() {
        let handler = MockInputHandler::new();

        // Red text
        handler.send_text("\x1b[31m");
        // Reset
        handler.send_text("\x1b[0m");
        // Bold green
        handler.send_text("\x1b[1;32m");

        let event = handler.pop_input();
        assert!(matches!(event, Some(InputEvent::Text(t)) if t == "\x1b[31m"));
    }

    #[test]
    fn test_binary_data_handling() {
        let handler = MockInputHandler::new();

        let binary_data: Vec<u8> = (0..256).collect();
        let binary_str = String::from_utf8_lossy(&binary_data);
        handler.send_text(&binary_str);

        let event = handler.pop_input();
        assert!(event.is_some());
    }

    #[test]
    fn test_repeated_operations() {
        let handler = MockInputHandler::new();

        for i in 0..1000 {
            handler.send_text(&format!("text{}", i));
        }

        for i in 0..1000 {
            let event = handler.pop_input();
            assert!(event.is_some());
        }

        assert!(!handler.has_input());
    }
}
