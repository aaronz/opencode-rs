pub mod types;

use crate::config::{PermissionAction, PermissionRule};
use crate::skill::Skill;
use crate::OpenCodeError;
use crate::SkillManager;
use opencode_permission::ApprovalResult;
use std::collections::{HashMap, HashSet};

pub use types::SkillState;

pub struct SkillResolver {
    manager: SkillManager,
    states: HashMap<String, SkillState>,
    skill_permission: Option<PermissionRule>,
}

impl SkillResolver {
    pub fn new(manager: SkillManager) -> Self {
        let mut states = HashMap::new();
        if let Ok(skills) = manager.list() {
            for skill in skills {
                states.insert(skill.name, SkillState::AutoMatch);
            }
        }

        Self {
            manager,
            states,
            skill_permission: None,
        }
    }

    pub fn with_permission_rule(mut self, rule: PermissionRule) -> Self {
        self.skill_permission = Some(rule);
        self
    }

    pub fn set_skill_permission(&mut self, rule: Option<PermissionRule>) {
        self.skill_permission = rule;
    }

    pub fn get_skill_permission(&self) -> Option<&PermissionRule> {
        self.skill_permission.as_ref()
    }

    fn check_skill_permission(&self, skill_name: &str) -> ApprovalResult {
        match &self.skill_permission {
            None => ApprovalResult::AutoApprove,
            Some(rule) => evaluate_skill_permission(skill_name, rule),
        }
    }

    pub fn get_enabled_skills(&self) -> Vec<Skill> {
        self.manager
            .list()
            .unwrap_or_default()
            .into_iter()
            .filter(|skill| {
                let state = self
                    .states
                    .get(&skill.name)
                    .copied()
                    .unwrap_or(SkillState::AutoMatch);
                state == SkillState::Enabled
                    && self.check_skill_permission(&skill.name) != ApprovalResult::Denied
            })
            .collect()
    }

    pub fn match_and_enable(&mut self, input: &str) -> Vec<Skill> {
        let mut seen = HashSet::new();
        let mut enabled = Vec::new();

        if let Ok(matches) = self.manager.match_skill(input) {
            for matched in matches {
                if seen.insert(matched.skill.name.clone()) {
                    let permission = self.check_skill_permission(&matched.skill.name);
                    let new_state = match permission {
                        ApprovalResult::AutoApprove => SkillState::Enabled,
                        ApprovalResult::RequireApproval => SkillState::PendingApproval,
                        ApprovalResult::Denied => SkillState::Disabled,
                    };
                    self.states.insert(matched.skill.name.clone(), new_state);
                    if new_state == SkillState::Enabled {
                        enabled.push(matched.skill);
                    }
                }
            }
        }

        enabled
    }

    pub fn approve_skill(&mut self, skill_name: &str) -> Option<Skill> {
        if let Some(state) = self.states.get(skill_name) {
            if *state == SkillState::PendingApproval {
                self.states
                    .insert(skill_name.to_string(), SkillState::Enabled);
                return self.manager.get_skill(skill_name);
            }
        }
        None
    }

    pub fn deny_skill(&mut self, skill_name: &str) -> Option<Skill> {
        if let Some(state) = self.states.get(skill_name) {
            if *state == SkillState::PendingApproval {
                self.states
                    .insert(skill_name.to_string(), SkillState::Disabled);
                return self.manager.get_skill(skill_name);
            }
        }
        None
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

fn evaluate_skill_permission(skill_name: &str, rule: &PermissionRule) -> ApprovalResult {
    match rule {
        PermissionRule::Action(action) => match action {
            PermissionAction::Allow => ApprovalResult::AutoApprove,
            PermissionAction::Ask => ApprovalResult::RequireApproval,
            PermissionAction::Deny => ApprovalResult::Denied,
        },
        PermissionRule::Object(map) => {
            if let Some(action) = map.get(skill_name) {
                match action {
                    PermissionAction::Allow => ApprovalResult::AutoApprove,
                    PermissionAction::Ask => ApprovalResult::RequireApproval,
                    PermissionAction::Deny => ApprovalResult::Denied,
                }
            } else {
                ApprovalResult::RequireApproval
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PermissionAction;
    use std::collections::HashMap;

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

    #[test]
    fn skill_permission_allow_all() {
        let rule = PermissionRule::Action(PermissionAction::Allow);
        assert_eq!(
            evaluate_skill_permission("code-review", &rule),
            ApprovalResult::AutoApprove
        );
    }

    #[test]
    fn skill_permission_deny_all() {
        let rule = PermissionRule::Action(PermissionAction::Deny);
        assert_eq!(
            evaluate_skill_permission("code-review", &rule),
            ApprovalResult::Denied
        );
    }

    #[test]
    fn skill_permission_ask_all() {
        let rule = PermissionRule::Action(PermissionAction::Ask);
        assert_eq!(
            evaluate_skill_permission("code-review", &rule),
            ApprovalResult::RequireApproval
        );
    }

    #[test]
    fn skill_permission_per_skill_allow() {
        let mut map = HashMap::new();
        map.insert("code-review".to_string(), PermissionAction::Allow);
        map.insert("security-auditor".to_string(), PermissionAction::Deny);
        let rule = PermissionRule::Object(map);

        assert_eq!(
            evaluate_skill_permission("code-review", &rule),
            ApprovalResult::AutoApprove
        );
        assert_eq!(
            evaluate_skill_permission("security-auditor", &rule),
            ApprovalResult::Denied
        );
        assert_eq!(
            evaluate_skill_permission("unknown-skill", &rule),
            ApprovalResult::RequireApproval
        );
    }

    #[test]
    fn skill_permission_with_resolver_allow() {
        let rule = PermissionRule::Action(PermissionAction::Allow);
        let resolver = SkillResolver::default().with_permission_rule(rule);
        assert_eq!(
            resolver.check_skill_permission("code-review"),
            ApprovalResult::AutoApprove
        );
    }

    #[test]
    fn skill_permission_with_resolver_deny() {
        let rule = PermissionRule::Action(PermissionAction::Deny);
        let resolver = SkillResolver::default().with_permission_rule(rule);
        assert_eq!(
            resolver.check_skill_permission("code-review"),
            ApprovalResult::Denied
        );
    }

    #[test]
    fn skill_permission_with_resolver_ask() {
        let rule = PermissionRule::Action(PermissionAction::Ask);
        let resolver = SkillResolver::default().with_permission_rule(rule);
        assert_eq!(
            resolver.check_skill_permission("code-review"),
            ApprovalResult::RequireApproval
        );
    }

    #[test]
    fn match_and_enable_respects_deny_permission() {
        let mut resolver = SkillResolver::default();
        resolver.set_skill_permission(Some(PermissionRule::Action(PermissionAction::Deny)));

        let matched = resolver.match_and_enable("code-review");
        assert!(matched.is_empty());
        assert_eq!(
            resolver.skill_state("code-review"),
            Some(SkillState::Disabled)
        );
    }

    #[test]
    fn match_and_enable_respects_ask_permission() {
        let mut resolver = SkillResolver::default();
        resolver.set_skill_permission(Some(PermissionRule::Action(PermissionAction::Ask)));

        let matched = resolver.match_and_enable("code-review");
        assert!(matched.is_empty());
        assert_eq!(
            resolver.skill_state("code-review"),
            Some(SkillState::PendingApproval)
        );
    }

    #[test]
    fn match_and_enable_respects_allow_permission() {
        let mut resolver = SkillResolver::default();
        resolver.set_skill_permission(Some(PermissionRule::Action(PermissionAction::Allow)));

        let matched = resolver.match_and_enable("code-review");
        assert!(!matched.is_empty());
        assert_eq!(
            resolver.skill_state("code-review"),
            Some(SkillState::Enabled)
        );
    }

    #[test]
    fn approve_skill_changes_state_to_enabled() {
        let mut resolver = SkillResolver::default();
        resolver.set_skill_permission(Some(PermissionRule::Action(PermissionAction::Ask)));
        resolver.match_and_enable("code-review");

        assert_eq!(
            resolver.skill_state("code-review"),
            Some(SkillState::PendingApproval)
        );

        let approved = resolver.approve_skill("code-review");
        assert!(approved.is_some());
        assert_eq!(
            resolver.skill_state("code-review"),
            Some(SkillState::Enabled)
        );
    }

    #[test]
    fn deny_skill_changes_state_to_disabled() {
        let mut resolver = SkillResolver::default();
        resolver.set_skill_permission(Some(PermissionRule::Action(PermissionAction::Ask)));
        resolver.match_and_enable("code-review");

        assert_eq!(
            resolver.skill_state("code-review"),
            Some(SkillState::PendingApproval)
        );

        let denied = resolver.deny_skill("code-review");
        assert!(denied.is_some());
        assert_eq!(
            resolver.skill_state("code-review"),
            Some(SkillState::Disabled)
        );
    }

    #[test]
    fn get_enabled_skills_excludes_denied_skills() {
        let mut resolver = SkillResolver::default();
        resolver.set_skill_state("code-review", SkillState::Enabled);
        resolver.set_skill_permission(Some(PermissionRule::Action(PermissionAction::Deny)));

        let enabled = resolver.get_enabled_skills();
        assert!(enabled.iter().all(|s| s.name != "code-review"));
    }

    #[test]
    fn get_enabled_skills_includes_pending_approval() {
        let mut resolver = SkillResolver::default();
        resolver.set_skill_state("code-review", SkillState::PendingApproval);
        resolver.set_skill_permission(Some(PermissionRule::Action(PermissionAction::Ask)));

        let enabled = resolver.get_enabled_skills();
        assert!(enabled.iter().all(|s| s.name != "code-review"));
    }

    #[test]
    fn per_skill_permission_rules() {
        let mut map = HashMap::new();
        map.insert("code-review".to_string(), PermissionAction::Allow);
        map.insert("security-auditor".to_string(), PermissionAction::Deny);
        let rule = PermissionRule::Object(map);

        let mut resolver = SkillResolver::default();
        resolver.set_skill_permission(Some(rule));

        resolver.match_and_enable("code-review");
        resolver.match_and_enable("security-auditor");
        resolver.match_and_enable("refactorer");

        assert_eq!(
            resolver.skill_state("code-review"),
            Some(SkillState::Enabled)
        );
        assert_eq!(
            resolver.skill_state("security-auditor"),
            Some(SkillState::Disabled)
        );
        assert_eq!(
            resolver.skill_state("refactorer"),
            Some(SkillState::PendingApproval)
        );
    }

    #[test]
    fn evaluate_skill_permission_function() {
        assert_eq!(
            evaluate_skill_permission("test", &PermissionRule::Action(PermissionAction::Allow)),
            ApprovalResult::AutoApprove
        );

        assert_eq!(
            evaluate_skill_permission("test", &PermissionRule::Action(PermissionAction::Deny)),
            ApprovalResult::Denied
        );

        assert_eq!(
            evaluate_skill_permission("test", &PermissionRule::Action(PermissionAction::Ask)),
            ApprovalResult::RequireApproval
        );

        let mut map = HashMap::new();
        map.insert("test".to_string(), PermissionAction::Allow);
        assert_eq!(
            evaluate_skill_permission("test", &PermissionRule::Object(map)),
            ApprovalResult::AutoApprove
        );

        let mut map2 = HashMap::new();
        map2.insert("other".to_string(), PermissionAction::Deny);
        assert_eq!(
            evaluate_skill_permission("test", &PermissionRule::Object(map2)),
            ApprovalResult::RequireApproval
        );
    }

    #[test]
    fn skills_permission_restrictions_respect_boundaries() {
        let mut resolver = SkillResolver::default();

        let mut per_skill_rules = HashMap::new();
        per_skill_rules.insert("code-review".to_string(), PermissionAction::Allow);
        per_skill_rules.insert("security-auditor".to_string(), PermissionAction::Deny);
        per_skill_rules.insert("refactorer".to_string(), PermissionAction::Ask);
        let rule = PermissionRule::Object(per_skill_rules);
        resolver.set_skill_permission(Some(rule));

        resolver.match_and_enable("code-review");
        resolver.match_and_enable("security-auditor");
        resolver.match_and_enable("refactorer");
        resolver.match_and_enable("debugger");

        assert_eq!(
            resolver.skill_state("code-review"),
            Some(SkillState::Enabled),
            "Allowed skill should be enabled"
        );
        assert_eq!(
            resolver.skill_state("security-auditor"),
            Some(SkillState::Disabled),
            "Denied skill should be disabled"
        );
        assert_eq!(
            resolver.skill_state("refactorer"),
            Some(SkillState::PendingApproval),
            "Skill requiring approval should be pending"
        );
        assert_eq!(
            resolver.skill_state("debugger"),
            Some(SkillState::PendingApproval),
            "Unknown skill with per-skill rules should require approval"
        );
    }

    #[test]
    fn skills_permission_unauthorized_actions_blocked() {
        let mut resolver = SkillResolver::default();
        resolver.set_skill_permission(Some(PermissionRule::Action(PermissionAction::Deny)));

        let matched = resolver.match_and_enable("code-review");
        assert!(
            matched.is_empty(),
            "No skills should be enabled when all are denied"
        );

        assert_eq!(
            resolver.skill_state("code-review"),
            Some(SkillState::Disabled),
            "Skill should be marked as disabled"
        );

        let enabled = resolver.get_enabled_skills();
        assert!(
            enabled.iter().all(|s| s.name != "code-review"),
            "Denied skill should not appear in enabled skills"
        );
    }

    #[test]
    fn skills_permission_errors_clear_and_actionable() {
        let mut resolver = SkillResolver::default();
        resolver.set_skill_permission(Some(PermissionRule::Action(PermissionAction::Ask)));

        resolver.match_and_enable("code-review");

        let state = resolver.skill_state("code-review");
        assert_eq!(
            state,
            Some(SkillState::PendingApproval),
            "Skill should require approval"
        );

        let approved = resolver.approve_skill("code-review");
        assert!(
            approved.is_some(),
            "approve_skill should return Some when skill is pending"
        );
        assert_eq!(
            resolver.skill_state("code-review"),
            Some(SkillState::Enabled),
            "After approval, skill should be enabled"
        );

        let denied = resolver.deny_skill("code-review");
        assert!(
            denied.is_none(),
            "deny_skill should return None for already approved skill (not PendingApproval)"
        );
        assert_eq!(
            resolver.skill_state("code-review"),
            Some(SkillState::Enabled),
            "Already approved skill should remain enabled after denied call"
        );
    }

    #[test]
    fn skills_permission_deny_overrides_ask_and_allow() {
        let mut resolver = SkillResolver::default();

        let mut per_skill_rules = HashMap::new();
        per_skill_rules.insert("code-review".to_string(), PermissionAction::Allow);
        per_skill_rules.insert("security-auditor".to_string(), PermissionAction::Deny);
        let rule = PermissionRule::Object(per_skill_rules);
        resolver.set_skill_permission(Some(rule));

        resolver.match_and_enable("code-review");
        resolver.match_and_enable("security-auditor");

        assert_eq!(
            resolver.skill_state("code-review"),
            Some(SkillState::Enabled),
            "Explicitly allowed skill should be enabled"
        );
        assert_eq!(
            resolver.skill_state("security-auditor"),
            Some(SkillState::Disabled),
            "Explicitly denied skill should be disabled even if other skills are allowed"
        );

        let enabled = resolver.get_enabled_skills();
        assert!(!enabled.iter().any(|s| s.name == "security-auditor"));
        assert!(enabled.iter().any(|s| s.name == "code-review"));
    }

    #[test]
    fn skills_permission_unknown_skill_requires_approval() {
        let mut resolver = SkillResolver::default();
        resolver.set_skill_permission(Some(PermissionRule::Action(PermissionAction::Ask)));

        resolver.match_and_enable("code-review");

        assert_eq!(
            resolver.skill_state("code-review"),
            Some(SkillState::PendingApproval),
            "Skill should require approval when permission is Ask"
        );
    }

    #[test]
    fn skills_permission_approve_denied_skill_returns_none() {
        let mut resolver = SkillResolver::default();
        resolver.set_skill_permission(Some(PermissionRule::Action(PermissionAction::Deny)));
        resolver.match_and_enable("code-review");

        assert_eq!(
            resolver.skill_state("code-review"),
            Some(SkillState::Disabled)
        );

        let approved = resolver.approve_skill("code-review");
        assert!(
            approved.is_none(),
            "approve_skill should return None for denied skill"
        );
    }

    #[test]
    fn skills_permission_deny_approved_skill_returns_none() {
        let mut resolver = SkillResolver::default();
        resolver.set_skill_permission(Some(PermissionRule::Action(PermissionAction::Allow)));
        resolver.match_and_enable("code-review");

        assert_eq!(
            resolver.skill_state("code-review"),
            Some(SkillState::Enabled)
        );

        let denied = resolver.deny_skill("code-review");
        assert!(
            denied.is_none(),
            "deny_skill should return None for already enabled skill"
        );
    }

    #[test]
    fn skills_permission_boundary_test_case_sensitivity() {
        let mut resolver = SkillResolver::default();

        let mut per_skill_rules = HashMap::new();
        per_skill_rules.insert("Code-Review".to_string(), PermissionAction::Allow);
        per_skill_rules.insert("CODE-REVIEW".to_string(), PermissionAction::Deny);
        let rule = PermissionRule::Object(per_skill_rules);
        resolver.set_skill_permission(Some(rule));

        resolver.match_and_enable("code-review");

        assert_eq!(
            resolver.skill_state("code-review"),
            Some(SkillState::PendingApproval),
            "Case-sensitive matching: 'code-review' should not match 'Code-Review' or 'CODE-REVIEW'"
        );
    }
}