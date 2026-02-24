//! Axum authentication middleware
//!
//! JWT Bearer token and API key authentication for Axum services.
//! Validates tokens by calling the auth-middleware service.

#[cfg(feature = "axum-support")]
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing;

/// Authenticated user extracted from JWT/API key
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub roles: Vec<String>,
    pub workspace_id: Option<String>,
}

/// Authentication layer configuration for Axum
#[derive(Clone)]
pub struct AxumAuthLayer {
    pub auth_service_url: String,
    pub auth_bypass_enabled: bool,
    http_client: reqwest::Client,
}

impl AxumAuthLayer {
    pub fn new(auth_service_url: String, auth_bypass_enabled: bool) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        Self {
            auth_service_url,
            auth_bypass_enabled,
            http_client,
        }
    }

    /// Validate a Bearer token against auth-middleware
    pub async fn verify_token(&self, token: &str) -> Result<AuthenticatedUser, String> {
        let url = format!("{}/auth/validate", self.auth_service_url);

        let res = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Auth service request failed: {}", e))?;

        if !res.status().is_success() {
            return Err(format!(
                "Auth service rejected token: {}",
                res.status()
            ));
        }

        let user: AuthenticatedUser = res
            .json()
            .await
            .map_err(|e| format!("Failed to parse auth response: {}", e))?;

        Ok(user)
    }

    /// Validate an API key against auth-middleware
    pub async fn validate_api_key(&self, key: &str) -> Result<AuthenticatedUser, String> {
        let url = format!("{}/auth/validate-api-key", self.auth_service_url);

        let res = self
            .http_client
            .post(&url)
            .header("X-API-Key", key)
            .send()
            .await
            .map_err(|e| format!("Auth service request failed: {}", e))?;

        if !res.status().is_success() {
            return Err(format!(
                "Auth service rejected API key: {}",
                res.status()
            ));
        }

        let user: AuthenticatedUser = res
            .json()
            .await
            .map_err(|e| format!("Failed to parse auth response: {}", e))?;

        Ok(user)
    }
}

/// Demo user for auth bypass in development
fn demo_user() -> AuthenticatedUser {
    AuthenticatedUser {
        id: "demo-user-001".to_string(),
        email: "demo@confuse.dev".to_string(),
        name: Some("Demo User".to_string()),
        picture: None,
        roles: vec!["user".to_string()],
        workspace_id: Some("demo-workspace-001".to_string()),
    }
}

/// Authentication middleware function for Axum
#[cfg(feature = "axum-support")]
pub async fn axum_auth_middleware(
    State(auth_layer): State<AxumAuthLayer>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    // Check for auth bypass (development only)
    if auth_layer.auth_bypass_enabled {
        tracing::debug!("Auth bypass enabled, using demo user");
        request.extensions_mut().insert(demo_user());
        return Ok(next.run(request).await);
    }

    // Try to extract authorization
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let api_key = request
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Extract workspace ID from headers (optional)
    let workspace_id = request
        .headers()
        .get("X-Workspace-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let mut user = if let Some(auth_value) = auth_header {
        if let Some(token) = auth_value.strip_prefix("Bearer ") {
            auth_layer.verify_token(token).await.map_err(|e| {
                tracing::warn!("Token verification failed: {}", e);
                (
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": { "code": "UNAUTHORIZED", "message": e }
                    })),
                )
                    .into_response()
            })?
        } else {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": { "code": "UNAUTHORIZED", "message": "Invalid authorization header format" }
                })),
            )
                .into_response());
        }
    } else if let Some(key) = api_key {
        auth_layer.validate_api_key(&key).await.map_err(|e| {
            tracing::warn!("API key validation failed: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": { "code": "UNAUTHORIZED", "message": e }
                })),
            )
                .into_response()
        })?
    } else {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": { "code": "UNAUTHORIZED", "message": "No authentication provided" }
            })),
        )
            .into_response());
    };

    // Set workspace_id if provided in headers
    if workspace_id.is_some() {
        user.workspace_id = workspace_id;
    }

    // Attach user to request extensions
    request.extensions_mut().insert(user);

    Ok(next.run(request).await)
}

/// Optional authentication — doesn't fail if no auth provided
#[cfg(feature = "axum-support")]
pub async fn axum_optional_auth_middleware(
    State(auth_layer): State<AxumAuthLayer>,
    mut request: Request,
    next: Next,
) -> Response {
    // Check for auth bypass
    if auth_layer.auth_bypass_enabled {
        request.extensions_mut().insert(demo_user());
        return next.run(request).await;
    }

    // Try to extract and validate authorization
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if let Some(auth_value) = auth_header {
        if let Some(token) = auth_value.strip_prefix("Bearer ") {
            if let Ok(user) = auth_layer.verify_token(token).await {
                request.extensions_mut().insert(user);
            }
        }
    }

    next.run(request).await
}
