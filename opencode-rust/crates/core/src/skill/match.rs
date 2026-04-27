//! Skill matching types.

use crate::skill::Skill;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MatchType {
    Exact,
    Prefix,
    Fuzzy,
    Semantic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMatch {
    pub skill: Skill,
    pub match_type: MatchType,
    pub confidence: f64,
}
