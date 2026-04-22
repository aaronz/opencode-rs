pub struct FormatService;

impl FormatService {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_service_creates() {
        let service = FormatService::new();
        assert!(true);
    }
}