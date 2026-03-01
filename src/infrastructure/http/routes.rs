//! Route definitions and proxy handler.

use axum::{
    extract::State,
    http::{HeaderMap, Method, StatusCode, Uri},
    response::IntoResponse,
    routing::any,
    Router,
};
use bytes::Bytes;
use std::sync::Arc;

use crate::application::pipeline;
use crate::error::AppError;
use crate::infrastructure::http::extractors::Context;
use crate::infrastructure::http::state::AppState;

pub fn router(state: Arc<AppState>) -> Router {
    use axum::middleware;
    use crate::infrastructure::http::layers;

    Router::new()
        .route("/", any(health))
        .route(
            "/*path",
            any(proxy_handler)
                .route_layer(middleware::from_fn(layers::auth::auth_layer))
                .route_layer(middleware::from_fn_with_state(state.clone(), layers::vault::vault_layer)),
        )
        .with_state(state)
}

async fn health() -> &'static str {
    "SecLLM: Sistema Ativo"
}

async fn proxy_handler(
    State(state): State<Arc<AppState>>,
    Context(ctx): Context,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
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
