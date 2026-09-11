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
use grass_core::{GrassError, Result};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrassEthBlock {
    pub number: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: u64,
    pub transactions: Vec<GrassEthTransaction>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrassEthTransaction {
    pub hash: String,
    pub from: String,
    pub to: Option<String>,
    pub value: String,
    pub gas: u64,
    pub input: String,
}

#[async_trait]
pub trait GrassEthProvider: Send + Sync {
    async fn get_block_number(&self) -> Result<u64>;
    async fn get_block(&self, number: u64) -> Result<GrassEthBlock>;
    async fn get_transaction(&self, hash: &str) -> Result<GrassEthTransaction>;
    async fn send_raw_transaction(&self, data: &str) -> Result<String>;
}

pub struct GrassEthHttpProvider {
    rpc_url: String,
    http: reqwest::Client,
}

impl GrassEthHttpProvider {
    pub fn new(rpc_url: &str) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            http: reqwest::Client::new(),
        }
    }

    async fn rpc_call(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });
        let resp: serde_json::Value = self
            .http
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| GrassError::Blockchain(e.to_string()))?
            .json()
            .await
            .map_err(|e| GrassError::Blockchain(e.to_string()))?;
        Ok(resp["result"].clone())
    }
}

#[async_trait]
impl GrassEthProvider for GrassEthHttpProvider {
    async fn get_block_number(&self) -> Result<u64> {
        let result = self
            .rpc_call("eth_blockNumber", serde_json::json!([]))
            .await?;
        let hex = result.as_str().unwrap_or("0x0");
        Ok(u64::from_str_radix(hex.trim_start_matches("0x"), 16).unwrap_or(0))
    }

    async fn get_block(&self, number: u64) -> Result<GrassEthBlock> {
        let hex = format!("0x{:x}", number);
        let result = self
            .rpc_call("eth_getBlockByNumber", serde_json::json!([hex, true]))
            .await?;
        Ok(GrassEthBlock {
            number,
            hash: result["hash"].as_str().unwrap_or("").to_string(),
            parent_hash: result["parentHash"].as_str().unwrap_or("").to_string(),
            timestamp: u64::from_str_radix(
                result["timestamp"]
                    .as_str()
                    .unwrap_or("0x0")
                    .trim_start_matches("0x"),
                16,
            )
            .unwrap_or(0),
            transactions: vec![],
        })
    }

    async fn get_transaction(&self, hash: &str) -> Result<GrassEthTransaction> {
        let result = self
            .rpc_call("eth_getTransactionByHash", serde_json::json!([hash]))
            .await?;
        Ok(GrassEthTransaction {
            hash: hash.to_string(),
            from: result["from"].as_str().unwrap_or("").to_string(),
            to: result["to"].as_str().map(String::from),
            value: result["value"].as_str().unwrap_or("0x0").to_string(),
            gas: 0,
            input: result["input"].as_str().unwrap_or("").to_string(),
        })
    }

    async fn send_raw_transaction(&self, data: &str) -> Result<String> {
        let result = self
            .rpc_call("eth_sendRawTransaction", serde_json::json!([data]))
            .await?;
        Ok(result.as_str().unwrap_or("").to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eth_provider_name() {
        let provider = GrassEthHttpProvider::new("http://localhost:8545");
        assert_eq!(provider.rpc_url, "http://localhost:8545");
    }
}
