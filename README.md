# ConFuse Shared Middleware

Core shared libraries and middleware for the ConFuse platform.

## Contents

- **rust/confuse-common**: Unified Rust library for events, database, observability, and models.
- **typescript/**: Shared TypeScript definitions and event clients.
- **python/**: Shared Python packages and event clients.
- **schemas/**: Protobuf definitions for cross-language events.

## Usage

This repository is designed to be included as a Git submodule or package dependency in ConFuse microservices.

### Local Development

This folder is expected to be present at `../shared-middleware` relative to service directories or mapped via Docker build context.

## License

Proprietary - ConFuse Platform
