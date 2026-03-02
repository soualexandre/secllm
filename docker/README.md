# SecLLM – Docker

## Subir tudo

```bash
docker compose up -d --build
```

- **SecLLM:** http://localhost:3000  
- **RabbitMQ Management:** http://localhost:15672 (guest/guest)  
- **Redis:** localhost:6379  
- **ClickHouse HTTP:** http://localhost:8123  

## Autenticação (Bearer token)

A rota **POST /auth/token** gera um JWT em troca de `client_id` + `client_secret`. O secret é validado no Redis.

**1. Definir o client_secret no Redis** (uma vez por cliente):

```bash
# Cliente "app1" com secret "minha-senha-secreta"
docker exec secllm-redis redis-cli SET "secllm:auth:app1" "minha-senha-secreta"
```

**2. Obter o Bearer token:**

```bash
curl -s -X POST http://localhost:3000/auth/token \
  -H "Content-Type: application/json" \
  -d '{"client_id":"app1","client_secret":"minha-senha-secreta","provider":"openai"}'
```

Resposta: `{"access_token":"eyJ...","token_type":"Bearer","expires_in":3600}`. Use `access_token` no header `Authorization: Bearer <token>` nas rotas protegidas.

## Chave de API no Redis (Vault)

Para o proxy funcionar, o Redis precisa ter ao menos uma chave por cliente/provedor:

```bash
# Exemplo: cliente "app1" usando OpenAI
docker exec secllm-redis redis-cli SET "secllm:vault:app1:openai" "sua-api-key-openai"

# Exemplo: cliente "app1" usando Anthropic
docker exec secllm-redis redis-cli SET "secllm:vault:app1:anthropic" "sua-api-key-anthropic"
```

O JWT (obtido via `/auth/token` ou gerado externamente) deve conter `client_id`/`sub` e opcionalmente `provider` ("openai" ou "anthropic").

## Parar

```bash
docker compose down
```

Com remoção de volumes (apaga dados de Redis, RabbitMQ e ClickHouse):

```bash
docker compose down -v
```
