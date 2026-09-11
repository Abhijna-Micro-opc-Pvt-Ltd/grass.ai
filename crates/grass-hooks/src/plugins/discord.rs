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

pub struct GrassDiscordHook {
    id: HookId,
    webhook_url: String,
    http: reqwest::Client,
}

impl GrassDiscordHook {
    pub fn new(webhook_url: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            webhook_url: webhook_url.to_string(),
            http: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl GrassHook for GrassDiscordHook {
    fn id(&self) -> HookId {
        self.id
    }
    fn name(&self) -> &str {
        "discord"
    }
    fn trigger(&self) -> GrassHookTrigger {
        GrassHookTrigger::OnCustom("discord".into())
    }

    async fn execute(&self, payload: &GrassHookPayload) -> Result<serde_json::Value> {
        let content = format!(
            "**[grass.ai]** {:?}: {}",
            payload.trigger,
            payload
                .data
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("event occurred")
        );
        self.http
            .post(&self.webhook_url)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({ "content": content }))
            .send()
            .await
            .map_err(|e| GrassError::Hook(format!("discord request failed: {}", e)))?;
        Ok(serde_json::json!({"status": "sent"}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_discord_hook_metadata() {
        let hook = GrassDiscordHook::new("https://discord.com/api/webhooks/test");
        assert_eq!(hook.name(), "discord");
        assert_eq!(format!("{:?}", hook.trigger()), "OnCustom(\"discord\")");
    }
}
