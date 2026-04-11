//! FalkorDB client — Redis-protocol connectivity for ConFuse platform.
//!
//! Replaces the former Memgraph/Bolt-based graph client.
//! FalkorDB speaks the standard Redis wire protocol, so we connect
//! with the `redis` and `bb8-redis` crates.

use redis::cmd;
use bb8_redis::{bb8::Pool, RedisConnectionManager};
use std::sync::Arc;
use anyhow::{anyhow, Result};

/// A shared, cloneable handle to a FalkorDB / Redis database connection pool.
#[derive(Clone)]
pub struct FalkorClient {
    pool: Arc<Pool<RedisConnectionManager>>,
    /// Default database/graph name.
    pub graph_name: String,
}

impl FalkorClient {
    /// Connect to FalkorDB over Redis protocol.
    pub async fn new(
        host: &str,
        port: u16,
        graph_name: &str,
        username: &str,
        password: &str,
    ) -> Result<Self> {
        let auth = if !password.is_empty() {
            if !username.is_empty() {
                format!("{}:{}@", username, password)
            } else {
                format!(":{}@", password)
            }
        } else {
            String::new()
        };
        let uri = format!("redis://{}{}:{}", auth, host, port);

        let manager = RedisConnectionManager::new(uri).map_err(|e| anyhow!("Redis connection error: {}", e))?;
        let pool = Pool::builder()
            .max_size(16)
            .build(manager)
            .await
            .map_err(|e| anyhow!("Redis pool error: {}", e))?;

        Ok(Self {
            pool: Arc::new(pool),
            graph_name: graph_name.to_string(),
        })
    }

    /// Execute a graph query
    pub async fn graph_query(&self, query: &str) -> Result<redis::Value> {
        let mut conn = self.pool.get().await.map_err(|e| anyhow!("Redis connection error: {}", e))?;
        let res: redis::Value = cmd("GRAPH.QUERY")
            .arg(&self.graph_name)
            .arg(query)
            .arg("--compact")
            .query_async(&mut *conn)
            .await
            .map_err(|e| anyhow!("GRAPH.QUERY error: {}", e))?;
        Ok(res)
    }

    /// Expose the underlying pool for advanced use.
    pub fn inner(&self) -> &Arc<Pool<RedisConnectionManager>> {
        &self.pool
    }
}
