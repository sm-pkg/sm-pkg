use serde::{Deserialize, Serialize};

/// Enum representing an integer boolean value. This functionality probably exists somewhere else, idk.
#[derive(Debug)]
pub enum IntBool {
    False = 0,
    True = 1,
}

impl Serialize for IntBool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            IntBool::False => serializer.serialize_bool(false),
            IntBool::True => serializer.serialize_bool(true),
        }
    }
}

impl<'de> Deserialize<'de> for IntBool {
    fn deserialize<'d, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = bool::deserialize(deserializer)?;
        match value {
            false => Ok(IntBool::False),
            true => Ok(IntBool::True),
        }
    }
}

impl std::fmt::Display for IntBool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntBool::False => write!(f, "0"),
            IntBool::True => write!(f, "1"),
        }
    }
}
