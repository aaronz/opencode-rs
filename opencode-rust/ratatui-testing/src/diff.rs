use anyhow::Result;

pub struct BufferDiff;

impl BufferDiff {
    pub fn new() -> Self {
        Self
    }

    pub fn diff(&self, _expected: &str, _actual: &str) -> Result<String> {
        Ok(String::new())
    }
}

impl Default for BufferDiff {
    fn default() -> Self {
        Self::new()
    }
}
