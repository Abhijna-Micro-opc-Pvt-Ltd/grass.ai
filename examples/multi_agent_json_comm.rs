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
    GrassEvent, GrassEventBus, GrassMessage, GrassTaskPayload, GrassTaskResult, GrassTaskStatus,
    Result,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AgentMessage {
    TaskRequest {
        task_id: String,
        action: String,
        payload: serde_json::Value,
    },
    TaskResponse {
        task_id: String,
        status: String,
        result: serde_json::Value,
    },
    Query {
        query_id: String,
        question: String,
        context: serde_json::Value,
    },
    Answer {
        query_id: String,
        answer: String,
        confidence: f64,
    },
    Broadcast {
        sender: String,
        event: String,
        data: serde_json::Value,
    },
    Heartbeat {
        agent_name: String,
        status: String,
    },
}

pub struct AgentNetwork {
    senders: Arc<RwLock<HashMap<String, mpsc::Sender<AgentMessage>>>>,
    event_bus: GrassEventBus,
}

impl AgentNetwork {
    pub fn new() -> Self {
        Self {
            senders: Arc::new(RwLock::new(HashMap::new())),
            event_bus: GrassEventBus::new(),
        }
    }

    pub async fn register_agent(&self, name: &str) -> mpsc::Receiver<AgentMessage> {
        let (tx, rx) = mpsc::channel(100);
        let mut senders = self.senders.write().await;
        senders.insert(name.to_string(), tx);
        tracing::info!("Agent '{}' registered in network", name);
        rx
    }

    pub async fn send(&self, to: &str, msg: AgentMessage) -> Result<()> {
        let senders = self.senders.read().await;
        let sender = senders
            .get(to)
            .ok_or_else(|| GrassError::Agent(format!("agent '{}' not found", to)))?;
        sender
            .send(msg)
            .await
            .map_err(|e| GrassError::Agent(format!("send failed: {}", e)))?;
        Ok(())
    }

    pub async fn broadcast(&self, msg: AgentMessage) {
        let senders = self.senders.read().await;
        for (name, sender) in senders.iter() {
            let _ = sender.send(msg.clone()).await;
            tracing::debug!("Broadcast to '{}'", name);
        }
    }
}

impl Default for AgentNetwork {
    fn default() -> Self {
        Self::new()
    }
}

struct AnalyzerAgent {
    metadata: GrassAgentMetadata,
    network: Arc<AgentNetwork>,
}

impl AnalyzerAgent {
    fn new(network: Arc<AgentNetwork>) -> Self {
        Self {
            metadata: GrassAgentBuilder::new("analyzer")
                .description("Analyzes text and extracts insights")
                .tag("analyzer")
                .build(),
            network,
        }
    }
}

#[async_trait]
impl GrassAgent for AnalyzerAgent {
    fn metadata(&self) -> &GrassAgentMetadata {
        &self.metadata
    }
    fn state(&self) -> GrassAgentState {
        GrassAgentState::Idle
    }

    async fn start(&mut self) -> Result<()> {
        Ok(())
    }
    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    async fn execute(&self, task: GrassTaskPayload) -> Result<GrassTaskResult> {
        let text = task
            .input
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let word_count = text.split_whitespace().count();
        let result = serde_json::json!({
            "word_count": word_count,
            "char_count": text.len(),
            "language": "en",
            "sentiment": if text.contains("great") { "positive" } else if text.contains("bad") { "negative" } else { "neutral" }
        });

        Ok(GrassTaskResult {
            task_id: task.task_id,
            agent_id: task.agent_id,
            status: GrassTaskStatus::Completed,
            output: result,
            error: None,
            duration_ms: 10,
        })
    }

    async fn handle_message(&self, msg: GrassMessage) -> Result<Option<GrassMessage>> {
        match msg.topic.as_str() {
            "task.analyze" => {
                let text = msg.payload["text"].as_str().unwrap_or("");
                let result = serde_json::json!({
                    "word_count": text.split_whitespace().count(),
                    "analysis": "complete"
                });
                Ok(Some(
                    GrassMessage::new(self.metadata.id, "task.analyze.result", result)
                        .directed_to(msg.from),
                ))
            }
            _ => Ok(None),
        }
    }
}

struct SummarizerAgent {
    metadata: GrassAgentMetadata,
}

impl SummarizerAgent {
    fn new() -> Self {
        Self {
            metadata: GrassAgentBuilder::new("summarizer")
                .description("Summarizes text content")
                .tag("summarizer")
                .build(),
        }
    }
}

#[async_trait]
impl GrassAgent for SummarizerAgent {
    fn metadata(&self) -> &GrassAgentMetadata {
        &self.metadata
    }
    fn state(&self) -> GrassAgentState {
        GrassAgentState::Idle
    }

    async fn start(&mut self) -> Result<()> {
        Ok(())
    }
    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    async fn execute(&self, task: GrassTaskPayload) -> Result<GrassTaskResult> {
        let text = task
            .input
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let summary = if text.len() > 100 {
            format!("{}...", &text[..100])
        } else {
            text.to_string()
        };

        Ok(GrassTaskResult {
            task_id: task.task_id,
            agent_id: task.agent_id,
            status: GrassTaskStatus::Completed,
            output: serde_json::json!({"summary": summary, "original_length": text.len()}),
            error: None,
            duration_ms: 5,
        })
    }

    async fn handle_message(&self, msg: GrassMessage) -> Result<Option<GrassMessage>> {
        match msg.topic.as_str() {
            "task.summarize" => {
                let text = msg.payload["text"].as_str().unwrap_or("");
                let summary = if text.len() > 100 {
                    format!("{}...", &text[..100])
                } else {
                    text.to_string()
                };
                Ok(Some(
                    GrassMessage::new(
                        self.metadata.id,
                        "task.summarize.result",
                        serde_json::json!({"summary": summary}),
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

    let network = Arc::new(AgentNetwork::new());

    let mut analyzer_rx = network.register_agent("analyzer").await;
    let mut summarizer_rx = network.register_agent("summarizer").await;

    let analyzer = AnalyzerAgent::new(network.clone());
    let summarizer = SummarizerAgent::new();

    let analyzer_id = analyzer.metadata().id;
    let summarizer_id = summarizer.metadata().id;

    tracing::info!("=== Multi-Agent JSON Communication Demo ===");
    tracing::info!("Analyzer ID: {}", analyzer_id);
    tracing::info!("Summarizer ID: {}", summarizer_id);

    let text = "This is a great product review. The quality is excellent and shipping was fast. I would recommend it to others.";

    let analysis_result = analyzer
        .execute(GrassTaskPayload {
            task_id: Uuid::new_v4(),
            agent_id: analyzer_id,
            action: "analyze".into(),
            input: serde_json::json!({"text": text}),
            metadata: HashMap::new(),
        })
        .await?;
    tracing::info!(
        "Analysis: {}",
        serde_json::to_string_pretty(&analysis_result.output).unwrap()
    );

    let summary_result = summarizer
        .execute(GrassTaskPayload {
            task_id: Uuid::new_v4(),
            agent_id: summarizer_id,
            action: "summarize".into(),
            input: serde_json::json!({"text": text}),
            metadata: HashMap::new(),
        })
        .await?;
    tracing::info!(
        "Summary: {}",
        serde_json::to_string_pretty(&summary_result.output).unwrap()
    );

    let msg = AgentMessage::TaskRequest {
        task_id: "task-001".into(),
        action: "analyze".into(),
        payload: serde_json::json!({"text": text}),
    };
    network.send("analyzer", msg).await?;

    let response = analyzer_rx.recv().await.unwrap();
    tracing::info!("Message response: {:?}", response);

    let broadcast = AgentMessage::Heartbeat {
        agent_name: "analyzer".into(),
        status: "healthy".into(),
    };
    network.broadcast(broadcast).await;

    tracing::info!("=== Demo Complete ===");
    Ok(())
}
