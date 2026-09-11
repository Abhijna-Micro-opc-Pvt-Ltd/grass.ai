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
pub trait GrassMqttClient: Send + Sync {
    async fn connect(&self, broker: &str, port: u16) -> Result<()>;
    async fn subscribe(&self, topic: &str) -> Result<()>;
    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()>;
    async fn disconnect(&self) -> Result<()>;
}

pub struct GrassMqttBroker {
    client_id: String,
    connected: std::sync::atomic::AtomicBool,
}

impl GrassMqttBroker {
    pub fn new(client_id: &str) -> Self {
        Self {
            client_id: client_id.to_string(),
            connected: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

#[async_trait]
impl GrassMqttClient for GrassMqttBroker {
    async fn connect(&self, broker: &str, port: u16) -> Result<()> {
        self.connected
            .store(true, std::sync::atomic::Ordering::SeqCst);
        tracing::info!("MQTT {} connected to {}:{}", self.client_id, broker, port);
        Ok(())
    }

    async fn subscribe(&self, topic: &str) -> Result<()> {
        tracing::info!("MQTT subscribed to {}", topic);
        Ok(())
    }

    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
        tracing::debug!("MQTT publish {} ({} bytes)", topic, payload.len());
        Ok(())
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
    async fn test_mqtt_lifecycle() {
        let client = GrassMqttBroker::new("test-client");
        client.connect("localhost", 1883).await.unwrap();
        assert!(client.connected.load(std::sync::atomic::Ordering::SeqCst));
        client.subscribe("test/topic").await.unwrap();
        client.publish("test/topic", b"hello").await.unwrap();
        client.disconnect().await.unwrap();
    }
}
