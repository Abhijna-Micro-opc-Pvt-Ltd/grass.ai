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

pub struct GrassCohereProvider {
    api_key: String,
    base_url: String,
    http: reqwest::Client,
}

impl GrassCohereProvider {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            base_url: "https://api.cohere.ai/v1".to_string(),
            http: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct CohereRequest {
    model: String,
    message: String,
    chat_history: Option<Vec<CohereChat>>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

#[derive(Serialize, Deserialize)]
struct CohereChat {
    role: String,
    message: String,
}

#[derive(Deserialize)]
struct CohereResponse {
    text: String,
    generation_id: String,
    meta: Option<CohereMeta>,
}

#[derive(Deserialize)]
struct CohereMeta {
    tokens: Option<CohereTokens>,
}

#[derive(Deserialize)]
struct CohereTokens {
    input_tokens: u32,
    output_tokens: u32,
}

#[async_trait]
impl GrassLlmProvider for GrassCohereProvider {
    fn name(&self) -> &str {
        "cohere"
    }

    async fn complete(&self, request: GrassLlmRequest) -> Result<GrassLlmResponse> {
        let _system_msg = request
            .messages
            .iter()
            .find(|m| m.role == GrassMessageRole::System)
            .map(|m| m.content.clone());

        let user_msg = request
            .messages
            .iter()
            .rev()
            .find(|m| m.role == GrassMessageRole::User)
            .map(|m| m.content.clone())
            .unwrap_or_default();

        let chat_history: Vec<CohereChat> = request
            .messages
            .iter()
            .filter(|m| m.role != GrassMessageRole::System && m.role != GrassMessageRole::User)
            .map(|m| CohereChat {
                role: match m.role {
                    GrassMessageRole::Assistant => "CHATBOT".into(),
                    _ => "USER".into(),
                },
                message: m.content.clone(),
            })
            .collect();

        let body = CohereRequest {
            model: request.model.clone(),
            message: user_msg,
            chat_history: if chat_history.is_empty() {
                None
            } else {
                Some(chat_history)
            },
            temperature: request.temperature,
            max_tokens: request.max_tokens,
        };

        let resp = self
            .http
            .post(format!("{}/chat", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| GrassError::Llm(format!("cohere failed: {}", e)))?;

        let r: CohereResponse = resp
            .json()
            .await
            .map_err(|e| GrassError::Llm(format!("cohere parse: {}", e)))?;

        let (prompt_tokens, completion_tokens) = r
            .meta
            .as_ref()
            .and_then(|m| m.tokens.as_ref())
            .map(|t| (t.input_tokens, t.output_tokens))
            .unwrap_or((0, 0));

        Ok(GrassLlmResponse {
            id: r.generation_id,
            model: request.model.clone(),
            choices: vec![GrassChoice {
                index: 0,
                message: GrassChatMessage {
                    role: GrassMessageRole::Assistant,
                    content: r.text,
                    name: None,
                },
                finish_reason: Some("stop".into()),
            }],
            usage: GrassUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
            },
        })
    }

    async fn stream<'a>(
        &'a self,
        _request: GrassLlmRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<GrassStreamEvent>> + Send + 'a>> {
        Err(GrassError::Llm(
            "Cohere streaming not yet implemented".into(),
        ))
    }
}
