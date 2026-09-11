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

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GrassError {
    #[error("agent error: {0}")]
    Agent(String),

    #[error("task error: {0}")]
    Task(String),

    #[error("plugin error: {0}")]
    Plugin(String),

    #[error("hook error: {0}")]
    Hook(String),

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("ipc error: {0}")]
    Ipc(String),

    #[error("sandbox error: {0}")]
    Sandbox(String),

    #[error("llm error: {0}")]
    Llm(String),

    #[error("inference error: {0}")]
    Inference(String),

    #[error("blockchain error: {0}")]
    Blockchain(String),

    #[error("iot error: {0}")]
    Iot(String),

    #[error("streaming error: {0}")]
    Streaming(String),

    #[error("vector db error: {0}")]
    VectorDb(String),

    #[error("integration error: {0}")]
    Integration(String),

    #[error("flow error: {0}")]
    Flow(String),

    #[error("messaging error: {0}")]
    Messaging(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("timeout after {0}ms")]
    Timeout(u64),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("{0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, GrassError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grass_error_display() {
        let err = GrassError::Agent("test failure".into());
        assert!(err.to_string().contains("test failure"));
    }

    #[test]
    fn test_grass_error_io_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing file");
        let grass_err: GrassError = io_err.into();
        assert!(matches!(grass_err, GrassError::Io(_)));
    }
}
