/**
 * Topic Constants for ConFuse Platform
 *
 * Defines all Kafka topics used in the platform.
 * These should match the topics pre-created in Confluent Cloud.
 */

export const Topics = {
    // =========================================================================
    // Code Processing Pipeline
    // =========================================================================

    /** Topic for code file ingestion events - Producer: data-connector, Consumer: unified-processor */
    CODE_INGESTED: 'code.ingested',

    /** Topic for processed code events - Producer: unified-processor, Consumer: embeddings-service */
    CODE_PROCESSED: 'code.processed',

    // =========================================================================
    // Document Processing Pipeline
    // =========================================================================

    /** Topic for document ingestion events - Producer: data-connector, Consumer: unified-processor */
    DOCS_INGESTED: 'docs.ingested',

    /** Topic for processed document events - Producer: unified-processor, Consumer: embeddings-service */
    DOCS_PROCESSED: 'docs.processed',

    // =========================================================================
    // Embedding Pipeline
    // =========================================================================

    /** Topic for embedding generation events - Producer: embeddings-service, Consumer: relation-graph */
    EMBEDDING_GENERATED: 'embedding.generated',

    // =========================================================================
    // Knowledge Graph
    // =========================================================================

    /** Topic for graph update events - Producer: relation-graph */
    GRAPH_UPDATED: 'graph.updated',

    /** Topic for graph build requests - Producer: unified-processor, api-backend, Consumer: relation-graph */
    GRAPH_BUILD_REQUESTED: 'graph.build.requested',

    /** Topic for graph build completion events - Producer: relation-graph */
    GRAPH_BUILD_COMPLETED: 'graph.build.completed',

    // =========================================================================
    // Source Sync
    // =========================================================================

    /** Topic for source sync requests - Producer: api-backend, Consumer: data-connector */
    SOURCE_SYNC_REQUESTED: 'source.sync.requested',

    /** Topic for source sync completion events - Producer: data-connector */
    SOURCE_SYNC_COMPLETED: 'source.sync.completed',

    /** Topic for source sync failure events - Producer: data-connector */
    SOURCE_SYNC_FAILED: 'source.sync.failed',

    // =========================================================================
    // Auth Events
    // =========================================================================

    /** Topic for authentication events - Producer: auth-service */
    AUTH_EVENTS: 'auth.events',

    /** Topic for session events - Producer: auth-service */
    SESSION_EVENTS: 'session.events',

    // =========================================================================
    // Dead Letter Queue
    // =========================================================================

    /** Topic for failed processing events - Producer: any service on failure */
    DLQ_PROCESSING_FAILED: 'dlq.processing.failed',

    /** Get all topics for Confluent Cloud provisioning */
    all(): string[] {
        return [
            this.CODE_INGESTED,
            this.CODE_PROCESSED,
            this.DOCS_INGESTED,
            this.DOCS_PROCESSED,
            this.EMBEDDING_GENERATED,
            this.GRAPH_UPDATED,
            this.GRAPH_BUILD_REQUESTED,
            this.GRAPH_BUILD_COMPLETED,
            this.SOURCE_SYNC_REQUESTED,
            this.SOURCE_SYNC_COMPLETED,
            this.SOURCE_SYNC_FAILED,
            this.DLQ_PROCESSING_FAILED,
        ];
    },
} as const;
