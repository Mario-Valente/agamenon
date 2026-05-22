use crate::error::StorageError;
use crate::models::{Schema, SchemaType};
use async_trait::async_trait;

pub mod postgres;
pub mod s3;

pub use postgres::PostgresSchemaStore;
pub use s3::S3SchemaStore;

#[async_trait]
pub trait SchemaStore: Send + Sync {
    /// Fetch schema by global ID
    async fn get_schema_by_id(&self, id: i32) -> Result<Schema, StorageError>;

    /// Register a new schema and return its global ID
    async fn register_schema(
        &self,
        subject: &str,
        schema_str: &str,
        schema_type: SchemaType,
    ) -> Result<i32, StorageError>;

    /// Get all versions of a subject
    async fn get_subject_versions(&self, subject: &str) -> Result<Vec<i32>, StorageError>;

    /// Get schema by subject and version
    async fn get_schema_by_version(
        &self,
        subject: &str,
        version: i32,
    ) -> Result<Schema, StorageError>;

    /// List all subjects
    async fn list_subjects(&self) -> Result<Vec<String>, StorageError>;

    /// Get the latest version of a subject
    async fn get_latest_version(&self, subject: &str) -> Result<i32, StorageError>;

    /// Lookup schema by body under a subject
    async fn lookup_schema(&self, subject: &str, schema_str: &str) -> Result<Schema, StorageError>;
}
