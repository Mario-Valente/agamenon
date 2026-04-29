use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum CompatibilityLevel {
    Backward,
    Forward,
    Full,
    None,
}

#[derive(Debug, Deserialize)]
pub struct CompatibilityCheckRequest {
    pub schema: String,
    #[serde(default)]
    pub compatibility: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CompatibilityCheckResponse {
    pub is_compatible: bool,
}

impl FromStr for CompatibilityLevel {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_uppercase().as_str() {
            "FORWARD" => CompatibilityLevel::Forward,
            "FULL" => CompatibilityLevel::Full,
            "NONE" => CompatibilityLevel::None,
            _ => CompatibilityLevel::Backward,
        })
    }
}

impl CompatibilityLevel {
    pub fn as_str(&self) -> &str {
        match self {
            CompatibilityLevel::Backward => "BACKWARD",
            CompatibilityLevel::Forward => "FORWARD",
            CompatibilityLevel::Full => "FULL",
            CompatibilityLevel::None => "NONE",
        }
    }
}
