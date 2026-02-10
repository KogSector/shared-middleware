//! Connectivity module (HTTP, Database, Caching, Tracing)

pub mod client;
pub mod resilience;
pub mod tracing;
pub mod config;
pub mod error;
pub mod cache;
pub mod database;

// Re-exports
pub use client::{ServiceClient, ServiceClientConfig};
pub use resilience::{CircuitBreaker, CircuitBreakerConfig, RetryPolicy};
pub use config::ConnectivityConfig;
pub use error::{ConnectivityError, Result};
pub use cache::MemoryCache;
pub use database::{ConnectionPoolManager, PoolConfig};
