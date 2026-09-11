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

use super::GrassIpcTransport;
use async_trait::async_trait;
use grass_core::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct GrassShmTransport {
    segments: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    connected: std::sync::atomic::AtomicBool,
}

impl GrassShmTransport {
    pub fn new() -> Self {
        Self {
            segments: Arc::new(RwLock::new(HashMap::new())),
            connected: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub async fn write_segment(&self, name: &str, data: &[u8]) -> Result<()> {
        let mut segments = self.segments.write().await;
        segments.insert(name.to_string(), data.to_vec());
        Ok(())
    }

    pub async fn read_segment(&self, name: &str) -> Result<Option<Vec<u8>>> {
        let segments = self.segments.read().await;
        Ok(segments.get(name).cloned())
    }
}

impl Default for GrassShmTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GrassIpcTransport for GrassShmTransport {
    async fn connect(&self, _endpoint: &str) -> Result<()> {
        self.connected
            .store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    async fn send(&self, data: &[u8]) -> Result<()> {
        let key = uuid::Uuid::new_v4().to_string();
        self.write_segment(&key, data).await
    }

    async fn receive(&self) -> Result<Vec<u8>> {
        let segments = self.segments.read().await;
        Ok(segments.values().next().cloned().unwrap_or_default())
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
    async fn test_shm_transport() {
        let transport = GrassShmTransport::new();
        transport.connect("").await.unwrap();
        transport.write_segment("seg1", b"hello").await.unwrap();
        let val = transport.read_segment("seg1").await.unwrap();
        assert_eq!(val, Some(b"hello".to_vec()));
    }
}
