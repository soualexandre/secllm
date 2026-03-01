//! Application layer – use cases and ports (traits).

pub mod ports;
pub mod pipeline;

pub use ports::{LoggerPort, PrivacyPort, ProxyPort, VaultPort};
pub use pipeline::handle_request;
