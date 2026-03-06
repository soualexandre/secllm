# 🛡️ SecLLM (Security & Governance LLM Gateway)

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Architecture](https://img.shields.io/badge/arch-Hexagonal-green.svg)](#-arquitetura)
[![Performance](https://img.shields.io/badge/latency-sub--5ms-brightgreen)](#-benchmarks)

**SecLLM** é um Gateway de Governança de IA de ultra-alta performance, desenvolvido em **Rust**. Ele atua como um Proxy Reverso inteligente posicionado entre suas aplicações e os provedores de LLM (OpenAI, Anthropic, Gemini, etc.), garantindo que cada interação seja auditada, anonimizada e segura.

Projetado para ambientes corporativos e governamentais onde a **privacidade de dados** e a **soberania das chaves de API** são inegociáveis.

---

## ✨ Funcionalidades Principais

* **🔑 API Vault:** Centralize suas chaves de API em um cofre seguro (Redis). Suas aplicações consomem o gateway e nunca tocam nas chaves reais.
* **🕵️ Real-time PII Masking:** Detecção e anonimização automática de dados sensíveis (CPF, CNPJ, nomes, segredos) em milissegundos.
* **📊 Async Audit Logging:** Registro massivo de auditoria utilizando **RabbitMQ** e **ClickHouse** para persistência assíncrona resiliente.
* **🚀 Zero-Cost Abstraction:** Latência mínima adicionada ao fluxo de IA (sub-5ms) graças ao runtime assíncrono **Tokio**.
* **🛡️ Policy Enforcement:** Bloqueio ou moderação de prompts baseado em contexto e diretrizes de governança (Allow/Deny lists).

---

## 🏗️ Arquitetura

O SecLLM utiliza **Arquitetura Hexagonal (Clean Architecture)** para garantir que as regras de governança sejam independentes de provedores externos ou bancos de dados.



### Stack Tecnológico
* **Engine:** Rust 1.75+ (Axum Framework)
* **Runtime:** Tokio (Multi-threaded I/O)
* **Message Broker:** RabbitMQ (Buffer de resiliência de logs)
* **Analytics DB:** ClickHouse (Logs colunares para auditoria massiva)
* **Cache/Vault:** Redis (Acesso ultra-rápido a credenciais e políticas)

---

## 🚀 Como Começar (Quick Start)

### Pré-requisitos
* Rust Toolchain (Stable)
* Docker & Docker Compose (para infraestrutura de apoio)

### Instalação

1.  **Clone o repositório:**
    ```bash
    git clone [https://github.com/seu-usuario/secllm.git](https://github.com/seu-usuario/secllm.git)
    cd secllm
    ```

2.  **Suba toda a stack (Redis, RabbitMQ, ClickHouse e SecLLM) com Docker:**
    ```bash
    docker compose up -d --build
    ```
    A API ficará em **http://localhost:3010** (porta 3010 no host). Detalhes (incl. como configurar chaves no Redis) em [docker/README.md](docker/README.md).

    **Desenvolvimento com auto-reload:** para recompilar e reiniciar o app ao alterar o código, sem reiniciar o container:
    ```bash
    docker compose -f docker-compose.yml -f docker-compose.dev.yml up --build
    ```
    Veja [docker/README.md](docker/README.md#modo-desenvolvimento-auto-reload).

    *Alternativa:* subir só a infraestrutura e rodar a app no host (veja abaixo).

3.  **Rodar a aplicação localmente (`cargo run`):**
    - Suba só os serviços de infraestrutura (as portas ficam expostas no host):
      ```bash
      docker compose up -d redis rabbitmq postgres clickhouse clickhouse-init
      ```
    - **Não** exporte no seu shell as variáveis de ambiente do Docker Compose (elas usam hostnames como `redis`, `rabbitmq`, que **não resolvem** no host e geram *"failed to lookup address information: Name or service not known"*).
    - O `config/default.toml` já usa `127.0.0.1` para tudo. Se precisar sobrescrever, use `config/local.toml` ou variáveis `SECLLM__REDIS__URL`, `SECLLM__RABBITMQ__URL`, etc., sempre com **localhost/127.0.0.1**.
    - Execute:
      ```bash
      cargo run --release
      ```
    - A API ficará em **http://localhost:3000**. Swagger: **http://localhost:3000/swagger-ui/**.

4.  **Erros "Name or service not known" ou "Connection refused":**
    - Significam que a aplicação está tentando conectar a Redis, RabbitMQ, ClickHouse ou Postgres em um host/porta inacessível.
    - **Se estiver rodando no host:** use apenas `127.0.0.1` nas URLs e garanta que os containers da infraestrutura estão de pé (`docker compose ps`). Não use variáveis de ambiente com hostnames do Docker (`redis`, `rabbitmq`, etc.) no seu terminal.
    - **Se estiver rodando no Docker:** o `docker-compose.yml` já define os hostnames corretos; garanta que todos os serviços estão na mesma rede e saudáveis.

---

## 🛠️ Pipeline de Governança

Cada requisição passa por um pipeline modular de processamento:



1.  **Auth Layer:** Validação de identidade e permissões da aplicação origem.
2.  **Vault Layer:** Injeção dinâmica da API Key do provedor final recuperada do Redis.
3.  **Privacy In:** Scan de PII (Personally Identifiable Information) no prompt.
4.  **Dispatch:** Encaminhamento seguro via Proxy para a LLM (OpenAI/Anthropic).
5.  **Privacy Out:** Verificação de vazamento de dados sensíveis na resposta da IA.
6.  **Async Log:** Disparo do log de auditoria para o RabbitMQ (sem travar a resposta).

---

## 📈 Benchmarks

A performance é o nosso maior diferencial. O SecLLM foi testado para lidar com volumes massivos de dados.

| Cenário | Latência Adicional | Throughput (Req/sec) |
| :--- | :--- | :--- |
| **Proxy Direto** | 1.1ms | 18.000+ |
| **PII Masking Ativo** | 3.5ms | 9.200+ |
| **Fluxo Completo (Log + Vault)** | 4.2ms | 7.500+ |

*Testes realizados em hardware padrão: 8 vCPU, 16GB RAM.*

---

## 🤝 Contribuição

Contribuições tornam a comunidade open source incrível. Siga os passos:

1.  Faça um **Fork** do projeto.
2.  Crie uma **Branch** (`git checkout -b feature/NovaFuncionalidade`).
3.  Faça o **Commit** (`git commit -m 'Adiciona funcionalidade X'`).
4.  Faça o **Push** (`git push origin feature/NovaFuncionalidade`).
5.  Abra um **Pull Request**.

---

## 📄 Licença

Distribuído sob a licença **Apache 2.0**. Veja o arquivo `LICENSE` para mais detalhes.

---
Desenvolvido por **[Seu Nome/Empresa]** - *Garantindo a soberania de dados na era da IA.*