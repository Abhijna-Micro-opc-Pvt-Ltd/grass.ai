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
use std::collections::HashMap;

pub struct GrassCustomProvider {
    name: String,
    base_url: String,
    headers: HashMap<String, String>,
    http: reqwest::Client,
    response_parser: fn(&str) -> Result<GrassLlmResponse>,
}

impl GrassCustomProvider {
    pub fn new(
        name: &str,
        base_url: &str,
        response_parser: fn(&str) -> Result<GrassLlmResponse>,
    ) -> Self {
        Self {
            name: name.to_string(),
            base_url: base_url.to_string(),
            headers: HashMap::new(),
            http: reqwest::Client::new(),
            response_parser,
        }
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
}

#[async_trait]
impl GrassLlmProvider for GrassCustomProvider {
    fn name(&self) -> &str {
        &self.name
    }

    async fn complete(&self, request: GrassLlmRequest) -> Result<GrassLlmResponse> {
        let mut req = self
            .http
            .post(&self.base_url)
            .header("Content-Type", "application/json");
        for (k, v) in &self.headers {
            req = req.header(k, v);
        }
        let body = serde_json::to_string(&request)
            .map_err(|e| GrassError::Llm(format!("serialize: {}", e)))?;
        let resp = req
            .body(body)
            .send()
            .await
            .map_err(|e| GrassError::Llm(format!("request: {}", e)))?;
        let text = resp
            .text()
            .await
            .map_err(|e| GrassError::Llm(format!("read body: {}", e)))?;
        (self.response_parser)(&text)
    }

    async fn stream<'a>(
        &'a self,
        _request: GrassLlmRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<GrassStreamEvent>> + Send + 'a>> {
        Err(GrassError::Llm(
            "streaming not supported by custom provider".into(),
        ))
    }
}
