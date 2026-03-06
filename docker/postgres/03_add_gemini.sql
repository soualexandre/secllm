-- Add Gemini to LLM provider enum (run after 01_schema.sql and 02_rls.sql).
-- Idempotent: safe to run if 'gemini' already exists.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_enum e
    JOIN pg_type t ON e.enumtypid = t.oid
    WHERE t.typname = 'llm_provider' AND e.enumlabel = 'gemini'
  ) THEN
    ALTER TYPE llm_provider ADD VALUE 'gemini';
  END IF;
END
$$;
