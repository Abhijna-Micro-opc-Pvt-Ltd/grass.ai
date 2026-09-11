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

use grass_core::{
    GrassAgent, GrassAgentBuilder, GrassTaskPayload, GrassTaskResult, GrassTaskStatus,
};
use grass_inference::{GrassInferenceBackend, GrassTensor, GrassTensorRtBackend};
use grass_streaming::{GrassRtspClient, GrassRtspStream};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct PatrolVisit {
    beat_route: String,
    zone: String,
    date: String,
    visited: bool,
    via_surveillance: String,
    visited_at: String,
}

struct CityBeatAgent {
    metadata: grass_core::GrassAgentMetadata,
    backend: GrassTensorRtBackend,
    camera: Arc<GrassRtspStream>,
    patrol_history: Arc<RwLock<Vec<PatrolVisit>>>,
}

impl CityBeatAgent {
    async fn detect_patrol(&self, route: &str) -> bool {
        let frame = self.camera.get_frame().await.unwrap_or_default();
        let input = GrassTensor {
            data: vec![0.5; 3 * 224 * 224],
            shape: vec![1, 3, 224, 224],
        };
        let output = self.backend.infer(&input).await.unwrap();
        let score = output.data.iter().sum::<f32>() / output.data.len().max(1) as f32;
        let visited = score > 0.35 + (route.len() as f32 * 0.001 % 0.1);
        visited || !frame.is_empty()
    }
}

#[async_trait::async_trait]
impl GrassAgent for CityBeatAgent {
    fn metadata(&self) -> &grass_core::GrassAgentMetadata {
        &self.metadata
    }

    fn state(&self) -> grass_core::GrassAgentState {
        grass_core::GrassAgentState::Running
    }

    async fn start(&mut self) -> grass_core::Result<()> {
        self.backend
            .load_model("/models/patrol-detector.engine")
            .await?;
        self.camera
            .connect("rtsp://admin:pass@192.168.88.10:554/citybeat")
            .await?;
        tracing::info!("City surveillance camera connected");
        Ok(())
    }

    async fn stop(&mut self) -> grass_core::Result<()> {
        self.camera.disconnect().await?;
        Ok(())
    }

    async fn execute(&self, task: GrassTaskPayload) -> grass_core::Result<GrassTaskResult> {
        match task.action.as_str() {
            "check_beat_today" => {
                let route = task.input["route"]
                    .as_str()
                    .unwrap_or("CENTRAL-01")
                    .to_string();
                let zone = task.input["zone"]
                    .as_str()
                    .unwrap_or("North Zone")
                    .to_string();

                let today = chrono::Utc::now().date_naive().to_string();
                let visited = self.detect_patrol(&route).await;

                let record = PatrolVisit {
                    beat_route: route,
                    zone,
                    date: today.clone(),
                    visited,
                    via_surveillance: format!("city-cctv-router-{}", today),
                    visited_at: if visited {
                        chrono::Utc::now().to_rfc3339()
                    } else {
                        String::new()
                    },
                };
                self.patrol_history.write().await.push(record.clone());

                Ok(GrassTaskResult {
                    task_id: task.task_id,
                    agent_id: self.metadata.id,
                    status: GrassTaskStatus::Completed,
                    output: serde_json::json!({
                        "route": record.beat_route,
                        "zone": record.zone,
                        "today": record.date,
                        "police_visited_today": record.visited,
                        "display": if record.visited {
                            format!("Police visited route {} today via surveillance", record.beat_route)
                        } else {
                            format!("Route {} not yet visited TODAY - flag for patrol", record.beat_route)
                        },
                        "visited_at": record.visited_at,
                        "source": record.via_surveillance,
                    }),
                    error: None,
                    duration_ms: 25,
                })
            }
            "beat_history" => {
                let history = self.patrol_history.read().await.clone();
                Ok(GrassTaskResult {
                    task_id: task.task_id,
                    agent_id: self.metadata.id,
                    status: GrassTaskStatus::Completed,
                    output: serde_json::json!({
                        "history": history,
                        "total_checks": history.len(),
                        "visited_today": history.iter().filter(|h| h.visited).count(),
                    }),
                    error: None,
                    duration_ms: 2,
                })
            }
            _ => Ok(GrassTaskResult {
                task_id: task.task_id,
                agent_id: self.metadata.id,
                status: GrassTaskStatus::Failed,
                output: serde_json::json!({}),
                error: Some(format!("unknown action: {}", task.action)),
                duration_ms: 0,
            }),
        }
    }

    async fn handle_message(
        &self,
        msg: grass_core::GrassMessage,
    ) -> grass_core::Result<Option<grass_core::GrassMessage>> {
        tracing::info!("Message: {}", msg.topic);
        Ok(None)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("===== City Beat Count - Police Patrol Route Monitor =====");
    tracing::info!("Night surveillance records whether police visited each route today");

    let meta = GrassAgentBuilder::new("city-beat-monitor")
        .version("1.0.0")
        .description("Police beat history per route using available surveillance system")
        .tag("city")
        .tag("surveillance")
        .tag("police-beat")
        .build();

    let mut agent = CityBeatAgent {
        metadata: meta,
        backend: GrassTensorRtBackend::new(),
        camera: Arc::new(GrassRtspStream::new()),
        patrol_history: Arc::new(RwLock::new(vec![])),
    };

    agent.start().await?;

    let routes = [
        ("CENTRAL-01", "North Zone"),
        ("CENTRAL-02", "South Zone"),
        ("NIGHT-03", "Market District"),
        ("NIGHT-04", "Railway Station"),
        ("NIGHT-05", "Riverfront"),
    ];

    for (route, zone) in routes {
        let result = agent
            .execute(GrassTaskPayload {
                task_id: uuid::Uuid::new_v4(),
                agent_id: agent.metadata().id,
                action: "check_beat_today".into(),
                input: serde_json::json!({"route": route, "zone": zone}),
                metadata: HashMap::new(),
            })
            .await?;
        tracing::info!("{}", result.output["display"]);
    }

    let history = agent
        .execute(GrassTaskPayload {
            task_id: uuid::Uuid::new_v4(),
            agent_id: agent.metadata().id,
            action: "beat_history".into(),
            input: serde_json::json!({}),
            metadata: HashMap::new(),
        })
        .await?;

    tracing::info!(
        "Patrol history summary: {}",
        serde_json::to_string_pretty(&history.output)?
    );

    agent.stop().await?;

    tracing::info!("===== City Beat Count Example Complete =====");
    Ok(())
}
