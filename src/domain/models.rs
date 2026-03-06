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

#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash)]
pub enum LlmProvider {
    OpenAI,
    Anthropic,
    Gemini,
}

impl LlmProvider {
    pub fn as_str(self) -> &'static str {
        match self {
            LlmProvider::OpenAI => "openai",
            LlmProvider::Anthropic => "anthropic",
            LlmProvider::Gemini => "gemini",
        }
    }

    pub fn all() -> &'static [LlmProvider] {
        &[LlmProvider::OpenAI, LlmProvider::Anthropic, LlmProvider::Gemini]
    }

    pub fn from_str(s: &str) -> Option<LlmProvider> {
        match s.to_lowercase().as_str() {
            "openai" => Some(LlmProvider::OpenAI),
            "anthropic" => Some(LlmProvider::Anthropic),
            "gemini" => Some(LlmProvider::Gemini),
            _ => None,
        }
    }
}

/// Span of text that was masked (PII or secret).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaskedSpan {
    pub start: usize,
    pub end: usize,
    pub kind: String,
    pub replacement: String,
}

/// Tamanho máximo do body (request/response) armazenado no audit (evita mensagens gigantes).
const MAX_BODY_LEN: usize = 200_000;

/// Audit event sent to the logging pipeline (RabbitMQ → Worker → ClickHouse).
/// Inclui entrada (request body) e saída (response body) para auditoria completa.
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
    pub input_size: Option<u64>,
    pub output_size: Option<u64>,
    /// Body da requisição (já mascarado por privacy). Truncado a MAX_BODY_LEN.
    pub request_body: Option<String>,
    /// Body da resposta LLM (já mascarado). Truncado a MAX_BODY_LEN.
    pub response_body: Option<String>,
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
        input_size: Option<u64>,
        output_size: Option<u64>,
        request_body: Option<String>,
        response_body: Option<String>,
    ) -> Self {
        let request_body = request_body.map(|s| truncate_str(&s, MAX_BODY_LEN));
        let response_body = response_body.map(|s| truncate_str(&s, MAX_BODY_LEN));
        Self {
            request_id,
            client_id,
            provider,
            model,
            prompt_tokens,
            completion_tokens,
            latency_ms,
            status,
            input_size,
            output_size,
            request_body,
            response_body,
            created_at: Utc::now(),
        }
    }
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}... [truncado {} chars]", &s[..max], s.len() - max)
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
