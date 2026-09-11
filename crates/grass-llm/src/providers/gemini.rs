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

pub struct GrassGeminiProvider {
    api_key: String,
    base_url: String,
    http: reqwest::Client,
}

impl GrassGeminiProvider {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            http: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    generation_config: Option<GeminiConfig>,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
    role: String,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Serialize)]
struct GeminiConfig {
    temperature: Option<f32>,
    max_output_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    usage_metadata: Option<GeminiUsage>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContentResponse,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct GeminiContentResponse {
    parts: Vec<GeminiPartResponse>,
}

#[derive(Deserialize)]
struct GeminiPartResponse {
    text: String,
}

#[derive(Deserialize)]
struct GeminiUsage {
    prompt_token_count: u32,
    candidates_token_count: u32,
    total_token_count: u32,
}

#[async_trait]
impl GrassLlmProvider for GrassGeminiProvider {
    fn name(&self) -> &str {
        "gemini"
    }

    async fn complete(&self, request: GrassLlmRequest) -> Result<GrassLlmResponse> {
        let contents: Vec<GeminiContent> = request
            .messages
            .iter()
            .map(|m| GeminiContent {
                parts: vec![GeminiPart {
                    text: m.content.clone(),
                }],
                role: match m.role {
                    GrassMessageRole::User => "user".into(),
                    GrassMessageRole::Assistant => "model".into(),
                    _ => "user".into(),
                },
            })
            .collect();

        let body = GeminiRequest {
            contents,
            generation_config: Some(GeminiConfig {
                temperature: request.temperature,
                max_output_tokens: request.max_tokens,
            }),
        };

        let model = request.model.clone();
        let resp = self
            .http
            .post(format!(
                "{}/models/{}:generateContent?key={}",
                self.base_url, model, self.api_key
            ))
            .json(&body)
            .send()
            .await
            .map_err(|e| GrassError::Llm(format!("gemini failed: {}", e)))?;

        let r: GeminiResponse = resp
            .json()
            .await
            .map_err(|e| GrassError::Llm(format!("gemini parse: {}", e)))?;

        let content = r
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .unwrap_or_default();

        let usage = r.usage_metadata.as_ref();
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
                finish_reason: r.candidates.first().and_then(|c| c.finish_reason.clone()),
            }],
            usage: GrassUsage {
                prompt_tokens: usage.map(|u| u.prompt_token_count).unwrap_or(0),
                completion_tokens: usage.map(|u| u.candidates_token_count).unwrap_or(0),
                total_tokens: usage.map(|u| u.total_token_count).unwrap_or(0),
            },
        })
    }

    async fn stream<'a>(
        &'a self,
        _request: GrassLlmRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<GrassStreamEvent>> + Send + 'a>> {
        Err(GrassError::Llm(
            "Gemini streaming not yet implemented".into(),
        ))
    }
}
