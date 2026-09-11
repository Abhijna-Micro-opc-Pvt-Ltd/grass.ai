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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum GrassRtcState {
    New,
    Connecting,
    Connected,
    Disconnected,
    Failed,
}

#[async_trait]
pub trait GrassRtcPeer: Send + Sync {
    async fn create_offer(&self) -> Result<String>;
    async fn set_remote_description(&self, sdp: &str) -> Result<()>;
    async fn add_ice_candidate(&self, candidate: &str) -> Result<()>;
    async fn send_data(&self, data: &[u8]) -> Result<()>;
    async fn close(&self) -> Result<()>;
    fn state(&self) -> GrassRtcState;
}

pub struct GrassWebRtcPeer {
    state: std::sync::RwLock<GrassRtcState>,
}

impl GrassWebRtcPeer {
    pub fn new() -> Self {
        Self {
            state: std::sync::RwLock::new(GrassRtcState::New),
        }
    }
}

impl Default for GrassWebRtcPeer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GrassRtcPeer for GrassWebRtcPeer {
    async fn create_offer(&self) -> Result<String> {
        Ok("v=0\r\no=- 0 0 IN IP4 0.0.0.0\r\n".into())
    }

    async fn set_remote_description(&self, _sdp: &str) -> Result<()> {
        *self
            .state
            .write()
            .map_err(|e| GrassError::Streaming(e.to_string()))? = GrassRtcState::Connected;
        Ok(())
    }

    async fn add_ice_candidate(&self, _candidate: &str) -> Result<()> {
        Ok(())
    }
    async fn send_data(&self, _data: &[u8]) -> Result<()> {
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        *self
            .state
            .write()
            .map_err(|e| GrassError::Streaming(e.to_string()))? = GrassRtcState::Disconnected;
        Ok(())
    }

    fn state(&self) -> GrassRtcState {
        self.state
            .read()
            .map(|s| s.clone())
            .unwrap_or(GrassRtcState::Failed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_webrtc_peer() {
        let peer = GrassWebRtcPeer::new();
        assert!(matches!(peer.state(), GrassRtcState::New));
        let offer = peer.create_offer().await.unwrap();
        assert!(offer.contains("v=0"));
        peer.set_remote_description(&offer).await.unwrap();
        assert!(matches!(peer.state(), GrassRtcState::Connected));
    }
}
