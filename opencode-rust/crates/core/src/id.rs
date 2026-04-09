use uuid::Uuid;

pub struct IdGenerator;

impl IdGenerator {
    pub fn new_uuid() -> String {
        Uuid::new_v4().to_string()
    }

    pub fn new_short() -> String {
        let uuid = Uuid::new_v4();
        uuid.to_string()[..8].to_string()
    }

    pub fn new_timestamped() -> String {
        let now = chrono::Utc::now();
        let uuid = Uuid::new_v4();
        format!("{}-{}", now.timestamp(), &uuid.to_string()[..8])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_uuid() {
        let id = IdGenerator::new_uuid();
        assert_eq!(id.len(), 36);
    }

    #[test]
    fn test_new_short() {
        let id = IdGenerator::new_short();
        assert_eq!(id.len(), 8);
    }

    #[test]
    fn test_new_timestamped() {
        let id = IdGenerator::new_timestamped();
        assert!(id.contains('-'));
    }
}
