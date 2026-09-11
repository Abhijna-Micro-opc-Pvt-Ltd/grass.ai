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

pub mod docker;
pub mod kata;
pub mod oci;
pub mod wasm;

use async_trait::async_trait;
use grass_core::{GrassSandboxConfig, Result};

pub use docker::*;
pub use kata::*;
pub use oci::*;
pub use wasm::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrassContainerStatus {
    Created,
    Running,
    Stopped,
    Unknown,
}

#[async_trait]
pub trait GrassSandboxBackend: Send + Sync {
    fn name(&self) -> &str;
    async fn create(&self, config: &GrassSandboxConfig) -> Result<String>;
    async fn start(&self, sandbox_id: &str) -> Result<()>;
    async fn stop(&self, sandbox_id: &str) -> Result<()>;
    async fn destroy(&self, sandbox_id: &str) -> Result<()>;
    async fn exec(&self, sandbox_id: &str, command: &str) -> Result<String>;
    async fn status(&self, sandbox_id: &str) -> Result<GrassContainerStatus>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_status_variants() {
        let statuses = vec![
            GrassContainerStatus::Created,
            GrassContainerStatus::Running,
            GrassContainerStatus::Stopped,
            GrassContainerStatus::Unknown,
        ];
        assert_eq!(statuses.len(), 4);
        assert_ne!(GrassContainerStatus::Running, GrassContainerStatus::Stopped);
    }
}
