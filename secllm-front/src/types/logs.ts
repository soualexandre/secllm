/** Uma linha de log de auditoria (formato retornado pelo backend). */
export interface LogEntry {
  request_id: string;
  client_id: string;
  provider: string;
  model: string | null;
  prompt_tokens: number | null;
  completion_tokens: number | null;
  latency_ms: number | null;
  status: string;
  input_size: number | null;
  output_size: number | null;
  /** Data formatada para exibição (ex: 01/03/2025 14:30). */
  created_at_formatted: string;
  created_at: string;
  /** Body da requisição (entrada do usuário). */
  request_body: string | null;
  /** Body da resposta LLM (saída). */
  response_body: string | null;
}

/** Resposta de GET /api/v1/logs */
export interface LogsResponse {
  items: LogEntry[];
  total: number;
}

/** Métricas agregadas (GET /api/v1/metrics). Suporta ?provider= & ?status= para filtrar. */
export interface MetricsResponse {
  total_requests: number;
  ok_count: number;
  error_count: number;
  avg_latency_ms: number | null;
  min_latency_ms: number | null;
  max_latency_ms: number | null;
  total_prompt_tokens: number;
  total_completion_tokens: number;
  by_provider: { provider: string; count: number }[];
  by_status: { status: string; count: number }[];
}
