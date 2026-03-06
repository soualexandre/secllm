CREATE TABLE IF NOT EXISTS secllm.audit_events (
  request_id String,
  client_id String,
  provider String,
  model Nullable(String),
  prompt_tokens Nullable(UInt32),
  completion_tokens Nullable(UInt32),
  latency_ms Nullable(UInt64),
  status String,
  input_size Nullable(UInt64),
  output_size Nullable(UInt64),
  request_body String DEFAULT '',
  response_body String DEFAULT '',
  created_at String
) ENGINE = MergeTree()
ORDER BY (client_id, created_at);
