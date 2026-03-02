-- Row-Level Security: users see only their own resources

ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE clients ENABLE ROW LEVEL SECURITY;
ALTER TABLE client_secrets ENABLE ROW LEVEL SECURITY;
ALTER TABLE api_keys ENABLE ROW LEVEL SECURITY;
ALTER TABLE governance_policies ENABLE ROW LEVEL SECURITY;
ALTER TABLE billing_logs ENABLE ROW LEVEL SECURITY;

-- users: users can read/update own row; admins can read all; INSERT allowed (app uses signup/admin)
CREATE POLICY users_select_update_delete ON users
    FOR ALL USING (
        id = current_setting('app.current_user_id', true)::UUID
        OR EXISTS (SELECT 1 FROM users u WHERE u.id = current_setting('app.current_user_id', true)::UUID AND u.role = 'admin')
    );
CREATE POLICY users_insert ON users FOR INSERT WITH CHECK (true);

-- clients: owner can do everything; admin can read all
CREATE POLICY clients_owner ON clients
    FOR ALL
    USING (
        user_id = current_setting('app.current_user_id', true)::UUID
        OR EXISTS (SELECT 1 FROM users u WHERE u.id = current_setting('app.current_user_id', true)::UUID AND u.role = 'admin')
    )
    WITH CHECK (
        user_id = current_setting('app.current_user_id', true)::UUID
        OR EXISTS (SELECT 1 FROM users u WHERE u.id = current_setting('app.current_user_id', true)::UUID AND u.role = 'admin')
    );

-- client_secrets: same as clients (via client ownership)
CREATE POLICY client_secrets_owner ON client_secrets
    FOR ALL
    USING (
        EXISTS (
            SELECT 1 FROM clients c
            WHERE c.id = client_secrets.client_id
            AND (c.user_id = current_setting('app.current_user_id', true)::UUID
                 OR EXISTS (SELECT 1 FROM users u WHERE u.id = current_setting('app.current_user_id', true)::UUID AND u.role = 'admin'))
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM clients c
            WHERE c.id = client_secrets.client_id
            AND (c.user_id = current_setting('app.current_user_id', true)::UUID
                 OR EXISTS (SELECT 1 FROM users u WHERE u.id = current_setting('app.current_user_id', true)::UUID AND u.role = 'admin'))
        )
    );

-- api_keys: same
CREATE POLICY api_keys_owner ON api_keys
    FOR ALL
    USING (
        EXISTS (
            SELECT 1 FROM clients c
            WHERE c.id = api_keys.client_id
            AND (c.user_id = current_setting('app.current_user_id', true)::UUID
                 OR EXISTS (SELECT 1 FROM users u WHERE u.id = current_setting('app.current_user_id', true)::UUID AND u.role = 'admin'))
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM clients c
            WHERE c.id = api_keys.client_id
            AND (c.user_id = current_setting('app.current_user_id', true)::UUID
                 OR EXISTS (SELECT 1 FROM users u WHERE u.id = current_setting('app.current_user_id', true)::UUID AND u.role = 'admin'))
        )
    );

-- governance_policies: global readable by all; client-scoped by owner
CREATE POLICY governance_policies_read ON governance_policies
    FOR SELECT USING (
        scope = 'global'
        OR (scope = 'client' AND client_id IN (
            SELECT id FROM clients WHERE user_id = current_setting('app.current_user_id', true)::UUID
        ))
        OR EXISTS (SELECT 1 FROM users u WHERE u.id = current_setting('app.current_user_id', true)::UUID AND u.role = 'admin')
    );
CREATE POLICY governance_policies_write ON governance_policies
    FOR ALL
    USING (
        (scope = 'client' AND client_id IN (SELECT id FROM clients WHERE user_id = current_setting('app.current_user_id', true)::UUID))
        OR EXISTS (SELECT 1 FROM users u WHERE u.id = current_setting('app.current_user_id', true)::UUID AND u.role = 'admin')
    )
    WITH CHECK (
        (scope = 'client' AND client_id IN (SELECT id FROM clients WHERE user_id = current_setting('app.current_user_id', true)::UUID))
        OR EXISTS (SELECT 1 FROM users u WHERE u.id = current_setting('app.current_user_id', true)::UUID AND u.role = 'admin')
    );

-- billing_logs: owner or admin
CREATE POLICY billing_logs_owner ON billing_logs
    FOR ALL
    USING (
        user_id = current_setting('app.current_user_id', true)::UUID
        OR EXISTS (SELECT 1 FROM users u WHERE u.id = current_setting('app.current_user_id', true)::UUID AND u.role = 'admin')
    )
    WITH CHECK (
        user_id = current_setting('app.current_user_id', true)::UUID
        OR EXISTS (SELECT 1 FROM users u WHERE u.id = current_setting('app.current_user_id', true)::UUID AND u.role = 'admin')
    );
