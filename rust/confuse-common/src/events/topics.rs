//! Kafka Topic Definitions for ConFuse Platform

/// Kafka topic names used across the ConFuse platform
pub struct Topics;

impl Topics {
    // Code Processing Topics
    pub const CODE_INGESTED: &'static str = "code.ingested";
    pub const CODE_PROCESSED: &'static str = "code.processed";
    
    // Document Processing Topics
    pub const DOCS_INGESTED: &'static str = "docs.ingested";
    pub const DOCS_PROCESSED: &'static str = "docs.processed";
    
    // Chunk Processing Topics (Simplified Flow)
    pub const CHUNKS_CREATED: &'static str = "chunks.created";
    pub const CHUNKS_RAW: &'static str = "chunks.raw";
    pub const CHUNKS_ENRICHED: &'static str = "chunks.enriched";
    
    // Embedding Topics
    pub const EMBEDDING_GENERATED: &'static str = "embedding.generated";
    
    // Graph Topics
    pub const GRAPH_UPDATED: &'static str = "graph.updated";
    pub const GRAPH_BUILD_REQUESTED: &'static str = "graph.build.requested";
    pub const GRAPH_BUILD_COMPLETED: &'static str = "graph.build.completed";
    
    // Source Sync Topics
    pub const SOURCE_SYNC_REQUESTED: &'static str = "source.sync.requested";
    pub const SOURCE_SYNC_COMPLETED: &'static str = "source.sync.completed";
    pub const SOURCE_SYNC_FAILED: &'static str = "source.sync.failed";
    
    // Auth Topics
    pub const AUTH_EVENTS: &'static str = "auth.events";
    pub const SESSION_EVENTS: &'static str = "session.events";
    
    // DLQ Topics
    pub const DLQ_PROCESSING_FAILED: &'static str = "dlq.processing.failed";
}

/// Get all topic names for configuration
pub fn get_all_topics() -> Vec<&'static str> {
    vec![
        // Code Processing
        Topics::CODE_INGESTED,
        Topics::CODE_PROCESSED,
        
        // Document Processing
        Topics::DOCS_INGESTED,
        Topics::DOCS_PROCESSED,
        
        // Chunk Processing
        Topics::CHUNKS_CREATED,
        Topics::CHUNKS_RAW,
        Topics::CHUNKS_ENRICHED,
        
        // Embeddings
        Topics::EMBEDDING_GENERATED,
        
        // Graph
        Topics::GRAPH_UPDATED,
        Topics::GRAPH_BUILD_REQUESTED,
        Topics::GRAPH_BUILD_COMPLETED,
        
        // Source Sync
        Topics::SOURCE_SYNC_REQUESTED,
        Topics::SOURCE_SYNC_COMPLETED,
        Topics::SOURCE_SYNC_FAILED,
        
        // Auth
        Topics::AUTH_EVENTS,
        Topics::SESSION_EVENTS,
        
        // DLQ
        Topics::DLQ_PROCESSING_FAILED,
    ]
}

/// Get simplified flow topics (unified-processor → embeddings-service → relation-graph)
pub fn get_simplified_flow_topics() -> Vec<&'static str> {
    vec![
        Topics::CHUNKS_RAW,
        Topics::EMBEDDING_GENERATED,
        Topics::GRAPH_UPDATED,
        Topics::DLQ_PROCESSING_FAILED,
    ]
}
