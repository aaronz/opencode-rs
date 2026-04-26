use crossterm::event::{KeyCode, KeyEvent, MouseEvent};

pub trait Action: Sized + Clone + std::fmt::Debug + PartialEq + Eq {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn from_key_code(code: KeyCode) -> Option<Self> {
        match code {
            KeyCode::Up => Some(Direction::Up),
            KeyCode::Down => Some(Direction::Down),
            KeyCode::Left => Some(Direction::Left),
            KeyCode::Right => Some(Direction::Right),
            _ => None,
        }
    }
}

pub trait InputMapper<A: Action>: Send + Sync {
    fn key_to_action(&self, key: KeyEvent) -> Option<A>;
    fn mouse_to_action(&self, event: MouseEvent, layout_info: &(dyn Send + Sync)) -> Option<A>;
}

pub mod helpers {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    pub fn key_to_action_simple<A: Action + From<BasicAction>, F>(key: KeyEvent, f: F) -> Option<A>
    where
        F: Fn(BasicAction) -> Option<A>,
    {
        let basic = match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::NONE) => Some(BasicAction::Quit),
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => Some(BasicAction::Quit),
            (KeyCode::Esc, _) | (KeyCode::Char('['), KeyModifiers::NONE) => {
                Some(BasicAction::Escape)
            }
            (KeyCode::Enter, _) => Some(BasicAction::Enter),
            (KeyCode::Tab, _) => Some(BasicAction::Tab),
            (KeyCode::Backspace, _) => Some(BasicAction::Backspace),
            (KeyCode::Delete, _) => Some(BasicAction::Delete),
            (KeyCode::Home, _) => Some(BasicAction::Home),
            (KeyCode::End, _) => Some(BasicAction::End),
            (KeyCode::PageUp, _) => Some(BasicAction::PageUp),
            (KeyCode::PageDown, _) => Some(BasicAction::PageDown),
            (KeyCode::Up, _) => Some(BasicAction::Move(Direction::Up)),
            (KeyCode::Down, _) => Some(BasicAction::Move(Direction::Down)),
            (KeyCode::Left, _) => Some(BasicAction::Move(Direction::Left)),
            (KeyCode::Right, _) => Some(BasicAction::Move(Direction::Right)),
            (KeyCode::Char('k'), KeyModifiers::NONE) => Some(BasicAction::Move(Direction::Up)),
            (KeyCode::Char('j'), KeyModifiers::NONE) => Some(BasicAction::Move(Direction::Down)),
            (KeyCode::Char('h'), KeyModifiers::NONE) => Some(BasicAction::Move(Direction::Left)),
            (KeyCode::Char('l'), KeyModifiers::NONE) => Some(BasicAction::Move(Direction::Right)),
            (KeyCode::Char(c), KeyModifiers::NONE) if c.is_ascii() => Some(BasicAction::Char(c)),
            (KeyCode::Char(c), KeyModifiers::SHIFT) if c.is_ascii() => {
                Some(BasicAction::Char(c.to_ascii_uppercase()))
            }
            _ => None,
        }?;

        f(basic)
    }

    pub fn vim_key_to_action<A: Action + From<BasicAction>, F>(key: KeyEvent, f: F) -> Option<A>
    where
        F: Fn(BasicAction) -> Option<A>,
    {
        key_to_action_simple(key, f)
    }

    pub fn emacs_key_to_action<A: Action + From<BasicAction>, F>(key: KeyEvent, f: F) -> Option<A>
    where
        F: Fn(BasicAction) -> Option<A>,
    {
        let basic = match (key.code, key.modifiers) {
            (KeyCode::Char('b'), KeyModifiers::CONTROL) => Some(BasicAction::Move(Direction::Left)),
            (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                Some(BasicAction::Move(Direction::Right))
            }
            (KeyCode::Char('p'), KeyModifiers::CONTROL) => Some(BasicAction::Move(Direction::Up)),
            (KeyCode::Char('n'), KeyModifiers::CONTROL) => Some(BasicAction::Move(Direction::Down)),
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => Some(BasicAction::Home),
            (KeyCode::Char('e'), KeyModifiers::CONTROL) => Some(BasicAction::End),
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => Some(BasicAction::Delete),
            (KeyCode::Char('k'), KeyModifiers::CONTROL) => Some(BasicAction::KillLine),
            _ => return key_to_action_simple(key, f),
        }?;
        f(basic)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BasicAction {
    Quit,
    Escape,
    Enter,
    Tab,
    Backspace,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Move(Direction),
    Char(char),
    KillLine,
}

impl Action for BasicAction {}

impl From<BasicAction> for () {
    fn from(_: BasicAction) {}
}

#[allow(unused_imports)]
pub mod prelude {
    pub use super::helpers::{emacs_key_to_action, key_to_action_simple, vim_key_to_action};
    pub use super::{Action, Direction};
}

#[cfg(test)]
mod tests {
    use super::helpers::*;
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn make_key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn direction_from_key_code() {
        assert_eq!(Direction::from_key_code(KeyCode::Up), Some(Direction::Up));
        assert_eq!(
            Direction::from_key_code(KeyCode::Down),
            Some(Direction::Down)
        );
        assert_eq!(
            Direction::from_key_code(KeyCode::Left),
            Some(Direction::Left)
        );
        assert_eq!(
            Direction::from_key_code(KeyCode::Right),
            Some(Direction::Right)
        );
        assert_eq!(Direction::from_key_code(KeyCode::Enter), None);
    }

    #[test]
    fn basic_action_quit_keys() {
        let quit_q = make_key(KeyCode::Char('q'), KeyModifiers::NONE);
        let quit_ctrl_c = make_key(KeyCode::Char('c'), KeyModifiers::CONTROL);

        let result_q: Option<BasicAction> = key_to_action_simple(quit_q, |a| match a {
            BasicAction::Quit => Some(a),
            _ => None,
        });
        assert!(result_q.is_some());

        let result_ctrl_c: Option<BasicAction> = key_to_action_simple(quit_ctrl_c, |a| match a {
            BasicAction::Quit => Some(a),
            _ => None,
        });
        assert!(result_ctrl_c.is_some());
    }

    #[test]
    fn basic_action_vim_motion() {
        let k = make_key(KeyCode::Char('k'), KeyModifiers::NONE);
        let j = make_key(KeyCode::Char('j'), KeyModifiers::NONE);

        let result_k: Option<BasicAction> = key_to_action_simple(k, Some);
        assert_eq!(result_k, Some(BasicAction::Move(Direction::Up)));

        let result_j: Option<BasicAction> = key_to_action_simple(j, Some);
        assert_eq!(result_j, Some(BasicAction::Move(Direction::Down)));
    }

    #[test]
    fn basic_action_arrow_keys() {
        let up = make_key(KeyCode::Up, KeyModifiers::NONE);
        let down = make_key(KeyCode::Down, KeyModifiers::NONE);

        let result_up: Option<BasicAction> = key_to_action_simple(up, Some);
        assert_eq!(result_up, Some(BasicAction::Move(Direction::Up)));

        let result_down: Option<BasicAction> = key_to_action_simple(down, Some);
        assert_eq!(result_down, Some(BasicAction::Move(Direction::Down)));
    }

    #[test]
    fn basic_action_char_input() {
        let a = make_key(KeyCode::Char('a'), KeyModifiers::NONE);
        let shift_a = make_key(KeyCode::Char('a'), KeyModifiers::SHIFT);

        let result_a: Option<BasicAction> = key_to_action_simple(a, Some);
        assert_eq!(result_a, Some(BasicAction::Char('a')));

        let result_shift_a: Option<BasicAction> = key_to_action_simple(shift_a, Some);
        assert_eq!(result_shift_a, Some(BasicAction::Char('A')));
    }

    #[test]
    fn emacs_ctrl_navigation() {
        let ctrl_b = make_key(KeyCode::Char('b'), KeyModifiers::CONTROL);
        let ctrl_f = make_key(KeyCode::Char('f'), KeyModifiers::CONTROL);

        let result_b: Option<BasicAction> = emacs_key_to_action(ctrl_b, Some);
        assert_eq!(result_b, Some(BasicAction::Move(Direction::Left)));

        let result_f: Option<BasicAction> = emacs_key_to_action(ctrl_f, Some);
        assert_eq!(result_f, Some(BasicAction::Move(Direction::Right)));
    }
}
