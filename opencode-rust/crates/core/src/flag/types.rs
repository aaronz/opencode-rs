use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

#[allow(dead_code)]
pub(crate) fn truthy(key: &str) -> bool {
    env::var(key)
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Flag {
    pub name: String,
    pub description: String,
    pub default: bool,
    pub value: bool,
}

#[allow(dead_code)]
pub(crate) struct FlagManager {
    pub(crate) flags: HashMap<String, Flag>,
    pub(crate) string_flags: HashMap<String, Option<String>>,
    pub(crate) number_flags: HashMap<String, Option<u64>>,
}
