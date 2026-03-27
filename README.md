# ConFuse Shared Middleware

Core shared libraries and utilities for ConFuse platform.

## Structure

This repository provides language-specific common libraries following consistent naming and structure:

### Rust
```
rust/confuse-common/
├── src/
│   ├── models/          # Data structures
│   ├── middleware/       # HTTP middleware
│   ├── connectivity/     # Database/network clients
│   ├── config/          # Configuration utilities
│   ├── observability/    # Logging/metrics
│   └── events/          # Event handling
└── Cargo.toml           # Package definition
```

### TypeScript
```
typescript/confuse-common/
├── src/
│   ├── events/          # Event schemas and types
│   ├── producer.ts      # Kafka event producer
│   ├── consumer.ts      # Kafka event consumer
│   ├── config.ts        # Configuration utilities
│   ├── middleware/      # Authentication, rate limiting
│   └── index.ts        # Unified exports
└── package.json         # @confuse/common package
```

### Python
```
python/confuse-common/
├── src/
│   └── confuse_common/
│       ├── events/      # Event schemas and Kafka helpers
│       ├── middleware/  # Authentication, rate limiting, security
│       └── config/      # Base classes and configuration
├── pyproject.toml        # confuse-common package
└── README.md
```

## Usage

### Rust
```toml
[dependencies]
confuse-common = { version = "0.1.0" }
```

### Python  
```python
from confuse_common.events import EventProducer
from confuse_common.middleware import AuthMiddleware
```

### TypeScript
```typescript
import { EventProducer } from '@confuse/common';
```

## Installation

Each language package can be installed independently:

- **Rust**: Add to Cargo.toml dependencies
- **Python**: `pip install confuse-common`
- **TypeScript**: `npm install @confuse/common`

## Version

All packages follow semantic versioning with consistent major/minor versions across languages.

## License

Proprietary - ConFuse Platform
