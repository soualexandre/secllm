//! SecLLM – bootstrap: config, single RabbitMQ connection, AppState, worker spawn, serve.

use std::sync::Arc;

use lapin::{Connection, ConnectionProperties};
use sqlx::postgres::PgPoolOptions;
use secllm::config::Config;
use secllm::domain::GovernancePolicy;
use secllm::infrastructure::http::{router, AppState};
use secllm::infrastructure::logging::{worker, RabbitMqPublisher};
use secllm::infrastructure::privacy::PrivacyService;
use secllm::infrastructure::proxy::ReqwestDispatcher;
use secllm::infrastructure::vault::RedisVault;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::load()?;

    let vault = Arc::new(RedisVault::new(&config.redis.url)?);
    let proxy = Arc::new(ReqwestDispatcher::new(
        config.llm.openai_base_url.clone(),
        config.llm.anthropic_base_url.clone(),
    )?);
    let governance = GovernancePolicy::default_strict();
    let privacy = Arc::new(PrivacyService::new(governance.clone()));

    let postgres = match config.postgres.as_ref().filter(|p| !p.url.is_empty()) {
        Some(p) => Some(PgPoolOptions::new().connect(&p.url).await?),
        None => None,
    };

    let conn = Connection::connect(&config.rabbitmq.url, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;
    RabbitMqPublisher::enable_confirms(&channel).await?;
    let logger = Arc::new(RabbitMqPublisher::new(
        channel,
        config.rabbitmq.audit_exchange.clone(),
        config.rabbitmq.audit_routing_key.clone(),
    ));

    let state = Arc::new(AppState {
        vault,
        logger,
        proxy,
        privacy,
        governance,
        postgres,
    });

    let worker_config = worker::WorkerConfig {
        amqp_url: config.rabbitmq.url.clone(),
        queue: config.rabbitmq.audit_queue.clone(),
        clickhouse_url: config.clickhouse.url.clone(),
        database: config.clickhouse.database.clone(),
        table: config.clickhouse.audit_table.clone(),
        batch_max_size: config.logging_worker.batch_max_size,
        batch_max_latency_ms: config.logging_worker.batch_max_latency_ms,
    };
    tokio::spawn(async move {
        if let Err(e) = worker::run_worker(worker_config).await {
            eprintln!("audit worker error: {}", e);
        }
    });

    let app = router(state);
    let addr = std::net::SocketAddr::from((
        config.server.host.parse::<std::net::IpAddr>()?,
        config.server.port,
    ));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("SecLLM listening on http://{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}
