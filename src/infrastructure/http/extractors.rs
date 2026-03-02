//! Extractors for request context (set by auth + vault layers).

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

use crate::domain::RequestContext;

/// Request context injected by auth + vault middlewares.
#[derive(Clone, Debug)]
pub struct Context(pub RequestContext);

impl<S> FromRequestParts<S> for Context
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    fn from_request_parts<'a, 'b, 'async_trait>(
        parts: &'a mut Parts,
        _state: &'b S,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>>
    where
        'a: 'async_trait,
        'b: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let ctx = parts
                .extensions
                .get::<RequestContext>()
                .cloned()
                .ok_or((StatusCode::UNAUTHORIZED, "missing request context"))?;
            Ok(Context(ctx))
        })
    }
}

/// Optional request ID from header (or generate).
pub fn request_id_from_parts(parts: &Parts) -> Uuid {
    parts
        .headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::new_v4)
}
