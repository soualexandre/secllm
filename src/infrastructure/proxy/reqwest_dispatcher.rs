//! Reqwest-based proxy: forward request to OpenAI, Anthropic or Gemini base URL.
//!
//! Sem retentativas: em caso de erro na chamada ao provedor, a resposta de erro é retornada
//! imediatamente. Gemini: usa endpoint generateContent e header x-goog-api-key; reescreve
//! request/response quando path é chat/completions ou responses.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use crate::application::ports::ProxyPort;
use crate::domain::{LlmProvider, RequestContext};
use crate::Result;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

fn proxy_debug_enabled() -> bool {
    std::env::var("SECLLM_PROXY_DEBUG").as_deref() == Ok("1")
}

fn debug_log_request(provider: LlmProvider, url: &str, method: &str, body_len: usize, headers_count: usize) {
    if !proxy_debug_enabled() {
        return;
    }
    eprintln!(
        "[SECLLM proxy debug] {} {} {} body_len={} forwarded_headers={}",
        provider_name(provider),
        method,
        url,
        body_len,
        headers_count
    );
}

fn debug_log_success(provider: LlmProvider, status: u16, body_len: usize) {
    if !proxy_debug_enabled() {
        return;
    }
    eprintln!(
        "[SECLLM proxy debug] {} response status={} body_len={}",
        provider_name(provider),
        status,
        body_len
    );
}

fn debug_log_error(provider: LlmProvider, stage: &str, e: &(dyn std::error::Error + 'static)) {
    if !proxy_debug_enabled() {
        return;
    }
    eprintln!(
        "[SECLLM proxy debug] {} error at {}: {}",
        provider_name(provider),
        stage,
        e
    );
    let mut src = e.source();
    while let Some(s) = src {
        eprintln!("[SECLLM proxy debug]   cause: {}", s);
        src = s.source();
    }
}

fn provider_name(p: LlmProvider) -> &'static str {
    match p {
        LlmProvider::OpenAI => "OpenAI",
        LlmProvider::Anthropic => "Anthropic",
        LlmProvider::Gemini => "Gemini",
    }
}

/// Monta a mensagem completa do erro incluindo a cadeia de causas (source).
fn error_message_with_causes(e: &(dyn std::error::Error + 'static)) -> String {
    let mut msg = e.to_string();
    let mut src = e.source();
    while let Some(s) = src {
        msg.push_str(" | causa: ");
        msg.push_str(&s.to_string());
        src = s.source();
    }
    msg
}

/// Converte body no formato OpenAI (messages ou input) para o formato Gemini generateContent.
/// Retorna (url_completa, body_json_bytes). URL = base + "/v1beta/models/MODEL:generateContent".
fn openai_body_to_gemini(base: &str, body: &[u8]) -> Result<(String, Vec<u8>)> {
    let v: Value = serde_json::from_slice(body)
        .map_err(|e| crate::AppError::Proxy(format!("Gemini: body JSON inválido: {}", e)))?;
    let model = v
        .get("model")
        .and_then(|m| m.as_str())
        .unwrap_or("gemini-1.5-flash")
        .to_string();
    let text = if let Some(input) = v.get("input") {
        input
            .as_str()
            .ok_or_else(|| {
                crate::AppError::Proxy("Gemini: campo 'input' deve ser string no body".into())
            })?
            .to_string()
    } else if let Some(messages) = v.get("messages").and_then(|m| m.as_array()) {
        let last = messages
            .iter()
            .rev()
            .find(|m| m.get("role").and_then(|r| r.as_str()) == Some("user"));
        let content = last
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| {
                crate::AppError::Proxy("Gemini: body deve ter messages[].content (user) ou input".into())
            })?;
        content.to_string()
    } else {
        return Err(crate::AppError::Proxy(
            "Gemini: body deve ter 'messages' (OpenAI) ou 'input' (Responses API)".into(),
        ));
    };
    let max_tokens = v.get("max_tokens").and_then(|m| m.as_u64()).unwrap_or(1024) as i64;
    let temperature = v
        .get("temperature")
        .and_then(|t| t.as_f64())
        .unwrap_or(0.7);
    let gemini_body = json!({
        "contents": [{ "parts": [{ "text": text }] }],
        "generationConfig": {
            "maxOutputTokens": max_tokens,
            "temperature": temperature
        }
    });
    let path = format!("/v1beta/models/{}:generateContent", model);
    let url = format!("{}{}", base.trim_end_matches('/'), path);
    let body_bytes =
        serde_json::to_vec(&gemini_body).map_err(|e| crate::AppError::Proxy(e.to_string()))?;
    Ok((url, body_bytes))
}

/// Converte resposta Gemini (generateContent) para formato estilo OpenAI (choices[].message.content).
fn gemini_response_to_openai(gemini_body: &[u8]) -> Result<Vec<u8>> {
    let v: Value = serde_json::from_slice(gemini_body)
        .map_err(|e| crate::AppError::Proxy(format!("Gemini: resposta JSON inválida: {}", e)))?;
    let text = v
        .get("candidates")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first())
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.as_array())
        .and_then(|a| a.first())
        .and_then(|p| p.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("");
    let prompt_tokens = v
        .get("usageMetadata")
        .and_then(|u| u.get("promptTokenCount"))
        .and_then(|n| n.as_u64())
        .unwrap_or(0);
    let completion_tokens = v
        .get("usageMetadata")
        .and_then(|u| u.get("candidatesTokenCount"))
        .and_then(|n| n.as_u64())
        .unwrap_or(0);
    let openai_style = json!({
        "choices": [{ "message": { "role": "assistant", "content": text } }],
        "usage": { "prompt_tokens": prompt_tokens, "completion_tokens": completion_tokens }
    });
    serde_json::to_vec(&openai_style).map_err(|e| crate::AppError::Proxy(e.to_string()))
}

pub struct ReqwestDispatcher {
    client: Client,
    openai_base: String,
    anthropic_base: String,
    gemini_base: String,
}

impl ReqwestDispatcher {
    pub fn new(openai_base: String, anthropic_base: String, gemini_base: String) -> Result<Self> {
        let client = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .http1_only()
            .pool_max_idle_per_host(0)
            .build()
            .map_err(|e| crate::AppError::Proxy(e.to_string()))?;
        Ok(Self {
            client,
            openai_base,
            anthropic_base,
            gemini_base,
        })
    }

    fn proxy_error(
        &self,
        error_detail: impl std::fmt::Display,
        provider: LlmProvider,
        api_key: &str,
        url: &str,
    ) -> crate::AppError {
        let msg = error_detail.to_string();
        let hint = if msg.contains("connection closed")
            || msg.contains("connection reset")
            || msg.contains("timed out")
        {
            " Verifique se a API key é válida e se o provedor está acessível."
        } else {
            ""
        };
        let debug_hint = if !proxy_debug_enabled() {
            " Para debug no servidor: SECLLM_PROXY_DEBUG=1."
        } else {
            ""
        };
        crate::AppError::Proxy(format!(
            "erro ao chamar {} | URL: {} | API key (debug): {} | motivo: {}{}{}",
            provider_name(provider),
            url,
            api_key,
            msg,
            hint,
            debug_hint
        ))
    }

    fn base_url(&self, provider: LlmProvider) -> &str {
        match provider {
            LlmProvider::OpenAI => &self.openai_base,
            LlmProvider::Anthropic => &self.anthropic_base,
            LlmProvider::Gemini => &self.gemini_base,
        }
    }

    fn build_request(
        &self,
        method: &str,
        url: &str,
        ctx: &RequestContext,
        body: Vec<u8>,
        filtered_headers: &[(String, String)],
    ) -> reqwest::RequestBuilder {
        let req = match method {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            "PUT" => self.client.put(url),
            "PATCH" => self.client.patch(url),
            "DELETE" => self.client.delete(url),
            _ => self.client.get(url),
        };
        let req = if ctx.provider == LlmProvider::Gemini {
            req.header("x-goog-api-key", ctx.api_key.as_str())
        } else {
            req.header("Authorization", format!("Bearer {}", ctx.api_key))
        };
        let req = req
            .header("Content-Type", "application/json")
            .header("Connection", "close")
            .body(body);
        filtered_headers
            .iter()
            .fold(req, |r, (k, v)| r.header(k.as_str(), v.as_str()))
    }
}

#[async_trait]
impl ProxyPort for ReqwestDispatcher {
    async fn forward(
        &self,
        ctx: &RequestContext,
        method: &str,
        path: &str,
        body: Vec<u8>,
        headers: Vec<(String, String)>,
    ) -> Result<(u16, Vec<u8>, Option<u32>, Option<u32>)> {
        if !matches!(method, "GET" | "POST" | "PUT" | "PATCH" | "DELETE") {
            return Err(crate::AppError::Proxy(format!("unsupported method {}", method)));
        }

        let base = self.base_url(ctx.provider);
        let (url, body) = if ctx.provider == LlmProvider::Gemini
            && (path.contains("chat/completions") || path.contains("responses"))
        {
            let (gemini_url, gemini_body) = openai_body_to_gemini(&base, &body)?;
            (gemini_url, gemini_body)
        } else {
            (format!("{}{}", base.trim_end_matches('/'), path), body)
        };

        let body_len = body.len();
        let headers_count = headers.len();
        debug_log_request(ctx.provider, &url, method, body_len, headers_count);

        let filtered_headers: Vec<(String, String)> = headers
            .into_iter()
            .filter(|(k, _)| {
                let n = k.trim();
                !n.eq_ignore_ascii_case("content-type") && !n.eq_ignore_ascii_case("authorization") && !n.eq_ignore_ascii_case("connection")
            })
            .collect();

        let req = self.build_request(method, &url, ctx, body, &filtered_headers);

        let resp = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                debug_log_error(ctx.provider, "send", &e);
                return Err(self.proxy_error(
                    error_message_with_causes(&e),
                    ctx.provider,
                    &ctx.api_key,
                    &url,
                ));
            }
        };

        let status = resp.status().as_u16();
        if proxy_debug_enabled() {
            let h: Vec<_> = resp.headers().iter().map(|(k, v)| (k.as_str(), v.to_str().unwrap_or("?"))).collect();
            eprintln!("[SECLLM proxy debug] {} response headers: {:?}", provider_name(ctx.provider), h);
        }
        let prompt_tokens = resp
            .headers()
            .get("openai-usage-prompt-tokens")
            .or_else(|| resp.headers().get("x-prompt-tokens"))
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());
        let completion_tokens = resp
            .headers()
            .get("openai-usage-completion-tokens")
            .or_else(|| resp.headers().get("x-completion-tokens"))
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());

        let mut body_bytes = match resp.bytes().await {
            Ok(b) => b.to_vec(),
            Err(e) => {
                debug_log_error(ctx.provider, "read_body", &e);
                return Err(self.proxy_error(
                    error_message_with_causes(&e),
                    ctx.provider,
                    &ctx.api_key,
                    &url,
                ));
            }
        };

        if ctx.provider == LlmProvider::Gemini
            && url.contains("generateContent")
            && status >= 200
            && status < 300
        {
            body_bytes = gemini_response_to_openai(&body_bytes)?;
        }

        debug_log_success(ctx.provider, status, body_bytes.len());
        Ok((status, body_bytes, prompt_tokens, completion_tokens))
    }
}