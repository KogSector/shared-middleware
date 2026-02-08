//! Distributed tracing utilities using OpenTelemetry

use opentelemetry::{
    global,
    trace::TraceContextExt,
    Context, KeyValue,
};
use opentelemetry_sdk::{
    trace::{self, RandomIdGenerator, Sampler},
    Resource,
};
use std::collections::HashMap;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize OpenTelemetry tracing with Jaeger exporter
pub fn init_tracing(
    service_name: &str,
    jaeger_endpoint: &str,
    sampling_rate: f64,
) -> Result<(), crate::connectivity::error::ConnectivityError> {
    // Create Jaeger exporter
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_endpoint(jaeger_endpoint)
        .with_service_name(service_name)
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::TraceIdRatioBased(sampling_rate))
                .with_id_generator(RandomIdGenerator::default())
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", service_name.to_string()),
                    KeyValue::new("service.version", crate::connectivity::VERSION.to_string()),
                ])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .map_err(|e| crate::connectivity::error::ConnectivityError::Configuration(e.to_string()))?;

    // Set up tracing subscriber with OpenTelemetry layer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer())
        .try_init()
        .map_err(|e| crate::connectivity::error::ConnectivityError::Configuration(e.to_string()))?;

    Ok(())
}

/// Shutdown tracing and flush remaining spans
pub fn shutdown_tracing() {
    global::shutdown_tracer_provider();
}

/// Extract trace context from HTTP headers
pub fn extract_trace_context(headers: &HashMap<String, String>) -> Context {
    use opentelemetry::propagation::TextMapPropagator;
    use opentelemetry_sdk::propagation::TraceContextPropagator;

    let propagator = TraceContextPropagator::new();
    propagator.extract(headers)
}

/// Inject trace context into HTTP headers
pub fn inject_trace_context(context: &Context) -> HashMap<String, String> {
    use opentelemetry::propagation::TextMapPropagator;
    use opentelemetry_sdk::propagation::TraceContextPropagator;

    let mut headers = HashMap::new();
    let propagator = TraceContextPropagator::new();
    propagator.inject_context(context, &mut headers);
    headers
}

/// Format trace context as W3C traceparent header
pub fn format_traceparent(context: &Context) -> String {
    let span = context.span();
    let span_context = span.span_context();
    
    let flags = if span_context.is_sampled() { 1u8 } else { 0u8 };
    
    format!(
        "00-{}-{}-{:02x}",
        span_context.trace_id(),
        span_context.span_id(),
        flags
    )
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
