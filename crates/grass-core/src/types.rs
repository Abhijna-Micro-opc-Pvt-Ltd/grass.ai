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

use std::collections::HashMap;
use uuid::Uuid;

pub type AgentId = Uuid;
pub type TaskId = Uuid;
pub type PluginId = Uuid;
pub type HookId = Uuid;
pub type MessageId = Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
pub enum GrassAgentState {
    Idle,
    Running,
    Waiting,
    Paused,
    Failed,
    Completed,
    Sandbox,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrassAgentMetadata {
    pub id: AgentId,
    pub name: String,
    pub version: String,
    pub description: String,
    pub tags: Vec<String>,
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrassTaskPayload {
    pub task_id: TaskId,
    pub agent_id: AgentId,
    pub action: String,
    pub input: serde_json::Value,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrassTaskResult {
    pub task_id: TaskId,
    pub agent_id: AgentId,
    pub status: GrassTaskStatus,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum GrassTaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrassSandboxConfig {
    pub backend: GrassSandboxBackend,
    pub rootfs: Option<String>,
    pub max_memory_bytes: usize,
    pub max_cpu_shares: u64,
    pub network_isolation: bool,
    pub allowed_syscalls: Vec<String>,
    pub env_vars: HashMap<String, String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum GrassSandboxBackend {
    Docker,
    Kata,
    OCI,
    WASM,
    Native,
}

impl Default for GrassSandboxConfig {
    fn default() -> Self {
        Self {
            backend: GrassSandboxBackend::Docker,
            rootfs: None,
            max_memory_bytes: 512 * 1024 * 1024,
            max_cpu_shares: 1024,
            network_isolation: true,
            allowed_syscalls: vec![],
            env_vars: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config_default() {
        let cfg = GrassSandboxConfig::default();
        assert_eq!(cfg.backend, GrassSandboxBackend::Docker);
        assert!(cfg.network_isolation);
        assert_eq!(cfg.max_memory_bytes, 512 * 1024 * 1024);
    }

    #[test]
    fn test_task_payload_serialization() {
        let payload = GrassTaskPayload {
            task_id: uuid::Uuid::new_v4(),
            agent_id: uuid::Uuid::new_v4(),
            action: "test".into(),
            input: serde_json::json!({"key": "value"}),
            metadata: std::collections::HashMap::new(),
        };
        let json = serde_json::to_string(&payload).unwrap();
        let _deserialized: GrassTaskPayload = serde_json::from_str(&json).unwrap();
    }
}
