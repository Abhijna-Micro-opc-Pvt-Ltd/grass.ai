# grass.ai Developer Guide

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Getting Started](#getting-started)
3. [Crate Reference](#crate-reference)
4. [Creating Agents](#creating-agents)
5. [LLM Integration](#llm-integration)
6. [Hook System](#hook-system)
7. [Protocol Adapters](#protocol-adapters)
8. [IPC Transport](#ipc-transport)
9. [Sandboxing](#sandboxing)
10. [Plugin System](#plugin-system)
11. [Inference Engine](#inference-engine)
12. [IoT Integration](#iot-integration)
13. [Vector Database](#vector-database)
14. [Task Orchestration](#task-orchestration)
15. [Examples](#examples)

---

## Architecture Overview

grass.ai is a modular Rust framework for building agentic AI infrastructure. The workspace contains 18 specialized crates:

```
grass-core          -> Foundational types, traits, errors
grass-runtime       -> Async executor, task scheduling
grass-llm           -> LLM provider abstraction (12 providers)
grass-hooks         -> Hook registry + notification plugins
grass-protocol      -> MCP, ACP, gRPC codec
grass-ipc           -> ZeroMQ, Valkey, shared memory
grass-sandbox       -> Docker, Kata, OCI, WASM
grass-plugin        -> Dynamic plugin loader/registry
grass-inference     -> TensorRT, ONNX Runtime
grass-iot           -> MQTT, Modbus TCP
grass-streaming     -> RTSP, WebRTC
grass-vector        -> ChromaDB vector store
grass-blockchain    -> Ethereum, blockchain streaming
grass-integrations  -> ONDC, IPFS, FISS
grass-flow          -> N8N, Make.com webhooks
grass-messaging     -> Kafka, QMQP
grass-orchestrator  -> Task graph, crew, manager
grass-binary        -> Main entry point
```

### Dependency Graph

```
                    grass-core
                       |
    +---------+--------+--------+---------+
    |         |        |        |         |
 runtime    llm    hooks   protocol    ipc
    |         |        |        |         |
    +---------+--------+--------+---------+
                       |
         +-------------+-------------+
         |             |             |
     sandbox       plugin     orchestrator
         |             |             |
     inference       iot        blockchain
         |             |             |
     streaming     vector     integrations
         |             |             |
         +------+------+------+------+
                |             |
              flow        messaging
                |             |
                +------+------+
                       |
                   binary
```

---

## Getting Started

### Prerequisites

- Rust 1.75+ (installed via rustup)
- No system OpenSSL required (uses rustls-tls)

### Build

```bash
cargo build --workspace
```

### Run Tests

```bash
cargo test --workspace
```

### Run Examples

```bash
cargo run --example planner_agent
cargo run --example manager_agent
cargo run --example multi_agent_json_comm
cargo run --example chromadb_integration
cargo run --example agent_pipeline
cargo run --example ot_manufacturing_simple
```

---

## Creating Agents

All agents implement the `GrassAgent` trait from `grass-core`:

```rust
use async_trait::async_trait;
use grass_core::{
    GrassAgent, GrassAgentMetadata, GrassAgentState,
    GrassAgentBuilder, GrassTaskPayload, GrassTaskResult,
    GrassTaskStatus, GrassMessage, Result,
};

struct MyAgent {
    metadata: GrassAgentMetadata,
    state: GrassAgentState,
}

impl MyAgent {
    fn new() -> Self {
        Self {
            metadata: GrassAgentBuilder::new("my-agent")
                .version("0.1.0")
                .description("My custom agent")
                .tag("custom")
                .build(),
            state: GrassAgentState::Idle,
        }
    }
}

#[async_trait]
impl GrassAgent for MyAgent {
    fn metadata(&self) -> &GrassAgentMetadata { &self.metadata }
    fn state(&self) -> GrassAgentState { self.state.clone() }

    async fn start(&mut self) -> Result<()> {
        self.state = GrassAgentState::Idle;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.state = GrassAgentState::Idle;
        Ok(())
    }

    async fn execute(&self, task: GrassTaskPayload) -> Result<GrassTaskResult> {
        Ok(GrassTaskResult {
            task_id: task.task_id,
            agent_id: task.agent_id,
            status: GrassTaskStatus::Completed,
            output: serde_json::json!({"result": "done"}),
            error: None,
            duration_ms: 0,
        })
    }

    async fn handle_message(&self, msg: GrassMessage) -> Result<Option<GrassMessage>> {
        Ok(None)
    }
}
```

### Agent States

| State | Description |
|-------|-------------|
| `Idle` | Agent is initialized but not processing |
| `Running` | Agent is actively processing tasks |
| `Waiting` | Agent is waiting for external input |
| `Paused` | Agent is temporarily suspended |
| `Failed` | Agent encountered an error |
| `Completed` | Agent finished its work |
| `Sandbox` | Agent is running in isolated environment |

---

## LLM Integration

### Supported Providers

| Provider | Crate Module | Models |
|----------|-------------|--------|
| OpenAI | `GrassOpenAiProvider` | gpt-4, gpt-4-turbo, gpt-3.5-turbo |
| Anthropic | `GrassAnthropicProvider` | claude-3-opus, claude-3-sonnet |
| Ollama | `GrassOllamaProvider` | llama2, codellama, mistral |
| OpenRouter | `GrassOpenRouterProvider` | All OpenRouter models |
| HuggingFace | `GrassHuggingFaceProvider` | Any HF Inference model |
| Perplexity | `GrassPerplexityProvider` | pplx-7b, pplx-70b |
| OpenCode | `GrassOpenCodeProvider` | OpenCode models |
| Groq | `GrassGroqProvider` | llama2-70b, mixtral |
| Mistral | `GrassMistralProvider` | mistral-large, mixtral |
| Cohere | `GrassCohereProvider` | command-r, command-r-plus |
| Gemini | `GrassGeminiProvider` | gemini-pro, gemini-ultra |
| Azure OpenAI | `GrassAzureOpenAiProvider` | Any Azure deployed model |
| Sarvam | `GrassSarvamProvider` | Sarvam-2B, Sarvam-7B |

### Usage Example

```rust
use grass_llm::{GrassLlmService, GrassOpenAiProvider, GrassLlmRequest, GrassChatMessage, GrassMessageRole};

let mut service = GrassLlmService::new();
service.register_provider("openai", Box::new(GrassOpenAiProvider::new("sk-...")));
service.register_provider("anthropic", Box::new(GrassAnthropicProvider::new("sk-ant-...")));
service.set_default("openai");

let response = service.complete(GrassLlmRequest {
    model: "gpt-4".into(),
    messages: vec![GrassChatMessage {
        role: GrassMessageRole::User,
        content: "Hello, world!".into(),
        name: None,
    }],
    temperature: Some(0.7),
    max_tokens: Some(100),
    stream: false,
    tools: None,
}).await?;
```

---

## Hook System

```rust
use grass_hooks::{GrassHookRegistry, GrassSlackHook, GrassDiscordHook};
use std::sync::Arc;

let registry = GrassHookRegistry::new();

registry.register(Arc::new(GrassSlackHook::new(
    "https://hooks.slack.com/services/xxx",
    "#alerts",
))).await;

registry.register(Arc::new(GrassDiscordHook::new(
    "https://discord.com/api/webhooks/xxx",
))).await;
```

---

## Sandboxing

```rust
use grass_sandbox::{GrassDockerSandbox, GrassWasmSandbox, GrassWasmConfig};
use grass_core::GrassSandboxConfig;

// Docker
let docker = GrassDockerSandbox::new("/var/run/docker.sock");
let container_id = docker.create(&GrassSandboxConfig::default()).await?;
docker.start(&container_id).await?;

// WASM
let wasm = GrassWasmSandbox::new(GrassWasmConfig::default());
let instance_id = wasm.create(&GrassSandboxConfig::default()).await?;
```

---

## Task Orchestration

```rust
use grass_orchestrator::{GrassTaskGraph, GrassTaskNode, GrassTaskType, GrassCrewManager};

let graph = GrassTaskGraph::new();
graph.add_node(GrassTaskNode { id: "a".into(), ... }).await;
graph.add_node(GrassTaskNode { id: "b".into(), ... }).await;
graph.add_edge("a", "b").await;

let crew = GrassCrewManager::new();
crew.add_agent(metadata).await;
crew.assign_task("summarize", agent_id).await?;
```

---

## Crate Reference

| Crate | Key Types | Purpose |
|-------|-----------|---------|
| `grass-core` | `GrassAgent`, `GrassMessage`, `GrassEventBus` | Foundation |
| `grass-runtime` | `GrassTaskExecutor`, `GrassTaskScheduler` | Execution |
| `grass-llm` | `GrassLlmService`, `GrassLlmProvider` | LLM abstraction |
| `grass-hooks` | `GrassHookRegistry`, `GrassSlackHook` | Notifications |
| `grass-protocol` | `GrassMcpCodec`, `GrassAcpRouter` | Communication |
| `grass-ipc` | `GrassIpcTransport`, `GrassZeroMqTransport` | IPC |
| `grass-sandbox` | `GrassDockerSandbox`, `GrassWasmSandbox` | Isolation |
| `grass-plugin` | `GrassPluginRegistry`, `GrassPluginLoader` | Plugins |
| `grass-inference` | `GrassInferenceBackend`, `GrassTensor` | ML inference |
| `grass-iot` | `GrassMqttClient`, `GrassModbusTcpClient` | IoT |
| `grass-vector` | `GrassVectorStore`, `GrassChromaDb` | Vector DB |
| `grass-blockchain` | `GrassEthProvider`, `GrassBlockchainStream` | Blockchain |
| `grass-integrations` | `GrassOndcClient`, `GrassIpfsClient` | Integrations |
| `grass-flow` | `GrassN8nClient`, `GrassMakeComClient` | Flow editors |
| `grass-messaging` | `GrassKafkaProducer`, `GrassQmqpClient` | Messaging |
| `grass-orchestrator` | `GrassTaskGraph`, `GrassCrewManager` | Orchestration |
