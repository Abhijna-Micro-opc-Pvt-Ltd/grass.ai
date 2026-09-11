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
use grass_core::{GrassError, GrassSandboxConfig, Result};

pub struct GrassDockerSandbox {
    http_base: String,
    http: reqwest::Client,
}

impl GrassDockerSandbox {
    pub fn new(_socket_path: &str) -> Self {
        Self {
            http_base: "http://localhost/v1.43/containers".to_string(),
            http: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl GrassSandboxBackend for GrassDockerSandbox {
    fn name(&self) -> &str {
        "docker"
    }

    async fn create(&self, config: &GrassSandboxConfig) -> Result<String> {
        let body = serde_json::json!({
            "Image": "grass-agent:latest",
            "HostConfig": {
                "Memory": config.max_memory_bytes as i64,
                "CpuShares": config.max_cpu_shares,
                "NetworkMode": if config.network_isolation { "none" } else { "bridge" }
            },
            "Env": config.env_vars.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
        });
        let resp: serde_json::Value = self
            .http
            .post(&self.http_base)
            .json(&body)
            .send()
            .await
            .map_err(|e| GrassError::Sandbox(format!("docker create: {}", e)))?
            .json()
            .await
            .map_err(|e| GrassError::Sandbox(format!("docker parse: {}", e)))?;
        let id = resp["Id"].as_str().unwrap_or("").to_string();
        tracing::info!("docker container created: {}", id);
        Ok(id)
    }

    async fn start(&self, container_id: &str) -> Result<()> {
        self.http
            .post(format!("{}/{}/start", self.http_base, container_id))
            .send()
            .await
            .map_err(|e| GrassError::Sandbox(format!("docker start: {}", e)))?;
        Ok(())
    }

    async fn stop(&self, container_id: &str) -> Result<()> {
        self.http
            .post(format!("{}/{}/stop", self.http_base, container_id))
            .send()
            .await
            .map_err(|e| GrassError::Sandbox(format!("docker stop: {}", e)))?;
        Ok(())
    }

    async fn destroy(&self, container_id: &str) -> Result<()> {
        self.http
            .delete(format!("{}/{}", self.http_base, container_id))
            .send()
            .await
            .map_err(|e| GrassError::Sandbox(format!("docker destroy: {}", e)))?;
        Ok(())
    }

    async fn exec(&self, container_id: &str, command: &str) -> Result<String> {
        let body = serde_json::json!({
            "AttachStdout": true, "AttachStderr": true,
            "Cmd": command.split_whitespace().collect::<Vec<_>>()
        });
        let resp: serde_json::Value = self
            .http
            .post(format!("{}/{}/exec", self.http_base, container_id))
            .json(&body)
            .send()
            .await
            .map_err(|e| GrassError::Sandbox(format!("docker exec: {}", e)))?
            .json()
            .await
            .map_err(|e| GrassError::Sandbox(format!("docker exec parse: {}", e)))?;
        Ok(resp["Id"].as_str().unwrap_or("").to_string())
    }

    async fn status(&self, container_id: &str) -> Result<GrassContainerStatus> {
        let resp: serde_json::Value = self
            .http
            .get(format!("{}/{}", self.http_base, container_id))
            .send()
            .await
            .map_err(|e| GrassError::Sandbox(format!("docker status: {}", e)))?
            .json()
            .await
            .map_err(|e| GrassError::Sandbox(format!("docker status parse: {}", e)))?;
        let state = resp["State"]["Status"].as_str().unwrap_or("unknown");
        match state {
            "created" => Ok(GrassContainerStatus::Created),
            "running" => Ok(GrassContainerStatus::Running),
            "exited" | "stopped" => Ok(GrassContainerStatus::Stopped),
            _ => Ok(GrassContainerStatus::Unknown),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_sandbox_name() {
        let sandbox = GrassDockerSandbox::new("/var/run/docker.sock");
        assert_eq!(sandbox.name(), "docker");
    }
}
