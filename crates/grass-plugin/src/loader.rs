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

use grass_core::{GrassError, GrassPluginManifest, Result};

pub struct GrassPluginLoader {
    manifest_dir: String,
}

impl GrassPluginLoader {
    pub fn new(manifest_dir: &str) -> Self {
        Self {
            manifest_dir: manifest_dir.to_string(),
        }
    }

    pub async fn load_manifest(&self, plugin_name: &str) -> Result<GrassPluginManifest> {
        let path = format!("{}/{}.json", self.manifest_dir, plugin_name);
        let data = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| GrassError::Plugin(format!("failed to read {}: {}", path, e)))?;
        let manifest: GrassPluginManifest = serde_json::from_str(&data)
            .map_err(|e| GrassError::Plugin(format!("failed to parse manifest: {}", e)))?;
        Ok(manifest)
    }

    pub async fn list_plugins(&self) -> Result<Vec<String>> {
        let mut entries = tokio::fs::read_dir(&self.manifest_dir)
            .await
            .map_err(|e| GrassError::Plugin(format!("failed to read dir: {}", e)))?;
        let mut plugins = vec![];
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| GrassError::Plugin(format!("entry error: {}", e)))?
        {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    plugins.push(name.trim_end_matches(".json").to_string());
                }
            }
        }
        Ok(plugins)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_loader_creation() {
        let loader = GrassPluginLoader::new("/tmp/plugins");
        assert_eq!(loader.manifest_dir, "/tmp/plugins");
    }
}
