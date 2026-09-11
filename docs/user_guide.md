#  User Guide

## Table of Contents

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [Core Concepts](#core-concepts)
5. [Agents](#agents)
6. [LLM Providers](#llm-providers)
7. [Notifications](#notifications)
8. [Sensor Monitoring (OT)](#sensor-monitoring)
9. [Data Processing](#data-processing)
10. [Configuration](#configuration)
11. [Examples](#examples)
12. [FAQ](#faq)

---

## Introduction

 is a Rust-based agentic AI framework for building intelligent automation systems. It provides:

- **Multi-agent coordination** with planner, manager, and worker agents
- **12+ LLM providers** with a unified API
- **Notification hooks** for Slack, Discord, and custom webhooks
- **OT/Manufacturing integration** with Modbus/MQTT sensor support
- **Task orchestration** with parallel execution and dependency graphs
- **Sandboxing** with Docker, Kata, OCI, and WASM isolation

---

## Installation

### From Source

```bash
git clone https://github.com/abhijna-micro-opc-pvt-ltd/grass.ai
cd grass.ai
cargo build --release
```

### As a Dependency

```toml
[dependencies]
grass-core = { path = "crates/grass-core" }
grass-llm = { path = "crates/grass-llm" }
grass-hooks = { path = "crates/grass-hooks" }
```

---

## Quick Start

### 1. Create an Agent

```rust
use grass_core::{GrassAgentBuilder, GrassTaskPayload, GrassTaskStatus};

let metadata = GrassAgentBuilder::new("my-agent")
    .version("1.0.0")
    .description("My first agent")
    .tag("starter")
    .build();

println!("Agent: {} v{}", metadata.name, metadata.version);
```

### 2. Use an LLM

```rust
use grass_llm::{GrassLlmService, GrassOpenAiProvider, GrassLlmRequest, GrassChatMessage, GrassMessageRole};

let mut service = GrassLlmService::new();
service.register_provider("openai", Box::new(GrassOpenAiProvider::new("sk-...")));

let response = service.complete(GrassLlmRequest {
    model: "gpt-4".into(),
    messages: vec![GrassChatMessage {
        role: GrassMessageRole::User,
        content: "What is Rust?".into(),
        name: None,
    }],
    temperature: Some(0.7),
    max_tokens: Some(200),
    stream: false,
    tools: None,
}).await?;

println!("Response: {}", response.choices[0].message.content);
```

### 3. Send Notifications

```rust
use grass_hooks::{GrassSlackHook, GrassHookRegistry, GrassHookPayload, GrassHookTrigger};
use std::sync::Arc;

let registry = GrassHookRegistry::new();
registry.register(Arc::new(GrassSlackHook::new(
    "https://hooks.slack.com/services/YOUR/WEBHOOK",
    "#alerts",
))).await;

registry.fire(&GrassHookTrigger::OnTaskComplete, &GrassHookPayload {
    trigger: GrassHookTrigger::OnTaskComplete,
    data: serde_json::json!({"message": "Task completed successfully"}),
    metadata: std::collections::HashMap::new(),
}).await?;
```

---

## Core Concepts

### Agents

Agents are autonomous units that perform tasks. Every agent implements:

- `metadata()` - Returns agent info (name, version, tags)
- `state()` - Returns current state (idle, running, etc.)
- `execute()` - Runs a task and returns a result
- `handle_message()` - Processes incoming messages

### Messages

Agents communicate via `GrassMessage`:

```rust
use grass_core::{GrassMessage, AgentId};

let msg = GrassMessage::new(
    sender_id,
    "task.analyze",
    serde_json::json!({"text": "analyze this"}),
).with_header("x-trace-id", "abc123")
 .directed_to(target_agent_id);
```

### Events

The `GrassEventBus` broadcasts system events:

```rust
use grass_core::{GrassEventBus, GrassEvent};

let mut bus = GrassEventBus::new();
bus.subscribe(|event| {
    println!("Event: {:?}", event);
});

bus.publish(&GrassEvent::TaskCompleted { task_id });
```

---

## LLM Providers

### Supported Providers

| Provider | API Key Format | Base URL |
|----------|---------------|----------|
| OpenAI | `sk-...` | api.openai.com |
| Anthropic | `sk-ant-...` | api.anthropic.com |
| Ollama | N/A (local) | localhost:11434 |
| OpenRouter | `sk-or-...` | openrouter.ai |
| HuggingFace | `hf_...` | api-inference.huggingface.co |
| Perplexity | `pplx-...` | api.perplexity.ai |
| OpenCode | varies | api.opencode.ai |
| Groq | `gsk_...` | api.groq.com |
| Mistral | varies | api.mistral.ai |
| Cohere | varies | api.cohere.ai |
| Gemini | varies | generativelanguage.googleapis.com |
| Azure OpenAI | varies | your-resource.openai.azure.com |
| Sarvam | varies | api.sarvam.ai |

### Provider Selection

```rust
let mut service = GrassLlmService::new();

// Register multiple providers
service.register_provider("openai", Box::new(GrassOpenAiProvider::new("sk-...")));
service.register_provider("anthropic", Box::new(GrassAnthropicProvider::new("sk-ant-...")));
service.register_provider("groq", Box::new(GrassGroqProvider::new("gsk_...")));

// Set default
service.set_default("groq");

// Or use specific provider
let provider = service.provider("anthropic").unwrap();
```

### Request Filters

```rust
use grass_llm::{GrassFilterChain, GrassSystemPromptFilter};

let chain = GrassFilterChain::new()
    .add_request_filter(Box::new(GrassSystemPromptFilter {
        system_prompt: "You are a helpful assistant.".into(),
    }));

let mut request = ...;
chain.apply_request(&mut request);
```

---

## Notifications

### Slack

```rust
use grass_hooks::GrassSlackHook;

let hook = GrassSlackHook::new(
    "https://hooks.slack.com/services/T.../B.../xxx",
    "#manufacturing-alerts",
);
```

### Discord

```rust
use grass_hooks::GrassDiscordHook;

let hook = GrassDiscordHook::new(
    "https://discord.com/api/webhooks/xxx/yyy",
);
```

### Custom Webhook

```rust
use grass_hooks::GrassWebhookHook;

let hook = GrassWebhookHook::new("https://api.example.com/webhook", "POST")
    .with_header("Authorization", "Bearer token123")
    .with_header("X-Custom", "value");
```

---

## Sensor Monitoring (OT Framework)

### Register Sensors

```rust
use grass_iot::{GrassSensorConfig, GrassSensorType};

let sensor = GrassSensorConfig {
    sensor_id: "TEMP-001".into(),
    sensor_type: GrassSensorType::Temperature,
    machine_id: "CNC-001".into(),
    location: "Spindle".into(),
    min_threshold: Some(20.0),
    max_threshold: Some(120.0),
    sampling_rate_ms: 1000,
};

ot_agent.register_sensor(sensor).await;
```

### Poll Sensors

```rust
let result = ot_agent.execute(GrassTaskPayload {
    task_id: Uuid::new_v4(),
    agent_id: ot_agent.metadata().id,
    action: "poll_sensors".into(),
    input: serde_json::json!({}),
    metadata: HashMap::new(),
}).await?;

// Result contains readings and alarms
let readings = result.output["readings"];
let alarms = result.output["alarms"];
```

### Threshold Alarms

Alarms are automatically generated when sensor values exceed thresholds:

- `THRESHOLD_EXCEEDED` - Value above max
- `THRESHOLD_BELOW` - Value below min

---

## Data Processing

### Agent Pipeline

```rust
use grass_core::GrassTaskPayload;

// Stage 1: Validate
let validated = validate(input).await?;

// Stage 2: Analyze
let analyzed = analyze(validated).await?;

// Stage 3: Enrich
let enriched = enrich(analyzed).await?;

// Stage 4: Store
store(enriched).await?;
```

### Multi-Agent Communication

```rust
// Send message from agent A to agent B
let msg = GrassMessage::new(agent_a_id, "task.analyze", payload)
    .directed_to(agent_b_id);

// Agent B handles the message
let response = agent_b.handle_message(msg).await?;
```

---

## Configuration

### Workspace Cargo.toml

Key dependencies:

```toml
[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls", "stream"] }
uuid = { version = "1", features = ["v4", "serde"] }
```

### Environment Variables

```bash
RUST_LOG=info                    # Logging level
OPENAI_API_KEY=sk-...           # OpenAI key
ANTHROPIC_API_KEY=sk-ant-...    # Anthropic key
GROQ_API_KEY=gsk_...            # Groq key
```

---

## Examples

### Run All Examples

```bash
# Planner Agent
cargo run --example planner_agent

# Manager Agent
cargo run --example manager_agent

# Multi-Agent Communication
cargo run --example multi_agent_json_comm

# ChromaDB Integration
cargo run --example chromadb_integration

# Agent Pipeline
cargo run --example agent_pipeline

# OT Manufacturing
cargo run --example ot_manufacturing_simple
```

---

## FAQ

### Q: How do I add a custom LLM provider?

A: Implement the `GrassLlmProvider` trait:

```rust
struct MyCustomProvider { api_key: String }

#[async_trait]
impl GrassLlmProvider for MyCustomProvider {
    fn name(&self) -> &str { "custom" }
    async fn complete(&self, request: GrassLlmRequest) -> Result<GrassLlmResponse> { ... }
    async fn stream<'a>(&'a self, request: GrassLlmRequest) -> Result<Box<dyn Stream<...>>> { ... }
}
```

### Q: How do I send data to Slack?

A: Use `GrassSlackHook` from `grass-hooks`:

```rust
let hook = GrassSlackHook::new("webhook-url", "#channel");
hook.execute(&GrassHookPayload { ... }).await?;
```

### Q: How do I run agents in containers?

A: Use the sandbox crate:

```rust
let docker = GrassDockerSandbox::new("/var/run/docker.sock");
let id = docker.create(&GrassSandboxConfig::default()).await?;
docker.start(&id).await?;
```

### Q: How do I store vector embeddings?

A: Use ChromaDB:

```rust
let store = GrassChromaDb::new("http://localhost:8000");
store.create_collection("my_collection").await?;
store.insert("my_collection", record).await?;
let results = store.query("my_collection", &embedding, 10).await?;
```
