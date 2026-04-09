use anyhow::Result;

pub struct TestDsl;

impl TestDsl {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, _widget: impl std::fmt::Debug) -> Result<()> {
        Ok(())
    }
}

impl Default for TestDsl {
    fn default() -> Self {
        Self::new()
    }
}
