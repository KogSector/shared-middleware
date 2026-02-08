//! Kafka Configuration with CONFLUENT_* Environment Variable Support

use rdkafka::ClientConfig;
use std::env;
use thiserror::Error;
use tracing::{info, warn};

/// Environment mode for the service
#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Production,
}

impl Environment {
    /// Parse from ENVIRONMENT env var
    pub fn from_env() -> Self {
        match env::var("ENVIRONMENT").unwrap_or_default().to_lowercase().as_str() {
            "production" | "prod" => Environment::Production,
            _ => Environment::Development,
        }
    }
}

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing required environment variable in production: {0}")]
    MissingEnvVar(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Kafka configuration builder
#[derive(Debug, Clone)]
pub struct KafkaConfig {
    pub bootstrap_servers: String,
    pub security_protocol: String,
    pub sasl_mechanism: Option<String>,
    pub sasl_username: Option<String>,
    pub sasl_password: Option<String>,
    pub client_id: String,
    pub group_id: Option<String>,
    pub environment: Environment,
}

impl KafkaConfig {
    /// Create a new KafkaConfig from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        let environment = Environment::from_env();
        
        // Get bootstrap servers (required)
        let bootstrap_servers = env::var("CONFLUENT_BOOTSTRAP_SERVERS")
            .or_else(|_| env::var("KAFKA_BOOTSTRAP_SERVERS"))
            .unwrap_or_else(|_| {
                if environment == Environment::Production {
                    String::new() // Will fail validation below
                } else {
                    "localhost:9092".to_string()
                }
            });
        
        // In production, require bootstrap servers
        if environment == Environment::Production && bootstrap_servers.is_empty() {
            return Err(ConfigError::MissingEnvVar("CONFLUENT_BOOTSTRAP_SERVERS".into()));
        }
        
        // Get SASL credentials (required in production)
        let sasl_username = env::var("CONFLUENT_API_KEY").ok();
        let sasl_password = env::var("CONFLUENT_API_SECRET").ok();
        
        // In production, require SASL credentials
        if environment == Environment::Production {
            if sasl_username.is_none() {
                return Err(ConfigError::MissingEnvVar("CONFLUENT_API_KEY".into()));
            }
            if sasl_password.is_none() {
                return Err(ConfigError::MissingEnvVar("CONFLUENT_API_SECRET".into()));
            }
        }
        
        // Determine security settings based on environment
        let (security_protocol, sasl_mechanism) = if environment == Environment::Production {
            ("SASL_SSL".to_string(), Some("PLAIN".to_string()))
        } else if sasl_username.is_some() {
            // Dev with Confluent Cloud
            ("SASL_SSL".to_string(), Some("PLAIN".to_string()))
        } else {
            // Local dev without auth
            ("PLAINTEXT".to_string(), None)
        };
        
        // Get client and group IDs
        let client_id = env::var("KAFKA_CLIENT_ID")
            .unwrap_or_else(|_| "confuse-service".to_string());
        
        let group_id = env::var("KAFKA_GROUP_ID").ok();
        
        let config = KafkaConfig {
            bootstrap_servers,
            security_protocol,
            sasl_mechanism,
            sasl_username,
            sasl_password,
            client_id,
            group_id,
            environment,
        };
        
        // Log configuration (without secrets)
        info!(
            "Kafka config: bootstrap_servers={}, security={}, client_id={}, env={:?}",
            config.bootstrap_servers,
            config.security_protocol,
            config.client_id,
            config.environment
        );
        
        Ok(config)
    }
    
    /// Build an rdkafka ClientConfig for producers
    pub fn to_producer_config(&self) -> ClientConfig {
        let mut config = ClientConfig::new();
        
        config.set("bootstrap.servers", &self.bootstrap_servers);
        config.set("client.id", &self.client_id);
        config.set("security.protocol", &self.security_protocol);
        
        if let Some(ref mechanism) = self.sasl_mechanism {
            config.set("sasl.mechanism", mechanism);
        }
        
        if let Some(ref username) = self.sasl_username {
            config.set("sasl.username", username);
        }
        
        if let Some(ref password) = self.sasl_password {
            config.set("sasl.password", password);
        }
        
        // Producer-specific settings
        config.set("acks", "all");
        config.set("retries", "5");
        config.set("retry.backoff.ms", "100");
        config.set("request.timeout.ms", "30000");
        
        // Enable idempotence for exactly-once semantics
        config.set("enable.idempotence", "true");
        
        config
    }
    
    /// Build an rdkafka ClientConfig for consumers
    pub fn to_consumer_config(&self) -> Result<ClientConfig, ConfigError> {
        let group_id = self.group_id.clone()
            .ok_or_else(|| ConfigError::InvalidConfig("KAFKA_GROUP_ID is required for consumers".into()))?;
        
        let mut config = ClientConfig::new();
        
        config.set("bootstrap.servers", &self.bootstrap_servers);
        config.set("client.id", &self.client_id);
        config.set("group.id", &group_id);
        config.set("security.protocol", &self.security_protocol);
        
        if let Some(ref mechanism) = self.sasl_mechanism {
            config.set("sasl.mechanism", mechanism);
        }
        
        if let Some(ref username) = self.sasl_username {
            config.set("sasl.username", username);
        }
        
        if let Some(ref password) = self.sasl_password {
            config.set("sasl.password", password);
        }
        
        // Consumer-specific settings
        config.set("enable.auto.commit", "false"); // Manual commit for reliability
        config.set("auto.offset.reset", "earliest");
        config.set("session.timeout.ms", "45000");
        
        Ok(config)
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.bootstrap_servers.is_empty() {
            return Err(ConfigError::InvalidConfig("bootstrap_servers cannot be empty".into()));
        }
        
        if self.environment == Environment::Production {
            if self.sasl_username.is_none() || self.sasl_password.is_none() {
                return Err(ConfigError::InvalidConfig(
                    "SASL credentials are required in production".into()
                ));
            }
        }
        
        Ok(())
    }
}
