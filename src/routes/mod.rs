use crate::auth::BasicAuth;
use crate::cache::CachedSchemaStore;
use crate::error::StorageError;
use crate::models::{
    CompatibilityCheckRequest, CompatibilityCheckResponse, CompatibilityLevel, RegisterSchemaRequest,
    SchemaResponse, SchemaType,
};
use crate::services::CompatibilityChecker;
use crate::storage::SchemaStore;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<CachedSchemaStore>,
}

/// GET /subjects - List all subjects
pub async fn list_subjects(
    _auth: BasicAuth,
    State(state): State<AppState>,
) -> Result<Json<Vec<String>>, StorageError> {
    let subjects = state.store.list_subjects().await?;
    Ok(Json(subjects))
}

/// GET /subjects/:name/versions - List all versions of a subject
pub async fn list_versions(
    _auth: BasicAuth,
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Vec<i32>>, StorageError> {
    let versions = state.store.get_subject_versions(&name).await?;
    Ok(Json(versions))
}

/// POST /subjects/:name/versions - Register new schema version
pub async fn register_schema(
    _auth: BasicAuth,
    State(state): State<AppState>,
    Path(subject): Path<String>,
    Json(payload): Json<RegisterSchemaRequest>,
) -> Result<(StatusCode, Json<SchemaResponse>), StorageError> {
    let schema_type = SchemaType::from_str(&payload.schema_type.unwrap_or_else(|| "AVRO".to_string()));

    // Register schema
    let id = state
        .store
        .register_schema(&subject, &payload.schema, schema_type)
        .await?;

    // Fetch registered schema
    let schema = state.store.get_schema_by_id(id).await?;

    Ok((
        StatusCode::CREATED,
        Json(SchemaResponse {
            subject: schema.subject,
            version: schema.version,
            id: schema.id,
            schema: schema.schema,
        }),
    ))
}

/// GET /schemas/ids/:id - Get schema by global ID
pub async fn get_schema_by_id(
    _auth: BasicAuth,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<SchemaResponse>, StorageError> {
    let schema = state.store.get_schema_by_id(id).await?;
    Ok(Json(SchemaResponse {
        subject: schema.subject,
        version: schema.version,
        id: schema.id,
        schema: schema.schema,
    }))
}

/// POST /compatibility/subjects/:name/versions/:version - Check compatibility
pub async fn check_compatibility(
    _auth: BasicAuth,
    State(state): State<AppState>,
    Path((subject, version)): Path<(String, i32)>,
    Json(payload): Json<CompatibilityCheckRequest>,
) -> Result<Json<CompatibilityCheckResponse>, (StatusCode, Json<serde_json::Value>)> {
    let old_schema = state
        .store
        .get_schema_by_version(&subject, version)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;

    let is_compatible = CompatibilityChecker::check(
        &payload.schema,
        &old_schema.schema,
        CompatibilityLevel::Backward,
    )
    .map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;

    Ok(Json(CompatibilityCheckResponse { is_compatible }))
}
