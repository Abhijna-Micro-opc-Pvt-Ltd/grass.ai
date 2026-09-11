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

use super::GrassContainerStatus;
use super::GrassSandboxBackend;
use async_trait::async_trait;
use grass_core::{GrassSandboxConfig, Result};

pub struct GrassWasmSandbox {
    wasmtime_config: GrassWasmConfig,
}

#[derive(Debug, Clone)]
pub struct GrassWasmConfig {
    pub max_memory_pages: u32,
    pub fuel_limit: u64,
    pub enable_simd: bool,
}

impl Default for GrassWasmConfig {
    fn default() -> Self {
        Self {
            max_memory_pages: 256,
            fuel_limit: 1_000_000,
            enable_simd: true,
        }
    }
}

impl GrassWasmSandbox {
    pub fn new(config: GrassWasmConfig) -> Self {
        Self {
            wasmtime_config: config,
        }
    }
}

#[async_trait]
impl GrassSandboxBackend for GrassWasmSandbox {
    fn name(&self) -> &str {
        "wasm"
    }

    async fn create(&self, _config: &GrassSandboxConfig) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        tracing::info!(
            "WASM instance created: {} (fuel={})",
            id,
            self.wasmtime_config.fuel_limit
        );
        Ok(id)
    }

    async fn start(&self, _container_id: &str) -> Result<()> {
        Ok(())
    }
    async fn stop(&self, _container_id: &str) -> Result<()> {
        Ok(())
    }
    async fn destroy(&self, _container_id: &str) -> Result<()> {
        Ok(())
    }

    async fn exec(&self, _container_id: &str, command: &str) -> Result<String> {
        tracing::info!("WASM exec: {}", command);
        Ok(String::new())
    }

    async fn status(&self, _container_id: &str) -> Result<GrassContainerStatus> {
        Ok(GrassContainerStatus::Running)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_sandbox_config_default() {
        let cfg = GrassWasmConfig::default();
        assert_eq!(cfg.max_memory_pages, 256);
        assert_eq!(cfg.fuel_limit, 1_000_000);
        assert!(cfg.enable_simd);
    }
}
