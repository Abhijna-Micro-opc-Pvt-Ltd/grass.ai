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
pub trait GrassN8nClient: Send + Sync {
    async fn trigger_workflow(
        &self,
        workflow_id: &str,
        data: serde_json::Value,
    ) -> Result<serde_json::Value>;
    async fn get_workflow(&self, workflow_id: &str) -> Result<serde_json::Value>;
    async fn list_workflows(&self) -> Result<Vec<serde_json::Value>>;
}

pub struct GrassN8nAdapter {
    base_url: String,
    api_key: String,
    http: reqwest::Client,
}

impl GrassN8nAdapter {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            http: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl GrassN8nClient for GrassN8nAdapter {
    async fn trigger_workflow(
        &self,
        workflow_id: &str,
        data: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let resp: serde_json::Value = self
            .http
            .post(format!(
                "{}/api/v1/workflows/{}/activate",
                self.base_url, workflow_id
            ))
            .header("X-N8N-API-KEY", &self.api_key)
            .json(&data)
            .send()
            .await
            .map_err(|e| GrassError::Flow(e.to_string()))?
            .json()
            .await
            .map_err(|e| GrassError::Flow(e.to_string()))?;
        Ok(resp)
    }

    async fn get_workflow(&self, workflow_id: &str) -> Result<serde_json::Value> {
        let resp: serde_json::Value = self
            .http
            .get(format!(
                "{}/api/v1/workflows/{}",
                self.base_url, workflow_id
            ))
            .header("X-N8N-API-KEY", &self.api_key)
            .send()
            .await
            .map_err(|e| GrassError::Flow(e.to_string()))?
            .json()
            .await
            .map_err(|e| GrassError::Flow(e.to_string()))?;
        Ok(resp)
    }

    async fn list_workflows(&self) -> Result<Vec<serde_json::Value>> {
        let resp: serde_json::Value = self
            .http
            .get(format!("{}/api/v1/workflows", self.base_url))
            .header("X-N8N-API-KEY", &self.api_key)
            .send()
            .await
            .map_err(|e| GrassError::Flow(e.to_string()))?
            .json()
            .await
            .map_err(|e| GrassError::Flow(e.to_string()))?;
        Ok(resp["data"].as_array().cloned().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_n8n_adapter() {
        let adapter = GrassN8nAdapter::new("https://n8n.example.com", "key123");
        assert_eq!(adapter.base_url, "https://n8n.example.com");
    }
}
