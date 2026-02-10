//! Configuration management for connectivity infrastructure

use serde::{Deserialize, Serialize};

/// Main connectivity configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityConfig {
    /// Service discovery configuration
    pub service_discovery: ServiceDiscoveryConfig,

    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,

    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,

    /// Cache configuration
    pub cache: CacheConfig,

    /// Event bus configuration
    pub events: EventsConfig,

    /// Observability configuration
    pub observability: ObservabilityConfig,

    /// Security configuration
    pub security: SecurityConfig,
}

/// Service discovery configuration (Consul)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryConfig {
    /// Consul address
    pub consul_address: String,

    /// Service name
    pub service_name: String,

    /// Service ID (unique instance identifier)
    pub service_id: String,

    /// Service address
    pub service_address: String,

    /// Service port
    pub service_port: u16,

    /// Health check URL
    pub health_check_url: String,

    /// Health check interval in seconds
    #[serde(default = "default_health_check_interval")]
    pub health_check_interval_secs: u64,

    /// Deregister after critical for seconds
    #[serde(default = "default_deregister_after")]
    pub deregister_critical_after_secs: u64,

    /// Service tags
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold percentage (0-100)
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: u32,

    /// Success threshold for half-open state
    #[serde(default = "default_success_threshold")]
    pub success_threshold: u32,

    /// Timeout before attempting recovery (seconds)
    #[serde(default = "default_circuit_timeout")]
    pub timeout_secs: u64,

    /// Maximum calls in half-open state
    #[serde(default = "default_half_open_max_calls")]
    pub half_open_max_calls: u32,

    /// Minimum number of calls before calculating failure rate
    #[serde(default = "default_min_calls")]
    pub min_calls: u32,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {

    /// Per-user rate limit (requests per minute)
    #[serde(default = "default_per_user_limit")]
    pub per_user_limit: u32,

    /// Per-IP rate limit (requests per minute)
    #[serde(default = "default_per_ip_limit")]
    pub per_ip_limit: u32,

    /// Per-service rate limit (requests per minute)
    #[serde(default = "default_per_service_limit")]
    pub per_service_limit: u32,

    /// Rate limit window in seconds
    #[serde(default = "default_rate_limit_window")]
    pub window_secs: u64,

    /// Algorithm: "sliding_window" or "token_bucket"
    #[serde(default = "default_rate_limit_algorithm")]
    pub algorithm: String,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {

    /// Default TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub default_ttl_secs: u64,

    /// Maximum connections in pool
    #[serde(default = "default_cache_pool_size")]
    pub pool_size: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_cache_timeout")]
    pub connection_timeout_secs: u64,

    /// Enable cache invalidation pub/sub
    #[serde(default = "default_true")]
    pub enable_invalidation: bool,
}

/// Events configuration (Kafka)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventsConfig {
    /// Kafka brokers
    pub brokers: Vec<String>,

    /// Consumer group ID
    pub group_id: String,

    /// Enable auto-commit
    #[serde(default = "default_false")]
    pub enable_auto_commit: bool,

    /// Session timeout in milliseconds
    #[serde(default = "default_session_timeout")]
    pub session_timeout_ms: u64,

    /// Maximum retry attempts
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Dead letter queue topic
    #[serde(default = "default_dlq_topic")]
    pub dlq_topic: String,

    /// Schema registry URL (optional)
    pub schema_registry_url: Option<String>,
}

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Jaeger agent endpoint
    pub jaeger_endpoint: String,

    /// Service name for tracing
    pub service_name: String,

    /// Trace sampling rate (0.0 to 1.0)
    #[serde(default = "default_sampling_rate")]
    pub sampling_rate: f64,

    /// Enable metrics export
    #[serde(default = "default_true")]
    pub enable_metrics: bool,

    /// Prometheus metrics port
    #[serde(default = "default_metrics_port")]
    pub metrics_port: u16,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable mTLS
    #[serde(default = "default_false")]
    pub enable_mtls: bool,

    /// CA certificate path
    pub ca_cert_path: Option<String>,

    /// Client certificate path
    pub client_cert_path: Option<String>,

    /// Client key path
    pub client_key_path: Option<String>,

    /// Certificate rotation interval in hours
    #[serde(default = "default_cert_rotation_hours")]
    pub cert_rotation_hours: u64,
}

// Default value functions
fn default_health_check_interval() -> u64 { 10 }
fn default_deregister_after() -> u64 { 30 }
fn default_failure_threshold() -> u32 { 50 }
fn default_success_threshold() -> u32 { 3 }
fn default_circuit_timeout() -> u64 { 30 }
fn default_half_open_max_calls() -> u32 { 5 }
fn default_min_calls() -> u32 { 10 }
fn default_per_user_limit() -> u32 { 100 }
fn default_per_ip_limit() -> u32 { 1000 }
fn default_per_service_limit() -> u32 { 10000 }
fn default_rate_limit_window() -> u64 { 60 }
fn default_rate_limit_algorithm() -> String { "sliding_window".to_string() }
fn default_cache_ttl() -> u64 { 300 }
fn default_cache_pool_size() -> u32 { 10 }
fn default_cache_timeout() -> u64 { 5 }
fn default_session_timeout() -> u64 { 30000 }
fn default_max_retries() -> u32 { 3 }
fn default_dlq_topic() -> String { "dead-letter-queue".to_string() }
fn default_sampling_rate() -> f64 { 0.1 }
fn default_metrics_port() -> u16 { 9090 }
fn default_cert_rotation_hours() -> u64 { 24 }
fn default_true() -> bool { true }
fn default_false() -> bool { false }

impl Default for ConnectivityConfig {
    fn default() -> Self {
        Self {
            service_discovery: ServiceDiscoveryConfig {
                consul_address: "http://localhost:8500".to_string(),
                service_name: "unknown".to_string(),
                service_id: uuid::Uuid::new_v4().to_string(),
                service_address: "localhost".to_string(),
                service_port: 8080,
                health_check_url: "http://localhost:8080/health".to_string(),
                health_check_interval_secs: default_health_check_interval(),
                deregister_critical_after_secs: default_deregister_after(),
                tags: vec![],
            },
            circuit_breaker: CircuitBreakerConfig {
                failure_threshold: default_failure_threshold(),
                success_threshold: default_success_threshold(),
                timeout_secs: default_circuit_timeout(),
                half_open_max_calls: default_half_open_max_calls(),
                min_calls: default_min_calls(),
            },
            rate_limit: RateLimitConfig {
                per_user_limit: default_per_user_limit(),
                per_ip_limit: default_per_ip_limit(),
                per_service_limit: default_per_service_limit(),
                window_secs: default_rate_limit_window(),
                algorithm: default_rate_limit_algorithm(),
            },
            cache: CacheConfig {
                default_ttl_secs: default_cache_ttl(),
                pool_size: default_cache_pool_size(),
                connection_timeout_secs: default_cache_timeout(),
                enable_invalidation: true,
            },
            events: EventsConfig {
                brokers: vec!["localhost:9092".to_string()],
                group_id: "default-group".to_string(),
                enable_auto_commit: false,
                session_timeout_ms: default_session_timeout(),
                max_retries: default_max_retries(),
                dlq_topic: default_dlq_topic(),
                schema_registry_url: None,
            },
            observability: ObservabilityConfig {
                jaeger_endpoint: "http://localhost:14268/api/traces".to_string(),
                service_name: "unknown".to_string(),
                sampling_rate: default_sampling_rate(),
                enable_metrics: true,
                metrics_port: default_metrics_port(),
            },
            security: SecurityConfig {
                enable_mtls: false,
                ca_cert_path: None,
                client_cert_path: None,
                client_key_path: None,
                cert_rotation_hours: default_cert_rotation_hours(),
            },
        }
    }
}

impl ConnectivityConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, crate::connectivity::error::ConnectivityError> {
        // Try to load from CONNECTIVITY_CONFIG env var (JSON)
        if let Ok(json) = std::env::var("CONNECTIVITY_CONFIG") {
            return serde_json::from_str(&json)
                .map_err(|e| crate::connectivity::error::ConnectivityError::Configuration(e.to_string()));
        }

        // Otherwise build from individual env vars
        Ok(Self::default())
    }

    /// Load configuration from file
    pub fn from_file(path: &str) -> Result<Self, crate::connectivity::error::ConnectivityError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::connectivity::error::ConnectivityError::Configuration(e.to_string()))?;
        
        serde_json::from_str(&content)
            .map_err(|e| crate::connectivity::error::ConnectivityError::Configuration(e.to_string()))
    }
}
