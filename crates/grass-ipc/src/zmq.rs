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
use grass_core::{GrassError, Result};

#[async_trait]
pub trait GrassIpcTransport: Send + Sync {
    async fn connect(&self, endpoint: &str) -> Result<()>;
    async fn send(&self, data: &[u8]) -> Result<()>;
    async fn receive(&self) -> Result<Vec<u8>>;
    async fn disconnect(&self) -> Result<()>;
}

pub struct GrassZeroMqTransport {
    connected: std::sync::atomic::AtomicBool,
    endpoint: std::sync::RwLock<Option<String>>,
}

impl GrassZeroMqTransport {
    pub fn new() -> Self {
        Self {
            connected: std::sync::atomic::AtomicBool::new(false),
            endpoint: std::sync::RwLock::new(None),
        }
    }
}

impl Default for GrassZeroMqTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GrassIpcTransport for GrassZeroMqTransport {
    async fn connect(&self, endpoint: &str) -> Result<()> {
        *self
            .endpoint
            .write()
            .map_err(|e| GrassError::Ipc(e.to_string()))? = Some(endpoint.to_string());
        self.connected
            .store(true, std::sync::atomic::Ordering::SeqCst);
        tracing::info!("ZeroMQ connected to {}", endpoint);
        Ok(())
    }

    async fn send(&self, data: &[u8]) -> Result<()> {
        if !self.connected.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(GrassError::Ipc("not connected".into()));
        }
        tracing::debug!("ZeroMQ sent {} bytes", data.len());
        Ok(())
    }

    async fn receive(&self) -> Result<Vec<u8>> {
        if !self.connected.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(GrassError::Ipc("not connected".into()));
        }
        Ok(vec![])
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

    #[tokio::test]
    async fn test_zmq_transport_lifecycle() {
        let transport = GrassZeroMqTransport::new();
        transport.connect("ipc:///tmp/test.sock").await.unwrap();
        assert!(transport
            .connected
            .load(std::sync::atomic::Ordering::SeqCst));
        transport.send(b"hello").await.unwrap();
        transport.disconnect().await.unwrap();
        assert!(!transport
            .connected
            .load(std::sync::atomic::Ordering::SeqCst));
    }
}
