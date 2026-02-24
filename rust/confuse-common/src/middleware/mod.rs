// Actix middleware (legacy, used by mcp-server)
pub mod auth;
pub mod authorization;
pub mod cors;
pub mod logging;
pub mod rate_limiting;

// Axum middleware (used by relation-graph, unified-processor, embeddings-service)
#[cfg(feature = "axum-support")]
pub mod axum_auth;
#[cfg(feature = "axum-support")]
pub mod axum_rate_limit;
#[cfg(feature = "axum-support")]
pub mod security_headers;
#[cfg(feature = "axum-support")]
pub mod zero_trust;

// Framework-agnostic middleware
pub mod cache;
pub mod circuit_breaker;

// Re-export Actix types
pub use auth::*;
pub use authorization::*;
pub use cors::*;
pub use logging::*;
pub use rate_limiting::*;

// Re-export Axum types
#[cfg(feature = "axum-support")]
pub use axum_auth::{AuthenticatedUser, AxumAuthLayer, axum_auth_middleware, axum_optional_auth_middleware};
#[cfg(feature = "axum-support")]
pub use axum_rate_limit::{AxumRateLimitConfig, axum_rate_limit_middleware};
#[cfg(feature = "axum-support")]
pub use security_headers::security_headers_middleware;
#[cfg(feature = "axum-support")]
pub use zero_trust::{ZeroTrustLayer, zero_trust_middleware};

// Re-export framework-agnostic types
pub use cache::{ResponseCache, CacheConfig};
pub use circuit_breaker::{CircuitBreakerRegistry, CircuitBreakerConfig, CircuitState};
