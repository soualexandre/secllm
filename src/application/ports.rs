//! Ports (traits) for infrastructure adapters – vault, logger, proxy, privacy.

use async_trait::async_trait;
use crate::domain::{AuditEvent, GovernancePolicy, MaskedSpan, RequestContext};
use crate::Result;

/// Retrieve LLM API key for a client from the vault (e.g. Redis).
#[async_trait]
pub trait VaultPort: Send + Sync {
    async fn get_api_key(&self, client_id: &str, provider: &str) -> Result<String>;

    /// Client secret for authenticating and issuing JWT (e.g. Redis key secllm:auth:{client_id}).
    async fn get_client_secret(&self, client_id: &str) -> Result<Option<String>>;

    /// Replicate API key to Redis (after persisting in Postgres). Used by admin API.
    async fn set_api_key(&self, client_id: &str, provider: &str, api_key: &str) -> Result<()>;

    /// Remove API key from Redis (after deleting from Postgres).
    async fn del_api_key(&self, client_id: &str, provider: &str) -> Result<()>;

    /// Replicate client secret to Redis (after persisting in Postgres).
    async fn set_client_secret(&self, client_id: &str, secret: &str) -> Result<()>;

    /// Remove client secret from Redis.
    async fn del_client_secret(&self, client_id: &str) -> Result<()>;
}

/// Publish audit event and wait for broker confirm (Publisher Confirms).
#[async_trait]
pub trait LoggerPort: Send + Sync {
    async fn log_confirmed(&self, event: AuditEvent) -> Result<()>;
}

/// Forward request to LLM provider and return response body.
#[async_trait]
pub trait ProxyPort: Send + Sync {
    async fn forward(
        &self,
        ctx: &RequestContext,
        method: &str,
        path: &str,
        body: Vec<u8>,
        headers: Vec<(String, String)>,
    ) -> Result<(u16, Vec<u8>, Option<u32>, Option<u32>)>;
}

/// Scan text for PII/secrets and return masked text + spans.
pub trait PrivacyPort: Send + Sync {
    fn scan_and_mask(&self, text: &str) -> Result<(String, Vec<MaskedSpan>)>;

    /// Scan and mask using the given policy (e.g. from DB per request).
    fn scan_and_mask_with_policy(&self, text: &str, policy: &GovernancePolicy) -> Result<(String, Vec<MaskedSpan>)>;

    /// Detect PII that would be masked by policy (for block_on_pii check). Does not modify text.
    fn detect_with_policy(&self, text: &str, policy: &GovernancePolicy) -> Result<Vec<MaskedSpan>>;
}
