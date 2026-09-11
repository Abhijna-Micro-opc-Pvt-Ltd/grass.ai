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
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrassLlmRequest {
    pub model: String,
    pub messages: Vec<GrassChatMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: bool,
    pub tools: Option<Vec<GrassToolDefinition>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrassChatMessage {
    pub role: GrassMessageRole,
    pub content: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GrassMessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrassLlmResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<GrassChoice>,
    pub usage: GrassUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrassChoice {
    pub index: u32,
    pub message: GrassChatMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrassUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrassToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrassToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GrassStreamEvent {
    Delta { content: String },
    ToolCall { tool_call: GrassToolCall },
    Done { usage: GrassUsage },
    Error { error: String },
}

#[async_trait]
pub trait GrassLlmProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn complete(&self, request: GrassLlmRequest) -> Result<GrassLlmResponse>;
    async fn stream<'a>(
        &'a self,
        request: GrassLlmRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<GrassStreamEvent>> + Send + 'a>>;
}

pub struct GrassLlmService {
    providers: HashMap<String, Box<dyn GrassLlmProvider>>,
    default_provider: Option<String>,
}

impl GrassLlmService {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: None,
        }
    }

    pub fn register_provider(&mut self, name: &str, provider: Box<dyn GrassLlmProvider>) {
        if self.default_provider.is_none() {
            self.default_provider = Some(name.to_string());
        }
        self.providers.insert(name.to_string(), provider);
    }

    pub fn set_default(&mut self, name: &str) {
        self.default_provider = Some(name.to_string());
    }

    pub fn provider(&self, name: &str) -> Option<&dyn GrassLlmProvider> {
        self.providers.get(name).map(|p| p.as_ref())
    }

    pub fn list_providers(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }

    pub async fn complete(&self, request: GrassLlmRequest) -> Result<GrassLlmResponse> {
        let name = self
            .default_provider
            .as_ref()
            .ok_or_else(|| GrassError::Llm("no default provider set".into()))?;
        let provider = self
            .providers
            .get(name)
            .ok_or_else(|| GrassError::Llm(format!("provider '{}' not found", name)))?;
        provider.complete(request).await
    }
}

impl Default for GrassLlmService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_service_register() {
        let mut svc = GrassLlmService::new();
        // We can't easily test with real providers without network,
        // but we can test the registry logic.
        assert!(svc.list_providers().is_empty());
        assert!(svc.default_provider.is_none());
    }

    #[test]
    fn test_chat_message_serialization() {
        let msg = GrassChatMessage {
            role: GrassMessageRole::User,
            content: "hello".into(),
            name: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("hello"));
    }
}
