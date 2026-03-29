//! Event Producer for ConFuse Platform


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
