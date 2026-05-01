mod types;

pub use types::{IdGenerator, IdParseError, ProjectId, SessionId, UserId, WorkspaceId};

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
        let _ord = id1.cmp(&id2);
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
        set.insert(id1);
        set.insert(id2);

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
        map.insert(id3, "third".to_string());
        map.insert(id1, "first".to_string());
        map.insert(id2, "second".to_string());

        let mut iter = map.keys();
        let first = *iter.next().unwrap();
        let second = *iter.next().unwrap();
        let third = *iter.next().unwrap();
        assert!(first < second && second < third);
    }

    #[test]
    fn test_id_parse_error_display() {
        use std::num::NonZeroU64;
        let err = IdParseError::InvalidInt("not a number".parse::<NonZeroU64>().unwrap_err());
        assert!(err.to_string().contains("Invalid integer format"));
    }

    #[tokio::test]
    async fn test_id_collision_resistance_under_load() {
        use std::collections::HashSet;
        use tokio::task::JoinSet;

        let num_threads = 100;
        let ids_per_thread = 1000;
        let mut join_set = JoinSet::new();

        for _ in 0..num_threads {
            join_set.spawn(async move {
                let mut ids = Vec::with_capacity(ids_per_thread);
                for _ in 0..ids_per_thread {
                    ids.push(IdGenerator::new_uuid());
                }
                ids
            });
        }

        let mut all_ids = Vec::with_capacity(num_threads * ids_per_thread);
        while let Some(result) = join_set.join_next().await {
            all_ids.extend(result.unwrap());
        }

        let unique_ids: HashSet<_> = all_ids.iter().collect();
        assert_eq!(
            unique_ids.len(),
            num_threads * ids_per_thread,
            "ID collision detected - {} duplicates found",
            num_threads * ids_per_thread - unique_ids.len()
        );
    }

    #[test]
    fn test_id_uniqueness_1000_generations() {
        use std::collections::HashSet;

        let mut ids = Vec::with_capacity(1000);
        for _ in 0..1000 {
            ids.push(IdGenerator::new_uuid());
        }

        let unique_ids: HashSet<_> = ids.iter().collect();
        assert_eq!(unique_ids.len(), 1000, "Collision in 1000 ID generations");
    }

    #[test]
    fn test_session_id_uniqueness() {
        use std::collections::HashSet;

        let mut ids = Vec::with_capacity(1000);
        for _ in 0..1000 {
            ids.push(SessionId::new());
        }

        let unique_ids: HashSet<_> = ids.iter().collect();
        assert_eq!(
            unique_ids.len(),
            1000,
            "Collision in 1000 SessionId generations"
        );
    }

    #[test]
    fn test_id_format_immutability() {
        let id1 = IdGenerator::new_uuid();
        let id2 = IdGenerator::new_uuid();

        assert_eq!(id1.len(), 36, "UUID format should always be 36 characters");
        assert_eq!(id2.len(), 36);

        assert!(
            id1.chars().filter(|c| *c == '-').count() == 4,
            "UUID should have 4 hyphens"
        );
        assert!(
            id2.chars().filter(|c| *c == '-').count() == 4,
            "UUID should have 4 hyphens"
        );

        let short1 = IdGenerator::new_short();
        assert_eq!(short1.len(), 8, "Short ID should be 8 characters");
    }
}
