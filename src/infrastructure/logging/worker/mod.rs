//! Worker: consume from RabbitMQ, dynamic batch, bulk insert to ClickHouse.

mod batch;
mod clickhouse_writer;

pub use batch::DynamicBatch;
pub use clickhouse_writer::run_worker;
