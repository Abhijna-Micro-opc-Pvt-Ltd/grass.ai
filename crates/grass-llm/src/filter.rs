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

use crate::provider::{GrassChatMessage, GrassLlmRequest, GrassLlmResponse, GrassMessageRole};

pub trait GrassRequestFilter: Send + Sync {
    fn filter(&self, request: &mut GrassLlmRequest);
}

pub trait GrassResponseFilter: Send + Sync {
    fn filter(&self, response: &mut GrassLlmResponse);
}

pub struct GrassSystemPromptFilter {
    pub system_prompt: String,
}

impl GrassRequestFilter for GrassSystemPromptFilter {
    fn filter(&self, request: &mut GrassLlmRequest) {
        if !request
            .messages
            .iter()
            .any(|m| m.role == GrassMessageRole::System)
        {
            request.messages.insert(
                0,
                GrassChatMessage {
                    role: GrassMessageRole::System,
                    content: self.system_prompt.clone(),
                    name: None,
                },
            );
        }
    }
}

pub struct GrassTokenLimitFilter {
    pub max_tokens: u32,
}

impl GrassResponseFilter for GrassTokenLimitFilter {
    fn filter(&self, response: &mut GrassLlmResponse) {
        if response.usage.total_tokens > self.max_tokens {
            for choice in &mut response.choices {
                choice.message.content.truncate(1000);
            }
        }
    }
}

pub struct GrassContentSanitizerFilter;

impl GrassResponseFilter for GrassContentSanitizerFilter {
    fn filter(&self, response: &mut GrassLlmResponse) {
        for choice in &mut response.choices {
            choice.message.content = choice
                .message
                .content
                .replace("<script>", "")
                .replace("</script>", "");
        }
    }
}

pub struct GrassFilterChain {
    request_filters: Vec<Box<dyn GrassRequestFilter>>,
    response_filters: Vec<Box<dyn GrassResponseFilter>>,
}

impl GrassFilterChain {
    pub fn new() -> Self {
        Self {
            request_filters: vec![],
            response_filters: vec![],
        }
    }

    pub fn add_request_filter(mut self, f: Box<dyn GrassRequestFilter>) -> Self {
        self.request_filters.push(f);
        self
    }

    pub fn add_response_filter(mut self, f: Box<dyn GrassResponseFilter>) -> Self {
        self.response_filters.push(f);
        self
    }

    pub fn apply_request(&self, request: &mut GrassLlmRequest) {
        for f in &self.request_filters {
            f.filter(request);
        }
    }

    pub fn apply_response(&self, response: &mut GrassLlmResponse) {
        for f in &self.response_filters {
            f.filter(response);
        }
    }
}

impl Default for GrassFilterChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt_filter() {
        let filter = GrassSystemPromptFilter {
            system_prompt: "Be helpful".into(),
        };
        let mut req = GrassLlmRequest {
            model: "gpt-4".into(),
            messages: vec![GrassChatMessage {
                role: GrassMessageRole::User,
                content: "hi".into(),
                name: None,
            }],
            temperature: None,
            max_tokens: None,
            stream: false,
            tools: None,
        };
        filter.filter(&mut req);
        assert_eq!(req.messages.len(), 2);
        assert_eq!(req.messages[0].role, GrassMessageRole::System);
    }
}
