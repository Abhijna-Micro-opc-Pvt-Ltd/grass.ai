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
use grass_iot::{GrassModbusClient, GrassModbusFunction, GrassModbusTcpClient};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

const MITSUBISHI_SLAVE_ID: u8 = 1;
const SIEMENS_SLAVE_ID: u8 = 2;

#[derive(Clone)]
struct PowerMeterReading {
    meter_id: String,
    manufacturer: String,
    voltage: f64,
    current: f64,
    power: f64,
    energy: f64,
    power_factor: f64,
    frequency: f64,
    timestamp: String,
}

struct PowerMeterCollector {
    metadata: grass_core::GrassAgentMetadata,
    client: Arc<GrassModbusClient>,
    readings: Arc<RwLock<Vec<PowerMeterReading>>>,
}

#[async_trait::async_trait]
impl GrassAgent for PowerMeterCollector {
    fn metadata(&self) -> &grass_core::GrassAgentMetadata {
        &self.metadata
    }

    fn state(&self) -> grass_core::GrassAgentState {
        grass_core::GrassAgentState::Running
    }

    async fn start(&mut self) -> grass_core::Result<()> {
        self.client.connect("192.168.1.100", 502).await?;
        tracing::info!("Connected to Modbus TCP gateway");
        Ok(())
    }

    async fn stop(&mut self) -> grass_core::Result<()> {
        self.client.disconnect().await?;
        tracing::info!("Disconnected from Modbus TCP gateway");
        Ok(())
    }

    async fn execute(&self, task: GrassTaskPayload) -> grass_core::Result<GrassTaskResult> {
        let action = task.action.as_str();
        match action {
            "read_mitsubishi_power_meter" => {
                let registers = self
                    .client
                    .read_registers(MITSUBISHI_SLAVE_ID, 0, 20)
                    .await?;

                let voltage = decode_float(&registers, 0) as f64;
                let current = decode_float(&registers, 2) as f64;
                let power = decode_float(&registers, 4) as f64;
                let energy = decode_float(&registers, 6) as f64;
                let power_factor = decode_float(&registers, 8) as f64;
                let frequency = decode_float(&registers, 10) as f64;

                let reading = PowerMeterReading {
                    meter_id: "MITSUBISHI-PM-001".into(),
                    manufacturer: "Mitsubishi".into(),
                    voltage,
                    current,
                    power,
                    energy,
                    power_factor,
                    frequency,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };

                self.readings.write().await.push(reading.clone());

                Ok(GrassTaskResult {
                    task_id: task.task_id,
                    agent_id: self.metadata.id,
                    status: GrassTaskStatus::Completed,
                    output: serde_json::json!({
                        "meter": reading.meter_id,
                        "manufacturer": reading.manufacturer,
                        "voltage_v": reading.voltage,
                        "current_a": reading.current,
                        "power_w": reading.power,
                        "energy_kwh": reading.energy,
                        "power_factor": reading.power_factor,
                        "frequency_hz": reading.frequency,
                        "timestamp": reading.timestamp,
                    }),
                    error: None,
                    duration_ms: 15,
                })
            }
            "read_siemens_power_meter" => {
                let registers = self
                    .client
                    .read_registers(SIEMENS_SLAVE_ID, 100, 16)
                    .await?;

                let voltage = decode_float(&registers, 0) as f64;
                let current = decode_float(&registers, 2) as f64;
                let active_power = decode_float(&registers, 4) as f64;
                let reactive_power = decode_float(&registers, 6) as f64;
                let apparent_power = decode_float(&registers, 8) as f64;
                let energy_import = decode_float(&registers, 10) as f64;
                let energy_export = decode_float(&registers, 12) as f64;
                let power_factor = decode_float(&registers, 14) as f64;

                let reading = PowerMeterReading {
                    meter_id: "SIEMENS-PM-001".into(),
                    manufacturer: "Siemens".into(),
                    voltage,
                    current,
                    power: active_power,
                    energy: energy_import,
                    power_factor,
                    frequency: 50.0,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };

                self.readings.write().await.push(reading.clone());

                Ok(GrassTaskResult {
                    task_id: task.task_id,
                    agent_id: self.metadata.id,
                    status: GrassTaskStatus::Completed,
                    output: serde_json::json!({
                        "meter": reading.meter_id,
                        "manufacturer": reading.manufacturer,
                        "voltage_v": reading.voltage,
                        "current_a": reading.current,
                        "active_power_w": reading.power,
                        "reactive_power_var": reactive_power,
                        "apparent_power_va": apparent_power,
                        "energy_import_kwh": energy_import,
                        "energy_export_kwh": energy_export,
                        "power_factor": reading.power_factor,
                        "frequency_hz": 50.0,
                        "timestamp": reading.timestamp,
                    }),
                    error: None,
                    duration_ms: 20,
                })
            }
            "read_all_meters" => {
                let mut results = vec![];

                let mitsubishi = self
                    .client
                    .read_registers(MITSUBISHI_SLAVE_ID, 0, 20)
                    .await?;
                results.push(serde_json::json!({
                    "meter_id": "MITSUBISHI-PM-001",
                    "voltage_v": decode_float(&mitsubishi, 0) as f64,
                    "current_a": decode_float(&mitsubishi, 2) as f64,
                    "power_w": decode_float(&mitsubishi, 4) as f64,
                }));

                let siemens = self
                    .client
                    .read_registers(SIEMENS_SLAVE_ID, 100, 16)
                    .await?;
                results.push(serde_json::json!({
                    "meter_id": "SIEMENS-PM-001",
                    "voltage_v": decode_float(&siemens, 0) as f64,
                    "current_a": decode_float(&siemens, 2) as f64,
                    "power_w": decode_float(&siemens, 4) as f64,
                }));

                Ok(GrassTaskResult {
                    task_id: task.task_id,
                    agent_id: self.metadata.id,
                    status: GrassTaskStatus::Completed,
                    output: serde_json::json!({
                        "meters": results,
                        "total_readings": self.readings.read().await.len(),
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                    }),
                    error: None,
                    duration_ms: 35,
                })
            }
            _ => Ok(GrassTaskResult {
                task_id: task.task_id,
                agent_id: self.metadata.id,
                status: GrassTaskStatus::Failed,
                output: serde_json::json!({}),
                error: Some(format!("unknown action: {}", action)),
                duration_ms: 0,
            }),
        }
    }

    async fn handle_message(
        &self,
        msg: grass_core::GrassMessage,
    ) -> grass_core::Result<Option<grass_core::GrassMessage>> {
        tracing::info!("Received message on topic: {}", msg.topic);
        Ok(None)
    }
}

fn decode_float(registers: &[u16], offset: usize) -> f32 {
    if offset + 1 < registers.len() {
        let bits = ((registers[offset] as u32) << 16) | registers[offset + 1] as u32;
        f32::from_bits(bits)
    } else {
        0.0
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("=== Modbus TCP Power Meter Example ===");
    tracing::info!("Reading power values from Mitsubishi & Siemens meters");

    let meta = GrassAgentBuilder::new("power-meter-collector")
        .version("1.0.0")
        .description("Collects power data from industrial meters via Modbus TCP")
        .tag("manufacturing")
        .tag("modbus")
        .tag("power-monitoring")
        .config("gateway_ip", serde_json::json!("192.168.1.100"))
        .config("gateway_port", serde_json::json!(502))
        .build();

    let mut collector = PowerMeterCollector {
        metadata: meta,
        client: Arc::new(GrassModbusClient::new()),
        readings: Arc::new(RwLock::new(Vec::new())),
    };

    collector.start().await?;

    let task_id_mitsubishi = uuid::Uuid::new_v4();
    let result_mitsubishi = collector
        .execute(GrassTaskPayload {
            task_id: task_id_mitsubishi,
            agent_id: collector.metadata().id,
            action: "read_mitsubishi_power_meter".into(),
            input: serde_json::json!({}),
            metadata: HashMap::new(),
        })
        .await?;
    tracing::info!(
        "Mitsubishi reading: {}",
        serde_json::to_string_pretty(&result_mitsubishi.output)?
    );

    let task_id_siemens = uuid::Uuid::new_v4();
    let result_siemens = collector
        .execute(GrassTaskPayload {
            task_id: task_id_siemens,
            agent_id: collector.metadata().id,
            action: "read_siemens_power_meter".into(),
            input: serde_json::json!({}),
            metadata: HashMap::new(),
        })
        .await?;
    tracing::info!(
        "Siemens reading: {}",
        serde_json::to_string_pretty(&result_siemens.output)?
    );

    let task_id_all = uuid::Uuid::new_v4();
    let result_all = collector
        .execute(GrassTaskPayload {
            task_id: task_id_all,
            agent_id: collector.metadata().id,
            action: "read_all_meters".into(),
            input: serde_json::json!({}),
            metadata: HashMap::new(),
        })
        .await?;
    tracing::info!(
        "All meters: {}",
        serde_json::to_string_pretty(&result_all.output)?
    );

    collector.stop().await?;

    tracing::info!("=== Power Meter Example Complete ===");
    Ok(())
}
