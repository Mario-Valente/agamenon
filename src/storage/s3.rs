use crate::error::StorageError;
use crate::models::{Schema, SchemaType};
use async_trait::async_trait;
use aws_sdk_s3::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

use super::SchemaStore;

/// Schema entry stored in S3
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SchemaEntry {
    id: i32,
    subject: String,
    version: i32,
    schema: String,
    schema_type: String,
    references: Option<Vec<String>>,
}

/// Subject metadata stored in S3
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SubjectMetadata {
    versions: Vec<i32>,
    next_id: i32,
}

/// Global index stored in S3
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct GlobalIndex {
    next_schema_id: i32,
    subjects: HashMap<String, SubjectMetadata>,
}

pub struct S3SchemaStore {
    client: Client,
    bucket: String,
    index: RwLock<GlobalIndex>,
}

impl S3SchemaStore {
    pub async fn new(bucket: String) -> Result<Self, StorageError> {
        let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let client = Client::new(&config);

        // Try to load existing index
        let index = Self::load_index_static(&client, &bucket).await.unwrap_or_default();

        Ok(Self {
            client,
            bucket,
            index: RwLock::new(index),
        })
    }

    async fn load_index_static(client: &Client, bucket: &str) -> Result<GlobalIndex, StorageError> {
        let result = client
            .get_object()
            .bucket(bucket)
            .key("index.json")
            .send()
            .await
            .map_err(|e| StorageError::Internal(e.to_string()))?;

        let bytes = result
            .body
            .collect()
            .await
            .map_err(|e| StorageError::Internal(e.to_string()))?;

        serde_json::from_slice(&bytes.into_bytes())
            .map_err(|e| StorageError::Internal(e.to_string()))
    }

    async fn save_index(&self, index: &GlobalIndex) -> Result<(), StorageError> {
        let json = serde_json::to_vec(index)
            .map_err(|e| StorageError::Internal(e.to_string()))?;

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key("index.json")
            .body(json.into())
            .send()
            .await
            .map_err(|e| StorageError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn save_schema(&self, schema: &SchemaEntry) -> Result<(), StorageError> {
        let json = serde_json::to_vec(schema)
            .map_err(|e| StorageError::Internal(e.to_string()))?;

        // Save by ID
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(format!("schemas/{}.json", schema.id))
            .body(json.clone().into())
            .send()
            .await
            .map_err(|e| StorageError::Internal(e.to_string()))?;

        // Save by subject/version
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(format!("subjects/{}/versions/{}.json", schema.subject, schema.version))
            .body(json.into())
            .send()
            .await
            .map_err(|e| StorageError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn load_schema_by_id(&self, id: i32) -> Result<SchemaEntry, StorageError> {
        let result = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(format!("schemas/{}.json", id))
            .send()
            .await
            .map_err(|_| StorageError::NotFound)?;

        let bytes = result
            .body
            .collect()
            .await
            .map_err(|e| StorageError::Internal(e.to_string()))?;

        serde_json::from_slice(&bytes.into_bytes())
            .map_err(|e| StorageError::Internal(e.to_string()))
    }

    async fn load_schema_by_version(&self, subject: &str, version: i32) -> Result<SchemaEntry, StorageError> {
        let result = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(format!("subjects/{}/versions/{}.json", subject, version))
            .send()
            .await
            .map_err(|_| StorageError::NotFound)?;

        let bytes = result
            .body
            .collect()
            .await
            .map_err(|e| StorageError::Internal(e.to_string()))?;

        serde_json::from_slice(&bytes.into_bytes())
            .map_err(|e| StorageError::Internal(e.to_string()))
    }

    fn entry_to_schema(entry: SchemaEntry) -> Schema {
        Schema {
            id: entry.id,
            subject: entry.subject,
            version: entry.version,
            schema: entry.schema,
            references: entry.references,
            schema_type: entry.schema_type.parse().unwrap(),
        }
    }
}

#[async_trait]
impl SchemaStore for S3SchemaStore {
    async fn get_schema_by_id(&self, id: i32) -> Result<Schema, StorageError> {
        let entry = self.load_schema_by_id(id).await?;
        Ok(Self::entry_to_schema(entry))
    }

    async fn register_schema(
        &self,
        subject: &str,
        schema_str: &str,
        schema_type: SchemaType,
    ) -> Result<i32, StorageError> {
        let mut index = self.index.write().await;

        // Get next schema ID
        let schema_id = index.next_schema_id + 1;
        index.next_schema_id = schema_id;

        // Get or create subject metadata
        let subject_meta = index.subjects.entry(subject.to_string()).or_default();
        let version = subject_meta.versions.iter().max().unwrap_or(&0) + 1;
        subject_meta.versions.push(version);

        // Create schema entry
        let entry = SchemaEntry {
            id: schema_id,
            subject: subject.to_string(),
            version,
            schema: schema_str.to_string(),
            schema_type: schema_type.as_str().to_string(),
            references: None,
        };

        // Save schema
        self.save_schema(&entry).await?;

        // Save updated index
        self.save_index(&index).await?;

        Ok(schema_id)
    }

    async fn get_subject_versions(&self, subject: &str) -> Result<Vec<i32>, StorageError> {
        let index = self.index.read().await;
        index
            .subjects
            .get(subject)
            .map(|m| m.versions.clone())
            .ok_or(StorageError::NotFound)
    }

    async fn get_schema_by_version(
        &self,
        subject: &str,
        version: i32,
    ) -> Result<Schema, StorageError> {
        let entry = self.load_schema_by_version(subject, version).await?;
        Ok(Self::entry_to_schema(entry))
    }

    async fn list_subjects(&self) -> Result<Vec<String>, StorageError> {
        let index = self.index.read().await;
        Ok(index.subjects.keys().cloned().collect())
    }

    async fn get_latest_version(&self, subject: &str) -> Result<i32, StorageError> {
        let index = self.index.read().await;
        index
            .subjects
            .get(subject)
            .and_then(|m| m.versions.iter().max().copied())
            .ok_or(StorageError::NotFound)
    }

    async fn lookup_schema(&self, subject: &str, schema_str: &str) -> Result<Schema, StorageError> {
        let index = self.index.read().await;
        let subject_meta = index.subjects.get(subject).ok_or(StorageError::NotFound)?;

        // Check each version to find matching schema
        for version in &subject_meta.versions {
            if let Ok(entry) = self.load_schema_by_version(subject, *version).await {
                if entry.schema == schema_str {
                    return Ok(Self::entry_to_schema(entry));
                }
            }
        }

        Err(StorageError::NotFound)
    }
}
