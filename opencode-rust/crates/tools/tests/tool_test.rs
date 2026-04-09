#[cfg(test)]
mod tests {
    #[test]
    fn test_skill_discovery() {
        let skills = opencode_core::skill::SkillManager::new();
        let _list = skills.list();
    }
}
