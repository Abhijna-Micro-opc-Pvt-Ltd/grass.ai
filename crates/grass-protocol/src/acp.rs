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
use grass_core::{GrassError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrassAcpMessage {
    pub source: String,
    pub destination: String,
    pub content_type: String,
    pub payload: serde_json::Value,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrassAcpEnvelope {
    pub message: GrassAcpMessage,
    pub signature: Option<String>,
    pub timestamp: String,
}

#[async_trait]
pub trait GrassAcpCodec: Send + Sync {
    fn encode(&self, message: &GrassAcpMessage) -> Result<Vec<u8>>;
    fn decode(&self, data: &[u8]) -> Result<GrassAcpMessage>;
}

pub struct GrassAcpJsonCodec;

impl GrassAcpCodec for GrassAcpJsonCodec {
    fn encode(&self, message: &GrassAcpMessage) -> Result<Vec<u8>> {
        serde_json::to_vec(message).map_err(GrassError::Serialization)
    }

    fn decode(&self, data: &[u8]) -> Result<GrassAcpMessage> {
        serde_json::from_slice(data).map_err(GrassError::Serialization)
    }
}

pub struct GrassAcpRouter {
    handlers: std::collections::HashMap<String, std::sync::Arc<dyn GrassAcpHandler>>,
}

#[async_trait]
pub trait GrassAcpHandler: Send + Sync {
    async fn handle(&self, message: GrassAcpMessage) -> Result<GrassAcpMessage>;
}

impl GrassAcpRouter {
    pub fn new() -> Self {
        Self {
            handlers: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, content_type: &str, handler: std::sync::Arc<dyn GrassAcpHandler>) {
        self.handlers.insert(content_type.to_string(), handler);
    }

    pub async fn route(&self, message: GrassAcpMessage) -> Result<GrassAcpMessage> {
        let handler = self.handlers.get(&message.content_type).ok_or_else(|| {
            GrassError::Protocol(format!("no handler for: {}", message.content_type))
        })?;
        handler.handle(message).await
    }
}

impl Default for GrassAcpRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acp_json_codec_roundtrip() {
        let msg = GrassAcpMessage {
            source: "agent-a".into(),
            destination: "agent-b".into(),
            content_type: "text/plain".into(),
            payload: serde_json::json!("hello"),
            metadata: std::collections::HashMap::new(),
        };
        let codec = GrassAcpJsonCodec;
        let encoded = codec.encode(&msg).unwrap();
        let decoded = codec.decode(&encoded).unwrap();
        assert_eq!(decoded.source, "agent-a");
        assert_eq!(decoded.destination, "agent-b");
    }
}
