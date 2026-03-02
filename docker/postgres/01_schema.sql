-- SecLLM – PostgreSQL schema (Single Source of Truth)
-- Run by Postgres in /docker-entrypoint-initdb.d/

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TYPE user_role AS ENUM ('user', 'admin');
CREATE TYPE policy_scope AS ENUM ('global', 'client');
CREATE TYPE llm_provider AS ENUM ('openai', 'anthropic');

-- Users (platform login)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    name TEXT,
    password_hash TEXT NOT NULL,
    role user_role NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_users_email ON users(email);

-- Clients (apps) – owned by a user; client_id is the identifier used in JWT and Redis
CREATE TABLE clients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id TEXT NOT NULL UNIQUE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_clients_user_id ON clients(user_id);
CREATE INDEX idx_clients_client_id ON clients(client_id);

-- One active secret per client (for gateway login with client_id + client_secret)
CREATE TABLE client_secrets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    secret_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(client_id)
);

CREATE INDEX idx_client_secrets_client_id ON client_secrets(client_id);

-- API keys per client + provider; replicated to Redis on write/delete
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    provider llm_provider NOT NULL,
    encrypted_key TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(client_id, provider)
);

CREATE INDEX idx_api_keys_client_id ON api_keys(client_id);

-- Governance policies (JSONB per scope)
CREATE TABLE governance_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scope policy_scope NOT NULL,
    client_id UUID REFERENCES clients(id) ON DELETE CASCADE,
    policy JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT governance_client_scope CHECK (
        (scope = 'global' AND client_id IS NULL) OR
        (scope = 'client' AND client_id IS NOT NULL)
    )
);

CREATE INDEX idx_governance_scope ON governance_policies(scope);
CREATE INDEX idx_governance_client_id ON governance_policies(client_id);

-- Billing logs (summaries / invoices)
CREATE TABLE billing_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    client_id UUID REFERENCES clients(id) ON DELETE SET NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    amount_cents BIGINT NOT NULL DEFAULT 0,
    details JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_billing_logs_user_id ON billing_logs(user_id);
CREATE INDEX idx_billing_logs_period ON billing_logs(period_start, period_end);

-- Trigger to update updated_at on users
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE PROCEDURE set_updated_at();

CREATE TRIGGER governance_policies_updated_at
    BEFORE UPDATE ON governance_policies
    FOR EACH ROW EXECUTE PROCEDURE set_updated_at();

CREATE TRIGGER api_keys_updated_at
    BEFORE UPDATE ON api_keys
    FOR EACH ROW EXECUTE PROCEDURE set_updated_at();
