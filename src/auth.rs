use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
};
use base64::{engine::general_purpose, Engine as _};

#[derive(Debug, Clone)]
pub struct BasicAuth {
    pub username: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for BasicAuth
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Missing Authorization header".to_string(),
                )
            })?;

        if let Some(encoded) = header.strip_prefix("Basic ") {
            let decoded = general_purpose::STANDARD
                .decode(encoded)
                .map_err(|_| {
                    (
                        StatusCode::UNAUTHORIZED,
                        "Invalid base64 encoding".to_string(),
                    )
                })?;

            let creds = String::from_utf8(decoded).map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Invalid UTF-8 in credentials".to_string(),
                )
            })?;

            let (username, password) = creds.split_once(':').ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    "Invalid credential format".to_string(),
                )
            })?;

            // Get expected credentials from env
            let valid_user = std::env::var("SCHEMA_REGISTRY_USER")
                .unwrap_or_else(|_| "admin".to_string());
            let valid_pass = std::env::var("SCHEMA_REGISTRY_PASSWORD")
                .unwrap_or_else(|_| "admin".to_string());

            if username == valid_user && password == valid_pass {
                return Ok(BasicAuth {
                    username: username.to_string(),
                });
            }

            return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
        }

        Err((
            StatusCode::UNAUTHORIZED,
            "Invalid Authorization header format".to_string(),
        ))
    }
}
