use redis::{Client, Connection, RedisResult};

#[derive(Clone)]
pub struct FalkorClient {
    client: Client,
    graph_name: String,
}

impl FalkorClient {
    pub fn new(url: &str, graph_name: &str) -> RedisResult<Self> {
        let client = Client::open(url)?;
        Ok(Self {
            client,
            graph_name: graph_name.to_string(),
        })
    }

    pub async fn get_connection(&self) -> RedisResult<Connection> {
        self.client.get_connection()
    }

    pub async fn query(&self, query: &str) -> RedisResult<redis::Value> {
        let mut con = self.client.get_async_connection().await?;
        let result: redis::Value = redis::cmd("GRAPH.QUERY")
            .arg(&self.graph_name)
            .arg(query)
            .query_async(&mut con)
            .await?;
        Ok(result)
    }

    pub async fn ro_query(&self, query: &str) -> RedisResult<redis::Value> {
        let mut con = self.client.get_async_connection().await?;
        let result: redis::Value = redis::cmd("GRAPH.RO_QUERY")
            .arg(&self.graph_name)
            .arg(query)
            .query_async(&mut con)
            .await?;
        Ok(result)
    }

    // Vector search helper (using RediSearch/FalkorDB vector index syntax)
    // Assuming vector index is created on nodes
    pub async fn vector_search(&self, node_label: &str, vector: &[f32], limit: usize) -> RedisResult<redis::Value> {
        // This query syntax depends on FalkorDB vector support.
        // Usually: CALL db.idx.vector.queryNodes('Person', 'embedding', $vec, $limit, 'EUCLIDEAN')
        // Using parameterized query if supported via Redis command, or building string.
        // FalkorDB Cypher:
        // CALL db.idx.vector.queryNodes('NodeLabel', 'AttributeName', [...], 5, 'COSINE')
        
        let vec_str = format!("[{}]", vector.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(","));
        let query = format!(
            "CALL db.idx.vector.queryNodes('{}', 'embedding', {}, {}, 'COSINE') YIELD node, score RETURN node, score",
            node_label, vec_str, limit
        );
        
        self.ro_query(&query).await
    }
}
