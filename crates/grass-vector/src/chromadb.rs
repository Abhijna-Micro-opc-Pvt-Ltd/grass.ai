// MIT License
//
// Copyright (c) 2026 Abhijna Micro (OPC) PVT LTD
// Author: Abhijna Micro Team <team@abhijnamicro.in>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use async_trait::async_trait;
use grass_core::{GrassError, Result};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrassVectorRecord {
    pub id: String,
    pub embedding: Vec<f32>,
    pub document: String,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrassQueryResult {
    pub ids: Vec<String>,
    pub documents: Vec<String>,
    pub distances: Vec<f32>,
    pub metadatas: Vec<std::collections::HashMap<String, serde_json::Value>>,
}

#[async_trait]
pub trait GrassVectorStore: Send + Sync {
    fn name(&self) -> &str;
    async fn create_collection(&self, name: &str) -> Result<()>;
    async fn insert(&self, collection: &str, record: GrassVectorRecord) -> Result<()>;
    async fn query(
        &self,
        collection: &str,
        embedding: &[f32],
        n: usize,
    ) -> Result<GrassQueryResult>;
    async fn delete(&self, collection: &str, id: &str) -> Result<()>;
    async fn count(&self, collection: &str) -> Result<usize>;
}

pub struct GrassChromaDb {
    base_url: String,
    http: reqwest::Client,
}

impl GrassChromaDb {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            http: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl GrassVectorStore for GrassChromaDb {
    fn name(&self) -> &str {
        "chromadb"
    }

    async fn create_collection(&self, name: &str) -> Result<()> {
        self.http
            .post(format!("{}/api/v1/collections", self.base_url))
            .json(&serde_json::json!({"name": name}))
            .send()
            .await
            .map_err(|e| GrassError::VectorDb(e.to_string()))?;
        tracing::info!("ChromaDB collection created: {}", name);
        Ok(())
    }

    async fn insert(&self, collection: &str, record: GrassVectorRecord) -> Result<()> {
        self.http
            .post(format!(
                "{}/api/v1/collections/{}/add",
                self.base_url, collection
            ))
            .json(&serde_json::json!({
                "ids": [record.id],
                "embeddings": [record.embedding],
                "documents": [record.document],
                "metadatas": [record.metadata]
            }))
            .send()
            .await
            .map_err(|e| GrassError::VectorDb(e.to_string()))?;
        Ok(())
    }

    async fn query(
        &self,
        collection: &str,
        embedding: &[f32],
        n: usize,
    ) -> Result<GrassQueryResult> {
        let resp: serde_json::Value = self
            .http
            .post(format!(
                "{}/api/v1/collections/{}/query",
                self.base_url, collection
            ))
            .json(&serde_json::json!({"query_embeddings": [embedding], "n_results": n}))
            .send()
            .await
            .map_err(|e| GrassError::VectorDb(e.to_string()))?
            .json()
            .await
            .map_err(|e| GrassError::VectorDb(e.to_string()))?;

        Ok(GrassQueryResult {
            ids: resp["ids"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            documents: resp["documents"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            distances: resp["distances"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_f64().map(|f| f as f32))
                        .collect()
                })
                .unwrap_or_default(),
            metadatas: vec![],
        })
    }

    async fn delete(&self, collection: &str, id: &str) -> Result<()> {
        self.http
            .post(format!(
                "{}/api/v1/collections/{}/delete",
                self.base_url, collection
            ))
            .json(&serde_json::json!({"ids": [id]}))
            .send()
            .await
            .map_err(|e| GrassError::VectorDb(e.to_string()))?;
        Ok(())
    }

    async fn count(&self, collection: &str) -> Result<usize> {
        let resp: serde_json::Value = self
            .http
            .get(format!(
                "{}/api/v1/collections/{}/count",
                self.base_url, collection
            ))
            .send()
            .await
            .map_err(|e| GrassError::VectorDb(e.to_string()))?
            .json()
            .await
            .map_err(|e| GrassError::VectorDb(e.to_string()))?;
        Ok(resp.as_u64().unwrap_or(0) as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_record_creation() {
        let record = GrassVectorRecord {
            id: "doc1".into(),
            embedding: vec![0.1, 0.2, 0.3],
            document: "test document".into(),
            metadata: std::collections::HashMap::new(),
        };
        assert_eq!(record.id, "doc1");
        assert_eq!(record.embedding.len(), 3);
    }
}
