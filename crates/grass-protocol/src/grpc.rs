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

#[async_trait]
pub trait GrassGrpcService: Send + Sync {
    fn service_name(&self) -> &str;
    async fn invoke(&self, method: &str, request: &[u8]) -> Result<Vec<u8>>;
}

pub struct GrassGrpcRegistry {
    services: std::collections::HashMap<String, std::sync::Arc<dyn GrassGrpcService>>,
}

impl GrassGrpcRegistry {
    pub fn new() -> Self {
        Self {
            services: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, service: std::sync::Arc<dyn GrassGrpcService>) {
        self.services.insert(name.to_string(), service);
    }

    pub async fn invoke(&self, service: &str, method: &str, request: &[u8]) -> Result<Vec<u8>> {
        let svc = self
            .services
            .get(service)
            .ok_or_else(|| GrassError::Protocol(format!("service '{}' not found", service)))?;
        svc.invoke(method, request).await
    }
}

impl Default for GrassGrpcRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GrassProtobufCodec;

impl GrassProtobufCodec {
    pub fn encode<T: serde::Serialize>(value: &T) -> Result<Vec<u8>> {
        serde_json::to_vec(value).map_err(GrassError::Serialization)
    }

    pub fn decode<T: serde::de::DeserializeOwned>(data: &[u8]) -> Result<T> {
        serde_json::from_slice(data).map_err(GrassError::Serialization)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protobuf_codec_roundtrip() {
        #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
        struct TestMsg {
            field: String,
        }
        let msg = TestMsg {
            field: "test".into(),
        };
        let encoded = GrassProtobufCodec::encode(&msg).unwrap();
        let decoded: TestMsg = GrassProtobufCodec::decode(&encoded).unwrap();
        assert_eq!(decoded, msg);
    }
}
