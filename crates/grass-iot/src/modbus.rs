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

use async_trait::async_trait;
use grass_core::Result;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum GrassModbusFunction {
    ReadCoils,
    ReadHoldingRegisters,
    ReadInputRegisters,
    WriteSingleCoil,
    WriteSingleRegister,
    WriteMultipleCoils,
    WriteMultipleRegisters,
}

#[async_trait]
pub trait GrassModbusTcpClient: Send + Sync {
    async fn connect(&self, host: &str, port: u16) -> Result<()>;
    async fn read_register(
        &self,
        slave_id: u8,
        address: u16,
        function: GrassModbusFunction,
    ) -> Result<u16>;
    async fn write_register(&self, slave_id: u8, address: u16, value: u16) -> Result<()>;
    async fn read_registers(&self, slave_id: u8, address: u16, count: u16) -> Result<Vec<u16>>;
    async fn disconnect(&self) -> Result<()>;
}

pub struct GrassModbusClient {
    host: String,
    port: u16,
    connected: std::sync::atomic::AtomicBool,
}

impl GrassModbusClient {
    pub fn new() -> Self {
        Self {
            host: String::new(),
            port: 502,
            connected: std::sync::atomic::AtomicBool::new(false),
        }
    }
}

impl Default for GrassModbusClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GrassModbusTcpClient for GrassModbusClient {
    async fn connect(&self, host: &str, port: u16) -> Result<()> {
        tracing::info!("Modbus TCP connecting to {}:{}", host, port);
        self.connected
            .store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    async fn read_register(
        &self,
        _slave_id: u8,
        _address: u16,
        _function: GrassModbusFunction,
    ) -> Result<u16> {
        Ok(0)
    }

    async fn write_register(&self, _slave_id: u8, _address: u16, _value: u16) -> Result<()> {
        Ok(())
    }

    async fn read_registers(&self, _slave_id: u8, _address: u16, count: u16) -> Result<Vec<u16>> {
        Ok(vec![0; count as usize])
    }

    async fn disconnect(&self) -> Result<()> {
        self.connected
            .store(false, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modbus_function_serialization() {
        let func = GrassModbusFunction::ReadHoldingRegisters;
        let json = serde_json::to_string(&func).unwrap();
        assert!(json.contains("ReadHoldingRegisters"));
    }
}
