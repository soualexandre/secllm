//! Route definitions and proxy handler.

use axum::{
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode, Uri},
    response::IntoResponse,
    routing::{any, delete, post, put},
    Json, Router,
};
use argon2::password_hash::{PasswordHash, PasswordVerifier};
use argon2::Argon2;
use bytes::Bytes;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::sync::Arc;

use crate::application::pipeline;
use crate::error::AppError;
use crate::infrastructure::http::extractors::Context;
use crate::infrastructure::http::layers::auth::Claims;
use crate::infrastructure::http::state::AppState;

/// Body para POST /auth/token. Either (client_id + client_secret) or (email + password).
#[derive(serde::Deserialize)]
pub struct AuthTokenRequest {
    #[serde(default)]
    pub client_id: Option<String>,
    #[serde(default)]
    pub client_secret: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
}

/// Resposta de POST /auth/token
#[derive(serde::Serialize)]
pub struct AuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

pub fn router(state: Arc<AppState>) -> Router {
    use axum::middleware;
    use crate::infrastructure::http::layers;

    let api_routes = Router::new()
        .route(
            "/clients/:client_id/keys/:provider",
            put(put_api_key).delete(delete_api_key),
        )
        .route("/clients/:client_id/secret", put(put_client_secret).delete(delete_client_secret))
        .route_layer(middleware::from_fn(layers::auth::auth_layer));

    Router::new()
        .route("/", any(health))
        .route("/auth/token", post(auth_token))
        .nest("/api/v1", api_routes)
        .route(
            "/*path",
            any(proxy_handler).route_layer(middleware::from_fn(layers::auth::auth_layer)),
        )
        .with_state(state)
}

async fn auth_token(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AuthTokenRequest>,
) -> Result<Json<AuthTokenResponse>, (StatusCode, Json<serde_json::Value>)> {
    let jwt_secret = std::env::var("SECLLM_JWT_SECRET")
        .unwrap_or_else(|_| std::env::var("SECLLM__JWT__SECRET").unwrap_or_else(|_| "change-me-in-production".to_string()));
    let exp_secs = 3600u64;
    let now = Utc::now();
    let exp = (now + Duration::seconds(exp_secs as i64)).timestamp();

    let (sub, client_id, provider, scope) = if let (Some(email), Some(password)) = (&body.email, &body.password) {
        let email = email.trim();
        if email.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "email is required for password grant" })),
            ));
        }
        let pool = state.postgres.as_ref().ok_or((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "user login requires Postgres" })),
        ))?;
        let user = validate_user_password_postgres(pool, email, password).await.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "auth service error" })),
            )
        })?;
        let (user_id, role) = match user {
            Some((id, r)) => (id, r),
            None => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({ "error": "invalid email or password" })),
                ));
            }
        };
        (user_id.to_string(), None, None, Some(role))
    } else if let (Some(cid), Some(secret)) = (&body.client_id, &body.client_secret) {
        let client_id = cid.trim();
        if client_id.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "client_id is required" })),
            ));
        }
        let valid = if let Some(pool) = &state.postgres {
            validate_client_secret_postgres(pool, client_id, secret).await
        } else {
            validate_client_secret_redis(state.vault.as_ref(), client_id, secret).await
        };
        let valid = valid.map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "auth service error" })),
            )
        })?;
        if !valid {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "invalid client_id or client_secret" })),
            ));
        }
        let provider = body
            .provider
            .as_deref()
            .unwrap_or("openai")
            .to_string();
        (client_id.to_string(), Some(client_id.to_string()), Some(provider), None)
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "provide either (email + password) or (client_id + client_secret)" })),
        ));
    };

    let claims = Claims {
        sub,
        client_id,
        provider,
        scope,
        exp,
        iat: Some(now.timestamp()),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "failed to issue token" })),
        )
    })?;
    Ok(Json(AuthTokenResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: exp_secs,
    }))
}

/// Validate email + password against Postgres (users table). Returns (user_id, role) or None.
async fn validate_user_password_postgres(
    pool: &sqlx::PgPool,
    email: &str,
    password: &str,
) -> crate::Result<Option<(uuid::Uuid, String)>> {
    let row: Option<(uuid::Uuid, String, String)> = sqlx::query_as(
        "SELECT id, password_hash, role::text FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .map_err(|e| crate::AppError::Internal(anyhow::Error::from(e)))?;
    let (id, stored_hash, role) = match row {
        Some(r) => r,
        None => return Ok(None),
    };
    let parsed = match PasswordHash::new(&stored_hash) {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };
    if Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
    {
        Ok(Some((id, role)))
    } else {
        Ok(None)
    }
}

/// Validate client_id + client_secret against Postgres (clients + client_secrets, argon2 hash).
async fn validate_client_secret_postgres(
    pool: &sqlx::PgPool,
    client_id: &str,
    client_secret: &str,
) -> crate::Result<bool> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT cs.secret_hash FROM clients c INNER JOIN client_secrets cs ON cs.client_id = c.id WHERE c.client_id = $1",
    )
    .bind(client_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| crate::AppError::Internal(anyhow::Error::from(e)))?;
    let (stored_hash,) = match row {
        Some(r) => r,
        None => return Ok(false),
    };
    let parsed = match PasswordHash::new(&stored_hash) {
        Ok(p) => p,
        Err(_) => return Ok(false),
    };
    Ok(Argon2::default()
        .verify_password(client_secret.as_bytes(), &parsed)
        .is_ok())
}

/// Validate client_id + client_secret against Redis (plain comparison).
async fn validate_client_secret_redis(
    vault: &dyn crate::application::ports::VaultPort,
    client_id: &str,
    client_secret: &str,
) -> crate::Result<bool> {
    let stored = vault.get_client_secret(client_id).await?;
    Ok(stored
        .as_deref()
        .map(|s| s == client_secret)
        .unwrap_or(false))
}

async fn health() -> &'static str {
    "SecLLM: Sistema Ativo"
}

async fn proxy_handler(
    State(state): State<Arc<AppState>>,
    Context(partial_ctx): Context,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    let provider_str = match partial_ctx.provider {
        crate::domain::LlmProvider::OpenAI => "openai",
        crate::domain::LlmProvider::Anthropic => "anthropic",
    };
    let api_key = state
        .vault
        .get_api_key(&partial_ctx.client_id, provider_str)
        .await?;
    let ctx = crate::domain::RequestContext {
        api_key,
        ..partial_ctx
    };

    let path = uri.path();
    let path_query = uri
        .path_and_query()
        .map(|p| p.as_str())
        .unwrap_or(path);

    let forward_headers: Vec<(String, String)> = headers
        .iter()
        .filter(|(n, _)| {
            let s = n.as_str();
            !s.eq_ignore_ascii_case("authorization")
                && !s.eq_ignore_ascii_case("host")
                && !s.eq_ignore_ascii_case("connection")
        })
        .filter_map(|(n, v)| {
            let v = v.to_str().ok()?;
            Some((n.to_string(), v.to_string()))
        })
        .collect();

    let (status, body_bytes, _pt, _ct) = pipeline::handle_request(
        &ctx,
        method.as_str(),
        path_query,
        body.to_vec(),
        forward_headers,
        state.logger.as_ref(),
        state.proxy.as_ref(),
        state.privacy.as_ref(),
    )
    .await?;

    let status = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    Ok((status, Bytes::from(body_bytes)))
}
