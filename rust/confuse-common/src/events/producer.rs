//! Kafka Event Producer

use crate::events::config::{KafkaConfig, ConfigError};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use serde::Serialize;
use std::time::Duration;
use thiserror::Error;
use tracing::{info, debug};

/// Producer errors
#[derive(Error, Debug)]
pub enum ProducerError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Kafka error: {0}")]
    Kafka(String),
    
    #[error("Producer creation failed: {0}")]
    Creation(String),
}

/// Event producer for publishing events to Kafka
pub struct EventProducer {
    producer: FutureProducer,
    config: KafkaConfig,
}

impl EventProducer {
    /// Create a new event producer from environment configuration
    pub fn from_env() -> Result<Self, ProducerError> {
        let config = KafkaConfig::from_env()?;
        Self::new(config)
    }
    
    /// Create a new event producer with explicit configuration
    pub fn new(config: KafkaConfig) -> Result<Self, ProducerError> {
        let producer_config = config.to_producer_config();
        
        let producer: FutureProducer = producer_config
            .create()
            .map_err(|e| ProducerError::Creation(e.to_string()))?;
        
        info!(
            "Created Kafka producer for {} ({})",
            config.bootstrap_servers,
            config.client_id
        );
        
        Ok(Self { producer, config })
    }
    
    /// Publish an event to a specific topic
    pub async fn publish_to_topic<E>(&self, event: &E, topic: &str) -> Result<(i32, i64), ProducerError>
    where
        E: Serialize,
    {
        let payload = serde_json::to_string(event)?;
        
        debug!("Publishing event to topic '{}': {} bytes", topic, payload.len());
        
        let record = FutureRecord::to(topic)
            .payload(&payload)
            .key(""); // Use empty key for now
        
        let delivery_result = self.producer
            .send(record, Timeout::After(Duration::from_secs(30)))
            .await
            .map_err(|(e, _)| ProducerError::Kafka(e.to_string()))?;
        
        debug!(
            "Event published to topic '{}' partition {} offset {}",
            topic, delivery_result.0, delivery_result.1
        );
        
        Ok(delivery_result)
    }
    
    pub fn config(&self) -> &KafkaConfig {
        &self.config
    }
}
