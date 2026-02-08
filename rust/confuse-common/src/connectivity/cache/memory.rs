//! In-memory cache implementation

use std::sync::Arc;
use std::time::{Duration, Instant};
use dashmap::DashMap;
use std::sync::RwLock;
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

/// In-memory cache manager
pub struct MemoryCache {
    cache: Arc<DashMap<String, CacheEntry<Vec<u8>>>>,
    config: MemoryCacheConfig,
}

#[derive(Clone)]
pub struct MemoryCacheConfig {
    pub max_entries: usize,
    pub default_ttl: Duration,
    pub max_entry_size: usize,
}

impl Default for MemoryCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            default_ttl: Duration::from_secs(300),
            max_entry_size: 1024 * 1024,
        }
    }
}

struct CacheEntry<T> {
    data: T,
    created_at: Instant,
    ttl: Duration,
}

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Serialization error")]
    Serialization,
    #[error("Entry too large")]
    EntryTooLarge,
}

impl MemoryCache {
    pub fn new(config: MemoryCacheConfig) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            config,
        }
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let entry = self.cache.get(key)?;
        
        if entry.created_at.elapsed() > entry.ttl {
            drop(entry);
            self.cache.remove(key);
            return None;
        }

        serde_json::from_slice(&entry.data).ok()
    }

    pub fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Option<Duration>) -> Result<(), CacheError> {
        let data = serde_json::to_vec(value).map_err(|_| CacheError::Serialization)?;
        
        if data.len() > self.config.max_entry_size {
            return Err(CacheError::EntryTooLarge);
        }

        if self.cache.len() >= self.config.max_entries {
            // Simple random eviction for now if full
            if let Some(k) = self.cache.iter().next().map(|r| r.key().clone()) {
                self.cache.remove(&k);
            }
        }

        let entry = CacheEntry {
            data,
            created_at: Instant::now(),
            ttl: ttl.unwrap_or(self.config.default_ttl),
        };

        self.cache.insert(key.to_string(), entry);
        Ok(())
    }
    
    pub fn remove(&self, key: &str) {
        self.cache.remove(key);
    }
}
