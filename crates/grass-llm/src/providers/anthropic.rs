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

use crate::provider::*;
use async_trait::async_trait;
use futures::StreamExt;
use grass_core::{GrassError, Result};
use serde::{Deserialize, Serialize};

pub struct GrassAnthropicProvider {
    api_key: String,
    base_url: String,
    http: reqwest::Client,
}

impl GrassAnthropicProvider {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            base_url: "https://api.anthropic.com/v1".to_string(),
            http: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<GrassChatMessage>,
    system: Option<String>,
    temperature: Option<f32>,
    stream: bool,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    id: String,
    content: Vec<AnthropicContent>,
    stop_reason: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Deserialize)]
struct AnthropicContent {
    text: String,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[async_trait]
impl GrassLlmProvider for GrassAnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    async fn complete(&self, request: GrassLlmRequest) -> Result<GrassLlmResponse> {
        let model = request.model.clone();
        let system_msg = request
            .messages
            .iter()
            .find(|m| m.role == GrassMessageRole::System)
            .map(|m| m.content.clone());
        let messages: Vec<GrassChatMessage> = request
            .messages
            .into_iter()
            .filter(|m| m.role != GrassMessageRole::System)
            .collect();

        let body = AnthropicRequest {
            model: model.clone(),
            max_tokens: request.max_tokens.unwrap_or(4096),
            messages,
            system: system_msg,
            temperature: request.temperature,
            stream: false,
        };
        let resp = self
            .http
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|e| GrassError::Llm(format!("anthropic failed: {}", e)))?;
        let ar: AnthropicResponse = resp
            .json()
            .await
            .map_err(|e| GrassError::Llm(format!("anthropic parse: {}", e)))?;
        let content = ar
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();
        Ok(GrassLlmResponse {
            id: ar.id,
            model,
            choices: vec![GrassChoice {
                index: 0,
                message: GrassChatMessage {
                    role: GrassMessageRole::Assistant,
                    content,
                    name: None,
                },
                finish_reason: ar.stop_reason,
            }],
            usage: GrassUsage {
                prompt_tokens: ar.usage.input_tokens,
                completion_tokens: ar.usage.output_tokens,
                total_tokens: ar.usage.input_tokens + ar.usage.output_tokens,
            },
        })
    }

    async fn stream<'a>(
        &'a self,
        request: GrassLlmRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<GrassStreamEvent>> + Send + 'a>> {
        let model = request.model.clone();
        let system_msg = request
            .messages
            .iter()
            .find(|m| m.role == GrassMessageRole::System)
            .map(|m| m.content.clone());
        let messages: Vec<GrassChatMessage> = request
            .messages
            .into_iter()
            .filter(|m| m.role != GrassMessageRole::System)
            .collect();
        let body = AnthropicRequest {
            model,
            max_tokens: request.max_tokens.unwrap_or(4096),
            messages,
            system: system_msg,
            temperature: request.temperature,
            stream: true,
        };
        let resp = self
            .http
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|e| GrassError::Llm(format!("anthropic stream: {}", e)))?;
        let byte_stream = resp.bytes_stream();
        let mapped = byte_stream.map(move |chunk| {
            chunk
                .map(|bytes| GrassStreamEvent::Delta {
                    content: String::from_utf8_lossy(&bytes).to_string(),
                })
                .map_err(|e| GrassError::Llm(format!("stream: {}", e)))
        });
        Ok(Box::new(mapped))
    }
}
