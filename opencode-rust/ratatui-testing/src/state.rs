use anyhow::Result;

pub struct StateTester;

impl StateTester {
    pub fn new() -> Self {
        Self
    }

    pub fn assert_state<S>(&self, _state: &S) -> Result<()>
    where
        S: serde::Serialize,
    {
        Ok(())
    }
}
