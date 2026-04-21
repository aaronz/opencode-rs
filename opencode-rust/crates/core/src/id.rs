use std::str::FromStr;

use thiserror::Error;
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

#[derive(Error, Debug)]
pub enum IdParseError {
    #[error("Invalid UUID format: {0}")]
    InvalidUuid(#[from] uuid::Error),
    #[error("Invalid integer format: {0}")]
    InvalidInt(std::num::ParseIntError),
}

macro_rules! define_id_newtype {
    ($name:ident, $prefix:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}{}", $prefix, self.0)
            }
        }

        impl FromStr for $name {
            type Err = IdParseError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let s = s.strip_prefix($prefix).unwrap_or(s);
                let uuid = Uuid::from_str(s)?;
                Ok(Self(uuid))
            }
        }
    };
}

define_id_newtype!(SessionId, "session:");
define_id_newtype!(UserId, "user:");
define_id_newtype!(ProjectId, "project:");

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

    #[test]
    fn test_session_id_new() {
        let id = SessionId::new();
        assert!(!id.0.is_nil());
    }

    #[test]
    fn test_user_id_new() {
        let id = UserId::new();
        assert!(!id.0.is_nil());
    }

    #[test]
    fn test_project_id_new() {
        let id = ProjectId::new();
        assert!(!id.0.is_nil());
    }

    #[test]
    fn test_session_id_display() {
        let id = SessionId::new();
        let display = id.to_string();
        assert!(display.starts_with("session:"));
        assert_eq!(display.len(), 8 + 36);
    }

    #[test]
    fn test_user_id_display() {
        let id = UserId::new();
        let display = id.to_string();
        assert!(display.starts_with("user:"));
        assert_eq!(display.len(), 5 + 36);
    }

    #[test]
    fn test_project_id_display() {
        let id = ProjectId::new();
        let display = id.to_string();
        assert!(display.starts_with("project:"));
        assert_eq!(display.len(), 8 + 36);
    }

    #[test]
    fn test_session_id_from_str() {
        let id = SessionId::new();
        let parsed: SessionId = id.to_string().parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_user_id_from_str() {
        let id = UserId::new();
        let parsed: UserId = id.to_string().parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_project_id_from_str() {
        let id = ProjectId::new();
        let parsed: ProjectId = id.to_string().parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_session_id_from_str_without_prefix() {
        let id = SessionId::new();
        let uuid_str = id.0.to_string();
        let parsed: SessionId = uuid_str.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_user_id_from_str_without_prefix() {
        let id = UserId::new();
        let uuid_str = id.0.to_string();
        let parsed: UserId = uuid_str.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_project_id_from_str_without_prefix() {
        let id = ProjectId::new();
        let uuid_str = id.0.to_string();
        let parsed: ProjectId = uuid_str.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_newtype_invalid_session_id_parse() {
        let result: Result<SessionId, _> = "not-a-valid-uuid".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_newtype_invalid_user_id_parse() {
        let result: Result<UserId, _> = "invalid".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_newtype_invalid_project_id_parse() {
        let result: Result<ProjectId, _> = "12345".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_same_session_ids_equal() {
        let sid1 = SessionId::new();
        let sid2 = sid1;
        assert_eq!(sid1, sid2);
    }

    #[test]
    fn test_different_session_ids_not_equal() {
        let sid1 = SessionId::new();
        let sid2 = SessionId::new();
        assert_ne!(sid1, sid2);
    }

    #[test]
    fn test_newtypes_prevent_type_confusion_compile_time() {
        fn takes_session_id(_id: SessionId) {}
        fn takes_user_id(_id: UserId) {}
        fn takes_project_id(_id: ProjectId) {}

        let sid = SessionId::new();
        let uid = UserId::new();
        let pid = ProjectId::new();

        takes_session_id(sid);
        takes_user_id(uid);
        takes_project_id(pid);
    }

    #[test]
    fn test_newtypes_prevent_cross_type_parsing() {
        let session_id = SessionId::new();
        let session_id_str = session_id.to_string();

        let user_id_result: Result<UserId, _> = session_id_str.parse();
        assert!(
            user_id_result.is_err(),
            "Should not be able to parse SessionId string as UserId"
        );

        let project_id_result: Result<ProjectId, _> = session_id_str.parse();
        assert!(
            project_id_result.is_err(),
            "Should not be able to parse SessionId string as ProjectId"
        );

        let user_id = UserId::new();
        let user_id_str = user_id.to_string();

        let session_id_result: Result<SessionId, _> = user_id_str.parse();
        assert!(
            session_id_result.is_err(),
            "Should not be able to parse UserId string as SessionId"
        );

        let project_id_result: Result<ProjectId, _> = user_id_str.parse();
        assert!(
            project_id_result.is_err(),
            "Should not be able to parse UserId string as ProjectId"
        );
    }

    #[test]
    fn test_session_id_debug() {
        let id = SessionId::new();
        let debug = format!("{:?}", id);
        assert!(debug.contains("SessionId"));
    }

    #[test]
    fn test_user_id_debug() {
        let id = UserId::new();
        let debug = format!("{:?}", id);
        assert!(debug.contains("UserId"));
    }

    #[test]
    fn test_project_id_debug() {
        let id = ProjectId::new();
        let debug = format!("{:?}", id);
        assert!(debug.contains("ProjectId"));
    }

    #[test]
    fn test_session_id_copy() {
        let id1 = SessionId::new();
        let id2 = id1;
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_user_id_copy() {
        let id1 = UserId::new();
        let id2 = id1;
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_project_id_copy() {
        let id1 = ProjectId::new();
        let id2 = id1;
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_session_id_default() {
        let id: SessionId = Default::default();
        assert!(!id.0.is_nil());
    }

    #[test]
    fn test_user_id_default() {
        let id: UserId = Default::default();
        assert!(!id.0.is_nil());
    }

    #[test]
    fn test_project_id_default() {
        let id: ProjectId = Default::default();
        assert!(!id.0.is_nil());
    }

    #[test]
    fn test_session_id_ordering() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        // Direct Ord comparison (now that Ord is derived)
        let _ord = id1.cmp(&id2);
        // Also verify PartialOrd works
        assert!(id1 <= id2 || id1 >= id2);
    }

    #[test]
    fn test_user_id_ordering() {
        let id1 = UserId::new();
        let id2 = UserId::new();
        let _ord = id1.cmp(&id2);
        assert!(id1 <= id2 || id1 >= id2);
    }

    #[test]
    fn test_project_id_ordering() {
        let id1 = ProjectId::new();
        let id2 = ProjectId::new();
        let _ord = id1.cmp(&id2);
        assert!(id1 <= id2 || id1 >= id2);
    }

    #[test]
    fn test_session_id_ordering_consistent_with_uuid() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        // Ordering should be consistent with underlying UUID ordering
        let id_order = id1.cmp(&id2);
        let uuid_order = id1.0.cmp(&id2.0);
        assert_eq!(id_order, uuid_order);
    }

    #[test]
    fn test_ids_sortable_in_btree_set() {
        use std::collections::BTreeSet;
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        let id3 = SessionId::new();

        let mut set: BTreeSet<SessionId> = BTreeSet::new();
        set.insert(id3);
        set.insert(id1.clone());
        set.insert(id2.clone());

        // Should be sortable without custom comparator
        let mut iter = set.iter();
        let first = iter.next().unwrap();
        let second = iter.next().unwrap();
        let third = iter.next().unwrap();
        assert!(first <= second && second <= third);
    }

    #[test]
    fn test_ids_sortable_in_btree_map() {
        use std::collections::BTreeMap;
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        let id3 = SessionId::new();

        let mut map: BTreeMap<SessionId, String> = BTreeMap::new();
        map.insert(id3.clone(), "third".to_string());
        map.insert(id1.clone(), "first".to_string());
        map.insert(id2.clone(), "second".to_string());

        // Keys should be stored in sorted order in BTreeMap
        let mut iter = map.keys();
        let first = *iter.next().unwrap();
        let second = *iter.next().unwrap();
        let third = *iter.next().unwrap();
        // Verify the IDs are in sorted order (consistent with Ord)
        assert!(first < second && second < third);
    }

    #[test]
    fn test_id_parse_error_display() {
        use std::num::NonZeroU64;
        let err = IdParseError::InvalidInt("not a number".parse::<NonZeroU64>().unwrap_err());
        assert!(err.to_string().contains("Invalid integer format"));
    }
}
