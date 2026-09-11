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

pub struct GrassOllamaProvider {
    base_url: String,
    http: reqwest::Client,
}

impl GrassOllamaProvider {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            http: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<GrassChatMessage>,
    stream: bool,
    options: Option<OllamaOptions>,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: Option<f32>,
    num_predict: Option<u32>,
}

#[derive(Deserialize)]
struct OllamaResponse {
    model: String,
    message: GrassChatMessage,
    done: bool,
}

#[async_trait]
impl GrassLlmProvider for GrassOllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    async fn complete(&self, request: GrassLlmRequest) -> Result<GrassLlmResponse> {
        let body = OllamaRequest {
            model: request.model,
            messages: request.messages,
            stream: false,
            options: Some(OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens,
            }),
        };
        let resp = self
            .http
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| GrassError::Llm(format!("ollama failed: {}", e)))?;
        let or: OllamaResponse = resp
            .json()
            .await
            .map_err(|e| GrassError::Llm(format!("ollama parse: {}", e)))?;
        Ok(GrassLlmResponse {
            id: uuid::Uuid::new_v4().to_string(),
            model: or.model,
            choices: vec![GrassChoice {
                index: 0,
                message: or.message,
                finish_reason: Some("stop".into()),
            }],
            usage: GrassUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
        })
    }

    async fn stream<'a>(
        &'a self,
        request: GrassLlmRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<GrassStreamEvent>> + Send + 'a>> {
        let body = OllamaRequest {
            model: request.model,
            messages: request.messages,
            stream: true,
            options: Some(OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens,
            }),
        };
        let resp = self
            .http
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| GrassError::Llm(format!("ollama stream: {}", e)))?;
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
