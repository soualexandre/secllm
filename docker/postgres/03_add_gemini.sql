-- Add Gemini to LLM provider enum (run after 01_schema.sql and 02_rls.sql)
ALTER TYPE llm_provider ADD VALUE 'gemini';
