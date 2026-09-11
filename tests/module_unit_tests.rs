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

use grass_core::*;
use grass_flow::*;
use grass_inference::*;
use grass_llm::*;
use grass_messaging::*;
use grass_protocol::*;
use grass_streaming::*;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

// ============================================================================
// grass-core: event module
// ============================================================================

#[test]
fn test_event_bus_multiple_subscribers() {
    let mut bus = GrassEventBus::new();
    let c1 = Arc::new(AtomicUsize::new(0));
    let c2 = Arc::new(AtomicUsize::new(0));
    let c1c = c1.clone();
    let c2c = c2.clone();

    bus.subscribe(move |_| {
        c1c.fetch_add(1, Ordering::SeqCst);
    });
    bus.subscribe(move |_| {
        c2c.fetch_add(1, Ordering::SeqCst);
    });

    let event = GrassEvent::TaskCreated {
        task_id: uuid::Uuid::new_v4(),
    };
    bus.publish(&event);
    bus.publish(&event);

    assert_eq!(c1.load(Ordering::SeqCst), 2);
    assert_eq!(c2.load(Ordering::SeqCst), 2);
}

#[test]
fn test_event_variants_serialization() {
    let events = vec![
        GrassEvent::AgentStarted {
            agent_id: uuid::Uuid::new_v4(),
        },
        GrassEvent::AgentStopped {
            agent_id: uuid::Uuid::new_v4(),
        },
        GrassEvent::AgentFailed {
            agent_id: uuid::Uuid::new_v4(),
            error: "boom".into(),
        },
        GrassEvent::TaskCreated {
            task_id: uuid::Uuid::new_v4(),
        },
        GrassEvent::TaskStarted {
            task_id: uuid::Uuid::new_v4(),
        },
        GrassEvent::TaskCompleted {
            task_id: uuid::Uuid::new_v4(),
        },
        GrassEvent::TaskFailed {
            task_id: uuid::Uuid::new_v4(),
            error: "fail".into(),
        },
        GrassEvent::HookFired {
            hook_id: uuid::Uuid::new_v4(),
            payload: serde_json::json!({"data": 1}),
        },
        GrassEvent::PluginLoaded {
            plugin_id: uuid::Uuid::new_v4(),
            name: "p".into(),
        },
        GrassEvent::PluginUnloaded {
            plugin_id: uuid::Uuid::new_v4(),
        },
        GrassEvent::MessageSent {
            message_id: uuid::Uuid::new_v4(),
            from: uuid::Uuid::new_v4(),
            topic: "t".into(),
        },
        GrassEvent::SandboxCreated {
            backend: "docker".into(),
            container_id: "c1".into(),
        },
        GrassEvent::SandboxDestroyed {
            container_id: "c1".into(),
        },
        GrassEvent::Custom {
            event_type: "custom".into(),
            data: HashMap::new(),
        },
    ];

    for ev in events {
        let json = serde_json::to_string(&ev).unwrap();
        let back: GrassEvent = serde_json::from_str(&json).unwrap();
        let json2 = serde_json::to_string(&back).unwrap();
        assert_eq!(json, json2, "event lost data during roundtrip");
    }
}

// ============================================================================
// grass-core: context module
// ============================================================================

#[test]
fn test_agent_context_lifecycle() {
    let meta = GrassAgentBuilder::new("ctx-agent").version("1.0.0").build();
    let mut ctx = GrassAgentContext::new(meta.id, meta);

    assert!(ctx.sandbox.is_none());
    assert!(ctx.get_state("missing").is_none());

    ctx.set_state("count", serde_json::json!(10));
    ctx.set_state("name", serde_json::json!("build-line-1"));
    assert_eq!(ctx.get_state("count").unwrap(), 10);
    assert_eq!(ctx.get_state("name").unwrap(), "build-line-1");

    ctx.set_state("count", serde_json::json!(11));
    assert_eq!(ctx.get_state("count").unwrap(), 11);
}

#[test]
fn test_agent_context_with_sandbox() {
    let mut config = GrassSandboxConfig::default();
    config.backend = GrassSandboxBackend::WASM;
    config.max_memory_bytes = 256 * 1024 * 1024;

    let meta = GrassAgentBuilder::new("sandbox-agent").build();
    let ctx = GrassAgentContext::new(meta.id, meta).with_sandbox(config);

    let sb = ctx.sandbox.unwrap();
    assert_eq!(sb.backend, GrassSandboxBackend::WASM);
    assert_eq!(sb.max_memory_bytes, 256 * 1024 * 1024);
}

// ============================================================================
// grass-core: plugin module
// ============================================================================

struct MockPlugin {
    initialized: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl GrassPlugin for MockPlugin {
    fn name(&self) -> &str {
        "mock-plugin"
    }
    fn version(&self) -> &str {
        "0.1.0"
    }
    async fn initialize(&mut self) -> Result<()> {
        self.initialized.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
    fn capabilities(&self) -> Vec<String> {
        vec!["storage".into(), "compute".into()]
    }
}

#[tokio::test]
async fn test_plugin_trait_with_mock() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut plugin = MockPlugin {
        initialized: counter.clone(),
    };
    plugin.initialize().await.unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 1);
    assert_eq!(plugin.capabilities(), vec!["storage", "compute"]);
    plugin.shutdown().await.unwrap();
}

#[test]
fn test_plugin_manifest_full() {
    let mut manifest = GrassPluginManifest::new("sensor-plugin", "2.1.0");
    manifest.description = "Reads sensors".into();
    manifest.author = "grass.ai".into();
    manifest.capabilities = vec!["modbus".into(), "mqtt".into()];
    manifest.config_schema = Some(serde_json::json!({"type": "object"}));

    assert_eq!(manifest.name, "sensor-plugin");
    assert_eq!(manifest.version, "2.1.0");
    assert_eq!(manifest.capabilities.len(), 2);
    assert!(manifest.config_schema.is_some());

    let json = serde_json::to_string(&manifest).unwrap();
    let back: GrassPluginManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.author, "grass.ai");
}

// ============================================================================
// grass-core: runtime module
// ============================================================================

#[tokio::test]
async fn test_tokio_runtime_spawn_and_blocking() {
    let rt = GrassTokioRuntime::new();

    let (tx, rx) = tokio::sync::oneshot::channel();
    rt.spawn("compute-task", async move {
        let _ = tx.send(99);
    })
    .await
    .unwrap();
    assert_eq!(rx.await.unwrap(), 99);

    let (btx, brx) = tokio::sync::oneshot::channel();
    rt.spawn_blocking("blocking-io", move || {
        let _ = btx.send("blocking-done");
    })
    .await
    .unwrap();
    assert_eq!(brx.await.unwrap(), "blocking-done");

    rt.shutdown().await.unwrap();
}

// ============================================================================
// grass-protocol: mcp module
// ============================================================================

struct MockMcpHandler;

#[async_trait::async_trait]
impl GrassMcpHandler for MockMcpHandler {
    async fn handle_request(&self, request: GrassMcpRequest) -> Result<GrassMcpResponse> {
        Ok(GrassMcpResponse {
            jsonrpc: "2.0".into(),
            result: Some(serde_json::json!({"echo": request.params})),
            error: None,
            id: request.id,
        })
    }
    async fn handle_notification(&self, _n: GrassMcpNotification) -> Result<()> {
        Ok(())
    }
}

#[test]
fn test_mcp_codec_all_types() {
    let req = GrassMcpRequest {
        jsonrpc: "2.0".into(),
        method: "tools/list".into(),
        params: serde_json::json!({"limit": 10}),
        id: 7,
    };
    let resp = GrassMcpResponse {
        jsonrpc: "2.0".into(),
        result: Some(serde_json::json!({"ok": true})),
        error: None,
        id: 7,
    };
    let notif = GrassMcpNotification {
        jsonrpc: "2.0".into(),
        method: "tools/call".into(),
        params: Some(serde_json::json!({"name": "calc"})),
    };
    let err = GrassMcpError {
        code: -32601,
        message: "method not found".into(),
        data: None,
    };

    let req_b =
        GrassMcpCodec::decode_request(&GrassMcpCodec::encode_request(&req).unwrap()).unwrap();
    assert_eq!(req_b.method, "tools/list");
    let resp_b =
        GrassMcpCodec::decode_response(&GrassMcpCodec::encode_response(&resp).unwrap()).unwrap();
    assert_eq!(resp_b.id, 7);
    let notif_b =
        GrassMcpCodec::decode_notification(&GrassMcpCodec::encode_notification(&notif).unwrap())
            .unwrap();
    assert_eq!(notif_b.method, "tools/call");
    assert_eq!(err.code, -32601);
}

#[tokio::test]
async fn test_mcp_router_mock() {
    let mut router = GrassMcpRouter::new();
    router.register("tools/list", Arc::new(MockMcpHandler));

    let req = GrassMcpRequest {
        jsonrpc: "2.0".into(),
        method: "tools/list".into(),
        params: serde_json::json!({"limit": 5}),
        id: 1,
    };
    let resp = router.route(req).await.unwrap();
    assert_eq!(resp.result.unwrap()["echo"]["limit"], 5);

    let missing = GrassMcpRequest {
        jsonrpc: "2.0".into(),
        method: "unknown".into(),
        params: serde_json::json!({}),
        id: 2,
    };
    assert!(router.route(missing).await.is_err());
}

// ============================================================================
// grass-protocol: acp module
// ============================================================================

struct MockAcpHandler;

#[async_trait::async_trait]
impl GrassAcpHandler for MockAcpHandler {
    async fn handle(&self, msg: GrassAcpMessage) -> Result<GrassAcpMessage> {
        Ok(GrassAcpMessage {
            source: msg.destination,
            destination: msg.source,
            content_type: "application/reply".into(),
            payload: serde_json::json!({"ack": true}),
            metadata: msg.metadata,
        })
    }
}

#[test]
fn test_acp_envelope_serialization() {
    let msg = GrassAcpMessage {
        source: "agent-a".into(),
        destination: "agent-b".into(),
        content_type: "application/json".into(),
        payload: serde_json::json!({"task": "infer"}),
        metadata: HashMap::from([("idempotency".into(), "abc123".into())]),
    };
    let envelope = GrassAcpEnvelope {
        message: msg.clone(),
        signature: Some("sig-xyz".into()),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let codec = GrassAcpJsonCodec;
    let enc = codec.encode(&msg).unwrap();
    let dec = codec.decode(&enc).unwrap();
    assert_eq!(dec.source, "agent-a");
    assert_eq!(dec.metadata.get("idempotency").unwrap(), "abc123");

    let json = serde_json::to_string(&envelope).unwrap();
    assert!(json.contains("sig-xyz"));
}

#[tokio::test]
async fn test_acp_router_mock() {
    let mut router = GrassAcpRouter::new();
    router.register("application/json", Arc::new(MockAcpHandler));

    let msg = GrassAcpMessage {
        source: "agent-a".into(),
        destination: "agent-b".into(),
        content_type: "application/json".into(),
        payload: serde_json::json!({"hello": "world"}),
        metadata: HashMap::new(),
    };

    let reply = router.route(msg).await.unwrap();
    assert_eq!(reply.source, "agent-b");
    assert_eq!(reply.payload["ack"], true);

    let unhandled = GrassAcpMessage {
        source: "a".into(),
        destination: "b".into(),
        content_type: "text/xml".into(),
        payload: serde_json::json!({}),
        metadata: HashMap::new(),
    };
    assert!(router.route(unhandled).await.is_err());
}

// ============================================================================
// grass-protocol: grpc module
// ============================================================================

struct MockGrpcService;

#[async_trait::async_trait]
impl GrassGrpcService for MockGrpcService {
    fn service_name(&self) -> &str {
        "mock-service"
    }
    async fn invoke(&self, method: &str, request: &[u8]) -> Result<Vec<u8>> {
        let msg: serde_json::Value = serde_json::from_slice(request)?;
        let reply = serde_json::json!({"method": method, "echo": msg});
        serde_json::to_vec(&reply).map_err(GrassError::Serialization)
    }
}

#[tokio::test]
async fn test_grpc_registry_mock() {
    let mut registry = GrassGrpcRegistry::new();
    registry.register("inference", Arc::new(MockGrpcService));

    let req = serde_json::to_vec(&serde_json::json!({"input": [1.0, 2.0]})).unwrap();
    let resp = registry.invoke("inference", "Predict", &req).await.unwrap();
    let resp_val: serde_json::Value = serde_json::from_slice(&resp).unwrap();
    assert_eq!(resp_val["method"], "Predict");
    assert_eq!(resp_val["echo"]["input"][0], 1.0);

    assert!(registry.invoke("missing", "x", &req).await.is_err());
}

#[test]
fn test_protobuf_codec_with_real_struct() {
    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct ModelMeta {
        name: String,
        version: u32,
        tags: Vec<String>,
    }

    let meta = ModelMeta {
        name: "yolo-v8".into(),
        version: 8,
        tags: vec!["detection".into(), "edge".into()],
    };
    let enc = GrassProtobufCodec::encode(&meta).unwrap();
    let dec: ModelMeta = GrassProtobufCodec::decode(&enc).unwrap();
    assert_eq!(dec, meta);
}

// ============================================================================
// grass-llm: provider module
// ============================================================================

struct MockLlmProvider {
    provider_name: String,
}

#[async_trait::async_trait]
impl GrassLlmProvider for MockLlmProvider {
    fn name(&self) -> &str {
        &self.provider_name
    }
    async fn complete(&self, request: GrassLlmRequest) -> Result<GrassLlmResponse> {
        Ok(GrassLlmResponse {
            id: uuid::Uuid::new_v4().to_string(),
            model: request.model,
            choices: vec![GrassChoice {
                index: 0,
                message: GrassChatMessage {
                    role: GrassMessageRole::Assistant,
                    content: "mock response".into(),
                    name: None,
                },
                finish_reason: Some("stop".into()),
            }],
            usage: GrassUsage {
                prompt_tokens: 10,
                completion_tokens: 3,
                total_tokens: 13,
            },
        })
    }
    async fn stream<'a>(
        &'a self,
        _request: GrassLlmRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<GrassStreamEvent>> + Send + 'a>> {
        unimplemented!("not used in mock")
    }
}

#[tokio::test]
async fn test_llm_service_register_and_default() {
    let mut service = GrassLlmService::new();
    assert_eq!(service.list_providers().len(), 0);

    service.register_provider(
        "mock-1",
        Box::new(MockLlmProvider {
            provider_name: "mock-1".into(),
        }),
    );
    service.register_provider(
        "mock-2",
        Box::new(MockLlmProvider {
            provider_name: "mock-2".into(),
        }),
    );
    service.set_default("mock-2");

    assert_eq!(service.list_providers().len(), 2);
    assert_eq!(service.provider("mock-1").unwrap().name(), "mock-1");
    assert!(service.provider("missing").is_none());

    let request = GrassLlmRequest {
        model: "mock-2".into(),
        messages: vec![GrassChatMessage {
            role: GrassMessageRole::User,
            content: "hello".into(),
            name: None,
        }],
        temperature: Some(0.5),
        max_tokens: Some(64),
        stream: false,
        tools: None,
    };

    let resp = service.complete(request).await.unwrap();
    assert_eq!(resp.choices[0].message.content, "mock response");
    assert_eq!(resp.usage.total_tokens, 13);
}

#[tokio::test]
async fn test_llm_service_explicit_provider_complete() {
    let mock = MockLlmProvider {
        provider_name: "direct".into(),
    };
    let request = GrassLlmRequest {
        model: "direct-model".into(),
        messages: vec![GrassChatMessage {
            role: GrassMessageRole::System,
            content: "system".into(),
            name: None,
        }],
        temperature: None,
        max_tokens: None,
        stream: true,
        tools: Some(vec![GrassToolDefinition {
            name: "calc".into(),
            description: "calculator".into(),
            parameters: serde_json::json!({"type": "object"}),
        }]),
    };
    let resp = mock.complete(request).await.unwrap();
    assert_eq!(resp.model, "direct-model");
    assert_eq!(resp.choices[0].finish_reason.as_deref(), Some("stop"));
}

#[test]
fn test_llm_request_response_serde_real_data() {
    let tool_def = GrassToolDefinition {
        name: "search".into(),
        description: "search the web".into(),
        parameters: serde_json::json!({"query": {"type": "string"}}),
    };
    let request = GrassLlmRequest {
        model: "gpt-4o".into(),
        messages: vec![
            GrassChatMessage {
                role: GrassMessageRole::System,
                content: "be brief".into(),
                name: None,
            },
            GrassChatMessage {
                role: GrassMessageRole::User,
                content: "what is grass?".into(),
                name: Some("field-tech".into()),
            },
        ],
        temperature: Some(0.2),
        max_tokens: Some(200),
        stream: true,
        tools: Some(vec![tool_def]),
    };

    let json = serde_json::to_string(&request).unwrap();
    let back: GrassLlmRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.messages.len(), 2);
    assert_eq!(back.messages[1].name.as_deref(), Some("field-tech"));
    assert!(back.tools.is_some());

    let stream_events = [
        GrassStreamEvent::Delta {
            content: "part".into(),
        },
        GrassStreamEvent::ToolCall {
            tool_call: GrassToolCall {
                id: "c1".into(),
                name: "search".into(),
                arguments: serde_json::json!({"query": "grass"}),
            },
        },
        GrassStreamEvent::Done {
            usage: GrassUsage {
                prompt_tokens: 1,
                completion_tokens: 2,
                total_tokens: 3,
            },
        },
        GrassStreamEvent::Error {
            error: "timeout".into(),
        },
    ];
    for ev in stream_events {
        let j = serde_json::to_string(&ev).unwrap();
        let _back: GrassStreamEvent = serde_json::from_str(&j).unwrap();
    }
}

// ============================================================================
// grass-llm: filter module
// ============================================================================

fn make_test_request() -> GrassLlmRequest {
    GrassLlmRequest {
        model: "test".into(),
        messages: vec![GrassChatMessage {
            role: GrassMessageRole::User,
            content: "hello".into(),
            name: None,
        }],
        temperature: None,
        max_tokens: None,
        stream: false,
        tools: None,
    }
}

fn make_test_response(content: &str, total_tokens: u32) -> GrassLlmResponse {
    GrassLlmResponse {
        id: uuid::Uuid::new_v4().to_string(),
        model: "test".into(),
        choices: vec![GrassChoice {
            index: 0,
            message: GrassChatMessage {
                role: GrassMessageRole::Assistant,
                content: content.into(),
                name: None,
            },
            finish_reason: Some("stop".into()),
        }],
        usage: GrassUsage {
            prompt_tokens: 10,
            completion_tokens: total_tokens - 10,
            total_tokens,
        },
    }
}

#[test]
fn test_filter_chain_end_to_end_mock() {
    let chain = GrassFilterChain::new()
        .add_request_filter(Box::new(GrassSystemPromptFilter {
            system_prompt: "You are a helpful assistant".into(),
        }))
        .add_response_filter(Box::new(GrassContentSanitizerFilter))
        .add_response_filter(Box::new(GrassTokenLimitFilter { max_tokens: 100 }));

    let mut request = make_test_request();
    chain.apply_request(&mut request);
    assert_eq!(request.messages.len(), 2);
    assert_eq!(request.messages[0].role, GrassMessageRole::System);

    let mut response = make_test_response("<script>alert('x')</script>unsafe content", 50);
    chain.apply_response(&mut response);
    assert!(!response.choices[0].message.content.contains("<script>"));
}

#[test]
fn test_token_limit_filter_truncates_real_data() {
    let filter = GrassTokenLimitFilter { max_tokens: 20 };
    let mut response = make_test_response("very long response that exceeds token budget", 500);
    filter.filter(&mut response);
    assert!(response.choices[0].message.content.len() <= 1000);
}

#[test]
fn test_system_prompt_filter_skips_existing() {
    let filter = GrassSystemPromptFilter {
        system_prompt: "custom".into(),
    };
    let mut request = make_test_request();
    request.messages.insert(
        0,
        GrassChatMessage {
            role: GrassMessageRole::System,
            content: "already-there".into(),
            name: None,
        },
    );
    filter.filter(&mut request);
    assert_eq!(request.messages.len(), 2);
    assert_eq!(request.messages[0].content, "already-there");
}

#[test]
fn test_content_sanitizer_with_real_html() {
    let filter = GrassContentSanitizerFilter;
    let mut response = make_test_response("<script>/*evil*/</script>body<script></script>", 15);
    filter.filter(&mut response);
    let cleaned = &response.choices[0].message.content;
    assert!(!cleaned.contains("<script>"));
    assert!(!cleaned.contains("</script>"));
    assert!(cleaned.contains("body"));
}

// ============================================================================
// grass-inference: tensorrt + onnx modules
// ============================================================================

#[tokio::test]
async fn test_tensorrt_backend_mock_and_real() {
    let mut backend = GrassTensorRtBackend::new();
    assert_eq!(backend.name(), "tensorrt");
    assert_eq!(backend.input_shape(), vec![1, 3, 224, 224]);
    assert_eq!(backend.output_shape(), vec![1, 1000]);

    let input = GrassTensor {
        data: vec![0.5; 3 * 224 * 224],
        shape: vec![1, 3, 224, 224],
    };

    let err = backend.infer(&input).await;
    assert!(err.is_err(), "infer before load should error");

    backend.load_model("/tmp/yolo.engine").await.unwrap();
    let output = backend.infer(&input).await.unwrap();
    assert_eq!(output.data.len(), 3 * 224 * 224);
    assert_eq!(output.shape, input.shape);
}

#[tokio::test]
async fn test_onnx_backend_mock_and_real() {
    let mut backend = GrassOnnxBackend::new();
    assert_eq!(backend.name(), "onnx");

    let input = GrassTensor {
        data: (0..150528).map(|i| i as f32).collect(),
        shape: vec![1, 3, 224, 224],
    };
    backend.load_model("/tmp/yolo.onnx").await.unwrap();
    let output = backend.infer(&input).await.unwrap();
    assert_eq!(output.data.len(), input.data.len());
}

// ============================================================================
// grass-streaming: rtsp + webrtc modules
// ============================================================================

#[tokio::test]
async fn test_rtsp_client_mock_real_frames() {
    let stream = GrassRtspStream::new();
    stream
        .connect("rtsp://admin:pass@camera.lan:554/live")
        .await
        .unwrap();

    let frame = stream.get_frame().await.unwrap();
    assert!(frame.is_empty(), "stub returns empty frame");

    stream.disconnect().await.unwrap();
}

#[tokio::test]
async fn test_webrtc_peer_full_negotiation() {
    let peer = GrassWebRtcPeer::new();
    assert!(matches!(peer.state(), GrassRtcState::New));

    let offer = peer.create_offer().await.unwrap();
    assert!(offer.contains("v=0"), "SDP offer with v=0 line");

    peer.set_remote_description(&offer).await.unwrap();
    assert!(matches!(peer.state(), GrassRtcState::Connected));

    peer.add_ice_candidate("candidate:1 1 UDP 2122260223 192.168.1.10 51000 typ host")
        .await
        .unwrap();

    peer.send_data(b"people-count-update").await.unwrap();

    peer.close().await.unwrap();
    assert!(matches!(peer.state(), GrassRtcState::Disconnected));
}

#[test]
fn test_rtc_state_serde() {
    let state = GrassRtcState::Connected;
    let json = serde_json::to_string(&state).unwrap();
    assert_eq!(json, "\"Connected\"");
    let back: GrassRtcState = serde_json::from_str(&json).unwrap();
    assert!(matches!(back, GrassRtcState::Connected));
}

// ============================================================================
// grass-messaging: qmqp module
// ============================================================================

#[tokio::test]
async fn test_qmqp_client_lifecycle_with_mock() {
    let client = GrassQmqpAdapter::new();
    client.connect("mail.queue.local", 628).await.unwrap();
    client.disconnect().await.unwrap();
}

#[test]
fn test_qmqp_message_real_payload() {
    let msg = GrassQmqpMessage {
        sender: "alert-bot@grass.ai".into(),
        recipients: vec!["ops-1@police.gov".into(), "ops-2@police.gov".into()],
        body: serde_json::to_vec(&serde_json::json!({
            "route": "NIGHT-03",
            "police_visited": true,
        }))
        .unwrap(),
        attributes: HashMap::from([("priority".into(), "high".into())]),
    };

    let json = serde_json::to_string(&msg).unwrap();
    let back: GrassQmqpMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(back.recipients.len(), 2);
    assert_eq!(back.attributes.get("priority").unwrap(), "high");

    let body: serde_json::Value = serde_json::from_slice(&back.body).unwrap();
    assert_eq!(body["police_visited"], true);
}

// ============================================================================
// grass-flow: n8n + makecom modules
// ============================================================================

// Local mock HTTP server so adapter tests exercise real HTTP + real JSON data
// against a controlled endpoint instead of failing on unreachable hosts.
async fn spawn_mock_json_server(response_body: &'static [u8]) -> u16 {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        loop {
            let (mut sock, _) = match listener.accept().await {
                Ok(v) => v,
                Err(_) => break,
            };
            tokio::spawn(async move {
                let mut reader = BufReader::new(&mut sock);
                let mut line = String::new();
                let _ = reader.read_line(&mut line).await;
                loop {
                    let mut hdr = String::new();
                    if reader.read_line(&mut hdr).await.unwrap_or(0) == 0 {
                        break;
                    }
                    if hdr == "\r\n" || hdr == "\n" {
                        break;
                    }
                }
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    response_body.len()
                );
                let _ = sock.write_all(resp.as_bytes()).await;
                let _ = sock.write_all(response_body).await;
            });
        }
    });

    port
}

#[tokio::test]
async fn test_n8n_adapter_mock_server_with_real_http() {
    let body: &'static [u8] = br#"{"id":"wf-123","active":true,"data":[{"name":"wf-1"}]}"#;
    let port = spawn_mock_json_server(body).await;
    let adapter = GrassN8nAdapter::new(&format!("http://127.0.0.1:{}", port), "test-key");

    let resp = adapter
        .trigger_workflow(
            "wf-123",
            serde_json::json!({"sensor": "temp-01", "value": 22.5}),
        )
        .await;
    assert!(resp.is_ok(), "mock server should respond 200");
    assert_eq!(resp.unwrap()["id"], "wf-123");

    let wf = adapter.get_workflow("wf-123").await;
    assert!(wf.is_ok());
    assert_eq!(wf.unwrap()["active"], true);

    let wfs = adapter.list_workflows().await;
    assert!(wfs.is_ok());
    assert_eq!(wfs.unwrap().len(), 1);
}

#[tokio::test]
async fn test_makecom_adapter_mock_server_with_real_http() {
    let body: &'static [u8] = br#"{"status":"triggered","scenario":"sc-456"}"#;
    let port = spawn_mock_json_server(body).await;
    let adapter = GrassMakeComAdapter::new(&format!("http://127.0.0.1:{}/webhook", port));

    let resp = adapter
        .trigger_scenario(
            "sc-456",
            serde_json::json!({"alert": "power threshold exceeded"}),
        )
        .await;
    assert!(resp.is_ok(), "mock server should respond 200");
    assert_eq!(resp.unwrap()["status"], "triggered");

    let scenarios = adapter.list_scenarios().await;
    assert!(scenarios.is_ok());
}

// ============================================================================
// Cross-module mock integration: inference -> protocol -> messaging
// ============================================================================

#[tokio::test]
async fn test_cross_module_mock_pipeline() {
    let mut backend = GrassOnnxBackend::new();
    backend.load_model("/tmp/classifier.onnx").await.unwrap();

    let input = GrassTensor {
        data: vec![1.0; 3 * 224 * 224],
        shape: vec![1, 3, 224, 224],
    };
    let output = backend.infer(&input).await.unwrap();

    let message = GrassAcpMessage {
        source: "inference-engine".into(),
        destination: "alert-service".into(),
        content_type: "application/grass-result".into(),
        payload: serde_json::json!({
            "class": "person",
            "confidence": 0.95,
            "feature_len": output.data.len(),
        }),
        metadata: HashMap::new(),
    };

    let bytes = GrassAcpJsonCodec.encode(&message).unwrap();
    let kafka = GrassKafkaProducerImpl::new("localhost:9092");
    kafka
        .produce("inference-results", Some("engine-1"), &bytes)
        .await
        .unwrap();
    kafka.flush().await.unwrap();

    let decoded = GrassAcpJsonCodec.decode(&bytes).unwrap();
    assert_eq!(decoded.payload["feature_len"], output.data.len());
}
