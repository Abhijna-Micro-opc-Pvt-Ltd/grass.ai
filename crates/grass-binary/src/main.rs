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

use grass_core::{
    GrassAgentBuilder, GrassEvent, GrassEventBus, GrassSandboxBackend, GrassSandboxConfig, Result,
};
use grass_hooks::GrassHookRegistry;
use grass_ipc::GrassZeroMqTransport;
use grass_protocol::{GrassMcpCodec, GrassMcpRequest};
use grass_runtime::{GrassTaskExecutor, GrassTaskScheduler};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("grass.ai framework starting...");

    let meta = GrassAgentBuilder::new("core-agent")
        .version("0.1.0")
        .description("grass.ai core orchestrator agent")
        .tag("core")
        .tag("orchestrator")
        .build();

    tracing::info!("Agent: {} v{}", meta.name, meta.version);

    let sandbox_cfg = GrassSandboxConfig {
        backend: GrassSandboxBackend::Docker,
        max_memory_bytes: 512 * 1024 * 1024,
        network_isolation: true,
        ..Default::default()
    };

    tracing::info!(
        "Sandbox: {:?} ({}MB)",
        sandbox_cfg.backend,
        sandbox_cfg.max_memory_bytes / 1024 / 1024
    );

    let _executor = GrassTaskExecutor::new(4);
    let _scheduler = GrassTaskScheduler::new(8);

    let mut bus = GrassEventBus::new();
    bus.subscribe(|event| {
        tracing::info!("event: {:?}", event);
    });
    bus.publish(&GrassEvent::AgentStarted { agent_id: meta.id });

    let _registry = GrassHookRegistry::new();

    let request = GrassMcpRequest {
        jsonrpc: "2.0".into(),
        method: "initialize".into(),
        params: serde_json::json!({"name": "grass-ai"}),
        id: 1,
    };
    let encoded = GrassMcpCodec::encode_request(&request)?;
    let _decoded = GrassMcpCodec::decode_request(&encoded)?;

    let _transport = GrassZeroMqTransport::new();

    tracing::info!("grass.ai framework initialized successfully");
    tracing::info!("Registered crates: core, runtime, llm, hooks, protocol, ipc, sandbox, plugin, orchestrator");

    Ok(())
}
