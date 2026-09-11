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
pub struct GrassMcpRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrassMcpResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<GrassMcpError>,
    pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrassMcpError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrassMcpNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[async_trait]
pub trait GrassMcpHandler: Send + Sync {
    async fn handle_request(&self, request: GrassMcpRequest) -> Result<GrassMcpResponse>;
    async fn handle_notification(&self, notification: GrassMcpNotification) -> Result<()>;
}

pub struct GrassMcpCodec;

impl GrassMcpCodec {
    pub fn encode_request(request: &GrassMcpRequest) -> Result<Vec<u8>> {
        serde_json::to_vec(request).map_err(GrassError::Serialization)
    }

    pub fn decode_request(data: &[u8]) -> Result<GrassMcpRequest> {
        serde_json::from_slice(data).map_err(GrassError::Serialization)
    }

    pub fn encode_response(response: &GrassMcpResponse) -> Result<Vec<u8>> {
        serde_json::to_vec(response).map_err(GrassError::Serialization)
    }

    pub fn decode_response(data: &[u8]) -> Result<GrassMcpResponse> {
        serde_json::from_slice(data).map_err(GrassError::Serialization)
    }

    pub fn encode_notification(notification: &GrassMcpNotification) -> Result<Vec<u8>> {
        serde_json::to_vec(notification).map_err(GrassError::Serialization)
    }

    pub fn decode_notification(data: &[u8]) -> Result<GrassMcpNotification> {
        serde_json::from_slice(data).map_err(GrassError::Serialization)
    }
}

pub struct GrassMcpRouter {
    handlers: std::collections::HashMap<String, std::sync::Arc<dyn GrassMcpHandler>>,
}

impl GrassMcpRouter {
    pub fn new() -> Self {
        Self {
            handlers: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, method: &str, handler: std::sync::Arc<dyn GrassMcpHandler>) {
        self.handlers.insert(method.to_string(), handler);
    }

    pub async fn route(&self, request: GrassMcpRequest) -> Result<GrassMcpResponse> {
        let handler = self.handlers.get(&request.method).ok_or_else(|| {
            GrassError::Protocol(format!("no handler for method: {}", request.method))
        })?;
        handler.handle_request(request).await
    }
}

impl Default for GrassMcpRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_codec_roundtrip() {
        let req = GrassMcpRequest {
            jsonrpc: "2.0".into(),
            method: "test".into(),
            params: serde_json::json!({"key": "value"}),
            id: 1,
        };
        let encoded = GrassMcpCodec::encode_request(&req).unwrap();
        let decoded = GrassMcpCodec::decode_request(&encoded).unwrap();
        assert_eq!(decoded.method, "test");
        assert_eq!(decoded.id, 1);
    }
}
