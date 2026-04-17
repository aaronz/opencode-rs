use std::collections::VecDeque;

const DEFAULT_MAX_HISTORY: usize = 1000;

pub struct InputHistory {
    entries: VecDeque<String>,
    max_size: usize,
    current_index: Option<usize>,
}

impl InputHistory {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            max_size: DEFAULT_MAX_HISTORY,
            current_index: None,
        }
    }

    pub fn with_max_size(size: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_size: size,
            current_index: None,
        }
    }

    pub fn push(&mut self, input: String) {
        if input.trim().is_empty() {
            return;
        }

        if let Some(last) = self.entries.back() {
            if *last == input {
                return;
            }
        }

        if self.entries.len() >= self.max_size {
            self.entries.pop_front();
        }

        self.entries.push_back(input);
        self.current_index = None;
    }

    pub fn previous(&mut self) -> Option<String> {
        if self.entries.is_empty() {
            return None;
        }

        let new_index = match self.current_index {
            None => Some(self.entries.len().saturating_sub(1)),
            Some(0) => return None,
            Some(i) => Some(i - 1),
        };

        self.current_index = new_index;
        #[expect(clippy::expect_used)]
        let entry = self
            .entries
            .get(new_index.expect("current_index is Some at this point"));
        entry.cloned()
    }

    pub fn next(&mut self) -> Option<String> {
        if self.entries.is_empty() {
            return None;
        }

        let new_index = match self.current_index {
            None => return None,
            Some(i) if i >= self.entries.len() - 1 => {
                self.current_index = None;
                return Some(String::new());
            }
            Some(i) => Some(i + 1),
        };

        self.current_index = new_index;
        #[expect(clippy::expect_used)]
        let entry = self
            .entries
            .get(new_index.expect("current_index is Some at this point"));
        entry.cloned()
    }

    pub fn reset_navigation(&mut self) {
        self.current_index = None;
    }

    pub fn search(&self, prefix: &str) -> Vec<String> {
        self.entries
            .iter()
            .filter(|entry| entry.starts_with(prefix))
            .cloned()
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_index = None;
    }
}

impl Default for InputHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn history() -> InputHistory {
        InputHistory::new()
    }

    #[test]
    fn test_push_and_navigate() {
        let mut history = history();
        history.push("hello".to_string());
        history.push("world".to_string());

        assert_eq!(history.previous(), Some("world".to_string()));
        assert_eq!(history.previous(), Some("hello".to_string()));
        assert_eq!(history.next(), Some("world".to_string()));
    }

    #[test]
    fn test_deduplication() {
        let mut history = history();
        history.push("same".to_string());
        history.push("same".to_string());
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_empty_navigation() {
        let mut history = history();
        assert_eq!(history.previous(), None);
        assert_eq!(history.next(), None);
    }
}
