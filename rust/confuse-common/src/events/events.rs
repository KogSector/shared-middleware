//! Event Definitions for ConFuse Platform

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// =============================================================================
// Common Types
// =============================================================================

/// Event headers included in all events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventHeaders {
    pub event_id: String,
    pub event_type: String,
    pub timestamp: String,
    pub source_service: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

impl EventHeaders {
    pub fn new(source_service: impl Into<String>, event_type: impl Into<String>) -> Self {
        Self {
            event_id: Uuid::new_v4().to_string(),
            event_type: event_type.into(),
            timestamp: Utc::now().to_rfc3339(),
            source_service: source_service.into(),
            correlation_id: None,
            trace_id: None,
        }
    }
    
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }
    
    pub fn with_trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }
}

/// Event metadata for processing context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventMetadata {
    #[serde(default)]
    pub retry_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_event_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
}

/// File type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FileType {
    Unknown,
    Code,
    Document,
}

/// Source types for ingestion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Unknown,
    Github,
    Gitlab,
    Bitbucket,
    Local,
    GoogleDrive,
    Notion,
    FileUpload,
    Dropbox,
    Onedrive,
}

// =============================================================================
// Code Events
// =============================================================================

/// Event published when a code file is ingested
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeIngestedEvent {
    pub headers: EventHeaders,
    #[serde(default)]
    pub metadata: EventMetadata,
    pub file_id: String,
    pub source_id: String,
    pub file_path: String,
    pub file_name: String,
    pub file_extension: String,
    pub file_size_bytes: u64,
    pub content_hash: String,
    pub storage_location: String,
    pub language: String,
    #[serde(default)]
    pub is_config: bool,
}

impl CodeIngestedEvent {
    pub fn topic() -> &'static str {
        crate::events::topics::Topics::CODE_INGESTED
    }
}

/// Processed code chunk with AST information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChunk {
    pub chunk_id: String,
    pub content: String,
    pub start_line: u32,
    pub end_line: u32,
    pub chunk_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

/// Event published when a code file has been processed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeProcessedEvent {
    pub headers: EventHeaders,
    #[serde(default)]
    pub metadata: EventMetadata,
    pub file_id: String,
    pub source_id: String,
    pub language: String,
    pub chunks: Vec<CodeChunk>,
    #[serde(default)]
    pub functions_count: u32,
    #[serde(default)]
    pub classes_count: u32,
    #[serde(default)]
    pub imports: Vec<String>,
    pub processing_time_ms: u64,
}

impl CodeProcessedEvent {
    pub fn topic() -> &'static str {
        crate::events::topics::Topics::CODE_PROCESSED
    }
}

// =============================================================================
// Document Events
// =============================================================================

/// Event published when a document is ingested
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsIngestedEvent {
    pub headers: EventHeaders,
    #[serde(default)]
    pub metadata: EventMetadata,
    pub file_id: String,
    pub source_id: String,
    pub file_path: String,
    pub file_name: String,
    pub file_extension: String,
    pub file_size_bytes: u64,
    pub content_hash: String,
    pub storage_location: String,
    pub document_type: String,
}

impl DocsIngestedEvent {
    pub fn topic() -> &'static str {
        crate::events::topics::Topics::DOCS_INGESTED
    }
}

/// Processed document chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocChunk {
    pub chunk_id: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_number: Option<u32>,
    pub chunk_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_heading: Option<String>,
}

/// Table extracted from document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTable {
    pub table_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_number: Option<u32>,
    pub row_count: u32,
    pub column_count: u32,
    pub content_markdown: String,
}

/// Event published when a document has been processed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsProcessedEvent {
    pub headers: EventHeaders,
    #[serde(default)]
    pub metadata: EventMetadata,
    pub file_id: String,
    pub source_id: String,
    pub document_type: String,
    pub chunks: Vec<DocChunk>,
    #[serde(default)]
    pub page_count: u32,
    #[serde(default)]
    pub word_count: u32,
    #[serde(default)]
    pub has_tables: bool,
    #[serde(default)]
    pub tables: Vec<ExtractedTable>,
    #[serde(default)]
    pub has_images: bool,
    pub processing_time_ms: u64,
}

impl DocsProcessedEvent {
    pub fn topic() -> &'static str {
        crate::events::topics::Topics::DOCS_PROCESSED
    }
}

// =============================================================================
// Embedding Events
// =============================================================================

/// Event published when embeddings have been generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingGeneratedEvent {
    pub headers: EventHeaders,
    #[serde(default)]
    pub metadata: EventMetadata,
    pub file_id: String,
    pub source_id: String,
    pub chunk_ids: Vec<String>,
    pub embedding_model: String,
    pub embedding_dimension: u32,
    pub total_chunks: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_storage_location: Option<String>,
    pub processing_time_ms: u64,
}

impl EmbeddingGeneratedEvent {
    pub fn topic() -> &'static str {
        crate::events::topics::Topics::EMBEDDING_GENERATED
    }
}

// =============================================================================
// Graph Events
// =============================================================================

/// Relationship in knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRelationship {
    pub source_node_id: String,
    pub target_node_id: String,
    pub relationship_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<String>,
}

/// Event published when the knowledge graph has been updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphUpdatedEvent {
    pub headers: EventHeaders,
    #[serde(default)]
    pub metadata: EventMetadata,
    pub source_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    #[serde(default)]
    pub nodes_added: u32,
    #[serde(default)]
    pub nodes_updated: u32,
    #[serde(default)]
    pub edges_added: u32,
    #[serde(default)]
    pub edges_updated: u32,
    #[serde(default)]
    pub sample_relationships: Vec<GraphRelationship>,
    pub processing_time_ms: u64,
}

impl GraphUpdatedEvent {
    pub fn topic() -> &'static str {
        crate::events::topics::Topics::GRAPH_UPDATED
    }
}

/// Event to request a graph build
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphBuildRequestedEvent {
    pub headers: EventHeaders,
    #[serde(default)]
    pub metadata: EventMetadata,
    pub source_id: String,
    #[serde(default)]
    pub force_rebuild: bool,
    pub build_type: String,
}

impl GraphBuildRequestedEvent {
    pub fn topic() -> &'static str {
        crate::events::topics::Topics::GRAPH_BUILD_REQUESTED
    }
}

/// Event published when graph build completes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphBuildCompletedEvent {
    pub headers: EventHeaders,
    #[serde(default)]
    pub metadata: EventMetadata,
    pub source_id: String,
    pub relationships_created: u32,
    pub entities_created: u32,
    pub build_time_ms: u64,
    pub build_type: String,
}

impl GraphBuildCompletedEvent {
    pub fn topic() -> &'static str {
        crate::events::topics::Topics::GRAPH_BUILD_COMPLETED
    }
}

// =============================================================================
// Source Sync Events
// =============================================================================

/// Event triggered when source sync is requested
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSyncRequestedEvent {
    pub headers: EventHeaders,
    #[serde(default)]
    pub metadata: EventMetadata,
    pub source_id: String,
    pub source_type: SourceType,
    pub source_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(default)]
    pub full_sync: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_by: Option<String>,
}

impl SourceSyncRequestedEvent {
    pub fn topic() -> &'static str {
        crate::events::topics::Topics::SOURCE_SYNC_REQUESTED
    }

    pub fn new(source_id: impl Into<String>, source_type: SourceType, source_url: impl Into<String>) -> Self {
        Self {
            headers: EventHeaders::new("api-backend", "SOURCE_SYNC_REQUESTED"),
            metadata: EventMetadata::default(),
            source_id: source_id.into(),
            source_type,
            source_url: source_url.into(),
            branch: None,
            access_token: None,
            full_sync: false,
            requested_by: None,
        }
    }

    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        let uid = user_id.into();
        self.metadata.user_id = Some(uid.clone());
        self.requested_by = Some(uid);
        self
    }
}

/// Event triggered when source sync completes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSyncCompletedEvent {
    pub headers: EventHeaders,
    #[serde(default)]
    pub metadata: EventMetadata,
    pub source_id: String,
    pub files_processed: u32,
    #[serde(default)]
    pub code_files_count: u32,
    #[serde(default)]
    pub doc_files_count: u32,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
}

impl SourceSyncCompletedEvent {
    pub fn topic() -> &'static str {
        crate::events::topics::Topics::SOURCE_SYNC_COMPLETED
    }
}

/// Event triggered when source sync fails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSyncFailedEvent {
    pub headers: EventHeaders,
    #[serde(default)]
    pub metadata: EventMetadata,
    pub source_id: String,
    pub error: String,
    pub error_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
}

impl SourceSyncFailedEvent {
    pub fn topic() -> &'static str {
        crate::events::topics::Topics::SOURCE_SYNC_FAILED
    }
}

// =============================================================================
// Auth Events
// =============================================================================

/// Authentication event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthEvent {
    pub headers: EventHeaders,
    #[serde(default)]
    pub metadata: EventMetadata,
    pub user_id: String,
    pub event_type: String, // login, logout, token_refresh, login_failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
}

impl AuthEvent {
    pub fn topic() -> &'static str {
        crate::events::topics::Topics::AUTH_EVENTS
    }
}

/// Session event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    pub headers: EventHeaders,
    #[serde(default)]
    pub metadata: EventMetadata,
    pub session_id: String,
    pub user_id: String,
    pub event_type: String, // created, destroyed, expired, refreshed
    pub expires_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
}

impl SessionEvent {
    pub fn topic() -> &'static str {
        crate::events::topics::Topics::SESSION_EVENTS
    }
}

// =============================================================================
// DLQ Events
// =============================================================================

/// Event published when processing fails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingFailedEvent {
    pub headers: EventHeaders,
    #[serde(default)]
    pub metadata: EventMetadata,
    pub original_topic: String,
    pub original_event_id: String,
    pub error_message: String,
    pub error_type: String,
    pub error_code: String,
    pub retry_count: u32,
    #[serde(default)]
    pub is_retryable: bool,
    pub original_payload: String,
    pub error_timestamp: String,
    pub failed_service: String,
}

impl ProcessingFailedEvent {
    pub fn topic() -> &'static str {
        crate::events::topics::Topics::DLQ_PROCESSING_FAILED
    }
}
