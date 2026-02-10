//! Distributed tracing utilities
//!
//! Simple tracing setup using tracing-subscriber.
//! OpenTelemetry integration can be added back when upgrading to
//! opentelemetry 0.24+ with OTLP exporter.

use std::collections::HashMap;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize tracing with env filter and json/fmt layers
pub fn init_tracing(
    service_name: &str,
    _jaeger_endpoint: &str,
    _sampling_rate: f64,
) -> Result<(), crate::connectivity::error::ConnectivityError> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true))
        .try_init()
        .map_err(|e| crate::connectivity::error::ConnectivityError::Configuration(
            format!("Failed to initialize tracing for {}: {}", service_name, e)
        ))?;

    tracing::info!(service = service_name, "Tracing initialized");
    Ok(())
}

/// Shutdown tracing (no-op without OTel)
pub fn shutdown_tracing() {
    tracing::info!("Tracing shutdown");
}

/// Extract trace context from HTTP headers (stub without OTel)
pub fn extract_trace_context(_headers: &HashMap<String, String>) -> TracingContext {
    TracingContext::default()
}

/// Inject trace context into HTTP headers (stub without OTel)
pub fn inject_trace_context(_context: &TracingContext) -> HashMap<String, String> {
    HashMap::new()
}

/// Simple tracing context placeholder (replaces OTel Context)
#[derive(Debug, Default, Clone)]
pub struct TracingContext {
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
}

/// Format trace context as W3C traceparent header
pub fn format_traceparent(context: &TracingContext) -> String {
    match (&context.trace_id, &context.span_id) {
        (Some(trace_id), Some(span_id)) => {
            format!("00-{}-{}-01", trace_id, span_id)
        }
        _ => String::new(),
    }
}

/// Parse W3C traceparent header
pub fn parse_traceparent(traceparent: &str) -> Option<(String, String, u8)> {
    let parts: Vec<&str> = traceparent.split('-').collect();
    if parts.len() != 4 || parts[0] != "00" {
        return None;
    }

    Some((
        parts[1].to_string(), // trace_id
        parts[2].to_string(), // span_id
        u8::from_str_radix(parts[3], 16).ok()?, // flags
    ))
}

/// Create a new span with common attributes
#[macro_export]
macro_rules! trace_span {
    ($name:expr, $($key:expr => $value:expr),*) => {
        {
            use tracing::Span;
            let span = tracing::info_span!($name, $($key = $value),*);
            span
        }
    };
}

/// Record an error in the current span
pub fn record_error(error: &dyn std::error::Error) {
    use tracing::Span;
    
    let current_span = Span::current();
    current_span.record("error", &true);
    current_span.record("error.message", &error.to_string());
}

/// Add event to current span
pub fn add_event(name: &str, attributes: Vec<(&str, String)>) {
    use tracing::Span;
    
    let current_span = Span::current();
    current_span.record("event", &name);
    for (key, value) in attributes {
        current_span.record(key, &value.as_str());
    }
}
