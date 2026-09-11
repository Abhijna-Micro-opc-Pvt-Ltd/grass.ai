# grass.ai API Reference

## grass-core

### Types

```rust
pub type AgentId = Uuid;
pub type TaskId = Uuid;
pub type PluginId = Uuid;
pub type HookId = Uuid;
pub type MessageId = Uuid;

pub enum GrassAgentState { Idle, Running, Waiting, Paused, Failed, Completed, Sandbox }
pub enum GrassTaskStatus { Pending, Running, Completed, Failed, Cancelled }
pub enum GrassSandboxBackend { Docker, Kata, OCI, WASM, Native }

pub struct GrassAgentMetadata { id, name, version, description, tags, config }
pub struct GrassTaskPayload { task_id, agent_id, action, input, metadata }
pub struct GrassTaskResult { task_id, agent_id, status, output, error, duration_ms }
pub struct GrassSandboxConfig { backend, rootfs, max_memory_bytes, ... }
pub struct GrassMessage { id, from, to, topic, payload, headers, timestamp }
pub struct GrassAgentContext { agent_id, metadata, sandbox, shared_state }

pub enum GrassEvent {
    AgentStarted, AgentStopped, AgentFailed,
    TaskCreated, TaskStarted, TaskCompleted, TaskFailed,
    HookFired, PluginLoaded, PluginUnloaded,
    MessageSent, SandboxCreated, SandboxDestroyed, Custom
}
```

### Traits

```rust
pub trait GrassAgent: Send + Sync {
    fn metadata(&self) -> &GrassAgentMetadata;
    fn state(&self) -> GrassAgentState;
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    async fn execute(&self, task: GrassTaskPayload) -> Result<GrassTaskResult>;
    async fn handle_message(&self, msg: GrassMessage) -> Result<Option<GrassMessage>>;
}

pub trait GrassPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    async fn initialize(&mut self) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;
    fn capabilities(&self) -> Vec<String>;
}

pub trait GrassRuntime: Send + Sync {
    async fn spawn<F>(&self, name: &str, future: F) -> Result<()>;
    async fn spawn_blocking<F>(&self, name: &str, f: F) -> Result<()>;
    async fn shutdown(&self) -> Result<()>;
}
```

### Builder

```rust
GrassAgentBuilder::new("name")
    .version("1.0.0")
    .description("desc")
    .tag("tag")
    .config("key", value)
    .build() -> GrassAgentMetadata
```

---

## grass-llm

### Types

```rust
pub struct GrassLlmRequest { model, messages, temperature, max_tokens, stream, tools }
pub struct GrassLlmResponse { id, model, choices, usage }
pub struct GrassChatMessage { role, content, name }
pub struct GrassChoice { index, message, finish_reason }
pub struct GrassUsage { prompt_tokens, completion_tokens, total_tokens }
pub struct GrassToolDefinition { name, description, parameters }
pub struct GrassToolCall { id, name, arguments }
pub enum GrassStreamEvent { Delta, ToolCall, Done, Error }
pub enum GrassMessageRole { System, User, Assistant, Tool }
```

### Traits

```rust
pub trait GrassLlmProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn complete(&self, request: GrassLlmRequest) -> Result<GrassLlmResponse>;
    async fn stream<'a>(&'a self, request: GrassLlmRequest)
        -> Result<Box<dyn Stream<Item = Result<GrassStreamEvent>> + Send + 'a>>;
}
```

### Providers

| Struct | Provider | Streaming |
|--------|----------|-----------|
| `GrassOpenAiProvider` | OpenAI | Yes |
| `GrassAnthropicProvider` | Anthropic | Yes |
| `GrassOllamaProvider` | Ollama | Yes |
| `GrassOpenRouterProvider` | OpenRouter | Yes |
| `GrassHuggingFaceProvider` | HuggingFace | No |
| `GrassPerplexityProvider` | Perplexity | Yes |
| `GrassOpenCodeProvider` | OpenCode | Yes |
| `GrassGroqProvider` | Groq | Yes |
| `GrassMistralProvider` | Mistral | Yes |
| `GrassCohereProvider` | Cohere | No |
| `GrassGeminiProvider` | Google Gemini | No |
| `GrassAzureOpenAiProvider` | Azure OpenAI | Yes |
| `GrassSarvamProvider` | Sarvam.ai | Yes |
| `GrassCustomProvider` | Custom | Configurable |

---

## grass-hooks

### Types

```rust
pub enum GrassHookTrigger { OnAgentStart, OnAgentStop, OnTaskComplete, OnTaskFailed, OnMessage, OnCustom }
pub struct GrassHookPayload { trigger, data, metadata }
```

### Traits

```rust
pub trait GrassHook: Send + Sync {
    fn id(&self) -> HookId;
    fn name(&self) -> &str;
    fn trigger(&self) -> GrassHookTrigger;
    async fn execute(&self, payload: &GrassHookPayload) -> Result<serde_json::Value>;
}
```

### Plugins

| Struct | Service |
|--------|---------|
| `GrassSlackHook` | Slack webhooks |
| `GrassDiscordHook` | Discord webhooks |
| `GrassWebhookHook` | Generic HTTP webhooks |

---

## grass-ipc

### Traits

```rust
pub trait GrassIpcTransport: Send + Sync {
    async fn connect(&self, endpoint: &str) -> Result<()>;
    async fn send(&self, data: &[u8]) -> Result<()>;
    async fn receive(&self) -> Result<Vec<u8>>;
    async fn disconnect(&self) -> Result<()>;
}
```

### Implementations

| Struct | Transport |
|--------|-----------|
| `GrassZeroMqTransport` | ZeroMQ (IPC/TCP) |
| `GrassValkeyTransport` | Valkey/Redis |
| `GrassShmTransport` | Shared Memory |

---

## grass-sandbox

### Traits

```rust
pub trait GrassSandboxBackend: Send + Sync {
    fn name(&self) -> &str;
    async fn create(&self, config: &GrassSandboxConfig) -> Result<String>;
    async fn start(&self, container_id: &str) -> Result<()>;
    async fn stop(&self, container_id: &str) -> Result<()>;
    async fn destroy(&self, container_id: &str) -> Result<()>;
    async fn exec(&self, container_id: &str, command: &str) -> Result<String>;
    async fn status(&self, container_id: &str) -> Result<GrassContainerStatus>;
}
```

### Implementations

| Struct | Backend |
|--------|---------|
| `GrassDockerSandbox` | Docker |
| `GrassKataSandbox` | Kata Containers |
| `GrassOciSandbox` | OCI runtime |
| `GrassWasmSandbox` | WebAssembly |

---

## grass-orchestrator

### Types

```rust
pub struct GrassTaskGraph { nodes, edges }
pub struct GrassTaskNode { id, task, task_type, status }
pub struct GrassCrewManager { agents, assignments }
pub struct GrassTaskManager { tasks, queues }
pub enum GrassTaskType { Sequential, Parallel, Conditional, Loop }
pub enum GrassPriority { Low, Normal, High, Critical }
pub struct GrassScheduledTask { task, priority }
```

---

## grass-vector

### Traits

```rust
pub trait GrassVectorStore: Send + Sync {
    fn name(&self) -> &str;
    async fn create_collection(&self, name: &str) -> Result<()>;
    async fn insert(&self, collection: &str, record: GrassVectorRecord) -> Result<()>;
    async fn query(&self, collection: &str, embedding: &[f32], n: usize) -> Result<GrassQueryResult>;
    async fn delete(&self, collection: &str, id: &str) -> Result<()>;
    async fn count(&self, collection: &str) -> Result<usize>;
}
```

---

## grass-inference

### Traits

```rust
pub trait GrassInferenceBackend: Send + Sync {
    fn name(&self) -> &str;
    async fn load_model(&mut self, model_path: &str) -> Result<()>;
    async fn infer(&self, input: &GrassTensor) -> Result<GrassTensor>;
    fn input_shape(&self) -> Vec<usize>;
    fn output_shape(&self) -> Vec<usize>;
}
```

### Types

```rust
pub struct GrassTensor { pub data: Vec<f32>, pub shape: Vec<usize> }
```

---

## grass-iot

### Traits

```rust
pub trait GrassMqttClient: Send + Sync {
    async fn connect(&self, broker: &str, port: u16) -> Result<()>;
    async fn subscribe(&self, topic: &str) -> Result<()>;
    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()>;
    async fn disconnect(&self) -> Result<()>;
}

pub trait GrassModbusTcpClient: Send + Sync {
    async fn connect(&self, host: &str, port: u16) -> Result<()>;
    async fn read_register(&self, slave_id: u8, address: u16, function: GrassModbusFunction) -> Result<u16>;
    async fn write_register(&self, slave_id: u8, address: u16, value: u16) -> Result<()>;
    async fn disconnect(&self) -> Result<()>;
}
```

---

## grass-blockchain

### Traits

```rust
pub trait GrassEthProvider: Send + Sync {
    async fn get_block_number(&self) -> Result<u64>;
    async fn get_block(&self, number: u64) -> Result<GrassEthBlock>;
    async fn get_transaction(&self, hash: &str) -> Result<GrassEthTransaction>;
    async fn send_raw_transaction(&self, data: &str) -> Result<String>;
}
```

---

## grass-protocol

### Types

```rust
pub struct GrassMcpRequest { jsonrpc, method, params, id }
pub struct GrassMcpResponse { jsonrpc, result, error, id }
pub struct GrassAcpMessage { source, destination, content_type, payload, metadata }
```

---

## grass-integrations

### Traits

```rust
pub trait GrassOndcClient: Send + Sync {
    async fn search(&self, query: &str, location: &str) -> Result<serde_json::Value>;
    async fn select(&self, order: &serde_json::Value) -> Result<serde_json::Value>;
    async fn confirm(&self, order_id: &str) -> Result<serde_json::Value>;
}

pub trait GrassIpfsClient: Send + Sync {
    async fn add(&self, data: &[u8]) -> Result<String>;
    async fn get(&self, cid: &str) -> Result<Vec<u8>>;
    async fn pin(&self, cid: &str) -> Result<()>;
}
```

---

## grass-flow

### Traits

```rust
pub trait GrassN8nClient: Send + Sync {
    async fn trigger_workflow(&self, workflow_id: &str, data: serde_json::Value) -> Result<serde_json::Value>;
    async fn list_workflows(&self) -> Result<Vec<serde_json::Value>>;
}

pub trait GrassMakeComClient: Send + Sync {
    async fn trigger_scenario(&self, scenario_id: &str, data: serde_json::Value) -> Result<serde_json::Value>;
}
```

---

## grass-messaging

### Traits

```rust
pub trait GrassKafkaProducer: Send + Sync {
    async fn produce(&self, topic: &str, key: Option<&str>, value: &[u8]) -> Result<()>;
    async fn flush(&self) -> Result<()>;
}

pub trait GrassKafkaConsumer: Send + Sync {
    async fn subscribe(&self, topics: &[&str]) -> Result<()>;
    async fn poll(&self, timeout_ms: u64) -> Result<Option<GrassKafkaMessage>>;
    async fn commit(&self) -> Result<()>;
}
```

---

## Error Handling

All crates use `GrassError` from `grass-core`:

```rust
pub enum GrassError {
    Agent(String), Task(String), Plugin(String), Hook(String),
    Protocol(String), Ipc(String), Sandbox(String), Llm(String),
    Inference(String), Blockchain(String), Iot(String), Streaming(String),
    VectorDb(String), Integration(String), Flow(String), Messaging(String),
    Serialization(serde_json::Error), Io(std::io::Error),
    Timeout(u64), NotFound(String), PermissionDenied(String), Internal(String),
}
```
