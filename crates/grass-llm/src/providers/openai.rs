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

pub struct GrassOpenAiProvider {
    api_key: String,
    base_url: String,
    http: reqwest::Client,
}

impl GrassOpenAiProvider {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            http: reqwest::Client::new(),
        }
    }

    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }
}

#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<GrassChatMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    stream: bool,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    id: String,
    model: String,
    choices: Vec<OpenAiChoice>,
    usage: OpenAiUsage,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    index: u32,
    message: GrassChatMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[async_trait]
impl GrassLlmProvider for GrassOpenAiProvider {
    fn name(&self) -> &str {
        "openai"
    }

    async fn complete(&self, request: GrassLlmRequest) -> Result<GrassLlmResponse> {
        let body = OpenAiRequest {
            model: request.model,
            messages: request.messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: false,
        };
        let resp = self
            .http
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| GrassError::Llm(format!("request failed: {}", e)))?;
        let openai_resp: OpenAiResponse = resp
            .json()
            .await
            .map_err(|e| GrassError::Llm(format!("parse failed: {}", e)))?;
        Ok(GrassLlmResponse {
            id: openai_resp.id,
            model: openai_resp.model,
            choices: openai_resp
                .choices
                .into_iter()
                .map(|c| GrassChoice {
                    index: c.index,
                    message: c.message,
                    finish_reason: c.finish_reason,
                })
                .collect(),
            usage: GrassUsage {
                prompt_tokens: openai_resp.usage.prompt_tokens,
                completion_tokens: openai_resp.usage.completion_tokens,
                total_tokens: openai_resp.usage.total_tokens,
            },
        })
    }

    async fn stream<'a>(
        &'a self,
        request: GrassLlmRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<GrassStreamEvent>> + Send + 'a>> {
        let body = OpenAiRequest {
            model: request.model,
            messages: request.messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: true,
        };
        let resp = self
            .http
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| GrassError::Llm(format!("stream request failed: {}", e)))?;
        let byte_stream = resp.bytes_stream();
        let mapped = byte_stream.map(move |chunk| {
            chunk
                .map(|bytes| {
                    let text = String::from_utf8_lossy(&bytes).to_string();
                    if text.trim() == "[DONE]" {
                        return GrassStreamEvent::Done {
                            usage: GrassUsage {
                                prompt_tokens: 0,
                                completion_tokens: 0,
                                total_tokens: 0,
                            },
                        };
                    }
                    GrassStreamEvent::Delta { content: text }
                })
                .map_err(|e| GrassError::Llm(format!("stream error: {}", e)))
        });
        Ok(Box::new(mapped))
    }
}
