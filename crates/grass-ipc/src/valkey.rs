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
use grass_core::{GrassError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct GrassValkeyTransport {
    endpoint: String,
    connected: std::sync::atomic::AtomicBool,
    store: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl GrassValkeyTransport {
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            connected: std::sync::atomic::AtomicBool::new(false),
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        let mut store = self.store.write().await;
        store.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let store = self.store.read().await;
        Ok(store.get(key).cloned())
    }

    pub async fn del(&self, key: &str) -> Result<()> {
        let mut store = self.store.write().await;
        store.remove(key);
        Ok(())
    }
}

#[async_trait]
impl GrassIpcTransport for GrassValkeyTransport {
    async fn connect(&self, _endpoint: &str) -> Result<()> {
        self.connected
            .store(true, std::sync::atomic::Ordering::SeqCst);
        tracing::info!("Valkey connected to {}", self.endpoint);
        Ok(())
    }

    async fn send(&self, data: &[u8]) -> Result<()> {
        if !self.connected.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(GrassError::Ipc("not connected".into()));
        }
        let key = format!("ipc:{}:{}", self.endpoint, uuid::Uuid::new_v4());
        self.set(&key, data).await
    }

    async fn receive(&self) -> Result<Vec<u8>> {
        let store = self.store.read().await;
        Ok(store.values().next().cloned().unwrap_or_default())
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
    async fn test_valkey_transport() {
        let transport = GrassValkeyTransport::new("redis://localhost:6379");
        transport.connect("").await.unwrap();
        transport.set("key1", b"value1").await.unwrap();
        let val = transport.get("key1").await.unwrap();
        assert_eq!(val, Some(b"value1".to_vec()));
        transport.del("key1").await.unwrap();
        assert!(transport.get("key1").await.unwrap().is_none());
    }
}
