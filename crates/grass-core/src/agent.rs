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

use crate::{GrassAgentMetadata, GrassAgentState, GrassTaskPayload, GrassTaskResult};
use async_trait::async_trait;

#[async_trait]
pub trait GrassAgent: Send + Sync {
    /// Returns immutable reference to agent metadata.
    fn metadata(&self) -> &GrassAgentMetadata;
    /// Returns current lifecycle state of the agent.
    fn state(&self) -> GrassAgentState;
    /// Starts the agent and initializes resources.
    async fn start(&mut self) -> crate::Result<()>;
    /// Stops the agent and releases resources.
    async fn stop(&mut self) -> crate::Result<()>;
    /// Executes a single task and returns the result.
    async fn execute(&self, task: GrassTaskPayload) -> crate::Result<GrassTaskResult>;
    /// Handles an incoming message and optionally returns a reply.
    async fn handle_message(
        &self,
        msg: crate::GrassMessage,
    ) -> crate::Result<Option<crate::GrassMessage>>;
}

/// Builder pattern for constructing agent metadata.
pub struct GrassAgentBuilder {
    metadata: GrassAgentMetadata,
}

impl GrassAgentBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            metadata: GrassAgentMetadata {
                id: uuid::Uuid::new_v4(),
                name: name.to_string(),
                version: "0.1.0".to_string(),
                description: String::new(),
                tags: vec![],
                config: std::collections::HashMap::new(),
            },
        }
    }

    pub fn version(mut self, v: &str) -> Self {
        self.metadata.version = v.to_string();
        self
    }

    pub fn description(mut self, d: &str) -> Self {
        self.metadata.description = d.to_string();
        self
    }

    pub fn tag(mut self, t: &str) -> Self {
        self.metadata.tags.push(t.to_string());
        self
    }

    pub fn config(mut self, key: &str, val: serde_json::Value) -> Self {
        self.metadata.config.insert(key.to_string(), val);
        self
    }

    pub fn build(self) -> GrassAgentMetadata {
        self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_builder() {
        let meta = GrassAgentBuilder::new("test-agent")
            .version("1.0.0")
            .description("A test agent")
            .tag("testing")
            .config("key", serde_json::json!("value"))
            .build();

        assert_eq!(meta.name, "test-agent");
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.tags, vec!["testing".to_string()]);
        assert_eq!(meta.config.get("key").unwrap(), "value");
    }
}
