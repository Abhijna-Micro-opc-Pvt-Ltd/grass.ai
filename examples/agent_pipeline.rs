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
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

struct PipelineStage {
    name: String,
    handler: Arc<
        dyn Fn(
                serde_json::Value,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<serde_json::Value>> + Send>,
            > + Send
            + Sync,
    >,
}

struct GrassPipeline {
    stages: Vec<PipelineStage>,
    results: Arc<RwLock<Vec<(String, serde_json::Value)>>>,
}

impl GrassPipeline {
    pub fn new() -> Self {
        Self {
            stages: vec![],
            results: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn add_stage<F, Fut>(mut self, name: &str, handler: F) -> Self
    where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<serde_json::Value>> + Send + 'static,
    {
        let boxed: Arc<
            dyn Fn(
                    serde_json::Value,
                ) -> std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<serde_json::Value>> + Send>,
                > + Send
                + Sync,
        > = Arc::new(move |input: serde_json::Value| Box::pin(handler(input)));
        self.stages.push(PipelineStage {
            name: name.to_string(),
            handler: boxed,
        });
        self
    }

    pub async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        let mut current = input;
        let mut results = self.results.write().await;

        for stage in &self.stages {
            tracing::info!("Pipeline stage: {}", stage.name);
            current = (stage.handler)(current).await?;
            results.push((stage.name.clone(), current.clone()));
        }

        Ok(current)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("=== Agent Pipeline Demo ===");

    let pipeline = GrassPipeline::new()
        .add_stage("validate", |input| async move {
            let text = input["text"].as_str().unwrap_or("");
            if text.is_empty() {
                return Err(GrassError::Task("empty text".into()));
            }
            Ok(serde_json::json!({"text": text, "valid": true, "length": text.len()}))
        })
        .add_stage("analyze", |input| async move {
            let text = input["text"].as_str().unwrap_or("");
            let words: Vec<&str> = text.split_whitespace().collect();
            Ok(serde_json::json!({
                "text": text,
                "word_count": words.len(),
                "unique_words": words.iter().collect::<std::collections::HashSet<_>>().len(),
                "avg_word_length": words.iter().map(|w| w.len()).sum::<usize>() as f64 / words.len() as f64,
            }))
        })
        .add_stage("enrich", |input| async move {
            Ok(serde_json::json!({
                "text": input["text"],
                "analysis": {
                    "word_count": input["word_count"],
                    "unique_words": input["unique_words"],
                    "avg_word_length": input["avg_word_length"],
                },
                "metadata": {
                    "pipeline_version": "1.0",
                    "processed_at": chrono::Utc::now().to_rfc3339(),
                }
            }))
        });

    let input = serde_json::json!({
        "text": "Rust is a systems programming language that is fast and safe"
    });

    let result = pipeline.execute(input).await?;
    tracing::info!(
        "Pipeline result: {}",
        serde_json::to_string_pretty(&result).unwrap()
    );

    tracing::info!("=== Pipeline Demo Complete ===");
    Ok(())
}
