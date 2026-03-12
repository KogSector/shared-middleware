//! Event Producer for ConFuse Platform

/// Kafka topic name constants — always available regardless of kafka feature.
pub struct Topics;

impl Topics {
    pub const CHUNKS_RAW: &'static str = "chunks.raw";
    pub const CODE_INGESTED: &'static str = "code.ingested";
    pub const CODE_PROCESSED: &'static str = "code.processed";
    pub const DOCS_INGESTED: &'static str = "docs.ingested";
    pub const DOCS_PROCESSED: &'static str = "docs.processed";
    pub const CHUNKS_CREATED: &'static str = "chunks.created";
    pub const CHUNKS_ENRICHED: &'static str = "chunks.enriched";
    pub const CHUNKS_EMBEDDED: &'static str = "chunks.embedded";
    pub const EMBEDDING_GENERATED: &'static str = "embedding.generated";
    pub const GRAPH_UPDATED: &'static str = "graph.updated";
    pub const GRAPH_BUILD_REQUESTED: &'static str = "graph.build.requested";
    pub const GRAPH_BUILD_COMPLETED: &'static str = "graph.build.completed";
    pub const SOURCE_SYNC_REQUESTED: &'static str = "source.sync.requested";
    pub const SOURCE_SYNC_COMPLETED: &'static str = "source.sync.completed";
    pub const SOURCE_SYNC_FAILED: &'static str = "source.sync.failed";
    pub const AUTH_EVENTS: &'static str = "auth.events";
    pub const SESSION_EVENTS: &'static str = "session.events";
    pub const DLQ_PROCESSING_FAILED: &'static str = "dlq.processing.failed";
}

// EventProducer requires the kafka feature (rdkafka → librdkafka → OpenSSL).
// Gate the entire implementation so services that don't need Kafka can compile
// without a system OpenSSL / CMake installation.
#[cfg(feature = "kafka")]
mod kafka_impl {
    use rdkafka::producer::{FutureProducer, FutureRecord};
    use rdkafka::ClientConfig;
    use serde::Serialize;
    use anyhow::Result;

    /// Kafka event producer
    pub struct EventProducer {
        producer: FutureProducer,
    }

    impl EventProducer {
        pub fn new(bootstrap_servers: &str) -> Result<Self> {
            let producer: FutureProducer = ClientConfig::new()
                .set("bootstrap.servers", bootstrap_servers)
                .set("message.max.bytes", "1000000")
                .set("delivery.timeout.ms", "5000")
                .create()?;

            Ok(Self { producer })
        }

        pub async fn publish<T: Serialize>(&self, topic: &str, event: &T) -> Result<()> {
            let payload = serde_json::to_string(event)?;
            let record = FutureRecord::to(topic)
                .payload(&payload)
                .key("event");

            match self.producer.send(record, std::time::Duration::from_secs(0)).await {
                Ok((partition, offset)) => {
                    tracing::info!("Event sent to partition {} at offset {}", partition, offset);
                    Ok(())
                }
                Err((e, _)) => {
                    tracing::error!("Failed to send event: {}", e);
                    Err(anyhow::anyhow!("Failed to send event: {}", e))
                }
            }
        }
    }
}

#[cfg(feature = "kafka")]
pub use kafka_impl::EventProducer;
