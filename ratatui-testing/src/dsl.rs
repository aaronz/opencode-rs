use std::time::Duration;

pub struct TuiTestBuilder {
    actions: Vec<TestAction>,
    timeout: Duration,
}

enum TestAction {
    TypeText(String),
    PressKey(Vec<u8>),
    ExpectScreen(String),
    WaitFor(String, Duration),
    Wait(Duration),
}

impl TuiTestBuilder {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            timeout: Duration::from_secs(5),
        }
    }

    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = duration;
        self
    }

    pub fn type_text(mut self, text: &str) -> Self {
        self.actions.push(TestAction::TypeText(text.to_string()));
        self
    }

    pub fn press_key(mut self, key: &str) -> Self {
        let bytes = match key {
            "enter" => b"\n".to_vec(),
            "escape" => b"\x1b".to_vec(),
            "tab" => b"\t".to_vec(),
            "up" => b"\x1b[A".to_vec(),
            "down" => b"\x1b[B".to_vec(),
            "right" => b"\x1b[C".to_vec(),
            "left" => b"\x1b[D".to_vec(),
            _ => key.as_bytes().to_vec(),
        };
        self.actions.push(TestAction::PressKey(bytes));
        self
    }

    pub fn press_enter(mut self) -> Self {
        self.press_key("enter")
    }

    pub fn press_escape(mut self) -> Self {
        self.press_key("escape")
    }

    pub fn press_up(mut self) -> Self {
        self.press_key("up")
    }

    pub fn press_down(mut self) -> Self {
        self.press_key("down")
    }

    pub fn expect_screen(mut self, content: &str) -> Self {
        self.actions
            .push(TestAction::ExpectScreen(content.to_string()));
        self
    }

    pub fn wait_for(mut self, text: &str, timeout: Duration) -> Self {
        self.actions
            .push(TestAction::WaitFor(text.to_string(), timeout));
        self
    }

    pub fn wait(mut self, duration: Duration) -> Self {
        self.actions.push(TestAction::Wait(duration));
        self
    }

    pub fn actions(&self) -> &[TestAction] {
        &self.actions
    }

    pub fn timeout_duration(&self) -> Duration {
        self.timeout
    }
}

impl Default for TuiTestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn tui() -> TuiTestBuilder {
    TuiTestBuilder::new()
}
