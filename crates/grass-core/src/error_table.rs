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

use crate::{GrassError, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GrassErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for GrassErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrassErrorEntry {
    pub code: String,
    pub domain: String,
    pub module: String,
    pub severity: GrassErrorSeverity,
    pub message: String,
    pub causes: Vec<String>,
    pub remediation: Vec<String>,
}

impl GrassErrorEntry {
    pub fn new(
        code: &str,
        domain: &str,
        module: &str,
        severity: GrassErrorSeverity,
        message: &str,
    ) -> Self {
        Self {
            code: code.to_string(),
            domain: domain.to_string(),
            module: module.to_string(),
            severity,
            message: message.to_string(),
            causes: Vec::new(),
            remediation: Vec::new(),
        }
    }

    pub fn with_cause(mut self, cause: &str) -> Self {
        self.causes.push(cause.to_string());
        self
    }

    pub fn with_remediation(mut self, fix: &str) -> Self {
        self.remediation.push(fix.to_string());
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GrassErrorTable {
    entries: BTreeMap<String, GrassErrorEntry>,
}

impl GrassErrorTable {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn register(&mut self, entry: GrassErrorEntry) -> Result<()> {
        if self.entries.contains_key(&entry.code) {
            return Err(GrassError::Internal(format!(
                "duplicate error code registration: {}",
                entry.code
            )));
        }
        self.entries.insert(entry.code.clone(), entry);
        Ok(())
    }

    pub fn lookup(&self, code: &str) -> Option<&GrassErrorEntry> {
        self.entries.get(code)
    }

    pub fn lookup_domain(&self, domain: &str) -> Vec<&GrassErrorEntry> {
        self.entries
            .values()
            .filter(|e| e.domain == domain)
            .collect()
    }

    pub fn lookup_severity(&self, severity: GrassErrorSeverity) -> Vec<&GrassErrorEntry> {
        self.entries
            .values()
            .filter(|e| e.severity == severity)
            .collect()
    }

    pub fn contains(&self, code: &str) -> bool {
        self.entries.contains_key(code)
    }

    pub fn codes(&self) -> Vec<&str> {
        self.entries.keys().map(|k| k.as_str()).collect()
    }

    pub fn domains(&self) -> Vec<&str> {
        let mut set = BTreeSet::new();
        for e in self.entries.values() {
            set.insert(e.domain.as_str());
        }
        set.into_iter().collect()
    }

    pub fn entries(&self) -> Vec<&GrassErrorEntry> {
        self.entries.values().collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn merge(&mut self, other: &GrassErrorTable) -> Result<()> {
        for entry in other.entries.values() {
            self.register(entry.clone())?;
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(GrassError::Serialization)
    }

    pub fn from_json(json: &str) -> Result<Self> {
        let table: GrassErrorTable =
            serde_json::from_str(json).map_err(GrassError::Serialization)?;
        Ok(table)
    }

    pub fn to_file(&self, path: &Path) -> Result<()> {
        let json = self.to_json()?;
        std::fs::write(path, json).map_err(GrassError::Io)
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let json = std::fs::read_to_string(path).map_err(GrassError::Io)?;
        Self::from_json(&json)
    }
}

static BUILTIN_TABLE: OnceLock<GrassErrorTable> = OnceLock::new();

pub fn grass_error_table() -> &'static GrassErrorTable {
    BUILTIN_TABLE.get_or_init(register_builtin_errors)
}

pub fn reload_grass_error_table(table: GrassErrorTable) -> Result<()> {
    BUILTIN_TABLE
        .set(table)
        .map_err(|_| GrassError::Internal("global error table already initialized".into()))
}

pub fn register_builtin_errors() -> GrassErrorTable {
    let mut t = GrassErrorTable::new();

    let mut put = |code: &str,
                   domain: &str,
                   module: &str,
                   severity: GrassErrorSeverity,
                   message: &str,
                   causes: &[&str],
                   fixes: &[&str]| {
        let mut e = GrassErrorEntry::new(code, domain, module, severity, message);
        for c in causes {
            e = e.with_cause(c);
        }
        for f in fixes {
            e = e.with_remediation(f);
        }
        t.register(e).expect("builtin error codes must be unique");
    };

    put(
        "GR-CORE-0001",
        "core",
        "agent",
        GrassErrorSeverity::Error,
        "Agent execution failed",
        &[
            "start/stop transition raced",
            "execute panicked",
            "bad task payload",
        ],
        &[
            "inspect agent state machine",
            "enable RUST_LOG=debug",
            "replay task payload",
        ],
    );
    put(
        "GR-CORE-0002",
        "core",
        "task",
        GrassErrorSeverity::Error,
        "Task execution failed",
        &[
            "missing agent",
            "task payload invalid",
            "duration exceeded timeout",
        ],
        &[
            "validate task_id/agent_id",
            "raise GrassScheduler timeout",
            "check result.error",
        ],
    );
    put(
        "GR-CORE-0003",
        "core",
        "plugin",
        GrassErrorSeverity::Error,
        "Plugin lifecycle error",
        &[
            "manifest malformed",
            "ABI mismatch",
            "dependency unsatisfied",
        ],
        &[
            "verify GrassPluginManifest fields",
            "match ABI version",
            "fulfil declared deps",
        ],
    );
    put(
        "GR-CORE-0004",
        "core",
        "hook",
        GrassErrorSeverity::Error,
        "Hook dispatch error",
        &["webhook endpoint unreachable", "payload not serializable"],
        &["confirm endpoint reachable", "ensure payload is JSON"],
    );
    put(
        "GR-CORE-0005",
        "core",
        "serialize",
        GrassErrorSeverity::Error,
        "Serialization failure",
        &["non-JSON payload", "field type mismatch"],
        &[
            "inspect offending value",
            "use serde_json::Value for dynamic fields",
        ],
    );
    put(
        "GR-CORE-0006",
        "core",
        "io",
        GrassErrorSeverity::Error,
        "I/O failure",
        &["file missing", "permission denied", "disk full"],
        &["check path exists", "fix file mode", "free disk space"],
    );
    put(
        "GR-CORE-0007",
        "core",
        "timeout",
        GrassErrorSeverity::Warning,
        "Operation timed out",
        &["slow upstream", "incorrect deadline", "lock contention"],
        &["raise timeout", "retry with backoff", "inspect contention"],
    );
    put(
        "GR-CORE-0008",
        "core",
        "lookup",
        GrassErrorSeverity::Info,
        "Resource not found",
        &["id mismatch", "registry empty", "stale reference"],
        &["verify id", "populate registry", "refresh references"],
    );
    put(
        "GR-CORE-0009",
        "core",
        "security",
        GrassErrorSeverity::Critical,
        "Permission denied",
        &["missing role", "revoked token", "policy denies"],
        &["assign role", "rotate credential", "review policy"],
    );
    put(
        "GR-CORE-0010",
        "core",
        "internal",
        GrassErrorSeverity::Error,
        "Unclassified internal error",
        &["bug", "invariant violation"],
        &["collect backtrace", "open issue with reproduction"],
    );

    put(
        "GR-RUNTIME-0001",
        "runtime",
        "executor",
        GrassErrorSeverity::Error,
        "Executor queue saturated",
        &["worker pool exhausted", "blocked tasks"],
        &[
            "increase GrassTaskExecutor worker count",
            "add timeouts to tasks",
        ],
    );
    put(
        "GR-RUNTIME-0002",
        "runtime",
        "scheduler",
        GrassErrorSeverity::Warning,
        "Scheduler ordering conflict",
        &["priority inversion", "duplicate enqueue"],
        &["duplicate-id check", "rebalance priorities"],
    );
    put(
        "GR-RUNTIME-0003",
        "runtime",
        "timer",
        GrassErrorSeverity::Info,
        "Timer cancelled or expired",
        &["deadline passed", "timer reset"],
        &["use monotonic clock", "restart timer"],
    );

    put(
        "GR-LLM-0001",
        "llm",
        "service",
        GrassErrorSeverity::Error,
        "LLM provider not found",
        &["provider not registered", "name typo"],
        &[
            "check GrassLlmService::register_provider calls",
            "verify provider name",
        ],
    );
    put(
        "GR-LLM-0002",
        "llm",
        "provider",
        GrassErrorSeverity::Error,
        "LLM API request failed",
        &[
            "invalid API key",
            "network error",
            "rate limited",
            "quota exhausted",
        ],
        &[
            "rotate key",
            "retry with backoff",
            "check provider status page",
        ],
    );
    put(
        "GR-LLM-0003",
        "llm",
        "provider",
        GrassErrorSeverity::Error,
        "LLM response parse failed",
        &["schema drift", "empty body", "wrong endpoint"],
        &[
            "pin a model version",
            "log raw response body",
            "verify endpoint",
        ],
    );
    put(
        "GR-LLM-0004",
        "llm",
        "stream",
        GrassErrorSeverity::Info,
        "Streaming not supported by provider",
        &["provider lacks SSE", "feature disabled"],
        &["use complete() fallback", "enable stream flag"],
    );
    put(
        "GR-LLM-0005",
        "llm",
        "provider",
        GrassErrorSeverity::Warning,
        "Empty choices returned",
        &["max_tokens too low", "model refusal"],
        &["raise max_tokens", "inspect finish_reason"],
    );

    put(
        "GR-HOOKS-0001",
        "hooks",
        "slack",
        GrassErrorSeverity::Error,
        "Slack webhook delivery failed",
        &["bad webhook URL", "rate limited", "channel removed"],
        &["verify incoming-webhook URL", "backoff and retry"],
    );
    put(
        "GR-HOOKS-0002",
        "hooks",
        "discord",
        GrassErrorSeverity::Error,
        "Discord webhook delivery failed",
        &["webhook token invalid", "content too long"],
        &["recreate webhook", "truncate message"],
    );
    put(
        "GR-HOOKS-0003",
        "hooks",
        "webhook",
        GrassErrorSeverity::Error,
        "Generic webhook delivery failed",
        &["endpoint 4xx/5xx", "timeout", "TLS error"],
        &["inspect status code", "raise timeout", "check certificate"],
    );

    put(
        "GR-PROTO-0001",
        "protocol",
        "mcp",
        GrassErrorSeverity::Error,
        "MCP frame encode/decode failed",
        &["malformed JSON-RPC", "missing id or method"],
        &[
            "validate GrassMcpRequest",
            "use GrassMcpCodec decode round-trip",
        ],
    );
    put(
        "GR-PROTO-0002",
        "protocol",
        "acp",
        GrassErrorSeverity::Error,
        "ACP routing failed",
        &["unknown command", "payload mismatch"],
        &["register command handler", "validate envelope"],
    );
    put(
        "GR-PROTO-0003",
        "protocol",
        "grpc",
        GrassErrorSeverity::Error,
        "gRPC registry miss",
        &["service not registered", "protobuf mismatch"],
        &["register service in GrassGrpcRegistry", "regenerate protos"],
    );

    put(
        "GR-IPC-0001",
        "ipc",
        "zmq",
        GrassErrorSeverity::Error,
        "ZeroMQ transport failure",
        &["socket bind conflict", "peer disconnected"],
        &["change endpoint", "reconnect with retry"],
    );
    put(
        "GR-IPC-0002",
        "ipc",
        "valkey",
        GrassErrorSeverity::Error,
        "Valkey transport failure",
        &["server down", "auth failed"],
        &["check valkey health", "verify credentials"],
    );
    put(
        "GR-IPC-0003",
        "ipc",
        "shm",
        GrassErrorSeverity::Error,
        "Shared memory transport failure",
        &["region unavailable", "size overflow"],
        &["check /dev/shm space", "reduce payload size"],
    );

    put(
        "GR-SANDBOX-0001",
        "sandbox",
        "docker",
        GrassErrorSeverity::Error,
        "Docker sandbox operation failed",
        &["daemon unreachable", "image missing", "resource limit"],
        &[
            "verify /var/run/docker.sock",
            "build grass-agent:latest",
            "raise limit",
        ],
    );
    put(
        "GR-SANDBOX-0002",
        "sandbox",
        "kata",
        GrassErrorSeverity::Error,
        "Kata runtime failure",
        &["runtime_path invalid", "firecracker not available"],
        &["verify kata-runtime binary", "enable KVM"],
    );
    put(
        "GR-SANDBOX-0003",
        "sandbox",
        "oci",
        GrassErrorSeverity::Error,
        "OCI bundle not found",
        &["bundle_path empty", "config.json missing"],
        &["point to valid bundle", "create config.json"],
    );
    put(
        "GR-SANDBOX-0004",
        "sandbox",
        "wasm",
        GrassErrorSeverity::Error,
        "WASM instance failure",
        &["fuel exhausted", "memory limit hit"],
        &["raise fuel_limit", "optimize module"],
    );

    put(
        "GR-PLUGIN-0001",
        "plugin",
        "loader",
        GrassErrorSeverity::Error,
        "Dynamic plugin load failed",
        &["library incompatible", "never loaded symbol"],
        &["match ABI/version", "verify .so exports"],
    );
    put(
        "GR-PLUGIN-0002",
        "plugin",
        "manifest",
        GrassErrorSeverity::Error,
        "Plugin manifest invalid",
        &["missing name/version", "unparseable schema"],
        &["validate GrassPluginManifest", "fix schema"],
    );
    put(
        "GR-PLUGIN-0003",
        "plugin",
        "registry",
        GrassErrorSeverity::Error,
        "Plugin registration conflict",
        &["duplicate name", "version downgrade"],
        &["choose unique name", "bump version"],
    );

    put(
        "GR-INF-0001",
        "inference",
        "tensorrt",
        GrassErrorSeverity::Error,
        "TensorRT engine failed",
        &["engine version mismatch", "GPU unsupported"],
        &["rebuild engine", "match TensorRT version"],
    );
    put(
        "GR-INF-0002",
        "inference",
        "onnx",
        GrassErrorSeverity::Error,
        "ONNX runtime session failed",
        &["model path missing", "opset unsupported"],
        &["verify .onnx path", "re-export model"],
    );

    put(
        "GR-IOT-0001",
        "iot",
        "mqtt",
        GrassErrorSeverity::Error,
        "MQTT connection lost",
        &["broker down", "credentials rejected"],
        &["check broker health", "verify user/pass"],
    );
    put(
        "GR-IOT-0002",
        "iot",
        "modbus",
        GrassErrorSeverity::Error,
        "Modbus slave timeout",
        &["slave offline", "address range invalid"],
        &["ping slave device", "fix register address"],
    );

    put(
        "GR-STREAM-0001",
        "streaming",
        "rtsp",
        GrassErrorSeverity::Error,
        "RTSP handshake failed",
        &["bad URL", "camera not streaming"],
        &["verify rtsp:// path", "start camera stream"],
    );
    put(
        "GR-STREAM-0002",
        "streaming",
        "webrtc",
        GrassErrorSeverity::Error,
        "WebRTC peer negotiation failed",
        &["SDP mismatch", "ICE timeout"],
        &["enable TURN", "restart signaling"],
    );

    put(
        "GR-VEC-0001",
        "vector",
        "chromadb",
        GrassErrorSeverity::Error,
        "ChromaDB unreachable or method missing",
        &["server down", "wrong collection"],
        &["check localhost:8000", "verify collection name"],
    );
    put(
        "GR-VEC-0002",
        "vector",
        "embedding",
        GrassErrorSeverity::Error,
        "Query dimension mismatch",
        &["model changed", "index stale"],
        &["match embedding model", "re-index collection"],
    );

    put(
        "GR-BC-0001",
        "blockchain",
        "eth",
        GrassErrorSeverity::Error,
        "Ethereum RPC call failed",
        &["node unreachable", "rate limited"],
        &["check provider endpoint", "add backoff"],
    );
    put(
        "GR-BC-0002",
        "blockchain",
        "tx",
        GrassErrorSeverity::Error,
        "Transaction nonce mismatch",
        &["stale nonce", "pending txs"],
        &["refresh nonce", "await mined block"],
    );
    put(
        "GR-BC-0003",
        "blockchain",
        "tx",
        GrassErrorSeverity::Error,
        "Gas estimation failed",
        &["insufficient balance", "revert"],
        &["fund account", "inspect revert reason"],
    );

    put(
        "GR-INT-0001",
        "integrations",
        "ondc",
        GrassErrorSeverity::Error,
        "ONDC subscribe failed",
        &["invalid signing pub key", "callback unreachable"],
        &["verify pub key", "expose callback URL"],
    );
    put(
        "GR-INT-0002",
        "integrations",
        "ipfs",
        GrassErrorSeverity::Error,
        "IPFS pin failed",
        &["node offline", "content too large"],
        &["check ipfs daemon", "chunk upload"],
    );
    put(
        "GR-INT-0003",
        "integrations",
        "fiss",
        GrassErrorSeverity::Error,
        "FISS registration failed",
        &["gstin invalid", "HSN mismatch"],
        &["validate GSTIN", "correct HSN mapping"],
    );

    put(
        "GR-FLOW-0001",
        "flow",
        "n8n",
        GrassErrorSeverity::Error,
        "N8N trigger failed",
        &["workflow inactive", "bad credentials"],
        &["activate workflow", "verify API key"],
    );
    put(
        "GR-FLOW-0002",
        "flow",
        "make",
        GrassErrorSeverity::Error,
        "Make.com webhook failed",
        &["webhook removed", "payload schema mismatch"],
        &["recreate webhook", "match scenario schema"],
    );

    put(
        "GR-MSG-0001",
        "messaging",
        "kafka",
        GrassErrorSeverity::Error,
        "Kafka producer send failed",
        &["broker down", "topic missing", "acks timeout"],
        &[
            "check brokers list",
            "create topic",
            "increase request timeouts",
        ],
    );
    put(
        "GR-MSG-0002",
        "messaging",
        "kafka",
        GrassErrorSeverity::Error,
        "Kafka consumer failed",
        &["group rebalance", "offset out of range"],
        &["reset offsets", "handle rebalance events"],
    );
    put(
        "GR-MSG-0003",
        "messaging",
        "qmqp",
        GrassErrorSeverity::Error,
        "QMQP channel failed",
        &["broker closed channel", "invalid route"],
        &["reconnect channel", "verify routing key"],
    );

    put(
        "GR-ORCH-0001",
        "orchestrator",
        "graph",
        GrassErrorSeverity::Critical,
        "Task graph cycle detected",
        &["edge misconfigured", "self-loop"],
        &["run topological sort", "remove cyclic edge"],
    );
    put(
        "GR-ORCH-0002",
        "orchestrator",
        "crew",
        GrassErrorSeverity::Error,
        "Crew agent unassigned",
        &["agent died", "task not mapped to agent"],
        &["restart agent", "call assign_task"],
    );
    put(
        "GR-ORCH-0003",
        "orchestrator",
        "dag",
        GrassErrorSeverity::Error,
        "Execution DAG resolution failed",
        &["multiple entry points", "missing node"],
        &["declare single root", "add missing node"],
    );

    put(
        "GR-BIN-0001",
        "binary",
        "startup",
        GrassErrorSeverity::Error,
        "Startup configuration invalid",
        &["missing env var", "bad RUST_LOG"],
        &["set required env vars", "fix log filter"],
    );
    put(
        "GR-BIN-0002",
        "binary",
        "shutdown",
        GrassErrorSeverity::Warning,
        "Graceful shutdown incomplete",
        &["tasks still running", "hook drain timed out"],
        &["await task completion", "raise drain timeout"],
    );

    t
}

impl GrassError {
    pub fn code(&self) -> &'static str {
        match self {
            GrassError::Agent(_) => "GR-CORE-0001",
            GrassError::Task(_) => "GR-CORE-0002",
            GrassError::Plugin(_) => "GR-CORE-0003",
            GrassError::Hook(_) => "GR-CORE-0004",
            GrassError::Protocol(_) => "GR-PROTO-0001",
            GrassError::Ipc(_) => "GR-IPC-0001",
            GrassError::Sandbox(_) => "GR-SANDBOX-0001",
            GrassError::Llm(_) => "GR-LLM-0001",
            GrassError::Inference(_) => "GR-INF-0001",
            GrassError::Blockchain(_) => "GR-BC-0001",
            GrassError::Iot(_) => "GR-IOT-0001",
            GrassError::Streaming(_) => "GR-STREAM-0001",
            GrassError::VectorDb(_) => "GR-VEC-0001",
            GrassError::Integration(_) => "GR-INT-0001",
            GrassError::Flow(_) => "GR-FLOW-0001",
            GrassError::Messaging(_) => "GR-MSG-0001",
            GrassError::Serialization(_) => "GR-CORE-0005",
            GrassError::Io(_) => "GR-CORE-0006",
            GrassError::Timeout(_) => "GR-CORE-0007",
            GrassError::NotFound(_) => "GR-CORE-0008",
            GrassError::PermissionDenied(_) => "GR-CORE-0009",
            GrassError::Internal(_) => "GR-CORE-0010",
        }
    }

    pub fn domain(&self) -> &'static str {
        match self.code().split('-').nth(1) {
            Some(d) => match d {
                "CORE" => "core",
                "RUNTIME" => "runtime",
                "LLM" => "llm",
                "HOOKS" => "hooks",
                "PROTO" => "protocol",
                "IPC" => "ipc",
                "SANDBOX" => "sandbox",
                "PLUGIN" => "plugin",
                "INF" => "inference",
                "IOT" => "iot",
                "STREAM" => "streaming",
                "VEC" => "vector",
                "BC" => "blockchain",
                "INT" => "integrations",
                "FLOW" => "flow",
                "MSG" => "messaging",
                "ORCH" => "orchestrator",
                "BIN" => "binary",
                _ => "core",
            },
            None => "core",
        }
    }

    pub fn entry(&self) -> Option<&'static GrassErrorEntry> {
        grass_error_table().lookup(self.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_table_registers_all_domains() {
        let table = grass_error_table();
        assert!(!table.is_empty());
        assert!(
            table.len() >= 50,
            "expected >= 50 builtin entries, got {}",
            table.len()
        );
        let domains = table.domains();
        for d in [
            "core",
            "runtime",
            "llm",
            "hooks",
            "protocol",
            "ipc",
            "sandbox",
            "plugin",
            "inference",
            "iot",
            "streaming",
            "vector",
            "blockchain",
            "integrations",
            "flow",
            "messaging",
            "orchestrator",
            "binary",
        ] {
            assert!(domains.contains(&d), "missing domain {}", d);
        }
    }

    #[test]
    fn every_grass_error_maps_to_table_entry() {
        for err in [
            GrassError::Agent("x".into()),
            GrassError::Task("x".into()),
            GrassError::Plugin("x".into()),
            GrassError::Hook("x".into()),
            GrassError::Protocol("x".into()),
            GrassError::Ipc("x".into()),
            GrassError::Sandbox("x".into()),
            GrassError::Llm("x".into()),
            GrassError::Inference("x".into()),
            GrassError::Blockchain("x".into()),
            GrassError::Iot("x".into()),
            GrassError::Streaming("x".into()),
            GrassError::VectorDb("x".into()),
            GrassError::Integration("x".into()),
            GrassError::Flow("x".into()),
            GrassError::Messaging("x".into()),
            GrassError::Serialization(serde_json::from_str::<serde_json::Value>("x").unwrap_err()),
            GrassError::Io(std::io::Error::new(std::io::ErrorKind::Other, "x")),
            GrassError::Timeout(1),
            GrassError::NotFound("x".into()),
            GrassError::PermissionDenied("x".into()),
            GrassError::Internal("x".into()),
        ] {
            assert!(err.entry().is_some(), "no table entry for {:?}", err);
            assert_eq!(err.entry().unwrap().domain, err.domain());
        }
    }

    #[test]
    fn duplicate_registration_is_rejected() {
        let mut t = GrassErrorTable::new();
        t.register(GrassErrorEntry::new(
            "GR-TEST-0001",
            "test",
            "t",
            GrassErrorSeverity::Error,
            "first",
        ))
        .unwrap();
        let dup = GrassErrorEntry::new(
            "GR-TEST-0001",
            "test",
            "t",
            GrassErrorSeverity::Error,
            "second",
        );
        assert!(t.register(dup).is_err());
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn lookup_of_unknown_code_returns_none() {
        let table = grass_error_table();
        assert!(table.lookup("GR-NOPE-9999").is_none());
        assert!(!table.contains("GR-NOPE-9999"));
    }

    #[test]
    fn lookup_domain_filters_correctly() {
        let table = grass_error_table();
        let llm = table.lookup_domain("llm");
        assert!(!llm.is_empty());
        assert!(llm.iter().all(|e| e.domain == "llm"));
    }

    #[test]
    fn lookup_severity_returns_critical_entries() {
        let table = grass_error_table();
        let critical = table.lookup_severity(GrassErrorSeverity::Critical);
        assert!(!critical.is_empty());
        assert!(critical
            .iter()
            .all(|e| e.severity == GrassErrorSeverity::Critical));
    }

    #[test]
    fn json_roundtrip_preserves_entries() {
        let table = grass_error_table();
        let json = table.to_json().unwrap();
        let loaded = GrassErrorTable::from_json(&json).unwrap();
        assert_eq!(table.len(), loaded.len());
        for code in table.codes() {
            let original = table.lookup(code).unwrap();
            let restored = loaded.lookup(code).unwrap();
            assert_eq!(original.message, restored.message);
            assert_eq!(original.remediation, restored.remediation);
        }
    }

    #[test]
    fn file_persistence_roundtrip_tmpdir() {
        let table = grass_error_table();
        let path = std::env::temp_dir().join(format!("grass_errors_{}.json", uuid::Uuid::new_v4()));
        table.to_file(&path).unwrap();
        let loaded = GrassErrorTable::from_file(&path).unwrap();
        assert_eq!(loaded.len(), table.len());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn from_file_missing_path_returns_io_error() {
        let path = std::path::Path::new("/nonexistent/grass_errors_missing.json");
        let err = GrassErrorTable::from_file(path).unwrap_err();
        assert!(err.code().starts_with("GR-CORE"));
    }

    #[test]
    fn merge_rejects_collisions() {
        let mut a = GrassErrorTable::new();
        a.register(GrassErrorEntry::new(
            "GR-TEST-0002",
            "test",
            "t",
            GrassErrorSeverity::Error,
            "a",
        ))
        .unwrap();
        let mut b = GrassErrorTable::new();
        b.register(GrassErrorEntry::new(
            "GR-TEST-0002",
            "test",
            "t",
            GrassErrorSeverity::Error,
            "b",
        ))
        .unwrap();
        assert!(a.merge(&b).is_err());
    }

    #[test]
    fn entry_builder_chains_metadata() {
        let entry = GrassErrorEntry::new(
            "GR-TEST-0003",
            "test",
            "t",
            GrassErrorSeverity::Warning,
            "m",
        )
        .with_cause("c1")
        .with_remediation("r1")
        .with_remediation("r2");
        assert_eq!(entry.causes, vec!["c1".to_string()]);
        assert_eq!(entry.remediation.len(), 2);
        assert_eq!(entry.domain, "test");
        assert_eq!(entry.code, "GR-TEST-0003");
    }
}
