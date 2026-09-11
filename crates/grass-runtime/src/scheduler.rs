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

use grass_core::{GrassTaskPayload, Result};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum GrassPriority {
    Low,
    Normal,
    High,
    Critical,
}

pub struct GrassScheduledTask {
    pub task: GrassTaskPayload,
    pub priority: GrassPriority,
}

impl GrassScheduledTask {
    pub fn new(task: GrassTaskPayload, priority: GrassPriority) -> Self {
        Self { task, priority }
    }
}

pub struct GrassTaskScheduler {
    queue: Arc<Mutex<VecDeque<GrassScheduledTask>>>,
    max_concurrent: usize,
    active: Arc<Mutex<usize>>,
}

impl GrassTaskScheduler {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            max_concurrent,
            active: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn enqueue(&self, task: GrassScheduledTask) -> Result<()> {
        let mut queue = self.queue.lock().await;
        let pos = queue
            .iter()
            .position(|t| t.priority < task.priority)
            .unwrap_or(queue.len());
        queue.insert(pos, task);
        Ok(())
    }

    pub async fn dequeue(&self) -> Option<GrassScheduledTask> {
        let active = *self.active.lock().await;
        if active >= self.max_concurrent {
            return None;
        }
        let mut queue = self.queue.lock().await;
        let task = queue.pop_front();
        if task.is_some() {
            let mut active = self.active.lock().await;
            *active += 1;
        }
        task
    }

    pub async fn complete_task(&self) {
        let mut active = self.active.lock().await;
        if *active > 0 {
            *active -= 1;
        }
    }

    pub async fn queue_len(&self) -> usize {
        self.queue.lock().await.len()
    }

    pub async fn active_count(&self) -> usize {
        *self.active.lock().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task_payload() -> GrassTaskPayload {
        GrassTaskPayload {
            task_id: uuid::Uuid::new_v4(),
            agent_id: uuid::Uuid::new_v4(),
            action: "test".into(),
            input: serde_json::json!({}),
            metadata: std::collections::HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_scheduler_enqueue_dequeue_priority() {
        let sched = GrassTaskScheduler::new(2);
        let t1 = GrassScheduledTask::new(make_task_payload(), GrassPriority::Low);
        let t2 = GrassScheduledTask::new(make_task_payload(), GrassPriority::Critical);

        sched.enqueue(t1).await.unwrap();
        sched.enqueue(t2).await.unwrap();

        let first = sched.dequeue().await.unwrap();
        assert_eq!(first.priority, GrassPriority::Critical);
    }

    #[tokio::test]
    async fn test_scheduler_fifo_within_same_priority() {
        let sched = GrassTaskScheduler::new(2);
        let a = GrassScheduledTask::new(make_task_payload(), GrassPriority::Normal);
        let b = GrassScheduledTask::new(make_task_payload(), GrassPriority::Normal);
        let a_id = a.task.task_id;

        sched.enqueue(a).await.unwrap();
        sched.enqueue(b).await.unwrap();

        let first = sched.dequeue().await.unwrap();
        assert_eq!(
            first.task.task_id, a_id,
            "same-priority tasks must stay FIFO"
        );
    }

    #[tokio::test]
    async fn test_scheduler_dequeue_empty_queue_returns_none() {
        let sched = GrassTaskScheduler::new(1);
        assert!(sched.dequeue().await.is_none());
        assert_eq!(sched.queue_len().await, 0);
    }

    #[tokio::test]
    async fn test_scheduler_respects_max_concurrent() {
        let sched = GrassTaskScheduler::new(1);
        sched
            .enqueue(GrassScheduledTask::new(
                make_task_payload(),
                GrassPriority::High,
            ))
            .await
            .unwrap();
        sched
            .enqueue(GrassScheduledTask::new(
                make_task_payload(),
                GrassPriority::Normal,
            ))
            .await
            .unwrap();
        sched
            .enqueue(GrassScheduledTask::new(
                make_task_payload(),
                GrassPriority::Low,
            ))
            .await
            .unwrap();

        let t1 = sched.dequeue().await.unwrap();
        assert_eq!(t1.priority, GrassPriority::High);
        assert_eq!(sched.active_count().await, 1);
        assert_eq!(sched.queue_len().await, 2);

        assert!(
            sched.dequeue().await.is_none(),
            "concurrency limit must block further dispatch"
        );
        assert_eq!(sched.active_count().await, 1);

        sched.complete_task().await;
        assert_eq!(sched.active_count().await, 0);

        let t2 = sched.dequeue().await.unwrap();
        assert_eq!(t2.priority, GrassPriority::Normal);
        sched.complete_task().await;
    }

    #[tokio::test]
    async fn test_scheduler_ordering_across_many_priorities() {
        let sched = GrassTaskScheduler::new(8);
        let mut order = vec![
            GrassPriority::Low,
            GrassPriority::Critical,
            GrassPriority::Normal,
            GrassPriority::High,
            GrassPriority::Critical,
        ];
        for p in order.clone() {
            sched
                .enqueue(GrassScheduledTask::new(make_task_payload(), p))
                .await
                .unwrap();
        }
        order.sort();
        order.reverse();
        for expected in order {
            let t = sched.dequeue().await.expect("scheduled task");
            assert_eq!(t.priority, expected);
            sched.complete_task().await;
        }
    }
}
