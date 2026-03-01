//! Redis-backed vault: client_id + provider -> API key.

use async_trait::async_trait;
use redis::AsyncCommands;

use crate::application::ports::VaultPort;
use crate::Result;

pub struct RedisVault {
    client: redis::Client,
}

impl RedisVault {
    pub fn new(url: &str) -> Result<Self> {
        let client = redis::Client::open(url).map_err(|e| crate::AppError::Vault(e.to_string()))?;
        Ok(Self { client })
    }

    fn key(client_id: &str, provider: &str) -> String {
        format!("secllm:vault:{}:{}", client_id, provider)
    }
}

#[async_trait]
impl VaultPort for RedisVault {
    async fn get_api_key(&self, client_id: &str, provider: &str) -> Result<String> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| crate::AppError::Vault(e.to_string()))?;
        let key = Self::key(client_id, provider);
        let value: String = conn
            .get(&key)
            .await
            .map_err(|e| crate::AppError::Vault(e.to_string()))?;
        Ok(value)
    }
}
