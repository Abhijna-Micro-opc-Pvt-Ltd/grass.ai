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
use grass_core::{GrassError, Result};
use serde::{Deserialize, Serialize};

pub struct GrassHuggingFaceProvider {
    api_key: String,
    base_url: String,
    http: reqwest::Client,
}

impl GrassHuggingFaceProvider {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            base_url: "https://api-inference.huggingface.co".to_string(),
            http: reqwest::Client::new(),
        }
    }

    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }
}

#[derive(Serialize)]
struct HfRequest {
    inputs: String,
    parameters: HfParameters,
}

#[derive(Serialize)]
struct HfParameters {
    max_new_tokens: Option<u32>,
    temperature: Option<f32>,
    return_full_text: bool,
}

#[derive(Deserialize)]
struct HfResponse {
    generated_text: String,
}

#[async_trait]
impl GrassLlmProvider for GrassHuggingFaceProvider {
    fn name(&self) -> &str {
        "huggingface"
    }

    async fn complete(&self, request: GrassLlmRequest) -> Result<GrassLlmResponse> {
        let prompt = request
            .messages
            .iter()
            .map(|m| format!("{}: {}", format!("{:?}", m.role), m.content))
            .collect::<Vec<_>>()
            .join("\n");

        let body = HfRequest {
            inputs: prompt,
            parameters: HfParameters {
                max_new_tokens: request.max_tokens,
                temperature: request.temperature,
                return_full_text: false,
            },
        };

        let model = request.model.clone();
        let resp = self
            .http
            .post(format!("{}/models/{}", self.base_url, model))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| GrassError::Llm(format!("huggingface failed: {}", e)))?;

        let text = resp
            .text()
            .await
            .map_err(|e| GrassError::Llm(format!("huggingface read: {}", e)))?;

        let content = if let Ok(arr) = serde_json::from_str::<Vec<HfResponse>>(&text) {
            arr.first()
                .map(|r| r.generated_text.clone())
                .unwrap_or_default()
        } else {
            text
        };

        Ok(GrassLlmResponse {
            id: uuid::Uuid::new_v4().to_string(),
            model,
            choices: vec![GrassChoice {
                index: 0,
                message: GrassChatMessage {
                    role: GrassMessageRole::Assistant,
                    content,
                    name: None,
                },
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
        _request: GrassLlmRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<GrassStreamEvent>> + Send + 'a>> {
        Err(GrassError::Llm(
            "HuggingFace Inference API does not support streaming".into(),
        ))
    }
}
