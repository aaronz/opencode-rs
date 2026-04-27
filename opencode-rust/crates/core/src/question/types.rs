use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    pub question: String,
    pub options: Vec<String>,
    pub answer: Option<String>,
}

pub struct QuestionManager {
    pub(crate) questions: Vec<Question>,
}

impl QuestionManager {
    pub fn new() -> Self {
        Self {
            questions: Vec::new(),
        }
    }

    pub fn ask(&mut self, question: String, options: Vec<String>) -> String {
        let q = Question {
            id: uuid::Uuid::new_v4().to_string(),
            question,
            options,
            answer: None,
        };
        let id = q.id.clone();
        self.questions.push(q);
        id
    }

    pub fn answer(&mut self, id: &str, answer: String) {
        if let Some(q) = self.questions.iter_mut().find(|q| q.id == id) {
            q.answer = Some(answer);
        }
    }

    pub fn get(&self, id: &str) -> Option<&Question> {
        self.questions.iter().find(|q| q.id == id)
    }
}

impl Default for QuestionManager {
    fn default() -> Self {
        Self::new()
    }
}