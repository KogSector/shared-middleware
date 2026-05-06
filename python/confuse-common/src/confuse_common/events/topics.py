"""
Topic Constants for ConFuse Platform

Defines all Kafka topics used in the platform.
These should match the topics pre-created in Confluent Cloud.
"""


class Topics:
    """All Kafka topics used by ConFuse platform"""
    
    # =========================================================================
    # Code Processing Pipeline
    # =========================================================================
    
    # Topic for code file ingestion events
    # Producer: data-connector
    # Consumer: unified-processor
    CODE_INGESTED = "code.ingested"
    
    # Topic for processed code events
    # Producer: unified-processor
    # Consumer: embeddings-service
    CODE_PROCESSED = "code.processed"
    
    # =========================================================================
    # Document Processing Pipeline
    # =========================================================================
    
    # Topic for document ingestion events
    # Producer: data-connector
    # Consumer: unified-processor
    DOCS_INGESTED = "docs.ingested"
    
    # Topic for processed document events
    # Producer: unified-processor
    # Consumer: embeddings-service
    DOCS_PROCESSED = "docs.processed"
    
    # =========================================================================
    # Embedding Pipeline
    # =========================================================================
    
    # Topic for embedding generation events
    # Producer: embeddings-service
    # Consumer: relation-graph
    EMBEDDING_GENERATED = "embedding.generated"
    
    # =========================================================================
    # Knowledge Graph
    # =========================================================================
    
    # Topic for graph update events
    # Producer: relation-graph
    # Consumer: (monitoring, notifications)
    GRAPH_UPDATED = "graph.updated"
    
    # Topic for graph build requests
    # Producer: unified-processor, api-backend
    # Consumer: relation-graph
    GRAPH_BUILD_REQUESTED = "graph.build.requested"
    
    # Topic for graph build completion events
    # Producer: relation-graph
    # Consumer: (monitoring, notifications)
    GRAPH_BUILD_COMPLETED = "graph.build.completed"
    
    # =========================================================================
    # Source Sync
    # =========================================================================
    
    # Topic for source sync requests
    # Producer: api-backend
    # Consumer: data-connector
    SOURCE_SYNC_REQUESTED = "source.sync.requested"
    
    # Topic for source sync completion events
    # Producer: data-connector
    # Consumer: (monitoring, notifications)
    SOURCE_SYNC_COMPLETED = "source.sync.completed"
    
    # Topic for source sync failure events
    # Producer: data-connector
    # Consumer: (monitoring, notifications)
    SOURCE_SYNC_FAILED = "source.sync.failed"
    
    # =========================================================================
    # Event-Driven Pipeline (Refactored Architecture)
    # =========================================================================
    
    # Topic for repository ingestion and update events
    # Producer: data-connector
    # Consumer: unified-processor
    REPO_EVENTS = "repo-events"
    
    # Topic for code chunk events
    # Producer: unified-processor
    # Consumer: embeddings-service
    CODE_CHUNKS = "code-chunks"
    
    # Topic for embedding events
    # Producer: embeddings-service
    # Consumer: storage services
    EMBEDDING_EVENTS = "embedding-events"
    
    # Dead letter queues for event-driven pipeline
    REPO_EVENTS_DLQ = "repo-events-dlq"
    CODE_CHUNKS_DLQ = "code-chunks-dlq"
    EMBEDDING_EVENTS_DLQ = "embedding-events-dlq"
    
    # =========================================================================
    # Graphify Episode Pipeline
    # =========================================================================
    
    # Topic for Graphify episode ingestion
    # Producer: data-connector, unified-processor, embeddings-service
    # Consumer: Graphify knowledge graph
    GRAPHIFY_EPISODES = "graphify.episodes.v1"
    
    # Dead-letter queue for failed Graphify episode processing
    GRAPHIFY_EPISODES_DLQ = "graphify.episodes.dlq"
    
    # =========================================================================
    # Dead Letter Queue
    # =========================================================================
    
    # Topic for failed processing events
    # Producer: any service on processing failure
    # Consumer: (monitoring, retry service)
    DLQ_PROCESSING_FAILED = "dlq.processing.failed"
    
    @classmethod
    def all(cls) -> list[str]:
        """Get all topics for Confluent Cloud provisioning"""
        return [
            cls.CODE_INGESTED,
            cls.CODE_PROCESSED,
            cls.DOCS_INGESTED,
            cls.DOCS_PROCESSED,
            cls.EMBEDDING_GENERATED,
            cls.GRAPH_UPDATED,
            cls.GRAPH_BUILD_REQUESTED,
            cls.GRAPH_BUILD_COMPLETED,
            cls.SOURCE_SYNC_REQUESTED,
            cls.SOURCE_SYNC_COMPLETED,
            cls.SOURCE_SYNC_FAILED,
            cls.GRAPHIFY_EPISODES,
            cls.GRAPHIFY_EPISODES_DLQ,
            cls.DLQ_PROCESSING_FAILED,
            cls.REPO_EVENTS,
            cls.CODE_CHUNKS,
            cls.EMBEDDING_EVENTS,
            cls.REPO_EVENTS_DLQ,
            cls.CODE_CHUNKS_DLQ,
            cls.EMBEDDING_EVENTS_DLQ,
        ]

    @classmethod
    def graphify(cls) -> list[str]:
        """Get Graphify-specific topics."""
        return [
            cls.GRAPHIFY_EPISODES,
            cls.GRAPHIFY_EPISODES_DLQ,
        ]
