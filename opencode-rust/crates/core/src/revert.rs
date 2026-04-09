use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevertPoint {
    pub id: String,
    pub message_index: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub description: String,
}

pub struct RevertManager {
    points: VecDeque<RevertPoint>,
    max_points: usize,
}

impl RevertManager {
    pub fn new(max_points: usize) -> Self {
        Self {
            points: VecDeque::new(),
            max_points,
        }
    }

    pub fn create_point(&mut self, message_index: usize, description: String) -> RevertPoint {
        let point = RevertPoint {
            id: uuid::Uuid::new_v4().to_string(),
            message_index,
            timestamp: chrono::Utc::now(),
            description,
        };

        self.points.push_back(point.clone());

        if self.points.len() > self.max_points {
            self.points.pop_front();
        }

        point
    }

    pub fn get_point(&self, id: &str) -> Option<&RevertPoint> {
        self.points.iter().find(|p| p.id == id)
    }

    pub fn list_points(&self) -> Vec<&RevertPoint> {
        self.points.iter().collect()
    }

    pub fn revert_to(
        &self,
        session: &mut crate::Session,
        id: &str,
    ) -> Result<(), crate::OpenCodeError> {
        let point = self.get_point(id).ok_or_else(|| {
            crate::OpenCodeError::Session(format!("Revert point not found: {}", id))
        })?;

        if point.message_index >= session.messages.len() {
            return Err(crate::OpenCodeError::Session(
                "Invalid revert point".to_string(),
            ));
        }

        session.messages.truncate(point.message_index);
        Ok(())
    }
}

impl Default for RevertManager {
    fn default() -> Self {
        Self::new(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Message, Session};

    #[test]
    fn test_revert_manager_new() {
        let rm = RevertManager::new(5);
        assert!(rm.list_points().is_empty());
    }

    #[test]
    fn test_revert_manager_create_point() {
        let mut rm = RevertManager::new(5);
        let point = rm.create_point(0, "Initial".to_string());

        assert!(!point.id.is_empty());
        assert_eq!(point.message_index, 0);
        assert_eq!(point.description, "Initial");
    }

    #[test]
    fn test_revert_manager_get_point() {
        let mut rm = RevertManager::new(5);
        let point = rm.create_point(0, "Test".to_string());
        let id = point.id.clone();

        assert!(rm.get_point(&id).is_some());
    }

    #[test]
    fn test_revert_manager_list_points() {
        let mut rm = RevertManager::new(5);
        rm.create_point(0, "point1".to_string());
        rm.create_point(1, "point2".to_string());

        assert_eq!(rm.list_points().len(), 2);
    }

    #[test]
    fn test_revert_manager_max_points() {
        let mut rm = RevertManager::new(2);
        rm.create_point(0, "p1".to_string());
        rm.create_point(1, "p2".to_string());
        rm.create_point(2, "p3".to_string());

        assert_eq!(rm.list_points().len(), 2);
    }

    #[test]
    fn test_revert_manager_revert_to() {
        let mut rm = RevertManager::new(5);
        let point = rm.create_point(1, "Revert here".to_string());

        let mut session = Session::new();
        session.add_message(Message::user("msg1".to_string()));
        session.add_message(Message::assistant("msg2".to_string()));
        session.add_message(Message::user("msg3".to_string()));

        rm.revert_to(&mut session, &point.id).unwrap();

        assert_eq!(session.messages.len(), 1);
    }
}
