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
pub trait GrassMakeComClient: Send + Sync {
    async fn trigger_scenario(
        &self,
        scenario_id: &str,
        data: serde_json::Value,
    ) -> Result<serde_json::Value>;
    async fn list_scenarios(&self) -> Result<Vec<serde_json::Value>>;
}

pub struct GrassMakeComAdapter {
    webhook_url: String,
    http: reqwest::Client,
}

impl GrassMakeComAdapter {
    pub fn new(webhook_url: &str) -> Self {
        Self {
            webhook_url: webhook_url.to_string(),
            http: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl GrassMakeComClient for GrassMakeComAdapter {
    async fn trigger_scenario(
        &self,
        _scenario_id: &str,
        data: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let resp: serde_json::Value = self
            .http
            .post(&self.webhook_url)
            .json(&data)
            .send()
            .await
            .map_err(|e| GrassError::Flow(e.to_string()))?
            .json()
            .await
            .map_err(|e| GrassError::Flow(e.to_string()))?;
        Ok(resp)
    }

    async fn list_scenarios(&self) -> Result<Vec<serde_json::Value>> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_makecom_adapter() {
        let adapter = GrassMakeComAdapter::new("https://hook.make.com/abc123");
        assert_eq!(adapter.webhook_url, "https://hook.make.com/abc123");
    }
}
