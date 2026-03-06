# SecLLM – Docker

## Subir tudo

```bash
docker compose up -d --build
```

- **SecLLM:** http://localhost:3010 (porta 3010 no host → 3000 no container)  
- **Swagger UI (público):** http://localhost:3010/swagger-ui/  
- **RabbitMQ Management:** http://localhost:15672 (guest/guest)  
- **Redis:** localhost:6379  
- **PostgreSQL:** localhost:5432 (user: `secllm`, password: `secllm`, database: `secllm`)  
- **ClickHouse HTTP:** http://localhost:8123  

## Modo desenvolvimento (auto-reload)

Para que o container **recompile e reinicie sozinho** ao alterar código (sem reiniciar o container à mão):

```bash
docker compose -f docker-compose.yml -f docker-compose.dev.yml up --build
```

- O código do projeto é montado em **volume** (`.:/app`).
- Dentro do container roda **cargo watch**: ao salvar alterações em `src/`, `config/` ou `Cargo.toml`, o Rust recompila e o processo reinicia.
- O cache de compilação fica no volume `secllm_target`, para não perder entre reinícios.
- Deixe o comando rodando no terminal para ver os logs; ao editar e salvar, a recompilação e o restart aparecem ali.

### Só infraestrutura (app e front no Mac, sem container Rust)

Útil quando o disco é pouco ou o link no container dá OOM. Docker sobe só Redis, RabbitMQ, Postgres e ClickHouse; o backend e o front rodam no host.

```bash
make infra
```

Depois, em terminais separados:

```bash
# Backend (porta 3000)
cargo run

# Front (Next.js)
cd secllm-front && npm run dev
```

O `config/default.toml` já usa `127.0.0.1` para todos os serviços, porta **3010** para o backend e **5433** para Postgres (evita conflito com Postgres local em 5432). Para derrubar a infra: `make infra-down`.

Se aparecer **"database \"secllm\" does not exist"**, crie o banco (volume antigo sem o DB) e suba de novo:

```bash
docker exec secllm-postgres psql -U secllm -d postgres -c "CREATE DATABASE secllm;"
# Depois rode os scripts de schema (uma vez):
docker exec -i secllm-postgres psql -U secllm -d secllm < docker/postgres/01_schema.sql
docker exec -i secllm-postgres psql -U secllm -d secllm < docker/postgres/02_rls.sql
docker exec -i secllm-postgres psql -U secllm -d secllm < docker/postgres/03_add_gemini.sql
```

---

Para subir só a infra e o app em modo dev em background (não recomendado, pois você não vê o cargo watch):

```bash
docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d --build
```

## Atualizar o container com novo código (produção)

Depois de alterar o código Rust, é preciso **reconstruir a imagem e recriar o container** para o app usar o novo binário:

```bash
# Reconstruir a imagem da aplicação (sem cache para garantir)
docker compose build --no-cache secllm

# Subir de novo (recria o container com a nova imagem)
docker compose up -d secllm
```

Ou em um comando (reconstrói e sobe tudo):

```bash
docker compose up -d --build --force-recreate secllm
```

## Arquitetura de dados (Postgres SSOT + Redis cache)

- **PostgreSQL** é a fonte única da verdade: usuários, clientes (apps), client_secrets, api_keys, políticas de governança e logs de faturamento. O schema é aplicado na primeira subida via scripts em `docker/postgres/` (01_schema.sql, 02_rls.sql).
- **Redis** é um cache de credenciais: espelho das chaves de API e dos client secrets ativos. O gateway **só lê no Redis** no hot path (zero consulta ao Postgres por requisição).
- **Fluxo:** Dashboard/API persiste no Postgres (api_keys, client_secrets, etc.) e, em cada escrita bem-sucedida, **replica para o Redis** (SET/DEL nas chaves `secllm:vault:{client_id}:{provider}` e `secllm:auth:{client_id}`). O gateway continua usando apenas o Redis para obter a API key por requisição.

## Autenticação (Bearer token)

A rota **POST /auth/token** aceita dois fluxos:

1. **Client credentials** (proxy/API): `client_id` + `client_secret` (+ opcional `provider`).  
   - Com Postgres configurado: o secret é validado contra a tabela `client_secrets` (hash Argon2).  
   - Sem Postgres: o secret é validado contra o Redis (`secllm:auth:{client_id}`).

2. **User (dashboard):** `email` + `password`. Só funciona com Postgres. Retorna JWT com `scope` (admin/user) e `sub` = user_id.

**Exemplo – token com client_id (Redis, sem Postgres):**

```bash
# Definir o client_secret no Redis (uma vez por cliente)
docker exec secllm-redis redis-cli SET "secllm:auth:app1" "minha-senha-secreta"

curl -s -X POST http://localhost:3000/auth/token \
  -H "Content-Type: application/json" \
  -d '{"client_id":"app1","client_secret":"minha-senha-secreta","provider":"openai"}'
```

**Exemplo – token com client_id (Postgres):** após criar um cliente e um client_secret no Postgres (via API ou seed), use o mesmo body acima; a validação é feita contra o hash em `client_secrets`.

Resposta: `{"access_token":"eyJ...","token_type":"Bearer","expires_in":3600}`. Use `access_token` no header `Authorization: Bearer <token>` nas rotas protegidas.

## Chave de API (Vault)

O gateway lê a API key do Redis (`secllm:vault:{client_id}:{provider}`). Com Postgres ativo, o fluxo recomendado é **gerir chaves pela API**; a aplicação persiste no Postgres e replica para o Redis.

**Via API (Postgres + Redis):**

```bash
# Obter um token (user ou client com permissão sobre o client_id)
TOKEN="..."

# Criar/atualizar chave OpenAI para o cliente "app1"
curl -s -X PUT http://localhost:3000/api/v1/clients/app1/keys/openai \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"api_key":"sua-api-key-openai"}'

# Criar/atualizar chave Anthropic
curl -s -X PUT http://localhost:3000/api/v1/clients/app1/keys/anthropic \
  -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
  -d '{"api_key":"sua-api-key-anthropic"}'

# Remover chave
curl -s -X DELETE http://localhost:3000/api/v1/clients/app1/keys/openai \
  -H "Authorization: Bearer $TOKEN"
```

**Somente Redis (sem Postgres):** pode-se continuar definindo chaves manualmente no Redis:

```bash
docker exec secllm-redis redis-cli SET "secllm:vault:app1:openai" "sua-api-key-openai"
docker exec secllm-redis redis-cli SET "secllm:vault:app1:anthropic" "sua-api-key-anthropic"
```

O JWT (obtido via `/auth/token` ou gerado externamente) deve conter `client_id`/`sub` e opcionalmente `provider` ("openai" ou "anthropic").

## Postgres – init e credenciais

- **Porta:** 5432  
- **Usuário / senha / database:** `secllm` / `secllm` / `secllm`  
- **URL de conexão:** `postgres://secllm:secllm@localhost:5432/secllm`  
- **Init:** na primeira subida, o Postgres executa os scripts em `docker/postgres/` em ordem (`01_schema.sql`, `02_rls.sql`, `03_add_gemini.sql`), criando tabelas e o enum `llm_provider` (openai, anthropic, gemini). Se o banco foi criado antes de existir `03_add_gemini.sql`, rode manualmente: `docker exec -i secllm-postgres psql -U secllm -d secllm < docker/postgres/03_add_gemini.sql` (ajuste o nome do container se necessário).

## Parar

```bash
docker compose down
```

Com remoção de volumes (apaga dados de Redis, RabbitMQ, Postgres e ClickHouse):

```bash
docker compose down -v
```

## Troubleshooting

### Erro de I/O ao subir imagens (`blob sha256:... input/output error`)

O disco/cache do Docker (containerd) está corrompido ou com falha. Tente:

1. Remover a imagem afetada e puxar de novo:
   ```bash
   docker rmi rabbitmq:3-management-alpine
   docker compose pull && docker compose up -d
   ```

2. Limpar todo o cache e imagens não usadas (cuidado: remove imagens de outros projetos):
   ```bash
   docker system prune -a -f
   docker compose up -d
   ```

3. **Colima:** reiniciar a VM:
   ```bash
   colima stop && colima start
   docker compose up -d
   ```
   Se o erro persistir, recriar a VM (apaga imagens/containers locais):
   ```bash
   colima delete && colima start
   docker compose up -d
   ```

4. Verificar espaço em disco e integridade do filesystem (macOS: Utilitário de Disco).

### "failed to lookup address information: Name or service not known" / "Connection refused"

- **App rodando no host (`cargo run`):** a config está usando hostnames do Docker (`redis`, `rabbitmq`, `clickhouse`, `postgres`), que não existem no seu PC. Não exporte no terminal as variáveis do `docker-compose.yml`. Use só `config/default.toml` (já com `127.0.0.1`) ou defina `SECLLM__REDIS__URL=redis://127.0.0.1:6379`, etc., com **127.0.0.1**. Suba a infra antes: `docker compose up -d redis rabbitmq postgres clickhouse clickhouse-init`.
- **App rodando no container:** os hostnames (`redis`, `rabbitmq`, etc.) só funcionam dentro da rede do Compose. Confira se o serviço `secllm` está no mesmo `networks` e se os outros containers estão healthy (`docker compose ps`). O worker de audit inicia com um atraso de 3 segundos para o DNS do Docker estar pronto; se ainda aparecer "failed to lookup" no início, aguarde alguns segundos (o worker reconecta a cada 5 s até conseguir).
