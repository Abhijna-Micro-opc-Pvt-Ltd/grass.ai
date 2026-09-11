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
use grass_hooks::*;
use grass_iot::*;
use grass_messaging::*;
use grass_orchestrator::*;
use grass_runtime::*;
use std::collections::HashMap;

// ============================================================================
// grass-core integration tests
// ============================================================================

#[test]
fn test_agent_builder_full_config() {
    let meta = GrassAgentBuilder::new("integration-agent")
        .version("2.0.0")
        .description("Integration test agent")
        .tag("test")
        .tag("integration")
        .config("key1", serde_json::json!("val1"))
        .config("key2", serde_json::json!(42))
        .build();

    assert_eq!(meta.name, "integration-agent");
    assert_eq!(meta.version, "2.0.0");
    assert_eq!(meta.description, "Integration test agent");
    assert_eq!(meta.tags.len(), 2);
    assert_eq!(meta.config.get("key1").unwrap(), "val1");
    assert_eq!(meta.config.get("key2").unwrap(), 42);
}

#[test]
fn test_task_payload_roundtrip() {
    let mut metadata = HashMap::new();
    metadata.insert("source".into(), serde_json::json!("test"));

    let payload = GrassTaskPayload {
        task_id: uuid::Uuid::new_v4(),
        agent_id: uuid::Uuid::new_v4(),
        action: "process_data".into(),
        input: serde_json::json!({"data": [1, 2, 3]}),
        metadata,
    };

    let json = serde_json::to_string(&payload).unwrap();
    let deserialized: GrassTaskPayload = serde_json::from_str(&json).unwrap();

    assert_eq!(payload.task_id, deserialized.task_id);
    assert_eq!(payload.action, deserialized.action);
    assert_eq!(
        payload.metadata.get("source").unwrap(),
        deserialized.metadata.get("source").unwrap()
    );
}

#[test]
fn test_task_result_all_statuses() {
    let statuses = vec![
        GrassTaskStatus::Pending,
        GrassTaskStatus::Running,
        GrassTaskStatus::Completed,
        GrassTaskStatus::Failed,
        GrassTaskStatus::Cancelled,
    ];

    for status in statuses {
        let result = GrassTaskResult {
            task_id: uuid::Uuid::new_v4(),
            agent_id: uuid::Uuid::new_v4(),
            status: status.clone(),
            output: serde_json::json!({}),
            error: None,
            duration_ms: 100,
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: GrassTaskResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.status, deserialized.status);
    }
}

#[test]
fn test_grass_message_construction() {
    let from = uuid::Uuid::new_v4();
    let msg = GrassMessage::new(from, "data.sync", serde_json::json!({"key": "value"}));

    assert_eq!(msg.topic, "data.sync");
    assert_eq!(msg.from, from);
    assert!(msg.headers.is_empty());
}

#[test]
fn test_grass_message_with_headers() {
    let from = uuid::Uuid::new_v4();
    let to = uuid::Uuid::new_v4();
    let msg = GrassMessage::new(from, "task.assign", serde_json::json!({}))
        .with_header("priority".into(), "high".into())
        .with_header("ttl".into(), "30".into())
        .directed_to(to);

    assert_eq!(msg.headers.len(), 2);
    assert_eq!(msg.headers.get("priority").unwrap(), "high");
    assert_eq!(msg.to, Some(to));
}

#[test]
fn test_error_variants() {
    let errors = vec![
        GrassError::Agent("agent err".into()),
        GrassError::Task("task err".into()),
        GrassError::Plugin("plugin err".into()),
        GrassError::Hook("hook err".into()),
        GrassError::Protocol("protocol err".into()),
        GrassError::Ipc("ipc err".into()),
        GrassError::Sandbox("sandbox err".into()),
        GrassError::Llm("llm err".into()),
        GrassError::Inference("inference err".into()),
        GrassError::Blockchain("blockchain err".into()),
        GrassError::Iot("iot err".into()),
        GrassError::Streaming("streaming err".into()),
        GrassError::VectorDb("vector err".into()),
        GrassError::Integration("integration err".into()),
        GrassError::Flow("flow err".into()),
        GrassError::Messaging("messaging err".into()),
    ];

    for err in errors {
        let msg = err.to_string();
        assert!(!msg.is_empty());
    }
}

#[test]
fn test_sandbox_config_custom() {
    let cfg = GrassSandboxConfig {
        backend: GrassSandboxBackend::Kata,
        rootfs: Some("/var/rootfs".into()),
        max_memory_bytes: 1024 * 1024 * 1024,
        max_cpu_shares: 2048,
        network_isolation: false,
        allowed_syscalls: vec!["read".into(), "write".into()],
        env_vars: HashMap::from([("ENV".into(), "production".into())]),
    };

    assert_eq!(cfg.backend, GrassSandboxBackend::Kata);
    assert_eq!(cfg.max_memory_bytes, 1024 * 1024 * 1024);
    assert!(!cfg.network_isolation);
    assert_eq!(cfg.allowed_syscalls.len(), 2);
}

// ============================================================================
// grass-hooks integration tests
// ============================================================================

struct TestNotificationHook {
    id: HookId,
    name: String,
    trigger: GrassHookTrigger,
}

#[async_trait::async_trait]
impl GrassHook for TestNotificationHook {
    fn id(&self) -> HookId {
        self.id
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn trigger(&self) -> GrassHookTrigger {
        self.trigger.clone()
    }
    async fn execute(&self, payload: &GrassHookPayload) -> grass_core::Result<serde_json::Value> {
        Ok(serde_json::json!({
            "hook": self.name,
            "triggered": true,
            "data": payload.data,
        }))
    }
}

#[tokio::test]
async fn test_hook_registry_multiple_hooks() {
    let registry = GrassHookRegistry::new();

    let hook1: std::sync::Arc<dyn GrassHook> = std::sync::Arc::new(TestNotificationHook {
        id: uuid::Uuid::new_v4(),
        name: "slack-notifier".into(),
        trigger: GrassHookTrigger::OnTaskComplete,
    });

    let hook2: std::sync::Arc<dyn GrassHook> = std::sync::Arc::new(TestNotificationHook {
        id: uuid::Uuid::new_v4(),
        name: "email-notifier".into(),
        trigger: GrassHookTrigger::OnTaskComplete,
    });

    let hook3: std::sync::Arc<dyn GrassHook> = std::sync::Arc::new(TestNotificationHook {
        id: uuid::Uuid::new_v4(),
        name: "agent-start-hook".into(),
        trigger: GrassHookTrigger::OnAgentStart,
    });

    registry.register(hook1).await;
    registry.register(hook2).await;
    registry.register(hook3).await;

    let all_hooks = registry.list_hooks().await;
    assert_eq!(all_hooks.len(), 3);

    let payload = GrassHookPayload {
        trigger: GrassHookTrigger::OnTaskComplete,
        data: serde_json::json!({"task_id": "test-123"}),
        metadata: HashMap::new(),
    };

    let results = registry
        .fire(&GrassHookTrigger::OnTaskComplete, &payload)
        .await
        .unwrap();
    assert_eq!(results.len(), 2);

    let results_start = registry
        .fire(&GrassHookTrigger::OnAgentStart, &payload)
        .await
        .unwrap();
    assert_eq!(results_start.len(), 1);
}

#[tokio::test]
async fn test_hook_unregister() {
    let registry = GrassHookRegistry::new();
    let hook_id = uuid::Uuid::new_v4();

    let hook: std::sync::Arc<dyn GrassHook> = std::sync::Arc::new(TestNotificationHook {
        id: hook_id,
        name: "temp-hook".into(),
        trigger: GrassHookTrigger::OnTaskFailed,
    });

    registry.register(hook).await;
    assert_eq!(registry.list_hooks().await.len(), 1);

    registry.unregister(hook_id).await;
    assert_eq!(registry.list_hooks().await.len(), 0);
}

// ============================================================================
// grass-iot integration tests
// ============================================================================

#[tokio::test]
async fn test_modbus_lifecycle() {
    let client = GrassModbusClient::new();

    client.connect("192.168.1.100", 502).await.unwrap();

    let registers = client.read_registers(1, 0, 10).await.unwrap();
    assert_eq!(registers.len(), 10);

    let single = client
        .read_register(1, 0, GrassModbusFunction::ReadInputRegisters)
        .await
        .unwrap();
    assert_eq!(single, 0);

    client.write_register(1, 100, 42).await.unwrap();

    client.disconnect().await.unwrap();
}

#[test]
fn test_modbus_function_serialization_roundtrip() {
    let functions = vec![
        GrassModbusFunction::ReadCoils,
        GrassModbusFunction::ReadHoldingRegisters,
        GrassModbusFunction::ReadInputRegisters,
        GrassModbusFunction::WriteSingleCoil,
        GrassModbusFunction::WriteSingleRegister,
        GrassModbusFunction::WriteMultipleCoils,
        GrassModbusFunction::WriteMultipleRegisters,
    ];

    for func in functions {
        let json = serde_json::to_string(&func).unwrap();
        let deserialized: GrassModbusFunction = serde_json::from_str(&json).unwrap();
        let json2 = serde_json::to_string(&deserialized).unwrap();
        assert_eq!(json, json2);
    }
}

#[tokio::test]
async fn test_mqtt_full_lifecycle() {
    let broker = GrassMqttBroker::new("test-integration-client");

    broker.connect("localhost", 1883).await.unwrap();
    broker
        .subscribe("factory/sensors/temperature")
        .await
        .unwrap();
    broker.subscribe("factory/sensors/rainfall").await.unwrap();

    let temp_data = serde_json::json!({"value": 23.5, "unit": "celsius"});
    broker
        .publish(
            "factory/sensors/temperature",
            &serde_json::to_vec(&temp_data).unwrap(),
        )
        .await
        .unwrap();

    let rain_data = serde_json::json!({"value": 5.2, "unit": "mm"});
    broker
        .publish(
            "factory/sensors/rainfall",
            &serde_json::to_vec(&rain_data).unwrap(),
        )
        .await
        .unwrap();

    broker.disconnect().await.unwrap();
}

// ============================================================================
// grass-messaging integration tests
// ============================================================================

#[tokio::test]
async fn test_kafka_producer_full_cycle() {
    let producer = GrassKafkaProducerImpl::new("localhost:9092");

    let readings = vec![
        serde_json::json!({"sensor": "temp-001", "value": 22.3}),
        serde_json::json!({"sensor": "temp-002", "value": 24.1}),
        serde_json::json!({"sensor": "rain-001", "value": 8.7}),
    ];

    for (i, reading) in readings.iter().enumerate() {
        let key = format!("reading-{}", i);
        let bytes = serde_json::to_vec(reading).unwrap();
        producer
            .produce("sensor-data", Some(&key), &bytes)
            .await
            .unwrap();
    }

    producer.flush().await.unwrap();
}

#[tokio::test]
async fn test_kafka_consumer_subscribe() {
    let consumer = GrassKafkaConsumerImpl::new("localhost:9092", "test-group");
    consumer
        .subscribe(&["sensor-data", "alarms"])
        .await
        .unwrap();

    let msg = consumer.poll(100).await.unwrap();
    assert!(msg.is_none());

    consumer.commit().await.unwrap();
}

// ============================================================================
// grass-orchestrator integration tests
// ============================================================================

#[tokio::test]
async fn test_crew_manager_full_workflow() {
    let crew = GrassCrewManager::new();

    let agent1 = GrassAgentBuilder::new("data-collector").build();
    let agent2 = GrassAgentBuilder::new("data-processor").build();
    let agent3 = GrassAgentBuilder::new("alert-manager").build();

    let id1 = agent1.id;
    let id2 = agent2.id;
    let id3 = agent3.id;

    crew.add_agent(agent1).await;
    crew.add_agent(agent2).await;
    crew.add_agent(agent3).await;

    assert_eq!(crew.list_agents().await.len(), 3);

    crew.assign_task("collect-sensors", id1).await.unwrap();
    crew.assign_task("process-data", id2).await.unwrap();
    crew.assign_task("send-alerts", id3).await.unwrap();
    crew.assign_task("process-data", id1).await.unwrap();

    let collectors = crew.agents_for_task("collect-sensors").await;
    assert_eq!(collectors.len(), 1);

    let processors = crew.agents_for_task("process-data").await;
    assert_eq!(processors.len(), 2);

    crew.remove_agent(id2).await;
    assert_eq!(crew.list_agents().await.len(), 2);
}

#[tokio::test]
async fn test_task_manager_full_workflow() {
    let manager = GrassTaskManager::new();

    let task1_id = uuid::Uuid::new_v4();
    let task2_id = uuid::Uuid::new_v4();
    let agent_id = uuid::Uuid::new_v4();

    manager
        .create_task(GrassTaskPayload {
            task_id: task1_id,
            agent_id,
            action: "read-sensors".into(),
            input: serde_json::json!({}),
            metadata: HashMap::new(),
        })
        .await;

    manager
        .create_task(GrassTaskPayload {
            task_id: task2_id,
            agent_id,
            action: "process-data".into(),
            input: serde_json::json!({}),
            metadata: HashMap::new(),
        })
        .await;

    assert_eq!(manager.pending_count().await, 2);

    manager
        .complete_task(GrassTaskResult {
            task_id: task1_id,
            agent_id,
            status: GrassTaskStatus::Completed,
            output: serde_json::json!({"readings": 42}),
            error: None,
            duration_ms: 150,
        })
        .await;

    assert_eq!(manager.completed_count().await, 1);

    let result = manager.get_result(task1_id).await.unwrap();
    assert_eq!(result.status, GrassTaskStatus::Completed);
    assert_eq!(result.output["readings"], 42);
}

#[tokio::test]
async fn test_task_graph_dependency_chain() {
    let graph = GrassTaskGraph::new();

    let make = |id: &str, action: &str| GrassTaskNode {
        id: id.to_string(),
        task: GrassTaskPayload {
            task_id: uuid::Uuid::new_v4(),
            agent_id: uuid::Uuid::new_v4(),
            action: action.into(),
            input: serde_json::json!({}),
            metadata: HashMap::new(),
        },
        task_type: GrassTaskType::Sequential,
        status: GrassTaskStatus::Pending,
    };

    graph.add_node(make("a", "collect")).await;
    graph.add_node(make("b", "process")).await;
    graph.add_node(make("c", "store")).await;
    graph.add_node(make("d", "notify")).await;

    graph.add_edge("a", "b").await;
    graph.add_edge("b", "c").await;
    graph.add_edge("c", "d").await;

    assert_eq!(graph.dependencies("b").await, vec!["a"]);
    assert_eq!(graph.dependencies("c").await, vec!["b"]);
    assert_eq!(graph.dependencies("d").await, vec!["c"]);

    let ready = graph.ready_nodes().await;
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "a");

    let node_b = graph.get_node("b").await.unwrap();
    assert_eq!(node_b.task.action, "process");
}

#[tokio::test]
async fn test_task_graph_parallel_branches() {
    let graph = GrassTaskGraph::new();

    let make = |id: &str| GrassTaskNode {
        id: id.to_string(),
        task: GrassTaskPayload {
            task_id: uuid::Uuid::new_v4(),
            agent_id: uuid::Uuid::new_v4(),
            action: format!("action-{}", id),
            input: serde_json::json!({}),
            metadata: HashMap::new(),
        },
        task_type: GrassTaskType::Parallel,
        status: GrassTaskStatus::Pending,
    };

    graph.add_node(make("root")).await;
    graph.add_node(make("branch-a")).await;
    graph.add_node(make("branch-b")).await;
    graph.add_node(make("branch-c")).await;
    graph.add_node(make("merge")).await;

    graph.add_edge("root", "branch-a").await;
    graph.add_edge("root", "branch-b").await;
    graph.add_edge("root", "branch-c").await;
    graph.add_edge("branch-a", "merge").await;
    graph.add_edge("branch-b", "merge").await;
    graph.add_edge("branch-c", "merge").await;

    let ready = graph.ready_nodes().await;
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "root");

    assert_eq!(graph.dependencies("merge").await.len(), 3);
}

// ============================================================================
// grass-runtime integration tests
// ============================================================================

#[tokio::test]
async fn test_executor_submit_multiple_tasks() {
    let executor = GrassTaskExecutor::new(4);

    for i in 0..5 {
        let payload = GrassTaskPayload {
            task_id: uuid::Uuid::new_v4(),
            agent_id: uuid::Uuid::new_v4(),
            action: format!("task-{}", i),
            input: serde_json::json!({"index": i}),
            metadata: HashMap::new(),
        };

        executor
            .submit(payload, |t| async move {
                Ok(GrassTaskResult {
                    task_id: t.task_id,
                    agent_id: t.agent_id,
                    status: GrassTaskStatus::Completed,
                    output: serde_json::json!({"done": true}),
                    error: None,
                    duration_ms: 10,
                })
            })
            .await
            .unwrap();
    }

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert_eq!(executor.running_count().await, 0);
}

#[tokio::test]
async fn test_executor_cancel_task() {
    let executor = GrassTaskExecutor::new(2);

    let task_id = uuid::Uuid::new_v4();
    let payload = GrassTaskPayload {
        task_id,
        agent_id: uuid::Uuid::new_v4(),
        action: "long-running".into(),
        input: serde_json::json!({}),
        metadata: HashMap::new(),
    };

    executor
        .submit(payload, |t| async move {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            Ok(GrassTaskResult {
                task_id: t.task_id,
                agent_id: t.agent_id,
                status: GrassTaskStatus::Completed,
                output: serde_json::json!({}),
                error: None,
                duration_ms: 10000,
            })
        })
        .await
        .unwrap();

    assert_eq!(executor.running_count().await, 1);
    executor.cancel(task_id).await.unwrap();
    assert_eq!(executor.running_count().await, 0);
}

// ============================================================================
// Cross-crate integration test: full pipeline
// ============================================================================

#[tokio::test]
async fn test_full_iot_pipeline_mock() {
    let modbus = GrassModbusClient::new();
    modbus.connect("192.168.1.100", 502).await.unwrap();

    let mqtt = GrassMqttBroker::new("pipeline-test");
    mqtt.connect("localhost", 1883).await.unwrap();
    mqtt.subscribe("factory/sensors").await.unwrap();

    let kafka = GrassKafkaProducerImpl::new("localhost:9092");

    let registers = modbus.read_registers(1, 0, 10).await.unwrap();
    assert_eq!(registers.len(), 10);

    let sensor_data = serde_json::json!({
        "meter_id": "PM-001",
        "power_w": 1500.0,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let bytes = serde_json::to_vec(&sensor_data).unwrap();
    mqtt.publish("factory/sensors", &bytes).await.unwrap();
    kafka
        .produce("power-data", Some("PM-001"), &bytes)
        .await
        .unwrap();
    kafka.flush().await.unwrap();

    let registry = GrassHookRegistry::new();
    let hook: std::sync::Arc<dyn GrassHook> = std::sync::Arc::new(TestNotificationHook {
        id: uuid::Uuid::new_v4(),
        name: "power-alert".into(),
        trigger: GrassHookTrigger::OnCustom("power_threshold".into()),
    });
    registry.register(hook).await;

    let hook_payload = GrassHookPayload {
        trigger: GrassHookTrigger::OnCustom("power_threshold".into()),
        data: serde_json::json!({"power": 1500.0, "threshold": 1200.0}),
        metadata: HashMap::new(),
    };

    let hook_results = registry
        .fire(&hook_payload.trigger, &hook_payload)
        .await
        .unwrap();
    assert_eq!(hook_results.len(), 1);

    modbus.disconnect().await.unwrap();
    mqtt.disconnect().await.unwrap();
}
