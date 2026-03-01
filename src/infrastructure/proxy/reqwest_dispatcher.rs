//! Reqwest-based proxy: forward request to OpenAI or Anthropic base URL.

use async_trait::async_trait;
use reqwest::Client;
use crate::application::ports::ProxyPort;
use crate::domain::{LlmProvider, RequestContext};
use crate::Result;

pub struct ReqwestDispatcher {
    client: Client,
    openai_base: String,
    anthropic_base: String,
}

impl ReqwestDispatcher {
    pub fn new(openai_base: String, anthropic_base: String) -> Result<Self> {
        let client = Client::builder()
            .build()
            .map_err(|e| crate::AppError::Proxy(e.to_string()))?;
        Ok(Self {
            client,
            openai_base,
            anthropic_base,
        })
    }

    fn base_url(&self, provider: LlmProvider) -> &str {
        match provider {
            LlmProvider::OpenAI => &self.openai_base,
            LlmProvider::Anthropic => &self.anthropic_base,
        }
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
        let base = self.base_url(ctx.provider);
        let url = format!("{}{}", base.trim_end_matches('/'), path);
        let mut req = match method {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            "PATCH" => self.client.patch(&url),
            "DELETE" => self.client.delete(&url),
            _ => return Err(crate::AppError::Proxy(format!("unsupported method {}", method))),
        };

        req = req
            .header("Authorization", format!("Bearer {}", ctx.api_key))
            .header("Content-Type", "application/json")
            .body(body);

        for (k, v) in headers {
            req = req.header(k, v);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| crate::AppError::Proxy(e.to_string()))?;
        let status = resp.status().as_u16();
        let body_bytes = resp
            .bytes()
            .await
            .map_err(|e| crate::AppError::Proxy(e.to_string()))?
            .to_vec();

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

        Ok((status, body_bytes, prompt_tokens, completion_tokens))
    }
}
