# SecLLM – atalhos para Docker Compose (dev, prod, só infra)
# Uso: make infra + cargo run (app local) | make dev (tudo no Docker)

COMPOSE_BASE   := -f docker-compose.yml
COMPOSE_DEV    := -f docker-compose.yml -f docker-compose.dev.yml
COMPOSE_INFRA  := -f docker-compose.infra.yml

.PHONY: dev dev-build dev-down prod prod-build prod-down infra infra-down down logs help

# --- Desenvolvimento (hot-reload com cargo-watch) ---
dev:
	$(info Subindo stack em modo desenvolvimento (Ctrl+C para parar)...)
	@docker-compose $(COMPOSE_DEV) up --build

dev-build:
	@docker-compose $(COMPOSE_DEV) build

dev-down:
	@docker-compose $(COMPOSE_DEV) down

# --- Produção (imagem otimizada, restart unless-stopped) ---
prod:
	$(info Subindo stack em modo produção (Ctrl+C para parar)...)
	@docker-compose $(COMPOSE_BASE) up --build

prod-build:
	@docker-compose $(COMPOSE_BASE) up --build -d

prod-down:
	@docker-compose $(COMPOSE_BASE) down

# --- Só infraestrutura (app roda local: cargo run + front) ---
infra:
	$(info Subindo só Redis, RabbitMQ, Postgres, ClickHouse... Rode o backend: cargo run)
	@docker-compose $(COMPOSE_INFRA) up -d
	@echo "Infra no ar. Backend: cargo run (porta 3000). Front: cd secllm-front && npm run dev."

infra-down:
	@docker-compose $(COMPOSE_INFRA) down

# --- Comandos comuns ---
down:
	@docker-compose $(COMPOSE_DEV) down

logs:
	@docker-compose $(COMPOSE_BASE) logs -f

logs-dev:
	@docker-compose $(COMPOSE_DEV) logs -f

# --- Ajuda ---
help:
	@echo "SecLLM – alvos disponíveis:"
	@echo "  make infra      Só infra no Docker (Redis, RabbitMQ, Postgres, ClickHouse). Depois: cargo run"
	@echo "  make infra-down Parar só a infra"
	@echo "  make dev        Subir em desenvolvimento (hot-reload, foreground)"
	@echo "  make dev-build  Apenas build da imagem de dev (sem subir)"
	@echo "  make dev-down   Parar e remover containers do modo dev"
	@echo "  make prod       Subir em produção (foreground)"
	@echo "  make prod-build Subir em produção em background (-d) com rebuild"
	@echo "  make prod-down  Parar e remover containers do modo prod"
	@echo "  make down       Parar stack (usa arquivos do dev)"
	@echo "  make logs       Seguir logs (compose base)"
	@echo "  make logs-dev   Seguir logs (compose dev)"
	@echo "  make help       Mostrar esta ajuda"
