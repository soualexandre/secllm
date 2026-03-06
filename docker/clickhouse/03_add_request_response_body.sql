-- Migração: adiciona colunas de body (entrada do usuário e saída da LLM).
ALTER TABLE secllm.audit_events ADD COLUMN IF NOT EXISTS request_body String DEFAULT '';
ALTER TABLE secllm.audit_events ADD COLUMN IF NOT EXISTS response_body String DEFAULT '';
