use anyhow::Result;

pub struct PtySimulator;

impl PtySimulator {
    pub fn new() -> Self {
        Self
    }

    pub fn write_input(&mut self, _input: &str) -> Result<()> {
        Ok(())
    }

    pub fn read_output(&mut self) -> Result<String> {
        Ok(String::new())
    }
}

impl Default for PtySimulator {
    fn default() -> Self {
        Self::new()
    }
}
