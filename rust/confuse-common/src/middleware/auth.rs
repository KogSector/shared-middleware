use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    body::EitherBody,
    Error as ActixError, HttpMessage, HttpRequest, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;
use crate::models::auth::Claims; // CORRECTION: Updated import
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde_json::json;
use chrono;
use tracing;
// use once_cell::sync::Lazy; // Unused
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use reqwest::Client;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Clone)]
struct Auth0Config {
    #[allow(dead_code)]
    domain: String,
    issuer: String,
    audience: String,
    jwks_uri: String,
    leeway_secs: u64,
}

impl Auth0Config {
    fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let domain = std::env::var("AUTH0_DOMAIN")?;
        let issuer = std::env::var("AUTH0_ISSUER")?;
        let audience = std::env::var("AUTH0_AUDIENCE")?;
        let jwks_uri = std::env::var("AUTH0_JWKS_URI")
            .unwrap_or_else(|_| format!("https://{}/.well-known/jwks.json", domain));
        Ok(Self {
            domain,
            issuer,
            audience,
            jwks_uri,
            leeway_secs: 60,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Jwk {
    kid: String,
    kty: String,
    n: String,
    e: String,
    alg: Option<String>,
    #[serde(rename = "use")]
    use_: Option<String>,

    #[serde(flatten)]
    extra: std::collections::HashMap<String, JsonValue>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

struct Auth0JwksCache {
    client: Client,
    config: Auth0Config,
    cached: Option<(Jwks, Instant)>,
    ttl: Duration,
}

impl Auth0JwksCache {
    fn new(config: Auth0Config) -> Self {
        Self {
            client: Client::new(),
            config,
            cached: None,
            ttl: Duration::from_secs(600),
        }
    }

    async fn get_jwks(&mut self) -> Result<Jwks, Box<dyn std::error::Error>> {
        if let Some((jwks, ts)) = &self.cached {
            if ts.elapsed() < self.ttl {
                return Ok(jwks.clone());
            }
        }
        let resp = self.client.get(&self.config.jwks_uri).send().await?;
        if !resp.status().is_success() {
            return Err(format!("Failed to fetch JWKS: {}", resp.status()).into());
        }
        let jwks: Jwks = resp.json().await?;
        self.cached = Some((jwks.clone(), Instant::now()));
        Ok(jwks)
    }

    async fn get_key(&mut self, kid: &str) -> Result<Jwk, Box<dyn std::error::Error>> {
        let jwks = self.get_jwks().await?;
        jwks
            .keys
            .into_iter()
            .find(|k| k.kid == kid)
            .ok_or_else(|| "JWK not found for kid".into())
    }
}

#[derive(Debug, Deserialize)]
struct RawAuth0Claims {
    sub: String,
    iss: String,
    aud: JsonValue,
    exp: usize,
    iat: Option<usize>,
    email: Option<String>,
    scope: Option<String>,
    permissions: Option<Vec<String>>, 
    #[allow(dead_code)]
    extra: std::collections::HashMap<String, JsonValue>,
}

#[derive(Clone)]
struct Auth0Verifier {
    config: Auth0Config,
    jwks_cache: Arc<tokio::sync::Mutex<Auth0JwksCache>>,
}

#[derive(Clone)]
enum AuthMode {
    Enabled(Arc<Auth0Verifier>),
    Disabled(Claims),
}

pub struct AuthMiddleware<S> {
    service: Rc<S>,
    mode: AuthMode,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let mode = self.mode.clone();

        Box::pin(async move {
            // Always allow OPTIONS requests through for CORS preflight
            if req.method() == actix_web::http::Method::OPTIONS {
                let res = service.call(req).await?;
                return Ok(res.map_into_left_body());
            }
            
            // Check if this is a public endpoint that doesn't require authentication
            if is_public_endpoint(req.path()) {
                let res = service.call(req).await?;
                return Ok(res.map_into_left_body());
            }

            match mode {
                AuthMode::Enabled(verifier) => {
                    if let Some(auth_header) = req.headers().get("Authorization") {
                        if let Ok(auth_str) = auth_header.to_str() {
                            if auth_str.starts_with("Bearer ") {
                                let token = &auth_str[7..];
                                
                                // Try ConHub token first (issued by auth service after Auth0 exchange)
                                if let Ok(claims) = verify_conhub_jwt_token(token).await {
                                    req.extensions_mut().insert(claims);
                                    let res = service.call(req).await?;
                                    return Ok(res.map_into_left_body());
                                }
                                
                                // Fall back to Auth0 token verification
                                match verify_auth0_jwt_token(token, &verifier).await {
                                    Ok(claims) => {
                                        req.extensions_mut().insert(claims);
                                        let res = service.call(req).await?;
                                        return Ok(res.map_into_left_body());
                                    }
                                    Err(e) => {
                                        tracing::warn!("JWT verification failed: {}", e);
                                        return Ok(req.into_response(
                                            HttpResponse::Unauthorized()
                                                .json(json!({
                                                    "error": "Invalid or expired token",
                                                    "details": e.to_string()
                                                }))
                                        ).map_into_right_body());
                                    }
                                }
                            }
                        }
                    }
                    Ok(req.into_response(
                        HttpResponse::Unauthorized()
                            .json(json!({
                                "error": "Authentication required",
                                "message": "Please provide a valid Bearer token in the Authorization header"
                            }))
                    ).map_into_right_body())
                }
                AuthMode::Disabled(default_claims) => {
                    // Inject default claims and proceed
                    req.extensions_mut().insert(default_claims.clone());
                    let res = service.call(req).await?;
                    Ok(res.map_into_left_body())
                }
            }
        })
    }
}

#[derive(Clone)]
pub struct AuthMiddlewareFactory {
    mode: AuthMode,
}

impl AuthMiddlewareFactory {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Auth0Config::from_env()?;
        let cache = Auth0JwksCache::new(config.clone());
        let verifier = Auth0Verifier {
            config,
            jwks_cache: Arc::new(tokio::sync::Mutex::new(cache)),
        };
        Ok(Self { mode: AuthMode::Enabled(Arc::new(verifier)) })
    }

    pub fn disabled() -> Self {
        let claims = crate::models::auth::default_dev_claims(); // CORRECTION
        Self { mode: AuthMode::Disabled(claims) }
    }

    pub fn new_with_enabled(enabled: bool) -> Result<Self, Box<dyn std::error::Error>> {
        if enabled {
            Self::new()
        } else {
            Ok(Self::disabled())
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = ActixError;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
            mode: self.mode.clone(),
        }))
    }
}

async fn verify_auth0_jwt_token(
    token: &str,
    verifier: &Auth0Verifier,
) -> Result<crate::models::auth::Claims, Box<dyn std::error::Error>> { // CORRECTION
    let header = decode_header(token)?;
    let kid = header.kid.ok_or("Missing kid in JWT header")?;

    let mut cache = verifier.jwks_cache.lock().await;
    let jwk = cache.get_key(&kid).await?;

    if jwk.kty != "RSA" {
        return Err("Unsupported JWK kty".into());
    }

    let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.iss = Some(std::collections::HashSet::from([verifier.config.issuer.clone()]));
    validation.validate_aud = false;

    let token_data = decode::<RawAuth0Claims>(token, &decoding_key, &validation)?;
    let claims = token_data.claims;

    let now = chrono::Utc::now().timestamp() as usize;
    if claims.exp + (verifier.config.leeway_secs as usize) < now {
        return Err("Token expired".into());
    }

    let aud_ok = match &claims.aud {
        JsonValue::String(aud) => aud == &verifier.config.audience,
        JsonValue::Array(arr) => arr.iter().any(|v| v == &JsonValue::String(verifier.config.audience.clone())),
        _ => false,
    };
    if !aud_ok {
        return Err("Invalid audience".into());
    }

    if let Ok(required_scope) = std::env::var("AUTH0_REQUIRED_SCOPE") {
        if let Some(scope_str) = &claims.scope {
            let scopes: std::collections::HashSet<_> =
                scope_str.split_whitespace().map(|s| s.to_string()).collect();
            if !scopes.contains(&required_scope) {
                return Err("Missing required scope".into());
            }
        } else {
            return Err("Missing scope claim".into());
        }
    }

    let now = chrono::Utc::now().timestamp() as usize;
    let iat = claims.iat.unwrap_or(now);
    let email = claims.email.unwrap_or_default();

    let mut roles: Vec<String> = Vec::new();
    if let Some(perms) = claims.permissions.clone() {
        for p in perms {
            if p.starts_with("admin") && !roles.contains(&"admin".to_string()) {
                roles.push("admin".to_string());
            }
        }
    }
    if roles.is_empty() {
        roles.push("user".to_string());
    }

    let session_id = Uuid::new_v4().to_string();
    let jti = Uuid::new_v4().to_string();

    let internal_claims = crate::models::auth::Claims { // CORRECTION
        sub: claims.sub.clone(),
        email,
        roles,
        exp: claims.exp,
        iat,
        iss: claims.iss.clone(),
        aud: verifier.config.audience.clone(),
        session_id,
        jti,
    };

    Ok(internal_claims)
}

async fn verify_conhub_jwt_token(token: &str) -> Result<crate::models::auth::Claims, Box<dyn std::error::Error>> { // CORRECTION
    let header = decode_header(token)?;
    
    let kid = header.kid.as_deref().unwrap_or("");
    if kid != "conhub-auth-key" {
        return Err("Not a ConHub token".into());
    }
    
    let public_key_raw = std::env::var("CONHUB_AUTH_PUBLIC_KEY")
        .or_else(|_| std::env::var("JWT_PUBLIC_KEY"))
        .map_err(|_| "ConHub auth public key not configured")?;
    
    let public_key_pem = public_key_raw
        .trim_matches('"')
        .replace("\\n", "\n");
    
    let decoding_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())
        .map_err(|e| format!("Failed to parse ConHub public key: {}", e))?;
    
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&["conhub-auth"]);
    validation.set_audience(&["conhub-services"]);
    
    let token_data = decode::<crate::models::auth::Claims>(token, &decoding_key, &validation) // CORRECTION
        .map_err(|e| format!("ConHub token validation failed: {}", e))?;
    
    let now = chrono::Utc::now().timestamp() as usize;
    if token_data.claims.exp < now {
        return Err("ConHub token expired".into());
    }
    
    tracing::debug!("Successfully verified ConHub token for sub: {}", token_data.claims.sub);
    Ok(token_data.claims)
}

fn is_public_endpoint(path: &str) -> bool {
    if path.starts_with("/api/auth/oauth/exchange") {
        return false;
    }

    let public_paths = [
        "/health",
        "/metrics",
        "/auth/login",
        "/auth/register",
        "/auth/forgot-password",
        "/auth/reset-password",
        "/auth/verify-email",
        "/auth/oauth",
        "/docs",
        "/swagger",
        "/api/dashboard/stats",
        "/api/auth/auth0",
        "/api/auth/oauth",
        "/api/security/connections",
    ];
    
    public_paths.iter().any(|&public_path| path.starts_with(public_path))
}

pub fn extract_token_from_request(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(|s| s.to_string())
}

pub fn extract_claims_from_request(req: &HttpRequest) -> Option<Claims> {
    req.extensions().get::<Claims>().cloned()
}

pub fn extract_session_id_from_request(req: &HttpRequest) -> Option<uuid::Uuid> {
    extract_claims_from_request(req)?
        .session_id
        .parse()
        .ok()
}

pub fn extract_user_id_from_request(req: &HttpRequest) -> Option<uuid::Uuid> {
    extract_claims_from_request(req)?
        .sub
        .parse()
        .ok()
}

pub fn extract_user_id_from_http_request(req: &HttpRequest) -> Option<uuid::Uuid> {
    extract_user_id_from_request(req)
}

pub fn extract_claims_from_http_request(req: &HttpRequest) -> Option<Claims> {
    extract_claims_from_request(req)
}

pub struct RoleAuthMiddleware<S> {
    service: Rc<S>,
    required_roles: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for RoleAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let required_roles = self.required_roles.clone();

        Box::pin(async move {
            let has_permission = if let Some(claims) = req.extensions().get::<Claims>() {
                required_roles.is_empty() || 
                required_roles.iter().any(|role| claims.roles.contains(role))
            } else {
                false
            };

            if has_permission {
                let res = service.call(req).await?;
                Ok(res.map_into_left_body())
            } else {
                Ok(req.into_response(
                    HttpResponse::Forbidden()
                        .json(json!({
                            "error": "Insufficient permissions",
                            "required_roles": required_roles
                        }))
                ).map_into_right_body())
            }
        })
    }
}

pub struct RoleAuthMiddlewareFactory {
    required_roles: Vec<String>,
}

impl RoleAuthMiddlewareFactory {
    pub fn new(required_roles: Vec<String>) -> Self {
        Self { required_roles }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RoleAuthMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = ActixError;
    type Transform = RoleAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RoleAuthMiddleware {
            service: Rc::new(service),
            required_roles: self.required_roles.clone(),
        }))
    }
}
