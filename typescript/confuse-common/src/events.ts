/**
 * Event Definitions for ConFuse Platform
 *
 * TypeScript event interfaces that correspond to the Protobuf definitions.
 */

import { v4 as uuidv4 } from 'uuid';
import { Topics } from './topics';

// =============================================================================
// Common Types
// =============================================================================

export enum FileType {
    UNKNOWN = 'unknown',
    CODE = 'code',
    DOCUMENT = 'document',
}

export enum SourceType {
    UNKNOWN = 'unknown',
    GITHUB = 'github',
    GITLAB = 'gitlab',
    BITBUCKET = 'bitbucket',
    LOCAL = 'local',
    GOOGLE_DRIVE = 'google_drive',
    NOTION = 'notion',
    FILE_UPLOAD = 'file_upload',
    DROPBOX = 'dropbox',
    ONEDRIVE = 'onedrive',
}

export interface EventHeaders {
    event_id: string;
    event_type: string;
    timestamp: string;
    source_service: string;
    correlation_id?: string;
    trace_id?: string;
}

export interface EventMetadata {
    retry_count: number;
    original_event_id?: string;
    user_id?: string;
    tenant_id?: string;
}

export function createEventHeaders(
    sourceService: string,
    eventType: string
): EventHeaders {
    return {
        event_id: uuidv4(),
        event_type: eventType,
        timestamp: new Date().toISOString(),
        source_service: sourceService,
    };
}

export function createEventMetadata(): EventMetadata {
    return {
        retry_count: 0,
    };
}

// =============================================================================
// Code Events
// =============================================================================

export interface CodeIngestedEvent {
    headers: EventHeaders;
    metadata: EventMetadata;
    file_id: string;
    source_id: string;
    file_path: string;
    file_name: string;
    file_extension: string;
    file_size_bytes: number;
    content_hash: string;
    storage_location: string;
    language: string;
    is_config: boolean;
}

export interface CodeChunk {
    chunk_id: string;
    content: string;
    start_line: number;
    end_line: number;
    chunk_type: string;
    name?: string;
    parent_name?: string;
    signature?: string;
}

export interface CodeProcessedEvent {
    headers: EventHeaders;
    metadata: EventMetadata;
    file_id: string;
    source_id: string;
    language: string;
    chunks: CodeChunk[];
    functions_count: number;
    classes_count: number;
    imports: string[];
    processing_time_ms: number;
}

// =============================================================================
// Document Events
// =============================================================================

export interface DocsIngestedEvent {
    headers: EventHeaders;
    metadata: EventMetadata;
    file_id: string;
    source_id: string;
    file_path: string;
    file_name: string;
    file_extension: string;
    file_size_bytes: number;
    content_hash: string;
    storage_location: string;
    document_type: string;
}

export interface DocChunk {
    chunk_id: string;
    content: string;
    page_number?: number;
    chunk_type: string;
    section_heading?: string;
}

export interface ExtractedTable {
    table_id: string;
    page_number?: number;
    row_count: number;
    column_count: number;
    content_markdown: string;
}

export interface DocsProcessedEvent {
    headers: EventHeaders;
    metadata: EventMetadata;
    file_id: string;
    source_id: string;
    document_type: string;
    chunks: DocChunk[];
    page_count: number;
    word_count: number;
    has_tables: boolean;
    tables: ExtractedTable[];
    has_images: boolean;
    processing_time_ms: number;
}

// =============================================================================
// Embedding Events
// =============================================================================

export interface EmbeddingGeneratedEvent {
    headers: EventHeaders;
    metadata: EventMetadata;
    file_id: string;
    source_id: string;
    chunk_ids: string[];
    embedding_model: string;
    embedding_dimension: number;
    total_chunks: number;
    vector_storage_location?: string;
    processing_time_ms: number;
}

// =============================================================================
// Graph Events
// =============================================================================

export interface GraphRelationship {
    source_node_id: string;
    target_node_id: string;
    relationship_type: string;
    properties?: string;
}

export interface GraphUpdatedEvent {
    headers: EventHeaders;
    metadata: EventMetadata;
    source_id: string;
    file_id?: string;
    nodes_added: number;
    nodes_updated: number;
    edges_added: number;
    edges_updated: number;
    sample_relationships: GraphRelationship[];
    processing_time_ms: number;
}

export interface GraphBuildRequestedEvent {
    headers: EventHeaders;
    metadata: EventMetadata;
    source_id: string;
    force_rebuild: boolean;
    build_type: string;
}

export interface GraphBuildCompletedEvent {
    headers: EventHeaders;
    metadata: EventMetadata;
    source_id: string;
    relationships_created: number;
    entities_created: number;
    build_time_ms: number;
    build_type: string;
}

// =============================================================================
// Source Sync Events
// =============================================================================

export interface SourceSyncRequestedEvent {
    headers: EventHeaders;
    metadata: EventMetadata;
    source_id: string;
    source_type: SourceType;
    source_url: string;
    branch?: string;
    access_token?: string;
    full_sync: boolean;
    requested_by?: string;
}

export interface SourceSyncCompletedEvent {
    headers: EventHeaders;
    metadata: EventMetadata;
    source_id: string;
    files_processed: number;
    code_files_count: number;
    doc_files_count: number;
    duration_ms: number;
    job_id?: string;
}

export interface SourceSyncFailedEvent {
    headers: EventHeaders;
    metadata: EventMetadata;
    source_id: string;
    error: string;
    error_code: string;
    job_id?: string;
}

// =============================================================================
// Auth Events
// =============================================================================

export interface AuthEvent {
    headers: EventHeaders;
    metadata: EventMetadata;
    user_id: string;
    event_type: string; // login, logout, token_refresh, login_failed
    ip_address?: string;
    user_agent?: string;
    success: boolean;
    failure_reason?: string;
}

export interface SessionEvent {
    headers: EventHeaders;
    metadata: EventMetadata;
    session_id: string;
    user_id: string;
    event_type: string; // created, destroyed, expired, refreshed
    expires_at: number; // unix timestamp
    device_id?: string;
}

// =============================================================================
// DLQ Events
// =============================================================================

export interface ProcessingFailedEvent {
    headers: EventHeaders;
    metadata: EventMetadata;
    original_topic: string;
    original_event_id: string;
    error_message: string;
    error_type: string;
    error_code: string;
    retry_count: number;
    is_retryable: boolean;
    original_payload: string;
    error_timestamp: string;
    failed_service: string;
}

// =============================================================================
// Topic Helpers
// =============================================================================

export function getTopicForEvent(event: { headers: EventHeaders }): string {
    const eventType = event.headers.event_type;

    switch (eventType) {
        case 'CODE_INGESTED':
            return Topics.CODE_INGESTED;
        case 'CODE_PROCESSED':
            return Topics.CODE_PROCESSED;
        case 'DOCS_INGESTED':
            return Topics.DOCS_INGESTED;
        case 'DOCS_PROCESSED':
            return Topics.DOCS_PROCESSED;
        case 'EMBEDDING_GENERATED':
            return Topics.EMBEDDING_GENERATED;
        case 'GRAPH_UPDATED':
            return Topics.GRAPH_UPDATED;
        case 'GRAPH_BUILD_REQUESTED':
            return Topics.GRAPH_BUILD_REQUESTED;
        case 'GRAPH_BUILD_COMPLETED':
            return Topics.GRAPH_BUILD_COMPLETED;
        case 'SOURCE_SYNC_REQUESTED':
            return Topics.SOURCE_SYNC_REQUESTED;
        case 'SOURCE_SYNC_COMPLETED':
            return Topics.SOURCE_SYNC_COMPLETED;
        case 'SOURCE_SYNC_FAILED':
            return Topics.SOURCE_SYNC_FAILED;
        case 'PROCESSING_FAILED':
            return Topics.DLQ_PROCESSING_FAILED;
        default:
            throw new Error(`Unknown event type: ${eventType}`);
    }
}
