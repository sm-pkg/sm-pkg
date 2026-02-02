use serde::{Deserialize, Serialize};

/// Enum representing a string boolean value with On/Off as values. This functionality probably exists somewhere else, idk.
#[derive(Debug)]
pub enum OnOffBool {
    False = 0,
    True = 1,
}

impl Serialize for OnOffBool {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            OnOffBool::False => serializer.serialize_str("off"),
            OnOffBool::True => serializer.serialize_str("on"),
        }
    }
}

impl<'de> Deserialize<'de> for OnOffBool {
    fn deserialize<'d, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = bool::deserialize(deserializer)?;
        match value {
            false => Ok(OnOffBool::False),
            true => Ok(OnOffBool::True),
        }
    }
}

impl std::fmt::Display for OnOffBool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OnOffBool::False => write!(f, "off"),
            OnOffBool::True => write!(f, "On"),
        }
    }
}
