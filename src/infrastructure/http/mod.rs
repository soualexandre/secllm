//! HTTP ingress – Axum routes, state, extractors, layers.

pub mod extractors;
pub mod layers;
pub mod openapi;
pub mod routes;
pub mod state;

pub use routes::router;
pub use state::AppState;
