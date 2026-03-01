//! Consume audit events from RabbitMQ, accumulate in dynamic batch, bulk insert to ClickHouse.

use crate::domain::AuditEvent;
use crate::infrastructure::logging::worker::DynamicBatch;
use clickhouse::Row;
use futures_util::StreamExt;
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties};
use serde::Serialize;
use std::time::Duration;
use tokio::time::interval;

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

/// Run the logging worker: connect to RabbitMQ, consume, batch, write to ClickHouse.
pub async fn run_worker(config: WorkerConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let conn = Connection::connect(&config.amqp_url, ConnectionProperties::default()).await?;
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
                    None => break,
                };
                if let Ok(event) = serde_json::from_slice::<AuditEvent>(&delivery.data) {
                    batch.push(event);
                    let _ = delivery.ack(BasicAckOptions::default()).await;
                }
                if let Some(events) = batch.take_ready() {
                    if let Err(e) = insert_batch(&client, &config.table, &events).await {
                        eprintln!("clickhouse insert error: {}", e);
                    }
                }
            }
            _ = tick.tick() => {
                if let Some(events) = batch.take_ready() {
                    if let Err(e) = insert_batch(&client, &config.table, &events).await {
                        eprintln!("clickhouse insert error: {}", e);
                    }
                }
            }
        }
    }
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
