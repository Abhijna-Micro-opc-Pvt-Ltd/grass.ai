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

use grass_core::{GrassTaskPayload, GrassTaskResult, GrassTaskStatus, TaskId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct GrassTaskManager {
    tasks: Arc<RwLock<HashMap<TaskId, GrassTaskResult>>>,
    queues: Arc<RwLock<HashMap<String, Vec<GrassTaskPayload>>>>,
}

impl GrassTaskManager {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            queues: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_task(&self, task: GrassTaskPayload) {
        let mut queues = self.queues.write().await;
        queues.entry(task.action.clone()).or_default().push(task);
    }

    pub async fn complete_task(&self, result: GrassTaskResult) {
        let mut tasks = self.tasks.write().await;
        tasks.insert(result.task_id, result);
    }

    pub async fn get_result(&self, task_id: TaskId) -> Option<GrassTaskResult> {
        self.tasks.read().await.get(&task_id).cloned()
    }

    pub async fn pending_count(&self) -> usize {
        self.queues.read().await.values().map(|q| q.len()).sum()
    }

    pub async fn completed_count(&self) -> usize {
        self.tasks.read().await.len()
    }
}

impl Default for GrassTaskManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_task_manager() {
        let mgr = GrassTaskManager::new();
        let task_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();
        let payload = GrassTaskPayload {
            task_id,
            agent_id,
            action: "process".into(),
            input: serde_json::json!({}),
            metadata: HashMap::new(),
        };
        mgr.create_task(payload).await;
        assert_eq!(mgr.pending_count().await, 1);

        let result = GrassTaskResult {
            task_id,
            agent_id,
            status: GrassTaskStatus::Completed,
            output: serde_json::json!({"done": true}),
            error: None,
            duration_ms: 100,
        };
        mgr.complete_task(result).await;
        assert_eq!(mgr.completed_count().await, 1);
    }
}
