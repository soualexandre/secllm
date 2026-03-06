//! Consume audit events from RabbitMQ, accumulate in dynamic batch, bulk insert to ClickHouse.

use crate::domain::AuditEvent;
use crate::infrastructure::logging::worker::DynamicBatch;
use clickhouse::Row;
use futures_util::StreamExt;
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties};
use serde::Serialize;
use std::time::Duration;
use tokio::time::{interval, sleep};

#[derive(Row, Serialize)]
pub struct AuditRow {
    request_id: String,
    client_id: String,
    provider: String,
    model: Option<String>,
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    latency_ms: Option<u64>,
    status: String,
    created_at: String,
}

pub struct WorkerConfig {
    pub amqp_url: String,
    pub queue: String,
    pub clickhouse_url: String,
    pub database: String,
    pub table: String,
    pub batch_max_size: usize,
    pub batch_max_latency_ms: u64,
}

/// Connect to RabbitMQ with retry (DNS/network may not be ready when worker starts).
async fn connect_amqp_with_retry(url: &str) -> Connection {
    let mut backoff = Duration::from_secs(1);
    const MAX_BACKOFF: Duration = Duration::from_secs(30);
    loop {
        match Connection::connect(url, ConnectionProperties::default()).await {
            Ok(conn) => return conn,
            Err(e) => {
                eprintln!("audit worker: RabbitMQ connection failed (retry in {:?}): {}", backoff, e);
                sleep(backoff).await;
                backoff = (backoff * 2).min(MAX_BACKOFF);
            }
        }
    }
}

/// Run the logging worker: connect to RabbitMQ, consume, batch, write to ClickHouse.
/// Reconnects with retry on connection failure (e.g. "Name or service not known" in Docker/Colima).
/// Never returns (runs until process exit).
pub async fn run_worker(config: WorkerConfig) -> ! {
    loop {
        if let Err(e) = run_worker_once(&config).await {
            let msg = e.to_string();
            eprintln!("audit worker error (will reconnect in 5s): {}", msg);
            if msg.contains("lookup") || msg.contains("Name or service not known") || msg.contains("dns error") {
                eprintln!("  Dica: se a aplicação roda no host (cargo run), use SECLLM__CLICKHOUSE__URL=http://127.0.0.1:8123");
            }
            sleep(Duration::from_secs(5)).await;
        }
    }
}

async fn run_worker_once(config: &WorkerConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let conn = connect_amqp_with_retry(&config.amqp_url).await;
    let channel = conn.create_channel().await?;

    let mut consumer = channel
        .basic_consume(
            &config.queue,
            "secllm-audit-worker",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let client = clickhouse::Client::default()
        .with_url(&config.clickhouse_url)
        .with_database(&config.database);

    let mut batch = DynamicBatch::new(config.batch_max_size, config.batch_max_latency_ms);
    let mut tick = interval(Duration::from_millis(config.batch_max_latency_ms / 2));

    loop {
        tokio::select! {
            delivery = consumer.next() => {
                let delivery = match delivery {
                    Some(Ok(d)) => d,
                    Some(Err(e)) => {
                        eprintln!("consumer error: {}", e);
                        continue;
                    }
                    None => break Ok(()),
                };
                if let Ok(event) = serde_json::from_slice::<AuditEvent>(&delivery.data) {
                    batch.push(event);
                    let _ = delivery.ack(BasicAckOptions::default()).await;
                }
                if let Some(events) = batch.take_ready() {
                    if let Err(e) = insert_batch_with_retry(&client, &config.table, &events).await {
                        eprintln!("clickhouse insert error (retries exhausted): {}", e);
                        return Err(e);
                    }
                }
            }
            _ = tick.tick() => {
                if let Some(events) = batch.take_ready() {
                    if let Err(e) = insert_batch_with_retry(&client, &config.table, &events).await {
                        eprintln!("clickhouse insert error (retries exhausted): {}", e);
                        return Err(e);
                    }
                }
            }
        }
    }
}

const INSERT_RETRIES: u32 = 5;
const INSERT_RETRY_BASE: Duration = Duration::from_secs(1);

/// Insert batch with retry on connection/DNS errors (e.g. "Name or service not known" at startup).
async fn insert_batch_with_retry(
    client: &clickhouse::Client,
    table: &str,
    events: &[AuditEvent],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if events.is_empty() {
        return Ok(());
    }
    let mut backoff = INSERT_RETRY_BASE;
    for attempt in 0..INSERT_RETRIES {
        match insert_batch(client, table, events).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                let msg = e.to_string();
                let is_connection_error = msg.contains("lookup")
                    || msg.contains("Name or service not known")
                    || msg.contains("dns error")
                    || msg.contains("connection")
                    || msg.contains("timed out");
                if attempt + 1 < INSERT_RETRIES && is_connection_error {
                    eprintln!("clickhouse insert error (attempt {}/{}), retry in {:?}: {}", attempt + 1, INSERT_RETRIES, backoff, msg);
                    sleep(backoff).await;
                    backoff = (backoff * 2).min(Duration::from_secs(30));
                } else {
                    return Err(e);
                }
            }
        }
    }
    unreachable!()
}

async fn insert_batch(
    client: &clickhouse::Client,
    table: &str,
    events: &[AuditEvent],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if events.is_empty() {
        return Ok(());
    }
    let rows: Vec<AuditRow> = events
        .iter()
        .map(|e| AuditRow {
            request_id: e.request_id.to_string(),
            client_id: e.client_id.clone(),
            provider: e.provider.clone(),
            model: e.model.clone(),
            prompt_tokens: e.prompt_tokens,
            completion_tokens: e.completion_tokens,
            latency_ms: e.latency_ms,
            status: e.status.clone(),
            created_at: e.created_at.to_rfc3339(),
        })
        .collect();
    let mut insert = client.insert(table)?;
    for row in rows {
        insert.write(&row).await?;
    }
    insert.end().await?;
    Ok(())
}
