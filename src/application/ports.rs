//! Ports (traits) for infrastructure adapters – vault, logger, proxy, privacy.

use async_trait::async_trait;
use crate::domain::{AuditEvent, MaskedSpan, RequestContext};
use crate::Result;

/// Retrieve LLM API key for a client from the vault (e.g. Redis).
#[async_trait]
pub trait VaultPort: Send + Sync {
    async fn get_api_key(&self, client_id: &str, provider: &str) -> Result<String>;
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
}
