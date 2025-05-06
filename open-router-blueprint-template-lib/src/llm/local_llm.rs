use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::{
    ChatCompletionRequest, ChatCompletionResponse, EmbeddingRequest, EmbeddingResponse,
    LlmCapabilities, LlmClient, LlmError, ModelInfo, NodeMetrics, Result, TextCompletionRequest,
    TextCompletionResponse,
};

/// Configuration for a local LLM client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalLlmConfig {
    /// The base URL for the LLM API
    pub api_url: String,

    /// The timeout for API requests in seconds
    pub timeout_seconds: u64,

    /// The maximum number of concurrent requests
    pub max_concurrent_requests: usize,

    /// The models available on this LLM instance
    pub models: Vec<ModelInfo>,

    /// Additional configuration parameters
    pub additional_params: HashMap<String, String>,
}

impl Default for LocalLlmConfig {
    fn default() -> Self {
        Self {
            api_url: String::new(),
            timeout_seconds: 60,
            max_concurrent_requests: 1,
            models: Vec::new(),
            additional_params: HashMap::new(),
        }
    }
}

/// Generic local LLM client implementation for the OpenRouter Blueprint template.
///
/// This struct provides the structure and extension points for interacting with any local LLM.
/// To implement a specific LLM, derive from this template and override the LLM call logic.
pub struct LocalLlmClient {
    pub config: LocalLlmConfig,
    pub metrics: Arc<RwLock<NodeMetrics>>,
}

impl LocalLlmClient {
    /// Create a new local LLM client with the given configuration
    pub fn new(config: LocalLlmConfig) -> Self {
        let metrics = Arc::new(RwLock::new(NodeMetrics {
            cpu_utilization: 0.0,
            memory_utilization: 0.0,
            gpu_utilization: None,
            requests_per_minute: 0,
            average_response_time_ms: 0,
            active_requests: 0,
            last_updated: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }));

        Self { config, metrics }
    }

    /// Update the metrics for this client
    pub async fn update_metrics(&self, cpu: f32, memory: f32, gpu: Option<f32>) {
        let mut metrics = self.metrics.write().await;
        metrics.cpu_utilization = cpu;
        metrics.memory_utilization = memory;
        metrics.gpu_utilization = gpu;
        metrics.last_updated = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    // async fn record_request_start(&self) {
    //     let mut metrics = self.metrics.write().await;
    //     metrics.active_requests += 1;
    // }

    // async fn record_request_end(&self, duration_ms: u64) {
    //     let mut metrics = self.metrics.write().await;
    //     metrics.active_requests = metrics.active_requests.saturating_sub(1);

    //     // Update average response time with exponential moving average
    //     const ALPHA: f64 = 0.1; // Weight for new samples
    //     let old_avg = metrics.average_response_time_ms as f64;
    //     let new_avg = old_avg * (1.0 - ALPHA) + (duration_ms as f64) * ALPHA;
    //     metrics.average_response_time_ms = new_avg as u64;

    //     // Increment requests per minute (this is simplified and should be improved)
    //     metrics.requests_per_minute += 1;
    // }
}

#[async_trait]
impl LlmClient for LocalLlmClient {
    fn get_supported_models(&self) -> Vec<ModelInfo> {
        self.config.models.clone()
    }

    fn get_capabilities(&self) -> LlmCapabilities {
        LlmCapabilities {
            supports_streaming: false, // Template default; override in concrete implementation if needed
            max_concurrent_requests: self.config.max_concurrent_requests,
            supports_batching: false, // Template default; override in concrete implementation if needed
            features: HashMap::new(),
        }
    }

    fn get_metrics(&self) -> NodeMetrics {
        futures::executor::block_on(async { self.metrics.read().await.clone() })
    }

    /// Template method for chat completion. To use, override this method in your concrete blueprint.
    async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        if !self.config.models.iter().any(|m| m.id == request.model) {
            return Err(LlmError::ModelNotSupported(request.model));
        }
        // This is a template method. Implement your LLM call logic in your derived blueprint.
        Err(LlmError::NotImplemented(
            "chat_completion must be implemented in your blueprint (see LocalLlmClient in template)".to_string(),
        ))
    }

    /// Template method for text completion. To use, override this method in your concrete blueprint.
    async fn text_completion(
        &self,
        request: TextCompletionRequest,
    ) -> Result<TextCompletionResponse> {
        if !self.config.models.iter().any(|m| m.id == request.model) {
            return Err(LlmError::ModelNotSupported(request.model));
        }
        Err(LlmError::NotImplemented(
            "text_completion must be implemented in your blueprint (see LocalLlmClient in template)".to_string(),
        ))
    }

    /// Template method for embeddings. To use, override this method in your concrete blueprint.
    async fn embeddings(
        &self,
        request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse> {
        if !self.config.models.iter().any(|m| m.id == request.model) {
            return Err(LlmError::ModelNotSupported(request.model));
        }
        Err(LlmError::NotImplemented(
            "embeddings must be implemented in your blueprint (see LocalLlmClient in template)".to_string(),
        ))
    }
}
