//! Kafka Event Consumer

use crate::events::config::{KafkaConfig, ConfigError};
use rdkafka::consumer::{Consumer, StreamConsumer, CommitMode};
use rdkafka::message::{Message, BorrowedMessage};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::broadcast;
use tracing::{error, info, warn, debug};
use async_trait::async_trait;

/// Consumer errors
#[derive(Error, Debug)]
pub enum ConsumerError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    #[error("Deserialization error: {0}")]
    Deserialization(#[from] serde_json::Error),
    
    #[error("Kafka error: {0}")]
    Kafka(String),
    
    #[error("Consumer creation failed: {0}")]
    Creation(String),
    
    #[error("Handler error: {0}")]
    Handler(String),
}

/// Result type for event handlers
pub type HandlerResult = Result<(), ConsumerError>;

/// Trait for event handlers
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle a raw message from Kafka
    async fn handle(&self, topic: &str, payload: &[u8]) -> HandlerResult;
    
    /// Handle deserialization or processing errors
    async fn handle_error(&self, topic: &str, error: &ConsumerError, _payload: Option<&[u8]>) {
        error!(
            "Error processing message from topic '{}': {}",
            topic, error
        );
    }
}

/// Event consumer for subscribing to Kafka topics
pub struct EventConsumer {
    consumer: StreamConsumer,
    config: KafkaConfig,
    shutdown_tx: broadcast::Sender<()>,
}

impl EventConsumer {
    /// Create a new event consumer from environment configuration
    pub fn from_env() -> Result<Self, ConsumerError> {
        let config = KafkaConfig::from_env()?;
        Self::new(config)
    }
    
    /// Create a new event consumer with explicit configuration
    pub fn new(config: KafkaConfig) -> Result<Self, ConsumerError> {
        let consumer_config = config.to_consumer_config()?;
        
        let consumer: StreamConsumer = consumer_config
            .create()
            .map_err(|e| ConsumerError::Creation(e.to_string()))?;
        
        let (shutdown_tx, _) = broadcast::channel(1);
        
        info!(
            "Created Kafka consumer for {} (group: {:?})",
            config.bootstrap_servers,
            config.group_id
        );
        
        Ok(Self {
            consumer,
            config,
            shutdown_tx,
        })
    }
    
    /// Subscribe to one or more topics
    pub fn subscribe(&self, topics: &[&str]) -> Result<(), ConsumerError> {
        self.consumer
            .subscribe(topics)
            .map_err(|e| ConsumerError::Kafka(e.to_string()))?;
        
        info!("Subscribed to topics: {:?}", topics);
        Ok(())
    }
    
    /// Start consuming messages with the provided handler
    pub async fn run<H: EventHandler + 'static>(&self, handler: Arc<H>) -> Result<(), ConsumerError> {
        use futures::StreamExt;
        
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        let mut message_stream = self.consumer.stream();
        
        info!("Starting consumer loop");
        
        loop {
            tokio::select! {
                // Check for shutdown signal
                _ = shutdown_rx.recv() => {
                    info!("Received shutdown signal, stopping consumer");
                    break;
                }
                
                // Process messages
                message = message_stream.next() => {
                    match message {
                        Some(Ok(msg)) => {
                            if let Err(e) = self.process_message(&msg, handler.as_ref()).await {
                                handler.handle_error(
                                    msg.topic(),
                                    &e,
                                    msg.payload()
                                ).await;
                            }
                            
                            // Commit offset after processing
                            if let Err(e) = self.consumer.commit_message(&msg, CommitMode::Async) {
                                warn!("Failed to commit offset: {}", e);
                            }
                        }
                        Some(Err(e)) => {
                            error!("Kafka error: {}", e);
                        }
                        None => {
                            warn!("Message stream ended unexpectedly");
                            break;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Process a single message
    async fn process_message<H: EventHandler>(
        &self,
        msg: &BorrowedMessage<'_>,
        handler: &H,
    ) -> Result<(), ConsumerError> {
        let topic = msg.topic();
        let payload = msg.payload().unwrap_or_default();
        
        debug!(
            "Processing message from topic '{}' partition {} offset {}",
            topic,
            msg.partition(),
            msg.offset()
        );
        
        handler.handle(topic, payload).await
    }
    
    /// Signal the consumer to shut down
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }
    
    /// Get the underlying consumer configuration
    pub fn config(&self) -> &KafkaConfig {
        &self.config
    }
}

/// Helper function to deserialize a message payload
pub fn deserialize_event<E: DeserializeOwned>(payload: &[u8]) -> Result<E, ConsumerError> {
    serde_json::from_slice(payload).map_err(ConsumerError::from)
}
