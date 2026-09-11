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
use grass_core::Result;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrassQmqpMessage {
    pub sender: String,
    pub recipients: Vec<String>,
    pub body: Vec<u8>,
    pub attributes: std::collections::HashMap<String, String>,
}

#[async_trait]
pub trait GrassQmqpClient: Send + Sync {
    async fn connect(&self, host: &str, port: u16) -> Result<()>;
    async fn send(&self, message: &GrassQmqpMessage) -> Result<()>;
    async fn receive(&self) -> Result<Option<GrassQmqpMessage>>;
    async fn disconnect(&self) -> Result<()>;
}

pub struct GrassQmqpAdapter {
    connected: std::sync::atomic::AtomicBool,
}

impl GrassQmqpAdapter {
    pub fn new() -> Self {
        Self {
            connected: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl Default for GrassQmqpAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GrassQmqpClient for GrassQmqpAdapter {
    async fn connect(&self, host: &str, port: u16) -> Result<()> {
        tracing::info!("QMQP connecting to {}:{}", host, port);
        self.connected
            .store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    async fn send(&self, message: &GrassQmqpMessage) -> Result<()> {
        tracing::debug!("QMQP send to {:?}", message.recipients);
        Ok(())
    }

    async fn receive(&self) -> Result<Option<GrassQmqpMessage>> {
        Ok(None)
    }

    async fn disconnect(&self) -> Result<()> {
        self.connected
            .store(false, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qmqp_message() {
        let msg = GrassQmqpMessage {
            sender: "agent@grass.ai".into(),
            recipients: vec!["user@example.com".into()],
            body: b"hello".to_vec(),
            attributes: std::collections::HashMap::new(),
        };
        assert_eq!(msg.recipients.len(), 1);
    }
}
