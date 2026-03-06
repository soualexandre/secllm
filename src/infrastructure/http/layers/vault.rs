//! Vault layer: resolve API key from vault (Redis) and set full RequestContext.
//! Com SECLLM_MOCK_LLM=1 em rotas de gateway LLM, usa api_key "mock" e não exige chave no Redis.

use axum::{
    extract::Request,
    extract::State,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use crate::domain::RequestContext;
use crate::infrastructure::http::state::AppState;

fn is_gateway_llm_path(path: &str) -> bool {
    path.contains("chat/completions") || path.contains("/v1/responses") || path.ends_with("completions")
}

/// Mock ativo por padrão (quando SECLLM_MOCK_LLM não está definida). Desative com SECLLM_MOCK_LLM=0 ou false.
fn mock_llm_enabled() -> bool {
    let v = std::env::var("SECLLM_MOCK_LLM").unwrap_or_else(|_| String::new());
    let v = v.trim().to_lowercase();
    if v.is_empty() {
        return true; // padrão: mock ativo
    }
    matches!(v.as_str(), "1" | "true" | "yes")
}

pub async fn vault_layer(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let (mut parts, body) = request.into_parts();
    let path = parts.uri.path();
    if path == "/" {
        return Ok(next.run(Request::from_parts(parts, body)).await);
    }
    let ctx = parts
        .extensions
        .get::<RequestContext>()
        .cloned()
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "missing auth context").into_response())?;

    let provider_str = ctx.provider.as_str();
    let api_key = if mock_llm_enabled() && is_gateway_llm_path(path) {
        String::from("mock")
    } else {
        match state.vault.get_api_key(&ctx.client_id, provider_str).await {
            Ok(k) => k,
            Err(e) => {
                // Fallback para env em desenvolvimento: evita configurar o vault (Redis) para testar com LLM real.
                let env_key = match provider_str.to_lowercase().as_str() {
                    "openai" => std::env::var("SECLLM_OPENAI_API_KEY").ok(),
                    "anthropic" => std::env::var("SECLLM_ANTHROPIC_API_KEY").ok(),
                    "gemini" => std::env::var("SECLLM_GEMINI_API_KEY").ok(),
                    _ => None,
                };
                if let Some(k) = env_key.filter(|s| !s.trim().is_empty()) {
                    k
                } else {
                    return Err((
                        StatusCode::SERVICE_UNAVAILABLE,
                        format!("vault: {}", e),
                    )
                        .into_response());
                }
            }
        }
    };

    let full_ctx = RequestContext {
        api_key,
        ..ctx
    };
    parts.extensions.insert(full_ctx);
    let request = Request::from_parts(parts, body);
    Ok(next.run(request).await)
}
