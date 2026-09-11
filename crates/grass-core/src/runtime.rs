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
use std::future::Future;
use std::pin::Pin;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = crate::Result<T>> + Send + 'a>>;

#[async_trait]
pub trait GrassRuntime: Send + Sync {
    /// Spawns a named async task on the runtime.
    async fn spawn<F>(&self, name: &str, future: F) -> crate::Result<()>
    where
        F: Future<Output = ()> + Send + 'static;

    /// Spawns a blocking task on the runtime.
    async fn spawn_blocking<F>(&self, name: &str, f: F) -> crate::Result<()>
    where
        F: FnOnce() + Send + 'static;

    /// Shuts down the runtime gracefully.
    async fn shutdown(&self) -> crate::Result<()>;
}

pub struct GrassTokioRuntime {
    handle: tokio::runtime::Handle,
}

impl GrassTokioRuntime {
    pub fn new() -> Self {
        Self {
            handle: tokio::runtime::Handle::current(),
        }
    }
}

impl Default for GrassTokioRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GrassRuntime for GrassTokioRuntime {
    async fn spawn<F>(&self, _name: &str, future: F) -> crate::Result<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.handle.spawn(future);
        Ok(())
    }

    async fn spawn_blocking<F>(&self, _name: &str, f: F) -> crate::Result<()>
    where
        F: FnOnce() + Send + 'static,
    {
        tokio::task::spawn_blocking(f)
            .await
            .map_err(|e| crate::GrassError::Internal(format!("spawn_blocking: {}", e)))?;
        Ok(())
    }

    async fn shutdown(&self) -> crate::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tokio_runtime_spawn() {
        let rt = GrassTokioRuntime::new();
        let (tx, rx) = tokio::sync::oneshot::channel();
        rt.spawn("test-task", async move {
            let _ = tx.send(42);
        })
        .await
        .unwrap();
        let val = rx.await.unwrap();
        assert_eq!(val, 42);
    }
}
