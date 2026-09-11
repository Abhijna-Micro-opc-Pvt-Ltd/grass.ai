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

#[async_trait]
pub trait GrassKafkaProducer: Send + Sync {
    async fn produce(&self, topic: &str, key: Option<&str>, value: &[u8]) -> Result<()>;
    async fn flush(&self) -> Result<()>;
}

#[async_trait]
pub trait GrassKafkaConsumer: Send + Sync {
    async fn subscribe(&self, topics: &[&str]) -> Result<()>;
    async fn poll(&self, timeout_ms: u64) -> Result<Option<GrassKafkaMessage>>;
    async fn commit(&self) -> Result<()>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GrassKafkaMessage {
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
    pub key: Option<Vec<u8>>,
    pub value: Vec<u8>,
}

pub struct GrassKafkaProducerImpl {
    brokers: String,
}

impl GrassKafkaProducerImpl {
    pub fn new(brokers: &str) -> Self {
        Self {
            brokers: brokers.to_string(),
        }
    }
}

#[async_trait]
impl GrassKafkaProducer for GrassKafkaProducerImpl {
    async fn produce(&self, topic: &str, _key: Option<&str>, value: &[u8]) -> Result<()> {
        tracing::debug!("Kafka produce to {} ({} bytes)", topic, value.len());
        Ok(())
    }

    async fn flush(&self) -> Result<()> {
        Ok(())
    }
}

pub struct GrassKafkaConsumerImpl {
    brokers: String,
    group_id: String,
}

impl GrassKafkaConsumerImpl {
    pub fn new(brokers: &str, group_id: &str) -> Self {
        Self {
            brokers: brokers.to_string(),
            group_id: group_id.to_string(),
        }
    }
}

#[async_trait]
impl GrassKafkaConsumer for GrassKafkaConsumerImpl {
    async fn subscribe(&self, topics: &[&str]) -> Result<()> {
        tracing::info!(
            "Kafka consumer {} subscribed to {:?}",
            self.group_id,
            topics
        );
        Ok(())
    }

    async fn poll(&self, _timeout_ms: u64) -> Result<Option<GrassKafkaMessage>> {
        Ok(None)
    }

    async fn commit(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_kafka_producer() {
        let producer = GrassKafkaProducerImpl::new("localhost:9092");
        producer
            .produce("test", Some("key"), b"hello")
            .await
            .unwrap();
        producer.flush().await.unwrap();
    }

    #[tokio::test]
    async fn test_kafka_consumer() {
        let consumer = GrassKafkaConsumerImpl::new("localhost:9092", "test-group");
        consumer.subscribe(&["test"]).await.unwrap();
        let msg = consumer.poll(100).await.unwrap();
        assert!(msg.is_none());
    }
}
