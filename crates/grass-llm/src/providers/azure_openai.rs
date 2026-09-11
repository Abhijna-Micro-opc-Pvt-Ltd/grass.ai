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

pub struct GrassAzureOpenAiProvider {
    api_key: String,
    base_url: String,
    deployment: String,
    api_version: String,
    http: reqwest::Client,
}

impl GrassAzureOpenAiProvider {
    pub fn new(api_key: &str, base_url: &str, deployment: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            base_url: base_url.to_string(),
            deployment: deployment.to_string(),
            api_version: "2024-02-15-preview".to_string(),
            http: reqwest::Client::new(),
        }
    }

    pub fn with_api_version(mut self, version: &str) -> Self {
        self.api_version = version.to_string();
        self
    }
}

#[derive(Serialize)]
struct AzureRequest {
    messages: Vec<GrassChatMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    stream: bool,
}

#[derive(Deserialize)]
struct AzureResponse {
    id: String,
    choices: Vec<AzureChoice>,
    usage: AzureUsage,
}

#[derive(Deserialize)]
struct AzureChoice {
    index: u32,
    message: GrassChatMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct AzureUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[async_trait]
impl GrassLlmProvider for GrassAzureOpenAiProvider {
    fn name(&self) -> &str {
        "azure-openai"
    }

    async fn complete(&self, request: GrassLlmRequest) -> Result<GrassLlmResponse> {
        let body = AzureRequest {
            messages: request.messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: false,
        };
        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.base_url, self.deployment, self.api_version
        );
        let resp = self
            .http
            .post(&url)
            .header("api-key", &self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| GrassError::Llm(format!("azure failed: {}", e)))?;
        let r: AzureResponse = resp
            .json()
            .await
            .map_err(|e| GrassError::Llm(format!("azure parse: {}", e)))?;
        Ok(GrassLlmResponse {
            id: r.id,
            model: self.deployment.clone(),
            choices: r
                .choices
                .into_iter()
                .map(|c| GrassChoice {
                    index: c.index,
                    message: c.message,
                    finish_reason: c.finish_reason,
                })
                .collect(),
            usage: GrassUsage {
                prompt_tokens: r.usage.prompt_tokens,
                completion_tokens: r.usage.completion_tokens,
                total_tokens: r.usage.total_tokens,
            },
        })
    }

    async fn stream<'a>(
        &'a self,
        request: GrassLlmRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<GrassStreamEvent>> + Send + 'a>> {
        let body = AzureRequest {
            messages: request.messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: true,
        };
        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version={}",
            self.base_url, self.deployment, self.api_version
        );
        let resp = self
            .http
            .post(&url)
            .header("api-key", &self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| GrassError::Llm(format!("azure stream: {}", e)))?;
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
                .map_err(|e| GrassError::Llm(format!("stream: {}", e)))
        });
        Ok(Box::new(mapped))
    }
}
