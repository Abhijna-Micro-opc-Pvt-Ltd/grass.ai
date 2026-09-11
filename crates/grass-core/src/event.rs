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

use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum GrassEvent {
    AgentStarted {
        agent_id: crate::AgentId,
    },
    AgentStopped {
        agent_id: crate::AgentId,
    },
    AgentFailed {
        agent_id: crate::AgentId,
        error: String,
    },
    TaskCreated {
        task_id: crate::TaskId,
    },
    TaskStarted {
        task_id: crate::TaskId,
    },
    TaskCompleted {
        task_id: crate::TaskId,
    },
    TaskFailed {
        task_id: crate::TaskId,
        error: String,
    },
    HookFired {
        hook_id: crate::HookId,
        payload: serde_json::Value,
    },
    PluginLoaded {
        plugin_id: crate::PluginId,
        name: String,
    },
    PluginUnloaded {
        plugin_id: crate::PluginId,
    },
    MessageSent {
        message_id: crate::MessageId,
        from: crate::AgentId,
        topic: String,
    },
    SandboxCreated {
        backend: String,
        container_id: String,
    },
    SandboxDestroyed {
        container_id: String,
    },
    Custom {
        event_type: String,
        data: HashMap<String, serde_json::Value>,
    },
}

pub struct GrassEventBus {
    subscribers: Vec<Box<dyn Fn(&GrassEvent) + Send + Sync>>,
}

impl GrassEventBus {
    pub fn new() -> Self {
        Self {
            subscribers: vec![],
        }
    }

    pub fn subscribe<F: Fn(&GrassEvent) + Send + Sync + 'static>(&mut self, handler: F) {
        self.subscribers.push(Box::new(handler));
    }

    pub fn publish(&self, event: &GrassEvent) {
        for sub in &self.subscribers {
            sub(event);
        }
    }
}

impl Default for GrassEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[test]
    fn test_event_bus_subscribe_and_publish() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let mut bus = GrassEventBus::new();
        bus.subscribe(move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        bus.publish(&GrassEvent::AgentStarted {
            agent_id: uuid::Uuid::new_v4(),
        });

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
