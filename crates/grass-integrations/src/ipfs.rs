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

#[async_trait]
pub trait GrassIpfsClient: Send + Sync {
    async fn add(&self, data: &[u8]) -> Result<String>;
    async fn get(&self, cid: &str) -> Result<Vec<u8>>;
    async fn pin(&self, cid: &str) -> Result<()>;
    async fn unpin(&self, cid: &str) -> Result<()>;
    async fn ls(&self, cid: &str) -> Result<Vec<GrassIpfsEntry>>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrassIpfsEntry {
    pub name: String,
    pub cid: String,
    pub size: u64,
}

pub struct GrassIpfsStore {
    api_url: String,
    http: reqwest::Client,
}

impl GrassIpfsStore {
    pub fn new(api_url: &str) -> Self {
        Self {
            api_url: api_url.to_string(),
            http: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl GrassIpfsClient for GrassIpfsStore {
    async fn add(&self, data: &[u8]) -> Result<String> {
        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(data.to_vec()));
        let resp: serde_json::Value = self
            .http
            .post(format!("{}/api/v0/add", self.api_url))
            .multipart(form)
            .send()
            .await
            .map_err(|e| GrassError::Integration(e.to_string()))?
            .json()
            .await
            .map_err(|e| GrassError::Integration(e.to_string()))?;
        Ok(resp["Hash"].as_str().unwrap_or("").to_string())
    }

    async fn get(&self, cid: &str) -> Result<Vec<u8>> {
        let resp = self
            .http
            .post(format!("{}/api/v0/cat?arg={}", self.api_url, cid))
            .send()
            .await
            .map_err(|e| GrassError::Integration(e.to_string()))?;
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| GrassError::Integration(e.to_string()))?;
        Ok(bytes.to_vec())
    }

    async fn pin(&self, cid: &str) -> Result<()> {
        self.http
            .post(format!("{}/api/v0/pin/add?arg={}", self.api_url, cid))
            .send()
            .await
            .map_err(|e| GrassError::Integration(e.to_string()))?;
        Ok(())
    }

    async fn unpin(&self, cid: &str) -> Result<()> {
        self.http
            .post(format!("{}/api/v0/pin/rm?arg={}", self.api_url, cid))
            .send()
            .await
            .map_err(|e| GrassError::Integration(e.to_string()))?;
        Ok(())
    }

    async fn ls(&self, cid: &str) -> Result<Vec<GrassIpfsEntry>> {
        let resp: serde_json::Value = self
            .http
            .post(format!("{}/api/v0/ls?arg={}", self.api_url, cid))
            .send()
            .await
            .map_err(|e| GrassError::Integration(e.to_string()))?
            .json()
            .await
            .map_err(|e| GrassError::Integration(e.to_string()))?;
        let entries = resp["Objects"][0]["Links"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|l| GrassIpfsEntry {
                        name: l["Name"].as_str().unwrap_or("").to_string(),
                        cid: l["Hash"].as_str().unwrap_or("").to_string(),
                        size: l["Size"].as_u64().unwrap_or(0),
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipfs_store() {
        let store = GrassIpfsStore::new("http://localhost:5001");
        assert_eq!(store.api_url, "http://localhost:5001");
    }
}
