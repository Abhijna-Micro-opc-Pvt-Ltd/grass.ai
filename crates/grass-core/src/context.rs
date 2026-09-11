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

use crate::{AgentId, GrassAgentMetadata, GrassSandboxConfig};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GrassAgentContext {
    pub agent_id: AgentId,
    pub metadata: GrassAgentMetadata,
    pub sandbox: Option<GrassSandboxConfig>,
    pub shared_state: HashMap<String, serde_json::Value>,
}

impl GrassAgentContext {
    pub fn new(agent_id: AgentId, metadata: GrassAgentMetadata) -> Self {
        Self {
            agent_id,
            metadata,
            sandbox: None,
            shared_state: HashMap::new(),
        }
    }

    pub fn with_sandbox(mut self, config: GrassSandboxConfig) -> Self {
        self.sandbox = Some(config);
        self
    }

    pub fn set_state(&mut self, key: &str, val: serde_json::Value) {
        self.shared_state.insert(key.to_string(), val);
    }

    pub fn get_state(&self, key: &str) -> Option<&serde_json::Value> {
        self.shared_state.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_context() {
        let id = uuid::Uuid::new_v4();
        let meta = crate::GrassAgentBuilder::new("ctx-test").build();
        let mut ctx = GrassAgentContext::new(id, meta).with_sandbox(GrassSandboxConfig::default());

        ctx.set_state("key", serde_json::json!("val"));
        assert_eq!(ctx.get_state("key").unwrap(), "val");
        assert!(ctx.sandbox.is_some());
    }
}
