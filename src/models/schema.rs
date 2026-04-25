use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub id: i32,
    pub subject: String,
    pub version: i32,
    pub schema: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<Vec<String>>,
    pub schema_type: SchemaType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum SchemaType {
    Avro,
    Protobuf,
    Json,
}

#[derive(Debug, Serialize)]
pub struct SchemaResponse {
    pub subject: String,
    pub version: i32,
    pub id: i32,
    pub schema: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterSchemaRequest {
    pub schema: String,
    #[serde(default)]
    pub schema_type: Option<String>,
}

impl SchemaType {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "PROTOBUF" => SchemaType::Protobuf,
            "JSON" => SchemaType::Json,
            _ => SchemaType::Avro,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            SchemaType::Avro => "AVRO",
            SchemaType::Protobuf => "PROTOBUF",
            SchemaType::Json => "JSON",
        }
    }
}
