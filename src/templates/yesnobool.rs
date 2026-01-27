use serde::{Deserialize, Serialize};

/// Enum representing a string boolean value with On/Off as values. This functionality probably exists somewhere else, idk.
#[derive(Debug)]
pub enum YesNoBool {
    False = 0,
    True = 1,
}

impl Serialize for YesNoBool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            YesNoBool::False => serializer.serialize_str("no"),
            YesNoBool::True => serializer.serialize_str("yes"),
        }
    }
}

impl<'de> Deserialize<'de> for YesNoBool {
    fn deserialize<'d, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = bool::deserialize(deserializer)?;
        match value {
            false => Ok(YesNoBool::False),
            true => Ok(YesNoBool::True),
        }
    }
}

impl std::fmt::Display for YesNoBool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YesNoBool::False => write!(f, "no"),
            YesNoBool::True => write!(f, "yes"),
        }
    }
}
