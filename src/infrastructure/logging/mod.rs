//! Logging adapter – RabbitMQ publisher (LoggerPort) and worker (batch → ClickHouse).

mod rabbitmq_publisher;

pub mod worker;

pub use rabbitmq_publisher::RabbitMqPublisher;
