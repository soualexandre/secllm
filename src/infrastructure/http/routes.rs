//! Route definitions and proxy handler.

use axum::{
    extract::{Path as AxumPath, State},
    http::{HeaderMap, Method, StatusCode, Uri},
    response::IntoResponse,
    routing::{any, post, put},
    Json, Router,
};
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use bytes::Bytes;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::sync::Arc;

#[allow(unused_imports)]
use utoipa::openapi::path::ParameterIn::Path;
use utoipa::OpenApi;

use crate::application::pipeline;
use crate::error::AppError;
use crate::infrastructure::http::openapi::ApiDoc;
use crate::infrastructure::http::extractors::Context;
use crate::infrastructure::http::layers::auth::Claims;
use crate::infrastructure::http::state::AppState;
use utoipa_swagger_ui::{Config, SwaggerUi};

/// Body para POST /auth/token. Either (client_id + client_secret) or (email + password).
#[derive(serde::Deserialize, utoipa::ToSchema)]
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
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct AuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

/// Body para POST /auth/register
#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub name: Option<String>,
}

/// Resposta de POST /auth/register
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct RegisterResponse {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
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
        .route("/governance/global", axum::routing::get(get_governance_global).put(put_governance_global))
        .route("/governance/clients/:client_id", axum::routing::get(get_governance_client).put(put_governance_client))
        .route("/billing/logs", axum::routing::post(post_billing_log))
        .route_layer(middleware::from_fn(layers::auth::auth_layer));

    let proxy_routes = Router::new()
        .route("/*path", any(proxy_handler))
        .route_layer(middleware::from_fn(layers::auth::auth_layer));

    let users_public = Router::new().route("/register", post(register_user));

    // Swagger UI (utoipa + utoipa-swagger-ui): público; spec em /api-docs/openapi.json
    let swagger_config = Config::from("/api-docs/openapi.json").persist_authorization(true);
    let swagger_ui = SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi())
        .config(swagger_config);

    Router::new()
        .route("/", any(health))
        .route("/auth/token", post(auth_token))
        .route("/auth/register", post(register_user))
        .nest("/api/users", users_public)
        .nest("/api/v1", api_routes)
        .merge(swagger_ui)
        .merge(proxy_routes)
        .with_state(state)
}

#[utoipa::path(
    post,
    path = "/auth/token",
    request_body = AuthTokenRequest,
    responses(
        (status = 200, description = "Token JWT", body = AuthTokenResponse),
        (status = 400, description = "Bad request", body = crate::infrastructure::http::openapi::ApiError),
        (status = 401, description = "Invalid credentials", body = crate::infrastructure::http::openapi::ApiError),
        (status = 503, description = "Auth service unavailable", body = crate::infrastructure::http::openapi::ApiError)
    ),
    security([]),
    tag = "1 - Autenticação"
)]
pub async fn auth_token(
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
        let user = validate_user_password_postgres(pool, email, password).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("auth service error: {}", e) })),
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
        let valid = valid.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("auth service error: {}", e) })),
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
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("failed to issue token: {}", e) })),
        )
    })?;
    Ok(Json(AuthTokenResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: exp_secs,
    }))
}

#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Usuário criado", body = RegisterResponse),
        (status = 400, description = "Validação falhou", body = crate::infrastructure::http::openapi::ApiError),
        (status = 409, description = "Email já registrado", body = crate::infrastructure::http::openapi::ApiError),
        (status = 503, description = "Postgres não configurado", body = crate::infrastructure::http::openapi::ApiError)
    ),
    security([]),
    tag = "1 - Autenticação"
)]
pub async fn register_user(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), (StatusCode, Json<serde_json::Value>)> {
    let email = body.email.trim();
    let password = body.password.trim();

    // Validation
    if email.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "email is required" })),
        ));
    }
    if password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "password must be at least 8 characters" })),
        ));
    }

    let pool = state.postgres.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({ "error": "user registration requires Postgres" })),
    ))?;

    // Hash password
    let salt = SaltString::generate(&mut rand::rngs::OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": format!("password hashing failed: {}", e) })),
            )
        })?
        .to_string();

    // Insert user
    let user_id = uuid::Uuid::new_v4();
    let result = sqlx::query(
        "INSERT INTO users (id, email, name, password_hash, role) VALUES ($1, $2, $3, $4, $5::user_role)",
    )
    .bind(&user_id)
    .bind(email)
    .bind(&body.name)
    .bind(&password_hash)
    .bind("user")
    .execute(pool)
    .await;

    match result {
        Ok(_) => Ok((
            StatusCode::CREATED,
            Json(RegisterResponse {
                id: user_id.to_string(),
                email: email.to_string(),
                name: body.name,
            }),
        )),
        Err(sqlx::Error::Database(db_err)) if db_err.message().contains("unique") => Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "error": "email already registered" })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("registration failed: {}", e) })),
        )),
    }
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

#[utoipa::path(
    get,
    path = "/",
    responses((status = 200, description = "Health check")),
    security([]),
    tag = "1 - Autenticação"
)]
pub async fn health() -> &'static str {
    "SecLLM: Sistema Ativo"
}

// ---- Vault API (CRUD + Redis replication) ----

fn api_err(status: StatusCode, message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": message })))
}

/// Resolve client_id string to (client_uuid, owner_user_id). Returns 404 if not found.
async fn resolve_client(
    pool: &sqlx::PgPool,
    client_id: &str,
) -> Result<(uuid::Uuid, uuid::Uuid), (StatusCode, Json<serde_json::Value>)> {
    let row: Option<(uuid::Uuid, uuid::Uuid)> = sqlx::query_as(
        "SELECT id, user_id FROM clients WHERE client_id = $1",
    )
    .bind(client_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("database error: {}", e)))?;
    match row {
        Some((id, user_id)) => Ok((id, user_id)),
        None => Err(api_err(StatusCode::NOT_FOUND, "client not found")),
    }
}

/// Check if the current identity (from JWT) can manage this client.
fn can_manage_client(
    ctx: &crate::domain::RequestContext,
    path_client_id: &str,
    owner_user_id: uuid::Uuid,
) -> bool {
    if let Some(_scope) = &ctx.scope {
        ctx.client_id == owner_user_id.to_string()
    } else {
        ctx.client_id == path_client_id
    }
}

fn normalize_provider(provider: &str) -> &'static str {
    if provider.eq_ignore_ascii_case("anthropic") {
        "anthropic"
    } else {
        "openai"
    }
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct PutApiKeyBody {
    pub api_key: String,
}

#[utoipa::path(
    put,
    path = "/api/v1/clients/{client_id}/keys/{provider}",
    params(("client_id" = String, Path, description = "ID do cliente (app)"), ("provider" = String, Path, description = "openai ou anthropic")),
    request_body = PutApiKeyBody,
    responses(
        (status = 204, description = "Chave criada/atualizada"),
        (status = 400, description = "api_key vazio", body = crate::infrastructure::http::openapi::ApiError),
        (status = 403, description = "Forbidden", body = crate::infrastructure::http::openapi::ApiError),
        (status = 404, description = "Cliente não encontrado", body = crate::infrastructure::http::openapi::ApiError),
        (status = 503, description = "Postgres não configurado", body = crate::infrastructure::http::openapi::ApiError)
    ),
    security(("bearer_auth" = [])),
    tag = "2 - Cofre (Vault)"
)]
pub async fn put_api_key(
    State(state): State<Arc<AppState>>,
    Context(ctx): Context,
    AxumPath((client_id, provider)): AxumPath<(String, String)>,
    Json(body): Json<PutApiKeyBody>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let pool = state
        .postgres
        .as_ref()
        .ok_or_else(|| api_err(StatusCode::SERVICE_UNAVAILABLE, "Postgres not configured"))?;
    let (client_uuid, owner_user_id) = resolve_client(pool, &client_id).await?;
    if !can_manage_client(&ctx, &client_id, owner_user_id) {
        return Err(api_err(StatusCode::FORBIDDEN, "forbidden"));
    }
    let prov = normalize_provider(&provider);
    let api_key = body.api_key.trim();
    if api_key.is_empty() {
        return Err(api_err(StatusCode::BAD_REQUEST, "api_key is required"));
    }
    sqlx::query(
        r#"
        INSERT INTO api_keys (client_id, provider, encrypted_key)
        VALUES ($1, $2::llm_provider, $3)
        ON CONFLICT (client_id, provider) DO UPDATE SET encrypted_key = $3, updated_at = now()
        "#,
    )
    .bind(client_uuid)
    .bind(prov)
    .bind(api_key)
    .execute(pool)
    .await
    .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("database error: {}", e)))?;
    state
        .vault
        .set_api_key(&client_id, prov, api_key)
        .await
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("failed to replicate to Redis: {}", e)))?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/v1/clients/{client_id}/keys/{provider}",
    params(("client_id" = String, Path, description = "ID do cliente"), ("provider" = String, Path, description = "openai ou anthropic")),
    responses(
        (status = 204, description = "Chave removida"),
        (status = 403, description = "Forbidden", body = crate::infrastructure::http::openapi::ApiError),
        (status = 404, description = "Chave não encontrada", body = crate::infrastructure::http::openapi::ApiError),
        (status = 503, body = crate::infrastructure::http::openapi::ApiError)
    ),
    security(("bearer_auth" = [])),
    tag = "2 - Cofre (Vault)"
)]
pub async fn delete_api_key(
    State(state): State<Arc<AppState>>,
    Context(ctx): Context,
    AxumPath((client_id, provider)): AxumPath<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let pool = state
        .postgres
        .as_ref()
        .ok_or_else(|| api_err(StatusCode::SERVICE_UNAVAILABLE, "Postgres not configured"))?;
    let (client_uuid, owner_user_id) = resolve_client(pool, &client_id).await?;
    if !can_manage_client(&ctx, &client_id, owner_user_id) {
        return Err(api_err(StatusCode::FORBIDDEN, "forbidden"));
    }
    let prov = normalize_provider(&provider);
    let res = sqlx::query(
        "DELETE FROM api_keys WHERE client_id = $1 AND provider = $2::llm_provider",
    )
    .bind(client_uuid)
    .bind(prov)
    .execute(pool)
    .await
    .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("database error: {}", e)))?;
    if res.rows_affected() == 0 {
        return Err(api_err(StatusCode::NOT_FOUND, "api key not found"));
    }
    state
        .vault
        .del_api_key(&client_id, prov)
        .await
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("failed to replicate to Redis: {}", e)))?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct PutClientSecretBody {
    pub client_secret: String,
}

#[utoipa::path(
    put,
    path = "/api/v1/clients/{client_id}/secret",
    params(("client_id" = String, Path, description = "ID do cliente")),
    request_body = PutClientSecretBody,
    responses(
        (status = 204, description = "Secret criado/atualizado"),
        (status = 400, description = "client_secret vazio", body = crate::infrastructure::http::openapi::ApiError),
        (status = 403, description = "Forbidden", body = crate::infrastructure::http::openapi::ApiError),
        (status = 404, body = crate::infrastructure::http::openapi::ApiError),
        (status = 503, body = crate::infrastructure::http::openapi::ApiError)
    ),
    security(("bearer_auth" = [])),
    tag = "2 - Cofre (Vault)"
)]
pub async fn put_client_secret(
    State(state): State<Arc<AppState>>,
    Context(ctx): Context,
    AxumPath(client_id): AxumPath<String>,
    Json(body): Json<PutClientSecretBody>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let pool = state
        .postgres
        .as_ref()
        .ok_or_else(|| api_err(StatusCode::SERVICE_UNAVAILABLE, "Postgres not configured"))?;
    let (client_uuid, owner_user_id) = resolve_client(pool, &client_id).await?;
    if !can_manage_client(&ctx, &client_id, owner_user_id) {
        return Err(api_err(StatusCode::FORBIDDEN, "forbidden"));
    }
    let secret = body.client_secret.trim();
    if secret.is_empty() {
        return Err(api_err(StatusCode::BAD_REQUEST, "client_secret is required"));
    }
    let salt = SaltString::generate(&mut rand::rngs::OsRng);
    let hash = argon2::Argon2::default()
        .hash_password(secret.as_bytes(), &salt)
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("hashing failed: {}", e)))?
        .to_string();
    sqlx::query(
        r#"
        INSERT INTO client_secrets (client_id, secret_hash)
        VALUES ($1, $2)
        ON CONFLICT (client_id) DO UPDATE SET secret_hash = $2
        "#,
    )
    .bind(client_uuid)
    .bind(&hash)
    .execute(pool)
    .await
    .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("database error: {}", e)))?;
    state
        .vault
        .set_client_secret(&client_id, secret)
        .await
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("failed to replicate to Redis: {}", e)))?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/v1/clients/{client_id}/secret",
    params(("client_id" = String, Path, description = "ID do cliente")),
    responses(
        (status = 204, description = "Secret removido"),
        (status = 403, body = crate::infrastructure::http::openapi::ApiError),
        (status = 404, body = crate::infrastructure::http::openapi::ApiError),
        (status = 503, body = crate::infrastructure::http::openapi::ApiError)
    ),
    security(("bearer_auth" = [])),
    tag = "2 - Cofre (Vault)"
)]
pub async fn delete_client_secret(
    State(state): State<Arc<AppState>>,
    Context(ctx): Context,
    AxumPath(client_id): AxumPath<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let pool = state
        .postgres
        .as_ref()
        .ok_or_else(|| api_err(StatusCode::SERVICE_UNAVAILABLE, "Postgres not configured"))?;
    let (client_uuid, owner_user_id) = resolve_client(pool, &client_id).await?;
    if !can_manage_client(&ctx, &client_id, owner_user_id) {
        return Err(api_err(StatusCode::FORBIDDEN, "forbidden"));
    }
    let res = sqlx::query("DELETE FROM client_secrets WHERE client_id = $1")
        .bind(client_uuid)
        .execute(pool)
        .await
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("database error: {}", e)))?;
    if res.rows_affected() == 0 {
        return Err(api_err(StatusCode::NOT_FOUND, "client secret not found"));
    }
    state
        .vault
        .del_client_secret(&client_id)
        .await
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("failed to replicate to Redis: {}", e)))?;
    Ok(StatusCode::NO_CONTENT)
}

// ---- Billing logs API ----

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct PostBillingLogBody {
    pub period_start: String,
    pub period_end: String,
    pub amount_cents: i64,
    #[serde(default)]
    pub details: Option<serde_json::Value>,
    #[serde(default)]
    pub client_id: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/billing/logs",
    request_body = PostBillingLogBody,
    responses(
        (status = 201, description = "Log criado"),
        (status = 400, description = "Datas inválidas", body = crate::infrastructure::http::openapi::ApiError),
        (status = 403, description = "Token de usuário obrigatório", body = crate::infrastructure::http::openapi::ApiError),
        (status = 503, body = crate::infrastructure::http::openapi::ApiError)
    ),
    security(("bearer_auth" = [])),
    tag = "4 - Faturamento"
)]
pub async fn post_billing_log(
    State(state): State<Arc<AppState>>,
    Context(ctx): Context,
    Json(body): Json<PostBillingLogBody>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let user_id = ctx
        .scope
        .as_ref()
        .and_then(|_| uuid::Uuid::parse_str(&ctx.client_id).ok())
        .ok_or_else(|| api_err(StatusCode::FORBIDDEN, "user token required"))?;
    let pool = state
        .postgres
        .as_ref()
        .ok_or_else(|| api_err(StatusCode::SERVICE_UNAVAILABLE, "Postgres not configured"))?;
    let period_start: chrono::NaiveDate = body
        .period_start
        .parse()
        .map_err(|e| api_err(StatusCode::BAD_REQUEST, &format!("period_start invalid (expected YYYY-MM-DD): {}", e)))?;
    let period_end: chrono::NaiveDate = body
        .period_end
        .parse()
        .map_err(|e| api_err(StatusCode::BAD_REQUEST, &format!("period_end invalid (expected YYYY-MM-DD): {}", e)))?;
    let client_uuid = if let Some(cid) = &body.client_id {
        let (client_uuid, owner_user_id) = resolve_client(pool, cid).await?;
        if !can_manage_client(&ctx, cid, owner_user_id) {
            return Err(api_err(StatusCode::FORBIDDEN, "forbidden"));
        }
        Some(client_uuid)
    } else {
        None
    };
    let details = body.details.unwrap_or(serde_json::json!({}));
    sqlx::query(
        "INSERT INTO billing_logs (user_id, client_id, period_start, period_end, amount_cents, details) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(user_id)
    .bind(client_uuid)
    .bind(period_start)
    .bind(period_end)
    .bind(body.amount_cents)
    .bind(&details)
    .execute(pool)
    .await
    .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("database error: {}", e)))?;
    Ok(StatusCode::CREATED)
}

// ---- Governance policies API ----

#[utoipa::path(
    get,
    path = "/api/v1/governance/global",
    responses(
        (status = 200, description = "Política global (JSONB)"),
        (status = 503, body = crate::infrastructure::http::openapi::ApiError)
    ),
    security(("bearer_auth" = [])),
    tag = "3 - Governança"
)]
pub async fn get_governance_global(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let pool = state
        .postgres
        .as_ref()
        .ok_or_else(|| api_err(StatusCode::SERVICE_UNAVAILABLE, "Postgres not configured"))?;
    let row: Option<(serde_json::Value,)> = sqlx::query_as(
        "SELECT policy FROM governance_policies WHERE scope = 'global' AND client_id IS NULL LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("database error: {}", e)))?;
    Ok(Json(
        row.map(|(p,)| p).unwrap_or(serde_json::json!({ "mask_pii": [], "mask_response": true })),
    ))
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct PutGovernanceBody {
    pub policy: serde_json::Value,
}

#[utoipa::path(
    put,
    path = "/api/v1/governance/global",
    request_body = PutGovernanceBody,
    responses(
        (status = 204, description = "Política global atualizada"),
        (status = 403, description = "Apenas admin", body = crate::infrastructure::http::openapi::ApiError),
        (status = 503, body = crate::infrastructure::http::openapi::ApiError)
    ),
    security(("bearer_auth" = [])),
    tag = "3 - Governança"
)]
pub async fn put_governance_global(
    State(state): State<Arc<AppState>>,
    Context(ctx): Context,
    Json(body): Json<PutGovernanceBody>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    if ctx.scope.as_deref() != Some("admin") {
        return Err(api_err(StatusCode::FORBIDDEN, "admin only"));
    }
    let pool = state
        .postgres
        .as_ref()
        .ok_or_else(|| api_err(StatusCode::SERVICE_UNAVAILABLE, "Postgres not configured"))?;
    let res = sqlx::query(
        "UPDATE governance_policies SET policy = $1, updated_at = now() WHERE scope = 'global' AND client_id IS NULL",
    )
    .bind(&body.policy)
    .execute(pool)
    .await
    .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("database error: {}", e)))?;
    if res.rows_affected() == 0 {
        sqlx::query(
            "INSERT INTO governance_policies (scope, client_id, policy) VALUES ('global', NULL, $1)",
        )
        .bind(&body.policy)
        .execute(pool)
        .await
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("database error: {}", e)))?;
    }
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/governance/clients/{client_id}",
    params(("client_id" = String, Path, description = "ID do cliente")),
    responses(
        (status = 200, description = "Política do cliente"),
        (status = 403, body = crate::infrastructure::http::openapi::ApiError),
        (status = 404, body = crate::infrastructure::http::openapi::ApiError),
        (status = 503, body = crate::infrastructure::http::openapi::ApiError)
    ),
    security(("bearer_auth" = [])),
    tag = "3 - Governança"
)]
pub async fn get_governance_client(
    State(state): State<Arc<AppState>>,
    Context(ctx): Context,
    AxumPath(client_id): AxumPath<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let pool = state
        .postgres
        .as_ref()
        .ok_or_else(|| api_err(StatusCode::SERVICE_UNAVAILABLE, "Postgres not configured"))?;
    let (client_uuid, owner_user_id) = resolve_client(pool, &client_id).await?;
    if !can_manage_client(&ctx, &client_id, owner_user_id) {
        return Err(api_err(StatusCode::FORBIDDEN, "forbidden"));
    }
    let row: Option<(serde_json::Value,)> = sqlx::query_as(
        "SELECT policy FROM governance_policies WHERE scope = 'client' AND client_id = $1 LIMIT 1",
    )
    .bind(client_uuid)
    .fetch_optional(pool)
    .await
    .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("database error: {}", e)))?;
    Ok(Json(
        row.map(|(p,)| p).unwrap_or(serde_json::json!({ "mask_pii": [], "mask_response": true })),
    ))
}

#[utoipa::path(
    put,
    path = "/api/v1/governance/clients/{client_id}",
    params(("client_id" = String, Path, description = "ID do cliente")),
    request_body = PutGovernanceBody,
    responses(
        (status = 204, description = "Política do cliente atualizada"),
        (status = 403, body = crate::infrastructure::http::openapi::ApiError),
        (status = 404, body = crate::infrastructure::http::openapi::ApiError),
        (status = 503, body = crate::infrastructure::http::openapi::ApiError)
    ),
    security(("bearer_auth" = [])),
    tag = "3 - Governança"
)]
pub async fn put_governance_client(
    State(state): State<Arc<AppState>>,
    Context(ctx): Context,
    AxumPath(client_id): AxumPath<String>,
    Json(body): Json<PutGovernanceBody>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let pool = state
        .postgres
        .as_ref()
        .ok_or_else(|| api_err(StatusCode::SERVICE_UNAVAILABLE, "Postgres not configured"))?;
    let (client_uuid, owner_user_id) = resolve_client(pool, &client_id).await?;
    if !can_manage_client(&ctx, &client_id, owner_user_id) {
        return Err(api_err(StatusCode::FORBIDDEN, "forbidden"));
    }
    let res = sqlx::query(
        "UPDATE governance_policies SET policy = $1, updated_at = now() WHERE scope = 'client' AND client_id = $2",
    )
    .bind(&body.policy)
    .bind(client_uuid)
    .execute(pool)
    .await
    .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("database error: {}", e)))?;
    if res.rows_affected() == 0 {
        sqlx::query(
            "INSERT INTO governance_policies (scope, client_id, policy) VALUES ('client', $1, $2)",
        )
        .bind(client_uuid)
        .bind(&body.policy)
        .execute(pool)
        .await
        .map_err(|e| api_err(StatusCode::INTERNAL_SERVER_ERROR, &format!("database error: {}", e)))?;
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Documentação OpenAPI do gateway (rota real é catch-all /*path).
#[utoipa::path(
    post,
    path = "/v1/chat/completions",
    tag = "5 - Gateway LLM",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Resposta repassada do provedor (OpenAI/Anthropic). Qualquer path é encaminhado; provider do JWT."),
        (status = 401, description = "Token ausente ou inválido"),
        (status = 502, description = "Erro do provedor LLM")
    )
)]
pub fn proxy_handler_doc() {}

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
