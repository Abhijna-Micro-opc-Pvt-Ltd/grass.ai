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
pub trait GrassFissClient: Send + Sync {
    async fn connect(&self, endpoint: &str) -> Result<()>;
    async fn query(&self, dataset: &str, filter: &serde_json::Value) -> Result<serde_json::Value>;
    async fn mutate(
        &self,
        dataset: &str,
        operation: &serde_json::Value,
    ) -> Result<serde_json::Value>;
    async fn disconnect(&self) -> Result<()>;
}

pub struct GrassFissAdapter {
    endpoint: String,
    http: reqwest::Client,
}

impl GrassFissAdapter {
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            http: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl GrassFissClient for GrassFissAdapter {
    async fn connect(&self, _endpoint: &str) -> Result<()> {
        tracing::info!("FISS connected to {}", self.endpoint);
        Ok(())
    }

    async fn query(&self, dataset: &str, filter: &serde_json::Value) -> Result<serde_json::Value> {
        let resp: serde_json::Value = self
            .http
            .post(format!("{}/query/{}", self.endpoint, dataset))
            .json(filter)
            .send()
            .await
            .map_err(|e| GrassError::Integration(e.to_string()))?
            .json()
            .await
            .map_err(|e| GrassError::Integration(e.to_string()))?;
        Ok(resp)
    }

    async fn mutate(
        &self,
        dataset: &str,
        operation: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let resp: serde_json::Value = self
            .http
            .post(format!("{}/mutate/{}", self.endpoint, dataset))
            .json(operation)
            .send()
            .await
            .map_err(|e| GrassError::Integration(e.to_string()))?
            .json()
            .await
            .map_err(|e| GrassError::Integration(e.to_string()))?;
        Ok(resp)
    }

    async fn disconnect(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fiss_adapter() {
        let adapter = GrassFissAdapter::new("https://fiss.example.com");
        assert_eq!(adapter.endpoint, "https://fiss.example.com");
    }
}
