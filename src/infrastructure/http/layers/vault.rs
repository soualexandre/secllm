//! Vault layer: resolve API key from vault (Redis) and set full RequestContext.

use axum::{
    extract::Request,
    extract::State,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use crate::domain::RequestContext;
use crate::infrastructure::http::state::AppState;

pub async fn vault_layer(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let (mut parts, body) = request.into_parts();
    if parts.uri.path() == "/" {
        return Ok(next.run(Request::from_parts(parts, body)).await);
    }
    let ctx = parts
        .extensions
        .get::<RequestContext>()
        .cloned()
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "missing auth context").into_response())?;

    let provider_str = ctx.provider.as_str();
    let api_key = state
        .vault
        .get_api_key(&ctx.client_id, provider_str)
        .await
        .map_err(|e| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("vault: {}", e),
            )
                .into_response()
        })?;

    let full_ctx = RequestContext {
        api_key,
        ..ctx
    };
    parts.extensions.insert(full_ctx);
    let request = Request::from_parts(parts, body);
    Ok(next.run(request).await)
}
