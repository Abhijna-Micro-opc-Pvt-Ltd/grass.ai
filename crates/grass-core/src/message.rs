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

use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrassMessage {
    pub id: crate::MessageId,
    pub from: crate::AgentId,
    pub to: Option<crate::AgentId>,
    pub topic: String,
    pub payload: serde_json::Value,
    pub headers: HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl GrassMessage {
    pub fn new(from: crate::AgentId, topic: &str, payload: serde_json::Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            from,
            to: None,
            topic: topic.to_string(),
            payload,
            headers: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_header(mut self, key: &str, val: &str) -> Self {
        self.headers.insert(key.to_string(), val.to_string());
        self
    }

    pub fn directed_to(mut self, agent_id: crate::AgentId) -> Self {
        self.to = Some(agent_id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grass_message_creation() {
        let agent_id = uuid::Uuid::new_v4();
        let msg = GrassMessage::new(
            agent_id,
            "test.topic",
            serde_json::json!({"hello": "world"}),
        )
        .with_header("x-trace-id", "abc123")
        .directed_to(uuid::Uuid::new_v4());

        assert_eq!(msg.topic, "test.topic");
        assert_eq!(msg.headers.get("x-trace-id").unwrap(), "abc123");
        assert!(msg.to.is_some());
    }
}
