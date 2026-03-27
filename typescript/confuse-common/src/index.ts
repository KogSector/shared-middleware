/**
 * ConFuse Common - Shared TypeScript Library
 *
 * This package provides:
 * - Event schemas matching the Protobuf definitions
 * - Kafka producer/consumer helpers with CONFLUENT_* support
 * - Configuration utilities for development and production environments
 * - Authentication, rate limiting, and security middleware
 */

// Config
export { KafkaConfig, Environment, ConfigError } from './config';

// Topics
export { Topics } from './topics';

// Events
export {
    EventHeaders,
    EventMetadata,
    FileType,
    SourceType,
    CodeIngestedEvent,
    CodeProcessedEvent,
    CodeChunk,
    DocsIngestedEvent,
    DocsProcessedEvent,
    DocChunk,
    ExtractedTable,
    EmbeddingGeneratedEvent,
    GraphUpdatedEvent,
    GraphRelationship,
    GraphBuildRequestedEvent,
    GraphBuildCompletedEvent,
    SourceSyncRequestedEvent,
    SourceSyncCompletedEvent,
    SourceSyncFailedEvent,
    AuthEvent,
    SessionEvent,
    ProcessingFailedEvent,
    createEventHeaders,
    createEventMetadata,
} from './events';

// Producer/Consumer
export { EventProducer } from './producer';
export { EventConsumer, EventHandler } from './consumer';

// Middleware
export { AuthMiddleware, RateLimitMiddleware, SecurityHeadersMiddleware } from './middleware';
