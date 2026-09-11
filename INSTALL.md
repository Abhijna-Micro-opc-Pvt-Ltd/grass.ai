# Installation Guide

Copyright (c) 2026 Abhijna Micro (OPC) PVT LTD
Author: Abhijna Micro Team <team@abhijnamicro.in>

## Prerequisites

### Required Software

- **Rust** (1.75.0 or later)
  - Install via [rustup](https://rustup.rs/):
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
  - Verify installation:
    ```bash
    rustc --version
    cargo --version
    ```

- **Git**
  - Install on Ubuntu/Debian:
    ```bash
    sudo apt-get update
    sudo apt-get install git
    ```
  - Install on macOS:
    ```bash
    xcode-select --install
    ```

### Optional Dependencies

- **Docker** (for sandbox functionality)
  - Install Docker: https://docs.docker.com/get-docker/

- **ZeroMQ** (for IPC functionality)
  - Install on Ubuntu/Debian:
    ```bash
    sudo apt-get install libzmq3-dev
    ```
  - Install on macOS:
    ```bash
    brew install zeromq
    ```

## Installation

### 1. Clone the Repository

```bash
git clone https://github.com/abhijna-micro-opc-pvt-ltd/grass.ai.git
cd grass.ai
```

### 2. Build the Project

#### Debug Build
```bash
cargo build
```

#### Release Build
```bash
cargo build --release
```

The release binary will be located at `target/release/grass.ai`.

### 3. Run Tests

```bash
cargo test
```

### 4. Run the Application

```bash
cargo run
```

Or use the release binary:
```bash
./target/release/grass.ai
```

## Workspace Structure

This project is organized as a Cargo workspace with the following crates:

| Crate | Description |
|-------|-------------|
| `grass-core` | Core types and error handling |
| `grass-runtime` | Runtime executor and scheduler |
| `grass-llm` | LLM provider integrations |
| `grass-hooks` | Webhook and notification hooks |
| `grass-blockchain` | Ethereum blockchain integration |
| `grass-integrations` | Third-party service integrations |
| `grass-protocol` | Communication protocols (MCP, ACP, gRPC) |
| `grass-ipc` | Inter-process communication |
| `grass-sandbox` | Container sandbox backends |
| `grass-plugin` | Plugin loading and management |
| `grass-inference` | ML inference (ONNX, TensorRT) |
| `grass-iot` | IoT protocols (MQTT, Modbus) |
| `grass-streaming` | Media streaming (WebRTC, RTSP) |
| `grass-vector` | Vector database integration |
| `grass-orchestrator` | Agent orchestration |
| `grass-flow` | Workflow automation |
| `grass-messaging` | Message queue integration |
| `grass-binary` | Main binary entry point |

## Building Individual Crates

To build a specific crate:

```bash
cargo build -p grass-llm
cargo build -p grass-core
cargo build -p grass-sandbox
```

## Development Setup

### IDE Configuration

For VS Code, install the `rust-analyzer` extension for Rust language support.

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

### Generate Documentation

```bash
cargo doc --open
```

## Troubleshooting

### Common Issues

1. **OpenSSL not found**
   ```bash
   # Ubuntu/Debian
   sudo apt-get install libssl-dev pkg-config
   
   # macOS
   brew install openssl
   ```

2. **ZeroMQ not found**
   ```bash
   # Ubuntu/Debian
   sudo apt-get install libzmq3-dev
   
   # macOS
   brew install zeromq
   ```

3. **Docker permission denied**
   ```bash
   sudo usermod -aG docker $USER
   # Log out and log back in
   ```

### Getting Help

- Email: team@abhijnamicro.in
- Issues: https://github.com/abhijna-micro-opc-pvt-ltd/grass.ai/issues

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
