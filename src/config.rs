//! Application configuration loaded from file and environment.

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub redis: RedisConfig,
    pub rabbitmq: RabbitMqConfig,
    pub clickhouse: ClickHouseConfig,
    pub jwt: JwtConfig,
    pub llm: LlmConfig,
    pub logging_worker: LoggingWorkerConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RabbitMqConfig {
    pub url: String,
    pub audit_exchange: String,
    pub audit_queue: String,
    pub audit_routing_key: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ClickHouseConfig {
    pub url: String,
    pub database: String,
    pub audit_table: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: Option<String>,
    pub audience: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LlmConfig {
    pub openai_base_url: String,
    pub anthropic_base_url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct LoggingWorkerConfig {
    pub batch_max_size: usize,
    pub batch_max_latency_ms: u64,
}

impl Config {
    pub fn load() -> crate::Result<Self> {
        let base = std::path::Path::new("config");
        let builder = config::Config::builder()
            .add_source(
                config::File::from(base.join("default.toml")).required(false),
            )
            .add_source(
                config::File::from(base.join("local.toml")).required(false),
            )
            .add_source(
                config::Environment::with_prefix("SECLLM").separator("__"),
            );
        let c = builder.build()?;
        c.try_deserialize().map_err(Into::into)
    }
}
