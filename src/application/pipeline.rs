//! Request pipeline: privacy in → proxy → privacy out → log (confirmed) → response.

use std::time::Instant;

use crate::domain::{AuditEvent, GovernancePolicy, RequestContext};
use crate::application::ports::{LoggerPort, PrivacyPort, ProxyPort};
use crate::Result;

/// Extrai o campo "model" do body JSON (gateway LLM) para auditoria.
fn model_from_body(body: &[u8]) -> Option<String> {
    let v: serde_json::Value = serde_json::from_slice(body).ok()?;
    v.get("model")
        .and_then(|m| m.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Runs the full pipeline and returns (status_code, body, prompt_tokens, completion_tokens).
/// If `policy` is Some, it is used for masking/blocking (from DB); otherwise the default policy in `privacy` is used.
/// When policy.block_on_pii is true, request is rejected with 400 if PII is detected.
pub async fn handle_request(
    ctx: &RequestContext,
    method: &str,
    path: &str,
    body: Vec<u8>,
    headers: Vec<(String, String)>,
    logger: &dyn LoggerPort,
    proxy: &dyn ProxyPort,
    privacy: &dyn PrivacyPort,
    policy: Option<&GovernancePolicy>,
) -> Result<(u16, Vec<u8>, Option<u32>, Option<u32>)> {
    let start = Instant::now();
    let input_size = body.len() as u64;
    let body_str = String::from_utf8_lossy(&body);

    let (masked_request, out_body) = match policy {
        Some(p) => {
            if p.block_on_pii {
                let spans = privacy.detect_with_policy(&body_str, p)?;
                if !spans.is_empty() {
                    return Err(crate::error::AppError::BadRequest(
                        "Requisição contém dados sensíveis não permitidos.".into(),
                    ));
                }
            }
            let (masked, _) = privacy.scan_and_mask_with_policy(&body_str, p)?;
            (masked.clone(), masked.into_bytes())
        }
        None => {
            let (masked, _) = privacy.scan_and_mask(&body_str)?;
            (masked.clone(), masked.into_bytes())
        }
    };

    // Proxy: forward to LLM (ou mock se SECLLM_MOCK_LLM=1)
    let (status, response_body, prompt_tokens, completion_tokens) = proxy
        .forward(ctx, method, path, out_body, headers)
        .await?;

    let output_size = response_body.len() as u64;
    let response_str = String::from_utf8_lossy(&response_body);

    let (masked_response, final_body) = match policy {
        Some(p) if p.mask_response => {
            let (masked, _) = privacy.scan_and_mask_with_policy(&response_str, p)?;
            (masked.clone(), masked.into_bytes())
        }
        Some(_) => (response_str.to_string(), response_body),
        None => {
            let (masked, _) = privacy.scan_and_mask(&response_str)?;
            (masked.clone(), masked.into_bytes())
        }
    };

    let latency_ms = start.elapsed().as_millis() as u64;
    let status_label = if status >= 200 && status < 300 {
        "ok"
    } else {
        "error"
    };

    let model = model_from_body(&body);
    let event = AuditEvent::new(
        ctx.request_id,
        ctx.client_id.clone(),
        format!("{:?}", ctx.provider),
        model,
        prompt_tokens,
        completion_tokens,
        Some(latency_ms),
        status_label.to_string(),
        Some(input_size),
        Some(output_size),
        Some(masked_request),
        Some(masked_response),
    );

    logger.log_confirmed(event).await?;

    Ok((status, final_body, prompt_tokens, completion_tokens))
}
