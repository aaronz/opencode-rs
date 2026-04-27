#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillState {
    Enabled,
    Disabled,
    AutoMatch,
    PendingApproval,
}