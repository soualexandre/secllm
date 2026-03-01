//! SecLLM – High-performance reverse proxy for LLM consumption with governance.

pub mod config;
pub mod error;

pub mod domain;
pub mod application;
pub mod infrastructure;

pub use config::Config;
pub use error::{AppError, Result};
