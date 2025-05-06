//! Test suite for the OpenRouter Blueprint
//!
//! This module contains tests for the core functionality of the OpenRouter Blueprint.

use std::sync::Arc;

use crate::llm::{
    ChatCompletionRequest, ChatCompletionResponse, ChatMessage, EmbeddingRequest, EmbeddingResponse,
    LlmCapabilities, LlmClient, LlmError, ModelInfo, NodeMetrics, Result, StreamingLlmClient,
    TextCompletionRequest, TextCompletionResponse,
};
use crate::load_balancer::{LoadBalancer, LoadBalancerConfig, LoadBalancingStrategy};
use crate::config::BlueprintConfig;

mod config_tests;
mod load_balancer_tests;
mod llm_tests;

/// A mock LLM client for testing
pub struct MockLlmClient {
    pub models: Vec<ModelInfo>,
    pub capabilities: LlmCapabilities,
    pub metrics: NodeMetrics,
    pub should_fail: bool,
}

impl MockLlmClient {
    pub fn new() -> Self {
        Self {
            models: vec![
                ModelInfo {
                    id: "test-model".to_string(),
                    name: "Test Model".to_string(),
                    max_context_length: 4096,
                    supports_chat: true,
                    supports_text: true,
                    supports_embeddings: true,
                    parameters: Default::default(),
                },
            ],
            capabilities: LlmCapabilities {
                supports_streaming: true,
                max_concurrent_requests: 10,
                supports_batching: false,
                features: Default::default(),
            },
            metrics: NodeMetrics {
                cpu_utilization: 0.5,
                memory_utilization: 0.3,
                gpu_utilization: Some(0.7),
                requests_per_minute: 100,
                average_response_time_ms: 200,
                active_requests: 5,
                last_updated: 0,
            },
            should_fail: false,
        }
    }
    
    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

#[async_trait::async_trait]
impl LlmClient for MockLlmClient {
    fn get_supported_models(&self) -> Vec<ModelInfo> {
        self.models.clone()
    }
    
    fn get_capabilities(&self) -> LlmCapabilities {
        self.capabilities.clone()
    }
    
    fn get_metrics(&self) -> NodeMetrics {
        self.metrics.clone()
    }
    
    async fn chat_completion(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        if self.should_fail {
            return Err(LlmError::RequestFailed("Mock failure".to_string()));
        }
        
        Ok(ChatCompletionResponse {
            id: "mock-id".to_string(),
            object: "chat.completion".to_string(),
            created: 0,
            model: request.model,
            choices: vec![],
            usage: None,
        })
    }
    
    async fn text_completion(&self, request: TextCompletionRequest) -> Result<TextCompletionResponse> {
        if self.should_fail {
            return Err(LlmError::RequestFailed("Mock failure".to_string()));
        }
        
        Ok(TextCompletionResponse {
            id: "mock-id".to_string(),
            object: "text_completion".to_string(),
            created: 0,
            model: request.model,
            choices: vec![],
            usage: None,
        })
    }
    
    async fn embeddings(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse> {
        if self.should_fail {
            return Err(LlmError::RequestFailed("Mock failure".to_string()));
        }
        
        Ok(EmbeddingResponse {
            object: "list".to_string(),
            model: request.model,
            data: vec![],
            usage: None,
        })
    }
}

/// A mock streaming LLM client for testing
pub struct MockStreamingLlmClient {
    pub base: MockLlmClient,
}

impl MockStreamingLlmClient {
    pub fn new() -> Self {
        Self {
            base: MockLlmClient::new(),
        }
    }
    
    pub fn with_failure(mut self) -> Self {
        self.base.should_fail = true;
        self
    }
}

#[async_trait::async_trait]
impl LlmClient for MockStreamingLlmClient {
    fn get_supported_models(&self) -> Vec<ModelInfo> {
        self.base.get_supported_models()
    }
    
    fn get_capabilities(&self) -> LlmCapabilities {
        self.base.get_capabilities()
    }
    
    fn get_metrics(&self) -> NodeMetrics {
        self.base.get_metrics()
    }
    
    async fn chat_completion(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        self.base.chat_completion(request).await
    }
    
    async fn text_completion(&self, request: TextCompletionRequest) -> Result<TextCompletionResponse> {
        self.base.text_completion(request).await
    }
    
    async fn embeddings(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse> {
        self.base.embeddings(request).await
    }
}

#[async_trait::async_trait]
impl StreamingLlmClient for MockStreamingLlmClient {
    async fn streaming_chat_completion(&self, request: ChatCompletionRequest) -> Result<crate::llm::ChatCompletionStream> {
        if self.base.should_fail {
            return Err(LlmError::RequestFailed("Mock failure".to_string()));
        }
        
        // Create an empty stream
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let _ = tx.send(Ok(crate::llm::ChatCompletionChunk {
            id: "mock-id".to_string(),
            object: "chat.completion.chunk".to_string(),
            created: 0,
            model: request.model,
            choices: vec![],
        })).await;
        
        Ok(crate::llm::create_chat_completion_stream(rx))
    }
    
    async fn streaming_text_completion(&self, request: TextCompletionRequest) -> Result<crate::llm::TextCompletionStream> {
        if self.base.should_fail {
            return Err(LlmError::RequestFailed("Mock failure".to_string()));
        }
        
        // Create an empty stream
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let _ = tx.send(Ok(crate::llm::TextCompletionChunk {
            id: "mock-id".to_string(),
            object: "text_completion.chunk".to_string(),
            created: 0,
            model: request.model,
            choices: vec![],
        })).await;
        
        Ok(crate::llm::create_text_completion_stream(rx))
    }
}

/// Create a test load balancer with mock LLM clients
pub fn create_test_load_balancer() -> LoadBalancer {
    let config = LoadBalancerConfig {
        strategy: LoadBalancingStrategy::RoundRobin,
        max_retries: 3,
        selection_timeout_ms: 1000,
    };
    
    LoadBalancer::new(config)
}

/// Add mock clients to a load balancer
pub async fn add_mock_clients(load_balancer: &LoadBalancer, count: usize) {
    for i in 0..count {
        let client = Arc::new(MockLlmClient::new());
        load_balancer.add_node(format!("mock-{}", i), client).await;
    }
}

/// Create a test chat completion request
pub fn create_test_chat_request() -> ChatCompletionRequest {
    ChatCompletionRequest {
        model: "test-model".to_string(),
        messages: vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Hello, world!".to_string(),
                name: None,
            },
        ],
        temperature: Some(0.7),
        top_p: Some(1.0),
        max_tokens: Some(100),
        stream: Some(false),
        additional_params: Default::default(),
    }
}

/// Create a test text completion request
pub fn create_test_text_request() -> TextCompletionRequest {
    TextCompletionRequest {
        model: "test-model".to_string(),
        prompt: "Hello, world!".to_string(),
        temperature: Some(0.7),
        top_p: Some(1.0),
        max_tokens: Some(100),
        stream: Some(false),
        additional_params: Default::default(),
    }
}

/// Create a test embedding request
pub fn create_test_embedding_request() -> EmbeddingRequest {
    EmbeddingRequest {
        model: "test-model".to_string(),
        input: vec!["Hello, world!".to_string()],
        additional_params: Default::default(),
    }
}
