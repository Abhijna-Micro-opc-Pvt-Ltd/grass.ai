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

use grass_core::{GrassError, GrassTaskPayload, GrassTaskResult, Result, TaskId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};

pub struct GrassTaskExecutor {
    workers: u32,
    running: Arc<Mutex<HashMap<TaskId, oneshot::Sender<()>>>>,
}

impl GrassTaskExecutor {
    pub fn new(workers: u32) -> Self {
        Self {
            workers,
            running: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn submit<F, Fut>(&self, task: GrassTaskPayload, handler: F) -> Result<()>
    where
        F: FnOnce(GrassTaskPayload) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<GrassTaskResult>> + Send + 'static,
    {
        let task_id = task.task_id;
        let (tx, rx) = oneshot::channel();

        {
            let mut running = self.running.lock().await;
            running.insert(task_id, tx);
        }

        let running = self.running.clone();
        tokio::spawn(async move {
            let result = tokio::select! {
                res = handler(task) => res,
                _ = rx => return,
            };
            match result {
                Ok(r) => tracing::info!("task {} completed: {:?}", task_id, r.status),
                Err(e) => tracing::error!("task {} failed: {}", task_id, e),
            }
            let mut running = running.lock().await;
            running.remove(&task_id);
        });

        Ok(())
    }

    pub async fn cancel(&self, task_id: TaskId) -> Result<()> {
        let mut running = self.running.lock().await;
        if let Some(tx) = running.remove(&task_id) {
            let _ = tx.send(());
            Ok(())
        } else {
            Err(GrassError::NotFound(format!(
                "task {} not running",
                task_id
            )))
        }
    }

    pub async fn running_count(&self) -> usize {
        self.running.lock().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use grass_core::GrassTaskStatus;

    #[tokio::test]
    async fn test_executor_submit_and_count() {
        let exec = GrassTaskExecutor::new(4);
        let payload = GrassTaskPayload {
            task_id: uuid::Uuid::new_v4(),
            agent_id: uuid::Uuid::new_v4(),
            action: "test".into(),
            input: serde_json::json!({}),
            metadata: HashMap::new(),
        };
        exec.submit(payload, |t| async move {
            Ok(GrassTaskResult {
                task_id: t.task_id,
                agent_id: t.agent_id,
                status: GrassTaskStatus::Completed,
                output: serde_json::json!({}),
                error: None,
                duration_ms: 10,
            })
        })
        .await
        .unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert_eq!(exec.running_count().await, 0);
    }
}
