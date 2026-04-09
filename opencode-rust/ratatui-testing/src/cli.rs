use anyhow::Result;

pub struct CliTester;

impl CliTester {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, _args: &[&str]) -> Result<String> {
        Ok(String::new())
    }
}

impl Default for CliTester {
    fn default() -> Self {
        Self::new()
    }
}
