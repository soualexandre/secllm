//! OpenAPI 3 document and Swagger UI wiring.

use axum::response::IntoResponse;
use axum::Json;
use utoipa::openapi::security::{HttpBuilder, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::infrastructure::http::routes;

/// API error body returned on 4xx/5xx.
#[derive(utoipa::ToSchema)]
pub struct ApiError {
    /// Error message.
    pub error: String,
}

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "SecLLM API",
        version = "0.1",
        description = "Proxy de governança para LLMs (OpenAI/Anthropic). \
            **Fluxo recomendado:** 1) Registrar usuário (POST /auth/register ou /api/users/register). \
            2) Login (POST /auth/token) com email+senha ou client_id+client_secret para obter o Bearer token. \
            3) Gerenciar chaves no cofre (PUT/DELETE /api/v1/clients/.../keys/... e .../secret). \
            4) Chamar o gateway (qualquer path sob a raiz, ex: POST /v1/chat/completions) com header Authorization: Bearer <token>."
    ),
    tags(
        (name = "1 - Autenticação", description = "Registro e login para obter Bearer token"),
        (name = "2 - Cofre (Vault)", description = "CRUD de chaves de API e client secrets"),
        (name = "3 - Governança", description = "Políticas global e por cliente (JSONB)"),
        (name = "4 - Faturamento", description = "Logs de faturamento por período"),
        (name = "5 - Gateway LLM", description = "Proxy para OpenAI/Anthropic com privacidade e auditoria")
    ),
    paths(
        routes::health,
        routes::auth_token,
        routes::register_user,
        routes::put_api_key,
        routes::delete_api_key,
        routes::put_client_secret,
        routes::delete_client_secret,
        routes::get_governance_global,
        routes::put_governance_global,
        routes::get_governance_client,
        routes::put_governance_client,
        routes::post_billing_log,
        routes::proxy_handler_doc
    ),
    components(schemas(
        routes::AuthTokenRequest,
        routes::AuthTokenResponse,
        routes::RegisterRequest,
        routes::RegisterResponse,
        routes::PutApiKeyBody,
        routes::PutClientSecretBody,
        routes::PutGovernanceBody,
        routes::PostBillingLogBody,
        ApiError
    )),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Serves the OpenAPI JSON at GET /api-docs/openapi.json (public, no auth).
pub async fn serve_openapi_json() -> impl IntoResponse {
    Json(ApiDoc::openapi())
}
