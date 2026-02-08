"""
ConFuse Events - Shared Event Schemas and Kafka Helpers

This package provides:
- Event schemas matching the Protobuf definitions
- Kafka producer/consumer helpers for Confluent Cloud
- Configuration utilities for Confluent Cloud environments
"""

from confuse_events.config import KafkaConfig, Environment
from confuse_events.topics import Topics
from confuse_events.events import (
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
    EmbeddingGeneratedEvent,
    GraphUpdatedEvent,
    GraphBuildRequestedEvent,
    GraphBuildCompletedEvent,
    SourceSyncRequestedEvent,
    SourceSyncCompletedEvent,
    SourceSyncFailedEvent,
    ProcessingFailedEvent,
)
from confuse_events.producer import EventProducer
from confuse_events.consumer import EventConsumer, EventHandler

__version__ = "0.1.0"

__all__ = [
    # Config
    "KafkaConfig",
    "Environment",
    "Topics",
    # Common types
    "EventHeaders",
    "EventMetadata",
    "FileType",
    "SourceType",
    # Code events
    "CodeIngestedEvent",
    "CodeProcessedEvent",
    "CodeChunk",
    # Document events
    "DocsIngestedEvent",
    "DocsProcessedEvent",
    "DocChunk",
    # Other events
    "EmbeddingGeneratedEvent",
    "GraphUpdatedEvent",
    "GraphBuildRequestedEvent",
    "GraphBuildCompletedEvent",
    "SourceSyncRequestedEvent",
    "SourceSyncCompletedEvent",
    "SourceSyncFailedEvent",
    "ProcessingFailedEvent",
    # Producer/Consumer
    "EventProducer",
    "EventConsumer",
    "EventHandler",
]
