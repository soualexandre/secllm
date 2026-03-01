//! Publish audit events to RabbitMQ with Publisher Confirms.

use async_trait::async_trait;
use lapin::{options::BasicPublishOptions, options::ConfirmSelectOptions, BasicProperties, Channel};
use std::sync::Arc;

use crate::application::ports::LoggerPort;
use crate::domain::AuditEvent;
use crate::Result;

pub struct RabbitMqPublisher {
    channel: Arc<Channel>,
    exchange: String,
    routing_key: String,
}

impl RabbitMqPublisher {
    pub fn new(channel: Channel, exchange: String, routing_key: String) -> Self {
        Self {
            channel: Arc::new(channel),
            exchange,
            routing_key,
        }
    }

    /// Enable publisher confirms on the channel. Call once after creating the channel (e.g. in main).
    pub async fn enable_confirms(channel: &Channel) -> Result<()> {
        channel
            .confirm_select(ConfirmSelectOptions { nowait: false })
            .await
            .map_err(|e| crate::AppError::Logging(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl LoggerPort for RabbitMqPublisher {
    async fn log_confirmed(&self, event: AuditEvent) -> Result<()> {
        let payload = serde_json::to_vec(&event).map_err(|e| crate::AppError::Logging(e.to_string()))?;
        self.channel
            .basic_publish(
                &self.exchange,
                &self.routing_key,
                BasicPublishOptions::default(),
                payload,
                BasicProperties::default(),
            )
            .await
            .map_err(|e| crate::AppError::Logging(e.to_string()))?
            .await
            .map_err(|e| crate::AppError::Logging(e.to_string()))?;
        Ok(())
    }
}
