//! Memgraph client — Bolt-protocol connectivity for ConFuse platform.
//!
//! Replaces the former FalkorDB/Redis-based graph client.
//! Memgraph speaks the standard Bolt wire protocol on port 7687,
//! so we connect with `neo4rs` rather than the Redis crate.

use neo4rs::{Graph, ConfigBuilder, Query};
use std::sync::Arc;

/// A shared, cloneable handle to a Memgraph database connection pool.
#[derive(Clone)]
pub struct MemgraphClient {
    graph: Arc<Graph>,
    /// Default database/graph name (informational — Memgraph Community does not
    /// use named databases, but we keep the field for config parity).
    pub graph_name: String,
}

impl MemgraphClient {
    /// Connect to Memgraph over Bolt.
    ///
    /// # Arguments
    /// * `host` — hostname or IP, e.g. `"localhost"`
    /// * `port` — Bolt port, typically `7687`
    /// * `graph_name` — logical name used in logs / index creation
    /// * `username` — optional username (`""` for open instances)
    /// * `password` — optional password (`""` for open instances)
    pub async fn new(
        host: &str,
        port: u16,
        graph_name: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, neo4rs::Error> {
        let uri = format!("{}:{}", host, port);

        let config = ConfigBuilder::default()
            .uri(uri)
            .user(username)
            .password(password)
            .db("memgraph")
            .fetch_size(500)
            .max_connections(16)
            .build()?;

        let graph = Graph::connect(config).await?;

        Ok(Self {
            graph: Arc::new(graph),
            graph_name: graph_name.to_string(),
        })
    }

    /// Execute a write Cypher query; parameters are baked into the `Query`.
    pub async fn execute(&self, query: Query) -> Result<(), neo4rs::Error> {
        self.graph.run(query).await
    }

    /// Execute a read Cypher query and return a row-stream.
    pub async fn query(&self, query: Query) -> Result<neo4rs::RowStream, neo4rs::Error> {
        self.graph.execute(query).await
    }

    /// Convenience: execute a raw Cypher string with no parameters.
    pub async fn run_raw(&self, cypher: &str) -> Result<(), neo4rs::Error> {
        self.graph.run(neo4rs::query(cypher)).await
    }

    /// Convenience: query a raw Cypher string with no parameters.
    pub async fn query_raw(&self, cypher: &str) -> Result<neo4rs::RowStream, neo4rs::Error> {
        self.graph.execute(neo4rs::query(cypher)).await
    }

    /// Expose the underlying `neo4rs::Graph` handle for advanced use.
    pub fn inner(&self) -> &Arc<Graph> {
        &self.graph
    }
}
