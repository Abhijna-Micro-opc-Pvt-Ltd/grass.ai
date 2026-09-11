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

use grass_core::{GrassPlugin, PluginId, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct GrassPluginRegistry {
    plugins: Arc<RwLock<HashMap<PluginId, Arc<Box<dyn GrassPlugin>>>>>,
    name_map: Arc<RwLock<HashMap<String, PluginId>>>,
}

impl GrassPluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            name_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(
        &self,
        id: PluginId,
        plugin: Box<dyn GrassPlugin>,
        name: &str,
    ) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        let mut name_map = self.name_map.write().await;
        plugins.insert(id, Arc::new(plugin));
        name_map.insert(name.to_string(), id);
        tracing::info!("plugin '{}' registered with id {}", name, id);
        Ok(())
    }

    pub async fn unregister(&self, id: PluginId) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        let mut name_map = self.name_map.write().await;
        plugins.remove(&id);
        name_map.retain(|_, v| *v != id);
        Ok(())
    }

    pub async fn get(&self, name: &str) -> Option<Arc<Box<dyn GrassPlugin>>> {
        let name_map = self.name_map.read().await;
        let id = name_map.get(name)?;
        let plugins = self.plugins.read().await;
        plugins.get(id).cloned()
    }

    pub async fn list(&self) -> Vec<String> {
        self.name_map.read().await.keys().cloned().collect()
    }
}

impl Default for GrassPluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_registry() {
        let reg = GrassPluginRegistry::new();
        assert!(reg.list().await.is_empty());
    }
}
