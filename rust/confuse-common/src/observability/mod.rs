//! Observability module (Logging, Tracing, Metrics)

pub mod logging;

// Re-export connectivity tracing
pub use crate::connectivity::tracing::*;
pub use logging::{LoggingConfig, init_logging};
