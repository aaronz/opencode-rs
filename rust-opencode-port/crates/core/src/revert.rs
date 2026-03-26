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
