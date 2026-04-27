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
