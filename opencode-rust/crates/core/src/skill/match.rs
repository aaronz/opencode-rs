//! Skill matching types.

use serde::{Deserialize, Serialize};
use crate::skill::Skill;

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
