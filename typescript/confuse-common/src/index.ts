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
export { KafkaConfig, Environment, ConfigError } from './config.js';

// Topics
export { Topics } from './topics.js';

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
} from './events.js';

// Producer/Consumer
export { EventProducer } from './producer.js';
export { EventConsumer, EventHandler } from './consumer.js';

// Middleware
export { AuthMiddleware, RateLimitMiddleware, SecurityHeadersMiddleware } from './middleware/index.js';
