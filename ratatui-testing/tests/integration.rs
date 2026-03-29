use ratatui_testing::{char_key, AppEvent, AppState, StateTester, TestConfig, TestTerminal};

#[derive(Default, Debug, PartialEq, Clone)]
struct CounterState {
    counter: i32,
    quit: bool,
}

impl AppState for CounterState {}

fn update_counter(state: &mut CounterState, event: AppEvent) -> CounterState {
    if let AppEvent::Key(key) = event {
        match key.code {
            crossterm::event::KeyCode::Char('j') => state.counter += 1,
            crossterm::event::KeyCode::Char('k') => state.counter -= 1,
            crossterm::event::KeyCode::Char('q') => state.quit = true,
            _ => {}
        }
    }
    state.clone()
}

#[test]
fn test_state_transition() {
    let initial = CounterState::default();
    let mut tester = StateTester::new(initial);
    tester.send_event(char_key('j'), &mut update_counter);
    tester.assert_state(&CounterState {
        counter: 1,
        quit: false,
    });
    tester.send_event(char_key('j'), &mut update_counter);
    tester.assert_state(&CounterState {
        counter: 2,
        quit: false,
    });
}

#[test]
fn test_state_sequence() {
    let initial = CounterState::default();
    let mut tester = StateTester::new(initial);
    let events = vec![char_key('j'), char_key('j'), char_key('j'), char_key('k')];
    tester.send_sequence(events, &mut update_counter);
    tester.assert_state(&CounterState {
        counter: 2,
        quit: false,
    });
}

#[test]
fn test_quit_event() {
    let initial = CounterState::default();
    let mut tester = StateTester::new(initial);
    tester.send_event(char_key('q'), &mut update_counter);
    tester.assert_state(&CounterState {
        counter: 0,
        quit: true,
    });
}

#[test]
fn test_terminal_buffer() {
    let _terminal = TestTerminal::default_size();
}

#[test]
fn test_test_mode_config() {
    let _config = TestConfig::default();
}
