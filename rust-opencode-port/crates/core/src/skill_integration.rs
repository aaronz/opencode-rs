use crate::skill::Skill;
use crate::OpenCodeError;
use crate::SkillManager;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillState {
    Enabled,
    Disabled,
    AutoMatch,
}

pub struct SkillResolver {
    manager: SkillManager,
    states: HashMap<String, SkillState>,
}

impl SkillResolver {
    pub fn new(manager: SkillManager) -> Self {
        let mut states = HashMap::new();
        if let Ok(skills) = manager.list() {
            for skill in skills {
                states.insert(skill.name, SkillState::AutoMatch);
            }
        }

        Self { manager, states }
    }

    pub fn get_enabled_skills(&self) -> Vec<Skill> {
        self.manager
            .list()
            .unwrap_or_default()
            .into_iter()
            .filter(|skill| {
                self.states
                    .get(&skill.name)
                    .copied()
                    .unwrap_or(SkillState::AutoMatch)
                    == SkillState::Enabled
            })
            .collect()
    }

    pub fn match_and_enable(&mut self, input: &str) -> Vec<Skill> {
        let mut seen = HashSet::new();
        let mut enabled = Vec::new();

        if let Ok(matches) = self.manager.match_skill(input) {
            for matched in matches {
                if seen.insert(matched.skill.name.clone()) {
                    self.states
                        .insert(matched.skill.name.clone(), SkillState::Enabled);
                    enabled.push(matched.skill);
                }
            }
        }

        enabled
    }

    pub fn build_skill_prompt(&self) -> String {
        self.get_enabled_skills()
            .iter()
            .map(|skill| self.manager.inject_into_prompt(skill))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    pub fn list_skills(&self) -> Result<Vec<(Skill, SkillState)>, OpenCodeError> {
        let skills = self.manager.list()?;
        Ok(skills
            .into_iter()
            .map(|skill| {
                let state = self
                    .states
                    .get(&skill.name)
                    .copied()
                    .unwrap_or(SkillState::AutoMatch);
                (skill, state)
            })
            .collect())
    }

    pub fn set_skill_state(&mut self, name: &str, state: SkillState) -> Option<Skill> {
        let skill = self.manager.match_by_skill_name(name)?;
        self.states.insert(skill.name.clone(), state);
        Some(skill)
    }

    pub fn skill_state(&self, name: &str) -> Option<SkillState> {
        let skill = self.manager.match_by_skill_name(name)?;
        Some(
            self.states
                .get(&skill.name)
                .copied()
                .unwrap_or(SkillState::AutoMatch),
        )
    }

    pub fn manager(&self) -> &SkillManager {
        &self.manager
    }
}

impl Default for SkillResolver {
    fn default() -> Self {
        Self::new(SkillManager::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolver_matches_and_enables_skills() {
        let mut resolver = SkillResolver::default();
        let matched = resolver.match_and_enable("code-review");

        assert!(!matched.is_empty());
        assert!(resolver
            .get_enabled_skills()
            .iter()
            .any(|skill| skill.name == "code-review"));
    }

    #[test]
    fn resolver_builds_prompt_from_enabled_skills() {
        let mut resolver = SkillResolver::default();
        resolver.set_skill_state("code-review", SkillState::Enabled);

        let prompt = resolver.build_skill_prompt();

        assert!(!prompt.is_empty());
        assert!(prompt.contains("Skill:"));
    }

    #[test]
    fn resolver_tracks_explicit_state_changes() {
        let mut resolver = SkillResolver::default();
        let skill = resolver
            .set_skill_state("debugger", SkillState::Disabled)
            .expect("debugger should exist");

        assert_eq!(skill.name, "debugger");
        assert_eq!(resolver.skill_state("debugger"), Some(SkillState::Disabled));
    }
}
