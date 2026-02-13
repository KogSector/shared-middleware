//! Topic Constants for ConFuse Platform

/// All Kafka topics used by ConFuse platform
pub struct Topics;

impl Topics {
    // Code Processing
    pub const CODE_INGESTED: &'static str = "code.ingested";
    pub const CODE_PROCESSED: &'static str = "code.processed";
    
    // Document Processing
    pub const DOCS_INGESTED: &'static str = "docs.ingested";
    pub const DOCS_PROCESSED: &'static str = "docs.processed";
    pub const CHUNKS_CREATED: &'static str = "chunks.created";
    
    // Embeddings
    pub const EMBEDDING_GENERATED: &'static str = "embedding.generated";
    
    // Knowledge Graph
    pub const GRAPH_UPDATED: &'static str = "graph.updated";
    pub const GRAPH_BUILD_REQUESTED: &'static str = "graph.build.requested";
    pub const GRAPH_BUILD_COMPLETED: &'static str = "graph.build.completed";
    
    // Source Sync
    pub const SOURCE_SYNC_REQUESTED: &'static str = "source.sync.requested";
    pub const SOURCE_SYNC_COMPLETED: &'static str = "source.sync.completed";
    pub const SOURCE_SYNC_FAILED: &'static str = "source.sync.failed";
    
    // Auth
    pub const AUTH_EVENTS: &'static str = "auth.events";
    pub const SESSION_EVENTS: &'static str = "session.events";

    // Dead Letter Queue
    pub const DLQ_PROCESSING_FAILED: &'static str = "dlq.processing.failed";
    
    /// Get all topics
    pub fn all() -> Vec<&'static str> {
        vec![
            Self::CODE_INGESTED,
            Self::CODE_PROCESSED,
            Self::DOCS_INGESTED,
            Self::DOCS_PROCESSED,
            Self::EMBEDDING_GENERATED,
            Self::GRAPH_UPDATED,
            Self::GRAPH_BUILD_REQUESTED,
            Self::GRAPH_BUILD_COMPLETED,
            Self::SOURCE_SYNC_REQUESTED,
            Self::SOURCE_SYNC_COMPLETED,
            Self::SOURCE_SYNC_FAILED,
            Self::AUTH_EVENTS,
            Self::SESSION_EVENTS,
            Self::DLQ_PROCESSING_FAILED,
        ]
    }
}
