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

use grass_core::{GrassTaskPayload, GrassTaskStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum GrassTaskType {
    Sequential,
    Parallel,
    Conditional,
    Loop,
}

pub struct GrassTaskGraph {
    nodes: Arc<RwLock<HashMap<String, GrassTaskNode>>>,
    edges: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrassTaskNode {
    pub id: String,
    pub task: GrassTaskPayload,
    pub task_type: GrassTaskType,
    pub status: GrassTaskStatus,
}

impl GrassTaskGraph {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            edges: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_node(&self, node: GrassTaskNode) {
        let mut nodes = self.nodes.write().await;
        nodes.insert(node.id.clone(), node);
    }

    pub async fn add_edge(&self, from: &str, to: &str) {
        let mut edges = self.edges.write().await;
        edges
            .entry(to.to_string())
            .or_default()
            .push(from.to_string());
    }

    pub async fn get_node(&self, id: &str) -> Option<GrassTaskNode> {
        self.nodes.read().await.get(id).cloned()
    }

    pub async fn dependencies(&self, id: &str) -> Vec<String> {
        self.edges.read().await.get(id).cloned().unwrap_or_default()
    }

    pub async fn ready_nodes(&self) -> Vec<GrassTaskNode> {
        let nodes = self.nodes.read().await;
        let edges = self.edges.read().await;
        nodes
            .values()
            .filter(|n| n.status == GrassTaskStatus::Pending)
            .filter(|n| !edges.contains_key(&n.id))
            .cloned()
            .collect()
    }
}

impl Default for GrassTaskGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn make_node(id: &str) -> GrassTaskNode {
        GrassTaskNode {
            id: id.to_string(),
            task: GrassTaskPayload {
                task_id: Uuid::new_v4(),
                agent_id: Uuid::new_v4(),
                action: "test".into(),
                input: serde_json::json!({}),
                metadata: HashMap::new(),
            },
            task_type: GrassTaskType::Sequential,
            status: GrassTaskStatus::Pending,
        }
    }

    #[tokio::test]
    async fn test_task_graph() {
        let graph = GrassTaskGraph::new();
        graph.add_node(make_node("a")).await;
        graph.add_node(make_node("b")).await;
        graph.add_edge("a", "b").await;

        let deps = graph.dependencies("b").await;
        assert_eq!(deps, vec!["a".to_string()]);
    }

    #[tokio::test]
    async fn test_ready_nodes_excludes_dependent_nodes() {
        let graph = GrassTaskGraph::new();
        graph.add_node(make_node("a")).await;
        graph.add_node(make_node("b")).await;
        graph.add_edge("a", "b").await;

        let ready = graph.ready_nodes().await;
        let ids: Vec<String> = ready.iter().map(|n| n.id.clone()).collect();
        assert_eq!(ids, vec!["a".to_string()]);
    }

    #[tokio::test]
    async fn test_ready_nodes_ignores_completed_node() {
        let graph = GrassTaskGraph::new();
        graph.add_node(make_node("a")).await;
        let mut done = make_node("b");
        done.status = GrassTaskStatus::Completed;
        graph.add_node(done).await;

        let ready = graph.ready_nodes().await;
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "a");
    }

    #[tokio::test]
    async fn test_dependencies_of_unknown_node_returns_empty() {
        let graph = GrassTaskGraph::new();
        graph.add_node(make_node("a")).await;
        let deps = graph.dependencies("missing").await;
        assert!(deps.is_empty());
    }

    #[tokio::test]
    async fn test_task_graph_cycle_marked_by_unreachable_ready() {
        let graph = GrassTaskGraph::new();
        graph.add_node(make_node("a")).await;
        graph.add_node(make_node("b")).await;
        graph.add_edge("a", "b").await;
        graph.add_edge("b", "a").await;

        assert_eq!(
            graph.ready_nodes().await.len(),
            0,
            "cycle leaves no ready node"
        );
    }
}
