//! Domain models for request context, audit events, and masking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Context attached to a request after auth and vault layers.
#[derive(Clone, Debug)]
pub struct RequestContext {
    pub request_id: Uuid,
    /// For client tokens: the client_id. For user tokens (scope set): the user_id (sub).
    pub client_id: String,
    pub api_key: String,
    pub provider: LlmProvider,
    /// When set, token is a user token (sub = user_id). When None, token is client_credentials (sub = client_id).
    pub scope: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum LlmProvider {
    OpenAI,
    Anthropic,
}

/// Span of text that was masked (PII or secret).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaskedSpan {
    pub start: usize,
    pub end: usize,
    pub kind: String,
    pub replacement: String,
}

/// Audit event sent to the logging pipeline (RabbitMQ → Worker → ClickHouse).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    pub request_id: Uuid,
    pub client_id: String,
    pub provider: String,
    pub model: Option<String>,
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub latency_ms: Option<u64>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl AuditEvent {
    pub fn new(
        request_id: Uuid,
        client_id: String,
        provider: String,
        model: Option<String>,
        prompt_tokens: Option<u32>,
        completion_tokens: Option<u32>,
        latency_ms: Option<u64>,
        status: String,
    ) -> Self {
        Self {
            request_id,
            client_id,
            provider,
            model,
            prompt_tokens,
            completion_tokens,
            latency_ms,
            status,
            created_at: Utc::now(),
        }
    }
}

/// Chat request payload (OpenAI/Anthropic compatible).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: Option<String>,
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub stream: bool,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}
