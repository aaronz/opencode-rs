use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub created_at: Instant,
    pub last_active: Instant,
    pub message_count: usize,
}

impl Session {
    pub fn new(name: impl Into<String>) -> Self {
        let now = Instant::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            created_at: now,
            last_active: now,
            message_count: 0,
        }
    }

    pub fn time_since_active(&self) -> Duration {
        self.last_active.elapsed()
    }

    pub fn time_since_created(&self) -> Duration {
        self.created_at.elapsed()
    }
}

pub struct SessionManager {
    sessions: Vec<Session>,
    current_index: usize,
    sessions_file: Option<PathBuf>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: vec![],
            current_index: 0,
            sessions_file: None,
        }
    }

    pub fn with_file(path: PathBuf) -> Self {
        let mut manager = Self::new();
        manager.sessions_file = Some(path.clone());
        if let Err(e) = manager.load_from_file(&path) {
            tracing::error!("Failed to load sessions: {}", e);
        }
        manager
    }

    pub fn save(&self) {
        if let Some(ref path) = self.sessions_file {
            if let Err(e) = self.save_to_file(path) {
                tracing::error!("Failed to save sessions: {}", e);
            }
        }
    }

    pub fn add_session(&mut self, name: impl Into<String>) -> &Session {
        let session = Session::new(name);
        self.sessions.push(session);
        self.current_index = self.sessions.len() - 1;
        &self.sessions[self.current_index]
    }

    pub fn current(&self) -> Option<&Session> {
        self.sessions.get(self.current_index)
    }

    pub fn current_mut(&mut self) -> Option<&mut Session> {
        self.sessions.get_mut(self.current_index)
    }

    pub fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn get_session(&self, index: usize) -> Option<&Session> {
        self.sessions.get(index)
    }

    pub fn list(&self) -> Vec<&Session> {
        self.sessions.iter().collect()
    }

    pub fn list_names(&self) -> Vec<String> {
        self.sessions.iter().map(|s| s.name.clone()).collect()
    }

    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }

    pub fn search(&self, query: &str) -> Vec<&Session> {
        let query_lower = query.to_lowercase();
        self.sessions
            .iter()
            .filter(|s| s.name.to_lowercase().contains(&query_lower))
            .collect()
    }

    pub fn select(&mut self, index: usize) {
        if index < self.sessions.len() {
            self.current_index = index;
        }
    }

    pub fn delete_session(&mut self, index: usize) -> bool {
        if index < self.sessions.len() {
            self.sessions.remove(index);
            if self.current_index >= index && self.current_index > 0 {
                self.current_index -= 1;
            }
            true
        } else {
            false
        }
    }

    pub fn rename_session(&mut self, index: usize, new_name: String) -> bool {
        if let Some(session) = self.sessions.get_mut(index) {
            session.name = new_name;
            true
        } else {
            false
        }
    }

    pub fn save_to_file(&self, path: &PathBuf) -> std::io::Result<()> {
        let mut content = String::new();
        for session in &self.sessions {
            content.push_str(&format!(
                "{}|{}|{}|{}\n",
                session.id,
                session.name,
                session.created_at.elapsed().as_secs(),
                session.last_active.elapsed().as_secs()
            ));
        }
        fs::write(path, content)
    }

    pub fn load_from_file(&mut self, path: &PathBuf) -> std::io::Result<()> {
        if !path.exists() {
            return Ok(());
        }
        let content = fs::read_to_string(path)?;
        self.sessions.clear();
        for line in content.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                let mut session = Session::new(parts[1]);
                session.id = parts[0].to_string();
                self.sessions.push(session);
            }
        }
        Ok(())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_new() {
        let session = Session::new("test session");
        assert!(!session.id.is_empty());
        assert_eq!(session.name, "test session");
    }

    #[test]
    fn test_session_manager_add() {
        let mut manager = SessionManager::new();
        manager.add_session("session 1");
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_session_manager_current() {
        let mut manager = SessionManager::new();
        manager.add_session("test");
        assert!(manager.current().is_some());
        assert_eq!(manager.current().unwrap().name, "test");
    }

    #[test]
    fn test_session_manager_search() {
        let mut manager = SessionManager::new();
        manager.add_session("rust project");
        manager.add_session("python scripts");
        let results = manager.search("rust");
        assert_eq!(results.len(), 1);
    }
}
