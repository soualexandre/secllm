//! Application state shared across handlers (vault, logger, proxy, privacy, RabbitMQ channel).

use std::sync::Arc;

use sqlx::PgPool;
use crate::application::ports::{LoggerPort, PrivacyPort, ProxyPort, VaultPort};
use crate::domain::GovernancePolicy;

/// ClickHouse client for read-only queries (logs e métricas). Cloneable e seguro para uso em handlers.
pub type ClickHouseClient = clickhouse::Client;

/// Shared state: one connection/channel for RabbitMQ, and port implementations.
pub struct AppState {
    pub vault: Arc<dyn VaultPort>,
    pub logger: Arc<dyn LoggerPort>,
    pub proxy: Arc<dyn ProxyPort>,
    pub privacy: Arc<dyn PrivacyPort>,
    pub governance: GovernancePolicy,
    pub postgres: Option<PgPool>,
    /// Cliente ClickHouse para consultas de logs e métricas (leitura).
    pub clickhouse: Option<(ClickHouseClient, String)>,
}
