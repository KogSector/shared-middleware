//! Redis-based distributed cache

use crate::connectivity::{ConnectivityError, Result};
use redis::{aio::ConnectionManager, AsyncCommands, Client};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tracing::{debug, instrument};

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Redis connection URL
    pub redis_url: String,
    /// Default TTL in seconds
    pub default_ttl: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379".to_string(),
            default_ttl: Duration::from_secs(300),
        }
    }
}

/// Redis-based cache layer
pub struct CacheLayer {
    client: Client,
    config: CacheConfig,
}

impl CacheLayer {
    /// Create a new cache layer
    pub fn new(config: CacheConfig) -> Result<Self> {
        let client = Client::open(config.redis_url.as_str())
            .map_err(|e| ConnectivityError::Cache(e.to_string()))?;
        
        Ok(Self { client, config })
    }
    
    /// Get connection manager
    async fn get_connection(&self) -> Result<ConnectionManager> {
        ConnectionManager::new(self.client.clone())
            .await
            .map_err(|e| ConnectivityError::Redis(e))
    }
    
    /// Get a value from cache
    #[instrument(skip(self), fields(key = %key))]
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let mut conn = self.get_connection().await?;
        
        let value: Option<String> = conn.get(key).await?;
        
        match value {
            Some(v) => {
                debug!(key = %key, "Cache hit");
                let deserialized = serde_json::from_str(&v)
                    .map_err(|e| ConnectivityError::Serialization(e))?;
                Ok(Some(deserialized))
            }
            None => {
                debug!(key = %key, "Cache miss");
                Ok(None)
            }
        }
    }
    
    /// Set a value in cache with default TTL
    #[instrument(skip(self, value), fields(key = %key))]
    pub async fn set<T>(&self, key: &str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.set_ex(key, value, self.config.default_ttl).await
    }
    
    /// Set a value in cache with custom TTL
    #[instrument(skip(self, value), fields(key = %key, ttl_secs = ttl.as_secs()))]
    pub async fn set_ex<T>(&self, key: &str, value: &T, ttl: Duration) -> Result<()>
    where
        T: Serialize,
    {
        let mut conn = self.get_connection().await?;
        
        let serialized = serde_json::to_string(value)
            .map_err(|e| ConnectivityError::Serialization(e))?;
        
        conn.set_ex::<_, _, ()>(key, serialized, ttl.as_secs()).await?;
        
        debug!(key = %key, ttl_secs = ttl.as_secs(), "Value cached");
        Ok(())
    }
    
    /// Delete a value from cache
    #[instrument(skip(self), fields(key = %key))]
    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;
        conn.del::<_, ()>(key).await?;
        debug!(key = %key, "Cache entry deleted");
        Ok(())
    }
    
    /// Check if key exists
    #[instrument(skip(self), fields(key = %key))]
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let exists: bool = conn.exists(key).await?;
        Ok(exists)
    }
    
    /// Cache-aside pattern: get or fetch and cache
    #[instrument(skip(self, fetcher), fields(key = %key))]
    pub async fn get_or_fetch<T, F, Fut>(&self, key: &str, fetcher: F) -> Result<T>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Try cache first
        if let Some(cached) = self.get(key).await? {
            return Ok(cached);
        }
        
        // Cache miss - fetch from source
        debug!(key = %key, "Fetching from source");
        let value = fetcher().await?;
        
        // Store in cache
        self.set(key, &value).await?;
        
        Ok(value)
    }
    
    /// Invalidate cache entries by pattern
    #[instrument(skip(self), fields(pattern = %pattern))]
    pub async fn invalidate_pattern(&self, pattern: &str) -> Result<u64> {
        let mut conn = self.get_connection().await?;
        
        // Get all keys matching pattern
        let keys: Vec<String> = conn.keys(pattern).await?;
        
        if keys.is_empty() {
            return Ok(0);
        }
        
        let count = keys.len() as u64;
        conn.del::<_, ()>(&keys).await?;
        
        debug!(pattern = %pattern, count = count, "Cache entries invalidated");
        Ok(count)
    }
}
