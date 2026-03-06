//! Leitura de audit logs e métricas a partir do ClickHouse (SELECT).

use chrono::DateTime;
use clickhouse::Row;
use serde::{Deserialize, Serialize};

/// Formata created_at (RFC3339 ou similar) para exibição: "dd/MM/yyyy HH:mm".
fn format_created_at(s: &str) -> String {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.format("%d/%m/%Y %H:%M").to_string())
        .unwrap_or_else(|_| s.to_string())
}

/// Uma linha de log de auditoria retornada pela API (formato pronto para o frontend).
#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub request_id: String,
    pub client_id: String,
    pub provider: String,
    pub model: Option<String>,
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub latency_ms: Option<u64>,
    pub status: String,
    pub input_size: Option<u64>,
    pub output_size: Option<u64>,
    /// Data/hora formatada para exibição (ex: 01/03/2025 14:30).
    pub created_at_formatted: String,
    pub created_at: String,
    /// Body da requisição (entrada do usuário).
    pub request_body: Option<String>,
    /// Body da resposta LLM (saída).
    pub response_body: Option<String>,
}

/// Linha bruta do ClickHouse (schema mínimo: sem request_body/response_body).
#[derive(Row, Serialize, Deserialize)]
struct AuditLogRowMinimal {
    request_id: String,
    client_id: String,
    provider: String,
    model: Option<String>,
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    latency_ms: Option<u64>,
    status: String,
    created_at: String,
}

/// Linha com request_body e response_body (quando a tabela tiver as colunas).
#[derive(Row, Serialize, Deserialize)]
struct AuditLogRowFull {
    request_id: String,
    client_id: String,
    provider: String,
    model: Option<String>,
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    latency_ms: Option<u64>,
    status: String,
    created_at: String,
    request_body: String,
    response_body: String,
}

/// Métricas agregadas para o dashboard (retornadas pelo backend no formato esperado pelo frontend).
/// Suporta filtros opcionais (provider, status) para métricas por segmento.
#[derive(Debug, Clone, Serialize)]
pub struct MetricsResponse {
    pub total_requests: u64,
    pub ok_count: u64,
    pub error_count: u64,
    pub avg_latency_ms: Option<f64>,
    pub min_latency_ms: Option<u64>,
    pub max_latency_ms: Option<u64>,
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    pub by_provider: Vec<ProviderCount>,
    pub by_status: Vec<StatusCount>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderCount {
    pub provider: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatusCount {
    pub status: String,
    pub count: u64,
}

/// Filtros e ordenação para a listagem de logs.
#[derive(Default)]
pub struct LogsQueryParams {
    pub client_id: Option<String>,
    pub provider: Option<String>,
    pub status: Option<String>,
    pub sort: Option<String>,
    pub order: Option<String>,
}

fn sort_clause(sort: &str, order: &str) -> String {
    let ord = if order.eq_ignore_ascii_case("asc") { "ASC" } else { "DESC" };
    let col = match sort.to_lowercase().as_str() {
        "status" => "status",
        "latency_ms" | "latency" => "latency_ms",
        "prompt_tokens" | "tokens" => "prompt_tokens",
        "client_id" | "client" => "client_id",
        "provider" => "provider",
        _ => "created_at",
    };
    format!("{col} {ord}")
}

/// Consulta logs com paginação, filtros opcionais e ordenação.
pub async fn query_logs(
    client: &clickhouse::Client,
    table: &str,
    limit: u32,
    offset: u32,
    params: &LogsQueryParams,
) -> Result<(Vec<LogEntry>, u64), Box<dyn std::error::Error + Send + Sync>> {
    let table = table.replace([' ', ';'], "");
    let sort = params.sort.as_deref().unwrap_or("created_at").trim();
    let order = params.order.as_deref().unwrap_or("desc").trim();
    let order_sql = sort_clause(sort, order);

    let mut where_parts = Vec::<String>::new();

    if let Some(ref c) = params.client_id {
        if !c.trim().is_empty() {
            where_parts.push("client_id = ?".to_string());
        }
    }
    if let Some(ref p) = params.provider {
        if !p.trim().is_empty() {
            where_parts.push("provider = ?".to_string());
        }
    }
    if let Some(ref s) = params.status {
        if !s.trim().is_empty() {
            where_parts.push("status = ?".to_string());
        }
    }

    let where_sql = if where_parts.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", where_parts.join(" AND "))
    };

    #[derive(Row, Deserialize)]
    struct CountRow {
        total: u64,
    }

    let count_sql = format!("SELECT count() AS total FROM {table}{where_sql}");
    let mut count_q = client.query(&count_sql);
    if let Some(ref c) = params.client_id {
        if !c.trim().is_empty() {
            count_q = count_q.bind(c.trim());
        }
    }
    if let Some(ref p) = params.provider {
        if !p.trim().is_empty() {
            count_q = count_q.bind(p.trim());
        }
    }
    if let Some(ref s) = params.status {
        if !s.trim().is_empty() {
            count_q = count_q.bind(s.trim());
        }
    }
    let total: u64 = count_q.fetch_one::<CountRow>().await.map(|r| r.total)?;

    let cols = "request_id, client_id, provider, model, prompt_tokens, completion_tokens, latency_ms, status, created_at";
    let sql_full = format!(
        "SELECT {cols}, request_body, response_body FROM {table}{where_sql} ORDER BY {order_sql} LIMIT ? OFFSET ?"
    );
    let sql_minimal = format!(
        "SELECT {cols} FROM {table}{where_sql} ORDER BY {order_sql} LIMIT ? OFFSET ?"
    );

    fn bind_filters(
        q: clickhouse::query::Query,
        params: &LogsQueryParams,
        limit: u32,
        offset: u32,
    ) -> clickhouse::query::Query {
        let mut q = q;
        if let Some(ref c) = params.client_id {
            if !c.trim().is_empty() {
                q = q.bind(c.trim());
            }
        }
        if let Some(ref p) = params.provider {
            if !p.trim().is_empty() {
                q = q.bind(p.trim());
            }
        }
        if let Some(ref s) = params.status {
            if !s.trim().is_empty() {
                q = q.bind(s.trim());
            }
        }
        q.bind(limit).bind(offset)
    }

    let items: Vec<LogEntry> = match bind_filters(client.query(&sql_full), params, limit, offset)
        .fetch_all::<AuditLogRowFull>()
        .await
    {
        Ok(rows) => rows
            .into_iter()
            .map(|r| LogEntry {
                created_at_formatted: format_created_at(&r.created_at),
                request_body: Some(r.request_body).filter(|s: &String| !s.is_empty()),
                response_body: Some(r.response_body).filter(|s: &String| !s.is_empty()),
                ..log_entry_from_minimal(
                    r.request_id,
                    r.client_id,
                    r.provider,
                    r.model,
                    r.prompt_tokens,
                    r.completion_tokens,
                    r.latency_ms,
                    r.status,
                    None,
                    None,
                    r.created_at,
                )
            })
            .collect(),
        Err(_) => {
            let rows = bind_filters(client.query(&sql_minimal), params, limit, offset)
                .fetch_all::<AuditLogRowMinimal>()
                .await?;
            rows.into_iter()
                .map(|r| {
                    log_entry_from_minimal(
                        r.request_id,
                        r.client_id,
                        r.provider,
                        r.model,
                        r.prompt_tokens,
                        r.completion_tokens,
                        r.latency_ms,
                        r.status,
                        None,
                        None,
                        r.created_at,
                    )
                })
                .collect()
        }
    };

    Ok((items, total))
}

fn log_entry_from_minimal(
    request_id: String,
    client_id: String,
    provider: String,
    model: Option<String>,
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    latency_ms: Option<u64>,
    status: String,
    input_size: Option<u64>,
    output_size: Option<u64>,
    created_at: String,
) -> LogEntry {
    LogEntry {
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
        created_at_formatted: format_created_at(&created_at),
        created_at,
        request_body: None,
        response_body: None,
    }
}

/// Parâmetros opcionais para filtrar métricas (ex.: latência por status, por provider).
#[derive(Default)]
pub struct MetricsQueryParams {
    pub provider: Option<String>,
    pub status: Option<String>,
}

/// Retorna métricas agregadas (totais e por provider/status), opcionalmente filtradas.
pub async fn query_metrics(
    client: &clickhouse::Client,
    table: &str,
    params: &MetricsQueryParams,
) -> Result<MetricsResponse, Box<dyn std::error::Error + Send + Sync>> {
    let table = table.replace([' ', ';'], "");

    let mut where_parts = Vec::<String>::new();
    if let Some(ref p) = params.provider {
        if !p.trim().is_empty() {
            where_parts.push("provider = ?".to_string());
        }
    }
    if let Some(ref s) = params.status {
        if !s.trim().is_empty() {
            where_parts.push("status = ?".to_string());
        }
    }
    let where_sql = if where_parts.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", where_parts.join(" AND "))
    };

    #[derive(Row, Deserialize)]
    struct Totals {
        total: u64,
        ok_count: u64,
        error_count: u64,
        avg_latency: Option<f64>,
        min_latency: Option<u64>,
        max_latency: Option<u64>,
        sum_pt: Option<u64>,
        sum_ct: Option<u64>,
    }

    let totals_sql = format!(
        "SELECT count() AS total, countIf(status = 'ok') AS ok_count, countIf(status = 'error') AS error_count, avg(latency_ms) AS avg_latency, min(latency_ms) AS min_latency, max(latency_ms) AS max_latency, sum(prompt_tokens) AS sum_pt, sum(completion_tokens) AS sum_ct FROM {table}{where_sql}"
    );
    let mut totals_q = client.query(&totals_sql);
    if let Some(ref p) = params.provider {
        if !p.trim().is_empty() {
            totals_q = totals_q.bind(p.trim());
        }
    }
    if let Some(ref s) = params.status {
        if !s.trim().is_empty() {
            totals_q = totals_q.bind(s.trim());
        }
    }
    let totals = totals_q.fetch_one::<Totals>().await?;

    #[derive(Row, Deserialize)]
    struct ProviderRow {
        provider: String,
        count: u64,
    }
    let by_provider_sql = format!("SELECT provider, count() AS count FROM {table}{where_sql} GROUP BY provider ORDER BY count DESC");
    let mut by_provider_q = client.query(&by_provider_sql);
    if let Some(ref p) = params.provider {
        if !p.trim().is_empty() {
            by_provider_q = by_provider_q.bind(p.trim());
        }
    }
    if let Some(ref s) = params.status {
        if !s.trim().is_empty() {
            by_provider_q = by_provider_q.bind(s.trim());
        }
    }
    let by_provider: Vec<ProviderCount> = by_provider_q
        .fetch_all::<ProviderRow>()
        .await?
        .into_iter()
        .map(|r| ProviderCount {
            provider: r.provider,
            count: r.count,
        })
        .collect();

    #[derive(Row, Deserialize)]
    struct StatusRow {
        status: String,
        count: u64,
    }
    let by_status_sql = format!("SELECT status, count() AS count FROM {table}{where_sql} GROUP BY status ORDER BY count DESC");
    let mut by_status_q = client.query(&by_status_sql);
    if let Some(ref p) = params.provider {
        if !p.trim().is_empty() {
            by_status_q = by_status_q.bind(p.trim());
        }
    }
    if let Some(ref s) = params.status {
        if !s.trim().is_empty() {
            by_status_q = by_status_q.bind(s.trim());
        }
    }
    let by_status: Vec<StatusCount> = by_status_q
        .fetch_all::<StatusRow>()
        .await?
        .into_iter()
        .map(|r| StatusCount {
            status: r.status,
            count: r.count,
        })
        .collect();

    Ok(MetricsResponse {
        total_requests: totals.total,
        ok_count: totals.ok_count,
        error_count: totals.error_count,
        avg_latency_ms: totals.avg_latency,
        min_latency_ms: totals.min_latency,
        max_latency_ms: totals.max_latency,
        total_prompt_tokens: totals.sum_pt.unwrap_or(0),
        total_completion_tokens: totals.sum_ct.unwrap_or(0),
        by_provider,
        by_status,
    })
}
