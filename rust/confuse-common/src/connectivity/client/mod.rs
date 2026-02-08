//! HTTP/gRPC client libraries with built-in resilience

pub mod http;

pub use http::{ServiceClient, ServiceClientConfig};
