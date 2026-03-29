pub trait AppState: Default + Clone + PartialEq + std::fmt::Debug {}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AppEvent {
    Key(crossterm::event::KeyEvent),
    Mouse(crossterm::event::MouseEvent),
    Resize(u16, u16),
    Tick,
    Custom(u16),
}

impl From<crossterm::event::KeyEvent> for AppEvent {
    fn from(k: crossterm::event::KeyEvent) -> Self {
        AppEvent::Key(k)
    }
}

pub struct StateTester<S: AppState> {
    state: S,
}

impl<S: AppState> StateTester<S> {
    pub fn new(initial: S) -> Self {
        Self { state: initial }
    }

    pub fn send_event<E: Into<AppEvent>>(
        &mut self,
        event: E,
        mut updater: impl FnMut(&mut S, AppEvent) -> S,
    ) {
        self.state = updater(&mut self.state, event.into());
    }

    pub fn send_sequence<E: Into<AppEvent>>(
        &mut self,
        events: Vec<E>,
        mut updater: impl FnMut(&mut S, AppEvent) -> S,
    ) {
        for event in events {
            self.send_event(event, &mut updater);
        }
    }

    pub fn assert_state(&self, expected: &S)
    where
        S: PartialEq,
    {
        assert_eq!(self.state, *expected);
    }
    pub fn get_state(&self) -> &S {
        &self.state
    }
}

pub fn key_event(
    code: crossterm::event::KeyCode,
    modifiers: crossterm::event::KeyModifiers,
) -> AppEvent {
    AppEvent::Key(crossterm::event::KeyEvent::new(code, modifiers))
}
pub fn enter() -> AppEvent {
    key_event(
        crossterm::event::KeyCode::Enter,
        crossterm::event::KeyModifiers::NONE,
    )
}
pub fn escape() -> AppEvent {
    key_event(
        crossterm::event::KeyCode::Esc,
        crossterm::event::KeyModifiers::NONE,
    )
}
pub fn char_key(c: char) -> AppEvent {
    key_event(
        crossterm::event::KeyCode::Char(c),
        crossterm::event::KeyModifiers::NONE,
    )
}
