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
    GrassAgent, GrassAgentBuilder, GrassTaskPayload, GrassTaskResult, GrassTaskStatus,
};
use grass_iot::{GrassMqttBroker, GrassMqttClient};
use grass_messaging::{GrassKafkaProducer, GrassKafkaProducerImpl};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct SensorReading {
    sensor_id: String,
    sensor_type: String,
    value: f64,
    unit: String,
    location: String,
    timestamp: String,
}

struct InfluxDbPoint {
    measurement: String,
    tags: HashMap<String, String>,
    fields: HashMap<String, f64>,
    timestamp_ns: i64,
}

struct SensorPipelineAgent {
    metadata: grass_core::GrassAgentMetadata,
    mqtt: Arc<GrassMqttBroker>,
    kafka: Arc<GrassKafkaProducerImpl>,
    readings: Arc<RwLock<Vec<SensorReading>>>,
    influx_buffer: Arc<RwLock<Vec<InfluxDbPoint>>>,
}

#[async_trait::async_trait]
impl GrassAgent for SensorPipelineAgent {
    fn metadata(&self) -> &grass_core::GrassAgentMetadata {
        &self.metadata
    }

    fn state(&self) -> grass_core::GrassAgentState {
        grass_core::GrassAgentState::Running
    }

    async fn start(&mut self) -> grass_core::Result<()> {
        self.mqtt.connect("localhost", 1883).await?;
        self.mqtt.subscribe("sensors/temperature").await?;
        self.mqtt.subscribe("sensors/rainfall").await?;
        tracing::info!("MQTT connected and subscribed to sensor topics");
        Ok(())
    }

    async fn stop(&mut self) -> grass_core::Result<()> {
        self.mqtt.disconnect().await?;
        tracing::info!("MQTT disconnected");
        Ok(())
    }

    async fn execute(&self, task: GrassTaskPayload) -> grass_core::Result<GrassTaskResult> {
        match task.action.as_str() {
            "read_temperature" => {
                let reading = SensorReading {
                    sensor_id: "TEMP-SENSOR-001".into(),
                    sensor_type: "temperature".into(),
                    value: 23.7,
                    unit: "celsius".into(),
                    location: "warehouse-north".into(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };

                self.readings.write().await.push(reading.clone());

                let payload = serde_json::json!({
                    "sensor_id": reading.sensor_id,
                    "type": reading.sensor_type,
                    "value": reading.value,
                    "unit": reading.unit,
                    "location": reading.location,
                    "timestamp": reading.timestamp,
                });

                let json_bytes = serde_json::to_vec(&payload)?;
                self.mqtt
                    .publish("sensors/temperature", &json_bytes)
                    .await?;
                self.kafka
                    .produce("sensor-readings", Some(&reading.sensor_id), &json_bytes)
                    .await?;

                tracing::info!(
                    "Temperature: {} {} at {}",
                    reading.value,
                    reading.unit,
                    reading.location
                );

                let influx_point = InfluxDbPoint {
                    measurement: "temperature".into(),
                    tags: HashMap::from([
                        ("sensor_id".into(), reading.sensor_id),
                        ("location".into(), reading.location),
                    ]),
                    fields: HashMap::from([("value".into(), reading.value)]),
                    timestamp_ns: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
                };
                self.influx_buffer.write().await.push(influx_point);

                Ok(GrassTaskResult {
                    task_id: task.task_id,
                    agent_id: self.metadata.id,
                    status: GrassTaskStatus::Completed,
                    output: serde_json::json!({
                        "readings": 1,
                        "kafka_sent": true,
                        "mqtt_sent": true,
                    }),
                    error: None,
                    duration_ms: 10,
                })
            }
            "read_rainfall" => {
                let reading = SensorReading {
                    sensor_id: "RAIN-GAUGE-001".into(),
                    sensor_type: "rainfall".into(),
                    value: 12.5,
                    unit: "mm".into(),
                    location: "weather-station-1".into(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };

                self.readings.write().await.push(reading.clone());

                let payload = serde_json::json!({
                    "sensor_id": reading.sensor_id,
                    "type": reading.sensor_type,
                    "value": reading.value,
                    "unit": reading.unit,
                    "location": reading.location,
                    "daily_total_mm": reading.value,
                    "timestamp": reading.timestamp,
                });

                let json_bytes = serde_json::to_vec(&payload)?;
                self.mqtt.publish("sensors/rainfall", &json_bytes).await?;
                self.kafka
                    .produce("sensor-readings", Some(&reading.sensor_id), &json_bytes)
                    .await?;

                tracing::info!(
                    "Rainfall: {} {} at {}",
                    reading.value,
                    reading.unit,
                    reading.location
                );

                let influx_point = InfluxDbPoint {
                    measurement: "rainfall".into(),
                    tags: HashMap::from([
                        ("sensor_id".into(), reading.sensor_id),
                        ("location".into(), reading.location),
                    ]),
                    fields: HashMap::from([("daily_total_mm".into(), reading.value)]),
                    timestamp_ns: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
                };
                self.influx_buffer.write().await.push(influx_point);

                Ok(GrassTaskResult {
                    task_id: task.task_id,
                    agent_id: self.metadata.id,
                    status: GrassTaskStatus::Completed,
                    output: serde_json::json!({
                        "readings": 1,
                        "kafka_sent": true,
                        "mqtt_sent": true,
                    }),
                    error: None,
                    duration_ms: 10,
                })
            }
            "flush_to_influxdb" => {
                let buffer: Vec<InfluxDbPoint> =
                    self.influx_buffer.write().await.drain(..).collect();
                let count = buffer.len();
                for point in &buffer {
                    tracing::info!(
                        "InfluxDB write: {} -> {}",
                        point.measurement,
                        serde_json::to_string(&point.fields).unwrap_or_default()
                    );
                }
                Ok(GrassTaskResult {
                    task_id: task.task_id,
                    agent_id: self.metadata.id,
                    status: GrassTaskStatus::Completed,
                    output: serde_json::json!({
                        "points_written": count,
                        "influxdb_endpoint": "http://localhost:8086",
                        "database": "sensor_data",
                    }),
                    error: None,
                    duration_ms: 5,
                })
            }
            _ => Ok(GrassTaskResult {
                task_id: task.task_id,
                agent_id: self.metadata.id,
                status: GrassTaskStatus::Failed,
                output: serde_json::json!({}),
                error: Some(format!("unknown action: {}", task.action)),
                duration_ms: 0,
            }),
        }
    }

    async fn handle_message(
        &self,
        msg: grass_core::GrassMessage,
    ) -> grass_core::Result<Option<grass_core::GrassMessage>> {
        tracing::info!("Received message: {}", msg.topic);
        Ok(None)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("=== IoT Sensor -> Kafka/MQTT -> InfluxDB Pipeline ===");

    let meta = GrassAgentBuilder::new("sensor-pipeline")
        .version("1.0.0")
        .description("Reads sensors, sends via Kafka/MQTT, stores in InfluxDB")
        .tag("iot")
        .tag("sensors")
        .tag("influxdb")
        .build();

    let mut agent = SensorPipelineAgent {
        metadata: meta,
        mqtt: Arc::new(GrassMqttBroker::new("sensor-agent")),
        kafka: Arc::new(GrassKafkaProducerImpl::new("localhost:9092")),
        readings: Arc::new(RwLock::new(Vec::new())),
        influx_buffer: Arc::new(RwLock::new(Vec::new())),
    };

    agent.start().await?;

    for cycle in 1..=5 {
        tracing::info!("\n--- Cycle {} ---", cycle);

        let temp_result = agent
            .execute(GrassTaskPayload {
                task_id: uuid::Uuid::new_v4(),
                agent_id: agent.metadata().id,
                action: "read_temperature".into(),
                input: serde_json::json!({}),
                metadata: HashMap::new(),
            })
            .await?;
        tracing::info!("Temperature -> Kafka/MQTT: {:?}", temp_result.output);

        let rain_result = agent
            .execute(GrassTaskPayload {
                task_id: uuid::Uuid::new_v4(),
                agent_id: agent.metadata().id,
                action: "read_rainfall".into(),
                input: serde_json::json!({}),
                metadata: HashMap::new(),
            })
            .await?;
        tracing::info!("Rainfall -> Kafka/MQTT: {:?}", rain_result.output);

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    let flush_result = agent
        .execute(GrassTaskPayload {
            task_id: uuid::Uuid::new_v4(),
            agent_id: agent.metadata().id,
            action: "flush_to_influxdb".into(),
            input: serde_json::json!({}),
            metadata: HashMap::new(),
        })
        .await?;
    tracing::info!(
        "InfluxDB flush: {}",
        serde_json::to_string_pretty(&flush_result.output)?
    );

    agent.stop().await?;

    tracing::info!("=== Pipeline Complete ===");
    Ok(())
}
