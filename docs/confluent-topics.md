# Confluent Cloud Topic Provisioning List

## Topics to Create

Create the following 11 topics in Confluent Cloud before production deployment.

### Code Processing Pipeline
| Topic | Partitions | Retention | Description |
|-------|------------|-----------|-------------|
| `code.ingested` | 6 | 7 days | Code file ingestion events from data-connector |
| `code.processed` | 6 | 7 days | Processed code events from unified-processor |

### Document Processing Pipeline
| Topic | Partitions | Retention | Description |
|-------|------------|-----------|-------------|
| `docs.ingested` | 6 | 7 days | Document ingestion events from data-connector |
| `docs.processed` | 6 | 7 days | Processed document events from unified-processor |

### Embedding Pipeline
| Topic | Partitions | Retention | Description |
|-------|------------|-----------|-------------|
| `embedding.generated` | 6 | 7 days | Embedding generation events from embeddings-service |

### Knowledge Graph
| Topic | Partitions | Retention | Description |
|-------|------------|-----------|-------------|
| `graph.updated` | 3 | 7 days | Graph update events from relation-graph |
| `graph.build.requested` | 3 | 7 days | Graph build requests to relation-graph |
| `graph.build.completed` | 3 | 7 days | Graph build completion events from relation-graph |

### Source Sync
| Topic | Partitions | Retention | Description |
|-------|------------|-----------|-------------|
| `source.sync.requested` | 3 | 7 days | Source sync requests to data-connector |
| `source.sync.completed` | 3 | 7 days | Source sync completion events from data-connector |
| `source.sync.failed` | 3 | 7 days | Source sync failure events from data-connector |

### Dead Letter Queue
| Topic | Partitions | Retention | Description |
|-------|------------|-----------|-------------|
| `dlq.processing.failed` | 3 | 30 days | Failed processing events (longer retention for debugging) |

## Producer/Consumer Mapping

| Service | Produces | Consumes |
|---------|----------|----------|
| api-backend | `source.sync.requested`, `graph.build.requested` | - |
| data-connector | `code.ingested`, `docs.ingested`, `source.sync.completed`, `source.sync.failed` | `source.sync.requested` |
| unified-processor | `code.processed`, `docs.processed`, `graph.build.requested` | `code.ingested`, `docs.ingested` |
| embeddings-service | `embedding.generated` | `code.processed`, `docs.processed` |
| relation-graph | `graph.updated`, `graph.build.completed` | `embedding.generated`, `graph.build.requested` |
| data-vent | - | - (direct FalkorDB queries) |
| (any service on error) | `dlq.processing.failed` | - |

## Confluent Cloud CLI Commands

```bash
# Create all topics with default settings
confluent kafka topic create code.ingested --partitions 6 --config retention.ms=604800000
confluent kafka topic create code.processed --partitions 6 --config retention.ms=604800000
confluent kafka topic create docs.ingested --partitions 6 --config retention.ms=604800000
confluent kafka topic create docs.processed --partitions 6 --config retention.ms=604800000
confluent kafka topic create embedding.generated --partitions 6 --config retention.ms=604800000
confluent kafka topic create graph.updated --partitions 3 --config retention.ms=604800000
confluent kafka topic create graph.build.requested --partitions 3 --config retention.ms=604800000
confluent kafka topic create graph.build.completed --partitions 3 --config retention.ms=604800000
confluent kafka topic create source.sync.requested --partitions 3 --config retention.ms=604800000
confluent kafka topic create source.sync.completed --partitions 3 --config retention.ms=604800000
confluent kafka topic create source.sync.failed --partitions 3 --config retention.ms=604800000
confluent kafka topic create dlq.processing.failed --partitions 3 --config retention.ms=2592000000
```

## Notes

1. **Partition Count**: Higher partitions (6) for high-throughput topics (ingestion, processing), lower (3) for control plane topics
2. **Retention**: 7 days for normal topics, 30 days for DLQ to allow debugging
3. **Replication**: Confluent Cloud handles replication automatically (typically 3x)
4. **Architecture Change**: data-connector now communicates with unified-processor via direct gRPC calls instead of Kafka. data-vent queries FalkorDB directly instead of consuming `chunks.stored` notifications.
