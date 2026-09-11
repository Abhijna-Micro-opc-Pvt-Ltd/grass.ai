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
pub trait GrassRtspClient: Send + Sync {
    async fn connect(&self, url: &str) -> Result<()>;
    async fn get_frame(&self) -> Result<Vec<u8>>;
    async fn disconnect(&self) -> Result<()>;
}

pub struct GrassRtspStream {
    url: String,
    connected: std::sync::atomic::AtomicBool,
}

impl GrassRtspStream {
    pub fn new() -> Self {
        Self {
            url: String::new(),
            connected: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl Default for GrassRtspStream {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GrassRtspClient for GrassRtspStream {
    async fn connect(&self, url: &str) -> Result<()> {
        tracing::info!("RTSP connecting to {}", url);
        self.connected
            .store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    async fn get_frame(&self) -> Result<Vec<u8>> {
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
    async fn test_rtsp_lifecycle() {
        let stream = GrassRtspStream::new();
        stream.connect("rtsp://camera.local/stream").await.unwrap();
        assert!(stream.connected.load(std::sync::atomic::Ordering::SeqCst));
        stream.disconnect().await.unwrap();
    }
}
