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

pub struct GrassSlackHook {
    id: HookId,
    webhook_url: String,
    channel: String,
    http: reqwest::Client,
}

impl GrassSlackHook {
    pub fn new(webhook_url: &str, channel: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            webhook_url: webhook_url.to_string(),
            channel: channel.to_string(),
            http: reqwest::Client::new(),
        }
    }

    fn format_message(&self, payload: &GrassHookPayload) -> serde_json::Value {
        let text = format!(
            "[{}] {}",
            format!("{:?}", payload.trigger),
            payload
                .data
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("event occurred")
        );
        serde_json::json!({
            "channel": self.channel,
            "text": text,
            "blocks": [{
                "type": "section",
                "text": {"type": "mrkdwn", "text": format!("*grass.ai hook*\n{}", text)}
            }]
        })
    }
}

#[async_trait]
impl GrassHook for GrassSlackHook {
    fn id(&self) -> HookId {
        self.id
    }
    fn name(&self) -> &str {
        "slack"
    }
    fn trigger(&self) -> GrassHookTrigger {
        GrassHookTrigger::OnCustom("slack".into())
    }

    async fn execute(&self, payload: &GrassHookPayload) -> Result<serde_json::Value> {
        let body = self.format_message(payload);
        self.http
            .post(&self.webhook_url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| GrassError::Hook(format!("slack request failed: {}", e)))?;
        Ok(serde_json::json!({"status": "sent", "channel": self.channel}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_slack_hook_format_message() {
        let hook = GrassSlackHook::new("https://hooks.slack.com/test", "#general");
        let payload = GrassHookPayload {
            trigger: GrassHookTrigger::OnCustom("test".into()),
            data: serde_json::json!({"message": "hello"}),
            metadata: HashMap::new(),
        };
        let msg = hook.format_message(&payload);
        assert_eq!(msg["channel"], "#general");
        assert!(msg["text"].as_str().unwrap().contains("hello"));
    }
}
