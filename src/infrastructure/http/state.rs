//! Application state shared across handlers (vault, logger, proxy, privacy, RabbitMQ channel).

use std::sync::Arc;

use crate::application::ports::{LoggerPort, PrivacyPort, ProxyPort, VaultPort};
use crate::domain::GovernancePolicy;

/// Shared state: one connection/channel for RabbitMQ, and port implementations.
pub struct AppState {
    pub vault: Arc<dyn VaultPort>,
    pub logger: Arc<dyn LoggerPort>,
    pub proxy: Arc<dyn ProxyPort>,
    pub privacy: Arc<dyn PrivacyPort>,
    pub governance: GovernancePolicy,
}
