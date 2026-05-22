use crate::error::StorageError;
use crate::models::{Schema, SchemaType};
use crate::storage::SchemaStore;
use async_trait::async_trait;
use moka::future::Cache;
use std::sync::Arc;

pub struct CachedSchemaStore {
    inner: Arc<dyn SchemaStore>,
    cache: Arc<Cache<i32, Schema>>,
}

impl CachedSchemaStore {
    pub async fn new(store: Arc<dyn SchemaStore>, max_capacity: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .build();

        Self {
            inner: store,
            cache: Arc::new(cache),
        }
    }
}

#[async_trait]
impl SchemaStore for CachedSchemaStore {
    async fn get_schema_by_id(&self, id: i32) -> Result<Schema, StorageError> {
        // Try cache first
        if let Some(schema) = self.cache.get(&id).await {
            return Ok(schema);
        }

        // Cache miss - fetch from storage
        let schema = self.inner.get_schema_by_id(id).await?;

        // Insert into cache for future requests
        self.cache.insert(id, schema.clone()).await;

        Ok(schema)
    }

    async fn register_schema(
        &self,
        subject: &str,
        schema_str: &str,
        schema_type: SchemaType,
    ) -> Result<i32, StorageError> {
        // Register in storage (new ID won't need cache invalidation)
        self.inner
            .register_schema(subject, schema_str, schema_type)
            .await
    }

    async fn get_subject_versions(&self, subject: &str) -> Result<Vec<i32>, StorageError> {
        // Don't cache this (small list, fetched rarely)
        self.inner.get_subject_versions(subject).await
    }

    async fn get_schema_by_version(
        &self,
        subject: &str,
        version: i32,
    ) -> Result<Schema, StorageError> {
        // Fetch by version, then cache by ID
        let schema = self.inner.get_schema_by_version(subject, version).await?;
        self.cache.insert(schema.id, schema.clone()).await;
        Ok(schema)
    }

    async fn list_subjects(&self) -> Result<Vec<String>, StorageError> {
        // Don't cache (rarely changes)
        self.inner.list_subjects().await
    }

    async fn get_latest_version(&self, subject: &str) -> Result<i32, StorageError> {
        self.inner.get_latest_version(subject).await
    }

    async fn lookup_schema(&self, subject: &str, schema_str: &str) -> Result<Schema, StorageError> {
        let schema = self.inner.lookup_schema(subject, schema_str).await?;
        self.cache.insert(schema.id, schema.clone()).await;
        Ok(schema)
    }
}
