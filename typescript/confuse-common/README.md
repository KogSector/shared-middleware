# ConFuse Common TypeScript Library

Shared TypeScript library for ConFuse platform providing event schemas, middleware, and configuration utilities.

## Installation

```bash
npm install @confuse/common
```

## Usage

### Events and Kafka
```typescript
import { EventProducer, CodeIngestedEvent, Topics } from '@confuse/common';

// Create event producer
const producer = new EventProducer();

// Create and publish event
const event: CodeIngestedEvent = {
  headers: createEventHeaders(),
  fileId: "file123",
  filePath: "/src/main.ts",
  // ... other fields
};
await producer.publish(Topics.CODE_INGESTED, event);
```

### Authentication Middleware
```typescript
import { AuthMiddleware } from '@confuse/common';
import { Fastify } from 'fastify';

const app = Fastify();
app.register(AuthMiddleware);
```

### Configuration
```typescript
import { KafkaConfig } from '@confuse/common';

const config: KafkaConfig = {
  bootstrapServers: process.env.KAFKA_BOOTSTRAP_SERVERS,
  groupId: 'my-service-group',
  // ... other config
};
```

## Modules

- **events**: Event schemas and types
- **producer**: Kafka event producer
- **consumer**: Kafka event consumer
- **config**: Configuration utilities
- **middleware**: Authentication, rate limiting, security headers

## Version

0.2.0 - Breaking changes from previous package structure
