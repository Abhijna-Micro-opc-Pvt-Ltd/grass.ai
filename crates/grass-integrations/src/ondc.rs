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
use grass_core::Result;

#[async_trait]
pub trait GrassOndcClient: Send + Sync {
    async fn search(&self, query: &str, location: &str) -> Result<serde_json::Value>;
    async fn select(&self, order: &serde_json::Value) -> Result<serde_json::Value>;
    async fn confirm(&self, order_id: &str) -> Result<serde_json::Value>;
    async fn status(&self, order_id: &str) -> Result<serde_json::Value>;
}

pub struct GrassOndcAdapter {
    gateway_url: String,
    api_key: String,
    http: reqwest::Client,
}

impl GrassOndcAdapter {
    pub fn new(gateway_url: &str, api_key: &str) -> Self {
        Self {
            gateway_url: gateway_url.to_string(),
            api_key: api_key.to_string(),
            http: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl GrassOndcClient for GrassOndcAdapter {
    async fn search(&self, query: &str, location: &str) -> Result<serde_json::Value> {
        tracing::info!("ONDC search: {} in {}", query, location);
        Ok(serde_json::json!({"status": "search_initiated"}))
    }

    async fn select(&self, _order: &serde_json::Value) -> Result<serde_json::Value> {
        Ok(serde_json::json!({"status": "selected"}))
    }

    async fn confirm(&self, order_id: &str) -> Result<serde_json::Value> {
        Ok(serde_json::json!({"status": "confirmed", "order_id": order_id}))
    }

    async fn status(&self, order_id: &str) -> Result<serde_json::Value> {
        Ok(serde_json::json!({"status": "active", "order_id": order_id}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ondc_adapter() {
        let adapter = GrassOndcAdapter::new("https://gateway.ondc.org", "key123");
        assert_eq!(adapter.gateway_url, "https://gateway.ondc.org");
    }
}
