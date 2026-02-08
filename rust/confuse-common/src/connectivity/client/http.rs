//! HTTP service client with circuit breaker and retry

use crate::connectivity::resilience::{CircuitBreaker, CircuitBreakerConfig, ExponentialBackoff, RetryPolicy};
use crate::connectivity::{ConnectivityError, Result};
use reqwest::{Client, Method};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, instrument};

/// Service client configuration
#[derive(Debug, Clone)]
pub struct ServiceClientConfig {
    /// Base URL for the service
    pub base_url: String,
    /// Request timeout
    pub timeout: Duration,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
    /// Retry policy
    pub retry_policy: RetryPolicy,
}

impl Default for ServiceClientConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
            timeout: Duration::from_secs(30),
            circuit_breaker: CircuitBreakerConfig::default(),
            retry_policy: RetryPolicy::default(),
        }
    }
}

/// HTTP service client with resilience
pub struct ServiceClient {
    client: Client,
    config: ServiceClientConfig,
    circuit_breaker: Arc<CircuitBreaker>,
    retry: ExponentialBackoff,
}

impl ServiceClient {
    /// Create a new service client
    pub fn new(config: ServiceClientConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| ConnectivityError::Configuration(e.to_string()))?;
        
        let circuit_breaker = Arc::new(CircuitBreaker::new(
            &config.base_url,
            config.circuit_breaker.clone(),
        ));
        
        let retry = ExponentialBackoff::new(config.retry_policy.clone());
        
        Ok(Self {
            client,
            config,
            circuit_breaker,
            retry,
        })
    }
    
    /// Make a GET request
    #[instrument(skip(self), fields(url = %url))]
    pub async fn get<T>(&self, url: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.request(Method::GET, url, None::<()>).await
    }
    
    /// Make a POST request
    #[instrument(skip(self, body), fields(url = %url))]
    pub async fn post<B, T>(&self, url: &str, body: B) -> Result<T>
    where
        B: Serialize,
        T: DeserializeOwned,
    {
        self.request(Method::POST, url, Some(body)).await
    }
    
    /// Make a PUT request
    #[instrument(skip(self, body), fields(url = %url))]
    pub async fn put<B, T>(&self, url: &str, body: B) -> Result<T>
    where
        B: Serialize,
        T: DeserializeOwned,
    {
        self.request(Method::PUT, url, Some(body)).await
    }
    
    /// Make a DELETE request
    #[instrument(skip(self), fields(url = %url))]
    pub async fn delete<T>(&self, url: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.request(Method::DELETE, url, None::<()>).await
    }
    
    async fn request<B, T>(&self, method: Method, path: &str, body: Option<B>) -> Result<T>
    where
        B: Serialize,
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.config.base_url, path);
        
        self.circuit_breaker
            .call(|| async {
                self.retry
                    .retry(|| async {
                        self.execute_request(method.clone(), &url, &body).await
                    })
                    .await
            })
            .await
    }
    
    async fn execute_request<B, T>(&self, method: Method, url: &str, body: &Option<B>) -> Result<T>
    where
        B: Serialize,
        T: DeserializeOwned,
    {
        debug!(method = %method, url = %url, "Executing HTTP request");
        
        let mut request = self.client.request(method, url);
        
        if let Some(body) = body {
            request = request.json(body);
        }
        
        let response = request
            .send()
            .await
            .map_err(|e| ConnectivityError::HttpRequest(e))?;
        
        if !response.status().is_success() {
            return Err(ConnectivityError::HttpRequest(
                reqwest::Error::from(response.error_for_status().unwrap_err())
            ));
        }
        
        response
            .json::<T>()
            .await
            .map_err(|e| ConnectivityError::HttpRequest(e))
    }
    
    /// Get circuit breaker state
    pub fn circuit_state(&self) -> crate::connectivity::resilience::CircuitState {
        self.circuit_breaker.state()
    }
}
