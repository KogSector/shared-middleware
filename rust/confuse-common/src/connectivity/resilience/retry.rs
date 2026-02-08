//! Retry policies and exponential backoff

use crate::connectivity::Result;
use std::time::Duration;
use tracing::debug;

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial backoff duration
    pub initial_backoff: Duration,
    /// Maximum backoff duration
    pub max_backoff: Duration,
    /// Backoff multiplier
    pub multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(10),
            multiplier: 2.0,
        }
    }
}

impl RetryPolicy {
    /// Calculate backoff duration for a given attempt
    pub fn backoff_duration(&self, attempt: u32) -> Duration {
        let backoff_ms = (self.initial_backoff.as_millis() as f64
            * self.multiplier.powi(attempt as i32)) as u64;
        
        Duration::from_millis(backoff_ms.min(self.max_backoff.as_millis() as u64))
    }
}

/// Exponential backoff retry helper
pub struct ExponentialBackoff {
    policy: RetryPolicy,
}

impl ExponentialBackoff {
    pub fn new(policy: RetryPolicy) -> Self {
        Self { policy }
    }
    
    /// Execute a function with exponential backoff retry
    pub async fn retry<F, Fut, T>(&self, mut f: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempt = 0;
        
        loop {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) if attempt < self.policy.max_attempts && e.is_retryable() => {
                    let backoff = self.policy.backoff_duration(attempt);
                    debug!(
                        attempt = attempt + 1,
                        max_attempts = self.policy.max_attempts,
                        backoff_ms = backoff.as_millis(),
                        "Retrying after failure"
                    );
                    tokio::time::sleep(backoff).await;
                    attempt += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }
}
