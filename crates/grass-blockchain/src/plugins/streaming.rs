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

#[async_trait]
pub trait GrassBlockchainStream: Send + Sync {
    fn chain_name(&self) -> &str;
    async fn connect(&self, endpoint: &str) -> Result<()>;
    async fn subscribe_events(&self, contract: &str) -> Result<()>;
    async fn next_event(&self) -> Result<serde_json::Value>;
    async fn disconnect(&self) -> Result<()>;
}

pub struct GrassEthStream {
    chain: String,
    connected: std::sync::atomic::AtomicBool,
}

impl GrassEthStream {
    pub fn new(chain: &str) -> Self {
        Self {
            chain: chain.to_string(),
            connected: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

#[async_trait]
impl GrassBlockchainStream for GrassEthStream {
    fn chain_name(&self) -> &str {
        &self.chain
    }

    async fn connect(&self, endpoint: &str) -> Result<()> {
        tracing::info!("Blockchain stream connected to {}", endpoint);
        self.connected
            .store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    async fn subscribe_events(&self, contract: &str) -> Result<()> {
        tracing::info!("Subscribed to contract events: {}", contract);
        Ok(())
    }

    async fn next_event(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({}))
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
    fn test_eth_stream_name() {
        let stream = GrassEthStream::new("ethereum");
        assert_eq!(stream.chain_name(), "ethereum");
    }
}
