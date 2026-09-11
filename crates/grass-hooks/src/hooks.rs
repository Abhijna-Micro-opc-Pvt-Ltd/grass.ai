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
use grass_core::{HookId, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum GrassHookTrigger {
    OnAgentStart,
    OnAgentStop,
    OnTaskComplete,
    OnTaskFailed,
    OnMessage,
    OnCustom(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrassHookPayload {
    pub trigger: GrassHookTrigger,
    pub data: serde_json::Value,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
pub trait GrassHook: Send + Sync {
    fn id(&self) -> HookId;
    fn name(&self) -> &str;
    fn trigger(&self) -> GrassHookTrigger;
    async fn execute(&self, payload: &GrassHookPayload) -> Result<serde_json::Value>;
}

pub struct GrassHookRegistry {
    hooks: Arc<RwLock<HashMap<HookId, Arc<dyn GrassHook>>>>,
    trigger_map: Arc<RwLock<HashMap<String, Vec<HookId>>>>,
}

impl GrassHookRegistry {
    pub fn new() -> Self {
        Self {
            hooks: Arc::new(RwLock::new(HashMap::new())),
            trigger_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, hook: Arc<dyn GrassHook>) {
        let id = hook.id();
        let trigger_key = format!("{:?}", hook.trigger());
        let mut hooks = self.hooks.write().await;
        hooks.insert(id, hook);
        let mut trigger_map = self.trigger_map.write().await;
        trigger_map.entry(trigger_key).or_default().push(id);
    }

    pub async fn unregister(&self, id: HookId) {
        let mut hooks = self.hooks.write().await;
        hooks.remove(&id);
    }

    pub async fn fire(
        &self,
        trigger: &GrassHookTrigger,
        payload: &GrassHookPayload,
    ) -> Result<Vec<serde_json::Value>> {
        let trigger_key = format!("{:?}", trigger);
        let trigger_map = self.trigger_map.read().await;
        let hook_ids = trigger_map.get(&trigger_key).cloned().unwrap_or_default();
        drop(trigger_map);

        let hooks = self.hooks.read().await;
        let mut results = vec![];
        for hook_id in hook_ids {
            if let Some(hook) = hooks.get(&hook_id) {
                match hook.execute(payload).await {
                    Ok(result) => results.push(result),
                    Err(e) => tracing::error!("hook {} failed: {}", hook.name(), e),
                }
            }
        }
        Ok(results)
    }

    pub async fn list_hooks(&self) -> Vec<HookId> {
        self.hooks.read().await.keys().copied().collect()
    }
}

impl Default for GrassHookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHook {
        id: HookId,
    }

    #[async_trait]
    impl GrassHook for TestHook {
        fn id(&self) -> HookId {
            self.id
        }
        fn name(&self) -> &str {
            "test-hook"
        }
        fn trigger(&self) -> GrassHookTrigger {
            GrassHookTrigger::OnCustom("test".into())
        }
        async fn execute(&self, _payload: &GrassHookPayload) -> Result<serde_json::Value> {
            Ok(serde_json::json!({"ok": true}))
        }
    }

    #[tokio::test]
    async fn test_hook_registry_register_and_fire() {
        let reg = GrassHookRegistry::new();
        let hook: Arc<dyn GrassHook> = Arc::new(TestHook {
            id: uuid::Uuid::new_v4(),
        });
        reg.register(hook).await;

        let payload = GrassHookPayload {
            trigger: GrassHookTrigger::OnCustom("test".into()),
            data: serde_json::json!({}),
            metadata: HashMap::new(),
        };
        let results = reg.fire(&payload.trigger, &payload).await.unwrap();
        assert_eq!(results.len(), 1);
    }
}
