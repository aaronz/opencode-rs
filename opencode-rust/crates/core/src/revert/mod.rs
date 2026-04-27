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

    #[test]
    fn test_revert_manager_zero_max_points() {
        let mut rm = RevertManager::new(0);
        let point = rm.create_point(0, "First".to_string());

        let points = rm.list_points();
        assert!(points.is_empty());

        assert!(rm.get_point(&point.id).is_none());
    }

    #[test]
    fn test_revert_manager_revert_to_index_0() {
        let mut rm = RevertManager::new(5);
        let point = rm.create_point(0, "Revert to start".to_string());

        let mut session = Session::new();
        session.add_message(Message::user("msg1".to_string()));
        session.add_message(Message::assistant("msg2".to_string()));
        session.add_message(Message::user("msg3".to_string()));

        rm.revert_to(&mut session, &point.id).unwrap();

        assert!(session.messages.is_empty());
    }

    #[test]
    fn test_revert_manager_revert_to_last_message() {
        let mut rm = RevertManager::new(5);
        let point = rm.create_point(2, "Revert to last".to_string());

        let mut session = Session::new();
        session.add_message(Message::user("msg1".to_string()));
        session.add_message(Message::assistant("msg2".to_string()));
        session.add_message(Message::user("msg3".to_string()));

        rm.revert_to(&mut session, &point.id).unwrap();

        assert_eq!(session.messages.len(), 2);
    }

    #[test]
    fn test_revert_manager_revert_to_out_of_bounds() {
        let mut rm = RevertManager::new(5);
        let point = rm.create_point(100, "Out of bounds".to_string());

        let mut session = Session::new();
        session.add_message(Message::user("msg1".to_string()));
        session.add_message(Message::assistant("msg2".to_string()));

        let result = rm.revert_to(&mut session, &point.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_revert_manager_empty_description() {
        let mut rm = RevertManager::new(5);
        let point = rm.create_point(0, "".to_string());

        assert_eq!(point.description, "");
        let found = rm.get_point(&point.id).unwrap();
        assert_eq!(found.description, "");
    }

    #[test]
    fn test_revert_manager_multiple_points_same_index() {
        let mut rm = RevertManager::new(5);

        let point1 = rm.create_point(1, "First point at 1".to_string());
        let point2 = rm.create_point(1, "Second point at 1".to_string());
        let point3 = rm.create_point(1, "Third point at 1".to_string());

        let points = rm.list_points();
        assert_eq!(points.len(), 3);

        assert_eq!(points[0].message_index, 1);
        assert_eq!(points[1].message_index, 1);
        assert_eq!(points[2].message_index, 1);

        assert_ne!(point1.id, point2.id);
        assert_ne!(point2.id, point3.id);
    }

    #[test]
    fn test_revert_manager_revert_empty_session() {
        let mut rm = RevertManager::new(5);
        let point = rm.create_point(0, "Empty session".to_string());

        let mut session = Session::new();

        let result = rm.revert_to(&mut session, &point.id);
        assert!(result.is_err());
    }

    #[test]
    fn test_revert_manager_revert_then_add_messages() {
        let mut rm = RevertManager::new(5);

        let point = rm.create_point(1, "Checkpoint".to_string());

        let mut session = Session::new();
        session.add_message(Message::user("msg1".to_string()));
        session.add_message(Message::assistant("msg2".to_string()));
        session.add_message(Message::user("msg3".to_string()));

        rm.revert_to(&mut session, &point.id).unwrap();
        assert_eq!(session.messages.len(), 1);

        session.add_message(Message::assistant("msg4".to_string()));
        session.add_message(Message::user("msg5".to_string()));

        assert_eq!(session.messages.len(), 3);
        assert_eq!(session.messages[0].content, "msg1");
        assert_eq!(session.messages[1].content, "msg4");
        assert_eq!(session.messages[2].content, "msg5");
    }

    #[test]
    fn test_revert_manager_get_nonexistent_point() {
        let rm = RevertManager::new(5);
        let result = rm.get_point("nonexistent-id");
        assert!(result.is_none());
    }

    #[test]
    fn test_revert_manager_points_fifo_eviction() {
        let mut rm = RevertManager::new(3);

        let point1 = rm.create_point(0, "First".to_string());
        let point2 = rm.create_point(1, "Second".to_string());
        let point3 = rm.create_point(2, "Third".to_string());
        let point4 = rm.create_point(3, "Fourth".to_string());

        assert!(rm.get_point(&point1.id).is_none());
        assert!(rm.get_point(&point2.id).is_some());
        assert!(rm.get_point(&point3.id).is_some());
        assert!(rm.get_point(&point4.id).is_some());

        let points = rm.list_points();
        assert_eq!(points.len(), 3);
        assert_eq!(points[0].id, point2.id);
        assert_eq!(points[1].id, point3.id);
        assert_eq!(points[2].id, point4.id);
    }

    #[test]
    fn test_revert_manager_id_is_uuid_format() {
        let mut rm = RevertManager::new(5);
        let point = rm.create_point(0, "Test".to_string());

        assert_eq!(point.id.len(), 36);
        assert!(point.id.chars().all(|c| c.is_ascii_hexdigit() || c == '-'));
    }

    #[test]
    fn test_revert_manager_timestamp_is_set() {
        let mut rm = RevertManager::new(5);
        let before = chrono::Utc::now();
        let point = rm.create_point(0, "Test".to_string());
        let after = chrono::Utc::now();

        assert!(point.timestamp >= before);
        assert!(point.timestamp <= after);
    }

    #[test]
    fn test_revert_manager_revert_preserves_messages_before_index() {
        let mut rm = RevertManager::new(5);
        let point = rm.create_point(1, "Revert to middle".to_string());

        let mut session = Session::new();
        session.add_message(Message::user("msg1".to_string()));
        session.add_message(Message::assistant("msg2".to_string()));
        session.add_message(Message::user("msg3".to_string()));
        session.add_message(Message::assistant("msg4".to_string()));

        rm.revert_to(&mut session, &point.id).unwrap();

        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].content, "msg1");
    }

    #[test]
    fn test_revert_manager_single_point_behavior() {
        let mut rm = RevertManager::new(1);
        let point1 = rm.create_point(0, "Only point".to_string());

        let points = rm.list_points();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].id, point1.id);

        let point2 = rm.create_point(1, "New only point".to_string());

        let points = rm.list_points();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].id, point2.id);
        assert!(rm.get_point(&point1.id).is_none());
    }

    #[test]
    fn test_revert_manager_max_points_exact() {
        let mut rm = RevertManager::new(3);

        rm.create_point(0, "p0".to_string());
        rm.create_point(1, "p1".to_string());
        rm.create_point(2, "p2".to_string());

        let points = rm.list_points();
        assert_eq!(points.len(), 3);

        rm.create_point(3, "p3".to_string());

        let points = rm.list_points();
        assert_eq!(points.len(), 3);
    }

    #[test]
    fn test_revert_manager_revert_to_error_message() {
        let mut rm = RevertManager::new(5);
        let point = rm.create_point(100, "Way out of bounds".to_string());

        let mut session = Session::new();
        session.add_message(Message::user("only one".to_string()));

        let result = rm.revert_to(&mut session, &point.id);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, crate::OpenCodeError::Session(_)));
    }

    #[test]
    fn test_revert_manager_all_points_have_unique_ids() {
        let mut rm = RevertManager::new(10);
        let ids: Vec<String> = (0..10)
            .map(|i| {
                let point = rm.create_point(i, format!("point{}", i));
                point.id
            })
            .collect();

        let mut unique_ids = ids.clone();
        unique_ids.sort();
        unique_ids.dedup();
        assert_eq!(unique_ids.len(), ids.len());
    }
}
