//! Auth layer: validate JWT and set client_id + provider in extensions for vault.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use jsonwebtoken::{decode, DecodingKey, Validation};
use crate::domain::{LlmProvider, RequestContext};
use crate::infrastructure::http::extractors::request_id_from_parts;

/// JWT claims we expect (client_id or user_id, provider, optional scope). Used for decode and encode.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Claims {
    pub sub: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// For user tokens: "admin" | "user". Absent for client_credentials tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    pub exp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,
}

pub async fn auth_layer(
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let (mut parts, body) = request.into_parts();
    if parts.uri.path() == "/" {
        return Ok(next.run(Request::from_parts(parts, body)).await);
    }
    let request_id = request_id_from_parts(&parts);

    let auth_header = parts
        .headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "missing Authorization".into()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or((StatusCode::UNAUTHORIZED, "invalid Authorization format".into()))?;

    let secret = std::env::var("SECLLM_JWT_SECRET")
        .unwrap_or_else(|_| "change-me-in-production".to_string());
    let key = DecodingKey::from_secret(secret.as_bytes());
    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<Claims>(token, &key, &validation)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "invalid or expired token".into()))?;

    let client_id = token_data.claims.client_id
        .or(Some(token_data.claims.sub))
        .unwrap_or_default();
    let provider = token_data
        .claims
        .provider
        .as_deref()
        .unwrap_or("openai");
    let provider = match provider.to_lowercase().as_str() {
        "anthropic" => LlmProvider::Anthropic,
        _ => LlmProvider::OpenAI,
    };

    // We don't have api_key yet; vault layer will add RequestContext with api_key
    let ctx = RequestContext {
        request_id,
        client_id,
        api_key: String::new(),
        provider,
        scope: token_data.claims.scope.clone(),
        created_at: Utc::now(),
    };
    parts.extensions.insert(ctx);
    let request = Request::from_parts(parts, body);
    Ok(next.run(request).await)
}
