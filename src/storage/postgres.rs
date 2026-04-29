use crate::error::StorageError;
use crate::models::{Schema, SchemaType};
use async_trait::async_trait;
use sqlx::PgPool;

use super::SchemaStore;

#[derive(Clone)]
pub struct PostgresSchemaStore {
    pool: PgPool,
}

impl PostgresSchemaStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SchemaStore for PostgresSchemaStore {
    async fn get_schema_by_id(&self, id: i32) -> Result<Schema, StorageError> {
        let row = sqlx::query!(
            r#"
            SELECT s.id, subj.name as subject, s.version, s.schema_text as schema,
                   s.schema_type, s.references
            FROM schemas s
            JOIN subjects subj ON s.subject_id = subj.id
            WHERE s.id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        let row = row.ok_or(StorageError::NotFound)?;

        Ok(Schema {
            id: row.id,
            subject: row.subject,
            version: row.version,
            schema: row.schema,
            references: row.references.as_ref().and_then(|r| serde_json::from_str(r).ok()),
            schema_type: row.schema_type.parse().unwrap(),
        })
    }

    async fn register_schema(
        &self,
        subject: &str,
        schema_str: &str,
        schema_type: SchemaType,
    ) -> Result<i32, StorageError> {
        let mut tx = self.pool.begin().await?;

        // Ensure subject exists
        let subject_row = sqlx::query!(
            "INSERT INTO subjects (name) VALUES ($1) ON CONFLICT (name) DO UPDATE SET name = $1 RETURNING id",
            subject
        )
        .fetch_one(&mut *tx)
        .await?;

        let subject_id = subject_row.id;

        // Get next version for this subject
        let version_row = sqlx::query!("SELECT COALESCE(MAX(version), 0) + 1 as next_version FROM schemas WHERE subject_id = $1", subject_id)
            .fetch_one(&mut *tx)
            .await?;

        let next_version = version_row.next_version.unwrap_or(1);

        // Insert new schema
        let schema_row = sqlx::query!(
            "INSERT INTO schemas (subject_id, version, schema_text, schema_type) VALUES ($1, $2, $3, $4) RETURNING id",
            subject_id,
            next_version,
            schema_str,
            schema_type.as_str()
        )
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(schema_row.id)
    }

    async fn get_subject_versions(&self, subject: &str) -> Result<Vec<i32>, StorageError> {
        let rows = sqlx::query!(
            "SELECT s.version FROM schemas s JOIN subjects subj ON s.subject_id = subj.id WHERE subj.name = $1 ORDER BY s.version ASC",
            subject
        )
        .fetch_all(&self.pool)
        .await?;

        if rows.is_empty() {
            return Err(StorageError::NotFound);
        }

        Ok(rows.into_iter().map(|r| r.version).collect())
    }

    async fn get_schema_by_version(
        &self,
        subject: &str,
        version: i32,
    ) -> Result<Schema, StorageError> {
        let row = sqlx::query!(
            r#"
            SELECT s.id, subj.name as subject, s.version, s.schema_text as schema,
                   s.schema_type, s.references
            FROM schemas s
            JOIN subjects subj ON s.subject_id = subj.id
            WHERE subj.name = $1 AND s.version = $2
            "#,
            subject,
            version
        )
        .fetch_optional(&self.pool)
        .await?;

        let row = row.ok_or(StorageError::NotFound)?;

        Ok(Schema {
            id: row.id,
            subject: row.subject,
            version: row.version,
            schema: row.schema,
            references: row.references.as_ref().and_then(|r| serde_json::from_str(r).ok()),
            schema_type: row.schema_type.parse().unwrap(),
        })
    }

    async fn list_subjects(&self) -> Result<Vec<String>, StorageError> {
        let rows = sqlx::query!("SELECT DISTINCT name FROM subjects ORDER BY name ASC")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|r| r.name).collect())
    }

    async fn get_latest_version(&self, subject: &str) -> Result<i32, StorageError> {
        let row = sqlx::query!(
            "SELECT MAX(s.version) as max_version FROM schemas s JOIN subjects subj ON s.subject_id = subj.id WHERE subj.name = $1",
            subject
        )
        .fetch_one(&self.pool)
        .await?;

        row.max_version.ok_or(StorageError::NotFound)
    }
}
