//! Circuit breaker implementation

use crate::connectivity::{ConnectivityError, Result};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold percentage (0-100)
    pub failure_threshold: u32,
    /// Success threshold for half-open state
    pub success_threshold: u32,
    /// Timeout before attempting recovery
    pub timeout: Duration,
    /// Maximum calls in half-open state
    pub half_open_max_calls: u32,
    /// Minimum calls before calculating failure rate
    pub min_calls: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 50,
            success_threshold: 3,
            timeout: Duration::from_secs(30),
            half_open_max_calls: 5,
            min_calls: 10,
        }
    }
}

/// Circuit breaker metrics
#[derive(Debug)]
struct Metrics {
    total_calls: AtomicU64,
    failed_calls: AtomicU64,
    successful_calls: AtomicU64,
    rejected_calls: AtomicU64,
}

impl Metrics {
    fn new() -> Self {
        Self {
            total_calls: AtomicU64::new(0),
            failed_calls: AtomicU64::new(0),
            successful_calls: AtomicU64::new(0),
            rejected_calls: AtomicU64::new(0),
        }
    }
    
    fn failure_rate(&self) -> f64 {
        let total = self.total_calls.load(Ordering::Relaxed);
        let failed = self.failed_calls.load(Ordering::Relaxed);
        
        if total == 0 {
            0.0
        } else {
            (failed as f64 / total as f64) * 100.0
        }
    }
    
    fn reset(&self) {
        self.total_calls.store(0, Ordering::Relaxed);
        self.failed_calls.store(0, Ordering::Relaxed);
        self.successful_calls.store(0, Ordering::Relaxed);
    }
}

/// Internal state tracking
struct StateData {
    state: CircuitState,
    opened_at: Option<Instant>,
    half_open_successes: u32,
    half_open_calls: u32,
}

/// Circuit breaker for preventing cascading failures
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<Mutex<StateData>>,
    metrics: Arc<Metrics>,
    name: String,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(name: impl Into<String>, config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(StateData {
                state: CircuitState::Closed,
                opened_at: None,
                half_open_successes: 0,
                half_open_calls: 0,
            })),
            metrics: Arc::new(Metrics::new()),
            name: name.into(),
        }
    }
    
    /// Get current circuit state
    pub fn state(&self) -> CircuitState {
        self.state.lock().state
    }
    
    /// Get failure rate percentage
    pub fn failure_rate(&self) -> f64 {
        self.metrics.failure_rate()
    }
    
    /// Get total calls
    pub fn total_calls(&self) -> u64 {
        self.metrics.total_calls.load(Ordering::Relaxed)
    }
    
    /// Get rejected calls
    pub fn rejected_calls(&self) -> u64 {
        self.metrics.rejected_calls.load(Ordering::Relaxed)
    }
    
    /// Execute a function with circuit breaker protection
    pub async fn call<F, Fut, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Check if we should allow the call
        if !self.should_allow_call() {
            self.metrics.rejected_calls.fetch_add(1, Ordering::Relaxed);
            return Err(ConnectivityError::CircuitBreakerOpen(self.name.clone()));
        }
        
        self.metrics.total_calls.fetch_add(1, Ordering::Relaxed);
        
        // Execute the call
        match f().await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(e) => {
                let err: ConnectivityError = e;
                if err.should_trip_circuit_breaker() {
                    self.on_failure();
                }
                Err(err)
            }
        }
    }
    
    fn should_allow_call(&self) -> bool {
        let mut state = self.state.lock();
        
        match state.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has elapsed
                if let Some(opened_at) = state.opened_at {
                    if opened_at.elapsed() >= self.config.timeout {
                        info!(
                            circuit_breaker = %self.name,
                            "Transitioning to half-open state"
                        );
                        state.state = CircuitState::HalfOpen;
                        state.half_open_successes = 0;
                        state.half_open_calls = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited calls in half-open state
                state.half_open_calls < self.config.half_open_max_calls
            }
        }
    }
    
    fn on_success(&self) {
        self.metrics.successful_calls.fetch_add(1, Ordering::Relaxed);
        
        let mut state = self.state.lock();
        
        if state.state == CircuitState::HalfOpen {
            state.half_open_successes += 1;
            state.half_open_calls += 1;
            
            if state.half_open_successes >= self.config.success_threshold {
                info!(
                    circuit_breaker = %self.name,
                    "Service recovered, closing circuit"
                );
                state.state = CircuitState::Closed;
                state.opened_at = None;
                self.metrics.reset();
            }
        }
    }
    
    fn on_failure(&self) {
        self.metrics.failed_calls.fetch_add(1, Ordering::Relaxed);
        
        let mut state = self.state.lock();
        
        match state.state {
            CircuitState::Closed => {
                let total = self.metrics.total_calls.load(Ordering::Relaxed);
                
                if total >= self.config.min_calls as u64 {
                    let failure_rate = self.metrics.failure_rate();
                    
                    if failure_rate >= self.config.failure_threshold as f64 {
                        warn!(
                            circuit_breaker = %self.name,
                            failure_rate = %failure_rate,
                            "Opening circuit due to high failure rate"
                        );
                        state.state = CircuitState::Open;
                        state.opened_at = Some(Instant::now());
                    }
                }
            }
            CircuitState::HalfOpen => {
                warn!(
                    circuit_breaker = %self.name,
                    "Failure in half-open state, reopening circuit"
                );
                state.state = CircuitState::Open;
                state.opened_at = Some(Instant::now());
                state.half_open_successes = 0;
                state.half_open_calls = 0;
            }
            CircuitState::Open => {
                // Already open, nothing to do
            }
        }
    }
    
    /// Manually reset the circuit breaker
    pub fn reset(&self) {
        let mut state = self.state.lock();
        state.state = CircuitState::Closed;
        state.opened_at = None;
        state.half_open_successes = 0;
        state.half_open_calls = 0;
        self.metrics.reset();
        
        info!(circuit_breaker = %self.name, "Circuit breaker manually reset");
    }
}
