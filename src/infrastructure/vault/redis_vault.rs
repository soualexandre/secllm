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

    fn auth_key(client_id: &str) -> String {
        format!("secllm:auth:{}", client_id)
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

    async fn get_client_secret(&self, client_id: &str) -> Result<Option<String>> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| crate::AppError::Vault(e.to_string()))?;
        let key = Self::auth_key(client_id);
        let value: Option<String> = conn.get(&key).await.map_err(|e| crate::AppError::Vault(e.to_string()))?;
        Ok(value)
    }

    async fn set_api_key(&self, client_id: &str, provider: &str, api_key: &str) -> Result<()> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| crate::AppError::Vault(e.to_string()))?;
        let key = Self::key(client_id, provider);
        conn.set::<_, ()>(&key, api_key)
            .await
            .map_err(|e| crate::AppError::Vault(e.to_string()))?;
        Ok(())
    }

    async fn del_api_key(&self, client_id: &str, provider: &str) -> Result<()> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| crate::AppError::Vault(e.to_string()))?;
        let key = Self::key(client_id, provider);
        conn.del::<_, ()>(&key)
            .await
            .map_err(|e| crate::AppError::Vault(e.to_string()))?;
        Ok(())
    }

    async fn set_client_secret(&self, client_id: &str, secret: &str) -> Result<()> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| crate::AppError::Vault(e.to_string()))?;
        let key = Self::auth_key(client_id);
        conn.set::<_, ()>(&key, secret)
            .await
            .map_err(|e| crate::AppError::Vault(e.to_string()))?;
        Ok(())
    }

    async fn del_client_secret(&self, client_id: &str) -> Result<()> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| crate::AppError::Vault(e.to_string()))?;
        let key = Self::auth_key(client_id);
        conn.del::<_, ()>(&key)
            .await
            .map_err(|e| crate::AppError::Vault(e.to_string()))?;
        Ok(())
    }
}
