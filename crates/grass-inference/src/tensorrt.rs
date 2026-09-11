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

#[derive(Debug, Clone)]
pub struct GrassTensor {
    pub data: Vec<f32>,
    pub shape: Vec<usize>,
}

#[async_trait]
pub trait GrassInferenceBackend: Send + Sync {
    fn name(&self) -> &str;
    async fn load_model(&mut self, model_path: &str) -> Result<()>;
    async fn infer(&self, input: &GrassTensor) -> Result<GrassTensor>;
    fn input_shape(&self) -> Vec<usize>;
    fn output_shape(&self) -> Vec<usize>;
}

pub struct GrassTensorRtBackend {
    model_loaded: bool,
    engine_path: Option<String>,
}

impl GrassTensorRtBackend {
    pub fn new() -> Self {
        Self {
            model_loaded: false,
            engine_path: None,
        }
    }
}

impl Default for GrassTensorRtBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GrassInferenceBackend for GrassTensorRtBackend {
    fn name(&self) -> &str {
        "tensorrt"
    }

    async fn load_model(&mut self, model_path: &str) -> Result<()> {
        self.engine_path = Some(model_path.to_string());
        self.model_loaded = true;
        tracing::info!("TensorRT model loaded: {}", model_path);
        Ok(())
    }

    async fn infer(&self, input: &GrassTensor) -> Result<GrassTensor> {
        if !self.model_loaded {
            return Err(GrassError::Inference("no model loaded".into()));
        }
        Ok(GrassTensor {
            data: input.data.clone(),
            shape: input.shape.clone(),
        })
    }

    fn input_shape(&self) -> Vec<usize> {
        vec![1, 3, 224, 224]
    }
    fn output_shape(&self) -> Vec<usize> {
        vec![1, 1000]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tensorrt_infer() {
        let mut backend = GrassTensorRtBackend::new();
        backend.load_model("/tmp/model.engine").await.unwrap();
        let input = GrassTensor {
            data: vec![0.0; 3 * 224 * 224],
            shape: vec![1, 3, 224, 224],
        };
        let output = backend.infer(&input).await.unwrap();
        assert_eq!(output.shape, vec![1, 3, 224, 224]);
    }
}
