use opencode_core::{Skill, SkillResolver, SkillState};

pub struct SkillsActions {
    resolver: SkillResolver,
}

impl SkillsActions {
    pub fn new(resolver: SkillResolver) -> Self {
        Self { resolver }
    }

    pub fn skills_list(&self) -> Vec<Skill> {
        self.resolver
            .list_skills()
            .unwrap_or_default()
            .into_iter()
            .map(|(skill, _)| skill)
            .collect()
    }

    pub fn skills_enable(&mut self, name: &str) -> Option<Skill> {
        self.resolver.set_skill_state(name, SkillState::Enabled)
    }

    pub fn skills_disable(&mut self, name: &str) -> Option<Skill> {
        self.resolver.set_skill_state(name, SkillState::Disabled)
    }

    pub fn match_and_enable(&mut self, input: &str) -> Vec<Skill> {
        self.resolver.match_and_enable(input)
    }

    pub fn build_skill_prompt(&self) -> String {
        self.resolver.build_skill_prompt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skills_actions_enable_and_disable() {
        let mut actions = SkillsActions::new(SkillResolver::default());
        let enabled = actions.skills_enable("code-review");
        assert!(enabled.is_some());

        let disabled = actions.skills_disable("code-review");
        assert!(disabled.is_some());
    }

    #[test]
    fn skills_actions_list_builtin_skills() {
        let actions = SkillsActions::new(SkillResolver::default());
        let skills = actions.skills_list();
        assert!(!skills.is_empty());
    }
}
