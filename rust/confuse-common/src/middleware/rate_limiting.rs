use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse, HttpMessage,
    body::EitherBody,
};
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    future::{ready, Ready},
    rc::Rc,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use log::{debug, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_window: u32,
    pub window_duration_secs: u64,
    pub burst_limit: Option<u32>,
    pub strategy: RateLimitStrategy,
    pub key_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitStrategy {
    FixedWindow,
    SlidingWindow,
    TokenBucket,
    Leaky,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_window: 100,
            window_duration_secs: 60,
            burst_limit: Some(10),
            strategy: RateLimitStrategy::SlidingWindow,
            key_prefix: "rate_limit".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct RateLimitMiddleware {
    config: RateLimitConfig,
    local_cache: Arc<Mutex<HashMap<String, RateLimitEntry>>>,
}

#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: u32,
    window_start: u64,
    tokens: f64,
    last_refill: u64,
}

impl RateLimitMiddleware {
    pub fn new(config: RateLimitConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config,
            local_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    fn get_client_key(&self, req: &ServiceRequest) -> String {
        // Try to get user ID from JWT claims first (requires prior auth middleware)
        if let Some(user_id) = req.extensions().get::<String>() {
            return format!("{}:user:{}", self.config.key_prefix, user_id);
        }

        // Fall back to IP address
        let ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();
        format!("{}:ip:{}", self.config.key_prefix, ip)
    }

    fn check_rate_limit_local(&self, key: &str) -> bool {
        let mut cache = self.local_cache.lock().unwrap();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let entry = cache.entry(key.to_string()).or_insert(RateLimitEntry {
            count: 0,
            window_start: now,
            tokens: self.config.requests_per_window as f64,
            last_refill: now,
        });

        match self.config.strategy {
            RateLimitStrategy::FixedWindow => {
                let window_start = (now / self.config.window_duration_secs) * self.config.window_duration_secs;
                if entry.window_start != window_start {
                    entry.count = 0;
                    entry.window_start = window_start;
                }
                
                if entry.count >= self.config.requests_per_window {
                    return false;
                }
                
                entry.count += 1;
                true
            }
            RateLimitStrategy::SlidingWindow => {
                if now - entry.window_start >= self.config.window_duration_secs {
                    entry.count = 0;
                    entry.window_start = now;
                }
                
                if entry.count >= self.config.requests_per_window {
                    return false;
                }
                
                entry.count += 1;
                true
            }
            RateLimitStrategy::TokenBucket => {
                let time_passed = now - entry.last_refill;
                let tokens_to_add = (time_passed as f64 / self.config.window_duration_secs as f64) * self.config.requests_per_window as f64;
                entry.tokens = (entry.tokens + tokens_to_add).min(self.config.requests_per_window as f64);
                entry.last_refill = now;
                
                if entry.tokens < 1.0 {
                    return false;
                }
                
                entry.tokens -= 1.0;
                true
            }
            RateLimitStrategy::Leaky => {
                if now - entry.window_start >= self.config.window_duration_secs {
                    entry.count = 0;
                    entry.window_start = now;
                }
                
                if entry.count >= self.config.requests_per_window {
                    return false;
                }
                
                entry.count += 1;
                true
            }
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddlewareService {
            service: Rc::new(service),
            middleware: self.clone(),
        }))
    }
}

pub struct RateLimitMiddlewareService<S> {
    service: Rc<S>,
    middleware: RateLimitMiddleware,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let key = self.middleware.get_client_key(&req);
        let middleware = self.middleware.clone();
        let service = self.service.clone();
        
        Box::pin(async move {
            debug!("Checking rate limit for key: {}", key);
            
            let allowed = middleware.check_rate_limit_local(&key);

            if !allowed {
                warn!("Rate limit exceeded for key: {}", key);
                let response = HttpResponse::TooManyRequests()
                    .insert_header(("X-RateLimit-Limit", middleware.config.requests_per_window.to_string()))
                    .insert_header(("X-RateLimit-Window", middleware.config.window_duration_secs.to_string()))
                    .insert_header(("Retry-After", middleware.config.window_duration_secs.to_string()))
                    .json(serde_json::json!({
                        "error": "Rate limit exceeded",
                        "message": format!("Too many requests. Limit: {} requests per {} seconds", 
                                         middleware.config.requests_per_window, 
                                         middleware.config.window_duration_secs)
                    }));
                
                return Ok(req.into_response(response).map_into_right_body());
            }

            debug!("Rate limit check passed for key: {}", key);
            service.call(req).await.map(|res| res.map_into_left_body())
        })
    }
}

pub fn create_rate_limit_middleware() -> RateLimitMiddleware {
    RateLimitMiddleware::new(RateLimitConfig::default())
        .expect("Failed to create rate limit middleware")
}
