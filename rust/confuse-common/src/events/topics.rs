//! Topic Constants for ConFuse Platform
//!
//! Unified Content Architecture: Kafka-streamed processing pipeline.
//! All inter-service communication flows through these topics.

/// All Kafka topics used by ConFuse platform
pub struct Topics;

impl Topics {
    // =========================================================================
    // Content Ingestion (data-connector → unified-processor)
    // =========================================================================

    /// Full repository clone & processing requests
    /// Producer: data-connector | Consumer: unified-processor
    pub const REPO_PROCESSING_REQUESTS: &'static str = "repo-processing.requests";

    /// Document download & processing requests
    /// Producer: data-connector | Consumer: unified-processor
    pub const DOC_PROCESSING_REQUESTS: &'static str = "doc-processing.requests";

    /// Incremental repository updates (webhooks/polling)
    /// Producer: data-connector | Consumer: unified-processor
    pub const REPO_UPDATES_REQUESTS: &'static str = "repo-updates.requests";

    // =========================================================================
    // Processing Pipeline (unified-processor → embeddings → relation-graph)
    // =========================================================================

    /// Raw chunks without embeddings
    /// Producer: unified-processor | Consumer: embeddings-service
    pub const CHUNKS_RAW: &'static str = "chunks.raw";

    /// Chunks with embeddings ready for storage
    /// Producer: embeddings-service | Consumer: relation-graph
    pub const CHUNKS_EMBEDDED: &'static str = "chunks.embedded";

    /// Notification that chunks are stored in FalkorDB
    /// Producer: relation-graph | Consumer: data-vent
    pub const CHUNKS_STORED: &'static str = "chunks.stored";

    // =========================================================================
    // System Topics
    // =========================================================================

    /// Dead letter queue for failed processing messages
    pub const DLQ_PROCESSING_FAILED: &'static str = "dlq.processing.failed";

    /// Auth events (login, logout, token refresh)
    pub const AUTH_EVENTS: &'static str = "auth.events";

    /// Session lifecycle events
    pub const SESSION_EVENTS: &'static str = "session.events";

    // =========================================================================
    // Legacy Topics (kept for backward compatibility during migration)
    // =========================================================================

    pub const CODE_INGESTED: &'static str = "code.ingested";
    pub const CODE_PROCESSED: &'static str = "code.processed";
    pub const DOCS_INGESTED: &'static str = "docs.ingested";
    pub const DOCS_PROCESSED: &'static str = "docs.processed";
    pub const CHUNKS_CREATED: &'static str = "chunks.created";
    pub const EMBEDDING_GENERATED: &'static str = "embedding.generated";
    pub const GRAPH_UPDATED: &'static str = "graph.updated";
    pub const GRAPH_BUILD_REQUESTED: &'static str = "graph.build.requested";
    pub const GRAPH_BUILD_COMPLETED: &'static str = "graph.build.completed";
    pub const SOURCE_SYNC_REQUESTED: &'static str = "source.sync.requested";
    pub const SOURCE_SYNC_COMPLETED: &'static str = "source.sync.completed";
    pub const SOURCE_SYNC_FAILED: &'static str = "source.sync.failed";

    /// Get all active pipeline topics
    pub fn pipeline_topics() -> Vec<&'static str> {
        vec![
            Self::REPO_PROCESSING_REQUESTS,
            Self::DOC_PROCESSING_REQUESTS,
            Self::REPO_UPDATES_REQUESTS,
            Self::CHUNKS_RAW,
            Self::CHUNKS_EMBEDDED,
            Self::CHUNKS_STORED,
            Self::DLQ_PROCESSING_FAILED,
        ]
    }

    /// Get all topics (pipeline + system + legacy)
    pub fn all() -> Vec<&'static str> {
        vec![
            // Pipeline
            Self::REPO_PROCESSING_REQUESTS,
            Self::DOC_PROCESSING_REQUESTS,
            Self::REPO_UPDATES_REQUESTS,
            Self::CHUNKS_RAW,
            Self::CHUNKS_EMBEDDED,
            Self::CHUNKS_STORED,
            // System
            Self::DLQ_PROCESSING_FAILED,
            Self::AUTH_EVENTS,
            Self::SESSION_EVENTS,
            // Legacy
            Self::CODE_INGESTED,
            Self::CODE_PROCESSED,
            Self::DOCS_INGESTED,
            Self::DOCS_PROCESSED,
            Self::CHUNKS_CREATED,
            Self::EMBEDDING_GENERATED,
            Self::GRAPH_UPDATED,
            Self::GRAPH_BUILD_REQUESTED,
            Self::GRAPH_BUILD_COMPLETED,
            Self::SOURCE_SYNC_REQUESTED,
            Self::SOURCE_SYNC_COMPLETED,
            Self::SOURCE_SYNC_FAILED,
        ]
    }
}
