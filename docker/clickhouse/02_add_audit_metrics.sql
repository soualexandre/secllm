-- Migração: adiciona colunas de métricas de entrada/saída à audit_events.
-- Execute manualmente se a tabela já existia antes (CREATE IF NOT EXISTS não altera tabelas existentes).
ALTER TABLE secllm.audit_events ADD COLUMN IF NOT EXISTS input_size Nullable(UInt64);
ALTER TABLE secllm.audit_events ADD COLUMN IF NOT EXISTS output_size Nullable(UInt64);
