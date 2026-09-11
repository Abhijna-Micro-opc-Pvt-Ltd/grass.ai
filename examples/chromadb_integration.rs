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

use grass_vector::{GrassChromaDb, GrassQueryResult, GrassVectorRecord, GrassVectorStore};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let store = GrassChromaDb::new("http://localhost:8000");

    tracing::info!("=== ChromaDB Vector Store Demo ===");

    store.create_collection("agent_memory").await?;

    let documents = vec![
        GrassVectorRecord {
            id: "doc-001".into(),
            embedding: vec![0.1, 0.2, 0.3, 0.4, 0.5],
            document: "Rust is a systems programming language focused on safety and performance"
                .into(),
            metadata: HashMap::from([
                ("source".into(), serde_json::json!("documentation")),
                ("topic".into(), serde_json::json!("programming")),
            ]),
        },
        GrassVectorRecord {
            id: "doc-002".into(),
            embedding: vec![0.2, 0.3, 0.4, 0.5, 0.6],
            document: "gRPC is a high-performance RPC framework using Protocol Buffers".into(),
            metadata: HashMap::from([
                ("source".into(), serde_json::json!("documentation")),
                ("topic".into(), serde_json::json!("networking")),
            ]),
        },
        GrassVectorRecord {
            id: "doc-003".into(),
            embedding: vec![0.3, 0.4, 0.5, 0.6, 0.7],
            document: "Vector databases store embeddings for similarity search".into(),
            metadata: HashMap::from([
                ("source".into(), serde_json::json!("research")),
                ("topic".into(), serde_json::json!("database")),
            ]),
        },
        GrassVectorRecord {
            id: "doc-004".into(),
            embedding: vec![0.4, 0.5, 0.6, 0.7, 0.8],
            document: "Docker containers provide isolated environments for running applications"
                .into(),
            metadata: HashMap::from([
                ("source".into(), serde_json::json!("documentation")),
                ("topic".into(), serde_json::json!("devops")),
            ]),
        },
        GrassVectorRecord {
            id: "doc-005".into(),
            embedding: vec![0.5, 0.6, 0.7, 0.8, 0.9],
            document: "WebRTC enables real-time communication between browsers".into(),
            metadata: HashMap::from([
                ("source".into(), serde_json::json!("documentation")),
                ("topic".into(), serde_json::json!("networking")),
            ]),
        },
    ];

    for doc in &documents {
        store.insert("agent_memory", doc.clone()).await?;
        tracing::info!("Inserted: {} - {}", doc.id, &doc.document[..50]);
    }

    let count = store.count("agent_memory").await?;
    tracing::info!("Total documents: {}", count);

    let query_embedding = vec![0.2, 0.3, 0.4, 0.5, 0.6];
    let results = store.query("agent_memory", &query_embedding, 3).await?;
    tracing::info!("Query results (top 3):");
    for (i, (id, doc)) in results.ids.iter().zip(results.documents.iter()).enumerate() {
        tracing::info!("  {}. {} - {}", i + 1, id, doc);
    }

    store.delete("agent_memory", "doc-001").await?;
    tracing::info!("Deleted doc-001");

    let count_after = store.count("agent_memory").await?;
    tracing::info!("Documents after delete: {}", count_after);

    let query2 = store
        .query("agent_memory", &[0.5, 0.6, 0.7, 0.8, 0.9], 2)
        .await?;
    tracing::info!("Query 2 results:");
    for (id, doc) in query2.ids.iter().zip(query2.documents.iter()) {
        tracing::info!("  {} - {}", id, doc);
    }

    tracing::info!("=== ChromaDB Demo Complete ===");
    Ok(())
}
