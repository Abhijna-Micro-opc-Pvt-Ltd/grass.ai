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

use grass_core::{AgentId, GrassAgentMetadata, GrassError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct GrassCrewManager {
    agents: Arc<RwLock<HashMap<AgentId, GrassAgentMetadata>>>,
    assignments: Arc<RwLock<HashMap<String, Vec<AgentId>>>>,
}

impl GrassCrewManager {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            assignments: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_agent(&self, agent: GrassAgentMetadata) {
        let mut agents = self.agents.write().await;
        agents.insert(agent.id, agent);
    }

    pub async fn remove_agent(&self, id: AgentId) {
        let mut agents = self.agents.write().await;
        agents.remove(&id);
    }

    pub async fn assign_task(&self, task_name: &str, agent_id: AgentId) -> Result<()> {
        let agents = self.agents.read().await;
        if !agents.contains_key(&agent_id) {
            return Err(GrassError::Agent(format!("agent {} not found", agent_id)));
        }
        drop(agents);
        let mut assignments = self.assignments.write().await;
        assignments
            .entry(task_name.to_string())
            .or_default()
            .push(agent_id);
        Ok(())
    }

    pub async fn agents_for_task(&self, task_name: &str) -> Vec<AgentId> {
        self.assignments
            .read()
            .await
            .get(task_name)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn list_agents(&self) -> Vec<GrassAgentMetadata> {
        self.agents.read().await.values().cloned().collect()
    }
}

impl Default for GrassCrewManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_crew_manager() {
        let mgr = GrassCrewManager::new();
        let id = Uuid::new_v4();
        let meta = grass_core::GrassAgentBuilder::new("worker").build();
        let mut meta = meta;
        meta.id = id;
        mgr.add_agent(meta).await;
        mgr.assign_task("summarize", id).await.unwrap();
        assert_eq!(mgr.agents_for_task("summarize").await, vec![id]);
    }
}
