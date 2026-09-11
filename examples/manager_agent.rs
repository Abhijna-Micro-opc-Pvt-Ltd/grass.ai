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
use grass_core::{
    AgentId, GrassAgent, GrassAgentBuilder, GrassAgentMetadata, GrassAgentState, GrassError,
    GrassMessage, GrassTaskPayload, GrassTaskResult, GrassTaskStatus, Result,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct GrassManagerAgent {
    metadata: GrassAgentMetadata,
    state: GrassAgentState,
    workers: Arc<RwLock<HashMap<AgentId, GrassWorkerInfo>>>,
    task_queue: Arc<RwLock<Vec<GrassManagedTask>>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrassWorkerInfo {
    pub agent_id: AgentId,
    pub name: String,
    pub capabilities: Vec<String>,
    pub current_task: Option<String>,
    pub status: WorkerStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum WorkerStatus {
    Available,
    Busy,
    Offline,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrassManagedTask {
    pub task_id: String,
    pub assigned_to: Option<AgentId>,
    pub payload: serde_json::Value,
    pub status: GrassTaskStatus,
    pub priority: u8,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrassManagerRequest {
    pub action: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrassManagerResponse {
    pub status: String,
    pub data: serde_json::Value,
}

impl GrassManagerAgent {
    pub fn new() -> Self {
        let metadata = GrassAgentBuilder::new("manager-agent")
            .version("0.1.0")
            .description("Manages worker agents and distributes tasks")
            .tag("manager")
            .tag("orchestrator")
            .build();

        Self {
            metadata,
            state: GrassAgentState::Idle,
            workers: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn register_worker(&self, worker: GrassWorkerInfo) {
        let mut workers = self.workers.write().await;
        tracing::info!("Worker registered: {} ({})", worker.name, worker.agent_id);
        workers.insert(worker.agent_id, worker);
    }

    pub async fn unregister_worker(&self, agent_id: AgentId) {
        let mut workers = self.workers.write().await;
        workers.remove(&agent_id);
    }

    pub async fn submit_task(
        &self,
        task_id: &str,
        payload: serde_json::Value,
        priority: u8,
    ) -> Result<()> {
        let mut queue = self.task_queue.write().await;
        let task = GrassManagedTask {
            task_id: task_id.to_string(),
            assigned_to: None,
            payload,
            status: GrassTaskStatus::Pending,
            priority,
        };
        queue.push(task);
        queue.sort_by(|a, b| b.priority.cmp(&a.priority));
        tracing::info!("Task {} submitted with priority {}", task_id, priority);
        Ok(())
    }

    pub async fn assign_next_task(&self) -> Result<Option<GrassManagedTask>> {
        let workers = self.workers.read().await;
        let available = workers
            .values()
            .filter(|w| w.status == WorkerStatus::Available)
            .next();

        let worker_id = match available {
            Some(w) => w.agent_id,
            None => return Ok(None),
        };
        drop(workers);

        let mut queue = self.task_queue.write().await;
        let task = queue
            .iter_mut()
            .find(|t| t.status == GrassTaskStatus::Pending);

        if let Some(task) = task {
            task.assigned_to = Some(worker_id);
            task.status = GrassTaskStatus::Running;
            tracing::info!("Task {} assigned to worker {}", task.task_id, worker_id);
            return Ok(Some(task.clone()));
        }

        Ok(None)
    }

    pub async fn complete_task(
        &self,
        task_id: &str,
        status: GrassTaskStatus,
        output: serde_json::Value,
    ) -> Result<()> {
        let mut queue = self.task_queue.write().await;
        if let Some(task) = queue.iter_mut().find(|t| t.task_id == task_id) {
            task.status = status.clone();
            tracing::info!("Task {} completed: {:?}", task_id, status);
        }
        Ok(())
    }

    pub async fn get_status(&self) -> GrassManagerResponse {
        let workers = self.workers.read().await;
        let queue = self.task_queue.read().await;
        let available = workers
            .values()
            .filter(|w| w.status == WorkerStatus::Available)
            .count();
        let pending = queue
            .iter()
            .filter(|t| t.status == GrassTaskStatus::Pending)
            .count();
        let running = queue
            .iter()
            .filter(|t| t.status == GrassTaskStatus::Running)
            .count();

        GrassManagerResponse {
            status: "ok".into(),
            data: serde_json::json!({
                "workers": {
                    "total": workers.len(),
                    "available": available,
                },
                "tasks": {
                    "pending": pending,
                    "running": running,
                    "completed": queue.iter().filter(|t| t.status == GrassTaskStatus::Completed).count(),
                }
            }),
        }
    }

    pub async fn handle_request(
        &self,
        request: &GrassManagerRequest,
    ) -> Result<GrassManagerResponse> {
        match request.action.as_str() {
            "submit_task" => {
                let task_id = request.payload["task_id"].as_str().unwrap_or("unknown");
                let priority = request.payload["priority"].as_u64().unwrap_or(5) as u8;
                self.submit_task(task_id, request.payload.clone(), priority)
                    .await?;
                Ok(GrassManagerResponse {
                    status: "submitted".into(),
                    data: serde_json::json!({"task_id": task_id}),
                })
            }
            "assign" => {
                let task = self.assign_next_task().await?;
                Ok(GrassManagerResponse {
                    status: "ok".into(),
                    data: serde_json::to_value(&task).unwrap_or_default(),
                })
            }
            "status" => Ok(self.get_status().await),
            _ => Err(GrassError::Agent(format!(
                "unknown action: {}",
                request.action
            ))),
        }
    }
}

#[async_trait]
impl GrassAgent for GrassManagerAgent {
    fn metadata(&self) -> &GrassAgentMetadata {
        &self.metadata
    }
    fn state(&self) -> GrassAgentState {
        self.state.clone()
    }
    async fn start(&mut self) -> Result<()> {
        self.state = GrassAgentState::Idle;
        tracing::info!("Manager agent started");
        Ok(())
    }
    async fn stop(&mut self) -> Result<()> {
        self.state = GrassAgentState::Idle;
        tracing::info!("Manager agent stopped");
        Ok(())
    }

    async fn execute(&self, task: GrassTaskPayload) -> Result<GrassTaskResult> {
        let start = std::time::Instant::now();
        let request: GrassManagerRequest = serde_json::from_value(task.input)
            .map_err(|e| GrassError::Task(format!("invalid request: {}", e)))?;
        let response = self.handle_request(&request).await?;
        Ok(GrassTaskResult {
            task_id: task.task_id,
            agent_id: task.agent_id,
            status: GrassTaskStatus::Completed,
            output: serde_json::to_value(&response).unwrap_or_default(),
            error: None,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    async fn handle_message(&self, msg: GrassMessage) -> Result<Option<GrassMessage>> {
        match msg.topic.as_str() {
            "worker.register" => {
                let worker: GrassWorkerInfo = serde_json::from_value(msg.payload)?;
                self.register_worker(worker).await;
                Ok(Some(
                    GrassMessage::new(
                        self.metadata.id,
                        "worker.registered",
                        serde_json::json!({"status": "ok"}),
                    )
                    .directed_to(msg.from),
                ))
            }
            "task.complete" => {
                let task_id = msg.payload["task_id"].as_str().unwrap_or("");
                let status =
                    serde_json::from_value(msg.payload.get("status").cloned().unwrap_or_default())
                        .unwrap_or(GrassTaskStatus::Completed);
                let output = msg.payload.get("output").cloned().unwrap_or_default();
                self.complete_task(task_id, status, output).await?;
                Ok(None)
            }
            "manager.status" => {
                let status = self.get_status().await;
                Ok(Some(
                    GrassMessage::new(
                        self.metadata.id,
                        "manager.status.response",
                        serde_json::to_value(&status).unwrap_or_default(),
                    )
                    .directed_to(msg.from),
                ))
            }
            _ => Ok(None),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut manager = GrassManagerAgent::new();
    manager.start().await?;

    manager
        .register_worker(GrassWorkerInfo {
            agent_id: uuid::Uuid::new_v4(),
            name: "worker-1".into(),
            capabilities: vec!["text-analysis".into(), "summarization".into()],
            current_task: None,
            status: WorkerStatus::Available,
        })
        .await;

    manager
        .register_worker(GrassWorkerInfo {
            agent_id: uuid::Uuid::new_v4(),
            name: "worker-2".into(),
            capabilities: vec!["translation".into(), "sentiment".into()],
            current_task: None,
            status: WorkerStatus::Available,
        })
        .await;

    manager
        .submit_task(
            "task-001",
            serde_json::json!({"action": "analyze", "text": "hello"}),
            8,
        )
        .await?;
    manager
        .submit_task(
            "task-002",
            serde_json::json!({"action": "translate", "text": "hola"}),
            5,
        )
        .await?;

    let response = manager
        .handle_request(&GrassManagerRequest {
            action: "assign".into(),
            payload: serde_json::json!({}),
        })
        .await?;
    println!(
        "Assign: {}",
        serde_json::to_string_pretty(&response).unwrap()
    );

    let status = manager.get_status().await;
    println!("Status: {}", serde_json::to_string_pretty(&status).unwrap());

    manager.stop().await?;
    Ok(())
}
