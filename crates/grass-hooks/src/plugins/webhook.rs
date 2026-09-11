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

use crate::hooks::{GrassHook, GrassHookPayload, GrassHookTrigger};
use async_trait::async_trait;
use grass_core::{GrassError, HookId, Result};
use std::collections::HashMap;

pub struct GrassWebhookHook {
    id: HookId,
    url: String,
    method: String,
    headers: HashMap<String, String>,
    http: reqwest::Client,
}

impl GrassWebhookHook {
    pub fn new(url: &str, method: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            url: url.to_string(),
            method: method.to_string(),
            headers: HashMap::new(),
            http: reqwest::Client::new(),
        }
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
}

#[async_trait]
impl GrassHook for GrassWebhookHook {
    fn id(&self) -> HookId {
        self.id
    }
    fn name(&self) -> &str {
        "webhook"
    }
    fn trigger(&self) -> GrassHookTrigger {
        GrassHookTrigger::OnCustom("webhook".into())
    }

    async fn execute(&self, payload: &GrassHookPayload) -> Result<serde_json::Value> {
        let mut req = match self.method.as_str() {
            "POST" => self.http.post(&self.url),
            "PUT" => self.http.put(&self.url),
            "PATCH" => self.http.patch(&self.url),
            _ => self.http.get(&self.url),
        };
        for (k, v) in &self.headers {
            req = req.header(k.as_str(), v.as_str());
        }
        let resp = req
            .header("Content-Type", "application/json")
            .json(payload)
            .send()
            .await
            .map_err(|e| GrassError::Hook(format!("webhook request failed: {}", e)))?;
        let status = resp.status().as_u16();
        Ok(serde_json::json!({"status": status}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_builder() {
        let hook = GrassWebhookHook::new("https://example.com/hook", "POST")
            .with_header("Authorization", "Bearer tok")
            .with_header("X-Custom", "val");
        assert_eq!(hook.headers.len(), 2);
    }
}
