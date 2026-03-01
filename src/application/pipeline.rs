//! Request pipeline: privacy in → proxy → privacy out → log (confirmed) → response.

use std::time::Instant;

use crate::domain::{AuditEvent, RequestContext};
use crate::application::ports::{LoggerPort, PrivacyPort, ProxyPort};
use crate::Result;

/// Runs the full pipeline and returns (status_code, body, prompt_tokens, completion_tokens).
/// Caller must have already set ctx (auth + vault layers).
pub async fn handle_request(
    ctx: &RequestContext,
    method: &str,
    path: &str,
    body: Vec<u8>,
    headers: Vec<(String, String)>,
    logger: &dyn LoggerPort,
    proxy: &dyn ProxyPort,
    privacy: &dyn PrivacyPort,
) -> Result<(u16, Vec<u8>, Option<u32>, Option<u32>)> {
    let start = Instant::now();
    let body_str = String::from_utf8_lossy(&body);

    // Privacy In: scan and mask request body
    let (masked_request, _) = privacy.scan_and_mask(&body_str)?;
    let out_body = masked_request.into_bytes();

    // Proxy: forward to LLM
    let (status, response_body, prompt_tokens, completion_tokens) = proxy
        .forward(ctx, method, path, out_body, headers)
        .await?;

    // Privacy Out: scan and mask response body
    let response_str = String::from_utf8_lossy(&response_body);
    let (masked_response, _) = privacy.scan_and_mask(&response_str)?;
    let final_body = masked_response.into_bytes();

    let latency_ms = start.elapsed().as_millis() as u64;
    let status_label = if status >= 200 && status < 300 {
        "ok"
    } else {
        "error"
    };

    let event = AuditEvent::new(
        ctx.request_id,
        ctx.client_id.clone(),
        format!("{:?}", ctx.provider),
        None,
        prompt_tokens,
        completion_tokens,
        Some(latency_ms),
        status_label.to_string(),
    );

    logger.log_confirmed(event).await?;

    Ok((status, final_body, prompt_tokens, completion_tokens))
}
