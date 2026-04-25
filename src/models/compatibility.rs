use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Serialize)]
pub struct CompatibilityCheckResponse {
    pub is_compatible: bool,
}

impl CompatibilityLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "FORWARD" => CompatibilityLevel::Forward,
            "FULL" => CompatibilityLevel::Full,
            "NONE" => CompatibilityLevel::None,
            _ => CompatibilityLevel::Backward,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            CompatibilityLevel::Backward => "BACKWARD",
            CompatibilityLevel::Forward => "FORWARD",
            CompatibilityLevel::Full => "FULL",
            CompatibilityLevel::None => "NONE",
        }
    }
}
