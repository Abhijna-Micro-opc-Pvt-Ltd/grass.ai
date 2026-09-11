# grass.ai Framework Architecture

## Overview

grass.ai is a modular Rust-based agentic AI infrastructure framework providing
sandboxed agent execution, LLM integration, protocol adapters, and extensible
plugin architecture.

## Crate Dependency Graph

```
grass-core (foundational types, traits, errors)
    |
    +-- grass-runtime (async executor, scheduler, timers)
    +-- grass-llm (LLM provider abstraction + providers)
    +-- grass-hooks (hook registry + notification plugins)
    +-- grass-protocol (MCP, ACP, gRPC/Protobuf codec)
    +-- grass-ipc (ZeroMQ, Valkey, shared-memory transport)
    +-- grass-sandbox (Docker, Kata, OCI, WASM isolation)
    +-- grass-plugin (dynamic plugin loader/registry)
    +-- grass-inference (TensorRT, ONNX runtime bindings)
    +-- grass-iot (MQTT, Modbus TCP, IoT integration)
    +-- grass-streaming (RTSP, WebRTC media)
    +-- grass-vector (ChromaDB vector DB client)
    +-- grass-blockchain (Ethereum, streaming APIs)
    +-- grass-integrations (ONDC, IPFS, FISS)
    +-- grass-flow (N8N, Make.com flow editor webhooks)
    +-- grass-messaging (Kafka, QMQP message brokers)
    +-- grass-orchestrator (task manager, crew, parallel exec)
    +-- grass-binary (main entry point)
```

## Naming Conventions

All public structures and traits are prefixed with `Grass`:

| Concept             | Struct / Trait Name        | Crate           |
|---------------------|---------------------------|-----------------|
| Agent metadata      | `GrassAgentMetadata`      | grass-core      |
| Agent context       | `GrassAgentContext`       | grass-core      |
| Agent trait         | `GrassAgent`              | grass-core      |
| Agent builder       | `GrassAgentBuilder`       | grass-core      |
| Task payload        | `GrassTaskPayload`        | grass-core      |
| Task result         | `GrassTaskResult`         | grass-core      |
| Message             | `GrassMessage`            | grass-core      |
| Event bus           | `GrassEventBus`           | grass-core      |
| Error               | `GrassError`              | grass-core      |
| LLM request         | `GrassLlmRequest`         | grass-llm       |
| LLM response        | `GrassLlmResponse`        | grass-llm       |
| LLM provider trait  | `GrassLlmProvider`        | grass-llm       |
| LLM service         | `GrassLlmService`         | grass-llm       |
| Hook trait          | `GrassHook`               | grass-hooks     |
| Hook registry       | `GrassHookRegistry`       | grass-hooks     |
| Plugin trait        | `GrassPlugin`             | grass-core      |
| Plugin manifest     | `GrassPluginManifest`     | grass-core      |
| Sandbox config      | `GrassSandboxConfig`      | grass-core      |
| Runtime trait       | `GrassRuntime`            | grass-core      |
| IPC transport trait | `GrassIpcTransport`       | grass-ipc       |
| Protocol codec      | `GrassProtocolCodec`      | grass-protocol  |
| Task executor       | `GrassTaskExecutor`       | grass-runtime   |
| Task scheduler      | `GrassTaskScheduler`      | grass-runtime   |
| Crew manager        | `GrassCrewManager`        | grass-orchestrator |
| Sandbox backend     | `GrassSandboxBackend`     | grass-sandbox   |

## File Structure

```
├── Cargo.toml              # workspace root
├── rustfmt.toml            # formatting rules
├── .clippy.toml            # linting rules
├── doc/                    # architecture docs
├── tests/                  # integration tests
├── examples/               # usage examples
├── crates/
│   └── grass-*/            # per-crate source, tests, examples
│       ├── Cargo.toml
│       ├── src/
│       ├── tests/
│       └── examples/
```

## Design Principles

1. **Trait-based extensibility** - Every integration point is a trait.
2. **Zero-cost abstractions** - Use generics over dynamic dispatch where perf matters.
3. **Async-first** - All I/O operations use tokio async runtime.
4. **Sandbox isolation** - Agents execute in isolated environments by default.
5. **Plugin architecture** - Load/unload capabilities at runtime.
6. **Protocol agnostic** - Support MCP, ACP, gRPC, JSON-RPC via codec layer.
7. **Minimal external deps** - Custom implementations preferred over heavy deps.
