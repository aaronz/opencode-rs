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
