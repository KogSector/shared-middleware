# ConFuse Shared Middleware

> **Common Libraries and Utilities for Microservices**

## Overview

The **shared-middleware** repository contains common libraries, utilities, and definitions used across all ConFuse microservices. It provides consistent functionality for authentication, logging, events, database access, and communication patterns.

## Purpose

- **Code Reuse**: Shared functionality across services
- **Consistency**: Standardized patterns and interfaces
- **Maintainability**: Centralized updates and fixes
- **Interoperability**: Cross-language communication protocols

## Architecture

```
shared-middleware/
├── rust/                    # Rust libraries
│   ├── confuse-common/      # Core Rust library
│   ├── confuse-events/      # Event handling
│   └── kafka/              # Kafka integration
├── typescript/              # TypeScript packages
│   ├── types/              # Type definitions
│   ├── auth/               # Auth utilities
│   └── events/             # Event clients
├── python/                  # Python packages
│   ├── confuse_common/     # Core Python library
│   ├── events/             # Event handling
│   └── auth/               # Auth utilities
├── schemas/                 # Protocol definitions
│   ├── protobuf/           # gRPC definitions
│   ├── events/             # Event schemas
│   └── openapi/            # API specifications
└── docs/                    # Documentation
```

## Language-Specific Libraries

### Rust Libraries

#### confuse-common
**Purpose**: Core Rust library for all ConFuse services

**Features**:
- **Database**: FalkorDB and PostgreSQL clients
- **Authentication**: JWT validation and user context
- **Logging**: Structured logging with tracing
- **Metrics**: Prometheus metrics collection
- **Configuration**: Environment-based configuration
- **Error Handling**: Standardized error types

**Usage**:
```toml
[dependencies]
confuse-common = { path = "../shared-middleware/rust/confuse-common" }
confuse-common = { git = "https://github.com/confuse/shared-middleware", features = ["axum-support", "kafka"] }
```

**Features**:
- `axum-support`: Axum web framework integration
- `kafka`: Kafka event publishing
- `metrics`: Prometheus metrics
- `database`: Database client utilities

#### confuse-events
**Purpose**: Event handling and publishing for Rust services

**Features**:
- **Kafka Producer**: High-performance event publishing
- **Event Types**: Standardized event definitions
- **Serialization**: Efficient binary serialization
- **Error Handling**: Retry logic and error recovery

**Usage**:
```rust
use confuse_events::{EventProducer, Event, EventType};

let producer = EventProducer::new("kafka:9092")?;
let event = Event::new(EventType::ChunkProcessed, chunk_data);
producer.publish(event).await?;
```

### TypeScript Libraries

#### @confuse/types
**Purpose**: Shared TypeScript type definitions

**Features**:
- **API Types**: Request/response types for all services
- **Event Types**: Event payload definitions
- **Database Types**: Entity and model types
- **Configuration Types**: Environment variable types

**Usage**:
```typescript
import { User, Source, EmbeddingRequest } from '@confuse/types';

const user: User = {
  id: 'user-123',
  email: 'user@example.com',
  permissions: ['read', 'write']
};
```

#### @confuse/auth
**Purpose**: Authentication utilities for TypeScript services

**Features**:
- **JWT Validation**: Token validation and parsing
- **User Context**: User information extraction
- **Permission Checking**: Role-based authorization
- **Middleware**: Express.js authentication middleware

**Usage**:
```typescript
import { validateToken, requirePermission } from '@confuse/auth';

const user = await validateToken(token);
requirePermission(user, 'sources:read');
```

#### @confuse/events
**Purpose**: Event client for TypeScript services

**Features**:
- **Kafka Client**: Event publishing and consumption
- **Event Types**: Type-safe event definitions
- **Serialization**: JSON serialization utilities
- **Error Handling**: Connection and publishing errors

**Usage**:
```typescript
import { EventClient, ChunkProcessedEvent } from '@confuse/events';

const client = new EventClient('kafka:9092');
const event = new ChunkProcessedEvent(chunkId, metadata);
await client.publish(event);
```

### Python Libraries

#### confuse-common
**Purpose**: Core Python library for ConFuse services

**Features**:
- **Database**: Async database clients
- **Authentication**: JWT validation and user management
- **Logging**: Structured logging with correlation IDs
- **Configuration**: Environment-based configuration
- **HTTP Clients**: Async HTTP client utilities

**Usage**:
```python
from confuse_common import (
    DatabaseClient, 
    AuthManager, 
    Logger, 
    Config
)

db = DatabaseClient(config.database_url)
auth = AuthManager(config.auth_secret)
logger = Logger("service-name")
```

#### confuse-events
**Purpose**: Event handling for Python services

**Features**:
- **Kafka Producer**: Async event publishing
- **Event Types**: Pydantic event models
- **Serialization**: JSON and binary serialization
- **Error Handling**: Retry logic and error recovery

**Usage**:
```python
from confuse_events import EventProducer, ChunkProcessedEvent

producer = EventProducer("kafka:9092")
event = ChunkProcessedEvent(chunk_id="chunk-123")
await producer.publish(event)
```

## Schemas and Protocols

### Protocol Buffers

#### gRPC Service Definitions
**Location**: `schemas/protobuf/`

**Services**:
- `auth.proto`: Authentication service definitions
- `embeddings.proto`: Embeddings service definitions
- `data_connector.proto`: Data connector service definitions
- `unified_processor.proto`: Unified processor service definitions

**Example**:
```protobuf
syntax = "proto3";

package confuse.auth.v1;

service AuthService {
  rpc ValidateToken(ValidateTokenRequest) returns (ValidateTokenResponse);
  rpc GetUserInfo(GetUserInfoRequest) returns (GetUserInfoResponse);
}

message ValidateTokenRequest {
  string token = 1;
}

message ValidateTokenResponse {
  bool valid = 1;
  User user = 2;
}
```

### Event Schemas

#### Event Definitions
**Location**: `schemas/events/`

**Event Types**:
- `chunk_processed.json`: Chunk processing completion
- `embedding_generated.json`: Embedding generation events
- `source_synced.json`: Source synchronization events
- `user_action.json`: User action tracking events

**Example**:
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "title": "ChunkProcessedEvent",
  "properties": {
    "event_type": {
      "type": "string",
      "enum": ["chunk_processed"]
    },
    "chunk_id": {
      "type": "string",
      "format": "uuid"
    },
    "source_id": {
      "type": "string",
      "format": "uuid"
    },
    "processed_at": {
      "type": "string",
      "format": "date-time"
    },
    "metadata": {
      "type": "object",
      "properties": {
        "file_path": {"type": "string"},
        "language": {"type": "string"},
        "size": {"type": "integer"}
      }
    }
  },
  "required": ["event_type", "chunk_id", "source_id", "processed_at"]
}
```

### OpenAPI Specifications

#### API Definitions
**Location**: `schemas/openapi/`

**Specifications**:
- `auth.yaml`: Authentication API specification
- `data_connector.yaml`: Data connector API specification
- `embeddings.yaml`: Embeddings service API specification

## Integration Patterns

### Authentication Flow

#### Rust Service Integration
```rust
use confuse_common::{AuthMiddleware, UserContext};

// Middleware setup
let auth_middleware = AuthMiddleware::new(config.auth_secret);

// Route protection
async fn protected_route(
    Extension(user): Extension<UserContext>
) -> Result<Json<Response>, AuthError> {
    // User is authenticated
    if !user.has_permission("sources:read") {
        return Err(AuthError::InsufficientPermissions);
    }
    
    // Process request
    Ok(Json(Response { data: "protected data" }))
}
```

#### TypeScript Service Integration
```typescript
import { authMiddleware, requirePermission } from '@confuse/auth';

// Express middleware
app.use('/api/v1', authMiddleware);

// Route protection
app.get('/api/v1/sources', requirePermission('sources:read'), async (req, res) => {
    const user = req.user;
    // Process request
});
```

#### Python Service Integration
```python
from confuse_common import AuthMiddleware, require_permission
from fastapi import FastAPI, Depends

app = FastAPI()

# Authentication dependency
async def get_current_user(token: str) -> User:
    return await auth_manager.validate_token(token)

# Protected route
@app.get("/api/v1/sources")
@require_permission("sources:read")
async def get_sources(user: User = Depends(get_current_user)):
    # Process request
    pass
```

### Event Publishing

#### Rust Event Publishing
```rust
use confuse_events::{EventProducer, ChunkProcessedEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let producer = EventProducer::new("kafka:9092")?;
    
    let event = ChunkProcessedEvent {
        chunk_id: "chunk-123".to_string(),
        source_id: "source-456".to_string(),
        processed_at: Utc::now(),
        metadata: ChunkMetadata {
            file_path: "/path/to/file".to_string(),
            language: "rust".to_string(),
            size: 1024,
        }
    };
    
    producer.publish(event).await?;
    Ok(())
}
```

#### TypeScript Event Publishing
```typescript
import { EventClient, ChunkProcessedEvent } from '@confuse/events';

const client = new EventClient('kafka:9092');

const event = new ChunkProcessedEvent({
  chunkId: 'chunk-123',
  sourceId: 'source-456',
  processedAt: new Date(),
  metadata: {
    filePath: '/path/to/file',
    language: 'typescript',
    size: 1024
  }
});

await client.publish(event);
```

#### Python Event Publishing
```python
from confuse_events import EventProducer, ChunkProcessedEvent
import asyncio

async def main():
    producer = EventProducer("kafka:9092")
    
    event = ChunkProcessedEvent(
        chunk_id="chunk-123",
        source_id="source-456",
        processed_at=datetime.utcnow(),
        metadata=ChunkMetadata(
            file_path="/path/to/file",
            language="python",
            size=1024
        )
    )
    
    await producer.publish(event)

asyncio.run(main())
```

### Database Access

#### FalkorDB Integration
```rust
use confuse_common::{FalkorDBClient, VectorChunk};

let client = FalkorDBClient::new("localhost:6379")?;
let chunk = VectorChunk {
    id: "chunk-123".to_string(),
    embedding: vec![0.1, 0.2, 0.3],
    content: "Sample content".to_string(),
    metadata: serde_json::json!({"key": "value"}),
};

client.store_vector_chunk(&chunk).await?;
```

#### TypeScript FalkorDB Client
```typescript
import { FalkorDBClient, VectorChunk } from '@confuse/database';

const client = new FalkorDBClient('localhost:6379');

const chunk = new VectorChunk({
  id: 'chunk-123',
  embedding: [0.1, 0.2, 0.3],
  content: 'Sample content',
  metadata: { key: 'value' }
});

await client.storeVectorChunk(chunk);
```

#### Python FalkorDB Client
```python
from confuse_common import FalkorDBClient, VectorChunk

client = FalkorDBClient("localhost:6379")

chunk = VectorChunk(
    id="chunk-123",
    embedding=[0.1, 0.2, 0.3],
    content="Sample content",
    metadata={"key": "value"}
)

await client.store_vector_chunk(chunk)
```

## Configuration Management

### Environment Variables

#### Standard Configuration
```bash
# Authentication
AUTH_SECRET_KEY=your_secret_key
AUTH_TOKEN_TTL=3600

# Database
DATABASE_URL=postgresql://user:pass@localhost/db
FALKORDB_URL=redis://localhost:6379

# Events
KAFKA_BOOTSTRAP_SERVERS=localhost:9092
KAFKA_CLIENT_ID=service-name

# Logging
LOG_LEVEL=info
LOG_FORMAT=json

# Metrics
METRICS_ENABLED=true
METRICS_PORT=9090
```

#### Configuration Classes
```rust
// Rust configuration
use confuse_common::Config;

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub auth: AuthConfig,
    pub database: DatabaseConfig,
    pub events: EventsConfig,
    pub logging: LoggingConfig,
}

impl Config for ServiceConfig {
    fn from_env() -> Result<Self, ConfigError> {
        // Load from environment variables
    }
}
```

```typescript
// TypeScript configuration
export interface ServiceConfig {
  auth: AuthConfig;
  database: DatabaseConfig;
  events: EventsConfig;
  logging: LoggingConfig;
}

export const config: ServiceConfig = {
  auth: {
    secretKey: process.env.AUTH_SECRET_KEY!,
    tokenTtl: parseInt(process.env.AUTH_TOKEN_TTL || '3600')
  },
  // ... other config
};
```

```python
# Python configuration
from pydantic import BaseSettings

class ServiceConfig(BaseSettings):
    auth_secret_key: str
    auth_token_ttl: int = 3600
    database_url: str
    falkordb_url: str = "redis://localhost:6379"
    
    class Config:
        env_file = ".env"
```

## Error Handling

### Standardized Error Types

#### Rust Error Types
```rust
use confuse_common::{ConFuseError, ErrorKind};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Authentication failed: {0}")]
    Auth(#[from] AuthError),
    
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    #[error("Event publishing failed: {0}")]
    Events(#[from] EventError),
    
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
}

impl ConFuseError for ServiceError {
    fn kind(&self) -> ErrorKind {
        match self {
            ServiceError::Auth(_) => ErrorKind::Authentication,
            ServiceError::Database(_) => ErrorKind::Database,
            ServiceError::Events(_) => ErrorKind::Events,
            ServiceError::Config(_) => ErrorKind::Configuration,
        }
    }
}
```

#### TypeScript Error Types
```typescript
export class ConFuseError extends Error {
  constructor(
    public readonly kind: ErrorKind,
    public readonly message: string,
    public readonly cause?: Error
  ) {
    super(message);
  }
}

export enum ErrorKind {
  Authentication = 'authentication',
  Database = 'database',
  Events = 'events',
  Configuration = 'configuration',
}
```

#### Python Error Types
```python
from confuse_common import ConFuseError, ErrorKind

class ServiceError(ConFuseError):
    def __init__(self, kind: ErrorKind, message: str, cause: Exception = None):
        super().__init__(kind, message, cause)

class AuthError(ServiceError):
    def __init__(self, message: str, cause: Exception = None):
        super().__init__(ErrorKind.AUTHENTICATION, message, cause)
```

## Testing Utilities

### Mock Services

#### Rust Test Utilities
```rust
use confuse_common::testing::{MockAuth, MockDatabase, MockEventProducer};

#[tokio::test]
async fn test_service_logic() {
    let mock_auth = MockAuth::new();
    let mock_db = MockDatabase::new();
    let mock_events = MockEventProducer::new();
    
    // Setup mocks
    mock_auth.expect_validate_token().returning(|_| Ok(test_user()));
    
    // Test service logic
    let result = service_logic(&mock_auth, &mock_db, &mock_events).await;
    assert!(result.is_ok());
}
```

#### TypeScript Test Utilities
```typescript
import { MockAuthClient, MockEventClient } from '@confuse/testing';

describe('Service Logic', () => {
  let mockAuth: MockAuthClient;
  let mockEvents: MockEventClient;
  
  beforeEach(() => {
    mockAuth = new MockAuthClient();
    mockEvents = new MockEventClient();
  });
  
  it('should process request successfully', async () => {
    mockAuth.validateToken.mockResolvedValue(mockUser);
    
    const result = await serviceLogic(mockAuth, mockEvents);
    expect(result).toBeDefined();
  });
});
```

#### Python Test Utilities
```python
from confuse_common.testing import MockAuthClient, MockEventProducer
import pytest

@pytest.mark.asyncio
async def test_service_logic():
    mock_auth = MockAuthClient()
    mock_events = MockEventProducer()
    
    # Setup mocks
    mock_auth.validate_token.return_value = mock_user()
    
    # Test service logic
    result = await service_logic(mock_auth, mock_events)
    assert result is not None
```

## Development Guidelines

### Adding New Libraries

#### 1. Define Interface
```rust
// Rust trait definition
pub trait NewService {
    async fn process(&self, input: Input) -> Result<Output, ServiceError>;
}
```

#### 2. Implement for Each Language
```rust
// Rust implementation
pub struct RustNewService {
    client: HttpClient,
}

impl NewService for RustNewService {
    async fn process(&self, input: Input) -> Result<Output, ServiceError> {
        // Implementation
    }
}
```

```typescript
// TypeScript implementation
export class TypeScriptNewService implements NewService {
  private client: HttpClient;
  
  async process(input: Input): Promise<Output> {
    // Implementation
  }
}
```

```python
# Python implementation
class PythonNewService(NewService):
    def __init__(self, client: HttpClient):
        self.client = client
    
    async def process(self, input: Input) -> Output:
        # Implementation
```

#### 3. Add Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_new_service() {
        // Test implementation
    }
}
```

#### 4. Update Documentation
- Update README files
- Add API documentation
- Include usage examples

### Version Management

#### Semantic Versioning
- **Major**: Breaking changes
- **Minor**: New features, backward compatible
- **Patch**: Bug fixes, backward compatible

#### Release Process
1. Update version numbers
2. Update CHANGELOG
3. Tag release
4. Update dependent services

## Best Practices

### Code Organization
- **Consistent Structure**: Follow established patterns
- **Clear Interfaces**: Well-defined APIs
- **Documentation**: Comprehensive documentation
- **Testing**: Full test coverage

### Performance
- **Efficient Serialization**: Use binary formats when possible
- **Connection Pooling**: Reuse connections
- **Caching**: Cache frequently accessed data
- **Async Operations**: Use async/await patterns

### Security
- **Input Validation**: Validate all inputs
- **Error Handling**: Don't leak sensitive information
- **Authentication**: Secure token handling
- **Communication**: Use TLS for all network communication

## Troubleshooting

### Common Issues

#### "Import errors"
- Check package installation
- Verify correct import paths
- Ensure compatible versions

#### "Authentication failures"
- Verify secret key configuration
- Check token format
- Ensure proper clock synchronization

#### "Event publishing failures"
- Check Kafka connectivity
- Verify topic configuration
- Ensure proper serialization

#### "Database connection issues"
- Verify connection strings
- Check network connectivity
- Ensure proper permissions

## Contributing

### Development Setup
```bash
# Clone repository
git clone https://github.com/confuse/shared-middleware.git
cd shared-middleware

# Setup Rust
cargo build

# Setup TypeScript
cd typescript && npm install

# Setup Python
cd python && pip install -e .

# Run tests
cargo test
npm test
pytest
```

### Pull Request Process
1. Create feature branch
2. Implement changes with tests
3. Update documentation
4. Ensure all tests pass
5. Submit pull request

## License

Proprietary - ConFuse Team
