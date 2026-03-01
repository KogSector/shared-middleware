//! Error types for the connectivity infrastructure

/// Result type alias for connectivity operations
pub type Result<T> = std::result::Result<T, ConnectivityError>;

/// Comprehensive error type for connectivity operations
#[derive(Debug, thiserror::Error)]
pub enum ConnectivityError {
    /// Service discovery errors
    #[error("Service discovery error: {0}")]
    ServiceDiscovery(String),

    /// Service not found in registry
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    /// No healthy instances available
    #[error("No healthy instances available for service: {0}")]
    NoHealthyInstances(String),

    /// Circuit breaker is open
    #[error("Circuit breaker is open for service: {0}")]
    CircuitBreakerOpen(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// HTTP request error
    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest::Error),

    /// Cache error
    #[error("Cache error: {0}")]
    Cache(String),


    /// Event publishing error
    #[error("Event publishing failed: {0}")]
    EventPublish(String),

    /// Event consumption error
    #[error("Event consumption failed: {0}")]
    EventConsume(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Health check failed
    #[error("Health check failed: {0}")]
    HealthCheck(String),

    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Authentication error
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Authorization error
    #[error("Authorization failed: {0}")]
    Authorization(String),

    /// TLS/mTLS error
    #[error("TLS error: {0}")]
    Tls(String),

    /// Database connection error
    #[error("Database connection error: {0}")]
    DatabaseConnection(String),

    /// Saga execution error
    #[error("Saga execution failed: {0}")]
    SagaExecution(String),

    /// Invalid state transition
    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),

    /// Generic internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Wrapped anyhow error for compatibility
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl ConnectivityError {
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ConnectivityError::HttpRequest(_)
                | ConnectivityError::Timeout(_)
                | ConnectivityError::NoHealthyInstances(_)
                | ConnectivityError::DatabaseConnection(_)
                | ConnectivityError::Cache(_)
        )
    }

    /// Check if error should trigger circuit breaker
    pub fn should_trip_circuit_breaker(&self) -> bool {
        matches!(
            self,
            ConnectivityError::HttpRequest(_)
                | ConnectivityError::Timeout(_)
                | ConnectivityError::ServiceNotFound(_)
                | ConnectivityError::NoHealthyInstances(_)
        )
    }

    /// Get HTTP status code for error
    pub fn status_code(&self) -> u16 {
        match self {
            ConnectivityError::ServiceNotFound(_) => 404,
            ConnectivityError::NoHealthyInstances(_) => 503,
            ConnectivityError::CircuitBreakerOpen(_) => 503,
            ConnectivityError::RateLimitExceeded(_) => 429,
            ConnectivityError::Authentication(_) => 401,
            ConnectivityError::Authorization(_) => 403,
            ConnectivityError::Timeout(_) => 504,
            ConnectivityError::Configuration(_) => 500,
            _ => 500,
        }
    }
}
