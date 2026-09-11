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
    GrassAgent, GrassAgentBuilder, GrassAgentMetadata, GrassAgentState, GrassError, GrassMessage,
    GrassTaskPayload, GrassTaskResult, GrassTaskStatus, Result,
};
use std::collections::HashMap;
use uuid::Uuid;

pub struct GrassPlannerAgent {
    metadata: GrassAgentMetadata,
    state: GrassAgentState,
    plan: Vec<GrassPlannerStep>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrassPlannerStep {
    pub step_id: String,
    pub description: String,
    pub action: String,
    pub dependencies: Vec<String>,
    pub status: GrassTaskStatus,
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrassPlanRequest {
    pub goal: String,
    pub context: serde_json::Value,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrassPlanResponse {
    pub plan_id: String,
    pub steps: Vec<GrassPlannerStep>,
    pub estimated_duration_ms: u64,
    pub required_agents: Vec<String>,
}

impl GrassPlannerAgent {
    pub fn new() -> Self {
        let metadata = GrassAgentBuilder::new("planner-agent")
            .version("0.1.0")
            .description("Decomposes goals into executable task plans")
            .tag("planner")
            .tag("orchestrator")
            .build();

        Self {
            metadata,
            state: GrassAgentState::Idle,
            plan: vec![],
        }
    }

    pub async fn create_plan(&mut self, request: &GrassPlanRequest) -> Result<GrassPlanResponse> {
        self.state = GrassAgentState::Running;

        let steps = self.decompose_goal(&request.goal, &request.context).await?;
        let required_agents = self.determine_required_agents(&steps);

        self.plan = steps.clone();
        let step_count = steps.len();

        let response = GrassPlanResponse {
            plan_id: Uuid::new_v4().to_string(),
            steps,
            estimated_duration_ms: step_count as u64 * 5000,
            required_agents,
        };

        self.state = GrassAgentState::Completed;
        Ok(response)
    }

    async fn decompose_goal(
        &self,
        goal: &str,
        context: &serde_json::Value,
    ) -> Result<Vec<GrassPlannerStep>> {
        let mut steps = vec![];

        steps.push(GrassPlannerStep {
            step_id: "analyze".into(),
            description: format!("Analyze goal: {}", goal),
            action: "analyze_goal".into(),
            dependencies: vec![],
            status: GrassTaskStatus::Pending,
            result: None,
        });

        steps.push(GrassPlannerStep {
            step_id: "research".into(),
            description: "Research available data and resources".into(),
            action: "research_context".into(),
            dependencies: vec!["analyze".into()],
            status: GrassTaskStatus::Pending,
            result: None,
        });

        steps.push(GrassPlannerStep {
            step_id: "execute".into(),
            description: "Execute core task".into(),
            action: "execute_task".into(),
            dependencies: vec!["research".into()],
            status: GrassTaskStatus::Pending,
            result: None,
        });

        steps.push(GrassPlannerStep {
            step_id: "verify".into(),
            description: "Verify results and validate output".into(),
            action: "verify_results".into(),
            dependencies: vec!["execute".into()],
            status: GrassTaskStatus::Pending,
            result: None,
        });

        steps.push(GrassPlannerStep {
            step_id: "report".into(),
            description: "Generate final report".into(),
            action: "generate_report".into(),
            dependencies: vec!["verify".into()],
            status: GrassTaskStatus::Pending,
            result: None,
        });

        Ok(steps)
    }

    fn determine_required_agents(&self, steps: &[GrassPlannerStep]) -> Vec<String> {
        let mut agents = vec![];
        for step in steps {
            match step.action.as_str() {
                "analyze_goal" => agents.push("analyzer-agent".into()),
                "research_context" => {
                    agents.push("researcher-agent".into());
                    agents.push("vector-search-agent".into());
                }
                "execute_task" => agents.push("executor-agent".into()),
                "verify_results" => agents.push("validator-agent".into()),
                "generate_report" => agents.push("reporter-agent".into()),
                _ => {}
            }
        }
        agents.sort();
        agents.dedup();
        agents
    }

    pub async fn update_step(
        &mut self,
        step_id: &str,
        status: GrassTaskStatus,
        result: Option<serde_json::Value>,
    ) -> Result<()> {
        for step in &mut self.plan {
            if step.step_id == step_id {
                step.status = status;
                step.result = result;
                return Ok(());
            }
        }
        Err(GrassError::Task(format!("step '{}' not found", step_id)))
    }

    pub fn current_plan(&self) -> &[GrassPlannerStep] {
        &self.plan
    }
}

#[async_trait]
impl GrassAgent for GrassPlannerAgent {
    fn metadata(&self) -> &GrassAgentMetadata {
        &self.metadata
    }
    fn state(&self) -> GrassAgentState {
        self.state.clone()
    }
    async fn start(&mut self) -> Result<()> {
        self.state = GrassAgentState::Idle;
        Ok(())
    }
    async fn stop(&mut self) -> Result<()> {
        self.state = GrassAgentState::Idle;
        Ok(())
    }

    async fn execute(&self, task: GrassTaskPayload) -> Result<GrassTaskResult> {
        let start = std::time::Instant::now();
        let request: GrassPlanRequest = serde_json::from_value(task.input)
            .map_err(|e| GrassError::Task(format!("invalid plan request: {}", e)))?;
        let mut planner = GrassPlannerAgent::new();
        let plan = planner.create_plan(&request).await?;
        Ok(GrassTaskResult {
            task_id: task.task_id,
            agent_id: task.agent_id,
            status: GrassTaskStatus::Completed,
            output: serde_json::to_value(&plan).unwrap_or_default(),
            error: None,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    async fn handle_message(&self, msg: GrassMessage) -> Result<Option<GrassMessage>> {
        if msg.topic == "plan.request" {
            let request: GrassPlanRequest = serde_json::from_value(msg.payload)
                .map_err(|e| GrassError::Agent(format!("invalid message: {}", e)))?;
            let mut planner = GrassPlannerAgent::new();
            let plan = planner.create_plan(&request).await?;
            return Ok(Some(
                GrassMessage::new(
                    self.metadata.id,
                    "plan.response",
                    serde_json::to_value(&plan).unwrap_or_default(),
                )
                .directed_to(msg.from),
            ));
        }
        Ok(None)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut planner = GrassPlannerAgent::new();
    planner.start().await?;

    let request = GrassPlanRequest {
        goal: "Build a sentiment analysis pipeline for customer reviews".into(),
        context: serde_json::json!({
            "data_source": "s3://reviews-bucket",
            "output_format": "json",
            "languages": ["en", "es"]
        }),
        constraints: vec![
            "Must complete within 1 hour".into(),
            "Cost under $10".into(),
        ],
    };

    let plan = planner.create_plan(&request).await?;
    println!("Plan: {}", serde_json::to_string_pretty(&plan).unwrap());
    println!("Steps: {}", plan.steps.len());
    println!("Required agents: {:?}", plan.required_agents);

    planner.stop().await?;
    Ok(())
}
